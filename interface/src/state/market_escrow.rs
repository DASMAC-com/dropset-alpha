use pinocchio::pubkey::Pubkey;
use static_assertions::const_assert_eq;

use crate::state::{node::NODE_PAYLOAD_SIZE, U64_SIZE};

#[repr(C)]
pub struct MarketEscrow {
    pub trader: Pubkey,
    base: [u8; U64_SIZE],
    quote: [u8; U64_SIZE],
}

const_assert_eq!(core::mem::size_of::<MarketEscrow>(), NODE_PAYLOAD_SIZE);
