//! Shared helpers for Codama IDL code generation.

use proc_macro2::TokenStream;
use quote::quote;

/// The fully qualified path to the `codama` module in `instruction_macros`.
pub fn codama_traits_path() -> TokenStream {
    quote! { ::instruction_macros::codama }
}

/// Converts a Rust identifier to camelCase for Codama IDL output.
///
/// - `snake_case` -> `camelCase`
/// - `PascalCase` -> `camelCase`
/// - Already `camelCase` -> unchanged
pub fn to_camel_case(s: &str) -> String {
    if s.contains('_') {
        let mut result = String::new();
        let mut capitalize_next = false;
        for ch in s.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.extend(ch.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }
        return result;
    }

    if s.chars().next().is_some_and(|c| c.is_uppercase()) {
        let mut chars = s.chars();
        let first = chars.next().unwrap();
        return first.to_lowercase().chain(chars).collect();
    }

    s.to_string()
}
