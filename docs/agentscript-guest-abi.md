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
| `aether_strategy_step` | Advance one bar / decision point | **v0 preview:** `() -> ()` (see below); **target:** bar index + OHLCV / series context via linear memory or fixed layout (TBD) |

### v0 preview (`agentscript-compiler` emission today)

`agentscript-compiler` emits **both** exports as **`() -> ()`**: `init` is an empty body; `on_bar` / `aether_strategy_step` runs the lowered indicator body (lets + `plot` calls) with stack-balanced codegen. This matches what `wasmparser` validates today and what MWVM preflight can load **once** missing `aether` imports are stubbed.

**Next ABI bump:** change `init` to `() -> i32` (status code), and define `step` parameters (e.g. `i32 bar_index` plus pointer/length pairs for OHLCV buffers, or a single `i32` table offset into shared memory). Increment `guest_abi::VERSION` when that ships.

**Status:** Aether does **not** call these yet; the built-in `VectorBacktestEngine` still runs the demo path. The next integration step is: after sandbox preflight (hash + instantiate + limits), the host invokes `init` / `step` with data fed according to the finalized signature.

**MWVM preflight:** `aether-mwvm` (non–`mwvm-full`) registers wasmtime linker stubs for the full `aether` import table via `link_aether_guest_abi_v0` (`aether-mwvm` crate) so compiled strategy WASM from `agentscript-compiler` can **instantiate** in CI; stubs return neutral values / Rust `f64` `ln`/`exp`/`powf` for `math_*` (not Pine-identical until the real host lands).

### Compiler emission today (`agentscript-compiler`)

The compiler also exports legacy names **`init`** and **`on_bar`** as aliases of the same function indices as `aether_strategy_init` and `aether_strategy_step`, so older tooling keeps working while hosts adopt the reserved names.

It imports a developing host module **`aether`**, including at least: `series_close`, `input_int`, `input_float`, `ta_sma`, `ta_ema`, `ta_crossover`, `ta_crossunder`, `request_security`, `request_financial`, `plot`, `series_hist`, `math_log`, `math_exp`, `math_pow` (see `crates/agentscript-compiler/src/codegen/hir_wasm.rs` and `wasm/abi.rs` for exact signatures). **`request_financial`** is `(i32×7) -> f64` (string slices + ignore flag in guest memory); MWVM stubs may return `0.0` until the financial oracle is wired. Pure transcendentals not in core WASM (`math.sqrt`, `math.round`) are implemented as `f64.sqrt` / `f64.nearest` in the guest where possible; `math.log` / `math.exp` / `math.pow` delegate to host imports for NaN/`na` alignment with Pine. **`ta_crossover` / `ta_crossunder`** are stateful: the host keeps the previous bar’s `(a, b)` and returns `1.0` when the crossing condition holds, then updates stored values. Aether / MWVM stubs should implement these when wiring execution.

## Imports

Strategy modules may need **MWVM host imports** (`morpheum::*`) when built for full `mwvm-sdk` linking. The wasmtime-only preflight in `aether-mwvm` accepts modules **without** those imports; modules that import `morpheum_*` require **`aether-mwvm` with feature `mwvm-full`** for instantiation.

## Security

- `JobSpec::wasm_sha256` commits to bytecode; the runner passes the same bytes into the sandbox.
- `SandboxLimits` (see `aether-mwvm`) caps per-job linear memory growth and fuel during preflight; execution limits will apply the same knobs when guest calls are wired.
