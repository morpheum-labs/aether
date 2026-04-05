use aether_common::types::EnclaveQuote;
use aether_common::{AetherError, AetherResult};
use sha2::{Digest, Sha256};

/// On-chain–style verifier over the same binding as [`crate::SoftwareAttester`].
#[derive(Clone, Debug, Default)]
pub struct SoftwareVerifier;

impl SoftwareVerifier {
    pub fn new() -> Self {
        Self
    }

    pub fn expected_quote_bytes(digest: &[u8; 32]) -> Vec<u8> {
        let mut h = Sha256::new();
        h.update(b"aether.software-attest.v1");
        h.update(digest);
        h.finalize().to_vec()
    }

    pub fn verify(quote: &EnclaveQuote, expected_digest: &[u8; 32]) -> AetherResult<()> {
        let expected = Self::expected_quote_bytes(expected_digest);
        if quote.bytes == expected {
            Ok(())
        } else {
            Err(AetherError::Attestation("verification failed".into()))
        }
    }
}
