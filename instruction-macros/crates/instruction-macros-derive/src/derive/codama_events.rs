//! Derive helper for [`CodamaProgram`] on event enums.
//!
//! Identical to `derive_codama_program` but parses the enum as instruction events
//! (`as_instruction_events = true`) so that the "no accounts" validation passes.

use instruction_macros_impl::{
    parse::{
        instruction_variant::parse_instruction_variants,
        parsed_enum::ParsedEnum,
    },
    render::codama_program_impl,
};
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn derive_codama_events(input: DeriveInput) -> syn::Result<TokenStream> {
    let parsed_enum = ParsedEnum::new(input, true)?;
    let instruction_variants = parse_instruction_variants(&parsed_enum)?;

    Ok(codama_program_impl::render(
        &parsed_enum,
        &instruction_variants,
    ))
}
