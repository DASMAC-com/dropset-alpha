//! See [`DepositWithdrawContext`].

use dropset_interface::instructions::generated_program::Deposit;
use pinocchio::{
    account::AccountView,
    error::ProgramError,
};

use crate::validation::{
    market_account_view::MarketAccountView,
    mint_account_view::MintAccountView,
    token_account_view::TokenAccountView,
};

/// The account context for the [`Deposit`] and
/// [`dropset_interface::instructions::generated_program::Withdraw`] instructions, verifying token
/// ownership, mint consistency, and associated token account correctness.
#[derive(Clone)]
pub struct DepositWithdrawContext<'a> {
    // The event authority is validated by the inevitable `FlushEvents` self-CPI.
    pub event_authority: &'a AccountView,
    pub user: &'a AccountView,
    pub market_account: MarketAccountView<'a>,
    pub user_ata: TokenAccountView<'a>,
    pub market_ata: TokenAccountView<'a>,
    pub mint: MintAccountView<'a>,
}

impl<'a> DepositWithdrawContext<'a> {
    /// # Safety
    ///
    /// Caller guarantees no accounts passed have their data borrowed in any capacity. This is a
    /// more restrictive safety contract than is necessary for soundness but is much simpler.
    pub unsafe fn load(
        accounts: &'a [AccountView],
    ) -> Result<DepositWithdrawContext<'a>, ProgramError> {
        // `Withdraw`'s account info fields are in the same exact order as `Deposit`'s, so just use
        // `Deposit::load_accounts` for both. This invariant is checked below in unit tests.
        let Deposit {
            event_authority,
            user,
            market_account,
            user_ata,
            market_ata,
            mint,
            token_program: _,
            system_program: _,
            dropset_program: _,
        } = Deposit::load_accounts(accounts)?;

        // Safety: Scoped borrow of market account data.
        let (market_account, mint) = unsafe {
            let market_account = MarketAccountView::new(market_account)?;
            let market = market_account.load_unchecked();
            let mint = MintAccountView::new(mint, market)?;
            (market_account, mint)
        };

        // Safety: Scoped borrows of the user token account and market token account.
        let (user_ata, market_ata) = unsafe {
            let user_ata = TokenAccountView::new(user_ata, mint.account.address(), user.address())?;
            let market_ata = TokenAccountView::new(
                market_ata,
                mint.account.address(),
                market_account.account().address(),
            )?;
            (user_ata, market_ata)
        };

        Ok(Self {
            event_authority,
            user,
            market_account,
            user_ata,
            market_ata,
            mint,
        })
    }
}
