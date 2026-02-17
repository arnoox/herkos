//! Tests for the `select` operator (conditional ternary).

use herkos_tests::select;

// ── max via select ──

#[test]
fn test_max_first_larger() {
    assert_eq!(select::func_0(10, 5).unwrap(), 10);
}

#[test]
fn test_max_second_larger() {
    assert_eq!(select::func_0(3, 7).unwrap(), 7);
}

#[test]
fn test_max_equal() {
    assert_eq!(select::func_0(4, 4).unwrap(), 4);
}

#[test]
fn test_max_negative() {
    assert_eq!(select::func_0(-1, -5).unwrap(), -1);
}

// ── min via select ──

#[test]
fn test_min_first_smaller() {
    assert_eq!(select::func_1(3, 7).unwrap(), 3);
}

#[test]
fn test_min_second_smaller() {
    assert_eq!(select::func_1(10, 5).unwrap(), 5);
}

#[test]
fn test_min_equal() {
    assert_eq!(select::func_1(4, 4).unwrap(), 4);
}

#[test]
fn test_min_negative() {
    assert_eq!(select::func_1(-1, -5).unwrap(), -5);
}

// ── abs via select ──

#[test]
fn test_abs_positive() {
    assert_eq!(select::func_2(42).unwrap(), 42);
}

#[test]
fn test_abs_negative() {
    assert_eq!(select::func_2(-42).unwrap(), 42);
}

#[test]
fn test_abs_zero() {
    assert_eq!(select::func_2(0).unwrap(), 0);
}

// ── clamp to non-negative ──

#[test]
fn test_clamp_positive() {
    assert_eq!(select::func_3(5).unwrap(), 5);
}

#[test]
fn test_clamp_negative() {
    assert_eq!(select::func_3(-5).unwrap(), 0);
}

#[test]
fn test_clamp_zero() {
    assert_eq!(select::func_3(0).unwrap(), 0);
}

// ── conditional increment ──

#[test]
fn test_cond_inc_true() {
    assert_eq!(select::func_4(10, 1).unwrap(), 11);
}

#[test]
fn test_cond_inc_false() {
    assert_eq!(select::func_4(10, 0).unwrap(), 10);
}

#[test]
fn test_cond_inc_nonzero_flag() {
    assert_eq!(select::func_4(10, 99).unwrap(), 11);
}
