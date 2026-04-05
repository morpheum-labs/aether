use crate::errors::AetherResult;
use crate::types::{AttestedOutput, BacktestResult, JobSpec};

pub trait JobHandler {
    fn claim_and_execute(&mut self, spec: JobSpec) -> AetherResult<JobOutcome>;
}

#[derive(Clone, Debug)]
pub struct JobOutcome {
    pub result: BacktestResult,
    pub attestation: Option<AttestedOutput>,
}
