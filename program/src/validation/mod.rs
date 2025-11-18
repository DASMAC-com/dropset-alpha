//! Validated wrapper structs for converting raw [`pinocchio::account_info::AccountInfo`] inputs
//! into strongly typed, context-aware account representations used by `dropset` instructions.

pub mod event_authority;
pub mod market_account_info;
pub mod mint_info;
pub mod token_account_info;
pub mod uninitialized_account_info;
