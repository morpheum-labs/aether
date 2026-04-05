use std::fs;
use std::path::PathBuf;

use aether_common::utils::crypto::sha256_32;
use aether_mwvm::{instantiate_job_wasm_with_limits, SandboxLimits};
use clap::Args;

#[derive(Args)]
pub struct VerifyWasmArgs {
    /// Path to `.wasm` / `.wat` strategy artifact
    #[arg(value_name = "PATH")]
    pub path: PathBuf,
    /// Expected SHA-256 (64 hex chars, optional `0x` prefix). When set, file must match before instantiate.
    #[arg(long, value_name = "HEX")]
    pub expect_sha256: Option<String>,
    #[arg(long, default_value_t = 64)]
    pub max_memory_mb: u64,
    #[arg(long, default_value_t = 10_000_000_u64)]
    pub fuel: u64,
}

fn parse_sha256_hex(s: &str) -> Result<[u8; 32], String> {
    let s = s.trim();
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 64 {
        return Err(format!("expected 64 hex chars, got {}", s.len()));
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        let chunk = &s[i * 2..i * 2 + 2];
        out[i] = u8::from_str_radix(chunk, 16)
            .map_err(|_| format!("invalid hex at byte {i}"))?;
    }
    Ok(out)
}

pub fn run(args: VerifyWasmArgs) -> Result<(), Box<dyn std::error::Error>> {
    let wasm = fs::read(&args.path)?;
    let digest = sha256_32(&wasm);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();

    if let Some(ref expected) = args.expect_sha256 {
        let want = parse_sha256_hex(expected).map_err(|e| e)?;
        if want != digest {
            return Err(format!("SHA-256 mismatch: file={hex} expected={expected}").into());
        }
    } else {
        println!("sha256={hex}");
    }

    let limits = SandboxLimits {
        max_memory_bytes: args.max_memory_mb.saturating_mul(1024 * 1024),
        fuel_units: args.fuel,
    };
    instantiate_job_wasm_with_limits(&wasm, &limits)?;
    println!("ok: instantiate under sandbox limits");
    Ok(())
}
