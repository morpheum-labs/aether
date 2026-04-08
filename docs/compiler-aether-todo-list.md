# Compiler ↔ Aether — consolidated TODO list

Synthesized from [`ROADMAP.md`](../ROADMAP.md) (this repo), **`agentscript-compiler`** [`ROADMAP.md`](https://github.com/morpheum-labs/agentscript-compiler/blob/main/ROADMAP.md), and integration-gap notes. Use this as a working backlog; keep [`docs/agentscript-guest-abi.md`](agentscript-guest-abi.md) the contract for names and signatures.

---

## P0 — Close the integration loop (Phase 1 “next”)

### Aether / MWVM

- [x] **Call guest exports** from the backtest runner when `JobSpec` carries strategy WASM: at least **`init` → `on_bar` / `step`** sequence aligned with [`agentscript-guest-abi.md`](agentscript-guest-abi.md) (same bar-loop assumptions as `VectorBacktestEngine`, or document intentional divergence). **Done (Phase 1 shell):** `aether_mwvm::run_guest_strategy_bar_loop_with_limits` + `TeeRunner::run_with_provider` when WASM bytes are present; `BacktestResult` is still a **placeholder** flat equity curve until guest PnL is read from the module.
- [ ] **Host import wiring** — define and implement WASM import namespaces for **`request.security`**, **`request.financial`**, and **`strategy.*`**, routed to oracle/feed **stubs** first, then real data paths. **Progress:** OHLCV-backed **`aether::series_*`**, **`series_hist`**, **`series_hist_at`**, and host-computed **`ta_sma` / `ta_ema` / `ta_tr` / `ta_atr` / `ta_crossover` / `ta_crossunder`** for the current replay (`aether-mwvm` `BarFeedState`, `GuestReplay::Ohlcv`); `request_*` / `strategy.*` / real oracles still open.
- [ ] **Contract tests** — instantiate compiler-emitted (or golden) guest modules, assert export names/signatures match ABI, optional **`wasm_sha256`** pin in job specs.
- [ ] **`mwvm-full` resource limits** — thread `SandboxLimits` / memory / fuel into `mwvm-sdk` path (Phase 1 notes: limits not yet applied there).
- [ ] Optional: enforce **`cargo_lock_hash` / toolchain** on jobs that claim reproducible builds.

### agentscript-compiler

- [ ] **Stable emitted WASM** — grow [`emit_hir_guest_wasm`](https://github.com/morpheum-labs/agentscript-compiler) (HIR subset) toward **production** modules: MWVM linker compatibility, fewer rejections (nested `plot`, more series, UDT methods, guest array ABI when ready).
- [ ] **`request.security` / `request.financial` end-to-end** — dynamic symbol/timeframe and args beyond literals; real host semantics + golden tests (compiler + Aether).
- [ ] **Guest ABI emission** — keep exports (`init`, `on_bar`, `aether_strategy_*`) and **`GUEST_ABI_V0_IMPORTS`** in lockstep with Aether/MWVM stubs and ABI doc; bump `guest_abi::VERSION` only when contract is agreed.
- [ ] **Full span coverage** — replace `Span::DUMMY` on semantic/codegen errors where AST/HIR has a real range (compiler ROADMAP near-term #1).

### Shared / cross-repo

- [ ] **ABI doc + stub + test triad** — any import/export change updates: `agentscript-compiler` codegen/ABI guard, **Aether** `docs/agentscript-guest-abi.md`, **aether-mwvm** linker stubs, **`wasmtime_guest_instantiate`** (or equivalent) in compiler tests.
- [ ] **Determinism story** — FP rules, fixed codegen options, toolchain metadata for **`wasm_sha256`** pins (compiler: “None” today; Aether jobs need a pinned story).

---

## P1 — Language & HIR (compiler-heavy)

- [ ] **Full type system** — enforce surface `array<>` / `matrix<>` / `map<>`; generics; deeper Pine/QAS parity.
- [ ] **Imports / exports through HIR** — typecheck can link `register_import_library` today; **HIR still rejects** lowered calls through linked libraries; no full module graph or WASM library inlining.
- [ ] **Widen HIR + WASM** — more OHLC series on guest path, more **`ta.*`** from registry, **`syminfo.*` / `timeframe.*`** beyond minimal typing, nested `plot` / let-values where subset still errors.
- [ ] **`strategy.*`** — specify host imports and effect ordering before deep codegen (semantics currently **None** for full order/PnL surface).
- [ ] **Other `request.*`** (e.g. economic) — same pattern as security/financial when prioritized.
- [ ] **`mcp.*`** — QAS builtins; host MCP proxy (currently **None**).
- [ ] **Bar execution model** — `barstate.*`, tick replay, full TV execution rules (host + compiler semantics).
- [ ] **Control flow** — reachability, Pine loop limits, full `switch`/`while` semantics where still open.
- [ ] **Side effects & order** — effect typing + schedule in IR (**None** today).
- [ ] **Constant folding** — optional post-typecheck (**None** today).
- [ ] **Codegen path** — optional move toward agreed **`wasm32-unknown-unknown`** (or triple) + **`aether-common`** alignment beyond prose ABI (compiler ROADMAP).
- [ ] **Tooling** — output file flag (`-o`), richer JSON diagnostics (compiler Phase 3 tooling; Aether Phase 4 references compiler CLI).

---

## P2 — Aether product / network ([`ROADMAP.md`](../ROADMAP.md) Phase 2)

- [ ] Real **`JobSource`**: L1 / indexer; claim and result persistence.
- [ ] Job-queue, attestation-verifier, slashing-registry execute/query beyond message stubs.
- [ ] Deposits, challenge windows, optional second verifier (per backtesting-infra).

---

## P3 — Confidential path (Phase 3)

- [ ] Hardware attestation (TDX / SEV) instead of `SoftwareAttester`.
- [ ] Encrypted payloads and enclave-local data paths.
- [ ] Verifier policy tied to quote measurements.

---

## P4 — Product depth (Phase 4)

- [ ] Sweeps / `OptimizeParameters`, richer reports for **the same compiled WASM** across many `JobSpec`s.
- [ ] Documented dev loop: **AgentScript → `agentscript-compiler` → WASM → `aether` / `aether-cli` backtest** (Aether does not parse `.qas` / `.pine`; WASM + ABI only).
- [ ] GPU / ZK / LLM-in-the-loop as priced tiers (orthogonal to batch AgentScript backtests).

---

## Hygiene / docs

- [ ] Restore or sync **`agentscript-compiler`** `docs/aether-integration-gap.md` and `docs/github-backlog.md` if your branch still references them from [`ROADMAP.md`](https://github.com/morpheum-labs/agentscript-compiler/blob/main/ROADMAP.md) but they are missing locally.
- [ ] Keep **upstream status** paragraph in Aether [`ROADMAP.md`](../ROADMAP.md) updated as compiler WASM moves from experimental to contract-tested stable.

---

## Already done (context — do not duplicate)

- Aether Phase 0 foundation; Phase 1 ABI doc + `aether_common::guest_abi`; CLI/node WASM paths + `SandboxLimits` on wasmtime; `verify-wasm`.
- Compiler: parse → analyze → HIR slice → experimental guest WASM + wasmtime **`init`+`step`** smoke; partial `request.*`, `ta.*` basics, `plot` (top-level), `var`/`varip` as wasm globals for supported scripts.
