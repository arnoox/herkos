//! Wasm numeric operations that require runtime checks.
//!
//! ## Float-to-integer truncation
//!
//! Rust's `as` cast for float-to-integer is **saturating** since Rust 1.45
//! (e.g. `f32::INFINITY as i32 == i32::MAX`). Wasm requires a **trap** for
//! out-of-range or NaN input. Each function below validates the input before
//! casting.
//!
//! ## Integer division / remainder
//!
//! `i32::checked_div` / `checked_rem` return `None` for both divide-by-zero
//! and signed overflow (`i32::MIN / -1`). Both cases trap in Wasm. Note that
//! `checked_rem(i32::MIN, -1)` returns `Some(0)`, which is the correct Wasm
//! result (the remainder after a would-be-overflowing division is 0).
//!
//! All functions are `#[inline(never)]` (outline pattern §13.3). There are no
//! generics here, so the public function IS the inner function — no wrapper
//! split is needed.
//!
//! `no_std` compatible: no alloc, no std, no panics.

use crate::{WasmResult, WasmTrap};

// ── Float → i32 trapping truncation ──────────────────────────────────────────

/// Wasm `i32.trunc_f32_s`: truncate f32 toward zero to i32, trapping on NaN/overflow.
#[inline(never)]
pub fn i32_trunc_f32_s(v: f32) -> WasmResult<i32> {
    if v.is_nan() || !(-2147483648.0f32..2147483648.0f32).contains(&v) {
        return Err(WasmTrap::IntegerOverflow);
    }
    Ok(v as i32)
}

/// Wasm `i32.trunc_f32_u`: truncate f32 toward zero to u32 (returned as i32),
/// trapping on NaN or out-of-range input.
///
/// The lower bound is `<= -1.0` (not `< 0.0`) because `-0.5` truncates to 0,
/// which is a valid unsigned value; only values at -1.0 or below are out of range.
#[inline(never)]
pub fn i32_trunc_f32_u(v: f32) -> WasmResult<i32> {
    if v.is_nan() || v >= 4294967296.0f32 || v <= -1.0f32 {
        return Err(WasmTrap::IntegerOverflow);
    }
    Ok(v as u32 as i32)
}

/// Wasm `i32.trunc_f64_s`: truncate f64 toward zero to i32, trapping on NaN/overflow.
#[inline(never)]
pub fn i32_trunc_f64_s(v: f64) -> WasmResult<i32> {
    if v.is_nan() || !(-2147483648.0f64..2147483648.0f64).contains(&v) {
        return Err(WasmTrap::IntegerOverflow);
    }
    Ok(v as i32)
}

/// Wasm `i32.trunc_f64_u`: truncate f64 toward zero to u32 (returned as i32),
/// trapping on NaN or out-of-range input.
#[inline(never)]
pub fn i32_trunc_f64_u(v: f64) -> WasmResult<i32> {
    if v.is_nan() || v >= 4294967296.0f64 || v <= -1.0f64 {
        return Err(WasmTrap::IntegerOverflow);
    }
    Ok(v as u32 as i32)
}

// ── Float → i64 trapping truncation ──────────────────────────────────────────

/// Wasm `i64.trunc_f32_s`: truncate f32 toward zero to i64, trapping on NaN/overflow.
#[inline(never)]
pub fn i64_trunc_f32_s(v: f32) -> WasmResult<i64> {
    if v.is_nan() || !(-9223372036854775808.0f32..9223372036854775808.0f32).contains(&v) {
        return Err(WasmTrap::IntegerOverflow);
    }
    Ok(v as i64)
}

/// Wasm `i64.trunc_f32_u`: truncate f32 toward zero to u64 (returned as i64),
/// trapping on NaN or out-of-range input.
#[inline(never)]
pub fn i64_trunc_f32_u(v: f32) -> WasmResult<i64> {
    if v.is_nan() || v >= 18446744073709551616.0f32 || v <= -1.0f32 {
        return Err(WasmTrap::IntegerOverflow);
    }
    Ok(v as u64 as i64)
}

/// Wasm `i64.trunc_f64_s`: truncate f64 toward zero to i64, trapping on NaN/overflow.
#[inline(never)]
pub fn i64_trunc_f64_s(v: f64) -> WasmResult<i64> {
    if v.is_nan() || !(-9223372036854775808.0f64..9223372036854775808.0f64).contains(&v) {
        return Err(WasmTrap::IntegerOverflow);
    }
    Ok(v as i64)
}

/// Wasm `i64.trunc_f64_u`: truncate f64 toward zero to u64 (returned as i64),
/// trapping on NaN or out-of-range input.
#[inline(never)]
pub fn i64_trunc_f64_u(v: f64) -> WasmResult<i64> {
    if v.is_nan() || v >= 18446744073709551616.0f64 || v <= -1.0f64 {
        return Err(WasmTrap::IntegerOverflow);
    }
    Ok(v as u64 as i64)
}

// ── i32 division / remainder ──────────────────────────────────────────────────

/// Wasm `i32.div_s`: signed integer division, trapping on divide-by-zero or
/// signed overflow (`i32::MIN / -1`). Both cases produce `None` from
/// `checked_div`, which maps to `DivisionByZero`.
#[inline(never)]
pub fn i32_div_s(lhs: i32, rhs: i32) -> WasmResult<i32> {
    lhs.checked_div(rhs).ok_or(WasmTrap::DivisionByZero)
}

/// Wasm `i32.div_u`: unsigned integer division, trapping on divide-by-zero.
#[inline(never)]
pub fn i32_div_u(lhs: i32, rhs: i32) -> WasmResult<i32> {
    (lhs as u32)
        .checked_div(rhs as u32)
        .map(|v| v as i32)
        .ok_or(WasmTrap::DivisionByZero)
}

/// Wasm `i32.rem_s`: signed remainder, trapping on divide-by-zero.
///
/// The Wasm spec (§4.3.2) defines `i32::MIN rem_s -1 = 0` — the mathematical
/// remainder is 0 and this does NOT trap. Rust's `checked_rem` returns `None`
/// for this case (because the underlying division overflows), so we handle it
/// with an explicit branch.
#[inline(never)]
pub const fn i32_rem_s(lhs: i32, rhs: i32) -> WasmResult<i32> {
    if rhs == 0 {
        return Err(WasmTrap::DivisionByZero);
    }
    // i32::MIN % -1 would overflow the division, but the remainder is 0.
    Ok(if lhs == i32::MIN && rhs == -1 {
        0
    } else {
        lhs % rhs
    })
}

/// Wasm `i32.rem_u`: unsigned remainder, trapping on divide-by-zero.
#[inline(never)]
pub fn i32_rem_u(lhs: i32, rhs: i32) -> WasmResult<i32> {
    (lhs as u32)
        .checked_rem(rhs as u32)
        .map(|v| v as i32)
        .ok_or(WasmTrap::DivisionByZero)
}

// ── i64 division / remainder ──────────────────────────────────────────────────

/// Wasm `i64.div_s`: signed integer division, trapping on divide-by-zero or
/// signed overflow (`i64::MIN / -1`).
#[inline(never)]
pub fn i64_div_s(lhs: i64, rhs: i64) -> WasmResult<i64> {
    lhs.checked_div(rhs).ok_or(WasmTrap::DivisionByZero)
}

/// Wasm `i64.div_u`: unsigned integer division, trapping on divide-by-zero.
#[inline(never)]
pub fn i64_div_u(lhs: i64, rhs: i64) -> WasmResult<i64> {
    (lhs as u64)
        .checked_div(rhs as u64)
        .map(|v| v as i64)
        .ok_or(WasmTrap::DivisionByZero)
}

/// Wasm `i64.rem_s`: signed remainder, trapping on divide-by-zero.
///
/// Same special case as `i32_rem_s`: `i64::MIN rem_s -1 = 0` per Wasm spec.
#[inline(never)]
pub const fn i64_rem_s(lhs: i64, rhs: i64) -> WasmResult<i64> {
    if rhs == 0 {
        return Err(WasmTrap::DivisionByZero);
    }
    Ok(if lhs == i64::MIN && rhs == -1 {
        0
    } else {
        lhs % rhs
    })
}

/// Wasm `i64.rem_u`: unsigned remainder, trapping on divide-by-zero.
#[inline(never)]
pub fn i64_rem_u(lhs: i64, rhs: i64) -> WasmResult<i64> {
    (lhs as u64)
        .checked_rem(rhs as u64)
        .map(|v| v as i64)
        .ok_or(WasmTrap::DivisionByZero)
}

// ── Wasm float min/max/nearest ────────────────────────────────────────────────

/// Wasm `f32.min`: propagates NaN (unlike Rust's `f32::min` which ignores it).
/// Also preserves the Wasm rule `min(-0.0, +0.0) = -0.0`.
pub const fn wasm_min_f32(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        return f32::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_negative() { a } else { b };
    }
    if a <= b {
        a
    } else {
        b
    }
}

/// Wasm `f32.max`: propagates NaN. `max(-0.0, +0.0) = +0.0`.
pub const fn wasm_max_f32(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        return f32::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_positive() { a } else { b };
    }
    if a >= b {
        a
    } else {
        b
    }
}

/// Wasm `f64.min`: propagates NaN. `min(-0.0, +0.0) = -0.0`.
pub const fn wasm_min_f64(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_negative() { a } else { b };
    }
    if a <= b {
        a
    } else {
        b
    }
}

/// Wasm `f64.max`: propagates NaN. `max(-0.0, +0.0) = +0.0`.
pub const fn wasm_max_f64(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_positive() { a } else { b };
    }
    if a >= b {
        a
    } else {
        b
    }
}

/// Wasm `f32.nearest` — round to nearest even (banker's rounding).
///
/// Uses `as i32` for truncation-toward-zero (safe since we guard against values >= 2^23,
/// which have no fractional bits). Avoids `f32::round`/`f32::trunc`
/// which are not available in `no_std` without `libm`.
pub fn wasm_nearest_f32(v: f32) -> f32 {
    if v.is_nan() || v.is_infinite() || v == 0.0 {
        return v;
    }
    // Floats >= 2^23 have no fractional bits — already an integer.
    const NO_FRAC: f32 = 8_388_608.0; // 2^23
    if v >= NO_FRAC || v <= -NO_FRAC {
        return v;
    }
    let trunc_i = v as i32; // truncates toward zero; safe since |v| < 2^23
    let trunc_f = trunc_i as f32;
    let frac = v - trunc_f; // in (-1.0, 1.0), same sign as v
    if frac > 0.5 {
        (trunc_i + 1) as f32
    } else if frac < -0.5 {
        (trunc_i - 1) as f32
    } else if frac == 0.5 {
        // Tie: round to even (trunc_i is the floor for positive v).
        if trunc_i % 2 == 0 {
            trunc_f
        } else {
            (trunc_i + 1) as f32
        }
    } else if frac == -0.5 {
        // Tie: round to even. copysign preserves -0.0 when trunc_i == 0.
        if trunc_i % 2 == 0 {
            f32::copysign(trunc_f, v)
        } else {
            (trunc_i - 1) as f32
        }
    } else {
        trunc_f
    }
}

/// Wasm `f64.nearest` — round to nearest even (banker's rounding).
///
/// Uses `as i64` for truncation-toward-zero (safe since we guard against values >= 2^52,
/// which have no fractional bits). Avoids `f64::round`/`f64::trunc`
/// which are not available in `no_std` without `libm`.
pub fn wasm_nearest_f64(v: f64) -> f64 {
    if v.is_nan() || v.is_infinite() || v == 0.0 {
        return v;
    }
    // Floats >= 2^52 have no fractional bits — already an integer.
    const NO_FRAC: f64 = 4_503_599_627_370_496.0; // 2^52
    if v >= NO_FRAC || v <= -NO_FRAC {
        return v;
    }
    let trunc_i = v as i64; // truncates toward zero; safe since |v| < 2^52
    let trunc_f = trunc_i as f64;
    let frac = v - trunc_f;
    if frac > 0.5 {
        (trunc_i + 1) as f64
    } else if frac < -0.5 {
        (trunc_i - 1) as f64
    } else if frac == 0.5 {
        if trunc_i % 2 == 0 {
            trunc_f
        } else {
            (trunc_i + 1) as f64
        }
    } else if frac == -0.5 {
        if trunc_i % 2 == 0 {
            f64::copysign(trunc_f, v)
        } else {
            (trunc_i - 1) as f64
        }
    } else {
        trunc_f
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── i32_trunc_f32_s ──────────────────────────────────────────────────────

    #[test]
    fn i32_trunc_f32_s_positive() {
        assert_eq!(i32_trunc_f32_s(1.9f32).unwrap(), 1);
    }

    #[test]
    fn i32_trunc_f32_s_negative() {
        assert_eq!(i32_trunc_f32_s(-1.9f32).unwrap(), -1);
    }

    #[test]
    fn i32_trunc_f32_s_zero() {
        assert_eq!(i32_trunc_f32_s(0.0f32).unwrap(), 0);
    }

    #[test]
    fn i32_trunc_f32_s_nan() {
        assert_eq!(i32_trunc_f32_s(f32::NAN), Err(WasmTrap::IntegerOverflow));
    }

    #[test]
    fn i32_trunc_f32_s_pos_inf() {
        assert_eq!(
            i32_trunc_f32_s(f32::INFINITY),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    #[test]
    fn i32_trunc_f32_s_neg_inf() {
        assert_eq!(
            i32_trunc_f32_s(f32::NEG_INFINITY),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    #[test]
    fn i32_trunc_f32_s_overflow() {
        // 2^31 is one past i32::MAX
        assert_eq!(
            i32_trunc_f32_s(2147483648.0f32),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    #[test]
    fn i32_trunc_f32_s_underflow() {
        assert_eq!(
            i32_trunc_f32_s(-2147483904.0f32),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    // ── i32_trunc_f32_u ──────────────────────────────────────────────────────

    #[test]
    fn i32_trunc_f32_u_positive() {
        assert_eq!(i32_trunc_f32_u(3.9f32).unwrap(), 3);
    }

    #[test]
    fn i32_trunc_f32_u_zero() {
        assert_eq!(i32_trunc_f32_u(0.0f32).unwrap(), 0);
    }

    #[test]
    fn i32_trunc_f32_u_neg_half_ok() {
        // trunc(-0.5) = 0, which is valid for unsigned
        assert_eq!(i32_trunc_f32_u(-0.5f32).unwrap(), 0);
    }

    #[test]
    fn i32_trunc_f32_u_neg_one_err() {
        assert_eq!(i32_trunc_f32_u(-1.0f32), Err(WasmTrap::IntegerOverflow));
    }

    #[test]
    fn i32_trunc_f32_u_overflow() {
        assert_eq!(
            i32_trunc_f32_u(4294967296.0f32),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    #[test]
    fn i32_trunc_f32_u_nan() {
        assert_eq!(i32_trunc_f32_u(f32::NAN), Err(WasmTrap::IntegerOverflow));
    }

    // ── i32_trunc_f64_s ──────────────────────────────────────────────────────

    #[test]
    fn i32_trunc_f64_s_max() {
        assert_eq!(i32_trunc_f64_s(2147483647.0f64).unwrap(), i32::MAX);
    }

    #[test]
    fn i32_trunc_f64_s_min() {
        assert_eq!(i32_trunc_f64_s(-2147483648.0f64).unwrap(), i32::MIN);
    }

    #[test]
    fn i32_trunc_f64_s_overflow() {
        assert_eq!(
            i32_trunc_f64_s(2147483648.0f64),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    #[test]
    fn i32_trunc_f64_s_underflow() {
        assert_eq!(
            i32_trunc_f64_s(-2147483649.0f64),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    #[test]
    fn i32_trunc_f64_s_nan() {
        assert_eq!(i32_trunc_f64_s(f64::NAN), Err(WasmTrap::IntegerOverflow));
    }

    // ── i32_trunc_f64_u ──────────────────────────────────────────────────────

    #[test]
    fn i32_trunc_f64_u_max() {
        // u32::MAX reinterpreted as i32 = -1
        assert_eq!(i32_trunc_f64_u(4294967295.0f64).unwrap(), -1i32);
    }

    #[test]
    fn i32_trunc_f64_u_overflow() {
        assert_eq!(
            i32_trunc_f64_u(4294967296.0f64),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    #[test]
    fn i32_trunc_f64_u_neg_one_err() {
        assert_eq!(i32_trunc_f64_u(-1.0f64), Err(WasmTrap::IntegerOverflow));
    }

    #[test]
    fn i32_trunc_f64_u_neg_half_ok() {
        assert_eq!(i32_trunc_f64_u(-0.9f64).unwrap(), 0);
    }

    #[test]
    fn i32_trunc_f64_u_nan() {
        assert_eq!(i32_trunc_f64_u(f64::NAN), Err(WasmTrap::IntegerOverflow));
    }

    // ── i64_trunc_f32_s ──────────────────────────────────────────────────────

    #[test]
    fn i64_trunc_f32_s_ok() {
        assert_eq!(i64_trunc_f32_s(1.5f32).unwrap(), 1i64);
    }

    #[test]
    fn i64_trunc_f32_s_nan() {
        assert_eq!(i64_trunc_f32_s(f32::NAN), Err(WasmTrap::IntegerOverflow));
    }

    #[test]
    fn i64_trunc_f32_s_overflow() {
        assert_eq!(
            i64_trunc_f32_s(9223372036854775808.0f32),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    #[test]
    fn i64_trunc_f32_s_pos_inf() {
        assert_eq!(
            i64_trunc_f32_s(f32::INFINITY),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    // ── i64_trunc_f32_u ──────────────────────────────────────────────────────

    #[test]
    fn i64_trunc_f32_u_ok() {
        assert_eq!(i64_trunc_f32_u(1000.0f32).unwrap(), 1000i64);
    }

    #[test]
    fn i64_trunc_f32_u_neg_one_err() {
        assert_eq!(i64_trunc_f32_u(-1.0f32), Err(WasmTrap::IntegerOverflow));
    }

    #[test]
    fn i64_trunc_f32_u_nan() {
        assert_eq!(i64_trunc_f32_u(f32::NAN), Err(WasmTrap::IntegerOverflow));
    }

    // ── i64_trunc_f64_s ──────────────────────────────────────────────────────

    #[test]
    fn i64_trunc_f64_s_ok() {
        // Largest f64 that fits exactly in i64
        assert_eq!(
            i64_trunc_f64_s(9223372036854774784.0f64).unwrap(),
            9223372036854774784i64
        );
    }

    #[test]
    fn i64_trunc_f64_s_nan() {
        assert_eq!(i64_trunc_f64_s(f64::NAN), Err(WasmTrap::IntegerOverflow));
    }

    #[test]
    fn i64_trunc_f64_s_overflow() {
        assert_eq!(
            i64_trunc_f64_s(9223372036854775808.0f64),
            Err(WasmTrap::IntegerOverflow)
        );
    }

    // ── i64_trunc_f64_u ──────────────────────────────────────────────────────

    #[test]
    fn i64_trunc_f64_u_ok() {
        assert_eq!(i64_trunc_f64_u(100.0f64).unwrap(), 100i64);
    }

    #[test]
    fn i64_trunc_f64_u_neg_one_err() {
        assert_eq!(i64_trunc_f64_u(-1.0f64), Err(WasmTrap::IntegerOverflow));
    }

    #[test]
    fn i64_trunc_f64_u_nan() {
        assert_eq!(i64_trunc_f64_u(f64::NAN), Err(WasmTrap::IntegerOverflow));
    }

    // ── i32_div_s ────────────────────────────────────────────────────────────

    #[test]
    fn i32_div_s_basic() {
        assert_eq!(i32_div_s(10, 3).unwrap(), 3);
    }

    #[test]
    fn i32_div_s_negative() {
        assert_eq!(i32_div_s(-10, 3).unwrap(), -3);
    }

    #[test]
    fn i32_div_s_zero_divisor() {
        assert_eq!(i32_div_s(5, 0), Err(WasmTrap::DivisionByZero));
    }

    #[test]
    fn i32_div_s_min_over_neg_one() {
        // i32::MIN / -1 overflows → trap
        assert_eq!(i32_div_s(i32::MIN, -1), Err(WasmTrap::DivisionByZero));
    }

    // ── i32_div_u ────────────────────────────────────────────────────────────

    #[test]
    fn i32_div_u_basic() {
        assert_eq!(i32_div_u(10, 3).unwrap(), 3);
    }

    #[test]
    fn i32_div_u_large() {
        // u32::MAX / 1 = u32::MAX, reinterpreted as i32 = -1
        assert_eq!(i32_div_u(-1i32, 1).unwrap(), -1i32);
    }

    #[test]
    fn i32_div_u_zero_divisor() {
        assert_eq!(i32_div_u(5, 0), Err(WasmTrap::DivisionByZero));
    }

    // ── i32_rem_s ────────────────────────────────────────────────────────────

    #[test]
    fn i32_rem_s_basic() {
        assert_eq!(i32_rem_s(10, 3).unwrap(), 1);
    }

    #[test]
    fn i32_rem_s_negative() {
        assert_eq!(i32_rem_s(-10, 3).unwrap(), -1);
    }

    #[test]
    fn i32_rem_s_zero_divisor() {
        assert_eq!(i32_rem_s(5, 0), Err(WasmTrap::DivisionByZero));
    }

    #[test]
    fn i32_rem_s_min_over_neg_one() {
        // checked_rem(i32::MIN, -1) = Some(0) — correct Wasm result
        assert_eq!(i32_rem_s(i32::MIN, -1).unwrap(), 0);
    }

    // ── i32_rem_u ────────────────────────────────────────────────────────────

    #[test]
    fn i32_rem_u_basic() {
        assert_eq!(i32_rem_u(10, 3).unwrap(), 1);
    }

    #[test]
    fn i32_rem_u_zero_divisor() {
        assert_eq!(i32_rem_u(5, 0), Err(WasmTrap::DivisionByZero));
    }

    // ── i64_div_s ────────────────────────────────────────────────────────────

    #[test]
    fn i64_div_s_basic() {
        assert_eq!(i64_div_s(100, 7).unwrap(), 14);
    }

    #[test]
    fn i64_div_s_zero_divisor() {
        assert_eq!(i64_div_s(5, 0), Err(WasmTrap::DivisionByZero));
    }

    #[test]
    fn i64_div_s_min_over_neg_one() {
        assert_eq!(i64_div_s(i64::MIN, -1), Err(WasmTrap::DivisionByZero));
    }

    // ── i64_div_u ────────────────────────────────────────────────────────────

    #[test]
    fn i64_div_u_basic() {
        assert_eq!(i64_div_u(17, 5).unwrap(), 3);
    }

    #[test]
    fn i64_div_u_zero_divisor() {
        assert_eq!(i64_div_u(5, 0), Err(WasmTrap::DivisionByZero));
    }

    // ── i64_rem_s ────────────────────────────────────────────────────────────

    #[test]
    fn i64_rem_s_basic() {
        assert_eq!(i64_rem_s(17, 5).unwrap(), 2);
    }

    #[test]
    fn i64_rem_s_zero_divisor() {
        assert_eq!(i64_rem_s(5, 0), Err(WasmTrap::DivisionByZero));
    }

    #[test]
    fn i64_rem_s_min_over_neg_one() {
        // i64::MIN rem_s -1 = 0 per Wasm spec (does not trap)
        assert_eq!(i64_rem_s(i64::MIN, -1).unwrap(), 0);
    }

    // ── i64_rem_u ────────────────────────────────────────────────────────────

    #[test]
    fn i64_rem_u_basic() {
        assert_eq!(i64_rem_u(17, 5).unwrap(), 2);
    }

    #[test]
    fn i64_rem_u_zero_divisor() {
        assert_eq!(i64_rem_u(5, 0), Err(WasmTrap::DivisionByZero));
    }

    // ── wasm_min_f32 ─────────────────────────────────────────────────────────

    #[test]
    fn wasm_min_f32_normal() {
        assert_eq!(wasm_min_f32(3.0, 5.0), 3.0);
        assert_eq!(wasm_min_f32(5.0, 3.0), 3.0);
    }

    #[test]
    fn wasm_min_f32_equal() {
        assert_eq!(wasm_min_f32(2.5, 2.5), 2.5);
    }

    #[test]
    fn wasm_min_f32_nan_left() {
        assert!(wasm_min_f32(f32::NAN, 5.0).is_nan());
    }

    #[test]
    fn wasm_min_f32_nan_right() {
        assert!(wasm_min_f32(5.0, f32::NAN).is_nan());
    }

    #[test]
    fn wasm_min_f32_both_nan() {
        assert!(wasm_min_f32(f32::NAN, f32::NAN).is_nan());
    }

    #[test]
    fn wasm_min_f32_neg_zero_pos_zero() {
        // min(-0.0, +0.0) should return -0.0
        let result = wasm_min_f32(-0.0, 0.0);
        assert_eq!(result, 0.0);
        assert!(result.is_sign_negative());
    }

    #[test]
    fn wasm_min_f32_pos_zero_neg_zero() {
        // min(+0.0, -0.0) should return -0.0
        let result = wasm_min_f32(0.0, -0.0);
        assert_eq!(result, 0.0);
        assert!(result.is_sign_negative());
    }

    #[test]
    fn wasm_min_f32_infinity() {
        assert_eq!(wasm_min_f32(f32::INFINITY, 100.0), 100.0);
        assert_eq!(wasm_min_f32(100.0, f32::INFINITY), 100.0);
    }

    #[test]
    fn wasm_min_f32_neg_infinity() {
        assert_eq!(wasm_min_f32(f32::NEG_INFINITY, 100.0), f32::NEG_INFINITY);
        assert_eq!(wasm_min_f32(100.0, f32::NEG_INFINITY), f32::NEG_INFINITY);
    }

    // ── wasm_max_f32 ─────────────────────────────────────────────────────────

    #[test]
    fn wasm_max_f32_normal() {
        assert_eq!(wasm_max_f32(3.0, 5.0), 5.0);
        assert_eq!(wasm_max_f32(5.0, 3.0), 5.0);
    }

    #[test]
    fn wasm_max_f32_equal() {
        assert_eq!(wasm_max_f32(2.5, 2.5), 2.5);
    }

    #[test]
    fn wasm_max_f32_nan_left() {
        assert!(wasm_max_f32(f32::NAN, 5.0).is_nan());
    }

    #[test]
    fn wasm_max_f32_nan_right() {
        assert!(wasm_max_f32(5.0, f32::NAN).is_nan());
    }

    #[test]
    fn wasm_max_f32_neg_zero_pos_zero() {
        // max(-0.0, +0.0) should return +0.0
        let result = wasm_max_f32(-0.0, 0.0);
        assert_eq!(result, 0.0);
        assert!(result.is_sign_positive());
    }

    #[test]
    fn wasm_max_f32_pos_zero_neg_zero() {
        // max(+0.0, -0.0) should return +0.0
        let result = wasm_max_f32(0.0, -0.0);
        assert_eq!(result, 0.0);
        assert!(result.is_sign_positive());
    }

    #[test]
    fn wasm_max_f32_infinity() {
        assert_eq!(wasm_max_f32(f32::INFINITY, 100.0), f32::INFINITY);
        assert_eq!(wasm_max_f32(100.0, f32::INFINITY), f32::INFINITY);
    }

    #[test]
    fn wasm_max_f32_neg_infinity() {
        assert_eq!(wasm_max_f32(f32::NEG_INFINITY, 100.0), 100.0);
        assert_eq!(wasm_max_f32(100.0, f32::NEG_INFINITY), 100.0);
    }

    // ── wasm_min_f64 ─────────────────────────────────────────────────────────

    #[test]
    fn wasm_min_f64_normal() {
        assert_eq!(wasm_min_f64(3.0, 5.0), 3.0);
        assert_eq!(wasm_min_f64(5.0, 3.0), 3.0);
    }

    #[test]
    fn wasm_min_f64_equal() {
        assert_eq!(wasm_min_f64(2.5, 2.5), 2.5);
    }

    #[test]
    fn wasm_min_f64_nan_left() {
        assert!(wasm_min_f64(f64::NAN, 5.0).is_nan());
    }

    #[test]
    fn wasm_min_f64_nan_right() {
        assert!(wasm_min_f64(5.0, f64::NAN).is_nan());
    }

    #[test]
    fn wasm_min_f64_both_nan() {
        assert!(wasm_min_f64(f64::NAN, f64::NAN).is_nan());
    }

    #[test]
    fn wasm_min_f64_neg_zero_pos_zero() {
        // min(-0.0, +0.0) should return -0.0
        let result = wasm_min_f64(-0.0, 0.0);
        assert_eq!(result, 0.0);
        assert!(result.is_sign_negative());
    }

    #[test]
    fn wasm_min_f64_pos_zero_neg_zero() {
        // min(+0.0, -0.0) should return -0.0
        let result = wasm_min_f64(0.0, -0.0);
        assert_eq!(result, 0.0);
        assert!(result.is_sign_negative());
    }

    // ── wasm_max_f64 ─────────────────────────────────────────────────────────

    #[test]
    fn wasm_max_f64_normal() {
        assert_eq!(wasm_max_f64(3.0, 5.0), 5.0);
        assert_eq!(wasm_max_f64(5.0, 3.0), 5.0);
    }

    #[test]
    fn wasm_max_f64_equal() {
        assert_eq!(wasm_max_f64(2.5, 2.5), 2.5);
    }

    #[test]
    fn wasm_max_f64_nan_left() {
        assert!(wasm_max_f64(f64::NAN, 5.0).is_nan());
    }

    #[test]
    fn wasm_max_f64_nan_right() {
        assert!(wasm_max_f64(5.0, f64::NAN).is_nan());
    }

    #[test]
    fn wasm_max_f64_neg_zero_pos_zero() {
        // max(-0.0, +0.0) should return +0.0
        let result = wasm_max_f64(-0.0, 0.0);
        assert_eq!(result, 0.0);
        assert!(result.is_sign_positive());
    }

    #[test]
    fn wasm_max_f64_pos_zero_neg_zero() {
        // max(+0.0, -0.0) should return +0.0
        let result = wasm_max_f64(0.0, -0.0);
        assert_eq!(result, 0.0);
        assert!(result.is_sign_positive());
    }

    // ── wasm_nearest_f32 ─────────────────────────────────────────────────────

    #[test]
    fn wasm_nearest_f32_integer() {
        assert_eq!(wasm_nearest_f32(5.0), 5.0);
        assert_eq!(wasm_nearest_f32(-3.0), -3.0);
    }

    #[test]
    fn wasm_nearest_f32_round_up() {
        // 2.7 rounds to 3.0 (frac = 0.7 > 0.5)
        assert_eq!(wasm_nearest_f32(2.7), 3.0);
    }

    #[test]
    fn wasm_nearest_f32_round_down() {
        // 2.2 rounds to 2.0 (frac = 0.2 < 0.5)
        assert_eq!(wasm_nearest_f32(2.2), 2.0);
    }

    #[test]
    fn wasm_nearest_f32_tie_round_to_even_down() {
        // 2.5: trunc_i = 2 (even), so round toward zero to 2
        assert_eq!(wasm_nearest_f32(2.5), 2.0);
    }

    #[test]
    fn wasm_nearest_f32_tie_round_to_even_up() {
        // 3.5: trunc_i = 3 (odd), so round away from zero to 4 (even)
        assert_eq!(wasm_nearest_f32(3.5), 4.0);
    }

    #[test]
    fn wasm_nearest_f32_negative_round_up() {
        // -2.7: frac = -0.7 < -0.5, so round toward zero is -2, away is -3
        // frac < -0.5 means round down (toward -inf) → -3
        assert_eq!(wasm_nearest_f32(-2.7), -3.0);
    }

    #[test]
    fn wasm_nearest_f32_negative_round_down() {
        // -2.2: frac = -0.2 > -0.5, so stays at -2
        assert_eq!(wasm_nearest_f32(-2.2), -2.0);
    }

    #[test]
    fn wasm_nearest_f32_negative_tie_round_to_even() {
        // -2.5: trunc_i = -2 (even), so round toward zero → -2.0
        let result = wasm_nearest_f32(-2.5);
        assert_eq!(result, -2.0);
        // Verify sign is preserved
        assert!(result.is_sign_negative() || result == 0.0);
    }

    #[test]
    fn wasm_nearest_f32_nan() {
        assert!(wasm_nearest_f32(f32::NAN).is_nan());
    }

    #[test]
    fn wasm_nearest_f32_infinity() {
        assert_eq!(wasm_nearest_f32(f32::INFINITY), f32::INFINITY);
        assert_eq!(wasm_nearest_f32(f32::NEG_INFINITY), f32::NEG_INFINITY);
    }

    #[test]
    fn wasm_nearest_f32_zero() {
        assert_eq!(wasm_nearest_f32(0.0), 0.0);
        assert_eq!(wasm_nearest_f32(-0.0), -0.0);
    }

    #[test]
    fn wasm_nearest_f32_large_no_frac() {
        // Values >= 2^23 have no fractional bits
        let large = 8_388_608.0f32; // 2^23
        assert_eq!(wasm_nearest_f32(large), large);
        assert_eq!(wasm_nearest_f32(large + 1.0), large + 1.0);
    }

    // ── wasm_nearest_f64 ─────────────────────────────────────────────────────

    #[test]
    fn wasm_nearest_f64_integer() {
        assert_eq!(wasm_nearest_f64(5.0), 5.0);
        assert_eq!(wasm_nearest_f64(-3.0), -3.0);
    }

    #[test]
    fn wasm_nearest_f64_round_up() {
        // 2.7 rounds to 3.0
        assert_eq!(wasm_nearest_f64(2.7), 3.0);
    }

    #[test]
    fn wasm_nearest_f64_round_down() {
        // 2.2 rounds to 2.0
        assert_eq!(wasm_nearest_f64(2.2), 2.0);
    }

    #[test]
    fn wasm_nearest_f64_tie_round_to_even_down() {
        // 2.5: trunc_i = 2 (even), so round toward zero to 2
        assert_eq!(wasm_nearest_f64(2.5), 2.0);
    }

    #[test]
    fn wasm_nearest_f64_tie_round_to_even_up() {
        // 3.5: trunc_i = 3 (odd), so round away from zero to 4 (even)
        assert_eq!(wasm_nearest_f64(3.5), 4.0);
    }

    #[test]
    fn wasm_nearest_f64_negative_round_up() {
        // -2.7: frac = -0.7 < -0.5, rounds down (toward -inf) → -3
        assert_eq!(wasm_nearest_f64(-2.7), -3.0);
    }

    #[test]
    fn wasm_nearest_f64_negative_round_down() {
        // -2.2: frac = -0.2 > -0.5, rounds to -2
        assert_eq!(wasm_nearest_f64(-2.2), -2.0);
    }

    #[test]
    fn wasm_nearest_f64_negative_tie_round_to_even() {
        // -2.5: trunc_i = -2 (even), round toward zero → -2.0
        let result = wasm_nearest_f64(-2.5);
        assert_eq!(result, -2.0);
        // Verify sign is preserved (or -0.0)
        assert!(result.is_sign_negative() || result == 0.0);
    }

    #[test]
    fn wasm_nearest_f64_nan() {
        assert!(wasm_nearest_f64(f64::NAN).is_nan());
    }

    #[test]
    fn wasm_nearest_f64_infinity() {
        assert_eq!(wasm_nearest_f64(f64::INFINITY), f64::INFINITY);
        assert_eq!(wasm_nearest_f64(f64::NEG_INFINITY), f64::NEG_INFINITY);
    }

    #[test]
    fn wasm_nearest_f64_zero() {
        assert_eq!(wasm_nearest_f64(0.0), 0.0);
        assert_eq!(wasm_nearest_f64(-0.0), -0.0);
    }

    #[test]
    fn wasm_nearest_f64_large_no_frac() {
        // Values >= 2^52 have no fractional bits
        let large = 4_503_599_627_370_496.0f64; // 2^52
        assert_eq!(wasm_nearest_f64(large), large);
        assert_eq!(wasm_nearest_f64(large + 1.0), large + 1.0);
    }
}
