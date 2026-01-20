use client::{
    context::market::MarketContext,
    transactions::CustomRpcClient,
};
use dropset_interface::{
    instructions::{
        CancelOrderInstructionData,
        PostOrderInstructionData,
    },
    state::{
        sector::SectorIndex,
        user_order_sectors::OrderSectors,
    },
};
use itertools::Itertools;
use price::client_helpers::{
    decimal_pow10_i16,
    to_order_info_args,
};
use rust_decimal::Decimal;
use solana_address::Address;
use solana_sdk::{
    message::Instruction,
    signature::Keypair,
};
use transaction_parser::views::{
    MarketSeatView,
    MarketViewAll,
    OrderView,
};

use crate::{
    calculate_spreads::{
        half_spread,
        reservation_price,
    },
    oanda::{
        CurrencyPair,
        OandaCandlestickResponse,
    },
};

const ORDER_SIZE: u64 = 10_000;

pub struct MakerState {
    pub transaction_version: u64,
    pub address: Address,
    pub seat: MarketSeatView,
    pub bids: Vec<OrderView>,
    pub asks: Vec<OrderView>,
    pub base_inventory: u64,
    pub quote_inventory: u64,
}

fn find_maker_seat(market: &MarketViewAll, maker: &Address) -> anyhow::Result<MarketSeatView> {
    let res = market.seats.binary_search_by_key(maker, |v| v.user);
    let seat = match res {
        Ok(found_index) => market
            .seats
            .get(found_index)
            .expect("Seat index should be valid")
            .clone(),
        Err(_insert_index) => anyhow::bail!("Couldn't find maker in seat list."),
    };

    Ok(seat)
}

impl MakerState {
    pub fn new_from_market(
        transaction_version: u64,
        maker_address: Address,
        market: &MarketViewAll,
    ) -> anyhow::Result<Self> {
        let seat = find_maker_seat(market, &maker_address)?;

        // Convert a user's order sectors into a Vec<u32> of prices.
        let to_prices = |order_sectors: &OrderSectors| -> Vec<u32> {
            order_sectors
                .iter()
                .filter(|b| !b.is_free())
                .map(|p| u32::from_le_bytes(p.encoded_price.as_array()))
                .collect_vec()
        };

        let bid_prices = to_prices(&seat.user_order_sectors.bids);
        let ask_prices = to_prices(&seat.user_order_sectors.asks);

        // Given a price and a collection of orders, find the unique order associated with the pric
        // passed. This is just for the bids and asks in this local function so all passed prices
        // should map to a valid order, hence the `.expect(...)` calls instead of returning Results.
        let find_order_by_price = |price: &u32, orders: &[OrderView]| {
            let order_list_index = orders
                .binary_search_by_key(price, |order| order.encoded_price)
                .expect("Should find order with matching encoded price");
            orders
                .get(order_list_index)
                .expect("Index should correspond to a valid order")
                .clone()
        };

        // Map each bid price to its corresponding order.
        let bids = bid_prices
            .iter()
            .map(|price| find_order_by_price(price, &market.bids))
            .collect_vec();

        // Map each ask price to its corresponding order.
        let asks = ask_prices
            .iter()
            .map(|price| find_order_by_price(price, &market.asks))
            .collect_vec();

        // Sum the maker's base inventory by adding the seat balance + the bid collateral amounts.
        let base_inventory = bids
            .iter()
            .fold(seat.base_available, |acc, seat| acc + seat.base_remaining);

        // Sum the maker's quote inventory by adding the seat balance + the ask collateral amounts.
        let quote_inventory = asks
            .iter()
            .fold(seat.quote_available, |acc, seat| acc + seat.quote_remaining);

        Ok(Self {
            transaction_version,
            address: maker_address,
            seat,
            bids,
            asks,
            base_inventory,
            quote_inventory,
        })
    }
}

pub struct MakerContext<'a> {
    /// The maker's keypair.
    keypair: Keypair,
    market_ctx: &'a MarketContext,
    /// The maker's address.
    address: Address,
    /// The currency pair.
    pair: CurrencyPair,
    /// The maker's initial state.
    initial_state: MakerState,
    /// The maker's latest state.
    latest_state: MakerState,
    /// The change in the market maker's base inventory value as a signed integer, in atoms.
    ///
    /// In the A-S model `q` represents the base inventory as a reflection of the maker's net short
    /// (negative) or long (positive) position. The change in base inventory from initial to
    /// current state thus can be used in place of `q` to achieve the effect of always returning to
    /// the initial base inventory amount.
    ///
    /// When `q` is negative, the maker is below the desired/target inventory amount, and when `q`
    /// is positive, the maker is above the desired/target inventory amount.
    ///
    /// In practice, this has two opposing effects.
    /// - When q is negative, it pushes the spread upwards so that bid prices are closer to the
    ///   [`crate::calculate_spreads::reservation_price`] and ask prices are further away. This
    ///   effectively increases the likelihood of getting bids filled and vice versa for asks.
    /// - When q is positive, it pushes the spread downwards so that ask prices are closer to the
    ///   [`crate::calculate_spreads::reservation_price`] price and bid prices are further away.
    ///   This effectively increases the likelihood of getting asks filled and vice versa for bids.
    pub base_inventory_delta: Decimal,
    /// The change in quote inventory since the initial maker state was created, in atoms.
    /// This isn't used by the A-S model but is helpful for debugging purposes.
    pub quote_inventory_delta: Decimal,
    /// The reference mid price, expressed as quote atom per 1 base atom.
    ///
    /// In the A–S model this is an exogenous “fair price” process; in practice you can source it
    /// externally (e.g. FX feed) or derive it internally from the venue’s top-of-book.
    /// It anchors the reservation price and thus the bid/ask quotes via the spread model.
    ///
    /// Note that the price as quote_atoms / base_atoms may differ from quote / base. Be sure to
    /// express the price as a ratio of atoms.
    mid_price: Decimal,
}

impl MakerContext<'_> {
    /// See [`MakerContext::mid_price`].
    pub fn mid_price(&self) -> Decimal {
        self.mid_price
    }

    pub fn maker_seat(&self) -> SectorIndex {
        self.latest_state.seat.index
    }

    pub async fn cancel_all_and_post_new(&mut self, rpc: &CustomRpcClient) -> anyhow::Result<()> {
        // NOTE: The bids and asks here might be stale due to fills. This will cause the cancel
        // order attempt to fail. This is an expected possible error.
        let cancel_bid_instructions = self
            .latest_state
            .bids
            .iter()
            .map(|bid| {
                self.market_ctx.cancel_order(
                    self.address,
                    CancelOrderInstructionData::new(bid.encoded_price, true, self.maker_seat()),
                )
            })
            .collect_vec();
        let cancel_ask_instructions = self
            .latest_state
            .asks
            .iter()
            .map(|ask| {
                self.market_ctx.cancel_order(
                    self.address,
                    CancelOrderInstructionData::new(ask.encoded_price, false, self.maker_seat()),
                )
            })
            .collect_vec();

        let (bid_price, ask_price) = self.get_bid_and_ask_prices();
        let to_post_ixn = |price: Decimal, size: u64, is_bid: bool, seat_index: SectorIndex| {
            to_order_info_args(price, size)
                .map_err(|e| anyhow::anyhow! {"{e:#?}"})
                .map(|args| {
                    PostOrderInstructionData::new(
                        args.0, args.1, args.2, args.3, is_bid, seat_index,
                    )
                })
        };

        let post_instructions = vec![
            self.market_ctx.post_order(
                self.address,
                to_post_ixn(bid_price, ORDER_SIZE, true, self.maker_seat())?,
            ),
            self.market_ctx.post_order(
                self.address,
                to_post_ixn(ask_price, ORDER_SIZE, false, self.maker_seat())?,
            ),
        ];

        let ixns = [
            cancel_ask_instructions,
            cancel_bid_instructions,
            post_instructions,
        ]
        .into_iter()
        .concat();

        rpc.send_and_confirm_txn(
            &self.keypair,
            &[&self.keypair],
            ixns.into_iter()
                .map(Instruction::from)
                .collect_vec()
                .as_ref(),
        )
        .await?;

        Ok(())
    }

    pub fn update_state_and_inventory_deltas(
        &mut self,
        transaction_version: u64,
        new_market_state: &MarketViewAll,
    ) -> anyhow::Result<()> {
        self.latest_state =
            MakerState::new_from_market(transaction_version, self.address, new_market_state)?;
        self.base_inventory_delta = Decimal::from(self.latest_state.base_inventory)
            - Decimal::from(self.initial_state.base_inventory);
        self.quote_inventory_delta = Decimal::from(self.latest_state.quote_inventory)
            - Decimal::from(self.initial_state.quote_inventory);

        Ok(())
    }

    pub fn update_price_from_candlestick(
        &mut self,
        candlestick_response: OandaCandlestickResponse,
    ) -> anyhow::Result<()> {
        let maker_pair = self.pair.to_string();
        let response_pair = candlestick_response.instrument;
        if maker_pair != response_pair {
            anyhow::bail!("Maker and and candlestick response pair don't match. {maker_pair} != {response_pair}");
        }

        if !candlestick_response.candles.is_sorted_by_key(|c| c.time) {
            anyhow::bail!("Candlesticks aren't sorted by time (ascending).");
        }

        let latest = candlestick_response.candles.last();
        let latest_price = match latest {
            Some(candlestick) => {
                candlestick
                    .mid
                    .as_ref()
                    .ok_or_else(|| {
                        let err = anyhow::anyhow!("`mid` price not found in the last candlestick.");
                        err
                    })?
                    .c
            }
            None => anyhow::bail!("There are zero candlesticks in the candlestick response"),
        };

        // Normalize the price based on the token decimals.
        let normalized_latest_price = normalize_non_atoms_price(
            latest_price,
            self.market_ctx.base.mint_decimals,
            self.market_ctx.quote.mint_decimals,
        );

        self.mid_price = normalized_latest_price;

        Ok(())
    }

    /// Calculates the model's output bid and ask prices as a function of the current mid price and
    /// the maker's base inventory delta.
    fn get_bid_and_ask_prices(&self) -> (Decimal, Decimal) {
        let reservation_price = reservation_price(self.mid_price(), self.base_inventory_delta);
        let bid_price = reservation_price - half_spread();
        let ask_price = reservation_price + half_spread();

        (bid_price, ask_price)
    }
}

/// Converts a token price not denominated in atoms to a token price denominated in atoms using
/// exponentiation based on the base and quote token's decimals.
fn normalize_non_atoms_price(
    non_atoms_price: Decimal,
    base_decimals: u8,
    quote_decimals: u8,
) -> Decimal {
    decimal_pow10_i16(
        non_atoms_price,
        quote_decimals as i16 - base_decimals as i16,
    )
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use crate::maker_context::normalize_non_atoms_price;

    #[test]
    fn varying_decimal_pair() {
        // Equal decimals => do nothing.
        assert_eq!(normalize_non_atoms_price(dec!(1.27), 6, 6), dec!(1.27));

        // 10 ^ (quote - base) == 10 ^ 1 == multiply by 10
        assert_eq!(normalize_non_atoms_price(dec!(1.27), 5, 6), dec!(12.7));

        // 10 ^ (quote - base) == 10 ^ -1 == divide by 10
        assert_eq!(normalize_non_atoms_price(dec!(1.27), 6, 5), dec!(0.127));

        // 10 ^ (quote - base) == 10 ^ (19 - 11) == multiply by 10 ^ 8
        assert_eq!(
            normalize_non_atoms_price(dec!(1.27), 11, 19),
            dec!(127_000_000)
        );

        // 10 ^ (quote - base) == 10 ^ (11 - 19) = divide by 10 ^ 8
        assert_eq!(
            normalize_non_atoms_price(dec!(1.27), 19, 11),
            dec!(0.0000000127)
        );
    }
}
