use std::fs;
use std::path::PathBuf;

use aether_backtester::{RunBacktest, VectorBacktestEngine};
use aether_common::types::{DataCommitment, ExecutionTier, JobSpec};
use aether_common::utils::crypto::sha256_32;
use aether_oracle::CommitmentCheckedProvider;
use aether_tee_runner::{SandboxLimits, TeeRunner};
use clap::Args;

#[derive(Args)]
pub struct BacktestArgs {
    #[arg(long, default_value = "10000")]
    pub capital: String,
    #[arg(long, default_value = "42")]
    pub seed: u64,
    /// Path to a `.wasm` strategy blob (e.g. future AgentScript output). Sets `JobSpec::wasm_sha256` and runs sandbox preflight via `TeeRunner`.
    #[arg(long, value_name = "PATH")]
    pub wasm: Option<PathBuf>,
    /// Max linear memory per memory object during WASM preflight (MiB).
    #[arg(long, default_value_t = 64)]
    pub wasm_max_memory_mb: u64,
    /// Wasmtime fuel units during preflight.
    #[arg(long, default_value_t = 10_000_000_u64)]
    pub wasm_fuel: u64,
}

pub fn run(args: BacktestArgs) -> Result<(), Box<dyn std::error::Error>> {
    let provider = CommitmentCheckedProvider::new();
    let rows: Vec<_> = (0..40)
        .map(|i| {
            let o = format!("{}", 100 + i);
            let h = format!("{}", 101 + i);
            let l = format!("{}", 99 + i);
            let c = format!("{}", 100 + i);
            let v = "1000".to_string();
            (o, h, l, c, v)
        })
        .collect();
    let cid = "cli-demo";
    let root = provider.publish(cid, rows);

    let wasm_bytes = args
        .wasm
        .as_ref()
        .map(|p| fs::read(p))
        .transpose()?;

    let wasm_sha256 = wasm_bytes
        .as_ref()
        .map(|b| sha256_32(b));

    let spec = JobSpec {
        id: [1u8; 32],
        tier: ExecutionTier::Public,
        data_commitment: DataCommitment {
            merkle_root: root,
            version_cid: cid.into(),
        },
        wasm_sha256,
        cargo_lock_hash: None,
        seed: args.seed,
        initial_capital: args.capital,
        fee_bps: 10,
        slippage_bps: 5,
    };

    if wasm_bytes.is_some() {
        let limits = SandboxLimits {
            max_memory_bytes: args.wasm_max_memory_mb.saturating_mul(1024 * 1024),
            fuel_units: args.wasm_fuel,
        };
        let runner = TeeRunner::with_sandbox_limits(limits);
        let wasm_slice = wasm_bytes.as_ref().map(|v| v.as_slice());
        let (result, _att) = runner.run_with_provider(&spec, &provider, wasm_slice)?;
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let engine = VectorBacktestEngine;
        let runner = RunBacktest {
            engine: &engine,
            provider: &provider,
        };
        let result = runner.execute(&spec)?;
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}
