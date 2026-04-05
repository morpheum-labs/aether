# AgentScript strategy guest ABI (Aether)

This document is the contract between **`agentscript-compiler`** (QAS → WASM) and **Aether** (load, sandbox, backtest). Rust constants live in `aether-common::guest_abi`.

## Versioning

- `guest_abi::VERSION` (`u32`): increment on breaking changes to exports or calling convention.
- Jobs may pin expected ABI version in `JobSpec` later; today only constants are defined.

## Module

- Logical name: `aether_strategy` (constant `guest_abi::MODULE_NAME` in `crates/aether-common/src/guest_abi.rs`).
- Target: `wasm32-unknown-unknown` unless MWVM documents otherwise.

## Exports (reserved)

| Symbol | Role | Signature (planned) |
|--------|------|---------------------|
| `aether_strategy_init` | One-time setup | `() -> i32` — zero = success |
| `aether_strategy_step` | Advance one bar / decision point | TBD (linear memory + ptr/len buffers or fixed layout) |

**Status:** Aether does **not** call these yet; the built-in `VectorBacktestEngine` still runs the demo path. The next integration step is: after sandbox preflight (hash + instantiate + limits), the host invokes `init` / `step` (or a single `run_backtest` export) with OHLCV fed according to the finalized signature.

## Imports

Strategy modules may need **MWVM host imports** (`morpheum::*`) when built for full `mwvm-sdk` linking. The wasmtime-only preflight in `aether-mwvm` accepts modules **without** those imports; modules that import `morpheum_*` require **`aether-mwvm` with feature `mwvm-full`** for instantiation.

## Security

- `JobSpec::wasm_sha256` commits to bytecode; the runner passes the same bytes into the sandbox.
- `SandboxLimits` (see `aether-mwvm`) caps per-job linear memory growth and fuel during preflight; execution limits will apply the same knobs when guest calls are wired.
