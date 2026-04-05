use aether_common::traits::JobHandler;
use aether_common::types::JobSpec;
use aether_common::{AetherError, AetherResult};
use aether_oracle::CommitmentCheckedProvider;
use aether_tee_runner::TeeRunner;

use crate::config::NodeConfig;

pub struct DefaultJobHandler<'a> {
    pub config: &'a NodeConfig,
    pub provider: &'a CommitmentCheckedProvider,
    pub runner: &'a TeeRunner<CommitmentCheckedProvider>,
}

impl JobHandler for DefaultJobHandler<'_> {
    fn claim_and_execute(&mut self, spec: JobSpec) -> AetherResult<aether_common::traits::JobOutcome> {
        if !self.config.can_claim(spec.tier) {
            return Err(AetherError::Config("node cannot claim this tier".into()));
        }
        let (result, att) = self.runner.run_with_provider(&spec, self.provider)?;
        Ok(aether_common::JobOutcome {
            result,
            attestation: att,
        })
    }
}
