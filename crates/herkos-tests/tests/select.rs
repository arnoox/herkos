//! Tests for the `select` operator (conditional ternary).

use herkos_tests::select;

// ── max via select ──

#[test]
fn test_max_first_larger() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_0(10, 5).unwrap(), 10);
}

#[test]
fn test_max_second_larger() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_0(3, 7).unwrap(), 7);
}

#[test]
fn test_max_equal() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_0(4, 4).unwrap(), 4);
}

#[test]
fn test_max_negative() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_0(-1, -5).unwrap(), -1);
}

// ── min via select ──

#[test]
fn test_min_first_smaller() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_1(3, 7).unwrap(), 3);
}

#[test]
fn test_min_second_smaller() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_1(10, 5).unwrap(), 5);
}

#[test]
fn test_min_equal() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_1(4, 4).unwrap(), 4);
}

#[test]
fn test_min_negative() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_1(-1, -5).unwrap(), -5);
}

// ── abs via select ──

#[test]
fn test_abs_positive() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_2(42).unwrap(), 42);
}

#[test]
fn test_abs_negative() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_2(-42).unwrap(), 42);
}

#[test]
fn test_abs_zero() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_2(0).unwrap(), 0);
}

// ── clamp to non-negative ──

#[test]
fn test_clamp_positive() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_3(5).unwrap(), 5);
}

#[test]
fn test_clamp_negative() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_3(-5).unwrap(), 0);
}

#[test]
fn test_clamp_zero() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_3(0).unwrap(), 0);
}

// ── conditional increment ──

#[test]
fn test_cond_inc_true() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_4(10, 1).unwrap(), 11);
}

#[test]
fn test_cond_inc_false() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_4(10, 0).unwrap(), 10);
}

#[test]
fn test_cond_inc_nonzero_flag() {
    let mut select_mod = select::new().unwrap();
    assert_eq!(select_mod.func_4(10, 99).unwrap(), 11);
}
