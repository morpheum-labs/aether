//! Pinned `agentscript-compiler` output: link `aether` stubs, instantiate, call **`aether_strategy_init`**
//! and **`aether_strategy_step`** (guest ABI v1). Keeps MWVM aligned with
//! `agentscript-compiler/tests/wasmtime_guest_instantiate.rs`.
//!
//! **Fixture:** [`fixtures/tiny_strategy_guest.wasm`](fixtures/tiny_strategy_guest.wasm) — see
//! [`fixtures/README.md`](fixtures/README.md) for regeneration.

use aether_common::guest_abi::VERSION as GUEST_ABI_VERSION;
use aether_mwvm::link_aether_guest_abi_v0;
use wasmtime::{Engine, Linker, Module, Store};

const TINY_STRATEGY_GUEST_WASM: &[u8] = include_bytes!("fixtures/tiny_strategy_guest.wasm");

#[test]
fn pinned_strategy_guest_wasm_calls_init_and_step() {
    assert_eq!(
        GUEST_ABI_VERSION, 3,
        "docs and compiler assume guest_abi::VERSION == 3 (includes series_string_utf8 import)"
    );

    let engine = Engine::default();
    let module = Module::new(&engine, TINY_STRATEGY_GUEST_WASM).expect("wasmtime parse module");
    let mut linker: Linker<()> = Linker::new(&engine);
    link_aether_guest_abi_v0(&mut linker).expect("link aether stubs");
    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("instantiate with aether imports");

    let init = instance
        .get_typed_func::<(), i32>(&mut store, "aether_strategy_init")
        .expect("aether_strategy_init export");
    let step = instance
        .get_typed_func::<(i32,), i32>(&mut store, "aether_strategy_step")
        .expect("aether_strategy_step export");

    assert_eq!(init.call(&mut store, ()).expect("init"), 0);
    for bar in 0..3 {
        assert_eq!(step.call(&mut store, (bar,)).expect("step"), 0);
    }
}
