# Changelog

All notable changes to the herkos project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Pre-open-source code review and cleanup
- Apache-2.0 license
- Community files (CONTRIBUTING.md, CHANGELOG.md)
- Cargo.toml metadata for all crates
- GitHub issue and PR templates

### Fixed
- i32 shift operations now correctly mask shift amounts to 5 bits (& 31) per WebAssembly spec
- Replaced panic-inducing `unwrap()` calls in IR builder with proper error handling
- Changed constructor panics to `Result` types for proper no_std compliance

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

- **0.1.0** (2026-02-16) — Initial release with safe backend, basic transpilation, and import/export support

[Unreleased]: https://github.com/YOUR_ORG/herkos/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/YOUR_ORG/herkos/releases/tag/v0.1.0
