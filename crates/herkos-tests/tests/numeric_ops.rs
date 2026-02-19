//! Runtime tests for Milestone 5: complete numeric operations.
//!
//! Tests i64 operations, f64 operations, and conversion operations.

use herkos_tests::{conversions, f64_ops, i64_ops};

// === i64 operations ===

#[test]
fn test_i64_div_s() {
    let mut i64_ops_mod = i64_ops::new().unwrap();
    assert_eq!(i64_ops_mod.div_s(100, 7).unwrap(), 14);
    assert_eq!(i64_ops_mod.div_s(-100, 7).unwrap(), -14);
    assert_eq!(i64_ops_mod.div_s(0, 5).unwrap(), 0);
}

#[test]
fn test_i64_div_s_trap_zero() {
    let mut i64_ops_mod = i64_ops::new().unwrap();
    assert!(i64_ops_mod.div_s(10, 0).is_err());
}

#[test]
fn test_i64_bitand() {
    let mut i64_ops_mod = i64_ops::new().unwrap();
    assert_eq!(i64_ops_mod.bitand(0xFF00, 0x0FF0).unwrap(), 0x0F00);
    assert_eq!(i64_ops_mod.bitand(-1, 0x1234).unwrap(), 0x1234);
}

#[test]
fn test_i64_shl() {
    let mut i64_ops_mod = i64_ops::new().unwrap();
    assert_eq!(i64_ops_mod.shl(1, 10).unwrap(), 1024);
    assert_eq!(i64_ops_mod.shl(1, 63).unwrap(), i64::MIN);
    // Shift amount is masked to 63
    assert_eq!(i64_ops_mod.shl(1, 64).unwrap(), 1);
}

#[test]
fn test_i64_lt_s() {
    let mut i64_ops_mod = i64_ops::new().unwrap();
    assert_eq!(i64_ops_mod.lt_s(5, 10).unwrap(), 1);
    assert_eq!(i64_ops_mod.lt_s(10, 5).unwrap(), 0);
    assert_eq!(i64_ops_mod.lt_s(5, 5).unwrap(), 0);
    assert_eq!(i64_ops_mod.lt_s(-1, 0).unwrap(), 1);
}

#[test]
fn test_i64_clz() {
    let mut i64_ops_mod = i64_ops::new().unwrap();
    assert_eq!(i64_ops_mod.clz(1).unwrap(), 63);
    assert_eq!(i64_ops_mod.clz(0).unwrap(), 64);
    assert_eq!(i64_ops_mod.clz(-1).unwrap(), 0);
}

#[test]
fn test_i64_rotl() {
    let mut i64_ops_mod = i64_ops::new().unwrap();
    assert_eq!(i64_ops_mod.rotl(1, 1).unwrap(), 2);
    assert_eq!(i64_ops_mod.rotl(1, 63).unwrap(), i64::MIN);
    assert_eq!(
        i64_ops_mod.rotl(0x0123456789ABCDEFu64 as i64, 4).unwrap(),
        0x123456789ABCDEF0u64 as i64
    );
}

#[test]
fn test_i64_rem_u() {
    let mut i64_ops_mod = i64_ops::new().unwrap();
    // Unsigned remainder: treat both operands as unsigned
    assert_eq!(i64_ops_mod.rem_u(17, 5).unwrap(), 2);
    // -1i64 as u64 = u64::MAX; u64::MAX % 3 = 0
    assert_eq!(i64_ops_mod.rem_u(-1, 3).unwrap(), 0);
}

#[test]
fn test_i64_rem_u_trap_zero() {
    let mut i64_ops_mod = i64_ops::new().unwrap();
    assert!(i64_ops_mod.rem_u(10, 0).is_err());
}

// === f64 operations ===

#[test]
fn test_f64_div() {
    let mut f64_ops_mod = f64_ops::new().unwrap();
    assert_eq!(f64_ops_mod.div(10.0, 3.0).unwrap(), 10.0 / 3.0);
    assert_eq!(f64_ops_mod.div(1.0, 0.0).unwrap(), f64::INFINITY);
    assert_eq!(f64_ops_mod.div(-1.0, 0.0).unwrap(), f64::NEG_INFINITY);
}

#[test]
fn test_f64_min() {
    let mut f64_ops_mod = f64_ops::new().unwrap();
    assert_eq!(f64_ops_mod.min(3.0, 7.0).unwrap(), 3.0);
    assert_eq!(f64_ops_mod.min(-1.0, 1.0).unwrap(), -1.0);
}

#[test]
fn test_f64_lt() {
    let mut f64_ops_mod = f64_ops::new().unwrap();
    assert_eq!(f64_ops_mod.lt(1.0, 2.0).unwrap(), 1);
    assert_eq!(f64_ops_mod.lt(2.0, 1.0).unwrap(), 0);
    assert_eq!(f64_ops_mod.lt(1.0, 1.0).unwrap(), 0);
}

#[test]
fn test_f64_sqrt() {
    let mut f64_ops_mod = f64_ops::new().unwrap();
    assert_eq!(f64_ops_mod.sqrt(4.0).unwrap(), 2.0);
    assert_eq!(f64_ops_mod.sqrt(9.0).unwrap(), 3.0);
    assert!(f64_ops_mod.sqrt(-1.0).unwrap().is_nan());
}

#[test]
fn test_f64_floor() {
    let mut f64_ops_mod = f64_ops::new().unwrap();
    assert_eq!(f64_ops_mod.floor(3.7).unwrap(), 3.0);
    assert_eq!(f64_ops_mod.floor(-3.2).unwrap(), -4.0);
    assert_eq!(f64_ops_mod.floor(5.0).unwrap(), 5.0);
}

#[test]
fn test_f64_ceil() {
    let mut f64_ops_mod = f64_ops::new().unwrap();
    assert_eq!(f64_ops_mod.ceil(3.2).unwrap(), 4.0);
    assert_eq!(f64_ops_mod.ceil(-3.7).unwrap(), -3.0);
    assert_eq!(f64_ops_mod.ceil(5.0).unwrap(), 5.0);
}

#[test]
fn test_f64_neg() {
    let mut f64_ops_mod = f64_ops::new().unwrap();
    assert_eq!(f64_ops_mod.neg(3.0).unwrap(), -3.0);
    assert_eq!(f64_ops_mod.neg(-7.0).unwrap(), 7.0);
    assert_eq!(f64_ops_mod.neg(0.0).unwrap(), -0.0);
}

// === Conversion operations ===

#[test]
fn test_i32_wrap_i64() {
    let mut conversions_mod = conversions::new().unwrap();
    assert_eq!(conversions_mod.wrap_i64(42).unwrap(), 42);
    assert_eq!(conversions_mod.wrap_i64(0x100000001).unwrap(), 1); // truncate high bits
    assert_eq!(conversions_mod.wrap_i64(-1).unwrap(), -1);
}

#[test]
fn test_i64_extend_i32_s() {
    let mut conversions_mod = conversions::new().unwrap();
    assert_eq!(conversions_mod.extend_i32_s(42).unwrap(), 42i64);
    assert_eq!(conversions_mod.extend_i32_s(-1).unwrap(), -1i64); // sign-extends
    assert_eq!(
        conversions_mod.extend_i32_s(i32::MIN).unwrap(),
        i32::MIN as i64
    );
}

#[test]
fn test_i64_extend_i32_u() {
    let mut conversions_mod = conversions::new().unwrap();
    assert_eq!(conversions_mod.extend_i32_u(42).unwrap(), 42i64);
    assert_eq!(conversions_mod.extend_i32_u(-1).unwrap(), 0xFFFFFFFFi64); // zero-extends
    assert_eq!(
        conversions_mod.extend_i32_u(i32::MIN).unwrap(),
        0x80000000i64
    );
}

#[test]
fn test_f64_convert_i32_s() {
    let mut conversions_mod = conversions::new().unwrap();
    assert_eq!(conversions_mod.convert_i32_s(42).unwrap(), 42.0);
    assert_eq!(conversions_mod.convert_i32_s(-1).unwrap(), -1.0);
    assert_eq!(conversions_mod.convert_i32_s(0).unwrap(), 0.0);
}

#[test]
fn test_i32_trunc_f64_s() {
    let mut conversions_mod = conversions::new().unwrap();
    assert_eq!(conversions_mod.trunc_f64_s(42.9).unwrap(), 42);
    assert_eq!(conversions_mod.trunc_f64_s(-3.7).unwrap(), -3);
    assert_eq!(conversions_mod.trunc_f64_s(0.0).unwrap(), 0);
}

#[test]
fn test_i32_trunc_f64_s_trap_nan() {
    let mut conversions_mod = conversions::new().unwrap();
    assert!(conversions_mod.trunc_f64_s(f64::NAN).is_err());
}

#[test]
fn test_i32_trunc_f64_s_trap_overflow() {
    let mut conversions_mod = conversions::new().unwrap();
    assert!(conversions_mod.trunc_f64_s(2147483648.0).is_err()); // > i32::MAX
    assert!(conversions_mod.trunc_f64_s(-2147483649.0).is_err()); // < i32::MIN
}

#[test]
fn test_f32_demote_f64() {
    let mut conversions_mod = conversions::new().unwrap();
    assert_eq!(conversions_mod.demote_f64(2.5).unwrap(), 2.5f32);
    assert_eq!(conversions_mod.demote_f64(0.0).unwrap(), 0.0f32);
}

#[test]
fn test_f64_promote_f32() {
    let mut conversions_mod = conversions::new().unwrap();
    assert_eq!(conversions_mod.promote_f32(2.5f32).unwrap(), 2.5f64);
    assert_eq!(conversions_mod.promote_f32(0.0f32).unwrap(), 0.0f64);
}

#[test]
fn test_i32_reinterpret_f32() {
    let mut conversions_mod = conversions::new().unwrap();
    assert_eq!(
        conversions_mod.reinterpret_f32(1.0f32).unwrap(),
        1.0f32.to_bits() as i32
    );
    assert_eq!(conversions_mod.reinterpret_f32(0.0f32).unwrap(), 0);
    assert_eq!(conversions_mod.reinterpret_f32(-0.0f32).unwrap(), i32::MIN); // sign bit
}

#[test]
fn test_f32_reinterpret_i32() {
    let mut conversions_mod = conversions::new().unwrap();
    assert_eq!(conversions_mod.reinterpret_i32(0).unwrap(), 0.0f32);
    assert_eq!(
        conversions_mod
            .reinterpret_i32(1.0f32.to_bits() as i32)
            .unwrap(),
        1.0f32
    );
}
