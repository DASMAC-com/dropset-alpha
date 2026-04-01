//! Renders the `CodamaProgram` trait implementation for a `derive(CodamaProgram)` enum.
//! This produces the full Codama IDL tree: instructions, accounts, arguments, discriminators,
//! and defined types.
//!
//! Custom (non-primitive) argument types are automatically collected from instruction variants,
//! wrapped in [`DefinedTypeNode`]s, and added to the program's `definedTypes`. Instruction
//! arguments reference them via [`DefinedTypeLinkNode`]s.

use proc_macro2::TokenStream;
use quote::{
    quote,
    ToTokens,
};

use super::codama_helpers::{
    codama_traits_path,
    to_camel_case,
};
use crate::parse::{
    argument_type::{
        ArgumentType,
        ParsedPackableType,
    },
    instruction_variant::InstructionVariant,
    parsed_enum::ParsedEnum,
};

/// Renders a `CodamaProgram` impl for the instruction enum.
pub fn render(parsed_enum: &ParsedEnum, variants: &[InstructionVariant]) -> TokenStream {
    let enum_ident = &parsed_enum.enum_ident;
    let codama = codama_traits_path();
    let string = quote! { ::instruction_macros::codama::_alloc::string::String };
    let vec_macro = quote! { ::instruction_macros::codama::_alloc::vec };

    let instruction_nodes: Vec<TokenStream> = variants
        .iter()
        .filter(|v| v.at_least_one_account_or_arg)
        .map(|v| {
            let name = to_camel_case(&v.variant_name.to_string());
            let disc = v.discriminant;

            let account_nodes: Vec<TokenStream> = v
                .accounts
                .iter()
                .map(|acc| {
                    let acc_name = to_camel_case(&acc.name);
                    let is_writable = acc.is_writable;
                    let is_signer = acc.is_signer;
                    let desc = &acc.description;
                    let docs = if desc.is_empty() {
                        quote! { #vec_macro::Vec::new() }
                    } else {
                        quote! { #vec_macro![#string::from(#desc)] }
                    };
                    quote! {
                        #codama::InstructionAccountNode {
                            kind: "instructionAccountNode",
                            name: #string::from(#acc_name),
                            is_writable: #is_writable,
                            is_signer: #is_signer,
                            is_optional: false,
                            docs: #docs,
                        }
                    }
                })
                .collect();

            let mut arg_nodes: Vec<TokenStream> = vec![quote! {
                #codama::InstructionArgumentNode {
                    kind: "instructionArgumentNode",
                    name: #string::from("discriminator"),
                    docs: #vec_macro::Vec::new(),
                    ty: #codama::NumberFormat::U8.le(),
                    default_value: Some(#codama::NumberValueNode {
                        kind: "numberValueNode",
                        number: #disc as u64,
                    }),
                    default_value_strategy: Some("omitted"),
                }
            }];

            for arg in &v.arguments {
                let arg_name = to_camel_case(&arg.name.to_string());
                let desc = &arg.description;
                let docs = if desc.is_empty() {
                    quote! { #vec_macro::Vec::new() }
                } else {
                    quote! { #vec_macro![#string::from(#desc)] }
                };
                let type_node_expr = argument_type_to_inline_expr(&arg.ty, &codama);
                arg_nodes.push(quote! {
                    #codama::InstructionArgumentNode {
                        kind: "instructionArgumentNode",
                        name: #string::from(#arg_name),
                        docs: #docs,
                        ty: #type_node_expr,
                        default_value: None,
                        default_value_strategy: None,
                    }
                });
            }

            quote! {
                #codama::InstructionNode {
                    kind: "instructionNode",
                    name: #string::from(#name),
                    docs: #vec_macro::Vec::new(),
                    optional_account_strategy: "programId",
                    accounts: #vec_macro![#(#account_nodes),*],
                    arguments: #vec_macro![#(#arg_nodes),*],
                    discriminators: #vec_macro![#codama::discriminator(#codama::NumberFormat::U8, #disc as u64)],
                }
            }
        })
        .collect();

    // Collect all unique custom argument types. For each one, use
    // `CodamaType::name()` and `CodamaType::type_node()` to build a DefinedTypeNode.
    let mut seen_type_names = std::collections::HashSet::new();
    let mut defined_type_exprs: Vec<TokenStream> = Vec::new();

    for v in variants.iter().filter(|v| v.at_least_one_account_or_arg) {
        for arg in &v.arguments {
            if let ArgumentType::UnknownType(syn_ty) = &arg.ty {
                let type_name = syn_ty.to_token_stream().to_string();
                if seen_type_names.insert(type_name) {
                    defined_type_exprs.push(quote! {
                        #codama::DefinedTypeNode {
                            kind: "definedTypeNode",
                            name: #string::from(<#syn_ty as #codama::CodamaType>::name()),
                            docs: #vec_macro::Vec::new(),
                            ty: <#syn_ty as #codama::CodamaType>::type_node(),
                        }
                    });
                }
            }
        }
    }

    quote! {
        impl #codama::CodamaProgram for #enum_ident {
            fn codama_root(program_name: &str, program_id: &str) -> #codama::RootNode {
                let instructions = #vec_macro![#(#instruction_nodes),*];
                let defined_types = #vec_macro![#(#defined_type_exprs),*];
                let program = #codama::ProgramNode::new(
                    #string::from(program_name),
                    #string::from(program_id),
                    instructions,
                    defined_types,
                );
                #codama::RootNode::new(program)
            }
        }
    }
}

/// For instruction arguments: primitives get inlined via `CodamaType::type_node()`,
/// custom types get a `DefinedTypeLinkNode` (the full definition is in `definedTypes`).
fn argument_type_to_inline_expr(ty: &ArgumentType, codama: &TokenStream) -> TokenStream {
    match ty {
        ArgumentType::KnownType(kt) => {
            let fully_qualified = kt.as_fully_qualified_type();
            quote! { <#fully_qualified as #codama::CodamaType>::type_node() }
        }
        ArgumentType::UnknownType(syn_ty) => {
            quote! { #codama::defined_type_link(<#syn_ty as #codama::CodamaType>::name()) }
        }
    }
}
