use std::collections::HashMap;

use proc_macro2::{
    Literal,
    TokenStream,
};
use quote::{
    format_ident,
    ToTokens,
    TokenStreamExt,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Copy, strum_macros::Display, EnumIter, PartialEq, Eq, Hash)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Feature {
    SolanaProgram,
    Pinocchio,
    Client,
}

impl ToTokens for Feature {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Literal::string(&self.to_string()));
    }
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct FeatureNamespace(pub(crate) Feature);

impl ToTokens for FeatureNamespace {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let snake_namespace = self.0.to_string().replace("-", "_");
        tokens.append(format_ident!("generated_{snake_namespace}"));
    }
}

pub(crate) struct NamespacedTokenStream {
    pub tokens: TokenStream,
    pub namespace: FeatureNamespace,
}

pub fn merge_namespaced_token_streams(
    streams: Vec<Vec<NamespacedTokenStream>>,
) -> HashMap<FeatureNamespace, Vec<TokenStream>> {
    let mut hash_map: HashMap<FeatureNamespace, Vec<TokenStream>> = Feature::iter()
        .map(|f| (FeatureNamespace(f), vec![]))
        .collect();

    for NamespacedTokenStream { tokens, namespace } in streams.into_iter().flatten() {
        hash_map.entry(namespace).and_modify(|v| v.push(tokens));
    }

    hash_map
}
