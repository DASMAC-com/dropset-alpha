//! Account context definitions for each `dropset` instruction.
//!
//! Each context groups and validates the accounts required by its corresponding instruction before
//! execution.

pub mod batch_replace_context;
pub mod cancel_order_context;
pub mod close_seat_context;
pub mod deposit_context;
pub mod expand_market_context;
pub mod flush_events_context;
pub mod market_order_context;
pub mod post_order_context;
pub mod register_market_context;
pub mod withdraw_context;

/// The account infos necessary to emit events with the event buffer.
pub struct EventBufferContext<'a> {
    pub event_authority: &'a pinocchio::account::AccountView,
    pub market_account: crate::validation::market_account_view::MarketAccountView<'a>,
}
