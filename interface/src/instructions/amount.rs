use crate::{
    pack::{write_bytes, Pack},
    state::{transmutable::Transmutable, U64_SIZE},
};
use core::mem::MaybeUninit;

#[repr(C)]
pub struct AmountInstructionData {
    amount: [u8; U64_SIZE],
}

impl AmountInstructionData {
    pub fn new(amount: u64) -> Self {
        AmountInstructionData {
            amount: amount.to_le_bytes(),
        }
    }

    #[inline(always)]
    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }
}

impl Pack<8> for AmountInstructionData {
    fn pack_into_slice(&self, dst: &mut [MaybeUninit<u8>; 8]) {
        write_bytes(dst, &self.amount);
    }
}

unsafe impl Transmutable for AmountInstructionData {
    const LEN: usize = 8;
}
