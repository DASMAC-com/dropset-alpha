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
use solana_instruction_error::InstructionError;
use solana_transaction_error::TransactionError;

use crate::error::DropsetError;

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
    /// Attempts to extract a [`DropsetError`] from a [`TransactionError`].
    pub fn from_transaction_error(tx_err: TransactionError) -> Option<Self> {
        match tx_err {
            TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
                DropsetError::from_repr(code as u8)
            }
            _ => None,
        }
    }

    /// Attempts to extract a [`DropsetError`] from a [`ClientError`].
    pub fn from_client_error(error: &ClientError) -> Option<Self> {
        DropsetError::from_transaction_error(transaction_error_from_client(error)?)
    }
}
