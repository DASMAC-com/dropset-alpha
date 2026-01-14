//! See [`UninitializedAccountView`].

use dropset_interface::{
    error::DropsetError,
    state::SYSTEM_PROGRAM_ID,
    utils::owned_by,
};
use pinocchio::account::AccountView;

/// A validated wrapper around a raw [`AccountView`] expected to be uninitialized, confirming it is
/// writable, rent-exempt, and ready for allocation.
#[derive(Clone)]
pub struct UninitializedAccountView<'a> {
    pub account: &'a AccountView,
}

impl<'a> UninitializedAccountView<'a> {
    #[inline(always)]
    pub fn new(account: &'a AccountView) -> Result<UninitializedAccountView<'a>, DropsetError> {
        if !account.is_data_empty() {
            return Err(DropsetError::AlreadyInitializedAccount);
        }

        if !owned_by(account, &SYSTEM_PROGRAM_ID) {
            return Err(DropsetError::NotOwnedBySystemProgram);
        }

        Ok(Self { account })
    }
}
