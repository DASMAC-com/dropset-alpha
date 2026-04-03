//! Derive helper for the `CodamaType` trait.

use instruction_macros_impl::{
    parse::parsed_struct::ParsedStruct,
    render::render_codama_type_impl,
};
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn derive_codama_type(input: DeriveInput) -> syn::Result<TokenStream> {
    let parsed_struct = ParsedStruct::new(input)?;

    Ok(render_codama_type_impl(parsed_struct))
}
