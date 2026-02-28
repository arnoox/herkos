//! End-to-end tests for inter-module lending.
//!
//! These tests verify the complete inter-module interaction pattern:
//! 1. A host owns memory (simulating a Module's IsolatedMemory)
//! 2. A LibraryModule borrows that memory for read/write operations
//! 3. Import traits enable communication between modules and host
//! 4. Multiple library modules can operate on the same borrowed memory
//! 5. A single host can service import traits from different module types

use herkos_runtime::{IsolatedMemory, WasmResult};
use herkos_tests::{import_basic, import_memory, import_multi};

/// Host that owns memory and coordinates between multiple library modules.
struct InterModuleHost {
    /// Values logged by import_memory's print_i32
    logged_values: Vec<i32>,
    /// Value returned by import_basic's read_i32
    read_value: i32,
    /// Tracks add calls for import_multi
    add_call_count: usize,
}

impl InterModuleHost {
    fn new() -> Self {
        InterModuleHost {
            logged_values: Vec::new(),
            read_value: 42,
            add_call_count: 0,
        }
    }
}

// -- import_memory traits --

impl import_memory::EnvImports for InterModuleHost {
    fn print_i32(&mut self, value: i32) -> WasmResult<()> {
        self.logged_values.push(value);
        Ok(())
    }
}

// -- import_basic traits --

impl import_basic::EnvImports for InterModuleHost {
    fn print_i32(&mut self, value: i32) -> WasmResult<()> {
        self.logged_values.push(value);
        Ok(())
    }

    fn read_i32(&mut self) -> WasmResult<i32> {
        Ok(self.read_value)
    }
}

impl import_basic::WasiSnapshotPreview1Imports for InterModuleHost {
    fn fd_write(&mut self, _: i32, _: i32, _: i32, _: i32) -> WasmResult<i32> {
        Ok(0)
    }
}

// -- import_multi traits --

impl import_multi::EnvImports for InterModuleHost {
    fn add(&mut self, a: i32, b: i32) -> WasmResult<i32> {
        self.add_call_count += 1;
        Ok(a + b)
    }

    fn mul(&mut self, a: i32, b: i32) -> WasmResult<i32> {
        Ok(a * b)
    }

    fn log(&mut self, value: i32) -> WasmResult<()> {
        self.logged_values.push(value);
        Ok(())
    }
}

impl import_multi::WasiSnapshotPreview1Imports for InterModuleHost {
    fn fd_write(&mut self, _: i32, _: i32, _: i32, _: i32) -> WasmResult<i32> {
        Ok(0)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn test_host_writes_library_reads() {
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut lib = import_memory::new().unwrap();

    // Host writes values at several offsets
    memory.store_i32(0, 100).unwrap();
    memory.store_i32(4, 200).unwrap();
    memory.store_i32(8, 300).unwrap();

    // Library borrows memory and reads back
    assert_eq!(lib.read_at(0, &mut memory).unwrap(), 100);
    assert_eq!(lib.read_at(4, &mut memory).unwrap(), 200);
    assert_eq!(lib.read_at(8, &mut memory).unwrap(), 300);
}

#[test]
fn test_library_writes_host_reads() {
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut lib = import_memory::new().unwrap();

    // Library writes via borrowed memory
    lib.write_at(0, 999, &mut memory).unwrap();
    lib.write_at(4, 888, &mut memory).unwrap();

    // Host reads directly from its own memory
    assert_eq!(memory.load_i32(0).unwrap(), 999);
    assert_eq!(memory.load_i32(4).unwrap(), 888);
}

#[test]
fn test_roundtrip_through_library() {
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut lib = import_memory::new().unwrap();

    // Host writes initial data
    memory.store_i32(0, 50).unwrap();

    // Library reads the value
    let val = lib.read_at(0, &mut memory).unwrap();
    assert_eq!(val, 50);

    // Library writes a transformed value at a different offset
    lib.write_at(4, val * 3, &mut memory).unwrap();

    // Host reads the transformed value
    assert_eq!(memory.load_i32(4).unwrap(), 150);
}

#[test]
fn test_library_with_imports_and_memory() {
    let mut host = InterModuleHost::new();
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut lib = import_memory::new().unwrap();

    // Host writes a value into memory
    memory.store_i32(0, 77).unwrap();

    // Library's `process` reads from memory at offset 0 (gets 77),
    // calls import print_i32(77), then stores 77 at offset 0.
    lib.process(0, &mut memory, &mut host).unwrap();

    // Verify the import was called with the value from memory
    assert_eq!(host.logged_values, vec![77]);

    // Verify the value was stored back
    assert_eq!(memory.load_i32(0).unwrap(), 77);
}

#[test]
fn test_two_libraries_same_memory() {
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut lib_a = import_memory::new().unwrap();
    let mut lib_b = import_memory::new().unwrap();

    // Library A writes to the shared memory
    lib_a.write_at(0, 111, &mut memory).unwrap();
    lib_a.write_at(4, 222, &mut memory).unwrap();

    // Library B reads from the same memory — should see A's writes
    assert_eq!(lib_b.read_at(0, &mut memory).unwrap(), 111);
    assert_eq!(lib_b.read_at(4, &mut memory).unwrap(), 222);

    // Library B overwrites
    lib_b.write_at(0, 333, &mut memory).unwrap();

    // Library A sees B's change
    assert_eq!(lib_a.read_at(0, &mut memory).unwrap(), 333);
}

#[test]
fn test_multiple_module_types_shared_host() {
    let mut host = InterModuleHost::new();
    let mut basic_mod = import_basic::new().unwrap();
    let mut multi_mod = import_multi::new().unwrap();

    // Use import_basic: test_imports calls print_i32 and read_i32
    host.read_value = 10;
    let result = basic_mod.test_imports(5, &mut host).unwrap();
    // test_imports: increments counter, calls print_i32(5), returns read_i32() + 10 = 20
    assert_eq!(result, 20);
    assert!(host.logged_values.contains(&5));

    // Use import_multi: mixed_calls uses env.add import + local mul
    let result = multi_mod.mixed_calls(3, 4, &mut host).unwrap();
    // mixed_calls(3, 4): add(3,4)=7, local_mul(7,2)=14
    assert_eq!(result, 14);
    assert_eq!(host.add_call_count, 1);

    // Use import_multi: call_local_only uses no imports at all
    let result = multi_mod.call_local_only(2, 5).unwrap();
    // local_add(2,5)=7, local_mul(7,3)=21
    assert_eq!(result, 21);
}

#[test]
fn test_memory_grow_visible_across_modules() {
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(1).unwrap());
    let mut lib_a = import_memory::new().unwrap();
    let mut lib_b = import_memory::new().unwrap();

    // Initial size: 1 page
    assert_eq!(lib_a.memory_size(&mut memory).unwrap(), 1);

    // Library A grows memory by 1 page
    let prev = lib_a.try_grow(1, &mut memory).unwrap();
    assert_eq!(prev, 1, "previous size should be 1");

    // Library B sees the new size
    assert_eq!(lib_b.memory_size(&mut memory).unwrap(), 2);

    // Library B can write to the grown region (page 1 = offset 65536)
    lib_b.write_at(65536, 42, &mut memory).unwrap();
    assert_eq!(lib_a.read_at(65536, &mut memory).unwrap(), 42);
}
