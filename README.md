# Aether

Backtest compute layer for **AgentScript → WASM** strategies: job types, sandboxed WASM preflight (wasmtime / optional MWVM), deterministic demo engine, oracle commitments, and operator-shaped binaries.

See **[ROADMAP.md](./ROADMAP.md)** for phases and integration with [`agentscript-compiler`](https://github.com/morpheumlabs/agentscript-compiler) and [MWVM](../mwvm).

## Requirements

- Rust (see `rust-toolchain.toml`)

## Build & test

```bash
cargo test
```

## CLI (Phase 0)

```bash
# Demo backtest (no WASM)
cargo run -p aether-cli -- backtest

# Demo backtest + WASM preflight (WAT/WASM path)
cargo run -p aether-cli -- backtest --wasm crates/aether-mwvm/tests/fixtures/minimal_agent.wat

# WASM-only smoke: hash + instantiate under limits
cargo run -p aether-cli -- verify-wasm crates/aether-mwvm/tests/fixtures/minimal_agent.wat
```

## Node (demo)

```bash
cargo run -p aether-node
# With config + optional strategy WASM:
# cargo run -p aether-node -- /path/to/node.json /path/to/strategy.wasm
```

## Docs

- [AgentScript guest ABI](./docs/agentscript-guest-abi.md) — contract for compiler codegen.
