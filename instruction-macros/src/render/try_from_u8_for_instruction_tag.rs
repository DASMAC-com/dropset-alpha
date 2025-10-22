use itertools::Itertools;
use proc_macro2::{
    Literal,
    TokenStream,
};
use quote::quote;
use strum::IntoEnumIterator;

use crate::{
    parse::{
        error_path::ErrorPath,
        instruction_tags::InstructionTags,
        parsed_enum::ParsedEnum,
    },
    render::{
        error_type::ErrorType,
        feature_namespace::{
            Feature,
            FeatureNamespace,
            NamespacedTokenStream,
        },
    },
};

pub fn render_try_from_u8_for_instruction_tags(
    parsed_enum: &ParsedEnum,
    instruction_tags: &InstructionTags,
) -> Vec<NamespacedTokenStream> {
    Feature::iter()
        .map(|feature| NamespacedTokenStream {
            tokens: render_try_from_impl(parsed_enum, instruction_tags, feature),
            namespace: FeatureNamespace(feature),
        })
        .collect()
}

fn render_try_from_impl(
    parsed_enum: &ParsedEnum,
    instruction_tags: &InstructionTags,
    feature: Feature,
) -> TokenStream {
    let enum_ident = &parsed_enum.enum_ident;
    let ErrorPath { base, variant } = ErrorType::InvalidTag.to_path(feature);

    let mut cloned_variants = instruction_tags.0.clone().into_iter().collect_vec();
    cloned_variants.sort_by_key(|t| t.discriminant);

    // Build a 2d vector of disjoint ranges, grouped/chunked by contiguous discriminants.
    // For example: [0..2, 3..5, 7..99]
    let chunks = cloned_variants
        .chunk_by(|a, b| a.discriminant + 1 == b.discriminant)
        .collect_vec();

    let ranges = chunks.iter().map(|chunk| {
        let start = Literal::u8_unsuffixed(chunk[0].discriminant);
        if chunk.len() == 1 {
            quote! { #start }
        } else {
            let end =
                Literal::u8_unsuffixed(chunk.last().expect("Should have 1+ elements").discriminant);
            quote! { #start..=#end }
        }
    });

    quote! {
        impl TryFrom<u8> for super::#enum_ident {
            type Error = #base;

            fn try_from(tag: u8) -> Result<Self, Self::Error> {
                // Safety: Match arms ensure only valid discriminants are transmuted.
                match tag {
                    #(#ranges)|* => Ok(unsafe { core::mem::transmute::<u8, Self>(tag) }),
                    _ => Err(#base::#variant),
                }
            }
        }
    }
}
