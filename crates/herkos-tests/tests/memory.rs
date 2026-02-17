//! Runtime tests for memory operations (Milestone 2).
//!
//! These tests verify that the generated code actually works correctly
//! when executed with the herkos-runtime memory implementation.

use herkos_runtime::IsolatedMemory;

use herkos_tests::{memory_load, memory_roundtrip, memory_store, memory_sum};

#[test]
fn test_memory_store() {
    let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();

    // Store 42 at address 100
    memory_store::func_0(100, 42, &mut mem).unwrap();

    // Verify it was stored
    assert_eq!(mem.load_i32(100).unwrap(), 42);
}

#[test]
fn test_memory_load() {
    let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();

    // Manually store a value
    mem.store_i32(200, 99).unwrap();

    // Load via generated function
    let value = memory_load::func_0(200, &mut mem).unwrap();
    assert_eq!(value, 99);
}

#[test]
fn test_memory_roundtrip() {
    let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();

    // Store and load in one function
    let result = memory_roundtrip::func_0(50, 123, &mut mem).unwrap();
    assert_eq!(result, 123);
}

#[test]
fn test_memory_roundtrip_different_values() {
    let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();

    // Test multiple values
    for value in [0, 1, -1, i32::MAX, i32::MIN, 42, -42] {
        let result = memory_roundtrip::func_0(100, value, &mut mem).unwrap();
        assert_eq!(result, value, "roundtrip failed for value {}", value);
    }
}

#[test]
fn test_out_of_bounds() {
    let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();

    // Try to store beyond memory (1 page = 65536 bytes)
    // i32.store needs 4 bytes, so address 65533 would write to 65536 (out of bounds)
    let result = memory_store::func_0(65533, 42, &mut mem);
    assert!(result.is_err(), "should trap on out-of-bounds store");
}

#[test]
fn test_memory_store_at_boundary() {
    let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();

    // Store at the last valid position for i32 (65536 - 4 = 65532)
    memory_store::func_0(65532, 99, &mut mem).unwrap();

    // Verify it was stored
    let value = memory_load::func_0(65532, &mut mem).unwrap();
    assert_eq!(value, 99);
}

// Memory sum: combines memory loads with control flow
#[test]
fn test_memory_sum_basic() {
    let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();

    // Store [10, 20, 30] starting at address 0
    memory_sum::func_1(0, 10, &mut mem).unwrap();
    memory_sum::func_1(4, 20, &mut mem).unwrap();
    memory_sum::func_1(8, 30, &mut mem).unwrap();

    let sum = memory_sum::func_0(0, 3, &mut mem).unwrap();
    assert_eq!(sum, 60, "sum of [10, 20, 30] should be 60");
}

#[test]
fn test_memory_sum_empty() {
    let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();

    let sum = memory_sum::func_0(0, 0, &mut mem).unwrap();
    assert_eq!(sum, 0, "sum of empty array should be 0");
}

#[test]
fn test_memory_sum_single() {
    let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();

    memory_sum::func_1(100, 42, &mut mem).unwrap();

    let sum = memory_sum::func_0(100, 1, &mut mem).unwrap();
    assert_eq!(sum, 42, "sum of [42] should be 42");
}

#[test]
fn test_memory_sum_negative_values() {
    let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();

    memory_sum::func_1(0, -5, &mut mem).unwrap();
    memory_sum::func_1(4, 10, &mut mem).unwrap();
    memory_sum::func_1(8, -3, &mut mem).unwrap();

    let sum = memory_sum::func_0(0, 3, &mut mem).unwrap();
    assert_eq!(sum, 2, "sum of [-5, 10, -3] should be 2");
}
