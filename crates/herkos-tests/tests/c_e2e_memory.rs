//! End-to-end tests: C → Wasm → Rust (memory/pointer operations).
//!
//! These tests exercise C pointer-based load/store through Wasm linear memory.
//! Addresses are i32 offsets into IsolatedMemory. The clang-compiled Wasm uses
//! a shadow stack growing down from the top of memory for locals/spills.
//! We use low addresses (0..256) for our test data to avoid collisions.

use herkos_tests::c_e2e_memory;

fn new_module() -> c_e2e_memory::WasmModule {
    c_e2e_memory::new().expect("module instantiation should succeed")
}

// ── Basic pointer store/load ──

#[test]
fn test_store_and_load() {
    let mut m = new_module();
    m.store_i32(0, 42).unwrap();
    assert_eq!(m.load_i32(0).unwrap(), 42);

    m.store_i32(4, -1).unwrap();
    assert_eq!(m.load_i32(4).unwrap(), -1);

    // First value should still be there
    assert_eq!(m.load_i32(0).unwrap(), 42);
}

// ── Array sum ──

#[test]
fn test_array_sum() {
    let mut m = new_module();
    // Write [10, 20, 30, 40] starting at address 0
    m.store_i32(0, 10).unwrap();
    m.store_i32(4, 20).unwrap();
    m.store_i32(8, 30).unwrap();
    m.store_i32(12, 40).unwrap();

    assert_eq!(m.array_sum(0, 4).unwrap(), 100);
    assert_eq!(m.array_sum(0, 1).unwrap(), 10);
    assert_eq!(m.array_sum(0, 0).unwrap(), 0);
}

// ── Array max ──

#[test]
fn test_array_max() {
    let mut m = new_module();
    m.store_i32(0, 5).unwrap();
    m.store_i32(4, 3).unwrap();
    m.store_i32(8, 9).unwrap();
    m.store_i32(12, 1).unwrap();

    assert_eq!(m.array_max(0, 4).unwrap(), 9);
}

#[test]
fn test_array_max_negative() {
    let mut m = new_module();
    m.store_i32(0, -5).unwrap();
    m.store_i32(4, -3).unwrap();
    m.store_i32(8, -9).unwrap();

    assert_eq!(m.array_max(0, 3).unwrap(), -3);
}

// ── Dot product ──

#[test]
fn test_dot_product() {
    let mut m = new_module();
    // Vector a at offset 0: [1, 2, 3]
    m.store_i32(0, 1).unwrap();
    m.store_i32(4, 2).unwrap();
    m.store_i32(8, 3).unwrap();
    // Vector b at offset 32: [4, 5, 6]
    m.store_i32(32, 4).unwrap();
    m.store_i32(36, 5).unwrap();
    m.store_i32(40, 6).unwrap();

    // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    assert_eq!(m.dot_product(0, 32, 3).unwrap(), 32);
}

// ── Array reverse ──

#[test]
fn test_array_reverse() {
    let mut m = new_module();
    // [1, 2, 3, 4, 5] at offset 0
    for i in 0..5 {
        m.store_i32(i * 4, i + 1).unwrap();
    }

    m.array_reverse(0, 5).unwrap();

    assert_eq!(m.load_i32(0).unwrap(), 5);
    assert_eq!(m.load_i32(4).unwrap(), 4);
    assert_eq!(m.load_i32(8).unwrap(), 3);
    assert_eq!(m.load_i32(12).unwrap(), 2);
    assert_eq!(m.load_i32(16).unwrap(), 1);
}

#[test]
fn test_array_reverse_even() {
    let mut m = new_module();
    // [10, 20, 30, 40] at offset 0
    m.store_i32(0, 10).unwrap();
    m.store_i32(4, 20).unwrap();
    m.store_i32(8, 30).unwrap();
    m.store_i32(12, 40).unwrap();

    m.array_reverse(0, 4).unwrap();

    assert_eq!(m.load_i32(0).unwrap(), 40);
    assert_eq!(m.load_i32(4).unwrap(), 30);
    assert_eq!(m.load_i32(8).unwrap(), 20);
    assert_eq!(m.load_i32(12).unwrap(), 10);
}

// ── Bubble sort ──

#[test]
fn test_bubble_sort() {
    let mut m = new_module();
    // [5, 3, 1, 4, 2] at offset 0
    m.store_i32(0, 5).unwrap();
    m.store_i32(4, 3).unwrap();
    m.store_i32(8, 1).unwrap();
    m.store_i32(12, 4).unwrap();
    m.store_i32(16, 2).unwrap();

    m.bubble_sort(0, 5).unwrap();

    assert_eq!(m.load_i32(0).unwrap(), 1);
    assert_eq!(m.load_i32(4).unwrap(), 2);
    assert_eq!(m.load_i32(8).unwrap(), 3);
    assert_eq!(m.load_i32(12).unwrap(), 4);
    assert_eq!(m.load_i32(16).unwrap(), 5);
}

#[test]
fn test_bubble_sort_already_sorted() {
    let mut m = new_module();
    m.store_i32(0, 1).unwrap();
    m.store_i32(4, 2).unwrap();
    m.store_i32(8, 3).unwrap();

    m.bubble_sort(0, 3).unwrap();

    assert_eq!(m.load_i32(0).unwrap(), 1);
    assert_eq!(m.load_i32(4).unwrap(), 2);
    assert_eq!(m.load_i32(8).unwrap(), 3);
}
