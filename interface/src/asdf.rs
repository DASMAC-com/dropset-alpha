use instruction_macros::ProgramInstruction;

#[repr(u8)]
#[derive(ProgramInstruction)]
#[cfg_attr(test, derive(strum_macros::FromRepr, strum_macros::EnumIter))]
#[program_id(crate::program::ID)]
pub enum TestInstruction {
    #[account(0, signer, name = "event_authority")]
    #[args(sector_index_hint: u32, "A hint indicating which sector the user's seat resides in.")]
    #[args(boolean: bool)]
    MyFavoriteVariant,
}
