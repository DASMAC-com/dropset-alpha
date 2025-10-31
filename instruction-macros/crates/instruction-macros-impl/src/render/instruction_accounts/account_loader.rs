use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote,
};

use crate::{
    parse::{
        error_path::ErrorPath,
        error_type::ErrorType,
        instruction_variant::InstructionVariant,
    },
    render::Feature,
};

/// Render the account load function.
///
/// The account load function fallibly attempts to structure a slice of `AccountInfo`s into the
/// corresponding struct of ordered accounts.
pub fn render_account_loader(
    feature: Feature,
    instruction_variant: &InstructionVariant,
) -> TokenStream {
    // `accounts` arg needs to be a slice, and `client` uses owned pubkeys, so return an empty
    // token stream if this is for `client`.
    if feature == Feature::Client {
        return quote! {};
    }

    let lifetimed_ref = feature.lifetimed_ref();
    let account_field_type = feature.account_info_type_path();
    let accounts = instruction_variant
        .accounts
        .iter()
        .map(|acc| format_ident!("{}", acc.name))
        .collect::<Vec<_>>();

    let ErrorPath { base, variant } = ErrorType::IncorrectNumAccounts.to_path(feature);

    quote! {
        pub fn load_accounts(accounts: #lifetimed_ref [#account_field_type]) -> Result<Self, #base> {
            let [ #(#accounts),* ] = accounts else {
                return Err(#base::#variant);
            };

            Ok(Self {
                #(#accounts),*
            })
        }
    }
}
