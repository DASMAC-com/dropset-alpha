//! Describes the supported codegen features/targets and provides helpers to conditionally
//! enable or disable parts of the generated API.

use proc_macro2::{
    Literal,
    TokenStream,
};
use quote::{
    format_ident,
    quote,
    ToTokens,
    TokenStreamExt,
};
use strum_macros::EnumIter;
use syn::Ident;

#[derive(Debug, Clone, Copy, strum_macros::Display, EnumIter, PartialEq, Eq, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum Feature {
    SolanaProgram,
    Pinocchio,
    Client,
}

impl ToTokens for Feature {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Literal::string(&self.to_string()));
    }
}

impl Feature {
    pub fn account_view_lifetime(&self) -> TokenStream {
        match self {
            Feature::SolanaProgram => quote! { 'a },
            Feature::Pinocchio => quote! { 'a },
            Feature::Client => quote! {},
        }
    }

    pub fn lifetimed_ref(&self) -> TokenStream {
        match self {
            Feature::SolanaProgram => quote! { &'a },
            Feature::Pinocchio => quote! { &'a },
            Feature::Client => quote! {},
        }
    }

    /// The specific account view type path, without the lifetimed ref prefixed to it.
    pub fn account_view_type_path(&self) -> TokenStream {
        match self {
            Feature::SolanaProgram => quote! { ::solana_account_view::AccountView },
            Feature::Pinocchio => quote! { ::solana_account_view::AccountView },
            Feature::Client => quote! { ::solana_address::Address },
        }
    }

    /// The method name of the `unpack_*` function for a feature.
    pub fn unpack_method_name(&self) -> Ident {
        match self {
            Feature::SolanaProgram => format_ident!("unpack_solana_program"),
            Feature::Pinocchio => format_ident!("unpack_pinocchio"),
            Feature::Client => format_ident!("unpack_client"),
        }
    }
}
