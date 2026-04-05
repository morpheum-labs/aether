use aether_common::AetherError;
use std::fmt;

#[derive(Debug)]
pub enum ContractError {
    Common(AetherError),
    Unauthorized,
    JobNotFound,
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::Common(e) => write!(f, "{e}"),
            ContractError::Unauthorized => write!(f, "unauthorized"),
            ContractError::JobNotFound => write!(f, "job not found"),
        }
    }
}

impl std::error::Error for ContractError {}

impl From<AetherError> for ContractError {
    fn from(e: AetherError) -> Self {
        ContractError::Common(e)
    }
}
