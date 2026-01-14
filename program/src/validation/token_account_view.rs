//! See [`TokenAccountView`].

use dropset_interface::error::DropsetError;
use pinocchio::{
    account::AccountView,
    error::ProgramError,
};
use pinocchio_token_interface::state::{
    account::Account,
    load as pinocchio_load,
    load_unchecked as pinocchio_load_unchecked,
};
use solana_address::{
    address_eq,
    Address,
};

/// A validated wrapper around a raw associated token account [`AccountView`], ensuring correct mint
/// association, owner authority, and account state.
#[derive(Clone)]
pub struct TokenAccountView<'a> {
    pub account: &'a AccountView,
}

impl<'a> TokenAccountView<'a> {
    /// # Safety
    ///
    /// Caller guarantees:
    /// - WRITE accounts are not currently borrowed in *any* capacity.
    /// - READ accounts are not currently mutably borrowed.
    ///
    /// ### Accounts
    ///   0. `[READ]` Token account
    #[inline(always)]
    pub unsafe fn new(
        token_account: &'a AccountView,
        expected_mint: &Address,
        expected_owner: &Address,
    ) -> Result<TokenAccountView<'a>, ProgramError> {
        // NOTE: It's not necessary to check the token account owners here because if the token
        // accounts passed in aren't owned by one of the programs, the transfer instructions
        // won't be able to write to their account data and will fail.

        // Safety: Immutable borrow of token account data to check the expected mint/owner, dropped
        // before the function returns.
        let account_data = unsafe { token_account.borrow_unchecked() };

        // Note the load below also checks that the account has been initialized.
        // Safety: Mint account owner has been verified, so the account data is valid.
        let mint_token_account = unsafe { pinocchio_load::<Account>(account_data) }
            .map_err(|_| ProgramError::InvalidAccountData)?;

        if !address_eq(&mint_token_account.mint.into(), expected_mint) {
            return Err(DropsetError::MintAccountMismatch.into());
        }
        if !address_eq(&mint_token_account.owner.into(), expected_owner) {
            return Err(DropsetError::IncorrectTokenAccountOwner.into());
        }

        Ok(Self {
            account: token_account,
        })
    }

    /// # Safety
    ///
    /// Caller guarantees:
    /// - WRITE accounts are not currently borrowed in *any* capacity.
    /// - READ accounts are not currently mutably borrowed.
    ///
    /// ### Accounts
    ///   0. `[READ]` Token account
    #[inline(always)]
    pub unsafe fn get_balance(&self) -> Result<u64, ProgramError> {
        let data = unsafe { self.account.borrow_unchecked() };

        // Safety: Account is verified as initialized and owned by one of the spl token programs
        // upon construction of Self.
        Ok(unsafe { pinocchio_load_unchecked::<Account>(data) }
            .map_err(|_| ProgramError::InvalidAccountData)?
            .amount())
    }
}
