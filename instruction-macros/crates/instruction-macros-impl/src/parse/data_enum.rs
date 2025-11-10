//! See [`require_data_enum`].

use syn::{
    DataEnum,
    DeriveInput,
};

use crate::ParsingError;

/// Ensures the macro input is an enum and returns its `DataEnum` representation, or a typed error.
pub fn require_data_enum(input: DeriveInput) -> syn::Result<DataEnum> {
    match input.data {
        syn::Data::Enum(e) => Ok(e),
        _ => Err(ParsingError::NotAnEnum.new_err(input)),
    }
}
