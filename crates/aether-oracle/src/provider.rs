use aether_common::traits::MarketDataProvider;
use aether_common::types::DataCommitment;
use aether_common::{AetherError, AetherResult};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory mirror keyed by CID; verifies Merkle root of concatenated row hashes.
#[derive(Debug, Default)]
pub struct CommitmentCheckedProvider {
    rows: RwLock<HashMap<String, Vec<(String, String, String, String, String)>>>,
}

impl CommitmentCheckedProvider {
    pub fn new() -> Self {
        Self {
            rows: RwLock::new(HashMap::new()),
        }
    }

    fn row_hash(row: &(String, String, String, String, String)) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(row.0.as_bytes());
        h.update(row.1.as_bytes());
        h.update(row.2.as_bytes());
        h.update(row.3.as_bytes());
        h.update(row.4.as_bytes());
        let out = h.finalize();
        let mut a = [0u8; 32];
        a.copy_from_slice(&out);
        a
    }

    fn merkle_root(rows: &[(String, String, String, String, String)]) -> [u8; 32] {
        if rows.is_empty() {
            return [0u8; 32];
        }
        let mut level: Vec<[u8; 32]> = rows.iter().map(Self::row_hash).collect();
        while level.len() > 1 {
            let mut next = Vec::new();
            let mut i = 0;
            while i < level.len() {
                let left = level[i];
                let right = if i + 1 < level.len() {
                    level[i + 1]
                } else {
                    level[i]
                };
                let mut h = Sha256::new();
                h.update(left);
                h.update(right);
                let out = h.finalize();
                let mut a = [0u8; 32];
                a.copy_from_slice(&out);
                next.push(a);
                i += 2;
            }
            level = next;
        }
        level[0]
    }

    /// Insert or replace dataset for `cid` and return computed root (for test setup).
    pub fn publish(
        &self,
        cid: &str,
        rows: Vec<(String, String, String, String, String)>,
    ) -> [u8; 32] {
        let root = Self::merkle_root(&rows);
        let mut g = self.rows.write().expect("lock");
        g.insert(cid.to_string(), rows);
        root
    }
}

impl MarketDataProvider for CommitmentCheckedProvider {
    fn load_ohlcv(
        &self,
        commitment: &DataCommitment,
    ) -> AetherResult<Vec<(String, String, String, String, String)>> {
        let g = self.rows.read().map_err(|e| AetherError::MarketData(e.to_string()))?;
        let rows = g
            .get(&commitment.version_cid)
            .cloned()
            .ok_or_else(|| AetherError::MarketData("unknown dataset cid".into()))?;
        let root = Self::merkle_root(&rows);
        if root != commitment.merkle_root {
            return Err(AetherError::MarketData(
                "merkle root mismatch for dataset".into(),
            ));
        }
        Ok(rows)
    }
}
