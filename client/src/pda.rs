//! PDA helpers for deriving `dropset` program addresses.

use solana_address::Address;

pub fn find_market_address(base_mint: &Address, quote_mint: &Address) -> (Address, u8) {
    Address::find_program_address(
        &[
            base_mint.as_ref(),
            quote_mint.as_ref(),
            dropset_interface::seeds::market::MARKET_SEED_STR,
        ],
        &dropset::ID,
    )
}
