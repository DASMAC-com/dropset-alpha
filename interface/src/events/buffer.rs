use core::mem::{
    offset_of,
    MaybeUninit,
};

use pinocchio::pubkey::Pubkey;
use static_assertions::const_assert_eq;

use crate::{
    events::HeaderInstructionData,
    instructions::DropsetInstruction,
    state::transmutable::Transmutable,
};

pub const MAX_CPI_INSTRUCTION_DATA_LEN: usize = 10 * 1024;

pub struct EventBuffer {
    /// The stack-allocated, possibly initialized buffer bytes.
    ///
    /// The layout for the data is:
    /// - [0]: the instruction tag of the instruction that created this event buffer.
    /// - [1..HeaderInstructionData::LEN_WITH_TAG]: the header instruction data.
    /// - [HeaderInstructionData::LEN_WITH_TAG..]: the byte data for the other non-header events in
    ///   the buffer.
    pub data: [MaybeUninit<u8>; MAX_CPI_INSTRUCTION_DATA_LEN],
    /// The amount of initialized bytes. The index at `len` is the first uninitialized byte.
    pub len: usize,
}

const EMITTED_COUNT_OFFSET: usize = offset_of!(HeaderInstructionData, emitted_count);
const EMITTED_COUNT_SIZE: usize = size_of::<u16>();
const NONCE_OFFSET: usize = offset_of!(HeaderInstructionData, nonce);
const NONCE_SIZE: usize = size_of::<u64>();

impl EventBuffer {
    pub fn new(instruction_tag: DropsetInstruction, market: Pubkey) -> Self {
        let mut data = [MaybeUninit::uninit(); MAX_CPI_INSTRUCTION_DATA_LEN];
        // Manually pack the instruction tag for the CPI invocation.
        data[0].write(DropsetInstruction::FlushEvents as u8);
        let mut len = 1;
        // Then pack the event header.
        let header = HeaderInstructionData::new(instruction_tag as u8, 0, 0, market);
        // HeaderInstructionData::pack(&self)

        // Safety: data's length is sufficient and `len` increments by the header's length below.
        unsafe { header.pack_into_slice(&mut data, len) };

        len += HeaderInstructionData::LEN_WITH_TAG;

        debug_assert_eq!(
            len,
            size_of::<DropsetInstruction>() + HeaderInstructionData::LEN_WITH_TAG
        );

        Self { data, len }
    }

    pub fn increment_emitted_count(&mut self) {
        // Safety:
        // The first 1 + `HeaderInstructionData::LEN_WITH_TAG` bytes are always initialized.
        // No other reference to this data is currently held.
        unsafe {
            let emitted_count_slice = self
                .data
                .as_mut_ptr()
                // The first byte is the `FlushEvents` tag.
                .add(size_of::<DropsetInstruction>() + EMITTED_COUNT_OFFSET)
                as *mut [u8; EMITTED_COUNT_SIZE];
            let emitted_count = u16::from_le_bytes(*emitted_count_slice);
            let incremented = emitted_count + 1;
            core::ptr::copy_nonoverlapping(
                incremented.to_le_bytes().as_ptr(),
                emitted_count_slice as _,
                EMITTED_COUNT_SIZE,
            );
        };
    }

    pub fn increment_nonce(&mut self) {
        // Safety:
        // The first 1 + `HeaderInstructionData::LEN_WITH_TAG` bytes are always initialized.
        // No other reference to this data is currently held.
        unsafe {
            let nonce_slice = self
                .data
                .as_mut_ptr()
                // The first byte is the `FlushEvents` tag.
                .add(size_of::<DropsetInstruction>() + NONCE_OFFSET)
                as *mut [u8; NONCE_SIZE];
            let emitted_count = u64::from_le_bytes(*nonce_slice);
            let incremented = emitted_count + 1;
            core::ptr::copy_nonoverlapping(
                incremented.to_le_bytes().as_ptr(),
                nonce_slice as _,
                NONCE_SIZE,
            );
        };
    }
}

// Ensure `emitted_count` and `nonce` are the expected types and size.
const _: () = {
    fn assert_types(val: &HeaderInstructionData) {
        let _: &u16 = &val.emitted_count;
        let _: &u64 = &val.nonce;
        let _: [u8; EMITTED_COUNT_SIZE] = [0; size_of::<u16>()];
        let _: [u8; NONCE_SIZE] = [0; size_of::<u64>()];
    }
};
