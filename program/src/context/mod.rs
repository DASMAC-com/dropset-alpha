//! Account context definitions for each `dropset` instruction.
//!
//! Each context groups and validates the accounts required by its corresponding instruction before
//! execution.

pub mod close_seat_context;
pub mod deposit_withdraw_context;
pub mod flush_events_context;
pub mod register_market_context;
