//! Parses an enum variant's `#[args(...)]` attributes into typed instruction arguments with
//! optional human-readable descriptions.

use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    Ident,
    Lit,
    Token,
    Type,
};

use crate::{
    parse::primitive_arg::PrimitiveArg,
    ParsingError,
};

/// The parsed representation of a single instruction argument attribute for an enum variant.
#[derive(Debug, Clone)]
pub struct InstructionArgument {
    pub name: Ident,
    pub ty: PrimitiveArg,
    pub description: String,
}

impl Parse for InstructionArgument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let _colon: Token![:] = input.parse()?;
        let ty: Type = input.parse()?;

        // Optional: a single `key = value` pair as `desc = "argument description"`.
        let mut description: String = "".to_string();

        if input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
            match input.parse::<Lit>() {
                Ok(Lit::Str(s)) => description = s.value(),
                _ => return Err(ParsingError::ExpectedArgumentDescription.new_err(input.span())),
            }
        }

        Ok(InstructionArgument {
            name: ident.clone(),
            ty: PrimitiveArg::try_from(&ty)?,
            description,
        })
    }
}
