//! Client-side error conversions for [`DropsetError`].

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
use solana_instruction::Instruction;
use solana_instruction_error::InstructionError;
use solana_transaction_error::TransactionError;

use crate::{
    error::DropsetError,
    program,
};

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

impl DropsetError {
    /// Attempts to extract a [`DropsetError`] from a [`TransactionError`], verifying that the
    /// failing instruction belongs to the dropset program.
    pub fn from_transaction_error(
        tx_err: TransactionError,
        instructions: &[Instruction],
    ) -> Option<Self> {
        match tx_err {
            TransactionError::InstructionError(idx, InstructionError::Custom(code)) => {
                let instruction = instructions.get(idx as usize)?;
                if instruction.program_id != program::ID {
                    return None;
                }
                DropsetError::from_repr(code as u8)
            }
            _ => None,
        }
    }

    /// Attempts to extract a [`DropsetError`] from a [`ClientError`], verifying that the
    /// failing instruction belongs to the dropset program.
    pub fn from_client_error(error: &ClientError, instructions: &[Instruction]) -> Option<Self> {
        DropsetError::from_transaction_error(transaction_error_from_client(error)?, instructions)
    }
}
