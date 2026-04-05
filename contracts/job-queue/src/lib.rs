//! On-chain job queue types — keep execute/query logic in contract templates later.

pub mod error;
pub mod msg;
pub mod state;

pub use error::ContractError;
pub use msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
pub use state::{JobRecord, JobStatus};
