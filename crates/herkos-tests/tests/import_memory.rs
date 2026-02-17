//! End-to-end tests for memory import handling (Phase 3, Milestone 5).
//!
//! These tests verify that:
//! 1. LibraryModule with memory import generates correct trait bounds
//! 2. Export methods take a memory parameter
//! 3. Host can lend memory to the module (memory lending pattern)
//! 4. All memory operations (load, store, grow, size) work with imported memory

use herkos_runtime::{IsolatedMemory, WasmResult};
use herkos_tests::import_memory;

/// Mock host that provides import functions for LibraryModule
struct MemoryProvidingHost {
    last_logged: Option<i32>,
}

impl MemoryProvidingHost {
    fn new() -> Self {
        MemoryProvidingHost { last_logged: None }
    }
}

// Implement the EnvImports trait for the host
impl import_memory::EnvImports for MemoryProvidingHost {
    fn print_i32(&mut self, value: i32) -> WasmResult<()> {
        self.last_logged = Some(value);
        Ok(())
    }
}

#[test]
fn test_library_module_memory_lending() {
    // Create a host with memory that will be lent to the module
    let _host = MemoryProvidingHost::new();
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());

    // Create LibraryModule - it doesn't own memory, borrows from host
    let mut module = import_memory::new().unwrap();

    // Write some test data to memory at offset 0
    memory.store_i32(0, 42).unwrap();
    memory.store_i32(4, 100).unwrap();
    memory.store_i32(8, 200).unwrap();

    // Module reads from the borrowed memory
    let value = module.read_at(0, &mut memory).unwrap();
    assert_eq!(value, 42, "Should read value written by host");
}

#[test]
fn test_library_module_write_to_borrowed_memory() {
    let _host = MemoryProvidingHost::new();
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut module = import_memory::new().unwrap();

    // Module writes to the borrowed memory
    module.write_at(0, 123, &mut memory).unwrap();

    // Host reads back what module wrote
    let value = memory.load_i32(0).unwrap();
    assert_eq!(
        value, 123,
        "Module should be able to write to borrowed memory"
    );
}

#[test]
fn test_library_module_roundtrip() {
    let _host = MemoryProvidingHost::new();
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut module = import_memory::new().unwrap();

    let test_value = 12345;

    // Write value from host
    memory.store_i32(0, test_value).unwrap();

    // Module reads the value
    let read_value = module.read_at(0, &mut memory).unwrap();
    assert_eq!(read_value, test_value);

    // Module writes different value
    module.write_at(4, test_value + 1, &mut memory).unwrap();

    // Host reads back what module wrote
    let stored_value = memory.load_i32(4).unwrap();
    assert_eq!(stored_value, test_value + 1);
}

#[test]
fn test_library_module_multiple_offsets() {
    let _host = MemoryProvidingHost::new();
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut module = import_memory::new().unwrap();

    let values = [10, 20, 30, 40, 50];

    // Write values at different offsets
    for (i, &val) in values.iter().enumerate() {
        let offset = (i * 4) as i32;
        memory.store_i32(offset as usize, val).unwrap();
    }

    // Module reads values back
    for (i, &expected) in values.iter().enumerate() {
        let offset = (i * 4) as i32;
        let value = module.read_at(offset, &mut memory).unwrap();
        assert_eq!(value, expected, "Value at offset {}", offset);
    }
}

#[test]
fn test_library_module_imports_and_memory() {
    let mut host = MemoryProvidingHost::new();
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut module = import_memory::new().unwrap();

    // Set up memory with a test value at offset 0
    memory.store_i32(0, 42).unwrap();

    // Call process which:
    // 1. Loads from memory at offset 0 (which is 42)
    // 2. Calls imported print_i32 with that value
    // 3. Stores the value at offset 0 (which we don't check)
    module.process(0, &mut memory, &mut host).unwrap();

    // Verify the import was called with the loaded value
    assert_eq!(
        host.last_logged,
        Some(42),
        "Import function should have been called with the value from memory"
    );
}

#[test]
fn test_memory_size_with_import() {
    let _host = MemoryProvidingHost::new();
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut module = import_memory::new().unwrap();

    // memory.size should return current page count (2 in this case)
    let size = module.memory_size(&mut memory).unwrap();
    assert_eq!(size, 2, "memory.size should return current page count");
}

#[test]
fn test_memory_grow_borrowed() {
    let _host = MemoryProvidingHost::new();
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut module = import_memory::new().unwrap();

    // Grow memory by 1 page
    let prev_size = module.try_grow(1, &mut memory).unwrap();
    assert_eq!(prev_size, 2, "Should return previous size");

    // Verify new size
    let new_size = module.memory_size(&mut memory).unwrap();
    assert_eq!(new_size, 3, "Size should increase after grow");
}

#[test]
fn test_memory_isolation_different_modules() {
    let _host = MemoryProvidingHost::new();

    // Module 1: borrows this memory
    let mut memory1 = Box::new(IsolatedMemory::<4>::try_new(1).unwrap());
    let mut module1 = Box::new(import_memory::new().unwrap());

    // Module 2: borrows different memory
    let mut memory2 = Box::new(IsolatedMemory::<4>::try_new(1).unwrap());
    let mut module2 = Box::new(import_memory::new().unwrap());

    // Write to memory1
    memory1.store_i32(0, 111).unwrap();

    // Write to memory2
    memory2.store_i32(0, 222).unwrap();

    // Module 1 reads its own memory
    let val1 = module1.read_at(0, &mut memory1).unwrap();
    assert_eq!(val1, 111);

    // Module 2 reads its own memory (should be different)
    let val2 = module2.read_at(0, &mut memory2).unwrap();
    assert_eq!(val2, 222);

    // Verify isolation: memory1 and memory2 are separate
    assert_ne!(val1, val2, "Each module should use its own borrowed memory");
}
