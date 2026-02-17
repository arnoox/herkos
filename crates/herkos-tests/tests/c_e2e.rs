//! End-to-end tests: C → Wasm → Rust round-trip.
//!
//! These tests verify that:
//! 1. Freestanding C compiled to Wasm via clang produces valid Wasm
//! 2. herkos transpiles clang-generated Wasm correctly
//! 3. The transpiled Rust code compiles and produces correct results
//! 4. Results match C semantics (wrapping arithmetic, signed division, etc.)
//!
//! If clang is not installed, build.rs will skip generating this module
//! (with a cargo:warning). Install with: apt-get install clang lld

use herkos_tests::c_e2e_arith;

fn new_module() -> c_e2e_arith::WasmModule {
    c_e2e_arith::new().expect("module instantiation should succeed")
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
    assert_eq!(m.add_i32(i32::MAX, 1).unwrap(), i32::MIN);
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
    assert_eq!(m.sub_i32(i32::MIN, 1).unwrap(), i32::MAX);
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
    assert_eq!(m.mul_i32(i32::MAX, 2).unwrap(), i32::MAX.wrapping_mul(2));
}

#[test]
fn test_negate() {
    let mut m = new_module();
    assert_eq!(m.negate(42).unwrap(), -42);
    assert_eq!(m.negate(-42).unwrap(), 42);
    assert_eq!(m.negate(0).unwrap(), 0);
    // C negate of INT_MIN wraps (undefined behavior in C, but Wasm defines it)
    assert_eq!(m.negate(i32::MIN).unwrap(), i32::MIN);
}

#[test]
fn test_const_42() {
    let mut m = new_module();
    assert_eq!(m.const_42().unwrap(), 42);
}

// ── Division and remainder ──

#[test]
fn test_div_s() {
    let mut m = new_module();
    assert_eq!(m.div_s(10, 3).unwrap(), 3);
    assert_eq!(m.div_s(-10, 3).unwrap(), -3);
    assert_eq!(m.div_s(10, -3).unwrap(), -3);
    assert_eq!(m.div_s(-10, -3).unwrap(), 3);
    assert_eq!(m.div_s(0, 5).unwrap(), 0);
    assert_eq!(m.div_s(42, 1).unwrap(), 42);
}

#[test]
fn test_div_s_traps_on_zero() {
    let mut m = new_module();
    assert!(m.div_s(1, 0).is_err(), "division by zero should trap");
}

#[test]
fn test_rem_s() {
    let mut m = new_module();
    assert_eq!(m.rem_s(10, 3).unwrap(), 1);
    assert_eq!(m.rem_s(-10, 3).unwrap(), -1);
    assert_eq!(m.rem_s(10, -3).unwrap(), 1);
    assert_eq!(m.rem_s(0, 5).unwrap(), 0);
    assert_eq!(m.rem_s(7, 7).unwrap(), 0);
}

#[test]
fn test_rem_s_traps_on_zero() {
    let mut m = new_module();
    assert!(m.rem_s(1, 0).is_err(), "remainder by zero should trap");
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
    // Unsigned shift right of -1: high bit fills with 0
    assert_eq!(
        m.shift_right_u(-1, 1).unwrap(),
        ((-1i32 as u32) >> 1) as i32,
    );
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
    assert_eq!(m.add_i64(large, large).unwrap(), 6_000_000_000i64);
}

// ── Loops ──

#[test]
fn test_factorial() {
    let mut m = new_module();
    assert_eq!(m.factorial(0).unwrap(), 1);
    assert_eq!(m.factorial(1).unwrap(), 1);
    assert_eq!(m.factorial(5).unwrap(), 120);
    assert_eq!(m.factorial(10).unwrap(), 3_628_800);
}

#[test]
fn test_sum_1_to_n() {
    let mut m = new_module();
    assert_eq!(m.sum_1_to_n(0).unwrap(), 0);
    assert_eq!(m.sum_1_to_n(1).unwrap(), 1);
    assert_eq!(m.sum_1_to_n(10).unwrap(), 55);
    assert_eq!(m.sum_1_to_n(100).unwrap(), 5050);
}

// ── Recursive / iterative algorithms ──

#[test]
fn test_gcd() {
    let mut m = new_module();
    assert_eq!(m.gcd(12, 8).unwrap(), 4);
    assert_eq!(m.gcd(17, 13).unwrap(), 1, "coprime numbers");
    assert_eq!(m.gcd(100, 25).unwrap(), 25);
    assert_eq!(m.gcd(7, 0).unwrap(), 7, "gcd(n, 0) = n");
    assert_eq!(m.gcd(0, 7).unwrap(), 7, "gcd(0, n) = n");
}

// ── Composed operations ──

#[test]
fn test_diff_of_squares() {
    let mut m = new_module();
    // (5+3) * (5-3) = 8 * 2 = 16
    assert_eq!(m.diff_of_squares(5, 3).unwrap(), 16);
    // (10+1) * (10-1) = 11 * 9 = 99
    assert_eq!(m.diff_of_squares(10, 1).unwrap(), 99);
    // (a+0)(a-0) = a^2
    assert_eq!(m.diff_of_squares(7, 0).unwrap(), 49);
}

// ── Cross-validation: transpiled output matches native computation ──

#[test]
fn test_matches_native_c_semantics() {
    let mut m = new_module();

    let test_pairs: &[(i32, i32)] = &[
        (0, 0),
        (1, 2),
        (10, 20),
        (-5, 5),
        (i32::MAX, 1),
        (i32::MIN, -1),
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

// ── Commutativity ──

#[test]
fn test_commutative_ops() {
    let mut m = new_module();

    let pairs: &[(i32, i32)] = &[(3, 7), (0, 0), (-5, 5), (i32::MAX, 1)];

    for &(a, b) in pairs {
        assert_eq!(m.add_i32(a, b).unwrap(), m.add_i32(b, a).unwrap());
        assert_eq!(m.mul_i32(a, b).unwrap(), m.mul_i32(b, a).unwrap());
        assert_eq!(m.bitwise_and(a, b).unwrap(), m.bitwise_and(b, a).unwrap());
        assert_eq!(m.bitwise_or(a, b).unwrap(), m.bitwise_or(b, a).unwrap());
        assert_eq!(m.bitwise_xor(a, b).unwrap(), m.bitwise_xor(b, a).unwrap());
    }
}

// ── Division edge cases ──

#[test]
fn test_division_edge_cases() {
    let mut m = new_module();
    // Verify various division results follow truncation-toward-zero (C99+)
    assert_eq!(m.div_s(7, 2).unwrap(), 3);
    assert_eq!(m.div_s(-7, 2).unwrap(), -3);
    assert_eq!(m.div_s(7, -2).unwrap(), -3);
    assert_eq!(m.div_s(-7, -2).unwrap(), 3);

    // Remainder follows: a == (a/b)*b + (a%b)
    for &(a, b) in &[(10, 3), (-10, 3), (10, -3), (-10, -3), (7, 2), (-7, 2)] {
        let q = m.div_s(a, b).unwrap();
        let r = m.rem_s(a, b).unwrap();
        assert_eq!(q * b + r, a, "invariant: ({a}/{b})*{b} + ({a}%{b}) == {a}");
    }
}
