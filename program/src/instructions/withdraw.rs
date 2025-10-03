use dropset_interface::{instructions::amount::AmountInstructionData, state::transmutable::load};
use pinocchio::{account_info::AccountInfo, ProgramResult};

pub fn process_withdraw(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // Safety: All bit patterns are valid.
    let amount = unsafe { load::<AmountInstructionData>(instruction_data) }?.amount();

    Ok(())
}
