# CLAUDE.md — herkos

## Project overview

`herkos` is a compilation pipeline that transpiles WebAssembly modules into memory-safe Rust code with compile-time isolation guarantees. The goal is to replace runtime hardware-based memory protection (MMU/MPU) with type-system-enforced safety.

The pipeline: **WebAssembly → Rust source → Safe binary**

See the `docs/` directory for the full draft specification.

## Repository structure

The project is a Rust workspace with three crates:

- `crates/herkos/` — CLI transpiler: parses `.wasm` binaries and emits Rust source code
- `crates/herkos-runtime/` — Runtime library shipped with transpiled output (`IsolatedMemory`, capability types, Wasm operations)
- `crates/herkos-tests/` — Integration test library

## Build and test

```bash
cargo build                    # build all crates
cargo test                     # run all tests
cargo clippy --all-targets     # lint (CI enforced)
cargo fmt --check              # format check (CI enforced)
```

Run a single crate's tests:
```bash
cargo test -p herkos
cargo test -p herkos-runtime
cargo test -p herkos-tests
```

Build documentation (sphinx):
```bash
cd docs && uv run make clean && uv run make html    # build Sphinx documentation
```

CLI usage (once built):
```bash
cargo run -p herkos -- input.wasm --mode hybrid --output output.rs --metadata verification.toml
```

## Key architectural concepts

### Three code generation backends

The transpiler can emit code in three distinct styles, trading off between auditability, performance, and proof requirements:

1. **Safe** (default)
   - **Overhead**: 15–30%
   - **`unsafe` in output**: None
   - **Proofs required**: None
   - Runtime bounds checks on every memory and arithmetic operation. Suitable for initial migration, testing, and modules where performance is not critical.

2. **Verified**
   - **Overhead**: 0–5%
   - **`unsafe` in output**: All memory accesses
   - **Proofs required**: Complete (all operations must have formal proofs)
   - `unsafe` operations justified by formal proofs from `wasm-verify`. Transpilation fails if any operation lacks a proof. Target for performance-critical modules.

3. **Hybrid**
   - **Overhead**: 5–15%
   - **`unsafe` in output**: Proven accesses only
   - **Proofs required**: Partial
   - Mix of `unsafe` unchecked operations (where proofs exist) and safe runtime checks (where they don't). Practical choice for production — enables iterative proof improvement.

### Memory model

Wasm linear memory is represented as a page-level const generic:

```rust
struct IsolatedMemory<const MAX_PAGES: usize> {
    pages: [[u8; PAGE_SIZE]; MAX_PAGES],  // Fully allocated at compile time, contiguous
    active_pages: usize,                  // Current live size (starts at initial_pages)
}
```

**Design decisions**:
- **`MAX_PAGES` const generic**: Derived from the Wasm module's declared maximum. Prevents any heap allocation (`no_std` compatible). Enables monomorphization for zero-cost dispatch.
- **`active_pages` tracking**: Starts at the module's initial page count. Incremented by `memory.grow`. Accesses beyond `active_pages * PAGE_SIZE` trap.
- **2D layout** `[[u8; PAGE_SIZE]; MAX_PAGES]`: Avoids `generic_const_exprs` (unstable Rust feature). Contiguous in memory. `as_flattened()` provides flat `&[u8]` views.

**Memory access API**:
- **Safe**: `load_i32(offset) -> WasmResult<i32>` (bounds-checked)
- **Verified**: `unsafe load_i32_unchecked(offset) -> i32` (proof-justified)

No `MemoryView` wrappers — the API is flat and fast. See SPECIFICATION.md §3 for complete details.

### Module representation

Two kinds of modules:

1. **`Module<G, MAX_PAGES, TABLE_SIZE>`** — Owns its own memory (like a process)
2. **`LibraryModule<G, TABLE_SIZE>`** — Borrows memory from caller (like a shared library)

Each has its own:
- **Globals struct** `G`: One typed field per mutable Wasm global; immutable globals are `const`
- **Table**: Indirect call table with function references (for `call_indirect`)

Modules can call each other via trait-bounded functions. Memory ownership enforces spatial isolation — one module's memory is inaccessible to others.

### Capability-based security via traits

**Imports** (what a module needs) become Rust trait bounds on generic host parameter `H`.
**Exports** (what a module provides) become trait implementations.

Example:
```rust
// Module imports socket functions → requires SocketOps trait
fn send<H: SocketOps + FileOps>(host: &mut H, ...) -> WasmResult<i32> { ... }

// Module provides transform export → implements ImageLibExports
impl ImageLibExports for ImageModule<MAX_PAGES> { ... }
```

**Zero-cost**: All dispatch via monomorphization (no vtables, no trait objects). If a module doesn't import a capability, the trait bound doesn't exist — no code path to call it.

WASI support is built-in via standard traits (`WasiFd`, `WasiPath`, `WasiClock`, `WasiRandom`, etc.) shipped with `herkos-runtime`. See SPECIFICATION.md §5 for complete details.

### Verification metadata and `wasm-verify`

The `wasm-verify` crate performs static analysis on `.wasm` binaries:

**Analysis categories**:
- **Memory bounds**: Prove a load/store address is within `[0, active_pages * PAGE_SIZE)`
- **Arithmetic overflow**: Prove integer operations cannot overflow for proven value ranges
- **Read-only regions**: Prove no store instruction targets a given address range (e.g., data segments)
- **Stack frame isolation**: Prove a function only accesses its own stack frame
- **Stack bounds**: Prove the stack pointer stays within the stack region

**Verification approach** (two-phase):
1. **Abstract interpretation** (fast): Tracks value ranges through control flow. Resolves most proof obligations without external solver.
2. **SMT solver** (precise): Unresolved obligations encoded as bitvector constraints. Z3 or Bitwuzla produce formal proofs or counterexamples.

**Output**: TOML metadata file with proof artifacts indexed by instruction offset. The transpiler verifies the metadata's source hash and uses proofs to decide per-access whether to emit safe or `unsafe` code.

See SPECIFICATION.md §7 for the complete metadata schema and §11.2 for solver details.

## `no_std` constraint

`herkos-runtime` and all transpiled output **must be `#![no_std]`**. No heap allocation without the optional `alloc` feature gate. No panics, no `format!`, no `String` in the runtime or generated code. Errors are `Result<T, WasmTrap>` only. The `herkos` CLI and `wasm-verify` crates are standard `std` binaries — this constraint applies only to the runtime and generated output.

## Coding conventions

- **Rust edition**: 2021
- **MSRV**: latest stable
- **Error handling**: use `thiserror` for library errors, `anyhow` in CLI binaries. Wasm execution errors use the `WasmTrap` / `WasmResult<T>` types from the runtime crate (no panics, no unwinding).
- **Naming**: follow Rust API guidelines. Wasm spec terminology (e.g., `i32.load`, `br_table`) maps to snake_case Rust (`i32_load`, `br_table`).
- **Unsafe**: avoid `unsafe` in the runtime crate as much as possible — the whole point is compile-time safety. Any `unsafe` block requires a `// SAFETY:` comment explaining the invariant. In the verified backend output, `unsafe` blocks carry `// PROOF:` references to verification metadata instead.
- **Tests**: unit tests live in `#[cfg(test)] mod tests` inside each module. Integration tests go in `tests/` directories per crate. End-to-end tests (Wasm → Rust → run) go in `tests/e2e/` at the workspace root.
- **Dependencies**: keep the dependency tree small. Prefer `wasmparser` for Wasm parsing. Avoid pulling in a full Wasm runtime — we are the runtime. `herkos-runtime` must have zero dependencies in the default (no `alloc`) configuration.

## Function calls and indirect dispatch

### Direct calls (`call`)
Transpile to regular Rust function calls with shared state (memory, globals, table) threaded through:
```rust
v5 = func_3(&mut memory, &mut globals, v3, v4)?;
```

### Indirect calls (`call_indirect`)
Implement safe dispatch via match statements. The transpiler:
1. Looks up the table entry and validates its type
2. Enumerates all functions matching the expected signature
3. Emits a static match statement (becomes jump table in machine code)
4. **No function pointers, no vtables, no dynamic dispatch** — 100% safe

Example:
```rust
let __entry = table.get(idx)?;
if __entry.type_index != canonical_type {
    return Err(WasmTrap::IndirectCallTypeMismatch);
}
v4 = match __entry.func_index {
    0 => func_0(v0, v1),
    1 => func_1(v0, v1),
    _ => return Err(WasmTrap::UndefinedElement),
};
```

See SPECIFICATION.md §8.5 for structural type equivalence and table initialization details.

## Integration patterns

### Primary: Trait-based (Recommended)
Host instantiates modules and provides capabilities through trait implementations:
```rust
struct MyHost { /* platform resources */ }
impl SocketOps for MyHost { /* ... */ }
impl WasiFd for MyHost { /* ... */ }

let mut module = Module::<MyGlobals, 256, 4>::new(16, MyGlobals::default(), table);
let result = module.process_data(&mut MyHost::new(), ptr, len)?;
```
**Benefits**: Full type safety, zero `unsafe`, zero-cost dispatch via monomorphization.

### Alternative: C-Compatible ABI
Optional `extern "C"` wrapper for integration with non-Rust systems. Erases generics using opaque types. See SPECIFICATION.md §10.2.

## Performance considerations

### Monomorphization bloat
Each distinct `MAX_PAGES` and trait bound combination generates separate code. **Mitigation**:

1. **Outline pattern** (critical): Move logic into non-generic inner functions that take sizes as runtime parameters. Generic wrapper is a thin shell.
   ```rust
   #[inline(never)]
   fn load_i32_inner(memory: &[u8], active_bytes: usize, offset: usize) -> WasmResult<i32> { ... }

   #[inline(always)]
   fn load_i32<const MAX_PAGES: usize>(mem: &IsolatedMemory<MAX_PAGES>, offset: usize) -> WasmResult<i32> {
       load_i32_inner(mem.pages.as_flattened(), mem.active_pages * PAGE_SIZE, offset)
   }
   ```

2. **Normalize `MAX_PAGES`**: Use standard sizes (16, 64, 256, 1024) instead of exact declared maximums.

3. **Trait objects for cold paths**: Use `&mut dyn Trait` instead of generics for rarely-called code.

4. **LTO**: Link-time optimization eliminates unreachable monomorphized copies.

See SPECIFICATION.md §13.3 for complete strategies.

### Parallelization
The transpilation pipeline can be parallelized. IR building and code generation are embarrassingly parallel (each function is independent). The transpiler should use `rayon` for parallel iteration when processing modules with 20+ functions. See SPECIFICATION.md §13.5 for implementation details and performance expectations.

## Wasm parsing and code generation

### Parsing
Use the `wasmparser` crate for reading `.wasm` binaries. Do NOT use `wasm-tools` or `walrus` unless there is a clear justification. The transpiler should operate on the structured output of `wasmparser` (types, functions, memory sections, etc.) and emit Rust source via string building or a small codegen IR — not via `syn`/`quote` procedural macro machinery.

### Generated Rust output
Transpiled code should be:
- Self-contained (only depends on `herkos-runtime`)
- Formatted (run through `rustfmt`)
- Readable and auditable — prefer clarity over compactness
- Include `// PROOF:` comments referencing verification metadata (verified/hybrid backends)
- No panics, no unwinding — use `Result<T, WasmTrap>` for error handling

## Future extensions

### Temporal isolation via fuel-based execution (§14.3 of SPECIFICATION.md)
Prevent modules from starving each other of CPU time through instrumentation at loop headers and function calls. Three levels:
- **Fuel-checked (safe)**: Runtime fuel decrements at loop back edges (~3–5% overhead)
- **WCET-proven (verified)**: Static loop bounds eliminate fuel checks (0% overhead)
- **Hybrid**: Mix of proven and checked loops (1–3% overhead)

This mirrors the spatial isolation (memory bounds) model exactly and uses the same `wasm-verify` infrastructure.

## PR and commit guidelines

- Keep commits focused: one logical change per commit
- Commit messages: imperative mood, short summary line, body if needed
- All PRs must pass `cargo test`, `cargo clippy`, and `cargo fmt --check`
- See docs/SPECIFICATION.md for the full technical specification of all components
