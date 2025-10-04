pub mod event_authority {
    use pinocchio::pubkey::Pubkey;
    use pinocchio_pubkey::pubkey;

    extern crate std;
    use std::*;

    pub const SEEDS: &[&[u8]] = &[b"event_authority", &[BUMP]];

    /// Regenerate with `print_pda` helper below if the program ID changes.
    pub const ID: Pubkey = pubkey!("GXuSQj95RW5HDLtYCAhFFwaqRWRXYfW3RHyfpeqSaY1i");

    /// Regenerate with `print_pda` helper below if the program ID changes.
    pub const BUMP: u8 = 254;

    #[test]
    /// Helper function to print the PDA for easy copy/paste into the const values above.
    pub fn print_pda() {
        // Must use `solana_pubkey` (not `pinocchio_pubkey`) because test is a non-"solana" target.
        use solana_pubkey::Pubkey;
        let program_id = Pubkey::new_from_array(crate::ID);
        let (pda, bump) = Pubkey::find_program_address(&[b"event_authority"], &program_id);
        println!("pda: {:?}\nbump: {}", pda, bump);
    }

    #[test]
    pub fn check_pda() {
        // Must use `solana_pubkey` (not `pinocchio_pubkey`) because test is a non-"solana" target.
        use solana_pubkey::Pubkey;
        let program_id = Pubkey::new_from_array(crate::ID);

        assert_eq!(
            ID,
            Pubkey::create_program_address(SEEDS, &program_id)
                .expect("Should be OK")
                .as_ref()
        );
    }
}

pub mod market {
    use pinocchio::pubkey::{find_program_address, Pubkey};

    pub const MARKET_SEED_STR: &[u8] = b"market";

    pub fn find_market_address(base_mint: &Pubkey, quote_mint: &Pubkey) -> (Pubkey, u8) {
        find_program_address(crate::market_seeds!(base_mint, quote_mint), &crate::ID)
    }
}

#[macro_export]
macro_rules! market_seeds {
    ( $base_mint:expr, $quote_mint:expr ) => {
        &[
            $base_mint.as_ref(),
            $quote_mint.as_ref(),
            $crate::seeds::market::MARKET_SEED_STR,
        ]
    };
}

#[macro_export]
macro_rules! market_seeds_with_bump {
    ( $base_mint:expr, $quote_mint:expr, $bump:expr ) => {
        &[&[
            $base_mint.as_ref(),
            $quote_mint.as_ref(),
            $crate::seeds::market::MARKET_SEED_STR,
            &[$bump],
        ]]
    };
}
