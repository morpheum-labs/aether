//! Host-side OHLCV replay for `aether::series_*` and related imports during guest `step` calls.
//!
//! Matches the contract in `docs/agentscript-guest-abi.md`: **`bar_index`** is passed to the
//! guest, while series values are read through imports bound to **host current bar** state.

use aether_common::{AetherError, AetherResult};

/// One bar in host replay memory (already parsed to `f64`).
#[derive(Clone, Copy, Debug)]
pub struct BarRow {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// `series_hist_at` / compiler `series_hist_kind` indices.
const SK_CLOSE: i32 = 0;
const SK_OPEN: i32 = 1;
const SK_HIGH: i32 = 2;
const SK_LOW: i32 = 3;
const SK_VOLUME: i32 = 4;
const SK_TIME: i32 = 5;

/// `ta_sma` / `ta_ema` first argument: close vs true range stream (`agentscript-compiler` ABI).
const MA_SRC_CLOSE: i32 = 0;
const MA_SRC_TRUE_RANGE: i32 = 1;

/// Stateful guest replay: current bar index + prior values for crossover builtins.
#[derive(Debug)]
pub struct BarFeedState {
    rows: Vec<BarRow>,
    /// Host cursor; must match the `bar_index` argument passed into `aether_strategy_step`.
    pub current_bar: i32,
    cross_prev_a: Option<f64>,
    cross_prev_b: Option<f64>,
    under_prev_a: Option<f64>,
    under_prev_b: Option<f64>,
}

impl BarFeedState {
    /// Parse oracle-style string OHLCV rows into [`BarRow`] (same order as `MarketDataProvider`).
    pub fn from_ohlcv_strings(
        rows: &[(String, String, String, String, String)],
    ) -> AetherResult<Self> {
        let mut out = Vec::with_capacity(rows.len());
        for (o, h, l, c, v) in rows {
            out.push(BarRow {
                open: parse_f64(o)?,
                high: parse_f64(h)?,
                low: parse_f64(l)?,
                close: parse_f64(c)?,
                volume: parse_f64(v)?,
            });
        }
        Ok(Self {
            rows: out,
            current_bar: 0,
            cross_prev_a: None,
            cross_prev_b: None,
            under_prev_a: None,
            under_prev_b: None,
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    fn row_at(&self, bar: i32) -> Option<BarRow> {
        if bar < 0 {
            return None;
        }
        let i = bar as usize;
        self.rows.get(i).copied()
    }

    fn value_at_bar(&self, bar: i32, kind: i32) -> f64 {
        let Some(r) = self.row_at(bar) else {
            return f64::NAN;
        };
        match kind {
            SK_OPEN => r.open,
            SK_HIGH => r.high,
            SK_LOW => r.low,
            SK_CLOSE => r.close,
            SK_VOLUME => r.volume,
            SK_TIME => bar as f64,
            _ => f64::NAN,
        }
    }

    /// `close[offset]` / `series_hist_at(kind, offset)` — `offset` bars ago from [`Self::current_bar`].
    pub fn hist_at(&self, kind: i32, offset: i32) -> f64 {
        let bar = self.current_bar as i64 - i64::from(offset);
        if bar < 0 || bar > i32::MAX as i64 {
            return f64::NAN;
        }
        self.value_at_bar(bar as i32, kind)
    }

    pub fn series_close(&self) -> f64 {
        self.hist_at(SK_CLOSE, 0)
    }

    pub fn series_open(&self) -> f64 {
        self.hist_at(SK_OPEN, 0)
    }

    pub fn series_high(&self) -> f64 {
        self.hist_at(SK_HIGH, 0)
    }

    pub fn series_low(&self) -> f64 {
        self.hist_at(SK_LOW, 0)
    }

    pub fn series_volume(&self) -> f64 {
        self.hist_at(SK_VOLUME, 0)
    }

    pub fn series_time(&self) -> f64 {
        self.hist_at(SK_TIME, 0)
    }

    fn true_range_at(&self, bar: i32) -> f64 {
        let Some(r) = self.row_at(bar) else {
            return f64::NAN;
        };
        let hl = r.high - r.low;
        if bar == 0 {
            return hl;
        }
        let Some(prev) = self.row_at(bar - 1) else {
            return f64::NAN;
        };
        let pc = prev.close;
        let t1 = (r.high - pc).abs();
        let t2 = (r.low - pc).abs();
        hl.max(t1).max(t2)
    }

    pub fn ta_tr(&self) -> f64 {
        self.true_range_at(self.current_bar)
    }

    pub fn ta_sma(&self, src_kind: i32, period: i32) -> f64 {
        let p = period as usize;
        if p == 0 || self.current_bar < 0 {
            return f64::NAN;
        }
        let end = self.current_bar as usize;
        if end + 1 < p {
            return f64::NAN;
        }
        let start = end + 1 - p;
        let mut sum = 0.0f64;
        for i in start..=end {
            let v = match src_kind {
                MA_SRC_CLOSE => self.rows.get(i).map(|r| r.close),
                MA_SRC_TRUE_RANGE => Some(self.true_range_at(i as i32)),
                _ => None,
            };
            let Some(x) = v else {
                return f64::NAN;
            };
            if x.is_nan() {
                return f64::NAN;
            }
            sum += x;
        }
        sum / p as f64
    }

    fn source_value_at_index(&self, i: usize, src_kind: i32) -> Option<f64> {
        let x = match src_kind {
            MA_SRC_CLOSE => self.rows.get(i).map(|r| r.close),
            MA_SRC_TRUE_RANGE => Some(self.true_range_at(i as i32)),
            _ => None,
        }?;
        if x.is_nan() {
            None
        } else {
            Some(x)
        }
    }

    /// Wilder-style EMA seed: SMA of the first `period` samples, then standard EMA recurrence.
    pub fn ta_ema(&self, src_kind: i32, period: i32) -> f64 {
        let p = period as usize;
        if p == 0 || self.current_bar < 0 {
            return f64::NAN;
        }
        let end = self.current_bar as usize;
        if end + 1 < p {
            return f64::NAN;
        }
        let k = 2.0 / (period as f64 + 1.0);

        let mut ema = 0.0f64;
        for i in 0..p {
            let Some(x) = self.source_value_at_index(i, src_kind) else {
                return f64::NAN;
            };
            ema += x;
        }
        ema /= p as f64;

        for i in p..=end {
            let Some(x) = self.source_value_at_index(i, src_kind) else {
                return f64::NAN;
            };
            ema = x * k + ema * (1.0 - k);
        }
        ema
    }

    pub fn ta_atr(&self, period: i32) -> f64 {
        let p = period as usize;
        if p == 0 || self.current_bar < 0 {
            return f64::NAN;
        }
        let end = self.current_bar as usize;
        if end + 1 < p {
            return f64::NAN;
        }

        let mut trs = Vec::with_capacity(end + 1);
        for i in 0..=end {
            trs.push(self.true_range_at(i as i32));
        }

        let mut atr = trs.iter().take(p).sum::<f64>() / p as f64;
        for i in p..=end {
            let tr = trs[i];
            atr = (atr * (p as f64 - 1.0) + tr) / p as f64;
        }
        atr
    }

    pub fn ta_crossover(&mut self, a: f64, b: f64) -> f64 {
        let out = match (self.cross_prev_a, self.cross_prev_b) {
            (Some(pa), Some(pb)) if !pa.is_nan() && !pb.is_nan() && !a.is_nan() && !b.is_nan() => {
                if pa <= pb && a > b {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };
        self.cross_prev_a = Some(a);
        self.cross_prev_b = Some(b);
        out
    }

    pub fn ta_crossunder(&mut self, a: f64, b: f64) -> f64 {
        let out = match (self.under_prev_a, self.under_prev_b) {
            (Some(pa), Some(pb)) if !pa.is_nan() && !pb.is_nan() && !a.is_nan() && !b.is_nan() => {
                if pa >= pb && a < b {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };
        self.under_prev_a = Some(a);
        self.under_prev_b = Some(b);
        out
    }
}

fn parse_f64(s: &str) -> AetherResult<f64> {
    s.trim()
        .parse::<f64>()
        .map_err(|e| AetherError::Sandbox(format!("invalid OHLCV numeric: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows_from_closes(closes: &[f64]) -> AetherResult<BarFeedState> {
        let mut v = Vec::new();
        for &c in closes {
            let s = c.to_string();
            v.push((s.clone(), s.clone(), s.clone(), s, "0".into()));
        }
        BarFeedState::from_ohlcv_strings(&v)
    }

    #[test]
    fn hist_close_offset_matches_pine_indexing() {
        let mut f = rows_from_closes(&[1.0, 2.0, 3.0]).expect("parse");
        f.current_bar = 2;
        assert!((f.hist_at(SK_CLOSE, 0) - 3.0).abs() < 1e-9);
        assert!((f.hist_at(SK_CLOSE, 1) - 2.0).abs() < 1e-9);
        assert!(f.hist_at(SK_CLOSE, 3).is_nan());
    }

    #[test]
    fn sma_close_end_window() {
        let mut f = rows_from_closes(&[1.0, 2.0, 3.0, 4.0]).expect("parse");
        f.current_bar = 3;
        assert!((f.ta_sma(MA_SRC_CLOSE, 2) - 3.5).abs() < 1e-9);
    }
}
