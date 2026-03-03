//! See [`ExpandMarketContext`].

use dropset_interface::instructions::generated_program::ExpandMarket;
use pinocchio::{
    account::AccountView,
    error::ProgramError,
};

use crate::validation::market_account_view::MarketAccountView;

/// The account context for the [`ExpandMarket`] instruction. Validates that the market account
/// passed in is a valid dropset market.
#[derive(Clone)]
pub struct ExpandMarketContext<'a> {
    // The event authority is validated by the inevitable `FlushEvents` self-CPI.
    pub event_authority: &'a AccountView,
    pub payer: &'a AccountView,
    pub market_account: MarketAccountView<'a>,
}

impl<'a> ExpandMarketContext<'a> {
    /// # Safety
    ///
    /// Caller guarantees no accounts passed have their data borrowed in any capacity. This is a
    /// more restrictive safety contract than is necessary for soundness but is much simpler.
    pub unsafe fn load(
        accounts: &'a [AccountView],
    ) -> Result<ExpandMarketContext<'a>, ProgramError> {
        let ExpandMarket {
            event_authority,
            payer,
            market_account,
            system_program: _,
            dropset_program: _,
        } = ExpandMarket::load_accounts(accounts)?;

        // Safety: Scoped borrow of market account data to validate it's a dropset market account.
        let market_account = MarketAccountView::new(market_account)?;

        Ok(Self {
            event_authority,
            payer,
            market_account,
        })
    }
}
