//! Regression tests for local variable aliasing in the IR builder.
//!
//! These tests pin the exact instruction patterns that exposed the `LocalGet`
//! aliasing bug.  The bug: `LocalGet` used to push the local's `VarId`
//! directly onto the value stack.  A later `local.tee` or `local.set` on the
//! same local would emit `vN = new_value`, overwriting the variable that was
//! already sitting deeper on the value stack — silently corrupting any
//! subsequent computation that tried to use that "old" stack entry.
//!
//! Fix: `LocalGet` now emits a fresh copy (`vM = vN`) and pushes `vM`, so the
//! snapshot is immutable from the perspective of later mutations to the local.
//!
//! See `data/wat/locals_aliasing.wat` for the minimal WAT reproductions.

use herkos_tests::locals_aliasing;

fn module() -> locals_aliasing::WasmModule {
    locals_aliasing::new().expect("module instantiation should succeed")
}

// ── mod10_via_tee ────────────────────────────────────────────────────────────
//
// REGRESSION: the exact inner-loop pattern of `digital_root` that triggered
// the original bug.
//
// Instruction sequence:
//   local.get 0          <- snapshot of n  (stays at stack bottom until sub)
//   local.get 0          <- n for division (consumed by div_u)
//   i32.const 10
//   i32.div_u            <- n / 10
//   local.tee 0          <- local 0 ← n/10  ← BUG POINT: overwrote v0
//   i32.const 10
//   i32.mul              <- (n/10)*10
//   i32.sub              <- n - (n/10)*10   ← used the (now-corrupted) v0
//
// Before the fix, digital_root(10) returned -9:
//   n/10 = 1, (1)*10 = 10, but v0 was already 1 → 1 - 10 = -9.
// After the fix it correctly returns 0: 10 - 10 = 0.

#[test]
fn test_mod10_tee_n10_regression() {
    // n=10 is the value that returned -9 before the fix.
    assert_eq!(module().mod10_via_tee(10).unwrap(), 0);
}

#[test]
fn test_mod10_tee_exact_zero_remainders() {
    let mut m = module();
    for n in [0u32, 10, 20, 50, 90] {
        assert_eq!(m.mod10_via_tee(n as i32).unwrap(), 0, "n={n}");
    }
}

#[test]
fn test_mod10_tee_nonzero_remainders() {
    let mut m = module();
    assert_eq!(m.mod10_via_tee(1).unwrap(), 1);
    assert_eq!(m.mod10_via_tee(7).unwrap(), 7);
    assert_eq!(m.mod10_via_tee(37).unwrap(), 7);
    assert_eq!(m.mod10_via_tee(99).unwrap(), 9);
    assert_eq!(m.mod10_via_tee(43).unwrap(), 3);
}

// ── preserve_across_set ──────────────────────────────────────────────────────
//
// A value captured by `local.get` must survive a `local.set` on the same
// local.  Returns: old_n - new_n = n - 3n = -2n.
//
// Bug path: both `local.get 0` calls pushed v0 directly.  After
// `local.set 0` wrote v0 = 3n, the bottom stack entry (the "old" read) also
// held 3n → subtraction returned 3n - 3n = 0 for all n.

#[test]
fn test_preserve_across_set_basic() {
    // n=5: 5 - 15 = -10
    assert_eq!(module().preserve_across_set(5).unwrap(), -10);
}

#[test]
fn test_preserve_across_set_one() {
    // n=1: 1 - 3 = -2
    assert_eq!(module().preserve_across_set(1).unwrap(), -2);
}

#[test]
fn test_preserve_across_set_zero() {
    // n=0: 0 - 0 = 0 (passes even with the bug, but confirms no crash)
    assert_eq!(module().preserve_across_set(0).unwrap(), 0);
}

#[test]
fn test_preserve_across_set_negative() {
    // n=-3: (-3) - (-9) = 6
    assert_eq!(module().preserve_across_set(-3).unwrap(), 6);
}

#[test]
fn test_preserve_across_set_varied() {
    let mut m = module();
    for n in [2, 7, 10, 100i32] {
        assert_eq!(
            m.preserve_across_set(n).unwrap(),
            n.wrapping_mul(-2),
            "n={n}"
        );
    }
}

// ── get_snap_vs_tee ──────────────────────────────────────────────────────────
//
// A prior `local.get` snapshot of `a` must be unaffected by a `local.tee`
// that stores (a + b) into the same local.  Returns: a - (a + b) = -b.
//
// Bug path: after `local.tee 0` emitted `v0 = a+b`, the bottom stack entry
// (from the first `local.get 0`) also held a+b → returned 0 instead of -b.

#[test]
fn test_get_snap_vs_tee_basic() {
    // a=10, b=3 → -3
    assert_eq!(module().get_snap_vs_tee(10, 3).unwrap(), -3);
}

#[test]
fn test_get_snap_vs_tee_zero_b() {
    // b=0 → -0 = 0 (passes with bug too, sanity check)
    assert_eq!(module().get_snap_vs_tee(5, 0).unwrap(), 0);
}

#[test]
fn test_get_snap_vs_tee_zero_a() {
    // a=0, b=7 → -7
    assert_eq!(module().get_snap_vs_tee(0, 7).unwrap(), -7);
}

#[test]
fn test_get_snap_vs_tee_negative_b() {
    // a=3, b=-4 → a-(a+b) = -(-4) = 4
    assert_eq!(module().get_snap_vs_tee(3, -4).unwrap(), 4);
}

#[test]
fn test_get_snap_vs_tee_equal() {
    // a=5, b=5 → -5
    assert_eq!(module().get_snap_vs_tee(5, 5).unwrap(), -5);
}

// ── get_tee_then_set ─────────────────────────────────────────────────────────
//
// A `local.get` snapshot must survive *two* consecutive overwrites of the same
// local: first by `local.tee` (stores a+b), then by `local.set` (stores
// (a+b)*2).  Returns: a - (a+b)*2.
//
// Bug path: v0 was overwritten twice.  The bottom stack entry kept tracking
// v0's latest value, eventually equalling (a+b)*2, so the subtraction
// returned 0 instead of a - (a+b)*2.

#[test]
fn test_get_tee_then_set_basic() {
    // a=2, b=3: a+b=5, (a+b)*2=10, result = 2 - 10 = -8
    assert_eq!(module().get_tee_then_set(2, 3).unwrap(), -8);
}

#[test]
fn test_get_tee_then_set_zero_b() {
    // b=0: a - a*2 = -a
    assert_eq!(module().get_tee_then_set(4, 0).unwrap(), -4);
}

#[test]
fn test_get_tee_then_set_zero_a() {
    // a=0, b=5: 0 - 10 = -10
    assert_eq!(module().get_tee_then_set(0, 5).unwrap(), -10);
}

#[test]
fn test_get_tee_then_set_negative() {
    // a=1, b=-3: a+b=-2, (a+b)*2=-4, result = 1 - (-4) = 5
    assert_eq!(module().get_tee_then_set(1, -3).unwrap(), 5);
}

#[test]
fn test_get_tee_then_set_varied() {
    let mut m = module();
    let cases: &[(i32, i32)] = &[(0, 0), (1, 1), (5, 2), (10, -3), (-1, 4)];
    for &(a, b) in cases {
        let expected = a - (a + b) * 2;
        assert_eq!(m.get_tee_then_set(a, b).unwrap(), expected, "a={a} b={b}");
    }
}
