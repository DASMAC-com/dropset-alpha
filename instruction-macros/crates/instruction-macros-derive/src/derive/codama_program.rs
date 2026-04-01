//! Derive helper for the [`CodamaProgram`] trait on instruction enums.

use instruction_macros_impl::{
    parse::{
        instruction_variant::parse_instruction_variants,
        parsed_enum::ParsedEnum,
    },
    render::codama_program_impl,
};
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn derive_codama_program(input: DeriveInput) -> syn::Result<TokenStream> {
    let parsed_enum = ParsedEnum::new(input, false)?;
    let instruction_variants = parse_instruction_variants(&parsed_enum)?;

    Ok(codama_program_impl::render(
        &parsed_enum,
        &instruction_variants,
    ))
}