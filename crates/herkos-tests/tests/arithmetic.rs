//! End-to-end tests for arithmetic operations.
//!
//! These tests verify that:
//! 1. The transpiler generates valid Rust code (compilation succeeds)
//! 2. The generated code has correct runtime behavior (execution succeeds)
//! 3. The semantics match WebAssembly's wrapping arithmetic

use herkos_tests::{
    abs, add, add_i64, const_i64, const_return, factorial, fibonacci, gcd, mul, nop, sub,
};

#[test]
fn test_add_correctness() {
    // Basic addition
    assert_eq!(add::func_0(2, 3).unwrap(), 5);
    assert_eq!(add::func_0(0, 0).unwrap(), 0);
    assert_eq!(add::func_0(-1, 1).unwrap(), 0);
    assert_eq!(add::func_0(100, -50).unwrap(), 50);
}

#[test]
fn test_add_wrapping() {
    // Wasm uses wrapping arithmetic (matches Rust's wrapping_add)
    assert_eq!(
        add::func_0(i32::MAX, 1).unwrap(),
        i32::MIN,
        "i32::MAX + 1 should wrap to i32::MIN"
    );

    assert_eq!(
        add::func_0(i32::MAX, i32::MAX).unwrap(),
        -2,
        "i32::MAX + i32::MAX should wrap"
    );
}

#[test]
fn test_sub_correctness() {
    assert_eq!(sub::func_0(10, 3).unwrap(), 7);
    assert_eq!(sub::func_0(0, 0).unwrap(), 0);
    assert_eq!(sub::func_0(5, 10).unwrap(), -5);
}

#[test]
fn test_sub_wrapping() {
    assert_eq!(
        sub::func_0(i32::MIN, 1).unwrap(),
        i32::MAX,
        "i32::MIN - 1 should wrap to i32::MAX"
    );
}

#[test]
fn test_mul_correctness() {
    assert_eq!(mul::func_0(6, 7).unwrap(), 42);
    assert_eq!(mul::func_0(0, 100).unwrap(), 0);
    assert_eq!(mul::func_0(-2, 3).unwrap(), -6);
    assert_eq!(mul::func_0(-2, -3).unwrap(), 6);
}

#[test]
fn test_mul_wrapping() {
    assert_eq!(
        mul::func_0(i32::MAX, 2).unwrap(),
        -2,
        "i32::MAX * 2 should wrap"
    );
}

#[test]
fn test_const_return() {
    // Function that returns a constant
    assert_eq!(const_return::func_0().unwrap(), 42);
}

#[test]
fn test_nop() {
    // Void function with no-op
    assert_eq!(nop::func_0().unwrap(), ());
}

// Comprehensive property: addition commutes
#[test]
fn test_add_commutative() {
    let test_cases = [
        (0, 0),
        (1, 2),
        (10, 20),
        (-5, 5),
        (i32::MAX, 1),
        (i32::MIN, -1),
    ];

    for (a, b) in test_cases {
        assert_eq!(
            add::func_0(a, b).unwrap(),
            add::func_0(b, a).unwrap(),
            "addition should be commutative: {} + {} != {} + {}",
            a,
            b,
            b,
            a
        );
    }
}

// i64 type safety: generated code declares i64 variables correctly
#[test]
fn test_i64_add_correctness() {
    assert_eq!(add_i64::func_0(2, 3).unwrap(), 5i64);
    assert_eq!(add_i64::func_0(0, 0).unwrap(), 0i64);
    assert_eq!(add_i64::func_0(-1, 1).unwrap(), 0i64);
}

#[test]
fn test_i64_add_large_values() {
    // Values that exceed i32 range — proves i64 typing is correct
    let large: i64 = 3_000_000_000;
    assert_eq!(
        add_i64::func_0(large, large).unwrap(),
        6_000_000_000i64,
        "i64 add with values > i32::MAX should work"
    );
}

#[test]
fn test_i64_add_wrapping() {
    assert_eq!(
        add_i64::func_0(i64::MAX, 1).unwrap(),
        i64::MIN,
        "i64::MAX + 1 should wrap to i64::MIN"
    );
}

#[test]
fn test_i64_const() {
    assert_eq!(
        const_i64::func_0().unwrap(),
        9_999_999_999i64,
        "i64 constant should preserve value beyond i32 range"
    );
}

// Fibonacci: exercises locals, loops, comparisons, and arithmetic together
#[test]
fn test_fibonacci() {
    let expected: &[(i32, i32)] = &[
        (0, 0),
        (1, 1),
        (2, 1),
        (3, 2),
        (4, 3),
        (5, 5),
        (6, 8),
        (7, 13),
        (10, 55),
        (20, 6765),
    ];

    for &(n, fib_n) in expected {
        assert_eq!(
            fibonacci::func_0(n).unwrap(),
            fib_n,
            "fib({}) should be {}",
            n,
            fib_n
        );
    }
}

// GCD: Euclidean algorithm
#[test]
fn test_gcd() {
    let cases: &[(i32, i32, i32)] = &[
        (12, 8, 4),
        (54, 24, 6),
        (7, 13, 1),
        (100, 100, 100),
        (0, 5, 5),
        (5, 0, 5),
        (1, 1, 1),
        (48, 18, 6),
    ];

    for &(a, b, expected) in cases {
        assert_eq!(
            gcd::func_0(a, b).unwrap(),
            expected,
            "gcd({}, {}) should be {}",
            a,
            b,
            expected
        );
    }
}

#[test]
fn test_gcd_commutative() {
    let pairs = [(12, 8), (54, 24), (7, 13), (100, 75)];
    for (a, b) in pairs {
        assert_eq!(
            gcd::func_0(a, b).unwrap(),
            gcd::func_0(b, a).unwrap(),
            "gcd should be commutative: gcd({}, {}) != gcd({}, {})",
            a,
            b,
            b,
            a
        );
    }
}

// Factorial: iterative, uses wrapping multiplication
#[test]
fn test_factorial() {
    let cases: &[(i32, i32)] = &[
        (0, 1),
        (1, 1),
        (2, 2),
        (3, 6),
        (4, 24),
        (5, 120),
        (6, 720),
        (7, 5040),
        (10, 3628800),
        (12, 479001600),
    ];

    for &(n, expected) in cases {
        assert_eq!(
            factorial::func_0(n).unwrap(),
            expected,
            "{}! should be {}",
            n,
            expected
        );
    }
}

#[test]
fn test_factorial_wrapping() {
    // 13! = 6227020800 which overflows i32, so wrapping arithmetic kicks in
    let result = factorial::func_0(13).unwrap();
    let expected = 1932053504i32; // 6227020800 as wrapping i32
    assert_eq!(result, expected, "13! should wrap to {}", expected);
}

// Absolute value
#[test]
fn test_abs() {
    assert_eq!(abs::func_0(42).unwrap(), 42);
    assert_eq!(abs::func_0(-42).unwrap(), 42);
    assert_eq!(abs::func_0(0).unwrap(), 0);
    assert_eq!(abs::func_0(1).unwrap(), 1);
    assert_eq!(abs::func_0(-1).unwrap(), 1);
    assert_eq!(abs::func_0(i32::MAX).unwrap(), i32::MAX);
}

#[test]
fn test_abs_min_wraps() {
    // abs(i32::MIN) overflows — Wasm wrapping gives i32::MIN back
    // because 0 - i32::MIN wraps to i32::MIN
    assert_eq!(
        abs::func_0(i32::MIN).unwrap(),
        i32::MIN,
        "abs(i32::MIN) should wrap to i32::MIN"
    );
}

// Property: matches Rust's wrapping_add
#[test]
fn test_add_matches_rust_wrapping() {
    let test_cases = [
        (0, 0),
        (1, 2),
        (i32::MAX, 1),
        (i32::MIN, -1),
        (i32::MAX, i32::MAX),
        (i32::MIN, i32::MIN),
    ];

    for (a, b) in test_cases {
        let wasm_result = add::func_0(a, b).unwrap();
        let rust_result = a.wrapping_add(b);
        assert_eq!(
            wasm_result, rust_result,
            "Wasm add({}, {}) = {} but Rust wrapping_add = {}",
            a, b, wasm_result, rust_result
        );
    }
}
