use aether_common::types::{DataCommitment, JobSpec};
use aether_common::{AetherError, AetherResult};
use aether_mwvm::SandboxLimits;

/// Validates job + dataset commitment before execution (WASM/MWVM hook point).
#[derive(Clone, Debug)]
pub struct JobSandbox {
    limits: SandboxLimits,
}

impl Default for JobSandbox {
    fn default() -> Self {
        Self {
            limits: SandboxLimits::default(),
        }
    }
}

impl JobSandbox {
    #[must_use]
    pub fn with_limits(limits: SandboxLimits) -> Self {
        Self { limits }
    }

    #[must_use]
    pub fn limits(&self) -> &SandboxLimits {
        &self.limits
    }

    pub fn validate(
        &self,
        spec: &JobSpec,
        commitment: &DataCommitment,
        wasm: Option<&[u8]>,
    ) -> AetherResult<()> {
        if spec.data_commitment.merkle_root != commitment.merkle_root
            || spec.data_commitment.version_cid != commitment.version_cid
        {
            return Err(AetherError::Sandbox(
                "job data commitment does not match envelope".into(),
            ));
        }

        if let Some(expected) = spec.wasm_sha256.as_ref() {
            let bytes = wasm.ok_or_else(|| {
                AetherError::Sandbox(
                    "JobSpec::wasm_sha256 is set but no wasm payload was supplied".into(),
                )
            })?;
            aether_mwvm::verify_sha256_and_instantiate_with_limits(bytes, expected, &self.limits)?;
        }

        Ok(())
    }
}
