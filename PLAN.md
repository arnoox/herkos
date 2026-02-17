# PLAN.md — herkos Implementation Plan

High-level roadmap. Each phase produces something testable. We go step by step.

---

## Phase 0: Workspace Scaffold ✅
- [x] Cargo workspace with three crates: `herkos-runtime`, `herkos`
- [x] `herkos-runtime` is `#![no_std]`, others are `std`
- [x] CI basics: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`
- [x] Minimal types compile (`WasmTrap`, `WasmResult<T>`)

## Phase 1: Runtime Core (`herkos-runtime`) ✅
- [x] `IsolatedMemory<const MAX_PAGES: usize>` — backing array, `active_pages`, `grow`
- [x] `load<T>` / `store<T>` (bounds-checked), `load_unchecked<T>` / `store_unchecked<T>`
- [x] Outline pattern for load/store internals (§13.3 — non-generic core)
- [x] `Table` struct, `FuncRef` (indirect calls); globals are plain typed fields in transpiler-generated structs
- [x] `WasmTrap` enum, `WasmResult<T>` type alias
- [x] `Module<G, MAX_PAGES, TABLE_SIZE>` and `LibraryModule<G, TABLE_SIZE>` structs + basic impls
- [x] Kani proof harnesses for core invariants

## Phase 2: Minimal Transpiler (`herkos`) — Safe Backend Only ✅

**Milestone 1: Hello World (Pure Math)** ✅ **COMPLETE**
- [x] Set up `herkos` crate structure (parser, ir, backend, codegen modules)
- [x] Parser: wrap `wasmparser` to extract `ParsedModule`
- [x] IR types: `IrFunction`, `IrBlock`, `IrInstr`, `IrTerminator`
- [x] IR builder: translate simple Wasm bytecode (`local.get`, `i32.add`, `return`) to IR
- [x] SafeBackend: implement `emit_binop` for arithmetic operations
- [x] Codegen: emit standalone Rust function from IR
- [x] End-to-end test: `fn add(a: i32, b: i32) -> i32` compiles and runs

**Milestone 2: Memory Operations** ✅ **COMPLETE**
- [x] IR: add `Load` and `Store` instructions
- [x] SafeBackend: implement `emit_load` / `emit_store` (bounds-checked)
- [x] Codegen: emit memory parameter in function signature
- [x] IrBuilder: translate Wasm load/store operators
- [x] Parser: extract memory section
- [x] End-to-end test: store/load roundtrip (16 tests added)

**Milestone 3: Control Flow** ✅ **COMPLETE**
- [x] IR builder: handle `block`, `loop`, `if`, `else`, `br`, `br_if`, `br_table`, `drop`
- [x] Build multi-block CFG with state machine pattern
- [x] SafeBackend: emit state machine with loop+match+continue
- [x] End-to-end test: conditional logic, loops, nesting (23 new tests)

**Milestone 4: Module Wrapper** ✅ **COMPLETE**
- [x] Parser: parse `GlobalSection`, `ExportSection`, `DataSection`
- [x] IR: add `GlobalGet` / `GlobalSet` instruction variants
- [x] Backend: `emit_global_get` / `emit_global_set` (mutable → `globals.gN`, immutable → `GN`)
- [x] Codegen: `ModuleInfo` struct, newtype `WasmModule(Module<G, MAX_PAGES, 0>)` wrapper
- [x] Codegen: `Globals` struct for mutable globals, `pub const GN` for immutable globals
- [x] Codegen: `::new()` constructor with data segment byte-by-byte `store_u8` init
- [x] Codegen: `impl WasmModule` block with exported functions as methods
- [x] Backwards compatible: wrapper only when mutable globals or data segments exist
- [x] End-to-end tests: 5 new e2e tests (mutable global, data segment, immutable global, backwards compat, combined)
- [x] WAT integration tests: `counter`, `hello_data`, `const_global` (8 runtime tests)

**Milestone 5: Complete Safe Backend** ✅ **COMPLETE**
- [x] All Wasm integer operations (i32, i64) — all arithmetic, bitwise, shifts, rotations, comparisons, unary (clz/ctz/popcnt/eqz)
- [x] All Wasm float operations (f32, f64) — arithmetic, comparisons, rounding (ceil/floor/trunc/nearest), abs/neg/sqrt, min/max/copysign
- [x] All conversion operations (23 ops) — wrap, extend, trunc (trapping), convert, demote, promote, reinterpret
- [x] All memory operations (load/store with different alignments)
- [x] `memory.grow` / `memory.size`
- [x] `select` (conditional ternary operator)
- [x] `unreachable` (trap instruction)
- [x] `return` (explicit mid-function return with dead-code continuation)
- [x] CLI: `--mode safe`, `--output`, `--max-pages`
- [x] End-to-end test: compile Rust (no_std) → Wasm → Rust → run (3 modules, 38 tests)
- [x] End-to-end test: compile C → Wasm → Rust → run (4 modules, 53 tests)
- [x] File-based test sources in `data/` directory (WAT, Rust, C scanned by build.rs)

## Phase 3: Imports, Exports, and Traits

Wasm imports become Rust trait bounds. Each import module name (e.g., `"env"`) maps to one Rust
trait. The host implements these traits to provide capabilities. If a module doesn't import a
capability, there's no trait bound — zero-cost via monomorphization. (SPECIFICATION.md §5)

Exports are already implemented (parser + codegen). Imports are not implemented at all yet.
Runtime types `Module` and `LibraryModule` already exist with the right patterns.

**Critical technical challenge:** Wasm puts imported functions *before* local functions in the
index space. A module with 3 imported functions has its first local function at index 3. Every
`call`, export index, and element segment reference uses this global numbering.

**Milestone 1: Import Parsing + Index Space Correction** ✅ **COMPLETE**
- [x] Parser: add `ImportInfo`, `ImportKind` types; add `imports`, `num_imported_functions`, `num_imported_globals` to `ParsedModule`; handle `Payload::ImportSection`
- [x] `transpile()`: build combined `func_sigs` list (imports first, then locals); offset export indices by `num_imported_functions`; offset element segment indices
- [x] Codegen: add `num_imported_functions` to `ModuleInfo`; adjust `call_indirect` dispatch; error on `call` to imported function (not yet supported)
- [x] All 353 existing tests pass (zero imports → no behavioral change) — 356 tests now passing
- [x] New parser unit tests: `parse_function_import`, `parse_mixed_imports`

**Milestone 2: Import Call Codegen (`CallImport` + Host Parameter)** ✅ **COMPLETE**
- [x] IR: add `IrInstr::CallImport { dest, import_idx, module_name, func_name, args }`
- [x] IR builder: `call N` where N < num_imports → `CallImport`, else `Call` with adjusted index
- [x] Backend: add `emit_call_import` → `host.func_name(args)?`
- [x] Codegen: add `host: &mut H` parameter to function signatures when imports exist; export methods forward host
- [x] Host is passed as parameter (not stored in struct) to avoid lifetime issues
- [x] Test: function with import generates `host.func_name()` calls and host parameter (357 tests passing)

**Milestone 3: Host Trait Generation** ✅ **COMPLETE**
- [x] Codegen: group imports by module name → one trait per module (e.g., `pub trait EnvImports`)
- [x] Naming: `"env"` → `EnvImports`, `"wasi_snapshot_preview1"` → `WasiSnapshotPreview1Imports`
- [x] Export methods get `<H: TraitA + TraitB>` generic bounds (uses generic parameters for multiple bounds)
- [x] New WAT + runtime tests: transpile module with imports, mock host, verify import invocation (3 tests in import_traits.rs)
- [x] Helper functions: `module_name_to_trait_name()` and `group_imports_by_module()`
- [x] Type inference for CallImport instructions (return type tracking)
- [x] Export filtering to exclude imported functions
- [x] FuncImport type added to ModuleInfo for trait generation
- [x] All 260+ tests passing

**Milestone 4: Memory and Global Imports**
- [ ] Memory import → emit `LibraryModule<G, TABLE_SIZE>` instead of `Module<G, MAX_PAGES, TABLE_SIZE>`; export methods take `memory: &mut IsolatedMemory<MP>`
- [ ] Global import → emit `host.get_name()` / `host.set_name(val)` via trait accessors
- [ ] Detect memory import: no local MemorySection but `ImportKind::Memory` exists

**Milestone 5: End-to-End Integration and Inter-Module Lending**
- [ ] WAT files: `import_basic.wat`, `import_memory.wat`, `import_multi.wat`
- [ ] Runtime tests with mock host impls verifying import invocation
- [ ] Memory lending: Module A owns memory, LibraryModule B borrows it
- [ ] Mixed local + import calls: correct dispatch (import → `host.f()`, local → `func_N()`)
- [ ] `cargo test` + `cargo clippy --all-targets` + `cargo fmt --check` all clean

Dependency chain: M1 → M2 → M3 → M4 → M5 (each milestone strictly depends on the previous)

## Phase 4: WASI Support
- [ ] WASI trait definitions in runtime (`WasiFd`, `WasiPath`, `WasiClock`, `WasiRandom`)
- [ ] Transpiler recognizes WASI import names → maps to WASI traits
- [ ] Reference host implementation (at least `fd_write`, `fd_read`, `proc_exit`)
- [ ] End-to-end test: C `printf` hello world → Wasm → Rust → run

## Phase 5: Performance and Optimization
- [ ] Runtime performance benchmarks (generated code performance)
  - [ ] Benchmark suite comparing generated Rust vs hand-written Rust vs native Wasm runtime
  - [ ] State machine overhead measurement (compare to theoretical minimum)
  - [ ] Hot path profiling (perf/flamegraph) on real workloads
  - [ ] Memory access pattern analysis
  - [ ] Branch prediction effectiveness on state machine dispatches
- [ ] Transpilation performance optimization
  - [ ] Parallel transpilation (rayon-based, heuristic activation for large modules)
  - [ ] Transpilation performance benchmarks (CI metrics)
- [ ] Code quality and hardening
  - [ ] `#[inline(never)]` on generated functions (prevent code bloat)
  - [ ] MAX_PAGES normalization to standard sizes
  - [ ] Trait object fallback for cold paths
  - [ ] C ABI wrapper generation (optional)
  - [ ] Proof coverage reports
  - [ ] Documentation and examples

---

## DEFERRED: Technical Debt and Code Quality Improvements

These items were identified during pre-open-source code review and deferred for future milestones:

### Phase 4: Important Features and Spec Compliance
- [ ] **Local variable tracking**: Implement proper local variable handling in IR builder
  - Currently stubbed at [builder.rs:209](crates/herkos/src/ir/builder.rs#L209) with TODO
  - Parse local declarations from function bodies (available in wasmparser)
  - Track locals in `IrFunction.locals: Vec<WasmType>`
  - Map Wasm local indices to VarIds correctly (params first, then locals)
  - Add E2E test for functions with explicit locals
- [ ] **IR type system cleanup**: Remove Rust-specific Display implementations from IR types
  - Files: [types.rs:40-50, 274-284](crates/herkos/src/ir/types.rs)
  - Move formatting logic to backend-specific code (e.g., `backend/safe.rs`)
  - Maintain backend-agnostic IR per SPECIFICATION.md §2
- [ ] **Multi-value blocks**: Document limitation or implement support
  - Currently rejected at [builder.rs:713, 732, 753](crates/herkos/src/ir/builder.rs)
  - Add "Known Limitations" section to README.md and SPECIFICATION.md
  - WebAssembly 1.0 spec requires multi-value support for full compliance

### Phase 5: Documentation Enhancements
- [ ] **Expand API documentation**: Achieve 100% rustdoc coverage for public APIs
  - Current: Only ~30 files have rustdoc comments
  - Priority files: all `pub` types in herkos-runtime, herkos/lib.rs public functions
  - Add module-level documentation for backend/, ir/, parser/, codegen/
- [ ] **Add examples directory**: Create `examples/` with working code samples
  - `examples/basic_arithmetic.rs` - Transpile and run simple arithmetic Wasm
  - `examples/memory_isolation.rs` - Demonstrate IsolatedMemory usage
  - `examples/capability_system.rs` - Show import trait pattern
  - Include corresponding `.wat` source files

### Phase 6: Technical Debt and Polish
- [ ] **Refactor dead block pattern**: Add IR cleanup pass or document pattern
  - Dead blocks created after Return/Br waste block IDs [builder.rs:468-469](crates/herkos/src/ir/builder.rs#L468-L469)
  - Either add post-processing cleanup or add comments explaining necessity
- [ ] **Store Wasm version and transpiler version in generated code**: Emit version as comment
  - TODO comment at [parser/mod.rs](crates/herkos/src/parser/mod.rs)
  - Add version field to `ParsedModule`
  - Emit as comment: `// Generated by herkos from WebAssembly version 1`
- [ ] **Enhanced error messages**: Replace generic bail!() with specific error types
  - Better debugging experience for users
  - More informative transpilation failures
- [ ] **Add badges to README**: CI status, crates.io version, license, docs.rs
  - Wait until after first crates.io publication

---

**Current phase**: Pre-open-source cleanup complete (Phases 1-3 of review), Phase 3 Milestone 3 complete in main implementation, ready for Milestone 4
