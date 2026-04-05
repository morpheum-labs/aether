mod config;
mod job_handler;
mod p2p;

use aether_common::traits::JobHandler;
use aether_common::types::{DataCommitment, ExecutionTier, JobSpec};
use aether_common::utils::crypto::sha256_32;
use aether_oracle::CommitmentCheckedProvider;
use aether_tee_runner::{SandboxLimits, TeeRunner};
use std::env;
use std::fs;
use std::path::Path;

use crate::config::NodeConfig;
use crate::job_handler::DefaultJobHandler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg_path = env::args().nth(1);
    let wasm_path = env::args().nth(2);

    let cfg = match cfg_path.as_deref() {
        Some(path) => NodeConfig::from_json_file(Path::new(path))?,
        None => NodeConfig::default_local(),
    };

    let wasm_bytes = match wasm_path.as_deref() {
        Some(p) => Some(fs::read(Path::new(p))?),
        None => None,
    };

    eprintln!(
        "aether-node operator={} max_memory_mb={} wasm={}",
        cfg.operator_id,
        cfg.max_memory_mb,
        wasm_path.as_deref().unwrap_or("(none)")
    );

    let provider = CommitmentCheckedProvider::new();
    let sample: Vec<_> = (0..40)
        .map(|i| {
            let o = format!("{}", 100 + i);
            let h = format!("{}", 101 + i);
            let l = format!("{}", 99 + i);
            let c = format!("{}", 100 + i);
            let v = "1000".to_string();
            (o, h, l, c, v)
        })
        .collect();
    let cid = "demo-dataset-v1";
    let root = provider.publish(cid, sample);

    let wasm_sha256 = wasm_bytes.as_ref().map(|b| sha256_32(b));

    let spec = JobSpec {
        id: [7u8; 32],
        tier: ExecutionTier::Public,
        data_commitment: DataCommitment {
            merkle_root: root,
            version_cid: cid.into(),
        },
        wasm_sha256,
        cargo_lock_hash: None,
        seed: 42,
        initial_capital: "10000".into(),
        fee_bps: 10,
        slippage_bps: 5,
    };

    let sandbox_limits = SandboxLimits::from_node_max_memory_mb(cfg.max_memory_mb);
    let runner = TeeRunner::with_sandbox_limits(sandbox_limits);
    let mut handler = DefaultJobHandler {
        config: &cfg,
        provider: &provider,
        runner: &runner,
    };

    let outcome = handler.claim_and_execute(spec, wasm_bytes.as_deref())?;
    println!("{}", serde_json::to_string_pretty(&outcome.result)?);
    if let Some(a) = outcome.attestation {
        println!("attested digest: {:02x?}", a.result_digest);
    }
    Ok(())
}
