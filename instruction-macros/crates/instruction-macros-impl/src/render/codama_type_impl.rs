//! Renders the `CodamaType` trait implementation for a `derive(CodamaType)` struct.

use proc_macro2::TokenStream;
use quote::quote;

use super::codama_helpers::{
    codama_traits_path,
    to_camel_case,
};
use crate::parse::parsed_struct::ParsedStruct;

/// Renders a `CodamaType` impl for a struct that returns the actual
/// `StructTypeNode` describing the struct's fields. If a field type doesn't
/// impl `CodamaType`, the generated code produces a compile error.
pub fn render(parsed_struct: ParsedStruct) -> TokenStream {
    let ParsedStruct {
        struct_ident,
        field_names,
        field_types,
    } = &parsed_struct;

    let codama = codama_traits_path();

    let field_nodes: Vec<TokenStream> = field_names
        .iter()
        .zip(field_types.iter())
        .map(|(name, ty)| {
            let camel_name = to_camel_case(&name.to_string());

            quote! {
                #codama::StructFieldTypeNode::new(
                    #camel_name,
                    <#ty as #codama::CodamaType>::type_node(),
                )
            }
        })
        .collect();

    let struct_name_str = to_camel_case(&struct_ident.to_string());

    quote! {
        impl #codama::CodamaType for #struct_ident {
            fn name() -> &'static str { #struct_name_str }
            fn type_node() -> #codama::TypeNode {
                #codama::TypeNode::Struct(#codama::StructTypeNode {
                    fields: ::instruction_macros::codama::_alloc::vec::Vec::from([
                        #(#field_nodes),*
                    ]),
                })
            }
        }
    }
}
