//! End-to-end tests for multiple imports (Phase 3, Milestone 5).
//!
//! These tests verify that:
//! 1. Modules with imports from multiple host modules work correctly
//! 2. Calls to imported functions are correctly dispatched
//! 3. Calls to local functions work correctly
//! 4. Mixed local and import calls are correctly dispatched
//! 5. Trait bounds include all necessary import traits

use herkos_runtime::WasmResult;
use herkos_tests::import_multi;

/// Mock host that provides all import functions
struct MultiImportHost {
    add_calls: usize,
    mul_calls: usize,
    log_value: Option<i32>,
    fd_write_result: i32,
}

impl MultiImportHost {
    fn new() -> Self {
        MultiImportHost {
            add_calls: 0,
            mul_calls: 0,
            log_value: None,
            fd_write_result: 0,
        }
    }
}

// Implement EnvImports trait
impl import_multi::EnvImports for MultiImportHost {
    fn add(&mut self, a: i32, b: i32) -> WasmResult<i32> {
        self.add_calls += 1;
        Ok(a + b)
    }

    fn mul(&mut self, a: i32, b: i32) -> WasmResult<i32> {
        self.mul_calls += 1;
        Ok(a * b)
    }

    fn log(&mut self, value: i32) -> WasmResult<()> {
        self.log_value = Some(value);
        Ok(())
    }
}

// Implement WasiSnapshotPreview1Imports trait
impl import_multi::WasiSnapshotPreview1Imports for MultiImportHost {
    fn fd_write(&mut self, _fd: i32, _iov: i32, _iovlen: i32, _nwritten: i32) -> WasmResult<i32> {
        Ok(self.fd_write_result)
    }
}

#[test]
fn test_mixed_local_and_import_calls() {
    let mut host = MultiImportHost::new();
    let mut module = import_multi::new().unwrap();

    // mixed_calls(5, 3) should:
    // 1. Call import add(5, 3) = 8
    // 2. Call local mul(8, 2) = 16
    let result = module.mixed_calls(5, 3, &mut host).unwrap();
    assert_eq!(result, 16);
    assert_eq!(host.add_calls, 1, "Should call imported add once");
}

#[test]
fn test_multiple_imports_called() {
    let mut host = MultiImportHost::new();
    let mut module = import_multi::new().unwrap();

    // use_multiple_imports(10, 20) should:
    // 1. Call import add(10, 20) = 30
    // 2. Call import log(30)
    // 3. Return 30
    let result = module.use_multiple_imports(10, 20, &mut host).unwrap();
    assert_eq!(result, 30);
    assert_eq!(host.add_calls, 1, "Should call add once");
    assert_eq!(
        host.log_value,
        Some(30),
        "Should have logged the result of add"
    );
}

#[test]
fn test_wasi_import() {
    let mut host = MultiImportHost::new();
    host.fd_write_result = 5;
    let mut module = import_multi::new().unwrap();

    let result = module.use_wasi(&mut host).unwrap();
    assert_eq!(result, 5);
}

#[test]
fn test_call_all_imports() {
    let mut host = MultiImportHost::new();
    host.fd_write_result = 7;
    let mut module = import_multi::new().unwrap();

    // call_all_imports(3, 4) should:
    // 1. Call add(3, 4) = 7
    // 2. Call log(7)
    // 3. Call mul(3, 4) = 12
    // 4. Add: 7 + 12 = 19
    // 5. Call fd_write() = 7
    // 6. Add: 19 + 7 = 26
    let result = module.call_all_imports(3, 4, &mut host).unwrap();
    assert_eq!(result, 26);
    assert_eq!(host.add_calls, 1);
    assert_eq!(host.mul_calls, 1);
    assert_eq!(host.log_value, Some(7));
}

#[test]
fn test_call_local_functions_only() {
    let mut module = import_multi::new().unwrap();

    // call_local_only(2, 3) should:
    // 1. Call local add(2, 3) = 5
    // 2. Call local mul(5, 3) = 15
    let result = module.call_local_only(2, 3).unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_local_then_import_sequence() {
    let mut host = MultiImportHost::new();
    let mut module = import_multi::new().unwrap();

    // local_then_import(5, 3) should:
    // 1. Call local add(5, 3) = 8
    // 2. Call import mul(8, 10) = 80
    let result = module.local_then_import(5, 3, &mut host).unwrap();
    assert_eq!(result, 80);
    assert_eq!(host.mul_calls, 1);
}

#[test]
fn test_counter_management() {
    let mut host = MultiImportHost::new();
    let mut module = import_multi::new().unwrap();

    // Get counter (should start at 0)
    let counter = module.get_counter().unwrap();
    assert_eq!(counter, 0);

    // Call mixed_calls which increments counter
    let _ = module.mixed_calls(1, 1, &mut host).unwrap();

    // Check counter was incremented
    let counter = module.get_counter().unwrap();
    assert_eq!(counter, 1);
}

#[test]
fn test_mixed_import_sources() {
    // Test that the same host can provide imports from multiple modules
    // (EnvImports and WasiSnapshotPreview1Imports)
    let mut host = MultiImportHost::new();
    host.fd_write_result = 10;

    let mut module = import_multi::new().unwrap();

    // Call function that uses EnvImports
    let _ = module.use_multiple_imports(5, 5, &mut host).unwrap();
    assert_eq!(host.add_calls, 1);

    // Call function that uses WasiSnapshotPreview1Imports
    let result = module.use_wasi(&mut host).unwrap();
    assert_eq!(result, 10);

    // Host should have serviced both types of imports
    assert!(host.add_calls > 0);
}

#[test]
fn test_call_sequence_complexity() {
    let mut host = MultiImportHost::new();
    host.fd_write_result = 3;
    let mut module = import_multi::new().unwrap();

    // call_all_imports tests a complex sequence:
    // - Multiple imports from same module
    // - Imports from different modules
    // - Parameter passing
    // - Return value propagation
    let result = module.call_all_imports(2, 5, &mut host).unwrap();

    // Verify:
    // add(2, 5) = 7
    // log(7) called
    // mul(2, 5) = 10
    // 7 + 10 = 17
    // fd_write() = 3
    // 17 + 3 = 20
    assert_eq!(result, 20);
    assert_eq!(host.add_calls, 1, "add should be called once");
    assert_eq!(host.mul_calls, 1, "mul should be called once");
    assert_eq!(host.log_value, Some(7), "log should have been called");
}

#[test]
fn test_multiple_hosts_with_different_implementations() {
    // Test that different host implementations can be used with the same module
    struct Host1;
    struct Host2;

    impl import_multi::EnvImports for Host1 {
        fn add(&mut self, a: i32, b: i32) -> WasmResult<i32> {
            Ok(a + b)
        }
        fn mul(&mut self, a: i32, b: i32) -> WasmResult<i32> {
            Ok(a * b)
        }
        fn log(&mut self, _value: i32) -> WasmResult<()> {
            Ok(())
        }
    }

    impl import_multi::WasiSnapshotPreview1Imports for Host1 {
        fn fd_write(
            &mut self,
            _fd: i32,
            _iov: i32,
            _iovlen: i32,
            _nwritten: i32,
        ) -> WasmResult<i32> {
            Ok(1)
        }
    }

    impl import_multi::EnvImports for Host2 {
        fn add(&mut self, a: i32, b: i32) -> WasmResult<i32> {
            Ok(a - b) // Different implementation
        }
        fn mul(&mut self, a: i32, b: i32) -> WasmResult<i32> {
            Ok(a + b) // Different implementation
        }
        fn log(&mut self, _value: i32) -> WasmResult<()> {
            Ok(())
        }
    }

    impl import_multi::WasiSnapshotPreview1Imports for Host2 {
        fn fd_write(
            &mut self,
            _fd: i32,
            _iov: i32,
            _iovlen: i32,
            _nwritten: i32,
        ) -> WasmResult<i32> {
            Ok(2)
        }
    }

    let mut module1 = import_multi::new().unwrap();
    let mut module2 = import_multi::new().unwrap();

    // With Host1 (add = normal addition)
    let mut host1 = Host1;
    let result1 = module1.mixed_calls(10, 5, &mut host1).unwrap();
    // add(10, 5) = 15, mul(15, 2) = 30
    assert_eq!(result1, 30);

    // With Host2 (add = subtraction)
    let mut host2 = Host2;
    let result2 = module2.mixed_calls(10, 5, &mut host2).unwrap();
    // add(10, 5) = 5, mul(5, 2) = 10
    assert_eq!(result2, 10);
}
