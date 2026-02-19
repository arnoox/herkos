//! Runtime tests for Module wrapper (Milestone 4).
//!
//! These tests verify that:
//! 1. Modules with mutable globals generate a proper Globals struct
//! 2. The new() constructor initializes globals and data segments
//! 3. Exported functions work correctly as methods on WasmModule

use herkos_tests::{const_global, counter, hello_data};

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
