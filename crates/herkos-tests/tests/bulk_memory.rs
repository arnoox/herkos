//! Integration tests for Wasm bulk-memory operations: memory.fill, memory.init, data.drop.

use herkos_tests::{memory_fill, memory_init};

// ── memory.fill ──────────────────────────────────────────────────────────────

#[test]
fn test_fill_writes_byte_pattern() {
    let mut m = memory_fill::new().unwrap();
    m.fill_region(100, 0xAB, 5).unwrap();
    for i in 0..5i32 {
        assert_eq!(m.load_byte(100 + i).unwrap(), 0xAB);
    }
}

#[test]
fn test_fill_zero_len_is_noop() {
    let mut m = memory_fill::new().unwrap();
    m.fill_region(0, 0xFF, 0).unwrap();
    // Nothing was written — memory stays zero-initialized
    assert_eq!(m.load_byte(0).unwrap(), 0);
}

#[test]
fn test_fill_out_of_bounds_traps() {
    let mut m = memory_fill::new().unwrap();
    // 1 page = 65536 bytes; fill 10 bytes starting near the end overflows
    assert!(m.fill_region(65530, 0, 10).is_err());
}

#[test]
fn test_fill_byte_truncation() {
    // val is i32 on the Wasm stack; only low 8 bits are used
    let mut m = memory_fill::new().unwrap();
    m.fill_region(0, 0x1FF, 1).unwrap(); // 0x1FF & 0xFF = 0xFF
    assert_eq!(m.load_byte(0).unwrap(), 0xFF);
}

#[test]
fn test_fill_entire_region() {
    let mut m = memory_fill::new().unwrap();
    m.fill_region(200, 42, 8).unwrap();
    for i in 0..8i32 {
        assert_eq!(m.load_byte(200 + i).unwrap(), 42);
    }
    // Byte just before and just after should be untouched (zero)
    assert_eq!(m.load_byte(199).unwrap(), 0);
    assert_eq!(m.load_byte(208).unwrap(), 0);
}

// ── memory.init ──────────────────────────────────────────────────────────────

#[test]
fn test_init_full_segment() {
    // PASSIVE_SEGMENT_0 = b"Hello"
    let mut m = memory_init::new().unwrap();
    m.init_region(10, 0, 5).unwrap();
    assert_eq!(m.load_byte(10).unwrap(), b'H' as i32);
    assert_eq!(m.load_byte(11).unwrap(), b'e' as i32);
    assert_eq!(m.load_byte(14).unwrap(), b'o' as i32);
}

#[test]
fn test_init_subrange() {
    // Copy "ell" (bytes 1..4 of "Hello") into address 0
    let mut m = memory_init::new().unwrap();
    m.init_region(0, 1, 3).unwrap();
    assert_eq!(m.load_byte(0).unwrap(), b'e' as i32);
    assert_eq!(m.load_byte(1).unwrap(), b'l' as i32);
    assert_eq!(m.load_byte(2).unwrap(), b'l' as i32);
    // Byte 3 should be zero (not written)
    assert_eq!(m.load_byte(3).unwrap(), 0);
}

#[test]
fn test_init_zero_len_is_noop() {
    let mut m = memory_init::new().unwrap();
    m.init_region(0, 0, 0).unwrap();
    assert_eq!(m.load_byte(0).unwrap(), 0);
}

#[test]
fn test_init_src_out_of_bounds_traps() {
    let mut m = memory_init::new().unwrap();
    // src_offset=3, len=5: 3+5=8 > 5 ("Hello".len())
    assert!(m.init_region(0, 3, 5).is_err());
}

#[test]
fn test_init_dst_out_of_bounds_traps() {
    let mut m = memory_init::new().unwrap();
    assert!(m.init_region(65534, 0, 5).is_err());
}

// ── data.drop ────────────────────────────────────────────────────────────────

#[test]
fn test_data_drop_is_noop() {
    // drop_segment must not trap
    let mut m = memory_init::new().unwrap();
    m.drop_segment().unwrap();
    // Segment data is still accessible after drop (no runtime enforcement)
    m.init_region(0, 0, 5).unwrap();
    assert_eq!(m.load_byte(0).unwrap(), b'H' as i32);
}
