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

#[cfg(test)]
mod tests {
    use super::*;
    use aether_common::types::{DataCommitment, ExecutionTier, JobSpec};

    fn sample_job(tier: ExecutionTier) -> JobSpec {
        JobSpec {
            id: [0u8; 32],
            tier,
            data_commitment: DataCommitment {
                merkle_root: [0u8; 32],
                version_cid: "test".into(),
            },
            wasm_sha256: None,
            cargo_lock_hash: None,
            seed: 1,
            initial_capital: "1".into(),
            fee_bps: 0,
            slippage_bps: 0,
        }
    }

    #[test]
    fn claimer_skips_confidential_when_no_tee() {
        use crate::p2p::discovery::VecJobSource;

        let jobs = vec![
            sample_job(ExecutionTier::Confidential),
            sample_job(ExecutionTier::Public),
        ];
        let mut claimer = StakeWeightedClaimer {
            source: VecJobSource::new(jobs),
            supports_tee: false,
        };
        let got = claimer.try_claim().expect("public job after skip");
        assert_eq!(got.tier, ExecutionTier::Public);
    }

    #[test]
    fn claimer_accepts_confidential_with_tee() {
        use crate::p2p::discovery::VecJobSource;

        let jobs = vec![sample_job(ExecutionTier::Confidential)];
        let mut claimer = StakeWeightedClaimer {
            source: VecJobSource::new(jobs),
            supports_tee: true,
        };
        let got = claimer.try_claim().expect("confidential ok");
        assert_eq!(got.tier, ExecutionTier::Confidential);
    }
}
