use aether_attest::SoftwareAttester;
use aether_backtester::VectorBacktestEngine;
use aether_common::traits::{Attester, DeterministicEngine, MarketDataProvider};
use aether_common::types::{AttestedOutput, BacktestResult, ExecutionTier, JobSpec};
use aether_common::{AetherError, AetherResult};

use crate::sandbox::JobSandbox;

/// Routes jobs by [`ExecutionTier`] and applies optional attestation.
#[derive(Clone, Debug)]
pub struct TeeRunner<P> {
    sandbox: JobSandbox,
    engine: VectorBacktestEngine,
    attester: SoftwareAttester,
    provider: std::marker::PhantomData<P>,
}

impl<P> Default for TeeRunner<P> {
    fn default() -> Self {
        Self {
            sandbox: JobSandbox::default(),
            engine: VectorBacktestEngine,
            attester: SoftwareAttester::new(),
            provider: std::marker::PhantomData,
        }
    }
}

impl<P: MarketDataProvider> TeeRunner<P> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run_with_provider(
        &self,
        spec: &JobSpec,
        provider: &P,
    ) -> AetherResult<(BacktestResult, Option<AttestedOutput>)> {
        self.sandbox
            .validate(spec, &spec.data_commitment)?;
        let ohlcv = provider.load_ohlcv(&spec.data_commitment)?;
        let result = DeterministicEngine::run_deterministic(&self.engine, spec, &ohlcv)?;

        match spec.tier {
            ExecutionTier::Public => Ok((result, None)),
            ExecutionTier::Confidential => {
                let att = self
                    .attester
                    .attest_result_digest(&result.result_digest)?;
                if att.result_digest != result.result_digest {
                    return Err(AetherError::Attestation("digest binding mismatch".into()));
                }
                Ok((result, Some(att)))
            }
        }
    }
}
