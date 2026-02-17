//! Runtime tests for indirect function calls (call_indirect).
//!
//! These tests verify that:
//! 1. call_indirect dispatches to the correct function based on table index
//! 2. Type checking works (wrong type traps with IndirectCallTypeMismatch)
//! 3. Out-of-bounds table index traps

use herkos_runtime::WasmTrap;
use herkos_tests::indirect_call;

// ── dispatch_binop: (i32, i32) -> i32 via $binop type ──

#[test]
fn test_binop_dispatch_add() {
    let mut module = indirect_call::new().unwrap();
    assert_eq!(module.dispatch_binop(10, 3, 0).unwrap(), 13);
}

#[test]
fn test_binop_dispatch_sub() {
    let mut module = indirect_call::new().unwrap();
    assert_eq!(module.dispatch_binop(10, 3, 1).unwrap(), 7);
}

#[test]
fn test_binop_dispatch_mul() {
    let mut module = indirect_call::new().unwrap();
    assert_eq!(module.dispatch_binop(10, 3, 2).unwrap(), 30);
}

#[test]
fn test_binop_dispatch_all_ops() {
    let mut module = indirect_call::new().unwrap();
    for (a, b) in [(1, 2), (100, 50), (-5, 3), (0, 0)] {
        assert_eq!(
            module.dispatch_binop(a, b, 0).unwrap(),
            a.wrapping_add(b),
            "add({a}, {b})"
        );
        assert_eq!(
            module.dispatch_binop(a, b, 1).unwrap(),
            a.wrapping_sub(b),
            "sub({a}, {b})"
        );
        assert_eq!(
            module.dispatch_binop(a, b, 2).unwrap(),
            a.wrapping_mul(b),
            "mul({a}, {b})"
        );
    }
}

#[test]
fn test_binop_direct_vs_indirect() {
    let mut module = indirect_call::new().unwrap();
    assert_eq!(
        module.add(7, 3).unwrap(),
        module.dispatch_binop(7, 3, 0).unwrap()
    );
    assert_eq!(
        module.sub(7, 3).unwrap(),
        module.dispatch_binop(7, 3, 1).unwrap()
    );
    assert_eq!(
        module.mul(7, 3).unwrap(),
        module.dispatch_binop(7, 3, 2).unwrap()
    );
}

// ── dispatch_unop: (i32) -> i32 via $unop type ──

#[test]
fn test_unop_dispatch_negate() {
    let mut module = indirect_call::new().unwrap();
    // table[3] = negate, type $unop
    assert_eq!(module.dispatch_unop(42, 3).unwrap(), -42);
    assert_eq!(module.dispatch_unop(0, 3).unwrap(), 0);
    assert_eq!(module.dispatch_unop(-7, 3).unwrap(), 7);
}

#[test]
fn test_unop_direct_vs_indirect() {
    let mut module = indirect_call::new().unwrap();
    for v in [0, 1, -1, 42, i32::MAX, i32::MIN] {
        assert_eq!(
            module.negate(v).unwrap(),
            module.dispatch_unop(v, 3).unwrap(),
            "negate({v})"
        );
    }
}

// ── Type mismatch: calling wrong type traps ──

#[test]
fn test_binop_dispatch_hits_unop_entry() {
    let mut module = indirect_call::new().unwrap();
    // table[3] = negate ($unop), but dispatch_binop expects $binop → type mismatch
    let result = module.dispatch_binop(1, 2, 3);
    assert_eq!(result.unwrap_err(), WasmTrap::IndirectCallTypeMismatch);
}

#[test]
fn test_unop_dispatch_hits_binop_entry() {
    let mut module = indirect_call::new().unwrap();
    // table[0] = add ($binop), but dispatch_unop expects $unop → type mismatch
    let result = module.dispatch_unop(1, 0);
    assert_eq!(result.unwrap_err(), WasmTrap::IndirectCallTypeMismatch);

    // Same for sub (index 1) and mul (index 2)
    assert_eq!(
        module.dispatch_unop(1, 1).unwrap_err(),
        WasmTrap::IndirectCallTypeMismatch
    );
    assert_eq!(
        module.dispatch_unop(1, 2).unwrap_err(),
        WasmTrap::IndirectCallTypeMismatch
    );
}

// ── Table boundary errors ──

#[test]
fn test_undefined_element() {
    let mut module = indirect_call::new().unwrap();
    // Table has 4 initialized entries (0..3), index 4 is uninitialized
    let result = module.dispatch_binop(1, 2, 4);
    assert_eq!(result.unwrap_err(), WasmTrap::UndefinedElement);
}

#[test]
fn test_table_out_of_bounds() {
    let mut module = indirect_call::new().unwrap();
    // Table size is 5, index 5 is past the end
    let result = module.dispatch_binop(1, 2, 5);
    assert_eq!(result.unwrap_err(), WasmTrap::TableOutOfBounds);
}

#[test]
fn test_negative_index_out_of_bounds() {
    let mut module = indirect_call::new().unwrap();
    // Negative i32 interpreted as u32 → very large index → out of bounds
    let result = module.dispatch_binop(1, 2, -1);
    assert_eq!(result.unwrap_err(), WasmTrap::TableOutOfBounds);
}
