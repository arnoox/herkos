//! Runtime tests for sub-width memory operations.
//!
//! Tests i32.load8_s/u, i32.load16_s/u, i32.store8, i32.store16,
//! i64.load8_s/u, i64.load32_s/u, i64.store8, i64.store32.

use herkos_tests::subwidth_mem;

// === i32 byte operations ===

#[test]
fn test_store_load_byte_unsigned() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.store_byte(0, 200).unwrap(); // store_byte(0, 200)
    let val = subwidth_mem_mod.load_byte_u(0).unwrap(); // load_byte_u(0)
    assert_eq!(val, 200); // zero-extended: 200
}

#[test]
fn test_store_load_byte_signed() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.store_byte(0, 200).unwrap(); // store 200 (0xC8)
    let val = subwidth_mem_mod.load_byte_s(0).unwrap(); // load_byte_s(0)
    assert_eq!(val, -56); // sign-extended: 200u8 as i8 = -56
}

#[test]
fn test_store_load_byte_positive() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.store_byte(0, 42).unwrap();
    let u = subwidth_mem_mod.load_byte_u(0).unwrap();
    let s = subwidth_mem_mod.load_byte_s(0).unwrap();
    assert_eq!(u, 42);
    assert_eq!(s, 42); // Both should agree for values < 128
}

#[test]
fn test_store_byte_truncates() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.store_byte(0, 0x1FF).unwrap(); // store 0x1FF, truncated to 0xFF
    let val = subwidth_mem_mod.load_byte_u(0).unwrap();
    assert_eq!(val, 0xFF);
}

// === i32 16-bit operations ===

#[test]
fn test_store_load_i16_unsigned() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.store_i16(0, 50000).unwrap(); // store_i16(0, 50000)
    let val = subwidth_mem_mod.load_i16_u(0).unwrap(); // load_i16_u(0)
    assert_eq!(val, 50000); // zero-extended
}

#[test]
fn test_store_load_i16_signed() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.store_i16(0, 50000).unwrap(); // store 50000 (0xC350)
    let val = subwidth_mem_mod.load_i16_s(0).unwrap(); // load_i16_s(0)
    assert_eq!(val, -15536); // sign-extended: 50000u16 as i16 = -15536
}

#[test]
fn test_store_i16_truncates() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.store_i16(0, 0x1FFFF).unwrap(); // store 0x1FFFF, truncated to 0xFFFF
    let val = subwidth_mem_mod.load_i16_u(0).unwrap();
    assert_eq!(val, 0xFFFF);
}

// === i64 byte operations ===

#[test]
fn test_i64_store_load_byte_unsigned() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.i64_store8(0, 200).unwrap(); // i64_store8(0, 200)
    let val = subwidth_mem_mod.i64_load8_u(0).unwrap(); // i64_load8_u(0)
    assert_eq!(val, 200i64);
}

#[test]
fn test_i64_store_load_byte_signed() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.i64_store8(0, 200).unwrap();
    let val = subwidth_mem_mod.i64_load8_s(0).unwrap(); // i64_load8_s(0)
    assert_eq!(val, -56i64); // sign-extended through i8
}

// === i64 32-bit operations ===

#[test]
fn test_i64_store32_load32_unsigned() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.i64_store32(0, 0xFFFFFFFF).unwrap(); // i64_store32(0, 0xFFFFFFFF)
    let val = subwidth_mem_mod.i64_load32_u(0).unwrap(); // i64_load32_u(0)
    assert_eq!(val, 0xFFFFFFFFi64); // zero-extended
}

#[test]
fn test_i64_store32_load32_signed() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.i64_store32(0, 0xFFFFFFFF).unwrap(); // store -1 as i32
    let val = subwidth_mem_mod.i64_load32_s(0).unwrap(); // i64_load32_s(0)
    assert_eq!(val, -1i64); // sign-extended: -1i32 as i64 = -1i64
}

#[test]
fn test_i64_store32_positive() {
    let mut subwidth_mem_mod = subwidth_mem::new().unwrap();
    subwidth_mem_mod.i64_store32(0, 42).unwrap();
    let u = subwidth_mem_mod.i64_load32_u(0).unwrap();
    let s = subwidth_mem_mod.i64_load32_s(0).unwrap();
    assert_eq!(u, 42i64);
    assert_eq!(s, 42i64);
}
