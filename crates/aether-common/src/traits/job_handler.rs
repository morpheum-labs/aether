use crate::errors::AetherResult;
use crate::types::{AttestedOutput, BacktestResult, JobSpec};

pub trait JobHandler {
    /// `wasm` must be supplied when `spec.wasm_sha256` is set (same bytes as hashed).
    fn claim_and_execute(
        &mut self,
        spec: JobSpec,
        wasm: Option<&[u8]>,
    ) -> AetherResult<JobOutcome>;
}

#[derive(Clone, Debug)]
pub struct JobOutcome {
    pub result: BacktestResult,
    pub attestation: Option<AttestedOutput>,
}
