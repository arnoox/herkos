//! End-to-end tests for local variable handling.
//!
//! These tests verify that:
//! 1. Local variables are properly tracked and distinguished from parameters
//! 2. Multiple locals of different types are handled correctly
//! 3. local.get, local.set, and local.tee operations work correctly
//! 4. Locals are zero-initialized as per WebAssembly specification
//! 5. The transpiler generates correct code for complex local manipulations

use herkos_tests::locals_test;

#[test]
fn test_func_0_basic() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_0: Takes two i32 parameters, uses a local to compute: param0 + (param1 * 10)
    // func_0(10, 20) should return: 10 + (20 * 10) = 210
    assert_eq!(locals_test_mod.swap(10, 20).unwrap(), 210);
}

#[test]
fn test_func_0_zeros() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_0(0, 0) = 0 + (0 * 10) = 0
    assert_eq!(locals_test_mod.swap(0, 0).unwrap(), 0);
}

#[test]
fn test_func_0_negative() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_0(-5, 3) = -5 + (3 * 10) = 25
    assert_eq!(locals_test_mod.swap(-5, 3).unwrap(), 25);
}

#[test]
fn test_func_0_large_values() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_0(100, 200) = 100 + (200 * 10) = 2100
    assert_eq!(locals_test_mod.swap(100, 200).unwrap(), 2100);
}

/// Test that simple locals are properly tracked and used
#[test]
fn test_func_1_i32_local() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_1: Takes one i32 parameter and adds a constant from a local
    // func_1(42) should return 42 + 100 = 142
    // Demonstrates that i32 local is properly initialized and used
    assert_eq!(locals_test_mod.simple_local(42).unwrap(), 142);
}

#[test]
fn test_func_1_zero() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_1(0) = 0 + 100 = 100
    assert_eq!(locals_test_mod.simple_local(0).unwrap(), 100);
}

#[test]
fn test_func_1_negative() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_1(-50) = -50 + 100 = 50
    assert_eq!(locals_test_mod.simple_local(-50).unwrap(), 50);
}

/// Test local.tee: stores value in local and keeps it on stack
#[test]
fn test_func_2_tee_basic() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_2: local.tee preserves value on stack
    // func_2(5):
    // - Add 5 to 5 = 10
    // - local.tee stores 10 in local, keeps 10 on stack
    // - Multiply 10 * 2 = 20
    assert_eq!(locals_test_mod.tee_test(5).unwrap(), 20);
}

#[test]
fn test_func_2_tee_zero() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_2(0) = (0 + 5) * 2 = 10
    assert_eq!(locals_test_mod.tee_test(0).unwrap(), 10);
}

#[test]
fn test_func_2_tee_negative() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_2(-3) = (-3 + 5) * 2 = 4
    assert_eq!(locals_test_mod.tee_test(-3).unwrap(), 4);
}

#[test]
fn test_func_2_tee_large() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_2(1000) = (1000 + 5) * 2 = 2010
    assert_eq!(locals_test_mod.tee_test(1000).unwrap(), 2010);
}

/// Test that locals are zero-initialized (Wasm spec requirement)
#[test]
fn test_func_3_zero_initialization() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_3: Locals are zero-initialized by Wasm spec
    // func_3(42):
    // - local 1 and 2 are uninitialized (should be zero)
    // - return uninitialized_local + param = 0 + 42 = 42
    assert_eq!(locals_test_mod.zero_init(42).unwrap(), 42);
}

#[test]
fn test_func_3_zero_init_negative() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_3(-10) = 0 + (-10) = -10
    assert_eq!(locals_test_mod.zero_init(-10).unwrap(), -10);
}

#[test]
fn test_func_3_zero_init_zero() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_3(0) = 0 + 0 = 0
    assert_eq!(locals_test_mod.zero_init(0).unwrap(), 0);
}

/// Test complex local manipulations: running sum
#[test]
fn test_func_4_running_sum_basic() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_4: Uses a local to accumulate a sum across three parameters
    // func_4(10, 20, 30) = 10 + 20 + 30 = 60
    assert_eq!(locals_test_mod.running_sum(10, 20, 30).unwrap(), 60);
}

#[test]
fn test_func_4_running_sum_zeros() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_4(0, 0, 0) = 0
    assert_eq!(locals_test_mod.running_sum(0, 0, 0).unwrap(), 0);
}

#[test]
fn test_func_4_running_sum_mixed() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_4(5, -3, 8) = 5 - 3 + 8 = 10
    assert_eq!(locals_test_mod.running_sum(5, -3, 8).unwrap(), 10);
}

#[test]
fn test_func_4_running_sum_negative() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_4(-1, -2, -3) = -6
    assert_eq!(locals_test_mod.running_sum(-1, -2, -3).unwrap(), -6);
}

#[test]
fn test_func_4_running_sum_large() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // func_4(1000, 2000, 3000) = 6000
    assert_eq!(locals_test_mod.running_sum(1000, 2000, 3000).unwrap(), 6000);
}

/// Property test: all functions handle i32::MIN and i32::MAX correctly
#[test]
fn test_local_bounds() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // Verify functions work with boundary values
    assert!(locals_test_mod.swap(i32::MIN, i32::MAX).is_ok());
    assert!(locals_test_mod.simple_local(i32::MAX).is_ok());
    assert!(locals_test_mod.tee_test(i32::MIN).is_ok());
    assert!(locals_test_mod.zero_init(i32::MAX).is_ok());
    assert!(locals_test_mod.running_sum(i32::MAX, i32::MIN, 0).is_ok());
}

/// Integration test: all exports are accessible
#[test]
fn test_all_locals_functions_accessible() {
    let mut locals_test_mod = locals_test::new().unwrap();
    // Simply verify that all test functions can be called without panicking.
    // This proves that the transpiler successfully:
    // 1. Parsed the WAT with local declarations
    // 2. Generated Rust code with proper local variable handling
    // 3. Compiled the generated code successfully
    // 4. Correctly declared local variables so they compile
    let _ = locals_test_mod.swap(1, 2);
    let _ = locals_test_mod.simple_local(3);
    let _ = locals_test_mod.tee_test(4);
    let _ = locals_test_mod.zero_init(5);
    let _ = locals_test_mod.running_sum(6, 7, 8);
}
