# herkos

[![CI](https://github.com/arnoox/herkos/actions/workflows/ci.yml/badge.svg)](https://github.com/arnoox/herkos/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/herkos.svg)](https://crates.io/crates/herkos)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![no_std](https://img.shields.io/badge/no__std-compatible-green.svg)]()

**Compile-time memory isolation via WebAssembly-to-Rust transpilation.**

herkos transpiles WebAssembly modules into safe Rust code, offering type-system-enforced isolation as a lightweight alternative to hardware-based memory protection (MMU/MPU). When hardware isolation is overkill or simply unavailable, herkos lets the Rust compiler enforce the guarantees instead: no MMU, no context switches, no runtime overhead for proven accesses.

> **Status:** Work in progress. Not all Wasm features or corner cases are covered. Do not use in production.

## The idea

Running untrusted or unsafe-language code usually requires hardware isolation (processes, hypervisors, MMU/MPU). That's the right tool for many jobs — but sometimes it's overkill: a small plugin, a single C library, a microcontroller with no MMU. For those cases, herkos offers a lighter-weight approach: transpile Wasm to safe Rust, so the compiler itself enforces spatial isolation.

```
                                Compile-time isolation
                               ┌──────────────────────┐
  C / C++ / Rust    ──────▶    │   .wasm   ──herkos──▶│   Safe Rust    ──rustc──▶   Binary
  (unsafe code)      clang/    │                      │   (no unsafe)              (isolated)
                     rustc     └──────────────────────┘
```

This enables:
- **Sandboxing C/C++ libraries** without hardware protection: isolate a JPEG decoder, a crypto library, or a plugin system
- **Porting unsafe code to Rust**: use Wasm as an IR to get a safe Rust version of existing C/C++ code
- **Bare-metal isolation**: `#![no_std]`, no heap, no OS required. Run isolated components on microcontrollers

## Example: C to safe Rust in 3 steps

**1. Write C code** (or any language that compiles to Wasm):
```c
int fibonacci(int n) {
    if (n <= 0) return 0;
    if (n == 1) return 1;
    int a = 0, b = 1, i = 2;
    while (i <= n) { int tmp = a + b; a = b; b = tmp; i++; }
    return b;
}
```

**2. Compile to Wasm, then transpile:**
```bash
clang --target=wasm32 -O2 -nostdlib -Wl,--no-entry -Wl,--export-all -o fibonacci.wasm fibonacci.c
herkos fibonacci.wasm --output fibonacci.rs
```

**3. Use from Rust — every call returns `Result`, traps become errors:**
```rust
mod fibonacci;

fn main() {
    let mut module = fibonacci::new().expect("init failed");
    let result = module.fibonacci(10).unwrap();
    assert_eq!(result, 55);
}
```

The generated `fibonacci.rs` contains **no `unsafe`**, **no function pointers**, **no heap allocations**. Memory access is bounds-checked, division by zero returns `Err(WasmTrap::DivisionByZero)`, and the module's memory is structurally isolated from the rest of your program.

See the full [C-to-Wasm-to-Rust example](examples/c-to-wasm-to-rust/).

## How it works

```
.wasm ─▶ Parser ─▶ IR Builder ─▶ Optimizer ─▶ Backend ─▶ Codegen ─▶ rustfmt ─▶ output.rs
         wasmparser   SSA-form    dead block    safe       Rust
                      IR          elimination   backend    source
```

The transpiler parses a `.wasm` binary, builds an SSA-form intermediate representation, optimizes it, and emits Rust source code through a safe backend. The output depends only on the `herkos-runtime` crate.

### Generated code structure

Each Wasm module becomes a Rust struct that owns its isolated memory:

```rust
pub struct WasmModule(pub Module<Globals, MAX_PAGES, TABLE_SIZE>);

impl WasmModule {
    pub fn fibonacci(&mut self, n: i32) -> WasmResult<i32> { ... }
    pub fn add(&mut self, a: i32, b: i32) -> WasmResult<i32> { ... }
}
```

- **Memory** — `IsolatedMemory<MAX_PAGES>`: a fixed-size 2D array, fully stack/BSS allocated, no heap. Every load/store is bounds-checked
- **Imports** — become trait bounds: `fn send<H: SocketOps>(host: &mut H, ...)`. No imports = no host parameter
- **Exports** — become methods on the module struct
- **Indirect calls** — safe `match` dispatch over function indices. No function pointers, no vtables
- **Errors** — `WasmResult<T> = Result<T, WasmTrap>`. No panics, no unwinding

### Security properties

| Property | How |
|----------|-----|
| No buffer overflows | Every memory access is bounds-checked against `active_pages * PAGE_SIZE` |
| No cross-module access | Each module owns a distinct `IsolatedMemory`; the type system prevents cross-access |
| No unauthorized syscalls | Imports are trait bounds — you can't call `socket_open` unless the host implements `SocketOps` |
| No ROP gadgets | No function pointers in generated code; indirect calls use static match dispatch |
| No panics | All errors are `Result<T, WasmTrap>`; no unwinding |

## Quick start

### Install

```bash
cargo install herkos
```

Or from source:

```bash
git clone https://github.com/arnoox/herkos.git
cd herkos
cargo install --path crates/herkos
```

### Transpile

```bash
herkos input.wasm --output output.rs
```

### Use from `build.rs` (compile-time pipeline)

```rust
let wasm_bytes = std::fs::read("module.wasm").unwrap();
let rust_code = herkos::transpile(&wasm_bytes, &herkos::Options::default()).unwrap();
std::fs::write(out_dir.join("module.rs"), rust_code).unwrap();
```

## Project structure

| Crate | Purpose | `no_std` |
|-------|---------|----------|
| [`herkos`](crates/herkos/) | CLI transpiler: `.wasm` binary in, Rust source out | No |
| [`herkos-runtime`](crates/herkos-runtime/) | Runtime library shipped with transpiled output | Yes |
| [`herkos-tests`](crates/herkos-tests/) | Integration tests + benchmarks | No |

## Build and test

```bash
cargo build                    # build all crates
cargo test                     # run all tests
cargo clippy --all-targets     # lint
cargo fmt --check              # format check
cargo bench -p herkos-tests    # benchmarks
```

## Documentation

- [Requirements](docs/REQUIREMENTS.md) — formal requirements (REQ_* IDs)
- [Specification](docs/SPECIFICATION.md) — architecture, transpilation rules, memory model, security analysis
- [Future work](docs/FUTURE.md) — verified backend, hybrid backend, temporal isolation

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).
