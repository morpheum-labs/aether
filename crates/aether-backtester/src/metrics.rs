use aether_common::types::Metrics;
use aether_common::{AetherError, AetherResult};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal::MathematicalOps;
use std::str::FromStr;

pub fn compute(
    initial_equity: Decimal,
    equity_curve: &[Decimal],
    winning_trades: u64,
    total_trades: u64,
) -> AetherResult<Metrics> {
    if equity_curve.is_empty() {
        return Err(AetherError::BacktestEngine(
            "empty equity curve".into(),
        ));
    }
    let last = *equity_curve.last().unwrap();
    let ret = if initial_equity.is_zero() {
        Decimal::ZERO
    } else {
        (last - initial_equity) / initial_equity
    };
    let total_return_bps = (ret * Decimal::from(10_000i64))
        .round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven)
        .to_i64()
        .unwrap_or(0);

    let mut peak = initial_equity;
    let mut max_dd = Decimal::ZERO;
    for &e in equity_curve {
        if e > peak {
            peak = e;
        }
        if peak > Decimal::ZERO {
            let dd = (peak - e) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    let max_drawdown_bps = (max_dd * Decimal::from(10_000u64))
        .round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven)
        .to_u64()
        .unwrap_or(0);

    let rets: Vec<Decimal> = equity_curve
        .windows(2)
        .map(|w| {
            if w[0].is_zero() {
                Decimal::ZERO
            } else {
                (w[1] - w[0]) / w[0]
            }
        })
        .collect();
    let sharpe_approx = if rets.len() < 2 {
        Decimal::ZERO
    } else {
        let mean: Decimal = rets.iter().copied().sum::<Decimal>() / Decimal::from(rets.len());
        let var: Decimal = rets
            .iter()
            .map(|r| {
                let d = *r - mean;
                d * d
            })
            .sum::<Decimal>()
            / Decimal::from(rets.len());
        let std = var.sqrt().unwrap_or(Decimal::ZERO);
        if std.is_zero() {
            Decimal::ZERO
        } else {
            mean / std
        }
    };

    let win_rate_bps = if total_trades == 0 {
        0
    } else {
        ((Decimal::from(winning_trades) / Decimal::from(total_trades)) * Decimal::from(10_000u64))
            .round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven)
            .to_u64()
            .unwrap_or(0)
    };

    Ok(Metrics {
        total_return_bps,
        max_drawdown_bps,
        sharpe_approx: sharpe_approx.normalize().to_string(),
        win_rate_bps,
        trade_count: total_trades,
    })
}

pub fn parse_decimal(s: &str) -> AetherResult<Decimal> {
    Decimal::from_str(s.trim()).map_err(|e| AetherError::BacktestEngine(e.to_string()))
}
