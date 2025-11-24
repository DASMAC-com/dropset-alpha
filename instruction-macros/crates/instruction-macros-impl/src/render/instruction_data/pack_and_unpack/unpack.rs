//! Renders the code that deserializes raw instruction data into structured arguments for program
//! execution.

use std::collections::HashMap;

use itertools::Itertools;
use proc_macro2::{
    Literal,
    TokenStream,
};
use quote::quote;
use syn::Ident;

use crate::{
    parse::{
        error_path::ErrorPath,
        error_type::ErrorType,
    },
    render::Feature,
};

/// Render the inner body of the fallible `unpack` method.
///
/// `unpack` deserializes raw instruction data bytes into structured arguments according to the
/// corresponding instruction variant's instruction arguments.
pub fn render(
    size_without_tag: &Literal,
    struct_name: &Ident,
    field_names: &[Ident],
    error_path: &ErrorPath,
    group: &UnpackGroup,
) -> TokenStream {
    let ErrorPath { base, variant } = error_path;

    let unpack_assignments = &group.assignments;

    // Build the cfg output.
    let feature_flag = match group.features.as_slice() {
        [] => quote! {},
        [one_feature] => {
            quote! { #[cfg(feature = #one_feature)] }
        }
        // multiple features → #[cfg(any(feature = "a", feature = "b", ...))]
        multiple_features => {
            quote! {
                #[cfg(any( #(feature = #multiple_features),* ))]
            }
        }
    };

    let unpack_body = match size_without_tag.to_string().as_str() {
        // If the instruction has 0 bytes of data after the tag, simply return the Ok(empty data
        // struct) because all passed slices are valid.
        "0" => quote! { Ok(Self {}) },
        _ => quote! {
            if instruction_data.len() < #size_without_tag {
                return Err(#base::#variant);
            }

            // Safety: The length was just verified; all dereferences are valid.
            unsafe {
                let p = instruction_data.as_ptr();
                #(#unpack_assignments)*

                Ok(Self {
                    #(#field_names),*
                })
            }
        },
    };

    quote! {
        /// This implementation handles unpacking instruction data that comes *after* the
        /// discriminant has already been peeled off of the front of the slice.
        /// Trailing bytes are ignored; the length must be sufficient, not exact.
        #feature_flag
        impl Unpack<#base> for #struct_name {
            #[inline(always)]
            fn unpack(instruction_data: &[u8]) -> Result<Self, #base> {
                #unpack_body
            }
        }
    }
}

/// For combining error type paths into a single hash map entry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ErrorPathKey {
    base: String,
    variant: String,
}

impl ErrorPathKey {
    fn from_error_path(ep: &ErrorPath) -> Self {
        let base = &ep.base;
        ErrorPathKey {
            base: quote::quote! {#base}.to_string(),
            variant: ep.variant.to_string(),
        }
    }
}

pub fn group_features_by_error_path(
    feature_map: HashMap<Feature, Vec<TokenStream>>,
) -> Vec<UnpackGroup> {
    let entries = make_entries(feature_map);
    let mut groups: HashMap<ErrorPathKey, UnpackGroup> = HashMap::new();

    for (key, error_path, feature, assignments) in entries {
        groups
            .entry(key)
            .and_modify(|g| {
                if g.assignments
                    .iter()
                    .zip_eq(assignments.iter())
                    .any(|(a, b)| a.to_string() != b.to_string())
                {
                    panic!("conflicting unpack assignments for same ErrorPath");
                }
                g.features.push(feature);
            })
            .or_insert(UnpackGroup {
                features: vec![feature],
                assignments,
                error_path,
            });
    }

    groups.into_values().collect()
}

fn make_entries(
    feature_map: HashMap<Feature, Vec<TokenStream>>,
) -> Vec<(ErrorPathKey, ErrorPath, Feature, Vec<TokenStream>)> {
    feature_map
        .into_iter()
        .map(|(feature, assignments)| {
            // Compute the error type for this feature
            let error_path = ErrorType::InvalidInstructionData.to_path(feature);

            // Turn it into a hashable grouping key
            let key = ErrorPathKey::from_error_path(&error_path);

            (key, error_path, feature, assignments)
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct UnpackGroup {
    pub features: Vec<Feature>,
    pub assignments: Vec<TokenStream>,
    pub error_path: ErrorPath,
}
