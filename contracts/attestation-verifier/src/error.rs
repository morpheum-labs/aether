use std::fmt;

#[derive(Debug)]
pub enum ContractError {
    InvalidQuote,
    DigestMismatch,
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::InvalidQuote => write!(f, "invalid quote"),
            ContractError::DigestMismatch => write!(f, "digest mismatch"),
        }
    }
}

impl std::error::Error for ContractError {}
