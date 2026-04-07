//! Phase 1 — when WASM bytes are supplied, [`TeeRunner`] drives the guest `init` + per-bar `step`
//! (see `aether_mwvm::run_guest_strategy_bar_loop_with_limits`) before returning a placeholder
//! [`BacktestResult`] from `aether_backtester::placeholder_guest_strategy_result`.

use aether_common::types::{DataCommitment, ExecutionTier, JobSpec};
use aether_common::utils::crypto::sha256_32;
use aether_mwvm::SandboxLimits;
use aether_oracle::CommitmentCheckedProvider;
use aether_tee_runner::TeeRunner;

const TINY_STRATEGY_GUEST_WASM: &[u8] = include_bytes!("../../aether-mwvm/tests/fixtures/tiny_strategy_guest.wasm");

fn sample_rows(n: usize) -> Vec<(String, String, String, String, String)> {
    (0..n)
        .map(|i| {
            let c = format!("{}", 100 + i);
            (
                "100".into(),
                "101".into(),
                "99".into(),
                c,
                "1000".into(),
            )
        })
        .collect()
}

#[test]
fn tee_runner_invokes_guest_when_wasm_attached() {
    let provider = CommitmentCheckedProvider::new();
    let rows = sample_rows(5);
    let cid = "guest-runner-cid";
    let root = provider.publish(cid, rows);

    let digest = sha256_32(TINY_STRATEGY_GUEST_WASM);
    let spec = JobSpec {
        id: [7u8; 32],
        tier: ExecutionTier::Public,
        data_commitment: DataCommitment {
            merkle_root: root,
            version_cid: cid.into(),
        },
        wasm_sha256: Some(digest),
        cargo_lock_hash: None,
        seed: 1,
        initial_capital: "10000".into(),
        fee_bps: 10,
        slippage_bps: 5,
    };

    let runner = TeeRunner::<CommitmentCheckedProvider>::with_sandbox_limits(SandboxLimits {
        max_memory_bytes: 64 * 1024 * 1024,
        fuel_units: 50_000_000,
    });

    let (result, att) = runner
        .run_with_provider(&spec, &provider, Some(TINY_STRATEGY_GUEST_WASM))
        .expect("guest bar loop + placeholder backtest");

    assert!(att.is_none());
    assert_eq!(result.equity_curve.len(), 5);
    assert!(result.trades.is_empty());
    assert_eq!(result.equity_curve[0], "10000");
    assert_ne!(result.result_digest, [0u8; 32]);
}
