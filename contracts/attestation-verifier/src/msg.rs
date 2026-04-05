use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstantiateMsg {
    pub trusted_measurement: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    VerifyAttestation {
        job_id: [u8; 32],
        result_digest: [u8; 32],
        quote: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Verified { job_id: [u8; 32] },
}
