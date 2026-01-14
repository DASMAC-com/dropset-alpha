//! Defines PDA seed constants for deriving PDA addresses.

/// PDA constants and helpers for a derived market address.
pub mod market {
    pub const MARKET_SEED_STR: &[u8] = b"market";
}

/// PDA constants and helpers for the derived event authority address.
pub mod event_authority {
    use pinocchio::Address;

    pub const EVENT_AUTHORITY_SEED_STR: &[u8] = b"event_authority";

    /// Regenerate with `print_pda` helper below if the program ID changes.
    pub const ID: Address = Address::from_str_const("GXuSQj95RW5HDLtYCAhFFwaqRWRXYfW3RHyfpeqSaY1i");

    /// Regenerate with `print_pda` helper below if the program ID changes.
    pub const BUMP: u8 = 254;

    #[cfg(test)]
    mod tests {
        use super::*;

        extern crate std;
        use std::*;

        #[test]
        /// Helper function to print the PDA for easy copy/paste into the const values above.
        pub fn print_pda() {
            let (pda, bump) = solana_address::Address::find_program_address(
                &[b"event_authority"],
                &crate::program::ID,
            );
            println!("pda: {pda}\nbump: {bump}");
        }

        #[test]
        pub fn check_pda() {
            assert_eq!(
                ID,
                solana_address::Address::create_program_address(
                    &[EVENT_AUTHORITY_SEED_STR, &[BUMP]],
                    &crate::program::ID
                )
                .expect("Should be OK")
            );
        }
    }
}
