//! Defines the Display-able types for event instruction data. Primarily for things like converting
//! `[u8; 32]` fields to `Pubkey` so that they're displayed as strings instead of arrays.

use dropset_interface::events::{
    HeaderEventInstructionData,
    RegisterMarketEventInstructionData,
};
use solana_address::Address;

#[derive(Debug)]
pub struct DisplayHeaderData {
    pub instruction_tag: u8,
    pub emitted_count: u16,
    pub num_events: u64,
    pub market: Address,
}

impl From<HeaderEventInstructionData> for DisplayHeaderData {
    fn from(value: HeaderEventInstructionData) -> Self {
        Self {
            instruction_tag: value.instruction_tag,
            emitted_count: value.emitted_count,
            num_events: value.num_events,
            market: value.market,
        }
    }
}

#[derive(Debug)]
pub struct DisplayRegisterMarketData {
    pub market: Address,
}

impl From<RegisterMarketEventInstructionData> for DisplayRegisterMarketData {
    fn from(value: RegisterMarketEventInstructionData) -> Self {
        Self {
            market: value.market,
        }
    }
}
