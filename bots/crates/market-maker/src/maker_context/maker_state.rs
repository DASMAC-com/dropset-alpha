use dropset_interface::state::{
    asks_dll::AskOrders,
    bids_dll::BidOrders,
    user_order_sectors::OrderSectors,
};
use itertools::Itertools;
use solana_address::Address;
use transaction_parser::views::{
    MarketSeatView,
    MarketViewAll,
    OrderView,
};

use crate::maker_context::utils::{
    find_maker_seat,
    find_order,
};

/// Tracks the market maker's seat, bids, asks, and total base and quote inventory for a market.
///
/// To simplify this struct's interface, the market maker must have already been registered prior
/// to instantiating this struct.
#[derive(Debug)]
pub struct MakerState {
    pub address: Address,
    pub seat: MarketSeatView,
    pub bids: Vec<OrderView>,
    pub asks: Vec<OrderView>,
    /// The maker's current base inventory; i.e., the [`MarketSeatView::base_available`] + the
    /// base in all open orders.
    pub base_inventory: u64,
    /// The maker's current quote inventory; i.e., the [`MarketSeatView::quote_available`] + the
    /// quote in all open orders.
    pub quote_inventory: u64,
}

impl MakerState {
    /// Creates the market maker's state based on the passed [`MarketViewAll`] state.
    /// If the maker doesn't have a seat registered yet this will fail.
    pub fn new_from_market(maker_address: Address, market: &MarketViewAll) -> anyhow::Result<Self> {
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

        // Map each bid price to its corresponding order.
        let bids = bid_prices
            .iter()
            .map(|price| find_order::<BidOrders>(*price, &market.bids, seat.index))
            .collect::<Option<Vec<_>>>()
            .expect("Should find the bid");

        // Map each ask price to its corresponding order.
        let asks = ask_prices
            .iter()
            .map(|price| find_order::<AskOrders>(*price, &market.asks, seat.index))
            .collect::<Option<Vec<_>>>()
            .expect("Should find the ask");

        // Sum the maker's base inventory by adding the seat balance + the bid collateral amounts.
        let base_inventory = bids
            .iter()
            .fold(seat.base_available, |v, order| v + order.base_remaining);

        // Sum the maker's quote inventory by adding the seat balance + the ask collateral amounts.
        let quote_inventory = asks
            .iter()
            .fold(seat.quote_available, |v, order| v + order.quote_remaining);

        Ok(Self {
            address: maker_address,
            seat,
            bids,
            asks,
            base_inventory,
            quote_inventory,
        })
    }
}
