//! MWVM integration for Aether backtest jobs.
//!
//! By default this crate uses **wasmtime** to compile and instantiate WASM (fast CI, no
//! dependency on the full `mwvm-sdk` / `morpheum-primitives` graph). Enable feature
//! **`mwvm-full`** to run the same path as production agents ([`mwvm_sdk::Agent`] with all
//! host functions registered).

#![forbid(unsafe_code)]

mod aether_guest_stubs;

pub use aether_guest_stubs::link_aether_guest_abi_v0;

use aether_common::utils::crypto::sha256_32;
use aether_common::{AetherError, AetherResult};

#[cfg(feature = "mwvm-full")]
use mwvm_sdk::{Agent, SdkConfig};

/// Per-job wasmtime caps: linear memory per memory object and fuel for execution during preflight.
#[derive(Clone, Debug)]
pub struct SandboxLimits {
    /// Max bytes a single linear memory may grow to (wasmtime `StoreLimits`).
    pub max_memory_bytes: u64,
    /// Fuel units for the store (instantiation may consume a small amount).
    pub fuel_units: u64,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024,
            fuel_units: 10_000_000,
        }
    }
}

impl SandboxLimits {
    /// Derive limits from operator `max_memory_mb` and a simple fuel heuristic.
    #[must_use]
    pub fn from_node_max_memory_mb(max_memory_mb: u64) -> Self {
        let max_memory_bytes = max_memory_mb.saturating_mul(1024 * 1024);
        let fuel_units = max_memory_mb.saturating_mul(100_000).max(1_000_000);
        Self {
            max_memory_bytes,
            fuel_units,
        }
    }
}

/// Instantiate WASM: full MWVM engine when `mwvm-full` is enabled, otherwise wasmtime-only
/// (sufficient for modules without `morpheum::*` imports).
pub fn instantiate_job_wasm(wasm: &[u8]) -> AetherResult<()> {
    instantiate_job_wasm_with_limits(wasm, &SandboxLimits::default())
}

pub fn instantiate_job_wasm_with_limits(wasm: &[u8], limits: &SandboxLimits) -> AetherResult<()> {
    instantiate_job_wasm_inner(wasm, limits)
}

#[cfg(feature = "mwvm-full")]
fn instantiate_job_wasm_inner(wasm: &[u8], _limits: &SandboxLimits) -> AetherResult<()> {
    let _agent = Agent::builder()
        .wasm_bytes(wasm.to_vec())
        .config(
            SdkConfig::new()
                .model_serving(false)
                .tee_simulation(false)
                .max_instances(64),
        )
        .build()
        .map_err(|e| AetherError::Sandbox(format!("mwvm-sdk instantiate: {e}")))?;
    Ok(())
}

#[cfg(not(feature = "mwvm-full"))]
fn instantiate_job_wasm_inner(wasm: &[u8], limits: &SandboxLimits) -> AetherResult<()> {
    use wasmtime::{
        Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder,
    };

    let max_mem = usize::try_from(limits.max_memory_bytes).unwrap_or(usize::MAX);

    let mut config = Config::new();
    config.consume_fuel(true);
    let engine = Engine::new(&config).map_err(|e| AetherError::Sandbox(format!("wasm engine: {e}")))?;

    let module = Module::new(&engine, wasm).map_err(|e| AetherError::Sandbox(format!("wasm compile: {e}")))?;
    let mut linker: Linker<StoreState> = Linker::new(&engine);
    aether_guest_stubs::link_aether_guest_abi_v0(&mut linker).map_err(|e| {
        AetherError::Sandbox(format!("wasm link aether stubs: {e}"))
    })?;

    let store_limits: StoreLimits = StoreLimitsBuilder::new()
        .memory_size(max_mem)
        .instances(2)
        .build();

    let mut store = Store::new(
        &engine,
        StoreState {
            limits: store_limits,
        },
    );
    store.limiter(|s| &mut s.limits);

    store
        .set_fuel(limits.fuel_units)
        .map_err(|e| AetherError::Sandbox(format!("wasm fuel: {e}")))?;

    linker
        .instantiate(&mut store, &module)
        .map_err(|e| AetherError::Sandbox(format!("wasm instantiate: {e}")))?;
    Ok(())
}

#[cfg(not(feature = "mwvm-full"))]
#[derive(Debug)]
struct StoreState {
    limits: wasmtime::StoreLimits,
}

/// Verify SHA-256 of `wasm` matches `expected`, then run [`instantiate_job_wasm`].
pub fn verify_sha256_and_instantiate(wasm: &[u8], expected: &[u8; 32]) -> AetherResult<()> {
    verify_sha256_and_instantiate_with_limits(wasm, expected, &SandboxLimits::default())
}

pub fn verify_sha256_and_instantiate_with_limits(
    wasm: &[u8],
    expected: &[u8; 32],
    limits: &SandboxLimits,
) -> AetherResult<()> {
    let digest = sha256_32(wasm);
    if digest != *expected {
        return Err(AetherError::Sandbox(
            "wasm bytes do not match JobSpec::wasm_sha256".into(),
        ));
    }
    instantiate_job_wasm_with_limits(wasm, limits)
}
