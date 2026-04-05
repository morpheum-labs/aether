use aether_common::types::{DataCommitment, JobSpec};
use aether_common::{AetherError, AetherResult};

/// Validates job + dataset commitment before execution (WASM/MWVM hook point).
#[derive(Clone, Debug, Default)]
pub struct JobSandbox;

impl JobSandbox {
    pub fn validate(&self, spec: &JobSpec, commitment: &DataCommitment) -> AetherResult<()> {
        if spec.data_commitment.merkle_root != commitment.merkle_root
            || spec.data_commitment.version_cid != commitment.version_cid
        {
            return Err(AetherError::Sandbox(
                "job data commitment does not match envelope".into(),
            ));
        }
        Ok(())
    }
}
