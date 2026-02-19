//! Runtime tests for memory operations (Milestone 2).
//!
//! These tests verify that the generated code actually works correctly
//! when executed with the herkos-runtime memory implementation.

use herkos_tests::{memory_load, memory_roundtrip, memory_store, memory_sum};

#[test]
fn test_memory_store() {
    let mut memory_store_mod = memory_store::new().unwrap();

    // Store 42 at address 100 - should succeed without error
    memory_store_mod.func_0(100, 42).unwrap();
}

#[test]
fn test_memory_load() {
    let mut memory_load_mod = memory_load::new().unwrap();

    // Load from address 0 (should be 0 by default from zero-initialized memory)
    let value = memory_load_mod.func_0(0).unwrap();
    assert_eq!(value, 0);
}

#[test]
fn test_memory_roundtrip() {
    let mut memory_roundtrip_mod = memory_roundtrip::new().unwrap();

    // Store and load in one function
    let result = memory_roundtrip_mod.func_0(50, 123).unwrap();
    assert_eq!(result, 123);
}

#[test]
fn test_memory_roundtrip_different_values() {
    let mut memory_roundtrip_mod = memory_roundtrip::new().unwrap();

    // Test multiple values
    for value in [0, 1, -1, i32::MAX, i32::MIN, 42, -42] {
        let result = memory_roundtrip_mod.func_0(100, value).unwrap();
        assert_eq!(result, value, "roundtrip failed for value {}", value);
    }
}

#[test]
fn test_out_of_bounds() {
    let mut memory_store_mod = memory_store::new().unwrap();

    // Try to store beyond memory (1 page = 65536 bytes)
    // i32.store needs 4 bytes, so address 65533 would write to 65536 (out of bounds)
    let result = memory_store_mod.func_0(65533, 42);
    assert!(result.is_err(), "should trap on out-of-bounds store");
}

#[test]
fn test_memory_store_at_boundary() {
    let mut memory_roundtrip_mod = memory_roundtrip::new().unwrap();

    // Store at the last valid position for i32 (65536 - 4 = 65532)
    // Use roundtrip to verify it was stored
    let result = memory_roundtrip_mod.func_0(65532, 99).unwrap();
    assert_eq!(result, 99);
}

// Memory sum: combines memory loads with control flow
#[test]
fn test_memory_sum_basic() {
    let mut memory_sum_mod = memory_sum::new().unwrap();

    // Store [10, 20, 30] starting at address 0
    memory_sum_mod.func_1(0, 10).unwrap();
    memory_sum_mod.func_1(4, 20).unwrap();
    memory_sum_mod.func_1(8, 30).unwrap();

    let sum = memory_sum_mod.func_0(0, 3).unwrap();
    assert_eq!(sum, 60, "sum of [10, 20, 30] should be 60");
}

#[test]
fn test_memory_sum_empty() {
    let mut memory_sum_mod = memory_sum::new().unwrap();

    let sum = memory_sum_mod.func_0(0, 0).unwrap();
    assert_eq!(sum, 0, "sum of empty array should be 0");
}

#[test]
fn test_memory_sum_single() {
    let mut memory_sum_mod = memory_sum::new().unwrap();

    memory_sum_mod.func_1(100, 42).unwrap();

    let sum = memory_sum_mod.func_0(100, 1).unwrap();
    assert_eq!(sum, 42, "sum of [42] should be 42");
}

#[test]
fn test_memory_sum_negative_values() {
    let mut memory_sum_mod = memory_sum::new().unwrap();

    memory_sum_mod.func_1(0, -5).unwrap();
    memory_sum_mod.func_1(4, 10).unwrap();
    memory_sum_mod.func_1(8, -3).unwrap();

    let sum = memory_sum_mod.func_0(0, 3).unwrap();
    assert_eq!(sum, 2, "sum of [-5, 10, -3] should be 2");
}
