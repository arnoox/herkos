//! Runtime tests for direct function calls.
//!
//! These tests verify that:
//! 1. Direct calls correctly forward arguments and return values
//! 2. Recursive calls work (fibonacci)
//! 3. Calls properly pass through memory and globals parameters

use herkos_tests::{call_helper, call_i64, call_with_globals, call_with_memory, recursive_fib};

// ── Simple helper call ──

#[test]
fn test_call_helper_basic() {
    // func_1 calls func_0 (which does i32.add)
    let mut module = call_helper::new().unwrap();
    assert_eq!(module.func_1(3, 4).unwrap(), 7);
    assert_eq!(module.func_1(0, 0).unwrap(), 0);
    assert_eq!(module.func_1(-1, 1).unwrap(), 0);
    assert_eq!(module.func_1(100, 200).unwrap(), 300);
}

#[test]
fn test_call_helper_wrapping() {
    // Wrapping arithmetic still works through the call
    let mut module = call_helper::new().unwrap();
    assert_eq!(module.func_1(i32::MAX, 1).unwrap(), i32::MIN);
}

#[test]
fn test_call_helper_direct_vs_indirect() {
    // Calling func_0 directly should give same result as calling via func_1
    let mut module = call_helper::new().unwrap();
    for (a, b) in [(1, 2), (10, 20), (-5, 5), (i32::MAX, 1)] {
        assert_eq!(
            module.func_0(a, b).unwrap(),
            module.func_1(a, b).unwrap(),
            "direct call and call-via-helper should match for ({}, {})",
            a,
            b
        );
    }
}

// ── Recursive fibonacci ──

#[test]
fn test_recursive_fib() {
    let mut module = recursive_fib::new().unwrap();
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
        (15, 610),
        (20, 6765),
    ];

    for &(n, fib_n) in expected {
        assert_eq!(
            module.func_0(n).unwrap(),
            fib_n,
            "fib({}) should be {}",
            n,
            fib_n
        );
    }
}

#[test]
fn test_recursive_fib_matches_iterative() {
    // Compare recursive fib against the iterative fibonacci from the existing tests
    use herkos_tests::fibonacci;

    let mut module = recursive_fib::new().unwrap();
    let mut fib_module = fibonacci::new().unwrap();

    for n in 0..=20 {
        assert_eq!(
            module.func_0(n).unwrap(),
            fib_module.func_0(n).unwrap(),
            "recursive and iterative fib should agree for n={}",
            n
        );
    }
}

// ── Call with memory ──

#[test]
fn test_call_with_memory_store_via_call() {
    let mut module = call_with_memory::new().unwrap();

    // func_1 (store_via_call) calls func_0 (store_value) internally
    module.func_1(0, 42).unwrap();

    // Verify the value was stored by loading directly
    let value = module.func_2(0).unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_call_with_memory_multiple_stores() {
    let mut module = call_with_memory::new().unwrap();

    // Store multiple values via the call-forwarding function
    module.func_1(0, 10).unwrap();
    module.func_1(4, 20).unwrap();
    module.func_1(8, 30).unwrap();

    assert_eq!(module.func_2(0).unwrap(), 10);
    assert_eq!(module.func_2(4).unwrap(), 20);
    assert_eq!(module.func_2(8).unwrap(), 30);
}

// ── Call with i64 return type ──

#[test]
fn test_call_i64_basic() {
    // func_1 calls func_0 (i64.add)
    let mut module = call_i64::new().unwrap();
    assert_eq!(module.func_1(10, 20).unwrap(), 30i64);
    assert_eq!(module.func_1(0, 0).unwrap(), 0i64);
    assert_eq!(module.func_1(-1, 1).unwrap(), 0i64);
}

#[test]
fn test_call_i64_large_values() {
    // Values beyond i32 range prove the return type is correctly i64
    let mut module = call_i64::new().unwrap();
    let a: i64 = 3_000_000_000;
    let b: i64 = 4_000_000_000;
    assert_eq!(module.func_1(a, b).unwrap(), 7_000_000_000i64);
}

#[test]
fn test_call_i64_wrapping() {
    let mut module = call_i64::new().unwrap();
    assert_eq!(module.func_1(i64::MAX, 1).unwrap(), i64::MIN);
}

// ── Call with globals (wrapper mode) ──

#[test]
fn test_call_with_globals_set_and_get() {
    let mut module = call_with_globals::new().unwrap();

    // set_and_get calls set_global then get_global internally
    let result = module.set_and_get(42).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_call_with_globals_multiple() {
    let mut module = call_with_globals::new().unwrap();

    assert_eq!(module.set_and_get(10).unwrap(), 10);
    assert_eq!(module.get().unwrap(), 10);

    assert_eq!(module.set_and_get(99).unwrap(), 99);
    assert_eq!(module.get().unwrap(), 99);
}

#[test]
fn test_call_with_globals_isolation() {
    let mut m1 = call_with_globals::new().unwrap();
    let mut m2 = call_with_globals::new().unwrap();

    m1.set_and_get(100).unwrap();

    // m2 should be unaffected
    assert_eq!(m2.get().unwrap(), 0);
    assert_eq!(m1.get().unwrap(), 100);
}
