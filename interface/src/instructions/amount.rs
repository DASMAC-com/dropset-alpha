use static_assertions::const_assert_eq;

use crate::{
    pack::{write_bytes, Pack},
    state::{
        sector::{NonNilSectorIndex, SectorIndex},
        transmutable::Transmutable,
        U32_SIZE, U64_SIZE,
    },
};
use core::mem::MaybeUninit;

#[repr(C)]
pub struct AmountInstructionData {
    /// The amount to deposit or withdraw.
    amount: [u8; U64_SIZE],
    /// A hint as to which sector index the calling user is located in the sectors array.
    /// The getter for this field exposes it as an Option<NonNilSectorIndex>, where `NIL` as the
    /// hint is equivalent to None.
    sector_index_hint: [u8; U32_SIZE],
}

impl AmountInstructionData {
    /// NIL as the sector index hint is the semantic equivalent of None here.
    pub fn new(amount: u64, sector_index_hint: SectorIndex) -> Self {
        AmountInstructionData {
            amount: amount.to_le_bytes(),
            sector_index_hint: sector_index_hint.into(),
        }
    }

    #[inline(always)]
    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    #[inline(always)]
    pub fn sector_index_hint(&self) -> Option<NonNilSectorIndex> {
        NonNilSectorIndex::new(self.sector_index_hint.into()).ok()
    }
}

impl Pack<12> for AmountInstructionData {
    fn pack_into_slice(&self, dst: &mut [MaybeUninit<u8>; 12]) {
        write_bytes(&mut dst[0..8], &self.amount);
        write_bytes(&mut dst[8..12], &self.sector_index_hint);
    }
}

unsafe impl Transmutable for AmountInstructionData {
    const LEN: usize = 12;
}

const_assert_eq!(
    AmountInstructionData::LEN,
    size_of::<AmountInstructionData>()
);
