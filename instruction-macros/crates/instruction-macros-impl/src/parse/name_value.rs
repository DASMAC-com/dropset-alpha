//! See [`parse_name_value_literal`].

use quote::ToTokens;
use syn::{
    Expr,
    Lit,
    Meta,
};

use crate::ParsingError;

/// A utility function for parsing `name = "value"` style [`Meta`]s into string values with proper
/// error reporting.
pub fn parse_name_value_literal(meta: &Meta) -> syn::Result<String> {
    let expr = &meta.require_name_value()?.value;
    if let Expr::Lit(syn::ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = expr
    {
        Ok(lit_str.value())
    } else {
        let got = expr.to_token_stream().to_string();
        Err(ParsingError::ExpectedNameValueLiteral(got).new_err(meta))
    }
}
