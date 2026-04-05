//! Phase 0 — sandbox validation paths (commitment + optional WASM hash).

use aether_common::types::{DataCommitment, ExecutionTier, JobSpec};
use aether_common::utils::crypto::sha256_32;
use aether_mwvm::SandboxLimits;
use aether_tee_runner::sandbox::JobSandbox;

const MINIMAL_WAT: &[u8] = include_bytes!("../../aether-mwvm/tests/fixtures/minimal_agent.wat");

fn sample_spec(wasm_sha256: Option<[u8; 32]>) -> JobSpec {
    JobSpec {
        id: [2u8; 32],
        tier: ExecutionTier::Public,
        data_commitment: DataCommitment {
            merkle_root: [9u8; 32],
            version_cid: "cid".into(),
        },
        wasm_sha256,
        cargo_lock_hash: None,
        seed: 0,
        initial_capital: "1".into(),
        fee_bps: 0,
        slippage_bps: 0,
    }
}

#[test]
fn rejects_commitment_mismatch() {
    let sandbox = JobSandbox::default();
    let spec = sample_spec(None);
    let bad = DataCommitment {
        merkle_root: [0u8; 32],
        version_cid: "cid".into(),
    };
    let err = sandbox
        .validate(&spec, &bad, None)
        .expect_err("mismatch");
    assert!(
        err.to_string().contains("commitment"),
        "unexpected: {err}"
    );
}

#[test]
fn wasm_sha256_without_bytes_errors() {
    let sandbox = JobSandbox::default();
    let digest = sha256_32(MINIMAL_WAT);
    let spec = sample_spec(Some(digest));
    let err = sandbox
        .validate(&spec, &spec.data_commitment, None)
        .expect_err("missing wasm");
    assert!(err.to_string().contains("wasm payload"));
}

#[test]
fn wasm_sha256_mismatch_errors() {
    let sandbox = JobSandbox::default();
    let spec = sample_spec(Some([1u8; 32]));
    let commitment = spec.data_commitment.clone();
    let err = sandbox
        .validate(&spec, &commitment, Some(MINIMAL_WAT))
        .expect_err("hash mismatch");
    assert!(err.to_string().contains("wasm_sha256"));
}

#[test]
fn wasm_preflight_ok_for_minimal_fixture() {
    let sandbox = JobSandbox::with_limits(SandboxLimits {
        max_memory_bytes: 64 * 1024 * 1024,
        fuel_units: 10_000_000,
    });
    let digest = sha256_32(MINIMAL_WAT);
    let spec = sample_spec(Some(digest));
    let commitment = spec.data_commitment.clone();
    sandbox
        .validate(&spec, &commitment, Some(MINIMAL_WAT))
        .expect("valid wasm preflight");
}
