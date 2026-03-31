//! Interprets RPC and on-chain errors into readable `dropset`/Solana instruction error messages.

use std::fmt::Display;

use dropset_interface::program;

use crate::{
    fmt_kv,
    pretty::instruction::KnownProgram,
    transactions::TransactionSubmitError,
    LogColor,
};

pub struct PrettyTransactionError<'a>(&'a TransactionSubmitError);

impl<'a> PrettyTransactionError<'a> {
    pub fn new(error: &'a TransactionSubmitError) -> Self {
        Self(error)
    }
}

impl Display for PrettyTransactionError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (error_type, program_id, error) = match self.0 {
            TransactionSubmitError::Dropset(e) => ("DropsetError", program::ID, e.to_string()),
            TransactionSubmitError::Token(e) => {
                ("TokenError", spl_token_interface::ID, e.to_string())
            }
            TransactionSubmitError::Token2022(e) => (
                "Token2022Error",
                spl_token_2022_interface::ID,
                e.to_string(),
            ),
            TransactionSubmitError::AssociatedTokenAccount(e) => (
                "AssociatedTokenAccountError",
                spl_associated_token_account_interface::program::ID,
                e.as_ref().into(),
            ),
            TransactionSubmitError::Program { program_id, error } => {
                ("ProgramError", *program_id, error.to_string())
            }
            TransactionSubmitError::Other(info) => {
                return writeln!(f, "{info}");
            }
        };

        let program_name = KnownProgram::from_program_id(&program_id)
            .map(|k| format!("{k}"))
            .unwrap_or("UnknownProgram".into());

        let message = format!("Program: {program_name}, error: {error}");
        let error_message = fmt_kv!(error_type, message, LogColor::Error);
        writeln!(f, "{error_message}")
    }
}
