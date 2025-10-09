use dropset_interface::{error::DropsetError, utils::owned_by};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{pubkey_eq, Pubkey},
};
use pinocchio_token_interface::state::{
    account::Account, load as pinocchio_load, load_unchecked as pinocchio_load_unchecked,
};

#[derive(Clone)]
pub struct TokenAccountInfo<'a> {
    pub info: &'a AccountInfo,
}

impl<'a> TokenAccountInfo<'a> {
    /// # Safety
    ///
    /// Caller guarantees the token account info isn't being actively borrowed.
    #[inline(always)]
    pub unsafe fn new(
        info: &'a AccountInfo,
        expected_mint: &Pubkey,
        expected_owner: &Pubkey,
    ) -> Result<TokenAccountInfo<'a>, ProgramError> {
        // NOTE: This check is most likely unnecessary since the token program checks this and fails
        // transfers if the check fails.
        if !owned_by(info, &pinocchio_token::ID) && !owned_by(info, &pinocchio_token_2022::ID) {
            return Err(DropsetError::OwnerNotTokenProgram.into());
        }

        // Safety: Caller adheres to the safety contract.
        let account_data = unsafe { info.borrow_data_unchecked() };

        // Note the load below also checks that the account has been initialized.
        // Safety: Mint info account owner has been verified, so the account data is valid.
        let mint_token_account = unsafe { pinocchio_load::<Account>(account_data) }?;

        if !pubkey_eq(&mint_token_account.mint, expected_mint) {
            return Err(DropsetError::MintInfoMismatch.into());
        }
        if !pubkey_eq(&mint_token_account.owner, expected_owner) {
            return Err(DropsetError::IncorrectTokenAccountOwner.into());
        }

        Ok(Self { info })
    }

    /// # Safety
    ///
    /// Caller guarantees the token account info isn't being actively borrowed.
    #[inline(always)]
    pub unsafe fn get_balance(&self) -> Result<u64, ProgramError> {
        // Safety: Caller adheres to the safety contract.
        let data = unsafe { self.info.borrow_data_unchecked() };

        // Safety: Account is verified as initialized and owned by one of the spl token programs
        // upon construction of Self.
        Ok(unsafe { pinocchio_load_unchecked::<Account>(data) }?.amount())
    }
}
