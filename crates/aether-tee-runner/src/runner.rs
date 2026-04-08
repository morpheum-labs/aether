use aether_attest::SoftwareAttester;
use aether_backtester::{placeholder_guest_strategy_result, VectorBacktestEngine};
use aether_common::traits::{Attester, DeterministicEngine, MarketDataProvider};
use aether_common::types::{AttestedOutput, BacktestResult, ExecutionTier, JobSpec};
use aether_common::{AetherError, AetherResult};
use aether_mwvm::{GuestReplay, SandboxLimits};

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

    /// Operator-facing limits (memory + fuel) for WASM preflight; derived from `NodeConfig::max_memory_mb`.
    #[must_use]
    pub fn with_sandbox_limits(limits: SandboxLimits) -> Self {
        Self {
            sandbox: JobSandbox::with_limits(limits),
            engine: VectorBacktestEngine,
            attester: SoftwareAttester::new(),
            provider: std::marker::PhantomData,
        }
    }

    pub fn run_with_provider(
        &self,
        spec: &JobSpec,
        provider: &P,
        wasm: Option<&[u8]>,
    ) -> AetherResult<(BacktestResult, Option<AttestedOutput>)> {
        self.sandbox
            .validate(spec, &spec.data_commitment, wasm)?;
        let ohlcv = provider.load_ohlcv(&spec.data_commitment)?;

        let result = if let Some(bytes) = wasm {
            if ohlcv.len() < 3 {
                return Err(AetherError::BacktestEngine(
                    "need at least 3 bars (same threshold as VectorBacktestEngine)".into(),
                ));
            }
            aether_mwvm::run_guest_strategy_bar_loop_with_limits(
                bytes,
                self.sandbox.limits(),
                GuestReplay::Ohlcv(ohlcv.as_slice()),
            )?;
            placeholder_guest_strategy_result(spec, ohlcv.len())?
        } else {
            DeterministicEngine::run_deterministic(&self.engine, spec, &ohlcv)?
        };

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
