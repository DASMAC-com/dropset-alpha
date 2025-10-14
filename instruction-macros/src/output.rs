use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{InstructionArgument, InstructionTags, InstructionVariant};

pub fn create_pack_fn(instruction_args: Vec<InstructionArgument>) -> syn::Result<TokenStream> {
    for arg in instruction_args.into_iter() {
        // arg.
    }

    Ok(quote! {})
}

pub fn create_instruction_tags(
    instruction_enum_ident: Ident,
    instruction_tags: InstructionTags,
) -> TokenStream {
    let variants = instruction_tags.0.iter().map(|variant| {
        let ident = format_ident!("{}", &variant.name);
        let discriminant = variant.discriminant;

        quote! { #ident = #discriminant, }
    });

    let tag_enum_ident = format_ident!("{instruction_enum_ident}Tag");

    quote! {
        #[repr(u8)]
        #[derive(Clone, Copy, Debug, PartialEq)]
        #[cfg_attr(test, derive(strum_macros::FromRepr, strum_macros::EnumIter))]
        pub enum #tag_enum_ident {
            #(#variants)*
        }
    }
}

pub fn create_try_from_u8_for_instruction_tag(instruction_tags: InstructionTags) -> TokenStream {
    let mut variants = instruction_tags.0.clone();
    let chunks = chunk_contiguous_variants_by_discriminant(&mut variants);

    TokenStream::new()
}

fn chunk_contiguous_variants_by_discriminant(
    variants: &mut Vec<InstructionVariant>,
) -> Vec<&[InstructionVariant]> {
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
    fn test_single_contiguous_range() {
        let mut variants = make_variants(&[0, 1, 2]);
        let chunks = chunk_contiguous_variants_by_discriminant(&mut variants);

        assert_eq!(chunks_to_discriminants(chunks), vec![vec![0, 1, 2]]);
    }

    #[test]
    fn test_multiple_disjoint_ranges() {
        let mut variants = make_variants(&[0, 1, 5, 6, 10]);
        let chunks = chunk_contiguous_variants_by_discriminant(&mut variants);

        assert_eq!(
            chunks_to_discriminants(chunks),
            vec![vec![0, 1], vec![5, 6], vec![10]]
        );
    }

    #[test]
    fn test_all_singletons() {
        let mut variants = make_variants(&[0, 3, 7, 12]);
        let chunks = chunk_contiguous_variants_by_discriminant(&mut variants);

        assert_eq!(
            chunks_to_discriminants(chunks),
            vec![vec![0], vec![3], vec![7], vec![12]]
        );
    }

    #[test]
    fn test_complex_pattern() {
        let mut variants = make_variants(&[0, 2, 3, 4, 7, 8, 15]);
        let chunks = chunk_contiguous_variants_by_discriminant(&mut variants);

        assert_eq!(
            chunks_to_discriminants(chunks),
            vec![vec![0], vec![2, 3, 4], vec![7, 8], vec![15]]
        );
    }

    #[test]
    #[should_panic(expected = "First discriminant is not zero")]
    fn test_panics_without_zero() {
        let mut variants = make_variants(&[1, 2]);
        chunk_contiguous_variants_by_discriminant(&mut variants);
    }
}
