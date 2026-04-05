use crate::errors::AetherResult;
use crate::types::DataCommitment;

/// Supplies OHLCV rows for a committed dataset (verified via Merkle root in sandbox).
pub trait MarketDataProvider {
    fn load_ohlcv(
        &self,
        commitment: &DataCommitment,
    ) -> AetherResult<Vec<(String, String, String, String, String)>>;
}
