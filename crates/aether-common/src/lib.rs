//! Shared foundations for the Aether confidential compute / backtesting stack.
//!
//! Crates in this workspace depend on `aether-common` for DRY types and trait
//! boundaries (Clean Architecture + dependency inversion).

pub mod errors;
pub mod traits;
pub mod types;
pub mod utils;

pub use errors::{AetherError, AetherResult};
pub use traits::JobOutcome;
pub use types::*;
