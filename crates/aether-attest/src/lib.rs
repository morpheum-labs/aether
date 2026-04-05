//! Attestation helpers: production paths would integrate TDX / SEV-SNP quotes.

pub mod attestation;
pub mod verifier;

pub use attestation::SoftwareAttester;
pub use verifier::SoftwareVerifier;
