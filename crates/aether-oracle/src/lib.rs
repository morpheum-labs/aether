//! Secure market-data adapter: verifies dataset commitment before exposing rows.

pub mod provider;

pub use provider::CommitmentCheckedProvider;
