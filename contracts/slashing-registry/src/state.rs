use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SlashRecord {
    pub operator: String,
    pub reason: String,
    pub evidence_hash: [u8; 32],
}
