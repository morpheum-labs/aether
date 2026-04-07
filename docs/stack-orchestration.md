# Stack orchestration: Python, Rust, and execution commitments

This document describes how **authoring surfaces** (Python and related services), the **Rust toolchain** (compiler + Aether runtime), and **execution commitments** (pinned WASM, job specs, and any on-chain or protocol-level contracts) are meant to fit together. It is a design sketch for engineers integrating repos and services; it does not replace the WASM guest ABI spec.

## What existed before

- **Compiler ↔ runtime contract:** [`aether/docs/agentscript-guest-abi.md`](../../aether/docs/agentscript-guest-abi.md) (mirrored conceptually from the compiler side in [`spec/guest-abi-v0.md`](../spec/guest-abi-v0.md) for emission details).
- **Compiler roadmap and semantics:** [`ROADMAP.md`](../ROADMAP.md).
- **EVM vs WASM positioning (storage / verification story):** [`docs/evm-wasm/README.md`](evm-wasm/README.md).
- **Product / language vision (including Python as a primary authoring path):** [`spec/agentscripts-v1.md`](../spec/agentscripts-v1.md).

There was no single place that names **orchestration** across all three concerns; this file fills that gap.

## Layered model

| Layer | Main artifacts | Responsibility |
|-------|----------------|----------------|
| **Authoring** | `.qas` / `.pine`, prompts, notebooks, agent outputs | Humans or agents produce or edit strategy source; optional transpilation from Python-like pseudocode into AgentScript is a *product* choice, not enforced by the compiler today. |
| **Build** | `wasm32` bytes, compiler version, optional lockfile digest | `agentscript-compiler` turns validated source into a guest module that imports `aether` per the guest ABI. |
| **Run** | wasmtime / MWVM, linked host imports, bar replay | **Aether** (`aether-mwvm`, runners) instantiates the module, binds imports to feeds and simulation state, and drives `init` / `step`. |
| **Commit** | `JobSpec`, `wasm_sha256`, `data_commitment`, seeds | Jobs pin **which bytecode** and **which data** define a reproducible run; verifiers compare supplied WASM to the hash before execution. |

Think of **Python** as living mostly in the **authoring** and **control-plane** tiers (services, CLIs, agents, CI scripts) that *invoke* the Rust compiler and *submit* jobs to Aether—or that generate QAS source for a later compile step. The **canonical execution artifact** for the strategy guest path is **WASM + ABI**, not a Python interpreter inside the sandbox.

## End-to-end flow (intended)

1. **Author:** Editor, agent, or pipeline produces AgentScript / QAS source (possibly after LLM or Python-assisted codegen).
2. **Compile:** A Rust binary or embedded `agentscript-compiler` library emits WASM; record **compiler version** and, if you use reproducible Rust deps for downstream crates, optional **`cargo_lock_hash`**-style metadata where your protocol defines it.
3. **Pin:** Compute **`wasm_sha256`** over the exact module bytes that runners will load. Any registry, chain, or job bulletin should store this hash (and ABI version expectations) alongside human-readable metadata.
4. **Schedule:** Build a **`JobSpec`** (see `aether-common::types::JobSpec`): execution tier, **data commitment**, optional **`wasm_sha256`**, simulation **seed**, capital / fee / slippage fields, etc.
5. **Execute:** A worker loads WASM bytes, checks they match `wasm_sha256` when set, links **`aether`** imports, then runs the guest **`init` / `step`** sequence described in the guest ABI doc.
6. **Attest / settle:** Higher layers (TEE runner, chain, vault) attach proofs or receipts to the same job id and commitments—out of scope for the compiler crate, but they must agree on **what was hashed** and **which ABI version** applies.

## Responsibility split

- **Python (or other non-Rust hosts):** UX, orchestration, storage pointers, calling `agentscriptc` or the compiler library, packaging job JSON, talking to nodes/APIs. Should not reinterpret guest semantics; treat WASM as opaque except for hashing and ABI version checks.
- **Rust compiler:** Parse, analyze, lower, emit WASM that matches **`GUEST_ABI_V0_IMPORTS`** and export names in lockstep with Aether (`agentscript-compiler` `codegen/wasm/abi.rs` ↔ `aether-mwvm` linker stubs ↔ `guest_abi::VERSION`).
- **Rust runtime (Aether):** Sandboxing, fuel/limits, feeding series and `request.*` implementations, enforcing **`JobSpec::wasm_sha256`** vs provided bytes, and eventually full backtest engines wired to the guest.
- **Contracts / commitments:** The word “contracts” here means both **interface contracts** (guest ABI) and **agreed-on bytes** (hashes, data commitments, job ids). Smart-contract specifics belong in whichever repo owns chain deployment; they should reference the same **`wasm_sha256`** and **data** commitments as `JobSpec`.

## Version skew and coordination

- Bump **`guest_abi::VERSION`** when export/import signatures or calling conventions change; compiler, MWVM stubs, and docs must move together.
- Jobs should pin **expected ABI version** once that field exists on `JobSpec`; until then, operational playbooks should record ABI version next to `wasm_sha256`.
- Python services should treat ABI version and compiler version as **opaque strings** in telemetry and job payloads so operators can reproduce builds.

## Open design choices (to resolve in implementation)

- Whether **Python strategies** ever run **natively** in production or only as a **development** path with mandatory compile-to-WASM for shared verification.
- Where **toolchain attestations** live (on-chain vs off-chain registry vs TEE policy).
- Single **job orchestrator** service vs many agents submitting jobs directly—does not change the WASM/commitment boundary.

## Related paths

| Topic | Location |
|-------|----------|
| Guest import/export table | [`aether/docs/agentscript-guest-abi.md`](../../aether/docs/agentscript-guest-abi.md) |
| Compiler emission notes | [`spec/guest-abi-v0.md`](../spec/guest-abi-v0.md) |
| Compiler ↔ Aether progress | [`ROADMAP.md`](../ROADMAP.md) (semantics table, downstream alignment) |
