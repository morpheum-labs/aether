use aether_common::traits::{BacktestEngine, DeterministicEngine};
use aether_common::types::{
    BacktestResult, JobSpec, Metrics, Position, Side, SimulationState, Trade,
};
use aether_common::utils::crypto::sha256_32;
use aether_common::{AetherError, AetherResult};
use rust_decimal::Decimal;
use serde::Serialize;

use crate::domain::StrategyParams;
use crate::metrics::{compute as compute_metrics, parse_decimal};

#[derive(Clone, Copy, Debug, Default)]
pub struct VectorBacktestEngine;

#[derive(Serialize)]
struct ResultDigestPayload<'a> {
    job_id: &'a [u8; 32],
    metrics: &'a Metrics,
    equity_curve: &'a [String],
    trades: &'a [Trade],
}

fn bps_to_factor(bps: u16) -> AetherResult<Decimal> {
    Ok(Decimal::from(bps) / Decimal::from(10_000u32))
}

fn digest_result(r: &BacktestResult) -> AetherResult<[u8; 32]> {
    let payload = ResultDigestPayload {
        job_id: &r.job_id,
        metrics: &r.metrics,
        equity_curve: &r.equity_curve,
        trades: &r.trades,
    };
    let bytes =
        serde_json::to_vec(&payload).map_err(|e| AetherError::BacktestEngine(e.to_string()))?;
    Ok(sha256_32(&bytes))
}

impl VectorBacktestEngine {
    fn run_inner(
        &self,
        spec: &JobSpec,
        ohlcv: &[(String, String, String, String, String)],
    ) -> AetherResult<(BacktestResult, Vec<SimulationState>)> {
        if ohlcv.len() < 3 {
            return Err(AetherError::BacktestEngine(
                "need at least 3 bars".into(),
            ));
        }

        let initial = parse_decimal(&spec.initial_capital)?;
        if initial <= Decimal::ZERO {
            return Err(AetherError::BacktestEngine(
                "initial_capital must be positive".into(),
            ));
        }

        let fee_rate = bps_to_factor(spec.fee_bps)?;
        let slip = bps_to_factor(spec.slippage_bps)?;
        let params = StrategyParams::from_seed(spec.seed);
        let fp = params.fast_period as usize;
        let sp = params.slow_period as usize;
        if sp >= ohlcv.len() {
            return Err(AetherError::BacktestEngine(
                "slow period exceeds series length".into(),
            ));
        }

        let closes: Vec<Decimal> = ohlcv
            .iter()
            .map(|(_, _, _, c, _)| parse_decimal(c))
            .collect::<AetherResult<_>>()?;

        let mut cash = initial;
        let mut position_qty = Decimal::ZERO;
        let mut avg_entry = Decimal::ZERO;
        let mut trades: Vec<Trade> = Vec::new();
        let mut states: Vec<SimulationState> = Vec::new();
        let mut equity_series: Vec<Decimal> = Vec::new();

        let mut fast_sum = Decimal::ZERO;
        let mut slow_sum = Decimal::ZERO;
        let mut prev_fast_ma: Option<Decimal> = None;
        let mut prev_slow_ma: Option<Decimal> = None;

        for i in 0..ohlcv.len() {
            let close = closes[i];

            fast_sum += close;
            if i >= fp {
                fast_sum -= closes[i - fp];
            }
            let fast_ma = if i + 1 >= fp {
                fast_sum / Decimal::from(fp as u64)
            } else {
                Decimal::ZERO
            };

            slow_sum += close;
            if i >= sp {
                slow_sum -= closes[i - sp];
            }
            let slow_ma = if i + 1 >= sp {
                slow_sum / Decimal::from(sp as u64)
            } else {
                Decimal::ZERO
            };

            if i + 1 >= sp {
                if let (Some(pf), Some(ps)) = (prev_fast_ma, prev_slow_ma) {
                    let bullish = fast_ma > slow_ma;
                    let was_bullish = pf > ps;

                    if bullish && !was_bullish && position_qty.is_zero() && cash > Decimal::ZERO {
                        let exec = close * (Decimal::ONE + slip);
                        if exec > Decimal::ZERO {
                            let fee = cash * fee_rate;
                            let spend = cash - fee;
                            let qty = spend / exec;
                            if qty > Decimal::ZERO {
                                trades.push(Trade {
                                    bar_index: i as u64,
                                    side: Side::Buy,
                                    qty: qty.normalize().to_string(),
                                    price: exec.normalize().to_string(),
                                    fee_paid: fee.normalize().to_string(),
                                });
                                position_qty = qty;
                                avg_entry = exec;
                                cash = Decimal::ZERO;
                            }
                        }
                    } else if !bullish && was_bullish && !position_qty.is_zero() {
                        let exec = close * (Decimal::ONE - slip);
                        let notional = position_qty * exec;
                        let fee = notional * fee_rate;
                        let proceeds = notional - fee;
                        trades.push(Trade {
                            bar_index: i as u64,
                            side: Side::Sell,
                            qty: position_qty.normalize().to_string(),
                            price: exec.normalize().to_string(),
                            fee_paid: fee.normalize().to_string(),
                        });
                        cash += proceeds;
                        position_qty = Decimal::ZERO;
                        avg_entry = Decimal::ZERO;
                    }
                }
            }

            prev_fast_ma = Some(fast_ma);
            prev_slow_ma = Some(slow_ma);

            let mark = if position_qty.is_zero() {
                cash
            } else {
                cash + position_qty * close
            };
            equity_series.push(mark);

            let pos = if position_qty.is_zero() {
                None
            } else {
                Some(Position {
                    qty: position_qty.normalize().to_string(),
                    avg_entry: avg_entry.normalize().to_string(),
                })
            };
            states.push(SimulationState {
                bar_index: i as u64,
                cash: cash.normalize().to_string(),
                position: pos,
                equity: mark.normalize().to_string(),
            });
        }

        let mut buy_stack: Vec<Decimal> = Vec::new();
        let mut winning_sells = 0u64;
        let mut sell_count = 0u64;
        for t in &trades {
            match t.side {
                Side::Buy => {
                    buy_stack.push(parse_decimal(&t.price)?);
                }
                Side::Sell => {
                    sell_count += 1;
                    let sell_px = parse_decimal(&t.price)?;
                    if let Some(buy_px) = buy_stack.pop() {
                        if sell_px > buy_px {
                            winning_sells += 1;
                        }
                    }
                }
            }
        }

        let metrics = compute_metrics(initial, &equity_series, winning_sells, sell_count)?;
        let equity_curve: Vec<String> = equity_series
            .iter()
            .map(|d| d.normalize().to_string())
            .collect();

        let mut result = BacktestResult {
            job_id: spec.id,
            metrics,
            equity_curve,
            trades,
            result_digest: [0u8; 32],
        };
        result.result_digest = digest_result(&result)?;
        Ok((result, states))
    }
}

impl BacktestEngine for VectorBacktestEngine {
    fn run(
        &self,
        spec: &JobSpec,
        ohlcv: &[(String, String, String, String, String)],
    ) -> AetherResult<(BacktestResult, Vec<SimulationState>, Vec<Trade>)> {
        let (result, states) = self.run_inner(spec, ohlcv)?;
        let trades = result.trades.clone();
        Ok((result, states, trades))
    }
}

impl DeterministicEngine for VectorBacktestEngine {
    fn run_deterministic(
        &self,
        spec: &JobSpec,
        ohlcv: &[(String, String, String, String, String)],
    ) -> AetherResult<BacktestResult> {
        let (r, _) = self.run_inner(spec, ohlcv)?;
        Ok(r)
    }
}
