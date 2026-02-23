//! See [`process_expand_market`].

use dropset_interface::{
    error::DropsetError,
    instructions::ExpandMarketInstructionData,
};
use pinocchio::{
    account::AccountView,
    error::ProgramError,
};

use crate::{
    context::{
        expand_market_context::ExpandMarketContext,
        EventBufferContext,
    },
    events::EventBuffer,
};

/// Instruction handler logic for increasing the size of a market account's data.
///
/// # Safety
///
/// Caller upholds the safety contract detailed in
/// [`dropset_interface::instructions::generated_program::ExpandMarket`].
#[inline(never)]
pub unsafe fn process_expand_market<'a>(
    accounts: &'a [AccountView],
    instruction_data: &[u8],
    event_buffer: &mut EventBuffer,
) -> Result<EventBufferContext<'a>, ProgramError> {
    let ExpandMarketInstructionData { num_sectors } =
        ExpandMarketInstructionData::unpack_untagged(instruction_data)?;

    if num_sectors == 0 {
        return Err(DropsetError::NumSectorsCannotBeZero.into());
    }

    // Safety: No account data in `accounts` is currently borrowed.
    let mut ctx = unsafe { ExpandMarketContext::load(accounts) }?;

    // Safety: Scoped mutable borrow to resize the market account and add new sectors.
    unsafe { ctx.market_account.resize(ctx.payer, num_sectors) }?;

    event_buffer.add_to_buffer(
        ExpandMarketInstructionData::new(num_sectors),
        ctx.event_authority,
        ctx.market_account.clone(),
    )?;

    Ok(EventBufferContext {
        event_authority: ctx.event_authority,
        market_account: ctx.market_account,
    })
}
