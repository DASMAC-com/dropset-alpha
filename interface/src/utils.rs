//! Lightweight helper functions and constants used throughout the interface and state modules.

use pinocchio::account::AccountView;
use solana_address::{
    address_eq,
    Address,
};

#[inline(always)]
pub fn owned_by(account: &AccountView, potential_owner: &Address) -> bool {
    // Safety: Scoped borrow of account owner.
    let owner = unsafe { account.owner() };
    address_eq(owner, potential_owner)
}

/// Checks if an account is owned by the `spl_token::ID`; i.e., not `spl_token_2022::ID`.
///
/// Note that this in and of itself isn't sufficient proof of a valid, initialized mint account.
/// You must either check that the account's data length is > 0 or indirectly validate it by calling
/// the program with the mint account.
#[inline(always)]
pub fn is_owned_by_spl_token(account: &AccountView) -> bool {
    owned_by(account, &pinocchio_token::ID)
}
