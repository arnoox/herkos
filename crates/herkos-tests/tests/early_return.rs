//! Tests for explicit `return` operator (mid-function return).

use herkos_tests::early_return;

// ── Early return if negative ──

#[test]
fn test_early_return_negative() {
    let mut early_return_mod = early_return::new().unwrap();
    assert_eq!(early_return_mod.func_0(-1).unwrap(), -1);
    assert_eq!(early_return_mod.func_0(-100).unwrap(), -1);
}

#[test]
fn test_early_return_positive() {
    let mut early_return_mod = early_return::new().unwrap();
    assert_eq!(early_return_mod.func_0(5).unwrap(), 10);
    assert_eq!(early_return_mod.func_0(0).unwrap(), 0);
    assert_eq!(early_return_mod.func_0(1).unwrap(), 2);
}

// ── First positive (return from inside loop) ──

#[test]
fn test_first_positive_already_positive() {
    let mut early_return_mod = early_return::new().unwrap();
    assert_eq!(early_return_mod.func_1(5).unwrap(), 5);
    assert_eq!(early_return_mod.func_1(1).unwrap(), 1);
}

#[test]
fn test_first_positive_from_zero() {
    let mut early_return_mod = early_return::new().unwrap();
    assert_eq!(early_return_mod.func_1(0).unwrap(), 1);
}

#[test]
fn test_first_positive_from_negative() {
    let mut early_return_mod = early_return::new().unwrap();
    assert_eq!(early_return_mod.func_1(-3).unwrap(), 1);
    assert_eq!(early_return_mod.func_1(-1).unwrap(), 1);
}

// ── Sum with early exit ──

#[test]
fn test_sum_early_exit_invalid() {
    let mut early_return_mod = early_return::new().unwrap();
    assert_eq!(early_return_mod.func_2(0).unwrap(), -1);
    assert_eq!(early_return_mod.func_2(-5).unwrap(), -1);
}

#[test]
fn test_sum_normal() {
    let mut early_return_mod = early_return::new().unwrap();
    assert_eq!(early_return_mod.func_2(1).unwrap(), 1);
    assert_eq!(early_return_mod.func_2(5).unwrap(), 15); // 1+2+3+4+5
    assert_eq!(early_return_mod.func_2(10).unwrap(), 55); // 1+2+..+10
    assert_eq!(early_return_mod.func_2(100).unwrap(), 5050);
}

// ── Nested return ──

#[test]
fn test_nested_return_first_positive() {
    let mut early_return_mod = early_return::new().unwrap();
    assert_eq!(early_return_mod.func_3(1, 0).unwrap(), 10);
    assert_eq!(early_return_mod.func_3(5, -1).unwrap(), 10);
}

#[test]
fn test_nested_return_second_positive() {
    let mut early_return_mod = early_return::new().unwrap();
    assert_eq!(early_return_mod.func_3(0, 1).unwrap(), 20);
    assert_eq!(early_return_mod.func_3(-1, 5).unwrap(), 20);
}

#[test]
fn test_nested_return_both_positive() {
    let mut early_return_mod = early_return::new().unwrap();
    // a > 0 is checked first, so returns 10
    assert_eq!(early_return_mod.func_3(1, 1).unwrap(), 10);
}

#[test]
fn test_nested_return_neither_positive() {
    let mut early_return_mod = early_return::new().unwrap();
    assert_eq!(early_return_mod.func_3(0, 0).unwrap(), 30);
    assert_eq!(early_return_mod.func_3(-1, -1).unwrap(), 30);
}
