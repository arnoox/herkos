# Changelog

All notable changes to the herkos project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0]

### Added
- Bulk memory operations: `memory.fill`, `memory.init`, `data.drop`
- Version info in generated code and module metadata
- Inter-module lending tests and examples with automation scripts
- Memory-intensive benchmarks (sorting, Fibonacci implementations)
- New benchmarks for control flow and arithmetic operations
- Optimization control via `HERKOS_OPTIMIZE` environment variable

### Changed
- Memory operations now use `usize` for better type safety
- Refactored host import handling with uniform `Env<H>` API pattern
- Enhanced SSA IR with improved phi-node lowering and branch resolution
- Improved dead code handling in IR builder with live-check methods for terminators
- Restructured `ControlFrame` enum for better control flow handling
- Simplified data segment parsing using zip for segment indexing

### Fixed
- Host parameter now properly handled in `call_indirect` dispatch (issue #19)
- Host parameter now transitively propagated through direct calls (issue #19)
- IR now enforces strict SSA form at compile time with `UseVar`/`DefVar` typing
- Removed panic for unoptimizations in transpile function
- Removed unnecessary crate-type configurations from Cargo.toml

### Removed
- Example C usage and header files from repository
- Herkos-bootstrap example implementation

## [0.1.1] - 2026-03-09

### Fixed
- Improved diagram formatting in README.md
- Updated .gitignore to include Cargo.lock
- Removed unused CLI options
- Updated repository and homepage URLs

### Added
- C to WebAssembly example with Rust transpilation

## [0.1.0] - 2026-02-16

### Added - Milestone 3: Import/Export System
- Import handling with trait-based capability system
- Export function generation
- Module and LibraryModule types for different memory ownership patterns
- Basic import trait codegen for function imports
- Tests for import/export functionality

### Added - Milestone 2: Control Flow
- If/else/then control flow structures
- Block and loop constructs with proper label tracking
- Branch (br, br_if, br_table) operations
- Select operator
- SSA-based IR with proper variable and block management
- Control flow integration tests

### Added - Milestone 1: Arithmetic and Memory Operations
- Memory load/store operations (i32, i64, f32, f64)
- Arithmetic operations (add, sub, mul, div, rem)
- Bitwise operations (and, or, xor, shl, shr, rotl, rotr)
- Comparison operations (eq, ne, lt, gt, le, ge)
- Type conversion operations (wrap, extend, trunc, convert, reinterpret)
- Basic function transpilation
- Safe backend code generation
- End-to-end transpilation tests

### Added - Phase 1: Runtime Core
- `herkos-runtime` crate with `#![no_std]` support
- `IsolatedMemory<const MAX_PAGES: usize>` with compile-time bounds
- `Table` for indirect function calls
- `WasmTrap` error type for runtime errors
- Bounds-checked memory operations (load/store for i32, i64, f32, f64)
- Unchecked memory operations for future verified backend
- Formal verification with Kani proofs (30+ harnesses)
- 100% test coverage for memory and table operations

### Added - Phase 0: Project Structure
- Workspace with three crates: herkos, herkos-runtime
- Parser module using wasmparser crate
- IR module for intermediate representation
- Backend trait and safe backend implementation
- CI/CD with GitHub Actions (build, test, clippy, fmt)
- Comprehensive documentation (SPECIFICATION.md, PLAN.md)

## Project Status

Many tests are missing!

See [docs/FUTURE.md](docs/FUTURE.md) for planned features.

## Version History

- **0.1.1** (2026-03-09) — C integration example and URL updates
- **0.1.0** (2026-02-16) — Initial release with safe backend, basic transpilation, and import/export support

[Unreleased]: https://github.com/YOUR_ORG/herkos/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/YOUR_ORG/herkos/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/YOUR_ORG/herkos/releases/tag/v0.1.0
