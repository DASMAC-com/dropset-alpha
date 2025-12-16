//! See [`process_batch`].

use pinocchio::{
    account_info::AccountInfo,
    ProgramResult,
};

/// Handler logic for executing multiple instructions in a single atomic batch.
///
/// # Safety
///
/// Since the accounts borrowed depend on the inner batch instructions, the most straightforward
/// safety contract is simply ensuring that **no Solana account data is currently borrowed** prior
/// to calling this instruction.
#[inline(never)]
pub fn process_batch(_accounts: &[AccountInfo], _instruction_data: &[u8]) -> ProgramResult {
    Ok(())
}
