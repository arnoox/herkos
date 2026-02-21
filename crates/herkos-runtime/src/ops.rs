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
    if v.is_nan() || v >= 2147483648.0f32 || v < -2147483648.0f32 {
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
    if v.is_nan() || v >= 2147483648.0f64 || v < -2147483648.0f64 {
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
    if v.is_nan() || v >= 9223372036854775808.0f32 || v < -9223372036854775808.0f32 {
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
    if v.is_nan() || v >= 9223372036854775808.0f64 || v < -9223372036854775808.0f64 {
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
pub fn i32_rem_s(lhs: i32, rhs: i32) -> WasmResult<i32> {
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
pub fn i64_rem_s(lhs: i64, rhs: i64) -> WasmResult<i64> {
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
}
