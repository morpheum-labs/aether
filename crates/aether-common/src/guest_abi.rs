//! Reserved **strategy guest** ABI for AgentScript → WASM output.
//!
//! `agentscript-compiler` codegen should target these names and conventions once
//! emission lands. Aether uses this module as the single source of truth for
//! constants; see `docs/agentscript-guest-abi.md` for the full contract.

/// Bump when guest layout or export semantics change in a breaking way.
pub const VERSION: u32 = 1;

/// Declared in the custom section or guest metadata (future); name reserved here.
pub const MODULE_NAME: &str = "aether_strategy";

/// One-time setup: seed sizes, tables, etc. `(void) -> i32` — `0` ok, non-zero error.
pub const EXPORT_INIT: &str = "aether_strategy_init";

/// Single simulation step / bar advance. **v0 preview:** compiler emits `() -> ()`; see `docs/agentscript-guest-abi.md` for the planned bar/memory signature.
pub const EXPORT_STEP: &str = "aether_strategy_step";
