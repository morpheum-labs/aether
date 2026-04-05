use aether_common::types::JobSpec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Claimed,
    Completed,
    Slashed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobRecord {
    pub spec: JobSpec,
    pub payload_hash: [u8; 32],
    pub status: JobStatus,
    pub claimant: Option<String>,
}
