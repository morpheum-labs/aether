use aether_common::traits::{BacktestEngine, MarketDataProvider};
use aether_common::types::JobSpec;
use aether_common::{AetherError, AetherResult};

/// Orchestrates data loading and engine execution (application use case).
pub struct RunBacktest<'a, E, P> {
    pub engine: &'a E,
    pub provider: &'a P,
}

impl<'a, E, P> RunBacktest<'a, E, P>
where
    E: BacktestEngine,
    P: MarketDataProvider,
{
    pub fn execute(&self, spec: &JobSpec) -> AetherResult<aether_common::BacktestResult> {
        let ohlcv = self
            .provider
            .load_ohlcv(&spec.data_commitment)
            .map_err(|e| AetherError::MarketData(e.to_string()))?;
        let (result, _, _) = self
            .engine
            .run(spec, &ohlcv)
            .map_err(|e| AetherError::BacktestEngine(e.to_string()))?;
        Ok(result)
    }
}
