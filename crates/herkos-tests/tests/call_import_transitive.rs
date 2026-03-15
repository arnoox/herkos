//! Regression test for issue #19 part 2: transitive host parameter in direct calls
//!
//! Tests that functions that transitively call imports (via direct `call` instructions)
//! correctly receive and forward the host parameter.

use herkos_runtime::WasmResult;
use herkos_tests::call_import_transitive;

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
impl call_import_transitive::ModuleHostTrait for MockHost {
    fn log(&mut self, value: i32) -> WasmResult<()> {
        self.last_logged = Some(value);
        Ok(())
    }
}

#[test]
fn test_direct_call_transitive_import() {
    let mut host = MockHost::new();
    let mut module = call_import_transitive::new().unwrap();

    // Test direct call to writer (should call import directly)
    module.writer(42, &mut host).unwrap();
    assert_eq!(
        host.last_logged,
        Some(42),
        "writer should call log import with 42"
    );

    // Reset for next test
    host.last_logged = None;

    // Test transitive call via caller
    // caller(99) → call writer(99) → call log(99)
    module.caller(99, &mut host).unwrap();
    assert_eq!(
        host.last_logged,
        Some(99),
        "caller should transitively call log import with 99"
    );
}

#[test]
fn test_caller_multiple_invocations() {
    let mut host = MockHost::new();
    let mut module = call_import_transitive::new().unwrap();

    // Verify multiple calls work correctly
    for i in 1..=5 {
        host.last_logged = None;
        module.caller(i * 10, &mut host).unwrap();
        assert_eq!(
            host.last_logged,
            Some(i * 10),
            "caller call {}: log should be called with {}",
            i,
            i * 10
        );
    }
}
