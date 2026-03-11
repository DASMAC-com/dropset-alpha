use solana_address::Address;
use transaction_parser::views::{
    MarketSeatView,
    MarketUserData,
    MarketViewAll,
    OrderView,
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
    pub fn new_from_market(maker_address: Address, market: MarketViewAll) -> anyhow::Result<Self> {
        let mut users = market.users;
        let MarketUserData { seat, bids, asks } = users.remove(&maker_address).ok_or(
            anyhow::anyhow!("Couldn't find market maker in market user data"),
        )?;

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
