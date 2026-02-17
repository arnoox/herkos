//! Runtime tests for Milestone 5: complete numeric operations.
//!
//! Tests i64 operations, f64 operations, and conversion operations.

use herkos_tests::{conversions, f64_ops, i64_ops};

// === i64 operations ===

#[test]
fn test_i64_div_s() {
    assert_eq!(i64_ops::func_0(100, 7).unwrap(), 14);
    assert_eq!(i64_ops::func_0(-100, 7).unwrap(), -14);
    assert_eq!(i64_ops::func_0(0, 5).unwrap(), 0);
}

#[test]
fn test_i64_div_s_trap_zero() {
    assert!(i64_ops::func_0(10, 0).is_err());
}

#[test]
fn test_i64_bitand() {
    assert_eq!(i64_ops::func_1(0xFF00, 0x0FF0).unwrap(), 0x0F00);
    assert_eq!(i64_ops::func_1(-1, 0x1234).unwrap(), 0x1234);
}

#[test]
fn test_i64_shl() {
    assert_eq!(i64_ops::func_2(1, 10).unwrap(), 1024);
    assert_eq!(i64_ops::func_2(1, 63).unwrap(), i64::MIN);
    // Shift amount is masked to 63
    assert_eq!(i64_ops::func_2(1, 64).unwrap(), 1);
}

#[test]
fn test_i64_lt_s() {
    assert_eq!(i64_ops::func_3(5, 10).unwrap(), 1);
    assert_eq!(i64_ops::func_3(10, 5).unwrap(), 0);
    assert_eq!(i64_ops::func_3(5, 5).unwrap(), 0);
    assert_eq!(i64_ops::func_3(-1, 0).unwrap(), 1);
}

#[test]
fn test_i64_clz() {
    assert_eq!(i64_ops::func_4(1).unwrap(), 63);
    assert_eq!(i64_ops::func_4(0).unwrap(), 64);
    assert_eq!(i64_ops::func_4(-1).unwrap(), 0);
}

#[test]
fn test_i64_rotl() {
    assert_eq!(i64_ops::func_5(1, 1).unwrap(), 2);
    assert_eq!(i64_ops::func_5(1, 63).unwrap(), i64::MIN);
    assert_eq!(
        i64_ops::func_5(0x0123456789ABCDEFu64 as i64, 4).unwrap(),
        0x123456789ABCDEF0u64 as i64
    );
}

#[test]
fn test_i64_rem_u() {
    // Unsigned remainder: treat both operands as unsigned
    assert_eq!(i64_ops::func_6(17, 5).unwrap(), 2);
    // -1i64 as u64 = u64::MAX; u64::MAX % 3 = 0
    assert_eq!(i64_ops::func_6(-1, 3).unwrap(), 0);
}

#[test]
fn test_i64_rem_u_trap_zero() {
    assert!(i64_ops::func_6(10, 0).is_err());
}

// === f64 operations ===

#[test]
fn test_f64_div() {
    assert_eq!(f64_ops::func_0(10.0, 3.0).unwrap(), 10.0 / 3.0);
    assert_eq!(f64_ops::func_0(1.0, 0.0).unwrap(), f64::INFINITY);
    assert_eq!(f64_ops::func_0(-1.0, 0.0).unwrap(), f64::NEG_INFINITY);
}

#[test]
fn test_f64_min() {
    assert_eq!(f64_ops::func_1(3.0, 7.0).unwrap(), 3.0);
    assert_eq!(f64_ops::func_1(-1.0, 1.0).unwrap(), -1.0);
}

#[test]
fn test_f64_lt() {
    assert_eq!(f64_ops::func_2(1.0, 2.0).unwrap(), 1);
    assert_eq!(f64_ops::func_2(2.0, 1.0).unwrap(), 0);
    assert_eq!(f64_ops::func_2(1.0, 1.0).unwrap(), 0);
}

#[test]
fn test_f64_sqrt() {
    assert_eq!(f64_ops::func_3(4.0).unwrap(), 2.0);
    assert_eq!(f64_ops::func_3(9.0).unwrap(), 3.0);
    assert!(f64_ops::func_3(-1.0).unwrap().is_nan());
}

#[test]
fn test_f64_floor() {
    assert_eq!(f64_ops::func_4(3.7).unwrap(), 3.0);
    assert_eq!(f64_ops::func_4(-3.2).unwrap(), -4.0);
    assert_eq!(f64_ops::func_4(5.0).unwrap(), 5.0);
}

#[test]
fn test_f64_ceil() {
    assert_eq!(f64_ops::func_5(3.2).unwrap(), 4.0);
    assert_eq!(f64_ops::func_5(-3.7).unwrap(), -3.0);
    assert_eq!(f64_ops::func_5(5.0).unwrap(), 5.0);
}

#[test]
fn test_f64_neg() {
    assert_eq!(f64_ops::func_6(3.0).unwrap(), -3.0);
    assert_eq!(f64_ops::func_6(-7.0).unwrap(), 7.0);
    assert_eq!(f64_ops::func_6(0.0).unwrap(), -0.0);
}

// === Conversion operations ===

#[test]
fn test_i32_wrap_i64() {
    assert_eq!(conversions::func_0(42).unwrap(), 42);
    assert_eq!(conversions::func_0(0x100000001).unwrap(), 1); // truncate high bits
    assert_eq!(conversions::func_0(-1).unwrap(), -1);
}

#[test]
fn test_i64_extend_i32_s() {
    assert_eq!(conversions::func_1(42).unwrap(), 42i64);
    assert_eq!(conversions::func_1(-1).unwrap(), -1i64); // sign-extends
    assert_eq!(conversions::func_1(i32::MIN).unwrap(), i32::MIN as i64);
}

#[test]
fn test_i64_extend_i32_u() {
    assert_eq!(conversions::func_2(42).unwrap(), 42i64);
    assert_eq!(conversions::func_2(-1).unwrap(), 0xFFFFFFFFi64); // zero-extends
    assert_eq!(conversions::func_2(i32::MIN).unwrap(), 0x80000000i64);
}

#[test]
fn test_f64_convert_i32_s() {
    assert_eq!(conversions::func_3(42).unwrap(), 42.0);
    assert_eq!(conversions::func_3(-1).unwrap(), -1.0);
    assert_eq!(conversions::func_3(0).unwrap(), 0.0);
}

#[test]
fn test_i32_trunc_f64_s() {
    assert_eq!(conversions::func_4(42.9).unwrap(), 42);
    assert_eq!(conversions::func_4(-3.7).unwrap(), -3);
    assert_eq!(conversions::func_4(0.0).unwrap(), 0);
}

#[test]
fn test_i32_trunc_f64_s_trap_nan() {
    assert!(conversions::func_4(f64::NAN).is_err());
}

#[test]
fn test_i32_trunc_f64_s_trap_overflow() {
    assert!(conversions::func_4(2147483648.0).is_err()); // > i32::MAX
    assert!(conversions::func_4(-2147483649.0).is_err()); // < i32::MIN
}

#[test]
fn test_f32_demote_f64() {
    assert_eq!(conversions::func_5(2.5).unwrap(), 2.5f32);
    assert_eq!(conversions::func_5(0.0).unwrap(), 0.0f32);
}

#[test]
fn test_f64_promote_f32() {
    assert_eq!(conversions::func_6(2.5f32).unwrap(), 2.5f64);
    assert_eq!(conversions::func_6(0.0f32).unwrap(), 0.0f64);
}

#[test]
fn test_i32_reinterpret_f32() {
    assert_eq!(
        conversions::func_7(1.0f32).unwrap(),
        1.0f32.to_bits() as i32
    );
    assert_eq!(conversions::func_7(0.0f32).unwrap(), 0);
    assert_eq!(conversions::func_7(-0.0f32).unwrap(), i32::MIN); // sign bit
}

#[test]
fn test_f32_reinterpret_i32() {
    assert_eq!(conversions::func_8(0).unwrap(), 0.0f32);
    assert_eq!(
        conversions::func_8(1.0f32.to_bits() as i32).unwrap(),
        1.0f32
    );
}
