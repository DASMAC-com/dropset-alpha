use instruction_macros::ProgramInstruction;

use crate::error::DropsetError;

pub mod close_seat;
pub mod deposit;
pub mod flush_events;
pub mod register_market;
pub mod withdraw;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, ProgramInstruction)]
#[program_id(crate::program::ID)]
#[cfg_attr(test, derive(strum_macros::FromRepr, strum_macros::EnumIter))]
#[rustfmt::skip]
pub enum InstructionTag {
    CloseSeat,
    Deposit,
    RegisterMarket,
    Withdraw,
    FlushEvents,
}

impl TryFrom<u8> for InstructionTag {
    type Error = DropsetError;

    #[inline(always)]
    fn try_from(tag: u8) -> Result<Self, Self::Error> {
        InstructionTag_try_from_tag!(tag, DropsetError::InvalidInstructionTag)
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::InstructionTag;

    #[test]
    fn test_instruction_tag_from_u8_exhaustive() {
        for variant in InstructionTag::iter() {
            let variant_u8 = variant as u8;
            assert_eq!(
                InstructionTag::from_repr(variant_u8).unwrap(),
                InstructionTag::try_from(variant_u8).unwrap(),
            );
            assert_eq!(InstructionTag::try_from(variant_u8).unwrap(), variant);
        }
    }
}
