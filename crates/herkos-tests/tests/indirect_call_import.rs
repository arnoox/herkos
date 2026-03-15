//! Regression test for issue #19: indirect calls missing host parameter
//!
//! Tests that call_indirect correctly dispatches to functions that call imports,
//! even when the dispatcher function itself has no direct import calls.

use herkos_runtime::WasmResult;
use herkos_tests::indirect_call_import;

// Mock host implementation
struct MockHost {
    last_logged: Option<i32>,
}

impl MockHost {
    fn new() -> Self {
        MockHost { last_logged: None }
    }
}

// Implement the generated EnvImports trait
impl indirect_call_import::EnvImports for MockHost {
    fn log(&mut self, value: i32) -> WasmResult<()> {
        self.last_logged = Some(value);
        Ok(())
    }
}

#[test]
fn test_call_indirect_with_import() {
    let mut host = MockHost::new();
    let mut module = indirect_call_import::new().unwrap();

    // Test direct call to writer (should call import directly)
    module.writer(42, &mut host).unwrap();
    assert_eq!(
        host.last_logged,
        Some(42),
        "writer should call log import with 42"
    );

    // Reset for next test
    host.last_logged = None;

    // Test call_indirect via dispatcher
    // dispatcher(99, 0) → call_indirect(0) → writer(99) → log(99)
    module.dispatcher(99, 0, &mut host).unwrap();
    assert_eq!(
        host.last_logged,
        Some(99),
        "dispatcher should call_indirect to writer which logs 99"
    );
}

#[test]
fn test_call_indirect_multiple_dispatches() {
    let mut host = MockHost::new();
    let mut module = indirect_call_import::new().unwrap();

    // Verify multiple dispatches work correctly
    for i in 1..=5 {
        host.last_logged = None;
        module.dispatcher(i * 10, 0, &mut host).unwrap();
        assert_eq!(
            host.last_logged,
            Some(i * 10),
            "dispatcher call {}: log should be called with {}",
            i,
            i * 10
        );
    }
}
