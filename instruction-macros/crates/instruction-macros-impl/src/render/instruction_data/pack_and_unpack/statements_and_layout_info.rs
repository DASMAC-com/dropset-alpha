//! Builds intermediate representations describing layout, ordering, and serialization statements
//! used by pack/unpack code generation.

use std::collections::HashMap;

use proc_macro2::{
    Literal,
    TokenStream,
};
use quote::quote;
use strum::IntoEnumIterator;
use syn::Ident;

use crate::{
    parse::{
        argument_type::{
            ArgumentType,
            ParsedPackableType,
        },
        instruction_variant::InstructionVariant,
    },
    render::Feature,
};

pub struct StatementsAndLayoutInfo {
    /// The total size of the struct without the tag byte as a literal `usize`.
    pub size_without_tag: Literal,
    /// The total size of the struct with the tag byte as a literal `usize`.
    pub size_with_tag: Literal,
    /// The layout docs indicating which bytes each field occupies in the struct layout.
    pub layout_docs: Vec<TokenStream>,
    /// Each field's individual `pack` statement.
    pub pack_statements: Vec<TokenStream>,
    /// Each field's tuple of feature + unpack assignments.
    pub unpack_assignments_map: HashMap<Feature, Vec<TokenStream>>,
}

impl StatementsAndLayoutInfo {
    pub fn new(instruction_variant: &InstructionVariant) -> StatementsAndLayoutInfo {
        let instruction_args = &instruction_variant.arguments;
        let (size_without_tag, layout_docs, pack_statements, unpack_assignments) =
            instruction_args.iter().fold(
                (
                    0,
                    vec![],
                    vec![],
                    HashMap::from_iter(Feature::iter().map(|f| (f, vec![]))),
                ),
                |(curr, mut layout_docs, mut pack_statements, mut unpack_assignments_map), arg| {
                    // Pack statements must also pack the discriminant first, so start at byte `1`
                    let pack_offset = curr + 1;
                    // Unpack statements operate on the instruction data *after* the tag byte has
                    // been peeled.
                    let unpack_offset = curr;

                    let arg_name = &arg.name;
                    let arg_type = &arg.ty;
                    let size = arg.ty.size();

                    let layout_comment = layout_doc_comment(arg_name, arg_type, pack_offset, size);
                    let pack = arg_type.pack_statement(arg_name, pack_offset);

                    layout_docs.push(layout_comment);
                    pack_statements.push(pack);

                    // Push to each individual namespaced `unpack` statements vec.
                    for feature in Feature::iter() {
                        let statement = arg_type.unpack_statement(arg_name, unpack_offset, feature);
                        unpack_assignments_map
                            .get_mut(&feature)
                            .expect("Should have feature")
                            .push(statement);
                    }

                    (
                        curr + size,
                        layout_docs,
                        pack_statements,
                        unpack_assignments_map,
                    )
                },
            );

        StatementsAndLayoutInfo {
            size_without_tag: Literal::usize_unsuffixed(size_without_tag),
            size_with_tag: Literal::usize_unsuffixed(size_without_tag + 1),
            layout_docs,
            pack_statements,
            unpack_assignments_map: unpack_assignments,
        }
    }
}

/// Create the layout doc string that indicates which bytes are being written to for a single arg.
fn layout_doc_comment(
    arg_name: &Ident,
    arg_type: &ArgumentType,
    pack_offset: usize,
    size: usize,
) -> TokenStream {
    let end = pack_offset + size;
    let layout_doc_string = match size {
        1 => format!(
            " - `[{}]` **{}** (`{}`, 1 byte)",
            pack_offset, arg_name, arg_type
        ),
        size => format!(
            " - `[{}..{}]` **{}** (`{}`, {} bytes)",
            pack_offset, end, arg_name, arg_type, size
        ),
    };

    quote! { #[doc = #layout_doc_string] }
}
