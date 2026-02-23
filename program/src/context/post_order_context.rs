//! See [`PostOrderContext`].

use dropset_interface::instructions::generated_program::PostOrder;
use pinocchio::{
    account::AccountView,
    error::ProgramError,
};

use crate::validation::market_account_view::MarketAccountView;

/// The account context for the [PostOrder] instruction. It validates the market account
/// passed in is a valid dropset account.
///
/// Note that the event authority is validated by the inevitable
/// [dropset_interface::instructions::generated_program::FlushEvents] self-CPI.
#[derive(Clone)]
pub struct PostOrderContext<'a> {
    pub event_authority: &'a AccountView,
    pub user: &'a AccountView,
    pub market_account: MarketAccountView<'a>,
}

impl<'a> PostOrderContext<'a> {
    /// # Safety
    ///
    /// Caller guarantees no accounts passed have their data borrowed in any capacity. This is a
    /// more restrictive safety contract than is necessary for soundness but is much simpler.
    pub unsafe fn load(accounts: &'a [AccountView]) -> Result<PostOrderContext<'a>, ProgramError> {
        let PostOrder {
            event_authority,
            user,
            market_account,
            dropset_program: _,
        } = PostOrder::load_accounts(accounts)?;

        // Safety: Scoped borrow of market account data.
        let market_account = unsafe { MarketAccountView::new(market_account) }?;

        Ok(Self {
            event_authority,
            user,
            market_account,
        })
    }
}
