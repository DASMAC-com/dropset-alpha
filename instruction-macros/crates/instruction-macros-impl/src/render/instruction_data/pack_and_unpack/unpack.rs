//! Renders the code that deserializes raw instruction data into structured arguments for program
//! execution.

use std::collections::HashMap;

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

/// Render the fallible `unpack_*` method for each feature config.
///
/// `unpack_*` deserializes raw instruction data bytes into structured arguments according to the
/// corresponding instruction variant's instruction arguments.
///
/// The various `unpack_*` functions are exactly the same except for the error they return based
/// on the feature SDK they're implemented for.
///
/// For example, `unpack_pinocchio` returns the `pinocchio` `ProgramError` type.
pub fn render(
    size_without_tag: &Literal,
    field_names: &[Ident],
    unpack_assignments_map: HashMap<Feature, Vec<TokenStream>>,
) -> TokenStream {
    unpack_assignments_map
        .into_iter()
        .map(|(feature, unpack_assignments)| {
            render_variant(size_without_tag, &unpack_assignments, field_names, feature)
        })
        .collect()
}

fn render_variant(
    size_without_tag: &Literal,
    unpack_assignments: &[TokenStream],
    field_names: &[Ident],
    feature: Feature,
) -> TokenStream {
    let ErrorPath { base, variant } = ErrorType::InvalidInstructionData.to_path(feature);

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

    let feature_flag = quote! { #[cfg(feature = #feature)] };
    let method_name = feature.unpack_method_name();

    quote! {
        /// This method unpacks the instruction data that comes *after* the discriminant has
        /// already been peeled off of the front of the slice.
        /// Trailing bytes are ignored; the length must be sufficient, not exact.
        #feature_flag
        #[inline(always)]
        pub fn #method_name(instruction_data: &[u8]) -> Result<Self, #base> {
            #unpack_body
        }
    }
}
