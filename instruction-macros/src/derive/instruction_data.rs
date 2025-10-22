use syn::DeriveInput;

use crate::{
    parse::{
        instruction_tags::InstructionTags,
        instruction_variants::parse_instruction_variants,
        parsed_enum::ParsedEnum,
    },
    render::{
        feature_namespace::NamespacedTokenStream,
        instruction_data_struct::render_instruction_data_structs,
        try_from_u8_for_instruction_tag::render_try_from_u8_for_instruction_tags,
    },
};

pub fn derive_instruction_data(input: DeriveInput) -> syn::Result<Vec<NamespacedTokenStream>> {
    let parsed_enum = ParsedEnum::try_from(input)?;
    let instruction_tags = InstructionTags::try_from(&parsed_enum.data_enum)?;
    let instruction_variants = parse_instruction_variants(&parsed_enum.data_enum)?;

    let instruction_data_variants =
        render_instruction_data_structs(&parsed_enum, instruction_variants);
    let tag_try_from = render_try_from_u8_for_instruction_tags(&parsed_enum, &instruction_tags);

    let res = instruction_data_variants
        .into_iter()
        .chain(tag_try_from)
        .collect();

    Ok(res)
}
