use instruction_macros::ProgramInstructions;
use pinocchio::{
    instruction::{Instruction, Signer},
    ProgramResult,
};

use crate::{
    instructions::InstructionTag,
    pack::{write_bytes, UNINIT_BYTE},
    state::sector::SectorIndex,
};

pub use pinocchio::account_info::AccountInfo;

use pinocchio::instruction::AccountMeta;

use pinocchio::program_error::ProgramError;

#[derive(ProgramInstructions)]
#[instruction_tag(DropsetInstructionTag, ProgramError::InvalidInstructionData)]
#[repr(u8)]
#[rustfmt::skip]
pub enum DropsetInstruction {
    #[account(0, signer,   name = "user",             desc = "The user closing their seat.")]
    #[account(1, writable, name = "market_account",   desc = "The market account PDA.")]
    #[account(2, writable, name = "base_user_ata",    desc = "The user's associated base mint token account.")]
    #[account(3, writable, name = "quote_user_ata",   desc = "The user's associated quote mint token account.")]
    #[account(4, writable, name = "base_market_ata",  desc = "The market's associated base mint token account.")]
    #[account(5, writable, name = "quote_market_ata", desc = "The market's associated quote mint token account.")]
    #[account(6,           name = "base_mint",        desc = "The base token mint account.")]
    #[account(7,           name = "quote_mint",       desc = "The quote token mint account.")]
    #[args(sector_index_hint: u32, "A hint indicating which sector the user's seat resides in.")]
    CloseSeat_,

    #[account(0, signer,   name = "user",           desc = "The user depositing or registering their seat.")]
    #[account(1, writable, name = "market_account", desc = "The market account PDA.")]
    #[account(2, writable, name = "user_ata",       desc = "The user's associated token account.")]
    #[account(3, writable, name = "market_ata",     desc = "The market's associated token account.")]
    #[account(4,           name = "mint",           desc = "The token mint account.")]
    #[args(amount: u64, "The amount to deposit.")]
    #[args(sector_index_hint: u32, "A hint indicating which sector the user's seat resides in (pass `NIL` when registering a new seat).")]
    Deposit_,

    #[account(0, signer, writable, name = "user",        desc = "The user registering the market.")]
    #[account(1, writable, name = "market_account",      desc = "The market account PDA.")]
    #[account(2, writable, name = "base_market_ata",     desc = "The market's associated token account for the base mint.")]
    #[account(3, writable, name = "quote_market_ata",    desc = "The market's associated token account for the quote mint.")]
    #[account(4,           name = "base_mint",           desc = "The base mint account.")]
    #[account(5,           name = "quote_mint",          desc = "The quote mint account.")]
    #[account(6,           name = "base_token_program",  desc = "The base mint's token program.")]
    #[account(7,           name = "quote_token_program", desc = "The quote mint's token program.")]
    #[account(8,           name = "system_program",      desc = "The system program.")]
    #[args(num_sectors: u16, "The number of sectors to preallocate for the market.")]
    RegisterMarket_,

    #[account(0, signer,   name = "user",           desc = "The user withdrawing.")]
    #[account(1, writable, name = "market_account", desc = "The market account PDA.")]
    #[account(2, writable, name = "user_ata",       desc = "The user's associated token account.")]
    #[account(3, writable, name = "market_ata",     desc = "The market's associated token account.")]
    #[account(4,           name = "mint",           desc = "The token mint account.")]
    #[args(amount: u64, "The amount to withdraw.")]
    #[args(sector_index_hint: u32, "A hint indicating which sector the user's seat resides in.")]
    Withdraw_,

    #[account(0, signer, name = "hello!")]
    #[args(amount: u32, "the amt")]
    MyFavoriteInstruction,

    #[account(0, signer, name = "asdf")]
    WellWellWell = 100,
    
    #[account(0, signer, name = "asdf")]
    WellWellWell2,

    #[account(0, signer, name = "asdf")]
    Well3 = 150,
    
    #[account(0, signer, name = "asdf")]
    WellWellWell3 = 51,
    
    #[account(0, signer, name = "asdf")]
    WellWellWell4,
}

/// Closes a market seat for a user by withdrawing all base and quote from their seat.
///
/// # Caller guarantees
///
/// When invoking this instruction, caller must ensure that:
/// - WRITE accounts are not currently borrowed in *any* capacity.
/// - READ accounts are not currently mutably borrowed.
///
/// ### Accounts
///   0. `[READ, SIGNER]` User
///   1. `[WRITE]` Market account
///   2. `[WRITE]` User base mint token account
///   3. `[WRITE]` User quote mint token account
///   4. `[WRITE]` Market base mint token account
///   5. `[WRITE]` Market quote mint token account
///   6. `[READ]` Base mint
///   7. `[READ]` Quote mint
pub struct CloseSeat<'a> {
    /// The user closing their seat.
    pub user: &'a AccountInfo,
    /// The market account PDA.
    pub market_account: &'a AccountInfo,
    /// The user's associated base mint token account.
    pub base_user_ata: &'a AccountInfo,
    /// The user's associated quote mint token account.
    pub quote_user_ata: &'a AccountInfo,
    /// The market's associated base mint token account.
    pub base_market_ata: &'a AccountInfo,
    /// The market's associated quote mint token account.
    pub quote_market_ata: &'a AccountInfo,
    /// The base token mint account.
    pub base_mint: &'a AccountInfo,
    /// The quote token mint account.
    pub quote_mint: &'a AccountInfo,
    /// A hint indicating which sector index the user's seat is at in the sectors array.
    pub sector_index_hint: SectorIndex,
}

impl CloseSeat<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[Signer]) -> ProgramResult {
        pinocchio::cpi::invoke_signed(
            &Instruction {
                program_id: &crate::program::ID,
                accounts: &[
                    AccountMeta::readonly_signer(self.user.key()),
                    AccountMeta::writable(self.market_account.key()),
                    AccountMeta::writable(self.base_user_ata.key()),
                    AccountMeta::writable(self.quote_user_ata.key()),
                    AccountMeta::writable(self.base_market_ata.key()),
                    AccountMeta::writable(self.quote_market_ata.key()),
                    AccountMeta::readonly(self.base_mint.key()),
                    AccountMeta::readonly(self.quote_mint.key()),
                ],
                data: &self.pack(),
            },
            &[
                self.user,
                self.market_account,
                self.base_user_ata,
                self.quote_user_ata,
                self.base_market_ata,
                self.quote_market_ata,
                self.base_mint,
                self.quote_mint,
            ],
            signers_seeds,
        )
    }

    // #[cfg(feature = "client")]
    // pub fn create_account_metas(&self) -> [AccountMeta; 8] {
    //     [
    //         AccountMeta::new_readonly((*self.user).into(), true),
    //         AccountMeta::new((*self.market_account).into(), false),
    //         AccountMeta::new((*self.base_user_ata).into(), false),
    //         AccountMeta::new((*self.quote_user_ata).into(), false),
    //         AccountMeta::new((*self.base_market_ata).into(), false),
    //         AccountMeta::new((*self.quote_market_ata).into(), false),
    //         AccountMeta::new_readonly((*self.base_mint).into(), false),
    //         AccountMeta::new_readonly((*self.quote_mint).into(), false),
    //     ]
    // }

    #[inline(always)]
    pub fn pack(&self) -> [u8; 5] {
        // Instruction data layout:
        //   - [0]: the instruction tag, 1 byte
        //   - [1..5]: the u32 `sector_index_hint` as little-endian bytes, 4 bytes
        let mut data = [UNINIT_BYTE; 5];

        data[0].write(InstructionTag::CloseSeat as u8);
        write_bytes(&mut data[1..5], &self.sector_index_hint.0.to_le_bytes());

        // Safety: All 5 bytes were written to.
        unsafe { *(data.as_ptr() as *const _) }
    }
}
