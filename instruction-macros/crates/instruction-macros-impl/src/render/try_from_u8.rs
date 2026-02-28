//! Generates a helper macro that maps raw instruction tags to their corresponding
//! enum variants using efficient, `unsafe` but sound transmutations.
//!
//! Includes compile-time checks to guarantee the generated code’s soundness. These checks output no
//! code in release builds.

use itertools::Itertools;
use proc_macro2::{
    Literal,
    TokenStream,
};
use quote::quote;

use crate::parse::{
    error_path::ErrorPath,
    error_type::ErrorType,
    instruction_variant::InstructionVariant,
    parsed_enum::ParsedEnum,
};

/// Renders a `TryFrom<u8>` implementation for an instruction tag enum type `T`.
///
/// ## Example
/// ```rust,ignore
/// #[repr(u8)]
/// #[derive(ProgramInstruction)]
/// pub enum MyInstruction {
///     CloseSeat = 0,
///     Deposit = 1,
///     Withdraw = 4,
///     TagWithImplicitDiscriminant, // implicit discriminant `5`
///     OutOfOrderDiscriminant = 3,
/// }
///
/// // Which then adds this implementation:
/// impl TryFrom<u8> for MyInstruction {
///     type Error = ProgramError;
///
///     #[inline(always)]
///     fn try_from(tag: u8) -> Result<Self, Self::Error> {
///         match tag {
///             0..=1 | 3..=5 => Ok(unsafe { ::core::mem::transmute::<u8, MyInstruction>(tag) }),
///             _ => Err(ProgramError::InvalidInstructionData),
///         }
///     }
/// }
///
/// // Calling it and matching on the tag variant with an early return:
/// match MyInstruction::try_from(tag)? {
///     MyInstruction::CloseSeat => { /* do close seat things */ },
///     MyInstruction::Deposit => { /* do deposit things */ },
///     _ => { /* etc */ },
/// }
/// ```
pub fn render(
    parsed_enum: &ParsedEnum,
    instruction_variants: &[InstructionVariant],
) -> TokenStream {
    let enum_ident = &parsed_enum.enum_ident;

    let sorted_by_discriminants = instruction_variants
        .iter()
        .sorted_by_key(|t| t.discriminant)
        .collect_vec();

    // Build a 2d collection of disjoint ranges, grouped by contiguous discriminants.
    // For example: [0..2, 3..5, 7..99]
    let chunks = sorted_by_discriminants
        .chunk_by(|a, b| a.discriminant + 1 == b.discriminant)
        .collect_vec();

    let ranges = chunks.iter().map(|chunk| {
        let start = Literal::u8_unsuffixed(chunk[0].discriminant);
        if chunk.len() == 1 {
            quote! { #start }
        } else {
            let end =
                Literal::u8_unsuffixed(chunk.last().expect("Should have 1+ elements").discriminant);
            quote! { #start..=#end }
        }
    });

    let ErrorPath { base, variant } = ErrorType::InvalidInstructionData.to_path();

    quote! {
        impl TryFrom<u8> for #enum_ident {
            type Error = #base;

            #[inline(always)]
            fn try_from(tag: u8) -> Result<Self, Self::Error> {
                // Safety: Only valid discriminants are transmuted.
                match tag {
                    #(#ranges)|* => Ok(unsafe { ::core::mem::transmute::<u8, #enum_ident>(tag) }),
                    _ => Err(#base::#variant),
                }
            }
        }
    }
}
