//! Runtime tests for memory.size and memory.grow operations.

use herkos_runtime::IsolatedMemory;
use herkos_tests::memory_grow;

fn mem() -> IsolatedMemory<4> {
    IsolatedMemory::try_new(1).expect("failed to create memory")
}

#[test]
fn test_initial_size() {
    let mut memory = mem();
    let size = memory_grow::func_0(&mut memory).unwrap();
    assert_eq!(size, 1); // Initial 1 page
}

#[test]
fn test_grow_success() {
    let mut memory = mem();
    let old_size = memory_grow::func_1(1, &mut memory).unwrap(); // grow by 1 page
    assert_eq!(old_size, 1); // Previous size was 1
    let new_size = memory_grow::func_0(&mut memory).unwrap();
    assert_eq!(new_size, 2); // Now 2 pages
}

#[test]
fn test_grow_multiple() {
    let mut memory = mem();
    let old = memory_grow::func_1(2, &mut memory).unwrap(); // grow by 2
    assert_eq!(old, 1);
    let size = memory_grow::func_0(&mut memory).unwrap();
    assert_eq!(size, 3); // 1 + 2 = 3 pages
}

#[test]
fn test_grow_failure_returns_neg1() {
    let mut memory = mem();
    // Max is 4 pages, initial is 1. Try to grow by 4 → would be 5, exceeds max.
    let result = memory_grow::func_1(4, &mut memory).unwrap();
    assert_eq!(result, -1);
    // Size unchanged
    let size = memory_grow::func_0(&mut memory).unwrap();
    assert_eq!(size, 1);
}

#[test]
fn test_grow_then_use_new_memory() {
    let mut memory = mem();
    // Grow by 1 page
    memory_grow::func_1(1, &mut memory).unwrap();
    // Store and load at an address in the second page (offset 65536)
    let val = memory_grow::func_2(65536, 42, &mut memory).unwrap();
    assert_eq!(val, 42);
}

#[test]
fn test_grow_zero() {
    let mut memory = mem();
    // Growing by 0 should succeed and return current size
    let old = memory_grow::func_1(0, &mut memory).unwrap();
    assert_eq!(old, 1);
    let size = memory_grow::func_0(&mut memory).unwrap();
    assert_eq!(size, 1); // unchanged
}
