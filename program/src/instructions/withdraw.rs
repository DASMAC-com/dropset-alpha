use dropset_interface::{
    error::DropsetError, pack::unpack_amount_and_sector_index, state::node::Node,
};
use pinocchio::{account_info::AccountInfo, ProgramResult};

use crate::{
    context::deposit_withdraw_context::DepositWithdrawContext,
    shared::{
        market_operations::find_mut_seat_with_hint,
        token_utils::market_transfers::withdraw_non_zero_from_market,
    },
};

/// # Safety
///
/// Caller guarantees:
/// - WRITE accounts are not currently borrowed in *any* capacity.
/// - READ accounts are not currently mutably borrowed.
///
/// ### Accounts
///   0. `[WRITE]` User token account (destination)
///   1. `[WRITE]` Market token account (source)
///   2. `[WRITE]` Market account
///   3. `[READ]`  Mint account
pub unsafe fn process_withdraw(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let (amount, hint) = unpack_amount_and_sector_index(instruction_data)?;

    // Safety: Scoped immutable borrow of market, user token, and market token accounts to validate.
    let mut ctx = unsafe { DepositWithdrawContext::load(accounts) }?;
    unsafe { withdraw_non_zero_from_market(&ctx, amount) }?;

    // Safety: Scoped mutable borrow of market account data to update the user's seat.
    let market = unsafe { ctx.market_account.load_unchecked_mut() };

    // User has provided a sector index hint; find the seat with it or fail and return early.
    Node::check_in_bounds(market.sectors, hint)?;
    // Safety: The hint was just verified as in-bounds.
    let seat = unsafe { find_mut_seat_with_hint(market, hint, ctx.user.key()) }?;

    // Update the market seat available/deposited, checking for underflow, as that means the user
    // tried to withdraw more than they have available.
    if ctx.mint.is_base_mint {
        seat.set_base_available(
            seat.base_available()
                .checked_sub(amount)
                .ok_or(DropsetError::InsufficientUserBalance)?,
        );
        seat.set_base_deposited(
            seat.base_deposited()
                .checked_sub(amount)
                .ok_or(DropsetError::InsufficientUserBalance)?,
        );
    } else {
        seat.set_quote_available(
            seat.quote_available()
                .checked_sub(amount)
                .ok_or(DropsetError::InsufficientUserBalance)?,
        );
        seat.set_quote_deposited(
            seat.quote_deposited()
                .checked_sub(amount)
                .ok_or(DropsetError::InsufficientUserBalance)?,
        );
    }

    Ok(())
}
