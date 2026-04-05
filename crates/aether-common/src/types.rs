use serde::{Deserialize, Serialize};

/// Public jobs skip TEE; confidential jobs expect attestation and encrypted payloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionTier {
    Public,
    Confidential,
}

/// Canonical dataset binding: Merkle root + content identifier (IPFS CID, Arweave tx, etc.).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataCommitment {
    pub merkle_root: [u8; 32],
    pub version_cid: String,
}

/// Submitted job description (on-chain or off-chain envelope).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JobSpec {
    pub id: [u8; 32],
    pub tier: ExecutionTier,
    pub data_commitment: DataCommitment,
    /// When set, the job must include WASM bytes whose SHA-256 matches (MWVM preflight).
    #[serde(default)]
    pub wasm_sha256: Option<[u8; 32]>,
    /// Optional pinned lockfile hash for reproducible builds (hex or raw digest).
    pub cargo_lock_hash: Option<[u8; 32]>,
    /// Deterministic RNG / simulation seed (e.g. derived from job id + user nonce).
    pub seed: u64,
    pub initial_capital: String,
    pub fee_bps: u16,
    pub slippage_bps: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trade {
    pub bar_index: u64,
    pub side: Side,
    pub qty: String,
    pub price: String,
    pub fee_paid: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub qty: String,
    pub avg_entry: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationState {
    pub bar_index: u64,
    pub cash: String,
    pub position: Option<Position>,
    pub equity: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Metrics {
    pub total_return_bps: i64,
    pub max_drawdown_bps: u64,
    pub sharpe_approx: String,
    pub win_rate_bps: u64,
    pub trade_count: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BacktestResult {
    pub job_id: [u8; 32],
    pub metrics: Metrics,
    pub equity_curve: Vec<String>,
    pub trades: Vec<Trade>,
    pub result_digest: [u8; 32],
}

/// Opaque TEE quote blob (platform-specific).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnclaveQuote {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestedOutput {
    pub result_digest: [u8; 32],
    pub quote: EnclaveQuote,
}
