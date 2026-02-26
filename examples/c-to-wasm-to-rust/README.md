# C → WebAssembly → Rust Example

This example demonstrates the full herkos pipeline: taking plain C code, compiling it to WebAssembly, and transpiling it to memory-safe Rust.

```
fibonacci.c  ──clang──▶  fibonacci.wasm  ──herkos──▶  src/fibonacci_wasm.rs
                                                             │
                                                       src/main.rs uses it
                                                             │
                                                        cargo build
```

The generated Rust module contains **no unsafe code**. Memory isolation is enforced through the type system at compile time.

## Prerequisites

- **clang** with wasm32 target support (`apt-get install clang lld`)
- **Rust** toolchain (`cargo`)
- **herkos** CLI (`cargo install --path ../../crates/herkos`)

## Usage

```bash
./run.sh          # compile C → Wasm → Rust, then build and run
./run.sh --clean  # remove generated artifacts
```

## What happens

1. **C → Wasm**: `clang` compiles `fibonacci.c` to `fibonacci.wasm` targeting `wasm32-unknown-unknown` in freestanding mode (no libc)
2. **Wasm → Rust**: `herkos` transpiles `fibonacci.wasm` into `src/fibonacci_wasm.rs`, a self-contained Rust module that depends only on `herkos-runtime`
3. **Build & Run**: `cargo run` compiles `src/main.rs` (which includes the generated module) and runs it

## Example output

```
Fibonacci sequence:
  F(0) = 0
  F(1) = 1
  F(2) = 1
  F(3) = 2
  F(4) = 3
  F(5) = 5
  ...
  F(15) = 610

Factorials:
  0! = 1
  1! = 1
  5! = 120
  10! = 3628800
  12! = 479001600

Greatest common divisor:
  gcd(12, 8) = 4
  gcd(100, 75) = 25
  gcd(17, 13) = 1

Arithmetic:
  add(40, 2) = 42
  mul(6, 7) = 42
```

## How the generated code works

The transpiled module exposes each C function as a method on a `WasmModule` struct. Every call returns `WasmResult<T>` (a `Result`): traps like division by zero or illegal memory access become errors instead of panics:

```rust
let mut module = fibonacci_wasm::new().expect("module instantiation failed");

let fib = module.fibonacci(10).expect("fibonacci trapped");
assert_eq!(fib, 55);
```
