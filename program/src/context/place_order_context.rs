//! See [`PlaceOrderContext`].

use dropset_interface::instructions::generated_pinocchio::PlaceOrder;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
};

use crate::validation::market_account_info::MarketAccountInfo;

/// The account context for the [`PlaceOrder`] instruction, validating the market account passed in.
#[derive(Clone)]
pub struct PlaceOrderContext<'a> {
    // The event authority is validated by the inevitable `FlushEvents` self-CPI.
    pub event_authority: &'a AccountInfo,
    pub user: &'a AccountInfo,
    pub market_account: MarketAccountInfo<'a>,
}

impl<'a> PlaceOrderContext<'a> {
    /// # Safety
    ///
    /// Caller guarantees:
    /// - WRITE accounts are not currently borrowed in *any* capacity.
    /// - READ accounts are not currently mutably borrowed.
    ///
    /// ### Accounts
    ///   0. `[READ]` Market account
    pub unsafe fn load(accounts: &'a [AccountInfo]) -> Result<PlaceOrderContext<'a>, ProgramError> {
        let PlaceOrder {
            event_authority,
            user,
            market_account,
            dropset_program: _,
        } = PlaceOrder::load_accounts(accounts)?;

        // Safety: Scoped borrow of market account data.
        let market_account = unsafe { MarketAccountInfo::new(market_account) }?;

        Ok(Self {
            event_authority,
            user,
            market_account,
        })
    }
}
