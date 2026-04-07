//! Trading simulation: domain entities, use cases, and a concrete vectorized engine.

pub mod application;
pub mod domain;
pub mod engine;
pub mod metrics;

#[cfg(test)]
mod engine_tests;

pub use application::run_backtest::RunBacktest;
pub use engine::{placeholder_guest_strategy_result, VectorBacktestEngine};
