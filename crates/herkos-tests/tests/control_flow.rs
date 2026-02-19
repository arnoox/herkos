//! Runtime tests for control flow (Milestone 3).
//!
//! These tests verify that the generated control flow code
//! executes correctly.

use herkos_tests::{countdown_loop, max, simple_if};

#[test]
fn test_simple_if_true() {
    let mut simple_if_mod = simple_if::new().unwrap();
    let result = simple_if_mod.func_0(1).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_simple_if_false() {
    let mut simple_if_mod = simple_if::new().unwrap();
    let result = simple_if_mod.func_0(0).unwrap();
    assert_eq!(result, 99);
}

#[test]
fn test_max_first_larger() {
    let mut max_mod = max::new().unwrap();
    let result = max_mod.func_0(100, 50).unwrap();
    assert_eq!(result, 100);
}

#[test]
fn test_max_second_larger() {
    let mut max_mod = max::new().unwrap();
    let result = max_mod.func_0(30, 70).unwrap();
    assert_eq!(result, 70);
}

#[test]
fn test_max_equal() {
    let mut max_mod = max::new().unwrap();
    let result = max_mod.func_0(42, 42).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_countdown_loop() {
    let mut countdown_loop_mod = countdown_loop::new().unwrap();
    let result = countdown_loop_mod.func_0(5).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn test_countdown_loop_zero() {
    let mut countdown_loop_mod = countdown_loop::new().unwrap();
    let result = countdown_loop_mod.func_0(0).unwrap();
    assert_eq!(result, 0);
}
