use dropset_interface::error::DropsetError;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::validation::{
    market_account_info::MarketAccountInfo, mint_info::MintInfo,
    token_account_info::TokenAccountInfo, token_program_info::TokenProgramInfo,
};

#[derive(Clone)]
pub struct DepositWithdrawContext<'a> {
    pub user: &'a AccountInfo,
    pub market_account: MarketAccountInfo<'a>,
    pub mint: MintInfo<'a>,
    pub user_ata: TokenAccountInfo<'a>,
    pub market_ata: TokenAccountInfo<'a>,
    pub token_program: TokenProgramInfo<'a>,
}

impl<'a> DepositWithdrawContext<'a> {
    /// # Safety
    ///
    /// Caller guarantees the market account info's data isn't actively being borrowed.
    pub unsafe fn load(
        accounts: &'a [AccountInfo],
    ) -> Result<DepositWithdrawContext<'a>, ProgramError> {
        let [user, market_account, mint, user_ata, market_ata, token_program] = accounts else {
            return Err(DropsetError::NotEnoughAccountKeys.into());
        };

        // Safety: Caller guarantees the market account info's data isn't being borrowed.
        let (market_account, mint) = unsafe {
            let market_account = MarketAccountInfo::new(market_account)?;
            let mint = MintInfo::new(mint, &market_account)?;
            (market_account, mint)
        };

        let user_ata = TokenAccountInfo::new(user_ata, mint.info.key(), user.key())?;
        let market_ata =
            TokenAccountInfo::new(market_ata, mint.info.key(), market_account.info().key())?;
        let token_program = TokenProgramInfo::new(token_program)?;

        Ok(Self {
            user,
            market_account,
            mint,
            user_ata,
            market_ata,
            token_program,
        })
    }
}
