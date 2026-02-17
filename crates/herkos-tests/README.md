# Test Cases

This directory previously held generated .wasm files. Now, all test WASM binaries are generated directly in `build.rs` and written to `OUT_DIR`.

## Test Cases

### Milestone 1: Pure Math (Current)

The following test cases are defined as inline WAT sources in `build.rs`:

1. **add.wasm** - Simple addition: `(a + b)`
2. **sub.wasm** - Subtraction: `(a - b)`
3. **mul.wasm** - Multiplication: `(a * b)`
4. **const_return.wasm** - Return constant: `42`
5. **nop.wasm** - No-op function

### Future Milestones

- **Milestone 2**: Memory operations (load, store)
- **Milestone 3**: Control flow (if, loop, block)
- **Milestone 4**: Multi-function modules, imports/exports

## How It Works

1. `build.rs` contains inline WAT (WebAssembly Text) sources for each test case
2. The `wat` crate compiles WAT → WASM in memory
3. Generated `.wasm` files are written to `OUT_DIR`
4. The `herkos` CLI transpiles each `.wasm` to Rust
5. Transpiled `.rs` modules are written to `OUT_DIR`
6. Tests include the generated modules via `include!(concat!(env!("OUT_DIR"), "/mod.rs"))`

All generation happens at build time. No files are committed to the repo.
