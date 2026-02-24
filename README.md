# herkos

> ⚠️ **This project is work in progress! Not all wasm features nor corner cases were tested!**

A compilation pipeline that transpiles WebAssembly modules into memory-safe Rust code with compile-time isolation guarantees (memory+capabilities), replacing runtime hardware-based memory protection (MMU/MPU) with type-system-enforced safety.

herkos approach: if the Rust compiler accepts the transpiled code, isolation is guaranteed; no MMU, no context switches, no runtime overhead for proven accesses.

**WebAssembly → Rust source → Safe binary**

## Motivation

Running untrusted or unsafe-language components alongside safe code usually requires hardware isolation (MMU/MPU, hypervisors) or process boundaries, all of which add runtime overhead and complexity. What if the compiler itself could enforce "spatial" isolation?

herkos explores this idea: transpile WebAssembly modules into safe Rust, so that memory isolation and capability restrictions are checked at compile time rather than at runtime. This opens up several use cases:

- **Isolating untrusted components** — sandbox C/C++ libraries without hardware protection
- **Porting unsafe-language code to Rust** — use Wasm as an intermediate representation to get a safe Rust version of existing C/C++ code
- **Efficient cross-partition communication** — how do "compile-time-MMU" partitions talk to each other efficiently?

## Architecture

The project is a Rust workspace with three core crates:

| Crate | Purpose |
|---|---|
| `herkos` | CLI transpiler: parses `.wasm` binaries, emits Rust source code |
| `herkos-runtime` | `#![no_std]` runtime library shipped with transpiled output (isolated memory, capability types, wasm operations) |
| `herkos-tests` | collection of wat/Rust/C sources that are compiled to .wasm, transpiled and tested. |

## Build and test

```bash
cargo build                    # build all crates
cargo test                     # run all tests
cargo clippy --all-targets     # lint
cargo fmt --check              # format check
```

Run a single crate's tests:

```bash
cargo test -p herkos
cargo test -p herkos-runtime
cargo test -p herkos-tests
```

## Usage

```bash
cargo run -p herkos -- input.wasm --output output.rs
```

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE))
