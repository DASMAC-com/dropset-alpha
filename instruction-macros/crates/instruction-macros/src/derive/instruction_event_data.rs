//! Derive helper for generating the `try_from_tag` macro, `pack`, and `unpack` functions for
//! instruction event data.
//!
//! Notably, the structs for these instruction event data types do *not* implement invoke methods,
//! since they are solely for emitting event data inside a self-CPI instruction.

use instruction_macros_impl::{
    parse::{
        instruction_variant::parse_instruction_variants,
        parsed_enum::ParsedEnum,
    },
    render::{
        render_instruction_data,
        render_pack_into_slice_trait,
        render_try_from_tag_macro,
        render_unpack_trait,
    },
};
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub struct DeriveInstructionEventData {
    pub try_from_u8_macro: TokenStream,
    pub pack_into_slice_trait: TokenStream,
    pub unpack_trait: TokenStream,
    pub instruction_data: TokenStream,
}

pub fn derive_instruction_event_data(
    input: DeriveInput,
) -> syn::Result<DeriveInstructionEventData> {
    let parsed_enum = ParsedEnum::try_from((true, input))?;
    let instruction_variants = parse_instruction_variants(&parsed_enum)?;

    let try_from_u8_macro = render_try_from_tag_macro(&parsed_enum, &instruction_variants);
    let instruction_data: TokenStream = render_instruction_data(&parsed_enum, instruction_variants);
    let pack_into_slice_trait = render_pack_into_slice_trait();
    let unpack_trait = render_unpack_trait();

    Ok(DeriveInstructionEventData {
        try_from_u8_macro,
        pack_into_slice_trait,
        unpack_trait,
        instruction_data,
    })
}
