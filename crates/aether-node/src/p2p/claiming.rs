use aether_common::types::{ExecutionTier, JobSpec};

use super::discovery::JobSource;

/// Stub claimer: accepts first job matching node capabilities.
pub struct StakeWeightedClaimer<S> {
    pub source: S,
    pub supports_tee: bool,
}

impl<S: JobSource> StakeWeightedClaimer<S> {
    pub fn try_claim(&mut self) -> Option<JobSpec> {
        while let Some(job) = self.source.next_job() {
            let ok = match job.tier {
                ExecutionTier::Public => true,
                ExecutionTier::Confidential => self.supports_tee,
            };
            if ok {
                return Some(job);
            }
        }
        None
    }
}
