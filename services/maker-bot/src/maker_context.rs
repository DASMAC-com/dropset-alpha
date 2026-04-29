use std::{
    collections::{HashSet, VecDeque},
    time::{Duration, Instant},
};

use anyhow::Context;
use client::{context::market::MarketContext, fmt_kv, transactions::CustomRpcClient};
use colored::Colorize;
use dropset_interface::{
    instructions::{BatchReplaceInstructionData, UnvalidatedOrders},
    state::{sector::SectorIndex, user_order_sectors::MAX_ORDERS_USIZE},
};
use dropset_services_shared::{
    config::ValidSharedConfig,
    oanda_types::{CurrencyPair, OandaCandlestickResponse},
};
use itertools::Itertools;
use price::{
    client_helpers::{to_order_info_args, try_encoded_u32_to_decoded_decimal},
    to_order_info,
};
use rand::{rngs::StdRng, RngExt, SeedableRng};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use solana_address::Address;
use solana_keypair::Signer;
use solana_sdk::{message::Instruction, signature::Keypair};
use transaction_parser::views::{try_market_view_all_from_owner_and_data, MarketViewAll};

use crate::{
    config::{MakerStyle, ValidMakerConfig},
    get_non_redundant_order_flow,
    logger::{divider, Logger, BUY_COLOR, SELL_COLOR},
    model::calculate_spreads::{half_spread, reservation_price},
    utils::get_normalized_mid_price,
    MakerState,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct HitSignals {
    ask_was_lifted: bool,
    bid_was_hit: bool,
}

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
    /// The latest market-wide view. Used for local imbalance and microprice signals.
    latest_market: MarketViewAll,
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
    style: MakerStyle,
    quote_ttl: Duration,
    min_refill_delay: Duration,
    max_refill_delay: Duration,
    replenish_ratio: Decimal,
    size_jitter_bps: u16,
    price_jitter_pct: u16,
    hit_widening_bps: u16,
    local_book_weight: Decimal,
    max_quote_levels: usize,
    spread_multiplier: Decimal,
    recent_fair_prices: VecDeque<Decimal>,
    buy_pressure: u8,
    sell_pressure: u8,
    bid_refill_blocked_until: Instant,
    ask_refill_blocked_until: Instant,
    last_quote_refresh: Instant,
    rng: StdRng,
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
            style,
            quote_ttl_ms,
            min_refill_delay_ms,
            max_refill_delay_ms,
            replenish_ratio_bps,
            size_jitter_bps,
            price_jitter_pct,
            hit_widening_bps,
            local_book_weight_bps,
            max_quote_levels,
            spread_multiplier_bps,
            seed,
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
        let latest_market =
            try_market_view_all_from_owner_and_data(market_account.owner, &market_account.data)?;
        let latest_state = MakerState::new_from_market(keypair.pubkey(), latest_market.clone())?;
        let mid_price =
            get_normalized_mid_price(initial_price_feed_response, &oanda_args.pair, &market_ctx)?;
        let maker_address = keypair.pubkey();
        // Maker may temporarily show as `0` balance until the balance fetch succeeds.
        let sol_lamports = rpc.client.get_balance(&maker_address).await.unwrap_or(0);
        let now = Instant::now();
        let recent_fair_prices = VecDeque::from([mid_price]);

        let mut ctx = Self {
            keypair,
            market_ctx,
            maker_address,
            pair: oanda_args.pair,
            latest_state,
            latest_market,
            base_target_atoms,
            mid_price_atoms: mid_price,
            batch_replace,
            bid_order_size,
            ask_order_size,
            visualize,
            needs_expand: false,
            logger: Logger::new(visualize),
            sol_lamports,
            style,
            quote_ttl: Duration::from_millis(quote_ttl_ms),
            min_refill_delay: Duration::from_millis(min_refill_delay_ms),
            max_refill_delay: Duration::from_millis(max_refill_delay_ms),
            replenish_ratio: Decimal::from(replenish_ratio_bps) / Decimal::from(10_000u64),
            size_jitter_bps,
            price_jitter_pct,
            hit_widening_bps,
            local_book_weight: Decimal::from(local_book_weight_bps) / Decimal::from(10_000u64),
            max_quote_levels,
            spread_multiplier: Decimal::from(spread_multiplier_bps) / Decimal::from(10_000u64),
            recent_fair_prices,
            buy_pressure: 0,
            sell_pressure: 0,
            bid_refill_blocked_until: now,
            ask_refill_blocked_until: now,
            last_quote_refresh: now,
            rng: StdRng::seed_from_u64(seed),
        };
        ctx.record_fair_price(mid_price);
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

        let now = Instant::now();
        let (bid_price, ask_price, step) = self.get_bid_and_ask_prices();
        let bid_layers = self.build_layers(bid_price, step, true, now)?;
        let ask_layers = self.build_layers(ask_price, step, false, now)?;

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

            let bid_args = bid_layers
                .into_iter()
                .map(|(price, size)| to_order_info_args(price, size).map_err(anyhow::Error::from))
                .collect::<anyhow::Result<Vec<_>>>()?;

            let ask_args = ask_layers
                .into_iter()
                .map(|(price, size)| to_order_info_args(price, size).map_err(anyhow::Error::from))
                .collect::<anyhow::Result<Vec<_>>>()?;

            let ixn = self.market_ctx.batch_replace(
                self.maker_address,
                BatchReplaceInstructionData::new(
                    self.seat_index(),
                    UnvalidatedOrders::new_from_slice(&bid_args)
                        .map_err(|e| anyhow::anyhow!("{e:?}"))?,
                    UnvalidatedOrders::new_from_slice(&ask_args)
                        .map_err(|e| anyhow::anyhow!("{e:?}"))?,
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
        let new_state = MakerState::new_from_market(self.maker_address, new_market_state.clone())?;
        self.observe_market_hits(&new_state);
        self.latest_state = new_state;
        self.latest_market = new_market_state;
        // Use the same fair-value series everywhere `recent_fair_prices` is fed
        // so the volatility estimate doesn't mix incompatible sources.
        self.record_fair_price(self.effective_mid_price());
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

        let mut asks = decode(&self.latest_market.asks);
        let mut bids = decode(&self.latest_market.bids);

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

        let mut lines = Vec::with_capacity(8 + MAX_ORDERS_USIZE * 2 + 1);

        let base_msg = format!("{bs:>col_w$} seat  |  {bo:>col_w$} orders  |  {bt:>col_w$} total");
        let quote_msg = format!("{qs:>col_w$} seat  |  {qo:>col_w$} orders  |  {qt:>col_w$} total");
        let value_msg = format!("{total_value_display:.value_precision$} quote");
        let book_mid = self
            .local_microprice_atoms()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "n/a".to_string());
        let spread_msg = self
            .current_market_spread_bps()
            .map(|spread| format!("{spread:.2} bps"))
            .unwrap_or_else(|| "n/a".to_string());
        let imbalance_msg = format!("{:.2}%", self.book_imbalance() * 100.0);
        lines.push(fmt_kv!("SOL  ", format!("{sol:.6}")));
        lines.push(fmt_kv!("Base ", base_msg));
        lines.push(fmt_kv!("Quote", quote_msg));
        lines.push(fmt_kv!("Value", value_msg));
        lines.push(fmt_kv!("Style", format!("{:?}", self.style)));
        lines.push(fmt_kv!("Mid  ", book_mid));
        lines.push(fmt_kv!("Sprd ", spread_msg));
        lines.push(fmt_kv!("Imbal", imbalance_msg));
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
        self.record_fair_price(self.effective_mid_price());

        Ok(())
    }

    /// Calculates the model's output bid and ask prices based on the market's current mid price
    /// and the maker's current state.
    ///
    /// Note that these prices are already normalized to being in atoms.
    fn get_bid_and_ask_prices(&self) -> (Decimal, Decimal, Decimal) {
        let effective_mid = self.effective_mid_price();
        let imbalance_skew_bps =
            (self.book_imbalance() * f64::from(self.hit_widening_bps)).round() as i64;
        let pressure_skew_bps = i64::from(self.buy_pressure) * i64::from(self.hit_widening_bps / 2)
            - i64::from(self.sell_pressure) * i64::from(self.hit_widening_bps / 2);
        let skew = effective_mid * Decimal::from(imbalance_skew_bps + pressure_skew_bps)
            / Decimal::from(10_000u64);
        let reservation_price = reservation_price(effective_mid + skew, self.q());
        let step = self.dynamic_half_spread();
        let bid_price = reservation_price - step;
        let ask_price = reservation_price + step;

        (bid_price, ask_price, step)
    }

    pub fn quote_ttl(&self) -> Duration {
        self.quote_ttl
    }

    pub fn mark_orders_submitted(&mut self) {
        self.last_quote_refresh = Instant::now();
        self.buy_pressure = self.buy_pressure.saturating_sub(1);
        self.sell_pressure = self.sell_pressure.saturating_sub(1);
    }

    pub fn should_refresh_quotes(&self) -> bool {
        self.last_quote_refresh.elapsed() >= self.quote_ttl
    }

    fn record_fair_price(&mut self, price: Decimal) {
        self.recent_fair_prices.push_back(price);
        while self.recent_fair_prices.len() > 24 {
            self.recent_fair_prices.pop_front();
        }
    }

    fn observe_market_hits(&mut self, new_state: &MakerState) {
        let prev_ask_base: u64 = self
            .latest_state
            .asks
            .iter()
            .map(|o| o.base_remaining)
            .sum();
        let next_ask_base: u64 = new_state.asks.iter().map(|o| o.base_remaining).sum();
        let prev_bid_base: u64 = self
            .latest_state
            .bids
            .iter()
            .map(|o| o.base_remaining)
            .sum();
        let next_bid_base: u64 = new_state.bids.iter().map(|o| o.base_remaining).sum();
        let now = Instant::now();
        let signals = detect_hit_signals(
            prev_ask_base,
            next_ask_base,
            prev_bid_base,
            next_bid_base,
            self.latest_state.base_inventory,
            new_state.base_inventory,
            self.latest_state.quote_inventory,
            new_state.quote_inventory,
        );

        if signals.ask_was_lifted {
            self.buy_pressure = self.buy_pressure.saturating_add(1).min(8);
            self.ask_refill_blocked_until = now + self.random_refill_delay();
        }

        if signals.bid_was_hit {
            self.sell_pressure = self.sell_pressure.saturating_add(1).min(8);
            self.bid_refill_blocked_until = now + self.random_refill_delay();
        }
    }

    fn random_refill_delay(&mut self) -> Duration {
        let min_ms = self.min_refill_delay.as_millis() as u64;
        let max_ms = self.max_refill_delay.as_millis() as u64;
        let delay_ms = if min_ms == max_ms {
            min_ms
        } else {
            self.rng.random_range(min_ms..=max_ms)
        };
        Duration::from_millis(delay_ms)
    }

    fn build_layers(
        &mut self,
        anchor_price: Decimal,
        step: Decimal,
        is_bid: bool,
        now: Instant,
    ) -> anyhow::Result<Vec<(Decimal, u64)>> {
        let refill_blocked = if is_bid {
            now < self.bid_refill_blocked_until
        } else {
            now < self.ask_refill_blocked_until
        };
        let start_offset = usize::from(refill_blocked);
        let target_levels = self.max_quote_levels.saturating_sub(start_offset).max(1);
        let replenish_ratio = if refill_blocked {
            self.replenish_ratio
        } else {
            Decimal::ONE
        };
        let mut raw_layers = Vec::with_capacity(target_levels);

        for i in 0..target_levels {
            let level = i + start_offset;
            let ladder_index = Decimal::from((i + 1) as u64);
            let mut price = if is_bid {
                anchor_price - step * Decimal::from(level as u64)
            } else {
                anchor_price + step * Decimal::from(level as u64)
            };
            let price_jitter =
                step * Decimal::from(self.rng.random_range(
                    -(self.price_jitter_pct as i32)..=(self.price_jitter_pct as i32),
                )) / Decimal::from(100u64);
            price += price_jitter;
            if price <= Decimal::ZERO {
                continue;
            }

            let base_size = if is_bid {
                (Decimal::from(self.bid_order_size) * ladder_index * replenish_ratio / price)
                    .round()
            } else {
                (Decimal::from(self.ask_order_size) * ladder_index * replenish_ratio).round()
            };
            let size_jitter =
                Decimal::ONE
                    + Decimal::from(self.rng.random_range(
                        -(self.size_jitter_bps as i32)..=(self.size_jitter_bps as i32),
                    )) / Decimal::from(10_000u64);
            let size = (base_size * size_jitter)
                .round()
                .max(Decimal::ONE)
                .to_u64()
                .with_context(|| format!("Couldn't convert order size to u64 at level {level}"))?;
            raw_layers.push((price, size));
        }

        if is_bid {
            raw_layers.sort_by(|a, b| b.0.cmp(&a.0));
        } else {
            raw_layers.sort_by(|a, b| a.0.cmp(&b.0));
        }

        let mut seen_prices = HashSet::new();
        let mut layers = Vec::with_capacity(raw_layers.len());
        for (price, size) in raw_layers {
            let info =
                to_order_info(to_order_info_args(price, size).map_err(anyhow::Error::from)?)?;
            if seen_prices.insert(info.encoded_price.as_u32()) {
                layers.push((price, size));
            }
        }

        Ok(layers)
    }

    fn effective_mid_price(&self) -> Decimal {
        let external = self.get_mid_price_atoms();
        match self.local_microprice_atoms() {
            Some(local) => {
                external * (Decimal::ONE - self.local_book_weight) + local * self.local_book_weight
            }
            None => external,
        }
    }

    fn dynamic_half_spread(&self) -> Decimal {
        let base = half_spread() * self.spread_multiplier;
        let mid = self.effective_mid_price();
        let pressure_units = u16::from(self.buy_pressure.max(self.sell_pressure)).min(4);
        let pressure_component = mid
            * Decimal::from(u64::from(self.hit_widening_bps) * u64::from(pressure_units))
            / Decimal::from(40_000u64);
        let volatility_component = mid
            * Decimal::from(self.recent_volatility_bps().round().clamp(0.0, 100.0) as u64)
            / Decimal::from(40_000u64);

        base + pressure_component + volatility_component
    }

    fn recent_volatility_bps(&self) -> f64 {
        if self.recent_fair_prices.len() < 2 {
            return 0.0;
        }

        let min = self.recent_fair_prices.iter().min().copied();
        let max = self.recent_fair_prices.iter().max().copied();
        let latest = self.recent_fair_prices.back().copied();
        match (min, max, latest) {
            (Some(min), Some(max), Some(latest)) if latest > Decimal::ZERO => max
                .checked_sub(min)
                .and_then(|range| range.checked_mul(Decimal::from(10_000u64)))
                .and_then(|scaled| scaled.checked_div(latest))
                .and_then(|bps| bps.to_f64())
                .unwrap_or_default(),
            _ => 0.0,
        }
    }

    fn local_microprice_atoms(&self) -> Option<Decimal> {
        let best_bid = self.latest_market.bids.first()?;
        let best_ask = self.latest_market.asks.first()?;
        let bid_price = try_encoded_u32_to_decoded_decimal(best_bid.encoded_price.as_u32()).ok()?;
        let ask_price = try_encoded_u32_to_decoded_decimal(best_ask.encoded_price.as_u32()).ok()?;
        let bid_weight = Decimal::from(best_bid.base_remaining);
        let ask_weight = Decimal::from(best_ask.base_remaining);
        let denom = bid_weight + ask_weight;
        if denom <= Decimal::ZERO {
            return None;
        }

        (ask_price * bid_weight + bid_price * ask_weight).checked_div(denom)
    }

    fn current_market_spread_bps(&self) -> Option<f64> {
        let best_bid = self.latest_market.bids.first()?;
        let best_ask = self.latest_market.asks.first()?;
        let bid_price = try_encoded_u32_to_decoded_decimal(best_bid.encoded_price.as_u32()).ok()?;
        let ask_price = try_encoded_u32_to_decoded_decimal(best_ask.encoded_price.as_u32()).ok()?;
        let mid = (bid_price + ask_price) / Decimal::from(2u8);
        if mid <= Decimal::ZERO {
            return None;
        }

        (ask_price - bid_price)
            .checked_mul(Decimal::from(10_000u64))
            .and_then(|scaled| scaled.checked_div(mid))
            .and_then(|bps| bps.to_f64())
    }

    fn book_imbalance(&self) -> f64 {
        let bid_depth: u64 = self
            .latest_market
            .bids
            .iter()
            .take(3)
            .map(|order| order.base_remaining)
            .sum();
        let ask_depth: u64 = self
            .latest_market
            .asks
            .iter()
            .take(3)
            .map(|order| order.base_remaining)
            .sum();
        let denom = bid_depth + ask_depth;
        if denom == 0 {
            0.0
        } else {
            (bid_depth as f64 - ask_depth as f64) / denom as f64
        }
    }
}

fn detect_hit_signals(
    prev_ask_base: u64,
    next_ask_base: u64,
    prev_bid_base: u64,
    next_bid_base: u64,
    prev_base_inventory: u64,
    next_base_inventory: u64,
    prev_quote_inventory: u64,
    next_quote_inventory: u64,
) -> HitSignals {
    HitSignals {
        ask_was_lifted: next_ask_base < prev_ask_base
            && next_base_inventory < prev_base_inventory
            && next_quote_inventory > prev_quote_inventory,
        bid_was_hit: next_bid_base < prev_bid_base
            && next_quote_inventory < prev_quote_inventory
            && next_base_inventory > prev_base_inventory,
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_hit_signals, HitSignals};

    #[test]
    fn self_requotes_do_not_look_like_hits() {
        let signals = detect_hit_signals(1_000, 600, 900, 700, 10_000, 10_000, 20_000, 20_000);

        assert_eq!(signals, HitSignals::default());
    }

    #[test]
    fn unilateral_cancels_do_not_look_like_hits() {
        // Ask depth shrank with no inventory transfer — a self-initiated cancel.
        let ask_cancel = detect_hit_signals(1_000, 600, 900, 900, 10_000, 10_000, 20_000, 20_000);
        assert_eq!(ask_cancel, HitSignals::default());

        // Bid depth shrank with no inventory transfer — also a cancel, not a hit.
        let bid_cancel = detect_hit_signals(1_000, 1_000, 900, 500, 10_000, 10_000, 20_000, 20_000);
        assert_eq!(bid_cancel, HitSignals::default());
    }

    #[test]
    fn inventory_transfer_confirms_real_hits() {
        let ask_fill = detect_hit_signals(1_000, 600, 900, 900, 10_000, 9_600, 20_000, 20_400);
        assert_eq!(
            ask_fill,
            HitSignals {
                ask_was_lifted: true,
                bid_was_hit: false,
            }
        );

        let bid_fill = detect_hit_signals(1_000, 1_000, 900, 500, 10_000, 10_400, 20_000, 19_600);
        assert_eq!(
            bid_fill,
            HitSignals {
                ask_was_lifted: false,
                bid_was_hit: true,
            }
        );
    }
}
