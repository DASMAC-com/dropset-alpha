use dropset_interface::error::DropsetError;
use solana_address::Address;
use solana_client::{
    client_error::{
        ClientError,
        ClientErrorKind,
    },
    rpc_request::{
        RpcError::RpcResponseError,
        RpcResponseErrorData,
    },
    rpc_response::RpcSimulateTransactionResult,
};
use solana_instruction_error::InstructionError;
use solana_program_error::ProgramError;
use solana_sdk::message::Instruction;
use solana_transaction_error::TransactionError;
use spl_token_2022_interface::error::TokenError;

/// Error returned by transaction submission. Distinguishes dropset program errors (which callers
/// may want to match on) from all other failures.
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
    pub fn from_client_error(error: ClientError, instructions: &[Instruction]) -> Self {
        let Some(tx_err) = transaction_error_from_client(&error) else {
            return Self::Other(error.into());
        };

        match tx_err {
            TransactionError::InstructionError(idx, InstructionError::Custom(code)) => {
                let instruction = instructions
                    .get(idx as usize)
                    .expect("Transaction should have been parsed correctly");

                let program_id = instruction.program_id;
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
                match ProgramError::try_from(ixn_err) {
                    Ok(e) => Self::Program {
                        program_id: instructions
                            .get(idx as usize)
                            .expect("Transaction should be parsed correctly")
                            .program_id,
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

/// Extracts the underlying [`TransactionError`] from a [`ClientError`], handling both preflight
/// simulation failures and confirmed on-chain failures.
pub fn transaction_error_from_client(error: &ClientError) -> Option<TransactionError> {
    match error.kind() {
        ClientErrorKind::RpcError(RpcResponseError {
            data:
                RpcResponseErrorData::SendTransactionPreflightFailure(RpcSimulateTransactionResult {
                    err: Some(ui_err),
                    ..
                }),
            ..
        }) => Some(TransactionError::from(ui_err.clone())),
        ClientErrorKind::TransactionError(tx_err) => Some(tx_err.clone()),
        _ => None,
    }
}
