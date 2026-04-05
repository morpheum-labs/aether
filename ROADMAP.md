# Aether roadmap

## Primary goal

**Aether runs backtesting workloads for trading strategies expressed in AgentScript (QAS) and compiled to WebAssembly.** The language and toolchain live in **`agentscript-compiler`**; Aether consumes **WASM + `JobSpec` + data commitments + tier policy**, validates in a sandbox, and executes the backtest path (wasmtime by default, full **MWVM** with `mwvm-full`, **TEE** later for confidential jobs).

MWVM is the **portable execution kernel**; AgentScript is the **strategy surface** compiled into that world.

## Upstream status (agentscript-compiler)

**Today:** parse → **AST** only (Chumsky, `//@version` QAS v1 or v6, miette diagnostics). The binary prints the parsed `Script`; **typecheck, codegen, and `wasm32` output are not implemented yet.**

**Implication for Aether:** Phase 1 ABI work can start as a **written contract** (doc + types in `aether-common`) in parallel with compiler milestones; end-to-end “`.qas` → WASM → `aether`” waits on compiler codegen.

## Secondary alignment

Design notes and economics in **`vaulted-knowledge-protocol/backtesting-infra`** (tiers, attestation, marketplace). Product spine remains **AgentScript → WASM → Aether backtest**.

## Phase 0 — Foundation (current)

- [x] Workspace crates: common, `aether-mwvm`, backtester, tee-runner, attest, oracle, node, CLI, contract stubs.
- [x] `ExecutionTier`, `DataCommitment`, deterministic vector engine (`VectorBacktestEngine`), Merkle-checked oracle.
- [x] **`aether-mwvm`** — `JobSpec::wasm_sha256` + optional WASM bytes on `TeeRunner::run_with_provider`; wasmtime instantiate (default); optional **`mwvm-full`** for `mwvm-sdk` when `morpheum-*` builds. `mwvm-sdk` `AgentBuilder` respects full `SdkConfig` (e.g. `model_serving = false`).
- [x] **`aether-cli`** — `backtest --wasm <path>` runs demo dataset + WASM preflight (hash + instantiate). Without `--wasm`, behavior is unchanged (in-process engine only).

## Phase 1 — AgentScript WASM sandbox + resource policy

- [x] **Shared ABI spec** — `docs/agentscript-guest-abi.md` + `aether_common::guest_abi` constants (`VERSION`, reserved export names). **Guest exports are not invoked yet**; `VectorBacktestEngine` still drives results.
- [x] **CLI / node** — CLI: `--wasm` + `--wasm-max-memory-mb` / `--wasm-fuel`. Node: `aether-node <config.json> <strategy.wasm>` (optional second path); `JobHandler::claim_and_execute(spec, wasm)` passes bytes into the runner.
- [x] **Resource limits (wasmtime path)** — `SandboxLimits` + per-memory `StoreLimits` + fuel during preflight in `aether-mwvm`. **`mwvm-full`:** limits not yet threaded into `mwvm-sdk` (same instantiate as before).
- [ ] Optional: enforce `cargo_lock_hash` / toolchain when jobs claim reproducibility.

## Phase 2 — Network + contracts

- [ ] Real `JobSource`: L1 / indexer; claim and result persistence.
- [ ] Job-queue, attestation-verifier, slashing-registry execute/query beyond message stubs.
- [ ] Deposits, challenge windows, optional second verifier (per backtesting-infra code-5).

## Phase 3 — Confidential path

- [ ] Hardware attestation (TDX / SEV) instead of `SoftwareAttester`.
- [ ] Encrypted payloads and enclave-local data paths.
- [ ] Verifier policy (on-chain or service) tied to quote measurements.

## Phase 4 — Product depth

- [ ] Sweeps / `OptimizeParameters`, richer reports for **the same compiled WASM** across many `JobSpec`s.
- [ ] Documented dev loop: **`agentscript-compiler` emits WASM** → **`aether` run** (no second parser in Aether).
- [ ] GPU / ZK / LLM-in-the-loop as priced tiers (orthogonal to batch AgentScript backtests).

## Repository layout

| Area | Repo / crate | Role |
|------|----------------|------|
| AgentScript → WASM | **`agentscript-compiler`** | QAS parser + AST **today**; typecheck + codegen + WASM **next** |
| WASM agents / tooling | **`mwvm`** (`mwvm-core`, `mwvm-sdk`, `cargo-mwvm`) | Runtime, hosts, `cargo mwvm` templates for WASM agents |
| Backtest compute | **`aether`** | Jobs, sandbox, oracle, tiered runner, node + CLI |
| Specs / papers | **`vaulted-knowledge-protocol/backtesting-infra`** | Architecture and economics |

## Success criteria by phase

| Phase | Done when |
|-------|-----------|
| **0** | `cargo test` green in `aether`; `wasm_sha256` jobs reject bad WASM before simulation. |
| **1** | Any **trusted** `.wasm` runs through CLI/node with bounded memory/fuel; ABI doc exists for **`agentscript-compiler` to target**. |
| **2** | Devnet path: submit → operator runs → result (+ optional attestation) recorded. |
| **3** | Confidential tier binds result digest to a verifiable hardware quote. |
| **4** | Documented **AgentScript → compile → Aether backtest** for builders; sweeps usable on compiled strategies. |
