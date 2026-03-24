use dropset_interface::error::DropsetError;
use solana_address::Address;
use solana_client::client_error::ClientError;
use solana_instruction_error::InstructionError;
use solana_program_error::ProgramError;
use solana_transaction_error::TransactionError;
use spl_token_2022_interface::error::TokenError;

use crate::transactions::InstructionDataAtIndex;

/// Error returned by transaction submission. Distinguishes dropset program errors (which callers
/// may want to match on) from all other failures.
#[derive(Debug)]
pub enum TransactionSubmitError {
    /// The transaction failed with a known dropset program error.
    Dropset(DropsetError),
    /// The transaction failed with a known token program error. This could possible be token 2022.
    Token {
        program_id: Address,
        error: TokenError,
    },
    /// The transaction failed with a generic program error.
    Program {
        program_id: Address,
        error: ProgramError,
    },
    /// Any other failure (RPC, signing, parsing, etc).
    Other(anyhow::Error),
}

impl TransactionSubmitError {
    pub fn from_client_error(
        error: ClientError,
        instructions: &impl InstructionDataAtIndex,
    ) -> Self {
        let Some(tx_err) = error.get_transaction_error() else {
            return Self::Other(error.into());
        };

        match tx_err {
            TransactionError::InstructionError(idx, InstructionError::Custom(code)) => {
                let program_id = *instructions
                    .program_id(idx as usize)
                    .expect("Transaction should have been parsed correctly");

                match program_id {
                    dropset_interface::program::ID => {
                        let dropset_code = u8::try_from(code)
                            .expect("Dropset error codes should be valid u8 values");
                        match DropsetError::from_repr(dropset_code) {
                            Some(e) => Self::Dropset(e),
                            None => {
                                Self::Other(anyhow::anyhow!("Unknown dropset error code: {code}"))
                            }
                        }
                    }
                    spl_token_interface::ID | spl_token_2022_interface::ID => {
                        let error = TokenError::try_from(code);
                        match error {
                            Ok(e) => Self::Token {
                                program_id,
                                error: e,
                            },
                            Err(e) => Self::Other(e.into()),
                        }
                    }
                    _ => Self::Other(error.into()),
                }
            }
            TransactionError::InstructionError(idx, ixn_err) => {
                let program_id = *instructions
                    .program_id(idx as usize)
                    .expect("Transaction should have been parsed correctly");

                match ProgramError::try_from(ixn_err) {
                    Ok(e) => Self::Program {
                        program_id,
                        error: e,
                    },
                    Err(_) => Self::Other(error.into()),
                }
            }
            _ => Self::Other(error.into()),
        }
    }
}

impl From<TransactionSubmitError> for anyhow::Error {
    fn from(e: TransactionSubmitError) -> Self {
        match e {
            TransactionSubmitError::Dropset(e) => anyhow::anyhow!("{e:?}"),
            TransactionSubmitError::Token { error: e, .. } => anyhow::anyhow!("{e:?}"),
            TransactionSubmitError::Program { error: e, .. } => {
                anyhow::anyhow!("{e:?}")
            }
            TransactionSubmitError::Other(e) => e,
        }
    }
}

impl From<anyhow::Error> for TransactionSubmitError {
    fn from(e: anyhow::Error) -> Self {
        TransactionSubmitError::Other(e)
    }
}
