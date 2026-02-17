//! `herkos-runtime` — Runtime library for herkos transpiled output.
//!
//! This crate is `#![no_std]` by default. It provides:
//! - `IsolatedMemory<const MAX_PAGES: usize>` for Wasm linear memory
//! - `WasmTrap` / `WasmResult<T>` for Wasm trap handling
//! - Trait definitions for capability-based host imports (Phase 3+)

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

/// WebAssembly page size: 64 KiB per the Wasm specification.
pub const PAGE_SIZE: usize = 65536;

mod memory;
pub use memory::IsolatedMemory;

mod table;
pub use table::{FuncRef, Table};

mod module;
pub use module::{LibraryModule, Module};

/// Wasm execution errors — no panics, no unwinding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmTrap {
    /// Memory access out of bounds.
    OutOfBounds,
    /// Integer division by zero.
    DivisionByZero,
    /// Integer overflow (e.g., `i32.trunc_f64_s` on out-of-range float).
    IntegerOverflow,
    /// Unreachable instruction executed.
    Unreachable,
    /// Indirect call type mismatch (`call_indirect` signature check).
    IndirectCallTypeMismatch,
    /// Table access out of bounds.
    TableOutOfBounds,
    /// Undefined element in table.
    UndefinedElement,
}

/// Result type for Wasm operations — `Result<T, WasmTrap>`.
pub type WasmResult<T> = Result<T, WasmTrap>;

/// Errors that occur during module/memory/table construction.
///
/// These are programming errors in the transpiler, not runtime Wasm traps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructionError {
    /// Initial pages exceeds MAX_PAGES for memory.
    MemoryInitialPagesExceedsMax { initial: usize, max: usize },
    /// Initial size exceeds MAX_SIZE for table.
    TableInitialSizeExceedsMax { initial: usize, max: usize },
}

impl From<ConstructionError> for WasmTrap {
    fn from(_: ConstructionError) -> Self {
        // Construction errors are programming errors, but we map them to
        // OutOfBounds for compatibility with the error propagation chain.
        WasmTrap::OutOfBounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_trap_is_copy() {
        let trap = WasmTrap::OutOfBounds;
        let trap2 = trap; // Copy
        assert_eq!(trap, trap2);
    }

    #[test]
    fn wasm_result_ok() {
        let result: WasmResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn wasm_result_err() {
        let result: WasmResult<i32> = Err(WasmTrap::DivisionByZero);
        assert!(result.is_err());
        assert_eq!(result, Err(WasmTrap::DivisionByZero));
    }
}
