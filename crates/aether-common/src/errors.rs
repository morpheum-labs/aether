use thiserror::Error;

#[derive(Debug, Error)]
pub enum AetherError {
    #[error("invalid job specification: {0}")]
    InvalidJobSpec(String),
    #[error("market data: {0}")]
    MarketData(String),
    #[error("backtest engine: {0}")]
    BacktestEngine(String),
    #[error("sandbox: {0}")]
    Sandbox(String),
    #[error("attestation: {0}")]
    Attestation(String),
    #[error("configuration: {0}")]
    Config(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub type AetherResult<T> = Result<T, AetherError>;
