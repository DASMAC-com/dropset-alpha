//! See [`ErrorType`].

use crate::{
    parse::error_path::ErrorPath,
    render::Feature,
};

/// Maps high-level instruction validation errors to concrete `ProgramError` variants
/// for each supported feature/target.
pub enum ErrorType {
    IncorrectNumAccounts,
    InvalidInstructionData,
}

impl ErrorType {
    pub fn to_path(&self, feature: Feature) -> ErrorPath {
        let base = match feature {
            Feature::Client => "::solana_sdk::program_error::ProgramError",
            Feature::Pinocchio => "::pinocchio::error::ProgramError",
            Feature::SolanaProgram => "::solana_sdk::program_error::ProgramError",
        };
        match self {
            ErrorType::InvalidInstructionData => ErrorPath::new(base, "InvalidInstructionData"),
            ErrorType::IncorrectNumAccounts => ErrorPath::new(base, "NotEnoughAccountKeys"),
        }
    }
}
