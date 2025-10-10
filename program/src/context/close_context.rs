use dropset_interface::error::DropsetError;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::validation::{
    market_account_info::MarketAccountInfo, mint_info::MintInfo,
    token_account_info::TokenAccountInfo, token_program_info::TokenProgramInfo,
};

#[derive(Clone)]
pub struct CloseContext<'a> {
    pub user: &'a AccountInfo,
    pub market_account: MarketAccountInfo<'a>,
    pub base_mint: MintInfo<'a>,
    pub quote_mint: MintInfo<'a>,
    pub user_base_ata: TokenAccountInfo<'a>,
    pub user_quote_ata: TokenAccountInfo<'a>,
    pub market_base_ata: TokenAccountInfo<'a>,
    pub market_quote_ata: TokenAccountInfo<'a>,
    pub base_token_program: TokenProgramInfo<'a>,
    pub quote_token_program: TokenProgramInfo<'a>,
}

impl<'a> CloseContext<'a> {
    /// # Safety
    ///
    /// Caller guarantees:
    /// - WRITE accounts are not currently borrowed in *any* capacity.
    /// - READ accounts are not currently mutably borrowed.
    ///
    /// ### Accounts
    ///   0. `[READ]` Market account
    ///   1. `[READ]` User base token account
    ///   2. `[READ]` User quote token account
    ///   3. `[READ]` Market base token account
    ///   4. `[READ]` Market quote token account
    pub unsafe fn load(accounts: &'a [AccountInfo]) -> Result<CloseContext<'a>, ProgramError> {
        let [user, market_account, base_mint, quote_mint, user_base_ata, user_quote_ata, market_base_ata, market_quote_ata, base_token_program, quote_token_program] =
            accounts
        else {
            return Err(DropsetError::NotEnoughAccountKeys.into());
        };

        // Safety: Scoped borrow of market account data.
        let (market_account, base_mint, quote_mint) = unsafe {
            let market_account = MarketAccountInfo::new(market_account)?;
            let market = market_account.load_unchecked();
            let (base_mint, quote_mint) =
                MintInfo::new_base_and_quote(base_mint, quote_mint, market)?;
            (market_account, base_mint, quote_mint)
        };

        // Safety: Scoped borrows of the various user/market + base/quote token accounts.
        let user_base_ata = TokenAccountInfo::new(user_base_ata, base_mint.info.key(), user.key())?;
        let user_quote_ata =
            TokenAccountInfo::new(user_quote_ata, quote_mint.info.key(), user.key())?;
        let market_base_ata = TokenAccountInfo::new(
            market_base_ata,
            base_mint.info.key(),
            market_account.info().key(),
        )?;
        let market_quote_ata = TokenAccountInfo::new(
            market_quote_ata,
            quote_mint.info.key(),
            market_account.info().key(),
        )?;

        let base_token_program = TokenProgramInfo::new(base_token_program)?;
        let quote_token_program = TokenProgramInfo::new(quote_token_program)?;

        Ok(Self {
            user,
            market_account,
            base_mint,
            quote_mint,
            user_base_ata,
            user_quote_ata,
            market_base_ata,
            market_quote_ata,
            base_token_program,
            quote_token_program,
        })
    }
}
