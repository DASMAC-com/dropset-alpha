//! Interprets RPC and on-chain errors into readable `dropset`/Solana instruction error messages.

use std::fmt::Display;

use dropset_interface::{
    error::DropsetError,
    instructions::DropsetInstruction,
};
use solana_client::client_error::ClientError;
use solana_instruction_error::InstructionError as SolanaInstructionError;
use solana_transaction_error::TransactionError;

use crate::{
    fmt_kv,
    transactions::InstructionDataAtIndex,
    LogColor,
};

enum InstructionError {
    Solana {
        instruction_tag: u8,
        error: SolanaInstructionError,
    },
    Dropset {
        dropset_instruction: DropsetInstruction,
        error: DropsetError,
    },
}

pub struct PrettyInstructionError(InstructionError);

impl PrettyInstructionError {
    pub fn new(error: &ClientError, instructions: &impl InstructionDataAtIndex) -> Option<Self> {
        let TransactionError::InstructionError(instruction_index, instruction_error) =
            error.get_transaction_error()?
        else {
            return None;
        };

        let program_id = *instructions
            .program_id(instruction_index as usize)
            .expect("Instruction index from error should be valid");
        let instruction_tag = instructions
            .data(instruction_index as usize)
            .expect("Instruction index from error should be valid")[0];

        let res = match instruction_error {
            SolanaInstructionError::Custom(code) => {
                if program_id == dropset::ID {
                    let dropset_code =
                        u8::try_from(code).expect("Dropset error codes should be valid u8 values");
                    let dropset_error =
                        DropsetError::from_repr(dropset_code).expect("Should be valid");
                    let dropset_tag =
                        DropsetInstruction::try_from(instruction_tag).expect("Should be valid");

                    Self(InstructionError::Dropset {
                        dropset_instruction: dropset_tag,
                        error: dropset_error,
                    })
                } else {
                    Self(InstructionError::Solana {
                        instruction_tag,
                        error: SolanaInstructionError::Custom(code),
                    })
                }
            }
            instruction_error => Self(InstructionError::Solana {
                instruction_tag,
                error: instruction_error,
            }),
        };

        Some(res)
    }
}

impl Display for PrettyInstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (error_type, instruction, error) = match &self.0 {
            InstructionError::Solana {
                instruction_tag,
                error,
            } => (
                "SolanaInstructionError",
                instruction_tag.to_string(),
                error.to_string(),
            ),
            InstructionError::Dropset {
                dropset_instruction,
                error,
            } => (
                "DropsetError",
                dropset_instruction.to_string(),
                error.to_string(),
            ),
        };

        let message = format!("error code: {instruction}, message: {error}");
        let error_message = fmt_kv!(error_type, message, LogColor::Error);
        writeln!(f, "{error_message}")
    }
}
