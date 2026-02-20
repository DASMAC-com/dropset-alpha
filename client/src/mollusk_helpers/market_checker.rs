use std::collections::HashMap;

use dropset_interface::state::sector::SectorIndex;
use mollusk_svm::MolluskContext;
use solana_account::Account;
use solana_address::Address;
use transaction_parser::views::MarketSeatView;

use crate::{
    context::market::MarketContext,
    mollusk_helpers::helper_trait::DropsetTestHelper,
};

pub struct MarketChecker<'a> {
    mollusk: &'a MolluskContext<HashMap<Address, Account>>,
    market_ctx: &'a MarketContext,
}

impl<'a> MarketChecker<'a> {
    pub fn new(
        mollusk: &'a MolluskContext<HashMap<Address, Account>>,
        market_ctx: &'a MarketContext,
    ) -> Self {
        Self {
            mollusk,
            market_ctx,
        }
    }

    pub fn base_balance(&self, owner: &Address, expected: u64) {
        let base_mint = &self.market_ctx.base.mint_address;
        assert_eq!(self.mollusk.get_token_balance(owner, base_mint), expected);
    }

    pub fn quote_balance(&self, owner: &Address, expected: u64) {
        let quote_mint = &self.market_ctx.quote.mint_address;
        assert_eq!(self.mollusk.get_token_balance(owner, quote_mint), expected);
    }

    pub fn seat_base_available(&self, user: &Address, expected: u64) {
        let market = self.mollusk.view_market(&self.market_ctx.market);
        let seat = self
            .market_ctx
            .find_seat(&market.seats, user)
            .unwrap_or_else(|| panic!("No seat found for user {user}"));
        assert_eq!(seat.base_available, expected);
    }

    pub fn seat_quote_available(&self, user: &Address, expected: u64) {
        let market = self.mollusk.view_market(&self.market_ctx.market);
        let seat = self
            .market_ctx
            .find_seat(&market.seats, user)
            .unwrap_or_else(|| panic!("No seat found for user {user}"));
        assert_eq!(seat.quote_available, expected);
    }

    pub fn num_asks(&self, expected: usize) {
        let market = self.mollusk.view_market(&self.market_ctx.market);
        assert_eq!(market.asks.len(), expected);
    }

    pub fn num_bids(&self, expected: usize) {
        let market = self.mollusk.view_market(&self.market_ctx.market);
        assert_eq!(market.bids.len(), expected);
    }

    pub fn num_seats(&self, expected: usize) {
        let market = self.mollusk.view_market(&self.market_ctx.market);
        assert_eq!(market.seats.len(), expected);
    }

    pub fn seat_index(&self, user: &Address, expected: SectorIndex) {
        let market = self.mollusk.view_market(&self.market_ctx.market);
        let seat = self
            .market_ctx
            .find_seat(&market.seats, user)
            .unwrap_or_else(|| panic!("No seat found for user {user}"));
        assert_eq!(seat.index, expected);
    }

    /// Retrieves the seat for `user` and passes it to `f` for custom assertions.
    /// Use this for fields not covered by the typed helpers, e.g. linked-list structure.
    ///
    /// ```ignore
    /// check.seat(&user, |seat| {
    ///     // Check that the user is the last seat in the seat list.
    ///     assert_eq!(seat.next_index, NIL);
    /// });
    /// ```
    pub fn seat(&self, user: &Address, f: impl FnOnce(&MarketSeatView)) {
        let market = self.mollusk.view_market(&self.market_ctx.market);
        let seat = self
            .market_ctx
            .find_seat(&market.seats, user)
            .unwrap_or_else(|| panic!("No seat found for user {user}"));
        f(&seat);
    }
}
