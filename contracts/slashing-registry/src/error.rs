use std::fmt;

#[derive(Debug)]
pub enum ContractError {
    Unauthorized,
    AlreadySlashed,
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::Unauthorized => write!(f, "unauthorized"),
            ContractError::AlreadySlashed => write!(f, "already slashed"),
        }
    }
}

impl std::error::Error for ContractError {}
