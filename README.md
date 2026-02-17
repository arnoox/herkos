# herkos

> ⚠️ *** This project is work in progress! Not all wasm features nor corner cases were tested!***

A compilation pipeline that transpiles WebAssembly modules into memory-safe Rust code with compile-time isolation guarantees, replacing runtime hardware-based memory protection (MMU/MPU) with type-system-enforced safety.

**WebAssembly → Rust source → Safe binary**

## Motivation

Safety-critical standards (ISO 26262, IEC 61508, DO-178C) require **freedom from interference** between software modules of different criticality levels. Today this is achieved via MMU/MPU or hypervisors — approaches that are expensive in performance (10-50% overhead), energy, and certification effort.

herkos takes a different approach: if the Rust compiler accepts the transpiled code, isolation is guaranteed; no MMU, no context switches, no runtime overhead for proven accesses.

## Architecture

The project is a Rust workspace with three core crates:

| Crate | Purpose |
|---|---|
| `herkos` | CLI transpiler: parses `.wasm` binaries, emits Rust source code |
| `herkos-runtime` | `#![no_std]` runtime library shipped with transpiled output (`IsolatedMemory`, capability types, Wasm operations) |

Features:
- compile time isolation
- compile time capability based security access via traits

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

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
