//! Runtime tests for memory.size and memory.grow operations.

use herkos_tests::memory_grow;

#[test]
fn test_initial_size() {
    let mut memory_grow_mod = memory_grow::new().unwrap();
    let size = memory_grow_mod.get_size().unwrap();
    assert_eq!(size, 1); // Initial 1 page
}

#[test]
fn test_grow_success() {
    let mut memory_grow_mod = memory_grow::new().unwrap();
    let old_size = memory_grow_mod.grow(1).unwrap(); // grow by 1 page
    assert_eq!(old_size, 1); // Previous size was 1
    let new_size = memory_grow_mod.get_size().unwrap();
    assert_eq!(new_size, 2); // Now 2 pages
}

#[test]
fn test_grow_multiple() {
    let mut memory_grow_mod = memory_grow::new().unwrap();
    let old = memory_grow_mod.grow(1).unwrap(); // grow by 1 page
    assert_eq!(old, 1);
    let size = memory_grow_mod.get_size().unwrap();
    assert_eq!(size, 2); // 1 + 1 = 2 pages
}

#[test]
fn test_grow_failure_returns_neg1() {
    let mut memory_grow_mod = memory_grow::new().unwrap();
    // Max is 2 pages, initial is 1. Try to grow by 2 → would be 3, exceeds max.
    let result = memory_grow_mod.grow(2).unwrap();
    assert_eq!(result, -1);
    // Size unchanged
    let size = memory_grow_mod.get_size().unwrap();
    assert_eq!(size, 1);
}

#[test]
fn test_grow_then_use_new_memory() {
    let mut memory_grow_mod = memory_grow::new().unwrap();
    // Grow by 1 page
    memory_grow_mod.grow(1).unwrap();
    // Store and load at an address in the second page (offset 65536)
    let val = memory_grow_mod.store_and_load(65536, 42).unwrap();
    assert_eq!(val, 42);
}

#[test]
fn test_grow_zero() {
    let mut memory_grow_mod = memory_grow::new().unwrap();
    // Growing by 0 should succeed and return current size
    let old = memory_grow_mod.grow(0).unwrap();
    assert_eq!(old, 1);
    let size = memory_grow_mod.get_size().unwrap();
    assert_eq!(size, 1); // unchanged
}
