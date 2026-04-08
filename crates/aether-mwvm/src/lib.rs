//! MWVM integration for Aether backtest jobs.
//!
//! By default this crate uses **wasmtime** to compile and instantiate WASM (fast CI, no
//! dependency on the full `mwvm-sdk` / `morpheum-primitives` graph). Enable feature
//! **`mwvm-full`** to run the same path as production agents ([`mwvm_sdk::Agent`] with all
//! host functions registered).

#![forbid(unsafe_code)]

mod aether_guest_stubs;
mod bar_series_host;

pub use aether_guest_stubs::link_aether_guest_abi_v0;
pub use bar_series_host::{BarFeedState, BarRow};

use aether_common::guest_abi::{EXPORT_INIT, EXPORT_STEP};
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

/// How the host should drive `aether::*` imports across guest `step` calls.
#[derive(Clone, Debug)]
pub enum GuestReplay<'a> {
    /// Run `step` `bar_count` times with neutral imports (zeros / pass-through), e.g. CI smoke.
    Synthetic {
        bar_count: usize,
    },
    /// Bind series and TA imports to this OHLCV replay; `bar_index` matches row `0..len`.
    Ohlcv(&'a [(String, String, String, String, String)]),
}

impl GuestReplay<'_> {
    #[must_use]
    pub fn bar_count(&self) -> usize {
        match self {
            GuestReplay::Synthetic { bar_count } => *bar_count,
            GuestReplay::Ohlcv(rows) => rows.len(),
        }
    }

    fn into_bar_feed(self) -> AetherResult<Option<BarFeedState>> {
        match self {
            GuestReplay::Synthetic { .. } => Ok(None),
            GuestReplay::Ohlcv(rows) => BarFeedState::from_ohlcv_strings(rows).map(Some),
        }
    }
}

/// Wasmtime `Store` payload: memory limiter state plus optional OHLCV replay for `aether` imports.
#[derive(Debug)]
pub struct AetherGuestStoreState {
    pub limits: wasmtime::StoreLimits,
    pub bar_feed: Option<BarFeedState>,
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
    let (_store, _instance) = wasmtime_prepare_aether_guest(wasm, limits, None)?;
    Ok(())
}

/// Link `aether::*` stubs, apply [`SandboxLimits`], compile, and instantiate. Used for both
/// preflight-only checks and [`run_guest_strategy_bar_loop_with_limits`].
///
/// This path is **always wasmtime**, even when the `mwvm-full` feature uses `mwvm-sdk` for
/// [`instantiate_job_wasm_inner`], so AgentScript guest modules keep a single execution route
/// for `aether_strategy_*` exports.
fn wasmtime_prepare_aether_guest(
    wasm: &[u8],
    limits: &SandboxLimits,
    bar_feed: Option<BarFeedState>,
) -> AetherResult<(wasmtime::Store<AetherGuestStoreState>, wasmtime::Instance)> {
    use wasmtime::{
        Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder,
    };

    let max_mem = usize::try_from(limits.max_memory_bytes).unwrap_or(usize::MAX);

    let mut config = Config::new();
    config.consume_fuel(true);
    let engine = Engine::new(&config).map_err(|e| AetherError::Sandbox(format!("wasm engine: {e}")))?;

    let module = Module::new(&engine, wasm).map_err(|e| AetherError::Sandbox(format!("wasm compile: {e}")))?;
    let mut linker: Linker<AetherGuestStoreState> = Linker::new(&engine);
    aether_guest_stubs::link_aether_guest_abi_v0(&mut linker).map_err(|e| {
        AetherError::Sandbox(format!("wasm link aether stubs: {e}"))
    })?;

    let store_limits: StoreLimits = StoreLimitsBuilder::new()
        .memory_size(max_mem)
        .instances(2)
        .build();

    let mut store = Store::new(
        &engine,
        AetherGuestStoreState {
            limits: store_limits,
            bar_feed,
        },
    );
    store.limiter(|s| &mut s.limits);

    store
        .set_fuel(limits.fuel_units)
        .map_err(|e| AetherError::Sandbox(format!("wasm fuel: {e}")))?;

    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| AetherError::Sandbox(format!("wasm instantiate: {e}")))?;

    Ok((store, instance))
}

/// Run the strategy guest ABI: call [`EXPORT_INIT`], then [`EXPORT_STEP`] once per bar.
///
/// With [`GuestReplay::Ohlcv`], updates [`BarFeedState::current_bar`] before each `step` so
/// `aether::series_*` / `series_hist*` / `ta_*` imports see the same bar as `VectorBacktestEngine`
/// would advance.
///
/// Caller must supply the same `wasm` bytes validated against [`JobSpec::wasm_sha256`] when the
/// job pins a hash. Uses wasmtime + [`link_aether_guest_abi_v0`] regardless of `mwvm-full`.
pub fn run_guest_strategy_bar_loop_with_limits(
    wasm: &[u8],
    limits: &SandboxLimits,
    replay: GuestReplay<'_>,
) -> AetherResult<()> {
    let bar_count = replay.bar_count();
    if bar_count == 0 {
        return Err(AetherError::Sandbox(
            "guest bar replay requires a positive bar_count".into(),
        ));
    }
    if bar_count > i32::MAX as usize {
        return Err(AetherError::Sandbox(
            "bar_count exceeds i32::MAX (guest step bar_index type)".into(),
        ));
    }

    let bar_feed = replay.into_bar_feed()?;
    let (mut store, instance) = wasmtime_prepare_aether_guest(wasm, limits, bar_feed)?;

    let init = instance
        .get_typed_func::<(), i32>(&mut store, EXPORT_INIT)
        .map_err(|e| AetherError::Sandbox(format!("missing {EXPORT_INIT} export: {e}")))?;
    let step = instance
        .get_typed_func::<(i32,), i32>(&mut store, EXPORT_STEP)
        .map_err(|e| AetherError::Sandbox(format!("missing {EXPORT_STEP} export: {e}")))?;

    let code = init
        .call(&mut store, ())
        .map_err(|e| AetherError::Sandbox(format!("guest {EXPORT_INIT}: {e}")))?;
    if code != 0 {
        return Err(AetherError::Sandbox(format!(
            "guest {EXPORT_INIT} returned error code {code}"
        )));
    }

    for bar in 0..bar_count as i32 {
        if let Some(feed) = store.data_mut().bar_feed.as_mut() {
            feed.current_bar = bar;
        }
        let code = step
            .call(&mut store, (bar,))
            .map_err(|e| AetherError::Sandbox(format!("guest {EXPORT_STEP} at bar {bar}: {e}")))?;
        if code != 0 {
            return Err(AetherError::Sandbox(format!(
                "guest {EXPORT_STEP} returned error code {code} at bar_index {bar}"
            )));
        }
    }

    Ok(())
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
