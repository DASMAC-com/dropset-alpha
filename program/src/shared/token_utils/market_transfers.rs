use pinocchio::{program_error::ProgramError, ProgramResult};

use crate::{context::deposit_withdraw_context::DepositWithdrawContext, market_signer};

/// Deposits `amount` of token `ctx.mint` from the user to the market account. This does not track
/// or update seat balances.
///
/// # Safety
///
/// Caller guarantees:
/// - WRITE accounts are not currently borrowed in *any* capacity.
/// - READ accounts are not currently mutably borrowed.
///
/// ### Accounts
///   0. `[WRITE]` User token account (source)
///   1. `[WRITE]` Market token account (destination)
///   2. `[READ]` User account (authority)
///   3. `[READ]` Mint account
pub unsafe fn deposit_to_market(
    ctx: &DepositWithdrawContext,
    amount: u64,
) -> Result<u64, ProgramError> {
    if ctx.token_program.is_spl_token {
        pinocchio_token::instructions::Transfer {
            from: ctx.user_ata.info, // WRITE
            to: ctx.market_ata.info, // WRITE
            authority: ctx.user,     // READ
            amount,
        }
        .invoke()?;

        // `spl_token` always transfers the exact amount passed in.
        Ok(amount)
    } else {
        // Safety: Scoped immutable borrow to read the mint account's mint decimals.
        let decimals = unsafe { ctx.mint.get_mint_decimals() }?;

        // Safety: Scoped immutable borrow of the market token account data to get its balance.
        let balance_before = unsafe { ctx.market_ata.get_balance() }?;

        pinocchio_token_2022::instructions::TransferChecked {
            from: ctx.user_ata.info, // WRITE
            to: ctx.market_ata.info, // WRITE
            mint: ctx.mint.info,     // READ
            authority: ctx.user,     // READ
            decimals,
            amount,
            token_program: ctx.token_program.info.key(),
        }
        .invoke()?;

        // Safety: Scoped immutable borrow of the market token account data to get its balance.
        let balance_after = unsafe { ctx.market_ata.get_balance() }?;

        // `spl_token_2022` amount deposited must be checked due to transfer hooks, fees, and other
        // extensions that may intercept a simple transfer and alter the amount transferred.
        let deposited = balance_after
            .checked_sub(balance_before)
            .ok_or(ProgramError::InvalidArgument)?;
        Ok(deposited)
    }
}

/// Withdraws `amount` of token `ctx.mint` from the market account to the user. This does not track
/// or update seat balances.
///
/// # Safety
///
/// Caller guarantees:
/// - WRITE accounts are not currently borrowed in *any* capacity.
/// - READ accounts are not currently mutably borrowed.
///
/// ### Accounts
///   0. `[WRITE]` User token account (destination)
///   1. `[WRITE]` Market token account (source)
///   2. `[READ]`  Market account (authority)
///   3. `[READ]`  Mint account
pub unsafe fn withdraw_from_market(ctx: &DepositWithdrawContext, amount: u64) -> ProgramResult {
    let (base_mint, quote_mint, market_bump) = {
        // Safety: Scoped immutable borrow of the market account.
        let market = unsafe { ctx.market_account.load_unchecked() };
        (
            market.header.base_mint,
            market.header.quote_mint,
            market.header.market_bump,
        )
    };

    if ctx.token_program.is_spl_token {
        pinocchio_token::instructions::Transfer {
            from: ctx.market_ata.info,            // WRITE
            to: ctx.user_ata.info,                // WRITE
            authority: ctx.market_account.info(), // READ
            amount,
        }
        .invoke_signed(&[market_signer!(base_mint, quote_mint, market_bump)])
    } else {
        // Safety: Scoped immutable borrow of mint account data to get the mint decimals.
        let decimals = unsafe { ctx.mint.get_mint_decimals() }?;

        pinocchio_token_2022::instructions::TransferChecked {
            from: ctx.market_ata.info,            // WRITE
            to: ctx.user_ata.info,                // WRITE
            mint: ctx.mint.info,                  // READ
            authority: ctx.market_account.info(), // READ
            amount,
            decimals,
            token_program: ctx.token_program.info.key(),
        }
        .invoke_signed(&[market_signer!(base_mint, quote_mint, market_bump)])
    }
}
