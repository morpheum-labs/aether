use aether_common::types::ExecutionTier;
use aether_common::{AetherError, AetherResult};
use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
pub struct NodeConfig {
    pub operator_id: String,
    pub max_memory_mb: u64,
    pub supports_tee: bool,
    pub preferred_tiers: Vec<ExecutionTier>,
}

impl NodeConfig {
    pub fn from_json_file(path: &Path) -> AetherResult<Self> {
        let raw = std::fs::read_to_string(path)?;
        let c: NodeConfig = serde_json::from_str(&raw)
            .map_err(|e| AetherError::Config(e.to_string()))?;
        Ok(c)
    }

    pub fn default_local() -> Self {
        Self {
            operator_id: "local-dev".into(),
            max_memory_mb: 512,
            supports_tee: false,
            preferred_tiers: vec![ExecutionTier::Public, ExecutionTier::Confidential],
        }
    }

    pub fn can_claim(&self, tier: ExecutionTier) -> bool {
        if !self.preferred_tiers.contains(&tier) {
            return false;
        }
        match tier {
            ExecutionTier::Confidential => self.supports_tee,
            ExecutionTier::Public => true,
        }
    }
}
