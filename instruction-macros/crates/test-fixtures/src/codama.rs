//! Test fixture for `CodamaType` and `CodamaProgram` derives.
//! Run `cargo expand --features codama` to see the generated output.

use instruction_macros::{
    CodamaProgram,
    CodamaType,
    Pack,
    ProgramInstruction,
    Unpack,
};
use solana_address::Address;

#[repr(C)]
#[derive(Debug, Clone, Pack, Unpack, PartialEq, Eq, CodamaType)]
pub struct SimpleArgs {
    pub amount: u64,
    pub flag: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Pack, Unpack, PartialEq, Eq, CodamaType)]
pub struct NestedArgs {
    pub value: u32,
    pub user: Address,
    pub inner: SimpleArgs,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, ProgramInstruction, CodamaProgram)]
#[program_id(crate::ID)]
pub enum CodamaTestInstruction {
    #[account(0, signer, name = "user", desc = "The user.")]
    #[account(1, writable, name = "target", desc = "The target account.")]
    #[args(amount: u64, "The amount.")]
    #[args(flag: bool, "A flag.")]
    SimpleIxn,

    #[account(0, signer, name = "authority", desc = "The authority.")]
    #[args(nested: NestedArgs, "Nested argument data.")]
    NestedIxn,
}
