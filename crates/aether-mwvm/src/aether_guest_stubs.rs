//! Wasmtime linker stubs for the `aether` import module emitted by `agentscript-compiler`.
//!
//! Keep in sync with `agentscript-compiler` `codegen/wasm/abi.rs` (`GUEST_ABI_V0_IMPORTS` order and signatures).
//! Guest **exports** are **`() -> i32`** (`init`) and **`(i32) -> i32`** (`step`) as of `guest_abi::VERSION` **3**; this file only registers **imports**.
//!
//! When [`crate::AetherGuestStoreState::bar_feed`] is [`Some`], series and TA imports read the
//! in-memory OHLCV replay (see [`crate::bar_series_host::BarFeedState`]). Otherwise they behave
//! like neutral CI stubs (zeros / pass-through).

use wasmtime::{Caller, Linker};

use crate::AetherGuestStoreState;

/// Register all `aether::*` functions required by `emit_hir_guest_wasm` in `agentscript-compiler`.
pub fn link_aether_guest_abi_v0(linker: &mut Linker<AetherGuestStoreState>) -> wasmtime::Result<()> {
    linker.func_wrap("aether", "series_close", |caller: Caller<'_, AetherGuestStoreState>| -> f64 {
        match caller.data().bar_feed.as_ref() {
            Some(feed) => feed.series_close(),
            None => 0.0,
        }
    })?;
    linker.func_wrap("aether", "input_int", |_: Caller<'_, AetherGuestStoreState>, _: i32| -> i32 {
        0
    })?;
    linker.func_wrap(
        "aether",
        "ta_sma",
        |caller: Caller<'_, AetherGuestStoreState>, src: i32, period: i32| -> f64 {
            match caller.data().bar_feed.as_ref() {
                Some(feed) => feed.ta_sma(src, period),
                None => 0.0,
            }
        },
    )?;
    linker.func_wrap(
        "aether",
        "request_security",
        |_: Caller<'_, AetherGuestStoreState>, _: i32, _: i32, _: i32, _: i32, inner: f64| -> f64 {
            inner
        },
    )?;
    linker.func_wrap("aether", "plot", |_: Caller<'_, AetherGuestStoreState>, _: f64| {})?;
    linker.func_wrap(
        "aether",
        "series_hist",
        |caller: Caller<'_, AetherGuestStoreState>, offset: i32| -> f64 {
            match caller.data().bar_feed.as_ref() {
                Some(feed) => feed.hist_at(0, offset),
                None => 0.0,
            }
        },
    )?;
    linker.func_wrap(
        "aether",
        "ta_ema",
        |caller: Caller<'_, AetherGuestStoreState>, src: i32, period: i32| -> f64 {
            match caller.data().bar_feed.as_ref() {
                Some(feed) => feed.ta_ema(src, period),
                None => 0.0,
            }
        },
    )?;
    linker.func_wrap(
        "aether",
        "input_float",
        |_: Caller<'_, AetherGuestStoreState>, _: i32| -> f64 { 0.0 },
    )?;
    linker.func_wrap(
        "aether",
        "ta_crossover",
        |mut caller: Caller<'_, AetherGuestStoreState>, a: f64, b: f64| -> f64 {
            match caller.data_mut().bar_feed.as_mut() {
                Some(feed) => feed.ta_crossover(a, b),
                None => 0.0,
            }
        },
    )?;
    linker.func_wrap(
        "aether",
        "ta_crossunder",
        |mut caller: Caller<'_, AetherGuestStoreState>, a: f64, b: f64| -> f64 {
            match caller.data_mut().bar_feed.as_mut() {
                Some(feed) => feed.ta_crossunder(a, b),
                None => 0.0,
            }
        },
    )?;
    linker.func_wrap("aether", "series_open", |caller: Caller<'_, AetherGuestStoreState>| -> f64 {
        match caller.data().bar_feed.as_ref() {
            Some(feed) => feed.series_open(),
            None => 0.0,
        }
    })?;
    linker.func_wrap("aether", "series_high", |caller: Caller<'_, AetherGuestStoreState>| -> f64 {
        match caller.data().bar_feed.as_ref() {
            Some(feed) => feed.series_high(),
            None => 0.0,
        }
    })?;
    linker.func_wrap("aether", "series_low", |caller: Caller<'_, AetherGuestStoreState>| -> f64 {
        match caller.data().bar_feed.as_ref() {
            Some(feed) => feed.series_low(),
            None => 0.0,
        }
    })?;
    linker.func_wrap(
        "aether",
        "series_volume",
        |caller: Caller<'_, AetherGuestStoreState>| -> f64 {
            match caller.data().bar_feed.as_ref() {
                Some(feed) => feed.series_volume(),
                None => 0.0,
            }
        },
    )?;
    linker.func_wrap(
        "aether",
        "series_time",
        |caller: Caller<'_, AetherGuestStoreState>| -> f64 {
            match caller.data().bar_feed.as_ref() {
                Some(feed) => feed.series_time(),
                None => 0.0,
            }
        },
    )?;
    linker.func_wrap(
        "aether",
        "series_hist_at",
        |caller: Caller<'_, AetherGuestStoreState>, kind: i32, offset: i32| -> f64 {
            match caller.data().bar_feed.as_ref() {
                Some(feed) => feed.hist_at(kind, offset),
                None => 0.0,
            }
        },
    )?;
    linker.func_wrap("aether", "ta_tr", |caller: Caller<'_, AetherGuestStoreState>| -> f64 {
        match caller.data().bar_feed.as_ref() {
            Some(feed) => feed.ta_tr(),
            None => 0.0,
        }
    })?;
    linker.func_wrap(
        "aether",
        "ta_atr",
        |caller: Caller<'_, AetherGuestStoreState>, period: i32| -> f64 {
            match caller.data().bar_feed.as_ref() {
                Some(feed) => feed.ta_atr(period),
                None => 0.0,
            }
        },
    )?;
    linker.func_wrap(
        "aether",
        "nz",
        |_: Caller<'_, AetherGuestStoreState>, x: f64, y: f64| -> f64 {
            if x.is_nan() {
                y
            } else {
                x
            }
        },
    )?;
    linker.func_wrap(
        "aether",
        "math_log",
        |_: Caller<'_, AetherGuestStoreState>, x: f64| -> f64 { x.ln() },
    )?;
    linker.func_wrap(
        "aether",
        "math_exp",
        |_: Caller<'_, AetherGuestStoreState>, x: f64| -> f64 { x.exp() },
    )?;
    linker.func_wrap(
        "aether",
        "math_pow",
        |_: Caller<'_, AetherGuestStoreState>, b: f64, e: f64| -> f64 { b.powf(e) },
    )?;
    linker.func_wrap(
        "aether",
        "request_financial",
        |_: Caller<'_, AetherGuestStoreState>,
         _: i32,
         _: i32,
         _: i32,
         _: i32,
         _: i32,
         _: i32,
         _: i32,
         _: i32,
         _: i32,
         _: i32|
         -> f64 { 0.0 },
    )?;
    linker.func_wrap(
        "aether",
        "series_string_utf8",
        |mut caller: Caller<'_, AetherGuestStoreState>, _kind: i32, dst: i32, max_len: i32| -> i32 {
            let Some(mem) = caller
                .get_export("memory")
                .and_then(|e| e.into_memory())
            else {
                return -1;
            };
            let demo = b"DEMO";
            let n = (demo.len() as i32).min(max_len).max(0) as usize;
            if n == 0 {
                return 0;
            }
            if mem.write(&mut caller, dst as usize, &demo[..n]).is_err() {
                return -1;
            }
            n as i32
        },
    )?;
    Ok(())
}
