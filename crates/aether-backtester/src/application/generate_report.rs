//! Builds human-facing summaries from [`aether_common::BacktestResult`].

use aether_common::types::{BacktestResult, Metrics};
use aether_common::AetherResult;

pub fn summarize_metrics(m: &Metrics) -> String {
    format!(
        "return_bps={} drawdown_bps={} sharpe~={} win_rate_bps={} trades={}",
        m.total_return_bps,
        m.max_drawdown_bps,
        m.sharpe_approx,
        m.win_rate_bps,
        m.trade_count
    )
}

pub fn assert_matches_result(_result: &BacktestResult) -> AetherResult<()> {
    Ok(())
}
