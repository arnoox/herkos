//! End-to-end tests: Rust → Wasm → Rust round-trip.
//!
//! These tests verify that:
//! 1. Rust source compiled to Wasm via rustc produces valid Wasm
//! 2. herkos transpiles rustc-generated Wasm correctly
//! 3. The transpiled Rust code compiles and produces correct results
//! 4. Results match direct Rust evaluation (the "ground truth")
//!
//! If the wasm32-unknown-unknown target is not installed, build.rs
//! will skip generating this module (with a cargo:warning). Install with:
//!   rustup target add wasm32-unknown-unknown

use herkos_tests::rust_e2e_arith;

fn new_module() -> rust_e2e_arith::WasmModule {
    rust_e2e_arith::new().expect("module instantiation should succeed")
}

// ── i32 arithmetic ──

#[test]
fn test_add_i32() {
    let mut m = new_module();
    assert_eq!(m.add_i32(2, 3).unwrap(), 5);
    assert_eq!(m.add_i32(0, 0).unwrap(), 0);
    assert_eq!(m.add_i32(-1, 1).unwrap(), 0);
    assert_eq!(m.add_i32(100, -50).unwrap(), 50);
}

#[test]
fn test_add_i32_wrapping() {
    let mut m = new_module();
    assert_eq!(m.add_i32(i32::MAX, 1).unwrap(), i32::MAX.wrapping_add(1),);
}

#[test]
fn test_sub_i32() {
    let mut m = new_module();
    assert_eq!(m.sub_i32(10, 3).unwrap(), 7);
    assert_eq!(m.sub_i32(0, 0).unwrap(), 0);
    assert_eq!(m.sub_i32(3, 10).unwrap(), -7);
}

#[test]
fn test_sub_i32_wrapping() {
    let mut m = new_module();
    assert_eq!(m.sub_i32(i32::MIN, 1).unwrap(), i32::MIN.wrapping_sub(1),);
}

#[test]
fn test_mul_i32() {
    let mut m = new_module();
    assert_eq!(m.mul_i32(6, 7).unwrap(), 42);
    assert_eq!(m.mul_i32(0, 999).unwrap(), 0);
    assert_eq!(m.mul_i32(-3, 4).unwrap(), -12);
}

#[test]
fn test_mul_i32_wrapping() {
    let mut m = new_module();
    assert_eq!(m.mul_i32(i32::MAX, 2).unwrap(), i32::MAX.wrapping_mul(2),);
}

// ── i64 arithmetic ──

#[test]
fn test_add_i64() {
    let mut m = new_module();
    assert_eq!(m.add_i64(10, 20).unwrap(), 30i64);
    assert_eq!(m.add_i64(0, 0).unwrap(), 0i64);
}

#[test]
fn test_add_i64_large() {
    let mut m = new_module();
    let large: i64 = 3_000_000_000;
    assert_eq!(
        m.add_i64(large, large).unwrap(),
        6_000_000_000i64,
        "i64 values beyond i32 range should work"
    );
}

#[test]
fn test_add_i64_wrapping() {
    let mut m = new_module();
    assert_eq!(m.add_i64(i64::MAX, 1).unwrap(), i64::MAX.wrapping_add(1),);
}

// ── Bitwise operations ──

#[test]
fn test_bitwise_and() {
    let mut m = new_module();
    assert_eq!(m.bitwise_and(0xFF, 0x0F).unwrap(), 0x0F);
    assert_eq!(m.bitwise_and(0, 0xFFFF).unwrap(), 0);
    assert_eq!(m.bitwise_and(-1, 42).unwrap(), 42);
}

#[test]
fn test_bitwise_or() {
    let mut m = new_module();
    assert_eq!(m.bitwise_or(0xF0, 0x0F).unwrap(), 0xFF);
    assert_eq!(m.bitwise_or(0, 0).unwrap(), 0);
}

#[test]
fn test_bitwise_xor() {
    let mut m = new_module();
    assert_eq!(m.bitwise_xor(0xFF, 0xFF).unwrap(), 0);
    assert_eq!(m.bitwise_xor(0xFF, 0x00).unwrap(), 0xFF);
    assert_eq!(m.bitwise_xor(0xAA, 0x55).unwrap(), 0xFF);
}

// ── Shifts ──

#[test]
fn test_shift_left() {
    let mut m = new_module();
    assert_eq!(m.shift_left(1, 0).unwrap(), 1);
    assert_eq!(m.shift_left(1, 4).unwrap(), 16);
    assert_eq!(m.shift_left(0xFF, 8).unwrap(), 0xFF00);
}

#[test]
fn test_shift_right_unsigned() {
    let mut m = new_module();
    assert_eq!(m.shift_right_u(16, 4).unwrap(), 1);
    assert_eq!(m.shift_right_u(0xFF00, 8).unwrap(), 0xFF);
    // Unsigned shift right of negative number: high bits fill with 0
    assert_eq!(
        m.shift_right_u(-1, 1).unwrap(),
        ((-1i32 as u32).wrapping_shr(1)) as i32,
    );
}

// ── Unary and constants ──

#[test]
fn test_negate() {
    let mut m = new_module();
    assert_eq!(m.negate(42).unwrap(), -42);
    assert_eq!(m.negate(-42).unwrap(), 42);
    assert_eq!(m.negate(0).unwrap(), 0);
}

#[test]
fn test_negate_wrapping() {
    let mut m = new_module();
    assert_eq!(
        m.negate(i32::MIN).unwrap(),
        i32::MIN.wrapping_neg(),
        "negate(i32::MIN) should wrap to i32::MIN"
    );
}

#[test]
fn test_const_42() {
    let mut m = new_module();
    assert_eq!(m.const_42().unwrap(), 42);
}

// ── Composed operations ──

#[test]
fn test_diff_of_squares() {
    let mut m = new_module();
    // (5+3) * (5-3) = 8 * 2 = 16
    assert_eq!(m.diff_of_squares(5, 3).unwrap(), 16);
    // (10+1) * (10-1) = 11 * 9 = 99
    assert_eq!(m.diff_of_squares(10, 1).unwrap(), 99);
    // a^2 - b^2 identity: (a+b)(a-b) = a^2 - b^2
    assert_eq!(m.diff_of_squares(7, 0).unwrap(), 49);
}

// ── Cross-validation: transpiled output matches native Rust ──

#[test]
fn test_matches_native_rust() {
    let mut m = new_module();

    let test_pairs: &[(i32, i32)] = &[
        (0, 0),
        (1, 2),
        (10, 20),
        (-5, 5),
        (i32::MAX, 1),
        (i32::MIN, -1),
        (i32::MAX, i32::MAX),
        (100, -100),
    ];

    for &(a, b) in test_pairs {
        assert_eq!(
            m.add_i32(a, b).unwrap(),
            a.wrapping_add(b),
            "add_i32({a}, {b})"
        );
        assert_eq!(
            m.sub_i32(a, b).unwrap(),
            a.wrapping_sub(b),
            "sub_i32({a}, {b})"
        );
        assert_eq!(
            m.mul_i32(a, b).unwrap(),
            a.wrapping_mul(b),
            "mul_i32({a}, {b})"
        );
        assert_eq!(m.bitwise_and(a, b).unwrap(), a & b, "bitwise_and({a}, {b})");
        assert_eq!(m.bitwise_or(a, b).unwrap(), a | b, "bitwise_or({a}, {b})");
        assert_eq!(m.bitwise_xor(a, b).unwrap(), a ^ b, "bitwise_xor({a}, {b})");
    }
}

// ── Commutativity property ──

#[test]
fn test_commutative_ops() {
    let mut m = new_module();

    let pairs: &[(i32, i32)] = &[(3, 7), (0, 0), (-5, 5), (i32::MAX, 1)];

    for &(a, b) in pairs {
        assert_eq!(
            m.add_i32(a, b).unwrap(),
            m.add_i32(b, a).unwrap(),
            "add commutative for ({a}, {b})"
        );
        assert_eq!(
            m.mul_i32(a, b).unwrap(),
            m.mul_i32(b, a).unwrap(),
            "mul commutative for ({a}, {b})"
        );
        assert_eq!(
            m.bitwise_and(a, b).unwrap(),
            m.bitwise_and(b, a).unwrap(),
            "and commutative for ({a}, {b})"
        );
        assert_eq!(
            m.bitwise_or(a, b).unwrap(),
            m.bitwise_or(b, a).unwrap(),
            "or commutative for ({a}, {b})"
        );
        assert_eq!(
            m.bitwise_xor(a, b).unwrap(),
            m.bitwise_xor(b, a).unwrap(),
            "xor commutative for ({a}, {b})"
        );
    }
}
