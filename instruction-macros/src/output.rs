use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{InstructionArgument, InstructionTags, InstructionVariant, TagConfig};

pub fn create_tag_variant_struct(
    tag_enum_ident: Ident,
    tag_variant: Ident,
    instruction_args: Vec<InstructionArgument>,
    tag_config: TagConfig,
) -> TokenStream {
    // The `0` is hardcoded for the discriminant, so start at byte `1..`
    let mut curr = 1;
    let (ret_names, field_assignments, struct_fields, layout_docs, writes, field_sizes): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = instruction_args
        .iter()
        .map(|arg| {
            let parsed_type = arg.ty.as_parsed_type();
            let (name, size) = (&arg.name, arg.ty.size());
            let (start, end) = (curr, curr + size);
            let desc = format!(" {}", &arg.description);
            let pack_doc_string = format!(
                " - [{}..{}]: the {} `{}` as little-endian bytes, {} bytes",
                start, end, arg.ty, name, size
            );

            assert_eq!(end - start, size);

            let write_bytes_line = quote! {
                ::core::ptr::copy_nonoverlapping(
                    (&self.#name.to_le_bytes()).as_ptr(),
                    (&mut data[#start..#end]).as_mut_ptr() as *mut u8,
                    #size,
                );
            };
            let struct_field = quote! {
                #[doc = #desc]
                pub #name: #parsed_type,
            };

            // The pointer offset is for the instruction data which has already peeled the tag byte.
            let ptr_offset = start - 1;
            let ptr_with_offset = if ptr_offset == 0 {
                quote! { p }
            } else {
                quote! { p.add(#ptr_offset) }
            };
            let field_assignment = quote! {
                let #name = #parsed_type::from_le_bytes(*(#ptr_with_offset as *const [u8; #size]));
            };

            curr = end;
            (
                name,
                field_assignment,
                struct_field,
                quote! {#[doc = #pack_doc_string]},
                write_bytes_line,
                size,
            )
        })
        .multiunzip();

    let size_with_tag = curr;
    let size_without_tag = size_with_tag - 1;
    let enum_variant = format!("{tag_enum_ident}::{tag_variant}");
    let discriminant_description = format!(" - [0]: the discriminant `{enum_variant}`, 1 byte");
    let writes = if writes.is_empty() {
        quote! {}
    } else {
        quote! { unsafe { #(#writes)* }}
    };

    let (error_base, error_variant) = (tag_config.error_base, tag_config.error_variant);

    let const_assertion = if instruction_args.is_empty() {
        quote! { const _: [(); #size_with_tag] = [(); 1]; }
    } else {
        quote! { const _: [(); #size_with_tag] = [(); 1 + #( #field_sizes )+* ]; }
    };

    quote! {
        pub struct #tag_variant {
            #(#struct_fields)*
        }

        /// Compile time assertion that the size with the tag == the sum of the field sizes.
        #const_assertion

        impl #tag_variant {
            #[doc = " Instruction data layout:"]
            #[doc = #discriminant_description]
            #(#layout_docs)*
            #[inline(always)]
            pub fn pack(&self) -> [u8; #size_with_tag] {
                let mut data: [::core::mem::MaybeUninit<u8>; #size_with_tag] = [::core::mem::MaybeUninit::uninit(); #size_with_tag];
                data[0].write(#tag_enum_ident::#tag_variant as u8);
                // Safety: The pointers are non-overlapping and the same exact size.
                #writes

                // All bytes initialized during the construction above.
                unsafe { *(data.as_ptr() as *const [u8; #size_with_tag]) }
            }

            /// This method unpacks the instruction data that comes *after* the discriminant has
            /// already been peeled off of the front of the slice.
            #[inline(always)]
            pub fn unpack(instruction_data: &[u8]) -> Result<Self, #error_base> {
                if instruction_data.len() < #size_without_tag {
                    return Err(#error_base::#error_variant);
                }

                // Safety: The length was just verified; all dereferences are valid.
                unsafe {
                    let p = instruction_data.as_ptr();
                    #(#field_assignments)*

                    Ok(Self {
                        #(#ret_names),*
                    })
                }
            }
        }
    }
}

pub fn create_tag_enum(tag_enum_ident: Ident, instruction_tags: InstructionTags) -> TokenStream {
    let variants = instruction_tags.0.iter().map(|variant| {
        let ident = variant.name.clone();
        let discriminant = variant.discriminant;

        quote! { #ident = #discriminant, }
    });

    quote! {
        #[repr(u8)]
        #[derive(Clone, Copy, Debug, PartialEq)]
        #[cfg_attr(test, derive(strum_macros::FromRepr, strum_macros::EnumIter))]
        pub enum #tag_enum_ident {
            #(#variants)*
        }
    }
}

pub fn create_try_from_u8_for_instruction_tag(
    tag_enum_ident: Ident,
    instruction_tags: InstructionTags,
    tag_config: TagConfig,
) -> TokenStream {
    let (error_base, error_variant) = (tag_config.error_base, tag_config.error_variant);
    let mut cloned_variants = instruction_tags.0.clone().into_iter().collect_vec();
    let chunks = sort_and_chunk_variants(&mut cloned_variants);

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

    quote! {
        impl TryFrom<u8> for #tag_enum_ident {
            type Error = #error_base;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                // Safety: Match arms ensure only valid discriminants are transmuted.
                match value {
                    #(#match_arms)*
                    _ => Err(#error_base::#error_variant),
                }
            }
        }
    }
}

fn sort_and_chunk_variants(variants: &mut [InstructionVariant]) -> Vec<&[InstructionVariant]> {
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
    use quote::format_ident;

    use super::*;

    fn make_variants(discriminants: &[u8]) -> Vec<InstructionVariant> {
        discriminants
            .iter()
            .map(|&disc| InstructionVariant {
                name: format_ident!("{disc}"),
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
