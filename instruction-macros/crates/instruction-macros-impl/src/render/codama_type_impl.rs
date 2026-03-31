//! Renders the `CodamaType` trait implementation for a `derive(CodamaType)` struct.

use proc_macro2::TokenStream;
use quote::quote;

use super::codama_helpers::{
    codama_traits_path,
    to_camel_case,
};
use crate::parse::parsed_struct::ParsedStruct;

/// Renders a `CodamaType` impl for a struct. Each field's type node is obtained by calling
/// `CodamaType::codama_type_node()` on the field type. If a field type doesn't impl
/// `CodamaType`, the generated code produces a compile error.
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
                    ::instruction_macros::codama::_alloc::string::String::from(#camel_name),
                    <#ty as #codama::CodamaType>::codama_type_node(),
                )
            }
        })
        .collect();

    let struct_name_str = to_camel_case(&struct_ident.to_string());

    quote! {
        #[cfg(feature = "codama")]
        impl #codama::CodamaType for #struct_ident {
            fn codama_type_node() -> #codama::TypeNode {
                #codama::defined_type_link(#struct_name_str)
            }
        }

        #[cfg(feature = "codama")]
        impl #struct_ident {
            /// Returns the full struct type node for this type's IDL definition.
            pub fn codama_defined_type() -> #codama::DefinedTypeNode {
                #codama::DefinedTypeNode {
                    kind: "definedTypeNode",
                    name: ::instruction_macros::codama::_alloc::string::String::from(#struct_name_str),
                    docs: ::instruction_macros::codama::_alloc::vec::Vec::new(),
                    ty: #codama::TypeNode::Struct(#codama::StructTypeNode {
                        fields: ::instruction_macros::codama::_alloc::vec![
                            #(#field_nodes),*
                        ],
                    }),
                }
            }
        }
    }
}
