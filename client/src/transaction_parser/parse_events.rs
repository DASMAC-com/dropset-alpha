use dropset_interface::instructions::DropsetInstruction;

use crate::{
    events::dropset_event::{
        unpack_instruction_events,
        DropsetEvent,
        EventError,
    },
    transaction_parser::ParsedInstruction,
};

pub struct ParsedTransactionWithEvents {}

pub fn parse_events(
    parsed_instruction: &ParsedInstruction,
) -> Result<Vec<DropsetEvent>, EventError> {
    let (tag_byte, instruction_event_data) = match parsed_instruction.data.split_at_checked(1) {
        Some(v) => v,
        None => return Ok(vec![]),
    };

    let tag = tag_byte
        .first()
        .and_then(|byte| DropsetInstruction::try_from(*byte).ok());

    match (parsed_instruction.program_id.to_bytes(), tag) {
        (dropset::ID, Some(DropsetInstruction::FlushEvents)) => {
            unpack_instruction_events(instruction_event_data)
        }
        _ => Ok(vec![]),
    }
}
