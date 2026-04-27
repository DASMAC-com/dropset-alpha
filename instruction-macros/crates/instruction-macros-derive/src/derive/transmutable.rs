//! Derive helper for the [`crate::Transmutable`] trait.

use instruction_macros_impl::parse::data_struct::require_data_struct;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, Meta, Token};

fn has_repr_c_or_transparent(input: &DeriveInput) -> bool {
    for attr in &input.attrs {
        if !attr.path().is_ident("repr") {
            continue;
        }
        if let Ok(nested) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            if nested
                .iter()
                .any(|m| m.path().is_ident("C") || m.path().is_ident("transparent"))
            {
                return true;
            }
        }
    }
    false
}

pub fn derive_transmutable(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_ident = input.ident.clone();

    if !has_repr_c_or_transparent(&input) {
        return Err(syn::Error::new_spanned(
            &struct_ident,
            "Transmutable requires `#[repr(C)]` or `#[repr(transparent)]`",
        ));
    }

    require_data_struct(input)?;

    Ok(quote! {
        unsafe impl crate::state::transmutable::Transmutable for #struct_ident {
            const LEN: usize = ::core::mem::size_of::<Self>();

            #[inline(always)]
            fn validate_bit_patterns(_bytes: &[u8]) -> crate::error::DropsetResult {
                Ok(())
            }
        }
    })
}
