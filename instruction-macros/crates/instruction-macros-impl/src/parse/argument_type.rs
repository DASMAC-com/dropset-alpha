//! Parsing implementations for the various [`ArgumentType`]s that can be used for the `args`
//! derive attribute.

use std::fmt::Display;

use itertools::Itertools;
use quote::ToTokens;
use strum::IntoEnumIterator;
use syn::{
    parse::Parse,
    Ident,
    Token,
    Type,
};

use crate::parse::{
    parsing_error::ParsingError,
    primitive_arg::PrimitiveArg,
};

#[derive(Debug, Clone)]
pub enum ArgumentType {
    PrimitiveArg(PrimitiveArg),
    Address,
}

impl ArgumentType {
    pub fn all_valid_types() -> String {
        format!("{}, {}", PrimitiveArg::iter().join(", "), ADDRESS_TYPE_STR)
    }
}

impl Parse for ArgumentType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_address = input.peek(Ident)
            && input
                .fork()
                .parse::<Ident>()
                .ok()
                .is_some_and(|id| id == "Address")
            && !input.peek2(Token![::]);

        if is_address {
            // Parse and consume the `Address` type.
            let _addr: syn::Ident = input.parse()?;
            Ok(Self::Address)
        } else {
            let ty: Type = input
                .parse()
                .map_err(|_| ParsingError::InvalidArgumentType.new_err(input.span()))?;
            Ok(Self::PrimitiveArg(PrimitiveArg::try_from(&ty)?))
        }
    }
}

pub trait ParsedPackableType {
    /// Returns the byte size of the argument type.
    fn size(&self) -> usize;

    fn as_parsed_type(&self) -> Type;
}

const ADDRESS_BYTES: usize = 32;
pub const ADDRESS_TYPE_STR: &str = "::solana_address::Address";

impl ParsedPackableType for ArgumentType {
    fn size(&self) -> usize {
        match self {
            Self::Address => ADDRESS_BYTES,
            Self::PrimitiveArg(arg) => arg.size(),
        }
    }

    fn as_parsed_type(&self) -> Type {
        match self {
            Self::Address => syn::parse_str(ADDRESS_TYPE_STR).expect("Should be a valid type"),
            Self::PrimitiveArg(arg) => arg.as_parsed_type(),
        }
    }
}

impl Display for ArgumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Print the string type for the address because `TokenStream`'s `Display` adds spaces.
            ArgumentType::Address => write!(f, "{}", ADDRESS_TYPE_STR),
            // Otherwise just use the `TokenStream` `Display` implementation.
            _ => write!(f, "{}", self.as_parsed_type().to_token_stream()),
        }
    }
}
