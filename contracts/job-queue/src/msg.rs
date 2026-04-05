use aether_common::types::JobSpec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SubmitJob { spec: JobSpec, payload_hash: [u8; 32] },
    ClaimJob { job_id: [u8; 32] },
    ReportResult {
        job_id: [u8; 32],
        result_digest: [u8; 32],
        quote: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Job { job_id: [u8; 32] },
    QueueHead {},
}
