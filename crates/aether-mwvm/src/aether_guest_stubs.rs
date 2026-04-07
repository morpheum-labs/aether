//! Wasmtime linker stubs for the `aether` import module emitted by `agentscript-compiler`.
//!
//! Keep in sync with `agentscript-compiler` `codegen/wasm/abi.rs` (`GUEST_ABI_V0_IMPORTS` order and signatures).
//! Guest **exports** are **`() -> i32`** (`init`) and **`(i32) -> i32`** (`step`) as of `guest_abi::VERSION` **3**; this file only registers **imports**.

use wasmtime::{Caller, Linker};

/// Register all `aether::*` functions required by `emit_hir_guest_wasm` in `agentscript-compiler`.
pub fn link_aether_guest_abi_v0<T>(linker: &mut Linker<T>) -> wasmtime::Result<()> {
    linker.func_wrap("aether", "series_close", || -> f64 { 0.0 })?;
    linker.func_wrap("aether", "input_int", |_: i32| -> i32 { 0 })?;
    linker.func_wrap("aether", "ta_sma", |_: i32, _: i32| -> f64 { 0.0 })?;
    linker.func_wrap(
        "aether",
        "request_security",
        |_: i32, _: i32, _: i32, _: i32, inner: f64| -> f64 { inner },
    )?;
    linker.func_wrap("aether", "plot", |_: f64| {})?;
    linker.func_wrap("aether", "series_hist", |_: i32| -> f64 { 0.0 })?;
    linker.func_wrap("aether", "ta_ema", |_: i32, _: i32| -> f64 { 0.0 })?;
    linker.func_wrap("aether", "input_float", |_: i32| -> f64 { 0.0 })?;
    linker.func_wrap("aether", "ta_crossover", |_: f64, _: f64| -> f64 { 0.0 })?;
    linker.func_wrap("aether", "ta_crossunder", |_: f64, _: f64| -> f64 { 0.0 })?;
    linker.func_wrap("aether", "series_open", || -> f64 { 0.0 })?;
    linker.func_wrap("aether", "series_high", || -> f64 { 0.0 })?;
    linker.func_wrap("aether", "series_low", || -> f64 { 0.0 })?;
    linker.func_wrap("aether", "series_volume", || -> f64 { 0.0 })?;
    linker.func_wrap("aether", "series_time", || -> f64 { 0.0 })?;
    linker.func_wrap("aether", "series_hist_at", |_: i32, _: i32| -> f64 { 0.0 })?;
    linker.func_wrap("aether", "ta_tr", || -> f64 { 0.0 })?;
    linker.func_wrap("aether", "ta_atr", |_: i32| -> f64 { 0.0 })?;
    linker.func_wrap("aether", "nz", |x: f64, y: f64| -> f64 {
        if x.is_nan() {
            y
        } else {
            x
        }
    })?;
    linker.func_wrap("aether", "math_log", |x: f64| -> f64 { x.ln() })?;
    linker.func_wrap("aether", "math_exp", |x: f64| -> f64 { x.exp() })?;
    linker.func_wrap("aether", "math_pow", |b: f64, e: f64| -> f64 { b.powf(e) })?;
    linker.func_wrap(
        "aether",
        "request_financial",
        |_: i32,
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
        |mut caller: Caller<'_, T>, _kind: i32, dst: i32, max_len: i32| -> i32 {
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
