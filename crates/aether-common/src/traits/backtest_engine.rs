use crate::errors::AetherResult;
use crate::types::{BacktestResult, JobSpec, SimulationState, Trade};

/// Runs a simulation for a job. Implementations may be vectorized or stepped.
pub trait BacktestEngine {
    fn run(
        &self,
        spec: &JobSpec,
        ohlcv: &[(String, String, String, String, String)],
    ) -> AetherResult<(BacktestResult, Vec<SimulationState>, Vec<Trade>)>;
}
