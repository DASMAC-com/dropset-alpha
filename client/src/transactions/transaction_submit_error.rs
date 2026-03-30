use dropset_interface::error::DropsetError;
use solana_address::Address;
use solana_client::client_error::ClientError;
use solana_instruction_error::InstructionError;
use solana_program_error::ProgramError;
use solana_transaction_error::TransactionError;
use spl_token_2022_interface::error::TokenError as Token2022Error;
use spl_token_interface::error::TokenError;

use crate::transactions::{
    inner_simulation_error,
    InstructionDataAtIndex,
};

#[derive(Debug, Copy, Clone, strum_macros::FromRepr, strum_macros::AsRefStr)]
#[repr(u8)]
/// There's only one, and no other crates export it.
pub enum AssociatedTokenAccountError {
    InvalidOwner,
}

impl From<AssociatedTokenAccountError> for ProgramError {
    fn from(e: AssociatedTokenAccountError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// Error returned by transaction submission. Distinguishes dropset program errors (which callers
/// may want to match on) from all other failures.
#[derive(Debug)]
pub enum TransactionSubmitError {
    /// The transaction failed with a known dropset program error.
    Dropset(DropsetError),
    /// The transaction failed with a known token program error.
    Token(TokenError),
    /// The transaction failed with a known token 2022 program error.
    Token2022(Token2022Error),
    /// The transaction failed with a known associated token account program error.
    AssociatedTokenAccount(AssociatedTokenAccountError),
    /// The transaction failed with a generic program error.
    Program {
        program_id: Address,
        error: ProgramError,
    },
    /// Any other failure (RPC, signing, parsing, etc).
    Other(anyhow::Error),
}

impl TransactionSubmitError {
    /// The flow below explained:
    ///
    /// - First, try to parse it as an inner instruction simulation error. This is more accurate
    ///   than the instruction info provided by [TransactionError::InstructionError], for reasons
    ///   explained in [inner_simulation_error].
    /// - Then, try to parse it as a transaction error, returning a [Self::Other] if it isn't one.
    /// - If it's a transaction error, match on it being an [InstructionError::Custom] or a simply a
    ///   generic [InstructionError].
    /// - All other cases fall back to a simple [Self::Other] error.
    pub fn from_client_error(
        error: ClientError,
        instructions: &impl InstructionDataAtIndex,
    ) -> Self {
        if let Some((program_id, code)) = inner_simulation_error(&error) {
            return Self::from_custom_error_info(program_id, code);
        }

        let Some(tx_err) = error.get_transaction_error() else {
            return Self::Other(error.into());
        };

        match tx_err {
            TransactionError::InstructionError(idx, ixn_err) => {
                let Some(program_id) = instructions.program_id(idx as usize) else {
                    let err: anyhow::Error = anyhow::Error::from(ixn_err);
                    return TransactionSubmitError::Other(err.context(format!(
                        "Couldn't get program ID for invalid instruction index \
                        ({idx}) while parsing error"
                    )));
                };

                match ixn_err {
                    InstructionError::Custom(code) => {
                        Self::from_custom_error_info(*program_id, code)
                    }
                    _ => match ProgramError::try_from(ixn_err.clone()) {
                        Ok(e) => Self::Program {
                            program_id: *program_id,
                            error: e,
                        },
                        Err(_) => Self::Other(ixn_err.into()),
                    },
                }
            }
            _ => Self::Other(error.into()),
        }
    }

    pub fn from_custom_error_info(program_id: Address, code: u32) -> TransactionSubmitError {
        match program_id {
            dropset_interface::program::ID => u8::try_from(code)
                .ok()
                .and_then(DropsetError::from_repr)
                .map_or_else(
                    || Self::Other(anyhow::anyhow!("Unknown dropset error code: {code}")),
                    Self::Dropset,
                ),
            spl_token_interface::ID => {
                let error = TokenError::try_from(code);
                match error {
                    Ok(e) => Self::Token(e),
                    Err(e) => Self::Other(e.into()),
                }
            }
            spl_token_2022_interface::ID => {
                let error = Token2022Error::try_from(code);
                match error {
                    Ok(e) => Self::Token2022(e),
                    Err(e) => Self::Other(e.into()),
                }
            }
            spl_associated_token_account_interface::program::ID => u8::try_from(code)
                .ok()
                .and_then(AssociatedTokenAccountError::from_repr)
                .map_or_else(
                    || Self::Other(anyhow::anyhow!("Unknown ATA error code: {code}")),
                    Self::AssociatedTokenAccount,
                ),
            _ => Self::Program {
                program_id,
                error: ProgramError::Custom(code),
            },
        }
    }
}

impl From<TransactionSubmitError> for anyhow::Error {
    fn from(e: TransactionSubmitError) -> Self {
        match e {
            TransactionSubmitError::Dropset(e) => anyhow::anyhow!("{e:?}"),
            TransactionSubmitError::Token(e) => anyhow::anyhow!("{e:?}"),
            TransactionSubmitError::Token2022(e) => anyhow::anyhow!("{e:?}"),
            TransactionSubmitError::AssociatedTokenAccount(e) => anyhow::anyhow!("{e:?}"),
            TransactionSubmitError::Program {
                error: e,
                program_id,
            } => {
                anyhow::anyhow!("Generic program error, program id: {program_id}, err: {e:?}")
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
