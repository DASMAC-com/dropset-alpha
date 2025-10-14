use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{InstructionArgument, InstructionTags, InstructionVariant, TagConfig};

pub fn create_pack_fn(instruction_args: Vec<InstructionArgument>) -> syn::Result<TokenStream> {
    for arg in instruction_args.into_iter() {
        // arg.
    }

    Ok(quote! {})
}

pub fn create_instruction_tags(
    instruction_enum_ident: Ident,
    instruction_tags: InstructionTags,
    tag_config: TagConfig,
) -> TokenStream {
    let variants = instruction_tags.0.iter().map(|variant| {
        let ident = format_ident!("{}", &variant.name);
        let discriminant = variant.discriminant;

        quote! { #ident = #discriminant, }
    });

    let tag_enum_ident = format_ident!("{}Tag", instruction_enum_ident);
    let (error_base, error_variant) = (tag_config.error_base, tag_config.error_variant);

    let mut cloned_variants = instruction_tags.0.clone().into_iter().collect_vec();
    let chunks = sort_and_chunk_variants(&mut cloned_variants);
    eprintln!("{:#?}", chunks);
    let match_arms = chunks.iter().map(|chunk| {
        let range = if chunk.len() == 1 {
            let start = chunk[0].discriminant;
            quote! { #start }
        } else {
            let (start, end) = (
                chunk[0].discriminant,
                chunk.last().expect("Should have 1+ elements").discriminant,
            );
            quote! { #start..=#end }
        };

        eprintln!("{}", range);

        quote! { #range => Ok(unsafe { core::mem::transmute::<u8, Self>(value) }), }
    });

    let safety_comment = quote! {
        // Safety: Match arms ensure only valid discriminants are transmuted.
    };

    quote! {
        #[repr(u8)]
        #[derive(Clone, Copy, Debug, PartialEq)]
        #[cfg_attr(test, derive(strum_macros::FromRepr, strum_macros::EnumIter))]
        pub enum #tag_enum_ident {
            #(#variants)*
        }

        impl TryFrom<u8> for #tag_enum_ident {
            type Error = #error_base;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                #safety_comment
                match value {
                    #(#match_arms)*
                    _ => Err(#error_base::#error_variant),
                }
            }
        }
    }
}

pub fn create_try_from_u8_for_instruction_tag(instruction_tags: InstructionTags) -> TokenStream {
    let mut variants = instruction_tags.0.clone();
    let chunks = sort_and_chunk_variants(&mut variants);

    TokenStream::new()
}

fn sort_and_chunk_variants(variants: &mut Vec<InstructionVariant>) -> Vec<&[InstructionVariant]> {
    variants.sort_by_key(|t| t.discriminant);
    assert_eq!(
        variants[0].discriminant, 0,
        "First discriminant is not zero."
    );

    // Build a 2d vector of disjoint ranges, grouped/chunked by contiguous discriminants.
    // For example: [0..2, 3..5, 7..99]
    let chunks = variants
        .chunk_by(|a, b| a.discriminant + 1 == b.discriminant)
        .collect();

    chunks
}

#[cfg(test)]
/// Tests for chunking contiguous variants by discriminant. Since this ultimately generates safe
/// unsafe code, it's important this is correct.
mod tests {
    use super::*;

    fn make_variants(discriminants: &[u8]) -> Vec<InstructionVariant> {
        discriminants
            .iter()
            .map(|&disc| InstructionVariant {
                name: disc.to_string(),
                discriminant: disc,
            })
            .collect()
    }

    fn chunks_to_discriminants(chunks: Vec<&[InstructionVariant]>) -> Vec<Vec<u8>> {
        chunks
            .into_iter()
            .map(|chunk| chunk.iter().map(|v| v.discriminant).collect())
            .collect()
    }

    #[test]
    fn test_single() {
        let mut variants = make_variants(&[0]);
        let chunks = sort_and_chunk_variants(&mut variants);

        assert_eq!(chunks_to_discriminants(chunks), vec![vec![0]]);
    }

    #[test]
    fn test_single_contiguous_range() {
        let mut variants = make_variants(&[0, 1, 2]);
        let chunks = sort_and_chunk_variants(&mut variants);

        assert_eq!(chunks_to_discriminants(chunks), vec![vec![0, 1, 2]]);
    }

    #[test]
    fn test_multiple_disjoint_ranges() {
        let mut variants = make_variants(&[0, 1, 5, 6, 10]);
        let chunks = sort_and_chunk_variants(&mut variants);

        assert_eq!(
            chunks_to_discriminants(chunks),
            vec![vec![0, 1], vec![5, 6], vec![10]]
        );
    }

    #[test]
    fn test_all_singletons() {
        let mut variants = make_variants(&[0, 3, 7, 12]);
        let chunks = sort_and_chunk_variants(&mut variants);

        assert_eq!(
            chunks_to_discriminants(chunks),
            vec![vec![0], vec![3], vec![7], vec![12]]
        );
    }

    #[test]
    fn test_complex_pattern() {
        let mut variants = make_variants(&[0, 2, 3, 100, 4, 7, 8, 15]);
        let chunks = sort_and_chunk_variants(&mut variants);

        assert_eq!(
            chunks_to_discriminants(chunks),
            vec![vec![0], vec![2, 3, 4], vec![7, 8], vec![15], vec![100]]
        );
    }

    #[test]
    #[should_panic(expected = "First discriminant is not zero.")]
    fn test_panics_without_zero() {
        let mut variants = make_variants(&[1, 2]);
        sort_and_chunk_variants(&mut variants);
    }
}
