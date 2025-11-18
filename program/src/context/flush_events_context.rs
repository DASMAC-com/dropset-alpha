//! See [`FlushEventsContext`].

use dropset_interface::instructions::generated_pinocchio::FlushEvents;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
};

use crate::validation::event_authority::EventAuthorityInfo;

/// The account context for the [`dropset_interface::`] instruction.
#[derive(Clone)]
pub struct FlushEventsContext<'a> {
    pub event_authority: EventAuthorityInfo<'a>,
}

impl<'a> FlushEventsContext<'a> {
    #[inline(always)]
    pub fn load(accounts: &'a [AccountInfo]) -> Result<FlushEventsContext<'a>, ProgramError> {
        let FlushEvents { event_authority } = FlushEvents::load_accounts(accounts)?;

        Ok(Self {
            event_authority: EventAuthorityInfo::new(event_authority)?,
        })
    }
}
