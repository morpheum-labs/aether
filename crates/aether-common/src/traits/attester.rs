use crate::errors::AetherResult;
use crate::types::{AttestedOutput, EnclaveQuote};

pub trait Attester {
    fn attest_result_digest(&self, digest: &[u8; 32]) -> AetherResult<AttestedOutput>;

    fn verify_quote(&self, quote: &EnclaveQuote, expected_digest: &[u8; 32]) -> AetherResult<()>;
}
