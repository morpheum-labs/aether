//! Pinned `agentscript-compiler` output: link `aether` stubs, instantiate, call **`aether_strategy_init`**
//! and **`aether_strategy_step`** (guest ABI v1). Keeps MWVM aligned with
//! `agentscript-compiler/tests/wasmtime_guest_instantiate.rs`.
//!
//! **Fixture:** [`fixtures/tiny_strategy_guest.wasm`](fixtures/tiny_strategy_guest.wasm) — see
//! [`fixtures/README.md`](fixtures/README.md) for regeneration.

use aether_common::guest_abi::VERSION as GUEST_ABI_VERSION;
use aether_mwvm::{run_guest_strategy_bar_loop_with_limits, GuestReplay, SandboxLimits};

const TINY_STRATEGY_GUEST_WASM: &[u8] = include_bytes!("fixtures/tiny_strategy_guest.wasm");

#[test]
fn pinned_strategy_guest_wasm_calls_init_and_step() {
    assert_eq!(
        GUEST_ABI_VERSION, 3,
        "docs and compiler assume guest_abi::VERSION == 3 (includes series_string_utf8 import)"
    );

    run_guest_strategy_bar_loop_with_limits(
        TINY_STRATEGY_GUEST_WASM,
        &SandboxLimits {
            max_memory_bytes: 64 * 1024 * 1024,
            fuel_units: 50_000_000,
        },
        GuestReplay::Synthetic { bar_count: 3 },
    )
    .expect("init + 3 steps with neutral imports");
}
