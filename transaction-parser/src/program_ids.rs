//! Exports common program ID addresses.

use solana_address::Address;

/// The SPL Token program ID/address.
pub const SPL_TOKEN_ID: Address =
    Address::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
/// The SPL Token 2022 program ID/address.
pub const SPL_TOKEN_2022_ID: Address =
    Address::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
/// The SPL Associated Token Account program ID/address.
pub const SPL_ASSOCIATED_TOKEN_ACCOUNT_ID: Address =
    Address::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
/// The Solana Compute Budget program ID/address.
pub const COMPUTE_BUDGET_ID: Address =
    Address::from_str_const("ComputeBudget111111111111111111111111111111");
