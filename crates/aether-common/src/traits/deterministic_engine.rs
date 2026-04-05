use crate::errors::AetherResult;
use crate::types::{BacktestResult, JobSpec};

/// Bit-exact deterministic path: fixed-point, pinned toolchain assumptions in `JobSpec`.
pub trait DeterministicEngine {
    fn run_deterministic(
        &self,
        spec: &JobSpec,
        ohlcv: &[(String, String, String, String, String)],
    ) -> AetherResult<BacktestResult>;
}
