use aether_common::traits::Attester;
use aether_common::types::{AttestedOutput, EnclaveQuote};
use aether_common::{AetherError, AetherResult};
use sha2::{Digest, Sha256};

/// Development attester: binds quote bytes to SHA256(digest || domain separator).
#[derive(Clone, Debug, Default)]
pub struct SoftwareAttester;

impl SoftwareAttester {
    pub fn new() -> Self {
        Self
    }

    fn quote_bytes(digest: &[u8; 32]) -> Vec<u8> {
        let mut h = Sha256::new();
        h.update(b"aether.software-attest.v1");
        h.update(digest);
        h.finalize().to_vec()
    }
}

impl Attester for SoftwareAttester {
    fn attest_result_digest(&self, digest: &[u8; 32]) -> AetherResult<AttestedOutput> {
        Ok(AttestedOutput {
            result_digest: *digest,
            quote: EnclaveQuote {
                bytes: Self::quote_bytes(digest),
            },
        })
    }

    fn verify_quote(&self, quote: &EnclaveQuote, expected_digest: &[u8; 32]) -> AetherResult<()> {
        let expected = Self::quote_bytes(expected_digest);
        if quote.bytes == expected {
            Ok(())
        } else {
            Err(AetherError::Attestation("quote mismatch".into()))
        }
    }
}
