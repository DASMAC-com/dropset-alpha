//! Renders the `CodamaType` trait implementation for a `derive(CodamaType)` struct.

use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::{
    known_type::KnownType,
    parsed_struct::ParsedStruct,
};

fn codama_traits_path() -> TokenStream {
    quote! { ::instruction_macros::codama }
}

/// Renders a `CodamaType` impl for a struct. Each field's type node is obtained by calling
/// `CodamaType::codama_type_node()` on the field type (for known types) or emitting a
/// `defined_type_link` (for unknown/custom types).
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
            let name_str = name.to_string();
            let camel_name = to_camel_case(&name_str);

            // Determine how to get the type node for this field.
            let type_node_expr = match KnownType::new(ty.clone()) {
                Some(_) => {
                    // Known type: call CodamaType::codama_type_node() directly.
                    quote! { <#ty as #codama::CodamaType>::codama_type_node() }
                }
                None => {
                    // Unknown type: call CodamaType::codama_type_node(). If the type doesn't
                    // impl CodamaType, this produces a compile error — enforcing that all
                    // instruction argument types are IDL-representable.
                    quote! { <#ty as #codama::CodamaType>::codama_type_node() }
                }
            };

            quote! {
                #codama::StructFieldTypeNode::new(
                    ::instruction_macros::codama::_alloc::string::String::from(#camel_name),
                    #type_node_expr,
                )
            }
        })
        .collect();

    let struct_name_str = to_camel_case(&struct_ident.to_string());

    quote! {
        #[cfg(feature = "codama")]
        impl #codama::CodamaType for #struct_ident {
            fn codama_type_node() -> #codama::TypeNode {
                // When referenced as a field in another struct, return a link to the defined type.
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

fn to_camel_case(s: &str) -> String {
    // snake_case -> camelCase
    if s.contains('_') {
        let mut result = String::new();
        let mut capitalize_next = false;
        for ch in s.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.extend(ch.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }
        return result;
    }

    // PascalCase -> camelCase
    if s.chars().next().is_some_and(|c| c.is_uppercase()) {
        let mut chars = s.chars();
        let first = chars.next().unwrap();
        return first.to_lowercase().chain(chars).collect();
    }

    s.to_string()
}
