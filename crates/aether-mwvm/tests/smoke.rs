use aether_common::utils::crypto::sha256_32;
use aether_mwvm::{instantiate_job_wasm, verify_sha256_and_instantiate};

const MINIMAL_AGENT_WAT: &[u8] = include_bytes!("fixtures/minimal_agent.wat");

#[test]
fn minimal_fixture_instantiates() {
    instantiate_job_wasm(MINIMAL_AGENT_WAT).expect("MWVM should load minimal test fixture");
}

#[test]
fn verify_sha256_rejects_wrong_digest() {
    let bad = [1u8; 32];
    let err = verify_sha256_and_instantiate(MINIMAL_AGENT_WAT, &bad).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("wasm_sha256"),
        "unexpected error message: {msg}"
    );
}

#[test]
fn verify_sha256_accepts_matching_digest() {
    let digest = sha256_32(MINIMAL_AGENT_WAT);
    verify_sha256_and_instantiate(MINIMAL_AGENT_WAT, &digest).unwrap();
}
