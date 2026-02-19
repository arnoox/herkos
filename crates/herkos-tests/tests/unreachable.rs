//! Tests for the `unreachable` operator (trap instruction).

use herkos_tests::unreachable;

// ── Always traps ──

#[test]
fn test_always_traps() {
    let mut unreachable_mod = unreachable::new().unwrap();
    let result = unreachable_mod.func_0();
    assert!(result.is_err(), "unreachable should trap");
}

// ── Conditional trap ──

#[test]
fn test_trap_on_zero() {
    let mut unreachable_mod = unreachable::new().unwrap();
    assert!(
        unreachable_mod.func_1(0).is_err(),
        "should trap when n == 0"
    );
}

#[test]
fn test_no_trap_on_nonzero() {
    let mut unreachable_mod = unreachable::new().unwrap();
    assert_eq!(unreachable_mod.func_1(5).unwrap(), 5);
    assert_eq!(unreachable_mod.func_1(1).unwrap(), 1);
    assert_eq!(unreachable_mod.func_1(-1).unwrap(), -1);
}

// ── Safe division via unreachable ──

#[test]
fn test_safe_div_normal() {
    let mut unreachable_mod = unreachable::new().unwrap();
    assert_eq!(unreachable_mod.func_2(10, 3).unwrap(), 3);
    assert_eq!(unreachable_mod.func_2(100, 10).unwrap(), 10);
    assert_eq!(unreachable_mod.func_2(-10, 3).unwrap(), -3);
}

#[test]
fn test_safe_div_traps_on_zero() {
    let mut unreachable_mod = unreachable::new().unwrap();
    assert!(
        unreachable_mod.func_2(10, 0).is_err(),
        "division by zero should trap"
    );
}

// ── Dead unreachable after return ──

#[test]
fn test_dead_unreachable() {
    let mut unreachable_mod = unreachable::new().unwrap();
    assert_eq!(
        unreachable_mod.func_3(42).unwrap(),
        42,
        "dead unreachable after return should not execute"
    );
    assert_eq!(unreachable_mod.func_3(0).unwrap(), 0);
    assert_eq!(unreachable_mod.func_3(-7).unwrap(), -7);
}
