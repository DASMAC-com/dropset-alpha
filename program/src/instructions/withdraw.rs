use dropset_interface::{
    error::DropsetError, instructions::amount::AmountInstructionData, state::transmutable::load,
};
use pinocchio::{account_info::AccountInfo, ProgramResult};

use crate::{
    context::deposit_withdraw_context::DepositWithdrawContext,
    shared::{
        market_operations::find_mut_seat_with_hint,
        token_utils::market_transfers::withdraw_from_market,
    },
};

pub fn process_withdraw(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // Safety: All bit patterns are valid.
    let args = unsafe { load::<AmountInstructionData>(instruction_data) }?;

    // The amount withdrawn is always just the input amount.
    let amount_withdrawn = args.amount();
    if amount_withdrawn == 0 {
        return Err(DropsetError::AmountCannotBeZero.into());
    }

    let ctx = DepositWithdrawContext::load(accounts)?;
    withdraw_from_market(&ctx, args.amount())?;

    // User must provide a valid, non-NIL sector index hint.
    let hint = args
        .sector_index_hint()
        .ok_or(DropsetError::InvalidIndexHint)?;

    // Safety: Single mutable borrow of market account data.
    let market = unsafe { ctx.market_account.load_unchecked_mut() }?;
    let seat = find_mut_seat_with_hint(market, hint, ctx.user.key())?;

    // Update the market seat available/deposited, checking for underflow, as that means the user
    // tried to withdraw more than they have available.
    if ctx.mint.is_base_mint {
        seat.set_base_available(
            seat.base_available()
                .checked_sub(amount_withdrawn)
                .ok_or(DropsetError::InsufficientUserBalance)?,
        );
        seat.set_base_deposited(
            seat.base_deposited()
                .checked_sub(amount_withdrawn)
                .ok_or(DropsetError::InsufficientUserBalance)?,
        );
    } else {
        seat.set_quote_available(
            seat.quote_available()
                .checked_sub(amount_withdrawn)
                .ok_or(DropsetError::InsufficientUserBalance)?,
        );
        seat.set_quote_deposited(
            seat.quote_deposited()
                .checked_sub(amount_withdrawn)
                .ok_or(DropsetError::InsufficientUserBalance)?,
        );
    }

    Ok(())
}
