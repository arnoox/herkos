//! Runtime tests for Module wrapper (Milestone 4).
//!
//! These tests verify that:
//! 1. Modules with mutable globals generate a proper Globals struct
//! 2. The new() constructor initializes globals and data segments
//! 3. Exported functions work correctly as methods on WasmModule

use herkos_tests::{const_global, counter, data_segments, hello_data};

// ── Counter: mutable global, no memory ──

#[test]
fn test_counter_initial_value() {
    let mut module = counter::new().unwrap();
    assert_eq!(module.get_count().unwrap(), 0, "initial count should be 0");
}

#[test]
fn test_counter_increment() {
    let mut module = counter::new().unwrap();
    assert_eq!(module.increment().unwrap(), 1);
    assert_eq!(module.increment().unwrap(), 2);
    assert_eq!(module.increment().unwrap(), 3);
}

#[test]
fn test_counter_get_count_after_increment() {
    let mut module = counter::new().unwrap();
    assert_eq!(module.get_count().unwrap(), 0);
    module.increment().unwrap();
    assert_eq!(module.get_count().unwrap(), 1);
    module.increment().unwrap();
    module.increment().unwrap();
    assert_eq!(module.get_count().unwrap(), 3);
}

#[test]
fn test_counter_instances_are_isolated() {
    let mut m1 = counter::new().unwrap();
    let mut m2 = counter::new().unwrap();

    m1.increment().unwrap();
    m1.increment().unwrap();

    // m2 should be unaffected
    assert_eq!(m2.get_count().unwrap(), 0);
    assert_eq!(m1.get_count().unwrap(), 2);
}

// ── Hello data: data segment initialization ──

#[test]
fn test_hello_data_init() {
    let mut module = hello_data::new().unwrap();

    // "Hello" = [72, 101, 108, 108, 111]
    // i32.load at address 0 reads 4 bytes as little-endian i32
    // bytes: 72, 101, 108, 108 → 0x6C_6C_65_48 = 1819043144
    let word = module.load_word(0).unwrap();
    let expected = i32::from_le_bytes([72, 101, 108, 108]);
    assert_eq!(word, expected, "first 4 bytes of 'Hello' as LE i32");
}

#[test]
fn test_hello_data_second_byte() {
    let mut module = hello_data::new().unwrap();

    // i32.load at address 1 reads bytes [101, 108, 108, 111] = "ello"
    let word = module.load_word(1).unwrap();
    let expected = i32::from_le_bytes([101, 108, 108, 111]);
    assert_eq!(word, expected, "bytes 1..5 of 'Hello' as LE i32");
}

// ── Data segments: active segments initialised, passive segment skipped ──

#[test]
fn test_data_segments_active_init() {
    let mut module = data_segments::new().unwrap();

    // Active segment 1: bytes [1, 2, 3, 4] at offset 0
    let word = module.load_i32(0).unwrap();
    assert_eq!(word, i32::from_le_bytes([1, 2, 3, 4]));
}

#[test]
fn test_data_segments_second_active_init() {
    let mut module = data_segments::new().unwrap();

    // Active segment 2: bytes [10, 11, 12, 13] at offset 16
    let word = module.load_i32(16).unwrap();
    assert_eq!(word, i32::from_le_bytes([10, 11, 12, 13]));
}

#[test]
fn test_data_segments_byte_access() {
    let mut module = data_segments::new().unwrap();

    // Individual bytes from the first active segment
    assert_eq!(module.load_byte(0).unwrap(), 1);
    assert_eq!(module.load_byte(1).unwrap(), 2);
    assert_eq!(module.load_byte(2).unwrap(), 3);
    assert_eq!(module.load_byte(3).unwrap(), 4);
}

#[test]
fn test_data_segments_passive_does_not_crash() {
    // Instantiating the module must succeed even though a passive data
    // segment is present (it is silently skipped by the transpiler).
    let _module = data_segments::new().unwrap();

    // TODO: we will need a future test that checks that passive segments can be initialized via `memory.init` once we support that instruction. For now we just want to verify that the presence of a passive segment doesn't cause a constructor failure.
}

// ── Const global: immutable global as const item ──

#[test]
fn test_const_global() {
    let mut module = const_global::new().unwrap();
    assert_eq!(module.get_answer().unwrap(), 42);
}

#[test]
fn test_const_global_value() {
    // The const item should be accessible
    assert_eq!(const_global::G0, 42);
}
