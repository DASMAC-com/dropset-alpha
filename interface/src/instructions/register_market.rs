use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

use crate::{
    instructions::{shared::num_sectors::NumSectorsInstructionData, InstructionTag},
    pack::{AsSlice, Pack, UNINIT_BYTE},
    pack_with_tag,
    state::transmutable::Transmutable,
};

pub struct RegisterMarket<'a, 'b> {
    /// The user/payer registering the market.
    pub user: &'a AccountInfo,
    /// The market account, a PDA derived from the base mint + quote mint as seeds.
    pub market_account: &'a AccountInfo,
    /// The base mint account.
    pub base_mint: &'a AccountInfo,
    /// The quote mint account.
    pub quote_mint: &'a AccountInfo,
    /// The market's associated token account for the base mint.
    pub base_market_ata: &'a AccountInfo,
    /// The market's associated token account for the quote mint.
    pub quote_market_ata: &'a AccountInfo,
    /// The base mint's token program.
    pub base_token_program: &'a AccountInfo,
    /// The quote mint's token program.
    pub quote_token_program: &'a AccountInfo,
    /// The system program.
    pub system_program: &'a AccountInfo,
    /// Instruction data.
    pub instruction_data: &'b NumSectorsInstructionData,
}

/// Registers a program-owned market account derived from the base mint and quote mint pubkeys.
///
/// Allocates the passed in number of sectors * SECTOR_SIZE bytes as extra initial account space.
///
///   0. `[WRITE, SIGNER]` User account
///   1. `[WRITE]` Market account
///   2. `[WRITE]` Market base mint token account
///   3. `[WRITE]` Market quote mint token account
///   4. `[READ]` Base mint
///   5. `[READ]` Quote mint
///   6. `[READ]` System program
///   7. `[READ]` Token program
impl<'a> RegisterMarket<'a, '_> {
    #[inline(always)]
    pub fn create_account_metas(&self) -> [AccountMeta; 9] {
        [
            AccountMeta::writable_signer(self.user.key()),
            AccountMeta::writable(self.market_account.key()),
            AccountMeta::writable(self.base_market_ata.key()),
            AccountMeta::writable(self.quote_market_ata.key()),
            AccountMeta::readonly(self.base_mint.key()),
            AccountMeta::readonly(self.quote_mint.key()),
            AccountMeta::readonly(self.system_program.key()),
            AccountMeta::readonly(self.base_token_program.key()),
            AccountMeta::readonly(self.quote_token_program.key()),
        ]
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[Signer]) -> ProgramResult {
        invoke_signed(
            &Instruction {
                program_id: &crate::program::ID,
                accounts: &self.create_account_metas(),
                // data: &tagged_data,
            },
            &[
                self.user,
                self.market_account,
                self.base_mint,
                self.quote_mint,
                self.base_market_ata,
                self.quote_market_ata,
                self.base_token_program,
                self.quote_token_program,
                self.system_program,
            ],
            signers_seeds,
        )
    }
}
