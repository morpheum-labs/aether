# Test fixtures

## `tiny_strategy_guest.wasm`

Pinned output of `agentscript-compiler` for [`tiny_indicator.pine`](tiny_indicator.pine) (`indicator` + `plot(close)`). Used by [`strategy_guest_smoke.rs`](../strategy_guest_smoke.rs) to assert MWVM can link `aether` stubs, instantiate, and call guest ABI v1 exports (`init` / `step`).

### Regenerate

From the **agentscript-compiler** repo (no `-o` flag; redirect stdout):

```bash
cargo run -p agentscript-compiler --bin agentscriptc -- \
  --emit=wasm \
  /path/to/aether/crates/aether-mwvm/tests/fixtures/tiny_indicator.pine \
  > /path/to/aether/crates/aether-mwvm/tests/fixtures/tiny_strategy_guest.wasm
```

After regeneration, run `cargo test -p aether-mwvm` in the **aether** workspace.
