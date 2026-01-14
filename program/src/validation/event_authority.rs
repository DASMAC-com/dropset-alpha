//! See [`EventAuthorityInfo`].

use dropset_interface::{
    error::DropsetError,
    seeds::event_authority,
};
use pinocchio::account::AccountView;

/// A validated wrapper around a raw market [`AccountView`] for the event authority account.
#[derive(Clone)]
pub struct EventAuthorityView<'a> {
    pub _account: &'a AccountView,
}

impl<'a> EventAuthorityView<'a> {
    #[inline(always)]
    pub fn new(event_authority_account: &'a AccountView) -> Result<Self, DropsetError> {
        if event_authority_account.address() != &event_authority::ID {
            return Err(DropsetError::IncorrectEventAuthority);
        }

        if !event_authority_account.is_signer() {
            return Err(DropsetError::EventAuthorityMustBeSigner);
        }

        Ok(Self {
            _account: event_authority_account,
        })
    }
}
