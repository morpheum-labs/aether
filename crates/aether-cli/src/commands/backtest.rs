use aether_backtester::{RunBacktest, VectorBacktestEngine};
use aether_common::types::{DataCommitment, ExecutionTier, JobSpec};
use aether_oracle::CommitmentCheckedProvider;
use clap::Args;

#[derive(Args)]
pub struct BacktestArgs {
    #[arg(long, default_value = "10000")]
    pub capital: String,
    #[arg(long, default_value = "42")]
    pub seed: u64,
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

    let spec = JobSpec {
        id: [1u8; 32],
        tier: ExecutionTier::Public,
        data_commitment: DataCommitment {
            merkle_root: root,
            version_cid: cid.into(),
        },
        cargo_lock_hash: None,
        seed: args.seed,
        initial_capital: args.capital,
        fee_bps: 10,
        slippage_bps: 5,
    };

    let engine = VectorBacktestEngine;
    let runner = RunBacktest {
        engine: &engine,
        provider: &provider,
    };
    let result = runner.execute(&spec)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
