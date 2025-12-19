//! See [`MarketOrderContext`].

use crate::context::deposit_withdraw_context::DepositWithdrawContext;

/// The contextual account infos required for a market order are exactly the same as deposit and
/// withdraw, so a type alias is used here.
pub type MarketOrderContext<'a> = DepositWithdrawContext<'a>;
