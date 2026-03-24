use anyhow::Context;
use client::{
    context::market::MarketContext,
    fmt_kv,
    transactions::CustomRpcClient,
};
use colored::Colorize;
use dropset_interface::{
    instructions::{
        BatchReplaceInstructionData,
        UnvalidatedOrders,
    },
    state::{
        sector::SectorIndex,
        user_order_sectors::MAX_ORDERS_USIZE,
    },
};
use dropset_services_shared::{
    config::ValidSharedConfig,
    oanda_types::{
        CurrencyPair,
        OandaCandlestickResponse,
    },
};
use itertools::Itertools;
use price::{
    client_helpers::{
        to_order_info_args,
        try_encoded_u32_to_decoded_decimal,
    },
    OrderInfoArgs,
};
use rust_decimal::{
    prelude::ToPrimitive,
    Decimal,
};
use solana_address::Address;
use solana_keypair::Signer;
use solana_sdk::{
    message::Instruction,
    signature::Keypair,
};
use transaction_parser::views::{
    try_market_view_all_from_owner_and_data,
    MarketViewAll,
};

use crate::{
    config::ValidMakerConfig,
    get_non_redundant_order_flow,
    logger::{
        divider,
        Logger,
        BUY_COLOR,
        SELL_COLOR,
    },
    model::calculate_spreads::{
        half_spread,
        reservation_price,
    },
    utils::get_normalized_mid_price,
    MakerState,
};

pub struct MakerContext {
    /// The maker's keypair.
    pub keypair: Keypair,
    pub market_ctx: MarketContext,
    /// The maker's address.
    maker_address: Address,
    /// The currency pair.
    pub pair: CurrencyPair,
    /// The maker's latest state.
    latest_state: MakerState,
    /// The target base amount in the maker's seat, in atoms.
    ///
    /// If the maker starts with 1,000 base atoms and the target base amount is 10,000, `q` will be
    /// equal to -9,000. This will indirectly influence the model to more aggressively place bids
    /// and thus return to a `q` value of zero.
    pub base_target_atoms: u64,
    /// The reference mid price, expressed as quote atom per 1 base atom.
    ///
    /// In the A–S model this is an exogenous “fair price” process; in practice you can source it
    /// externally (e.g. FX feed) or derive it internally from the venue’s top-of-book.
    /// It anchors the reservation price and thus the bid/ask quotes via the spread model.
    ///
    /// Note that the price as quote_atoms / base_atoms may differ from quote / base. Be sure to
    /// express the price as a ratio of atoms.
    mid_price_atoms: Decimal,

    /// Whether or not to use batch replace instead of individual instructions.
    pub batch_replace: bool,

    /// Whether to render the ASCII order book visualization instead of compact price arrays.
    pub visualize: bool,

    pub logger: Logger,

    /// Set to true when the last submission failed with
    /// [DropsetError::NoFreeSectorsRemaining](dropset_interface::error::DropsetError::NoFreeSectorsRemaining).
    ///
    /// The next call to [`Self::create_update_book_instructions`] will prepend an expand ix.
    pub needs_expand: bool,

    /// The order size in atoms for each order denominated in quote.
    pub bid_order_size: u64,

    /// The order size in atoms for each order denominated in base.
    pub ask_order_size: u64,

    /// The maker's SOL balance in lamports, updated after each successful transaction.
    sol_lamports: u64,
}

impl MakerContext {
    /// Creates a new maker context from a [ValidMakerConfig].
    pub async fn init(rpc: &CustomRpcClient, cfg: ValidMakerConfig) -> anyhow::Result<Self> {
        let ValidMakerConfig {
            shared,
            target_base: base_target_atoms,
            batch_replace,
            bid_order_size,
            ask_order_size,
            visualize,
            oanda_args,
            initial_price_feed_response,
            ..
        } = cfg;

        let ValidSharedConfig {
            keypair,
            base,
            quote,
            ..
        } = shared;

        let market_ctx = MarketContext::new(base, quote);

        let market_account = rpc
            .client
            .get_account(&market_ctx.market)
            .await
            .with_context(|| {
                anyhow::anyhow!(
                    "Couldn't find market account {} on-chain",
                    market_ctx.market
                )
            })?;
        let market =
            try_market_view_all_from_owner_and_data(market_account.owner, &market_account.data)?;
        let latest_state = MakerState::new_from_market(keypair.pubkey(), market)?;
        let mid_price =
            get_normalized_mid_price(initial_price_feed_response, &oanda_args.pair, &market_ctx)?;
        let maker_address = keypair.pubkey();
        // Maker may temporarily show as `0` balance until the balance fetch succeeds.
        let sol_lamports = rpc.client.get_balance(&maker_address).await.unwrap_or(0);

        let mut ctx = Self {
            keypair,
            market_ctx,
            maker_address,
            pair: oanda_args.pair,
            latest_state,
            base_target_atoms,
            mid_price_atoms: mid_price,
            batch_replace,
            bid_order_size,
            ask_order_size,
            visualize,
            needs_expand: false,
            logger: Logger::new(visualize),
            sol_lamports,
        };
        ctx.render_chart();
        Ok(ctx)
    }

    /// See [`MakerContext::mid_price_atoms`].
    pub fn get_mid_price_atoms(&self) -> Decimal {
        self.mid_price_atoms
    }

    /// Helper function for the maker's seat index.
    pub fn seat_index(&self) -> SectorIndex {
        self.latest_state.seat.index
    }

    /// In the A-S model `q` represents the base inventory as a reflection of the maker's net short
    /// (negative) or long (positive) position. The difference from the maker seat's current base
    /// to target base can thus be used as `q` to achieve the effect of always returning to the
    /// target base inventory amount.
    ///
    /// When `q` is negative, the maker is below the desired/target inventory amount, and when `q`
    /// is positive, the maker is above the desired/target inventory amount.
    ///
    /// In practice, this has two opposing effects.
    /// - When q is negative, it pushes the spread upwards so that bid prices are closer to the
    ///   [`crate::model::calculate_spreads::reservation_price`] and ask prices are further away.
    ///   This effectively increases the likelihood of getting bids filled and vice versa for asks.
    /// - When q is positive, it pushes the spread downwards so that ask prices are closer to the
    ///   [`crate::model::calculate_spreads::reservation_price`] price and bid prices are further
    ///   away. This effectively increases the likelihood of getting asks filled and vice versa for
    ///   bids.
    pub fn q(&self) -> Decimal {
        (Decimal::from(self.latest_state.base_inventory) - Decimal::from(self.base_target_atoms))
            / Decimal::from(10u64.pow(self.market_ctx.base.mint_decimals as u32))
    }

    pub fn create_update_book_instructions(&mut self) -> anyhow::Result<Vec<Instruction>> {
        let expand_ix = self.needs_expand.then(|| {
            self.market_ctx
                .expand(self.maker_address, (MAX_ORDERS_USIZE * 2) as u16)
        });

        let (bid_price, ask_price) = self.get_bid_and_ask_prices();
        let step = half_spread();

        // Generate MAX_ORDERS_USIZE layered (price, base_size) pairs per side, stepping outward by
        // half_spread per layer with order sizes scaling linearly (1x, 2x, ... Nx base size).
        let bid_layers: Vec<(Decimal, u64)> = (0..MAX_ORDERS_USIZE as u32)
            .map(|i| {
                let price = bid_price - step * Decimal::from(i);

                let error_msg =
                    || format!("Couldn't convert bid size to u64 at layer {i}: price={price}");
                let size = (Decimal::from(self.bid_order_size) * Decimal::from(i + 1) / price)
                    .to_u64()
                    .with_context(error_msg)?;
                Ok((price, size))
            })
            .collect::<anyhow::Result<_>>()?;

        let ask_layers: Vec<(Decimal, u64)> = (0..MAX_ORDERS_USIZE as u32)
            .map(|i| {
                let price = ask_price + step * Decimal::from(i);
                Ok((price, self.ask_order_size * u64::from(i + 1)))
            })
            .collect::<anyhow::Result<_>>()?;

        let (cancels, posts) = get_non_redundant_order_flow(
            self.latest_state.bids.clone(),
            self.latest_state.asks.clone(),
            bid_layers.clone(),
            ask_layers.clone(),
            self.seat_index(),
        )?;

        if self.batch_replace {
            if cancels.len() + posts.len() == 0 {
                return Ok(expand_ix.into_iter().collect());
            }

            let bid_args: [OrderInfoArgs; MAX_ORDERS_USIZE] = bid_layers
                .into_iter()
                .map(|(price, size)| to_order_info_args(price, size).map_err(anyhow::Error::from))
                .collect::<anyhow::Result<Vec<_>>>()?
                .try_into()
                .expect("exactly MAX_ORDERS_USIZE layers");

            let ask_args: [OrderInfoArgs; MAX_ORDERS_USIZE] = ask_layers
                .into_iter()
                .map(|(price, size)| to_order_info_args(price, size).map_err(anyhow::Error::from))
                .collect::<anyhow::Result<Vec<_>>>()?
                .try_into()
                .expect("exactly MAX_ORDERS_USIZE layers");

            let ixn = self.market_ctx.batch_replace(
                self.maker_address,
                BatchReplaceInstructionData::new(
                    self.seat_index(),
                    UnvalidatedOrders::new(bid_args),
                    UnvalidatedOrders::new(ask_args),
                ),
            );

            Ok(expand_ix.into_iter().chain([ixn]).collect_vec())
        } else {
            let ixns = expand_ix
                .into_iter()
                .chain(
                    cancels
                        .into_iter()
                        .map(|cancel| self.market_ctx.cancel_order(self.maker_address, cancel)),
                )
                .chain(
                    posts
                        .into_iter()
                        .map(|post| self.market_ctx.post_order(self.maker_address, post)),
                )
                .collect_vec();

            Ok(ixns)
        }
    }

    /// Returns an [Instruction] to deposit base to the maker's market seat.
    pub fn deposit_base(&self, amount: u64) -> Instruction {
        self.market_ctx
            .deposit_base(self.maker_address, amount, self.seat_index())
    }

    /// Returns an [Instruction] to deposit quote to the maker's market seat.
    pub fn deposit_quote(&self, amount: u64) -> Instruction {
        self.market_ctx
            .deposit_quote(self.maker_address, amount, self.seat_index())
    }

    pub fn update_maker_state(&mut self, new_market_state: MarketViewAll) -> anyhow::Result<()> {
        self.latest_state = MakerState::new_from_market(self.maker_address, new_market_state)?;
        self.render_chart();
        Ok(())
    }

    /// Renders the maker's current on-chain order book as a depth chart.
    pub fn render_chart(&mut self) {
        if !self.visualize {
            return;
        }

        const BAR_WIDTH: usize = 20;
        const PRICE_WIDTH: usize = 10;

        let decode = |orders: &[_]| -> Vec<(Decimal, u64)> {
            orders
                .iter()
                .filter_map(|o: &transaction_parser::views::OrderView| {
                    let price =
                        try_encoded_u32_to_decoded_decimal(o.encoded_price.as_u32()).ok()?;
                    Some((price, o.base_remaining))
                })
                .collect()
        };

        let mut asks = decode(&self.latest_state.asks);
        let mut bids = decode(&self.latest_state.bids);

        asks.sort_by_key(|(p, _)| std::cmp::Reverse(*p));
        bids.sort_by_key(|(p, _)| std::cmp::Reverse(*p));

        let max_size = asks
            .iter()
            .chain(bids.iter())
            .map(|(_, s)| *s)
            .max()
            .unwrap_or(1);

        let bar = |size: u64| -> String {
            let filled = ((size as f64 / max_size as f64) * BAR_WIDTH as f64).round() as usize;
            "█".repeat(filled.max(1))
        };

        let fmt_row = |p: &Decimal, s: u64| {
            format!(
                "{:>PRICE_WIDTH$}  {:<BAR_WIDTH$}",
                p,
                bar(s),
                PRICE_WIDTH = PRICE_WIDTH,
                BAR_WIDTH = BAR_WIDTH,
            )
        };

        let sol = Decimal::from(self.sol_lamports) / Decimal::from(1_000_000_000u64);
        let base_seat = self.latest_state.seat.base_available;
        let base_orders = self.latest_state.base_inventory.saturating_sub(base_seat);
        let base_total = base_seat + base_orders;
        let quote_seat = self.latest_state.seat.quote_available;
        let quote_orders = self.latest_state.quote_inventory.saturating_sub(quote_seat);
        let quote_total = quote_seat + quote_orders;

        let base_scale = Decimal::from(10u64.pow(self.market_ctx.base.mint_decimals as u32));
        let quote_scale = Decimal::from(10u64.pow(self.market_ctx.quote.mint_decimals as u32));

        // Compute precision for 8 sig figs based on a reference value (the row total),
        // then use that same precision for all three columns so they align.
        let sig8_prec = |reference: u64, scale: Decimal| -> usize {
            let d = Decimal::from(reference) / scale;
            d.to_f64()
                .filter(|f| f.is_normal())
                .map(|f| {
                    let mag = f.log10().floor() as i32;
                    if mag >= 7 {
                        0
                    } else {
                        (8 - mag - 1) as usize
                    }
                })
                .unwrap_or(2)
        };

        // Drive precision from the smallest value in each row so that all three columns
        // (seat, orders, total) share enough decimal places to show the least-significant
        // difference. Using the total (the largest value) would truncate the smaller columns
        // and hide sub-unit discrepancies from order-size truncation.
        let base_prec = sig8_prec(
            base_seat.min(base_orders).min(base_total).max(1),
            base_scale,
        );
        let quote_prec = sig8_prec(
            quote_seat.min(quote_orders).min(quote_total).max(1),
            quote_scale,
        );

        let fmt_base = |atoms: u64| format!("{:.base_prec$}", Decimal::from(atoms) / base_scale);
        let fmt_quote = |atoms: u64| format!("{:.quote_prec$}", Decimal::from(atoms) / quote_scale);

        let bs = fmt_base(base_seat);
        let bo = fmt_base(base_orders);
        let bt = fmt_base(base_total);
        let qs = fmt_quote(quote_seat);
        let qo = fmt_quote(quote_orders);
        let qt = fmt_quote(quote_total);

        let col_w = [&bs, &bo, &bt, &qs, &qo, &qt]
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(1);

        // Total portfolio value in quote tokens.
        let total_value =
            Decimal::from(base_total) * self.mid_price_atoms + Decimal::from(quote_total);
        let total_value_display = total_value / quote_scale;
        let value_precision = total_value_display
            .to_f64()
            .filter(|f| f.is_normal())
            .map(|f| {
                let mag = f.log10().floor() as i32;
                if mag >= 7 {
                    0
                } else {
                    (8 - mag - 1) as usize
                }
            })
            .unwrap_or(2);

        let mut lines = Vec::with_capacity(4 + MAX_ORDERS_USIZE * 2 + 1);

        let base_msg = format!("{bs:>col_w$} seat  |  {bo:>col_w$} orders  |  {bt:>col_w$} total");
        let quote_msg = format!("{qs:>col_w$} seat  |  {qo:>col_w$} orders  |  {qt:>col_w$} total");
        let value_msg = format!("{total_value_display:.value_precision$} quote");
        lines.push(fmt_kv!("SOL  ", format!("{sol:.6}")));
        lines.push(fmt_kv!("Base ", base_msg));
        lines.push(fmt_kv!("Quote", quote_msg));
        lines.push(fmt_kv!("Value", value_msg));
        lines.push(divider());

        // Only render the order book once there are orders to show.
        if !asks.is_empty() || !bids.is_empty() {
            // Asks: pad empty lines at top so best ask stays closest to center.
            for _ in 0..MAX_ORDERS_USIZE.saturating_sub(asks.len()) {
                lines.push(String::new());
            }
            for (p, s) in &asks {
                lines.push(format!("{}", fmt_row(p, *s).as_str().color(SELL_COLOR)));
            }

            lines.push(divider());

            // Bids: pad empty lines at bottom so best bid stays closest to center.
            for (p, s) in &bids {
                lines.push(format!("{}", fmt_row(p, *s).as_str().color(BUY_COLOR)));
            }
            for _ in 0..MAX_ORDERS_USIZE.saturating_sub(bids.len()) {
                lines.push(String::new());
            }
        }

        self.logger.update_chart(lines);
    }

    pub fn update_sol_balance(&mut self, lamports: u64) {
        self.sol_lamports = lamports;
    }

    pub fn update_price_from_candlestick(
        &mut self,
        candlestick_response: OandaCandlestickResponse,
    ) -> anyhow::Result<()> {
        self.mid_price_atoms =
            get_normalized_mid_price(candlestick_response, &self.pair, &self.market_ctx)?;

        Ok(())
    }

    /// Calculates the model's output bid and ask prices based on the market's current mid price
    /// and the maker's current state.
    ///
    /// Note that these prices are already normalized to being in atoms.
    fn get_bid_and_ask_prices(&self) -> (Decimal, Decimal) {
        let reservation_price = reservation_price(self.get_mid_price_atoms(), self.q());
        let bid_price = reservation_price - half_spread();
        let ask_price = reservation_price + half_spread();

        (bid_price, ask_price)
    }
}
