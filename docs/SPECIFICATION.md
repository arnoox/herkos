# Specification

This document specifies the design and behavior of the herkos transpilation pipeline. It takes the goals and constraints defined in [REQUIREMENTS.md](REQUIREMENTS.md) as input and describes **how** they are achieved.

Where the requirements say *what* the system must do, this specification says *how* it does it.

For features that are planned but not yet implemented (verified/hybrid backends, temporal isolation, etc.), see [FUTURE.md](FUTURE.md).

**Document Status**: Draft — Version 0.3 — 2026-04-12

---

## Table of Contents

1. [Getting Started](#1-getting-started)
2. [Module Representation](#2-module-representation)
3. [Architecture](#3-architecture)
4. [Transpilation Rules](#4-transpilation-rules)
5. [Integration](#5-integration)
6. [Performance](#6-performance)
7. [Security Properties](#7-security-properties)
8. [Open Questions](#8-open-questions)
9. [References](#9-references)

---

## 1. Getting Started

### 1.1 Installation

> Note: it is currently only possible to build from source.

```bash
git clone https://github.com/anthropics/herkos.git
cd herkos
cargo install --path crates/herkos
```

### 1.2 Basic Usage

```bash
herkos input.wasm --output output.rs
herkos input.wasm -O --output output.rs   # with IR optimizations enabled
```

| Option | Description | Required |
|--------|-------------|----------|
| `input.wasm` | Path to WebAssembly module | Yes |
| `--output`, `-o` | Output Rust file path (defaults to stdout) | No |
| `--optimize`, `-O` | Enable IR optimization passes | No |

> **Current limitations**: Only the `safe` backend is implemented. `--max-pages` is not yet exposed as a CLI flag (defaults to 256 in the library). See [FUTURE.md](FUTURE.md) for the verified and hybrid backend plans.

### 1.3 Understanding the Output

The transpiler produces a self-contained Rust source file that depends only on `herkos-runtime`. The output contains:

```
┌──────────────────────────────────────────────┐
│  Generated output.rs                         │
├──────────────────────────────────────────────┤
│  use herkos_runtime::*;                      │
│                                              │
│  struct Globals { ... }     ← mutable globals│
│  const G1: i64 = 42;       ← immutable       │
│                                              │
│  fn func_0(...) { ... }    ← Wasm functions  │
│  fn func_1(...) { ... }                      │
│                                              │
│  struct Module<MAX_PAGES, TABLE_SIZE> {      │
│      memory: IsolatedMemory<MAX_PAGES>,      │
│      globals: Globals,                       │
│      table: Table<TABLE_SIZE>,               │
│  }                                           │
│                                              │
│  impl Module { ... }       ← exports as      │
│                               methods        │
│  trait ModuleImports { ... }  ← required     │
│                                 capabilities │
└──────────────────────────────────────────────┘
```

### 1.4 Using Transpiled Code

#### Direct inclusion

```rust
use herkos_runtime::{IsolatedMemory, WasmResult};

include!("path/to/output.rs");

fn main() -> WasmResult<()> {
    let mut module = Module::<256, 4>::new(
        16,                      // initial pages
        Globals::default(),      // module globals
        Table::default(),        // call table
    )?;

    let result = module.my_function(42)?;
    println!("Result: {}", result);
    Ok(())
}
```

#### Via build.rs (recommended for automated workflows)

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    println!("cargo:rerun-if-changed=wasm-modules/math.wasm");

    let wasm_bytes = std::fs::read("wasm-modules/math.wasm").unwrap();
    let options = herkos::TranspileOptions::default();
    let rust_code = herkos::transpile(&wasm_bytes, &options).unwrap();
    std::fs::write(out_path.join("math_module.rs"), rust_code).unwrap();
}
```

```rust
// src/main.rs
use herkos_runtime::WasmResult;
include!(concat!(env!("OUT_DIR"), "/math_module.rs"));

fn main() -> WasmResult<()> {
    let mut module = Module::<16, 0>::new(1, Globals::default(), Table::default())?;
    let result = module.add(5, 3)?;
    println!("Result: {}", result);
    Ok(())
}
```

When including multiple modules, wrap them in Rust modules to avoid name collisions:

```rust
mod math {
    include!(concat!(env!("OUT_DIR"), "/math_module.rs"));
}
mod crypto {
    include!(concat!(env!("OUT_DIR"), "/crypto_module.rs"));
}
```

### 1.5 Example: C to Rust via Wasm

```c
// math.c
int add(int a, int b) { return a + b; }
int multiply(int a, int b) { return a * b; }
```

```bash
clang --target=wasm32 -O2 -c math.c -o math.o
wasm-ld math.o -o math.wasm --no-entry
herkos math.wasm --output math.rs
```

```rust
use herkos_runtime::WasmResult;
include!("math.rs");

fn main() -> WasmResult<()> {
    let mut module = Module::<16, 0>::new(1, Globals::default(), Table::default())?;
    println!("2 + 3 = {}", module.add(2, 3)?);
    println!("4 * 5 = {}", module.multiply(4, 5)?);
    Ok(())
}
```

---

## 2. Module Representation

This section describes how WebAssembly concepts map to Rust types. This is the core abstraction layer — everything else (transpilation, integration, performance) builds on these types.

### 2.1 Memory Model

```{spec} Memory Model
:id: SPEC_MEMORY_MODEL
:satisfies: REQ_MEM_PAGE_MODEL, REQ_MEM_COMPILE_TIME_SIZE, REQ_MEM_BOUNDS_CHECKED, REQ_MEM_GROW_NO_ALLOC
:tags: memory

How WebAssembly linear memory maps to `IsolatedMemory<MAX_PAGES>` in Rust.
```

#### 2.1.1 Page Model

WebAssembly linear memory is organized in pages of 64 KiB (65,536 bytes). A Wasm module declares an initial page count and an optional maximum page count.

```
Page size:    64 KiB (defined by the WebAssembly specification)
Initial size: declared in the Wasm module (e.g., 16 pages = 1 MiB)
Maximum size: declared in the Wasm module (e.g., 256 pages = 16 MiB)
```

#### 2.1.2 Rust Representation: `IsolatedMemory<MAX_PAGES>`

> Implementation: [crates/herkos-runtime/src/memory.rs](../crates/herkos-runtime/src/memory.rs)

```rust
const PAGE_SIZE: usize = 65536;

struct IsolatedMemory<const MAX_PAGES: usize> {
    pages: [[u8; PAGE_SIZE]; MAX_PAGES],
    active_pages: usize,
}
```

**Design decisions**:

| Decision | Rationale |
|----------|-----------|
| `MAX_PAGES` const generic | No heap allocation, `no_std` compatible, enables monomorphization |
| `active_pages` runtime tracking | Starts at initial page count, grows via `memory.grow`, bounds-checks against this |
| 2D array `[[u8; PAGE_SIZE]; MAX_PAGES]` | Avoids unstable `generic_const_exprs`. `as_flattened()` provides flat `&[u8]` views (stable Rust 1.80+) |
| No maximum → CLI configurable | If the Wasm module declares no maximum, the transpiler picks a default (configurable via `--max-pages`) |

#### 2.1.3 Memory Access API

All memory operations are flat — no `MemoryView` wrappers. One method per Wasm type, avoiding monomorphization of inner functions:

```rust
impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
    // Safe: bounds-checked against active_pages * PAGE_SIZE
    fn load_i32(&self, offset: usize) -> WasmResult<i32>;
    fn load_i64(&self, offset: usize) -> WasmResult<i64>;
    fn load_u8(&self, offset: usize) -> WasmResult<u8>;
    fn load_u16(&self, offset: usize) -> WasmResult<u16>;
    fn load_f32(&self, offset: usize) -> WasmResult<f32>;
    fn load_f64(&self, offset: usize) -> WasmResult<f64>;
    fn store_i32(&mut self, offset: usize, value: i32) -> WasmResult<()>;
    fn store_i64(&mut self, offset: usize, value: i64) -> WasmResult<()>;
    // ... and store_u8, store_u16, store_f32, store_f64
}
```

Read-only guarantees are not a Wasm primitive — they are an analysis result. Static analysis can prove that certain regions (e.g., `.rodata` data segments) are never targeted by store instructions. This is relevant to the future verified backend (see [FUTURE.md](FUTURE.md)).

#### 2.1.4 `memory.grow` Semantics

```rust
impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
    fn grow(&mut self, delta: u32) -> i32 {
        let old = self.active_pages;
        let new = old.wrapping_add(delta as usize);
        if new > MAX_PAGES {
            return -1;
        }
        for page in &mut self.pages[old..new] {
            page.fill(0);
        }
        self.active_pages = new;
        old as i32
    }
}
```

No allocation occurs. New pages are zero-initialized per the Wasm spec.

#### 2.1.5 Linear Memory Layout

When C/C++ compiles to Wasm, the compiler organizes linear memory into conventional regions:

```
┌─────────────────────────────────────────┐ MAX_PAGES * PAGE_SIZE
│           (unused / growable)           │
├─────────────────────────────────────────┤ ← __stack_pointer (grows ↓)
│           Shadow Stack                  │
│   (local variables, large structs,      │
│    spills, return values)               │
├─────────────────────────────────────────┤
│           Heap (grows ↑)               │
│   (malloc / C++ new)                    │
├─────────────────────────────────────────┤
│           Data Segments                 │
│   (.data, .rodata, .bss)                │
└─────────────────────────────────────────┘ 0
```

Key points:
- Wasm's value stack only holds scalars (i32, i64, f32, f64). Large structs and address-taken locals live in the **shadow stack** in linear memory.
- A "pure" C function returning a large struct actually writes to its shadow stack frame via `i32.store` instructions — not pure with respect to memory.

#### 2.1.6 Compile-Time Guarantees

- **Spatial safety**: all memory accesses bounds-checked against `active_pages * PAGE_SIZE`
- **Temporal safety**: Rust's lifetime system prevents use-after-free
- **Isolation**: each module has its own `IsolatedMemory` instance — distinct types, distinct backing arrays, no cross-module access possible

### 2.2 Module Types

```{spec} Module Types
:id: SPEC_MODULE_TYPES
:satisfies: REQ_MOD_TWO_TYPES
:tags: module

Process-like (owns memory) and library-like (borrows memory) module representations.
```

> Implementation: [crates/herkos-runtime/src/module.rs](../crates/herkos-runtime/src/module.rs)

```
                    ┌─────────────────────────────────────┐
                    │         Module Taxonomy             │
                    ├──────────────────┬──────────────────┤
                    │                  │                  │
              ┌─────┴───-──┐    ┌──────┴──────┐   ┌───-───┴─────┐
              │  Module    │    │ Library     │   │ Pure        │
              │ (owns mem) │    │ Module      │   │ (no memory) │
              │            │    │ (borrows)   │   │             │
              └────────────┘    └─────────────┘   └─────────────┘
              Like a process    Like a shared      Pure computation
                                library
```

#### Process-like Module (owns memory)

```rust
struct Module<G, const MAX_PAGES: usize, const TABLE_SIZE: usize> {
    memory: IsolatedMemory<MAX_PAGES>,
    globals: G,
    table: Table<TABLE_SIZE>,
}
```

#### Library Module (borrows memory)

```rust
struct LibraryModule<G, const TABLE_SIZE: usize> {
    globals: G,
    table: Table<TABLE_SIZE>,
    // no memory field — uses caller's memory
}
```

| Wasm declaration | Rust representation | Analogy |
|------------------|---------------------|---------|
| Module defines memory | `Module` owns `IsolatedMemory` | POSIX process |
| Module imports memory | `LibraryModule` borrows `&mut IsolatedMemory` | Shared library |
| Module has no memory | `LibraryModule` with no memory parameter | Pure computation |

### 2.3 Globals and Tables

```{spec} Globals and Tables
:id: SPEC_GLOBALS_TABLES
:satisfies: REQ_MOD_GLOBALS, REQ_MOD_TABLE
:tags: module, global, table

Globals as typed struct fields, tables for indirect call dispatch.
```

> Implementation: [crates/herkos-runtime/src/table.rs](../crates/herkos-runtime/src/table.rs)

#### Globals

```rust
// Generated by the transpiler — one struct per module
struct Globals {
    g0: i32,      // (global (mut i32) ...) — mutable, lives in struct
    // g1 is immutable → emitted as `const G1: i64 = 42;` instead
}
```

#### Tables

```rust
struct Table<const MAX_SIZE: usize> {
    entries: [Option<FuncRef>; MAX_SIZE],
    active_size: usize,
}

struct FuncRef {
    type_index: u32,   // canonical type index for signature check
    func_index: u32,   // index into module function space → match dispatch
}
```

Tables are initialized from element segments during module construction:

```rust
// From: (elem (i32.const 0) $add $sub $mul)
let mut table = Table::try_new(3);
table.set(0, Some(FuncRef { type_index: 0, func_index: 0 })).unwrap();
table.set(1, Some(FuncRef { type_index: 0, func_index: 1 })).unwrap();
table.set(2, Some(FuncRef { type_index: 0, func_index: 2 })).unwrap();
```

### 2.4 Imports as Trait Bounds

```{spec} Imports as Trait Bounds
:id: SPEC_IMPORTS
:satisfies: REQ_CAP_IMPORTS, REQ_CAP_ZERO_COST
:tags: capability, import

Wasm imports mapped to Rust trait bounds for capability-based security.
```

Capabilities are Rust **traits**, not bitflags. A Wasm module's imports become trait bounds on its functions:

```rust
// Generated from: (import "env" "socket_open" (func ...))
//                 (import "env" "socket_read" (func ...))
trait SocketOps {
    fn socket_open(&mut self, domain: i32, sock_type: i32) -> WasmResult<i32>;
    fn socket_read(&mut self, fd: i32, buf_ptr: u32, len: u32) -> WasmResult<i32>;
}

// Module function that calls socket imports requires the trait:
fn send_data<H: SocketOps + FileOps>(
    host: &mut H,
    memory: &mut IsolatedMemory<MAX_PAGES>,
    // ...
) -> WasmResult<i32> {
    let sock = host.socket_open(2, 1)?;
    // ...
}

// No imports → no host parameter, pure computation:
fn pure_math(a: i32, b: i32) -> i32 { a.wrapping_add(b) }
```

| Aspect | Bitflags (`const CAPS: u64`) | Traits |
|--------|------------------------------|--------|
| Granularity | Coarse (1 bit = 1 class) | Fine (exact function signatures) |
| Compile-time checking | Fails if bit not set | Fails if trait not implemented |
| Error messages | Opaque bit mismatch | Clear: "trait `SocketOps` not implemented" |
| Runtime cost | Zero | Zero (monomorphization) |
| Extensibility | Limited to 64 bits | Unlimited |
| Inter-module linking | Not supported | Natural via trait composition |

### 2.5 Exports as Trait Implementations

```{spec} Exports as Trait Implementations
:id: SPEC_EXPORTS
:satisfies: REQ_CAP_EXPORTS, REQ_CAP_ZERO_COST
:tags: capability, export

Wasm exports mapped to Rust trait implementations.
```

```rust
// Generated from: (export "transform" (func $transform))
trait ImageLibExports {
    fn transform(&mut self, ptr: u32, len: u32) -> WasmResult<i32>;
    fn init(&mut self) -> WasmResult<()>;
}

impl<const MAX_PAGES: usize> ImageLibExports for ImageModule<MAX_PAGES> {
    fn transform(&mut self, ptr: u32, len: u32) -> WasmResult<i32> {
        func_transform(&mut self.memory, &mut self.globals, ptr, len)
    }
    fn init(&mut self) -> WasmResult<()> {
        func_init(&mut self.memory, &mut self.globals)
    }
}
```

### 2.6 WASI Support

WASI is a standard set of import traits shipped by `herkos-runtime`:

```rust
trait WasiFd {
    fn fd_read(&mut self, fd: i32, iovs: u32, iovs_len: u32, nread: u32) -> WasmResult<i32>;
    fn fd_write(&mut self, fd: i32, iovs: u32, iovs_len: u32, nwritten: u32) -> WasmResult<i32>;
    fn fd_close(&mut self, fd: i32) -> WasmResult<i32>;
    // ...
}

trait WasiClock {
    fn clock_time_get(&mut self, clock_id: i32, precision: i64, time: u32) -> WasmResult<i32>;
}

trait WasiRandom {
    fn random_get(&mut self, buf: u32, len: u32) -> WasmResult<i32>;
}
```

The host implements whichever subset it supports:

```rust
// Bare-metal: only fd_write (UART) and clock
struct EmbeddedHost { /* ... */ }
impl WasiFd for EmbeddedHost { /* UART-backed */ }
impl WasiClock for EmbeddedHost { /* hardware timer */ }

// Full POSIX: everything
struct PosixHost { /* ... */ }
impl WasiFd for PosixHost { /* real file ops */ }
impl WasiClock for PosixHost { /* clock_gettime */ }
impl WasiRandom for PosixHost { /* /dev/urandom */ }
```

Custom platform-specific capabilities beyond WASI are just additional traits (e.g., `GpioOps`, `CanBusOps`).

### 2.7 Isolation Guarantees

```{spec} Isolation Guarantees
:id: SPEC_ISOLATION
:satisfies: REQ_ISOLATION_COMPILE_TIME, REQ_FREEDOM_FROM_INTERFERENCE, REQ_ISOLATION_SPATIAL, REQ_ISOLATION_CAPABILITY
:tags: isolation, safety

Compile-time freedom from interference via Rust ownership model.
```

The ownership model enforces freedom from interference structurally:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Module A      │     │   Module B      │     │   Library C     │
│                 │     │                 │     │                 │
│ ┌─────────────┐ │     │ ┌─────────────┐ │     │  (no memory)    │
│ │ Memory A    │ │     │ │ Memory B    │ │     │                 │
│ │ (owned)     │ │     │ │ (owned)     │ │     │  Borrows caller │
│ └─────────────┘ │     │ └─────────────┘ │     │  memory for     │
│ ┌─────────────┐ │     │ ┌─────────────┐ │     │  duration of    │
│ │ Globals A   │ │     │ │ Globals B   │ │     │  each call      │
│ └─────────────┘ │     │ └─────────────┘ │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
       ✗ cannot               ✗ cannot              ✓ borrows
       access B               access A               one at a time
```

1. **Module with its own memory**: cannot access another module's memory — each owns a distinct `IsolatedMemory` instance
2. **Library module**: can only access the specific memory it was handed via `&mut` borrow. Cannot hold the reference past the call (lifetime enforced), cannot access a different module's memory
3. **Pure module**: no memory at all — the type system provides no memory access methods

Inter-module calls lend memory for the duration of the call:

```rust
let mut app = Module::<AppGlobals, 256, 0>::new(16, AppGlobals::default(), table)?;
let mut lib = LibraryModule::<LibGlobals, 0>::new(LibGlobals::default(), table)?;

// Caller's memory is borrowed for this call only.
// Rust borrow checker guarantees the library cannot store the reference.
let result = lib.call_export_transform(&mut app.memory, ptr, len)?;
```

---

## 3. Architecture

### 3.1 Component Overview

```
┌───────────────────────────────────────────────────────────────────────────┐
│                           herkos workspace                                │
│                                                                           │
│  ┌──────────────┐  ┌────────────────────────┐  ┌──────────────────────┐   │
│  │ herkos (CLI) │  │    herkos-core         │  │   herkos-runtime     │   │
│  │              │  │                        │  │   #![no_std]         │   │
│  │  clap CLI    │  │  ┌──────────────────┐  │  │                      │   │
│  │  arg parsing │  │  │ Parser           │  │  │  IsolatedMemory      │   │
│  │              │  │  │ (wasmparser)     │  │  │  Table, FuncRef      │   │
│  │  calls       │  │  ├──────────────────┤  │  │  Module types        │   │
│  │  transpile() ├─►│  │ IR Builder       │  │  │  WasmTrap            │   │
│  │              │  │  │ (pure SSA)       │  │  │  Wasm ops            │   │
│  └──────────────┘  │  ├──────────────────┤  │  │                      │   │
│                    │  │ Optimizer        │  │  └──────────────────────┘   │
│  ┌──────────────┐  │  │ pre + post phase │  │           ▲                 │
│  │ herkos-tests │  │  ├──────────────────┤  │           │ depends on      │
│  │              │  │  │ Phi Lowering     │  │           │                 │
│  │  WAT/C/Rust  │  │  ├──────────────────┤  │           │                 │
│  │  → .wasm     │  │  │ Backend (safe)   │  │           │                 │
│  │  → transpile │  │  ├──────────────────┤  │           │                 │
│  │  → test      │◄─┤  │ Codegen          │  ├───────────┘                 │
│  │              │  │  └──────────────────┘  │                             │
│  │  benches/    │  │                        │                             │
│  └──────────────┘  └────────────────────────┘                             │
└───────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Runtime (`herkos-runtime`)

> Source: [crates/herkos-runtime/src/](../crates/herkos-runtime/src/)

The runtime is a `#![no_std]` crate providing the types that all transpiled code depends on. It has **zero external dependencies** in the default configuration.

| Module | Provides | Reference |
|--------|----------|-----------|
| `memory.rs` | `IsolatedMemory<MAX_PAGES>`, load/store methods, `memory.grow`/`memory.size` | §2.1 |
| `table.rs` | `Table<MAX_SIZE>`, `FuncRef` | §2.3 |
| `module.rs` | `Module<G, MAX_PAGES, TABLE_SIZE>`, `LibraryModule<G, TABLE_SIZE>` | §2.2 |
| `ops.rs` | Wasm arithmetic operations (`i32_div_s`, `i32_trunc_f32_s`, etc.) | §4.4 |
| `lib.rs` | `WasmTrap`, `WasmResult<T>`, `ConstructionError`, `PAGE_SIZE` | §4.3 |

**Constraints** (see [REQ_PLATFORM_NO_STD](REQUIREMENTS.md)):
- No heap allocation without the optional `alloc` feature gate
- No panics, no `format!`, no `String`
- Errors are `Result<T, WasmTrap>` only
- Optional `alloc` feature gate for targets with a global allocator

**Runtime verification with Kani**: The runtime includes `#[kani::proof]` harnesses that verify core invariants (no panics on any input, correct grow semantics, load/store roundtrip). Run via `cargo kani`. See [crates/herkos-runtime/KANI.md](../crates/herkos-runtime/KANI.md).

### 3.3 Transpiler (`herkos-core`)

> Source: [crates/herkos-core/src/](../crates/herkos-core/src/)

`herkos-core` is the transpiler library. The `herkos` crate is a thin CLI wrapper around it. The pipeline:

```
                              herkos-core::transpile()
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  .wasm ──→ Parser ──→ IR Builder ──→ optimize_ir() ──→ lower_phis(). │
│            │            │               │                  │         │
│            │ wasmparser │ pure SSA IR   │ pre-lowering     │ SSA     │
│            │ crate      │ phi nodes     │ passes           │ destruct│
│            ▼            ▼               ▼                  ▼         │
│          ParsedModule  ModuleInfo    ModuleInfo        LoweredModule │
│                        + IrFunctions (optimized)       Info          │
│                                                           │          │
│       ──→ optimize_lowered_ir() ──→ Codegen ──→ rustfmt   │          │
│               │                      │           │        │          │
│               │ post-lowering        │ Backend   │ format │          │
│               │ passes               │ (safe)    │        │          │
│               ▼                      ▼           ▼        │          │
│           LoweredModuleInfo        String      String ◄───┘          │
│           (re-optimized)           (raw)       (formatted)           │
└──────────────────────────────────────────────────────────────────────┘
```

#### 3.3.1 Parser

> Source: [crates/herkos-core/src/parser/](../crates/herkos-core/src/parser/)

Uses the `wasmparser` crate to extract module structure: types, functions, memories, tables, globals, imports, exports, data segments, element segments.

**Design choice**: `wasmparser` only, not `wasm-tools` or `walrus`. Keeps the dependency tree small and avoids pulling in a full Wasm runtime.

#### 3.3.2 IR (Intermediate Representation)

> Source: [crates/herkos-core/src/ir/](../crates/herkos-core/src/ir/)

A pure SSA-form IR that sits between Wasm bytecode and Rust source. Every variable is defined exactly once (`DefVar` token, non-`Copy`) and may be read many times (`UseVar` token, `Copy`).

```
           Wasm bytecode                  SSA IR                      Rust source
    ┌──────────────────────┐   ┌──────────────────────────┐   ┌──────────────────────────┐
    │ i32.const 5          │   │ v0 = Const(I32(5))       │   │ let mut v0: i32 = 0i32;  │
    │ i32.const 3          │   │ v1 = Const(I32(3))       │   │ let mut v1: i32 = 0i32;  │
    │ i32.add              │   │ v2 = BinOp(Add, v0, v1)  │   │ let mut v2: i32 = 0i32;  │
    │                      │   │                          │   │ v0 = 5i32;               │
    │                      │   │                          │   │ v1 = 3i32;               │
    │                      │   │                          │   │ v2 = v0.wrapping_add(v1);│
    └──────────────────────┘   └──────────────────────────┘   └──────────────────────────┘
```

Key types (defined in `ir/types.rs`):
- `ModuleInfo` — complete module metadata (types, functions, memories, globals, imports, exports)
- `LoweredModuleInfo` — newtype wrapper around `ModuleInfo` guaranteeing no `IrInstr::Phi` nodes remain; only produced by `lower_phis::lower()`
- `IrFunction` — one function's IR: entry block, all basic blocks, locals, return type, type index
- `IrBlock` — basic block with a `Vec<IrInstr>` body and an `IrTerminator`
- `IrInstr` — a single SSA instruction: `Const`, `BinOp`, `UnOp`, `Load`, `Store`, `Call`, `CallImport`, `CallIndirect`, `Assign`, `GlobalGet`, `GlobalSet`, `MemorySize`, `MemoryGrow`, `MemoryCopy`, `MemoryFill`, `MemoryInit`, `DataDrop`, `Select`, `Phi`
- `IrTerminator` — block exit: `Return`, `Jump`, `BranchIf`, `BranchTable`, `Unreachable`
- `VarId` — SSA variable identifier (displayed as `v0`, `v1`, ...)
- `BlockId` — basic block identifier (displayed as `block_0`, `block_1`, ...)
- `DefVar` / `UseVar` — single-use definition / multi-use read tokens enforcing SSA invariants at build time

Index types use a phantom-typed `Idx<TAG>` generic to prevent mixing `LocalFuncIdx`, `ImportIdx`, `GlobalIdx`, `TypeIdx`, etc.

The builder (`ir/builder/`) translates Wasm stack-based instructions to SSA IR by maintaining an explicit value stack of `UseVar`. Each function is independent — enabling future parallelization (see §6.2).

#### 3.3.3 SSA Phi Lowering

> Source: [crates/herkos-core/src/ir/lower_phis.rs](../crates/herkos-core/src/ir/lower_phis.rs)

SSA phi nodes are inserted by the builder at join points (if/else merges, loop headers). Before codegen they must be *destroyed* — converted to ordinary assignments in predecessor blocks.

```
    Before lowering (SSA IR):          After lowering (LoweredModuleInfo):

    block0:                             block0:
      br_if cond → block1, block2         br_if cond → block1, block2

    block1 (then):                      block1 (then):
      br → block3                          v2 = v0          ← Assign inserted
                                           br → block3

    block2 (else):                      block2 (else):
      br → block3                          v2 = v1          ← Assign inserted
                                           br → block3

    block3 (merge):                     block3 (merge):
      v2 = Phi(block1→v0, block2→v1)      (Phi removed)
```

The pass:
1. **Prunes stale sources**: removes `(pred, var)` entries whose predecessor was eliminated by dead block removal
2. **Simplifies trivial phis**: single-source or all-same-source phis become `Assign` in-place
3. **Lowers non-trivial phis**: inserts `Assign { dest, src }` at the end of each predecessor block, then removes the `Phi`

#### 3.3.4 Optimizer

> Source: [crates/herkos-core/src/optimizer/](../crates/herkos-core/src/optimizer/)

The optimizer is split into two phases separated by phi lowering:

**Pre-lowering passes** (operate on SSA IR with phi nodes intact):

| Pass | What it does |
|------|-------------|
| `dead_blocks` | Removes basic blocks unreachable from the entry block |
| `const_prop` | Propagates constant values through assignments and binary ops |
| `algebraic` | Algebraic simplifications (e.g., `x + 0 → x`, `x * 1 → x`) |
| `copy_prop` | Replaces uses of copy vars with their sources (`v1 = v0; use(v1)` → `use(v0)`) |

**Post-lowering passes** (operate on phi-free `LoweredModuleInfo`):

| Pass | What it does |
|------|-------------|
| `empty_blocks` | Removes blocks with no instructions and a single unconditional jump |
| `dead_blocks` | Second dead block pass after structural changes |
| `merge_blocks` | Merges a block with its sole successor when no other predecessor exists |
| `copy_prop` | Copy propagation on lowered IR |
| `local_cse` | Local common subexpression elimination within each block |
| `gvn` | Global value numbering across blocks |
| `dead_instrs` | Removes instructions whose results are never used |
| `branch_fold` | Folds constant-condition branches (`br_if true → jump`) |
| `licm` | Loop-invariant code motion: hoists invariant computations out of loops |

Both phases run up to 2 iterations until fixed point. Passes run only when `--optimize` / `-O` is passed.

#### 3.3.5 Backend

> Source: [crates/herkos-core/src/backend/](../crates/herkos-core/src/backend/)

The `Backend` trait abstracts the code emission strategy. Currently only `SafeBackend` is implemented:

- Emits 100% safe Rust
- Every memory access goes through bounds-checked wrappers returning `WasmResult<T>`
- No verification metadata required

For the planned verified and hybrid backends, see [FUTURE.md](FUTURE.md).

#### 3.3.6 Code Generator

> Source: [crates/herkos-core/src/codegen/](../crates/herkos-core/src/codegen/)

Walks the `LoweredModuleInfo` and emits Rust source code via the configured `Backend`:

| Codegen module | Responsibility |
|---------------|----------------|
| `module.rs` | Top-level orchestration; module struct definition |
| `function.rs` | Function signatures, local declarations, block state machines |
| `instruction.rs` | Individual `IrInstr` → Rust expression |
| `traits.rs` | `ModuleHostTrait` generation from function imports |
| `export.rs` | Export method generation (forwarding to internal functions) |
| `constructor.rs` | `new()` with data segment and element segment initialization |
| `env.rs` | `Env<H>` context struct bundling host + globals |
| `types.rs` | Wasm → Rust type name mapping |
| `utils.rs` | Call arg building, import grouping |

**Multi-block control flow** uses a local `Block` enum and a `loop { match __block { … } }` state machine. Single-block functions optimize to flat inline code.

### 3.4 Tests (`herkos-tests`)

> Source: [crates/herkos-tests/](../crates/herkos-tests/)

End-to-end test crate that compiles WAT/C/Rust sources to `.wasm`, transpiles them, and runs the output.

#### Test pipeline

```
WAT / C / Rust source
        │
        ▼ (build.rs)
    .wasm binary
        │
        ▼ (herkos::transpile)
    Generated .rs
        │
        ▼ (include! in test)
    Compiled & tested
```

#### Test categories

| Category | Test files | What's tested |
|----------|-----------|---------------|
| Arithmetic | `arithmetic.rs`, `numeric_ops.rs` | Wasm arithmetic, bitwise, comparison ops |
| Memory | `memory.rs`, `memory_grow.rs`, `subwidth_mem.rs`, `bulk_memory.rs` | Load/store, memory.grow, sub-width access, memory.copy/fill/init |
| Control flow | `control_flow.rs`, `early_return.rs`, `select.rs`, `unreachable.rs` | Block, loop, if, br, br_table, select |
| Functions | `function_calls.rs`, `indirect_calls.rs`, `indirect_call_import.rs` | Direct calls, call_indirect dispatch, indirect calls through imports |
| Imports/Exports | `import_traits.rs`, `import_memory.rs`, `import_multi.rs`, `module_wrapper.rs`, `call_import_transitive.rs` | Trait-based imports, module wrapper, transitive import calls |
| Locals | `locals.rs`, `locals_aliasing.rs` | Local variable handling |
| Inter-module | `inter_module_lending.rs` | Memory lending between modules |
| E2E (C) | `c_e2e.rs`, `c_e2e_i64.rs`, `c_e2e_loops.rs`, `c_e2e_memory.rs` | Full C → Wasm → Rust pipeline |
| E2E (Rust) | `rust_e2e.rs`, `rust_e2e_control.rs`, `rust_e2e_i64.rs`, `rust_e2e_heavy_fibo.rs`, `rust_e2e_memory_bench.rs` | Pre-generated Rust modules |

#### Running tests

```bash
cargo test -p herkos-core     # transpiler unit tests (IR, optimizer, codegen)
cargo test -p herkos-runtime  # runtime unit tests

# herkos-tests must always be run twice:
HERKOS_OPTIMIZE=0 cargo test -p herkos-tests   # unoptimized transpiler output
HERKOS_OPTIMIZE=1 cargo test -p herkos-tests   # optimized transpiler output
```

`HERKOS_OPTIMIZE` is read by `herkos-tests/build.rs` at compile time. When set to `"1"`, the test suite's `.wasm` sources are transpiled with the IR optimization pipeline enabled; any other value (or unset) disables it. Running both variants is required — it verifies that the optimizer is *semantics-preserving*: the same inputs must produce the same outputs regardless of optimization level. CI enforces both runs as separate steps. The variable has no effect on `herkos-core`, `herkos-runtime`, or any production code.

### 3.5 Benchmarks

> Source: [crates/herkos-tests/benches/](../crates/herkos-tests/benches/)

Performance benchmarks using Criterion. Currently includes Fibonacci benchmarks comparing transpiled Wasm execution against native Rust.

```bash
cargo bench -p herkos-tests
```

---

## 4. Transpilation Rules

This section describes how Wasm constructs map to Rust code in the safe backend.

### 4.1 Function Translation

```{spec} Function Translation
:id: SPEC_FUNCTION_TRANSLATION
:satisfies: REQ_TRANS_FUNCTIONS, REQ_TRANS_DETERMINISTIC
:tags: transpilation, function

Wasm functions to Rust functions via SSA IR.
```

Wasm functions become Rust functions. Module state is threaded through as parameters. Functions that call imports or touch mutable globals receive an `Env<H>` context struct; pure computation functions omit it.

```rust
// Wasm: (func $example (param i32) (result i32))
// No imports, no mutable globals → memory only
fn func_0(
    memory: &mut IsolatedMemory<MAX_PAGES>,
    v0: i32,
) -> WasmResult<i32> {
    // function body in SSA variable form
}

// Wasm: (func $send (param i32 i32) (result i32))
// Calls imported functions + uses mutable globals → Env<H> parameter
fn func_1<H: ModuleHostTrait>(
    memory: &mut IsolatedMemory<MAX_PAGES>,
    env: &mut Env<H>,
    v0: i32,
    v1: i32,
) -> WasmResult<i32> {
    // env.host.some_import(...)
    // env.globals.g0 = ...
}
```

Only state that the function actually uses is passed. Memory is omitted for memory-free functions; `env` is omitted when there are no imports and no mutable globals.

### 4.2 Control Flow

```{spec} Control Flow Mapping
:id: SPEC_CONTROL_FLOW
:satisfies: REQ_TRANS_CONTROL_FLOW
:tags: transpilation, control

Wasm structured control flow mapped to safe Rust (loop/break/if).
```

Wasm structured control flow is lowered to basic blocks in the IR, then emitted as a state-machine loop in Rust:

```rust
// Multi-block function body (any branching control flow):
#[derive(Clone, Copy)]
enum __Block { B0, B1, B2 }
let mut __block = __Block::B0;
loop {
    match __block {
        __Block::B0 => { /* block_0 instructions */; __block = __Block::B1; }
        __Block::B1 => { /* block_1 instructions */; if cond { __block = __Block::B2; } else { __block = __Block::B0; } }
        __Block::B2 => { return Ok(v_result); }
    }
}
```

| Wasm | IR | Rust (state machine) |
|------|----|----------------------|
| `block` / `end` | jump to successor block | `__block = __Block::BN` |
| `loop` | back-edge jump to loop header | `__block = __Block::BHeader` |
| `if / else / end` | `BranchIf { cond, if_true, if_false }` | `if cond { __block = BT } else { __block = BF }` |
| `br $label` | `Jump { target }` | `__block = __Block::BN` |
| `br_if $label` | `BranchIf` | `if cond { __block = BT } else { __block = BF }` |
| `br_table` | `BranchTable { index, targets, default }` | `match index { 0 => __block = B0, … _ => __block = BD }` |
| `unreachable` | `Unreachable` terminator | `return Err(WasmTrap::Unreachable)` |

Single-block functions (no branches) skip the state machine entirely and emit flat inline code.

### 4.3 Error Handling

```{spec} Error Handling
:id: SPEC_ERROR_HANDLING
:satisfies: REQ_ERR_TRAPS
:tags: transpilation, error

Trap-based error handling via WasmTrap/WasmResult.
```

> Implementation: [crates/herkos-runtime/src/lib.rs](../crates/herkos-runtime/src/lib.rs)

```rust
enum WasmTrap {
    OutOfBounds,              // Memory access out of bounds
    DivisionByZero,           // Integer division by zero
    IntegerOverflow,          // e.g., i32.trunc_f64_s on out-of-range float
    Unreachable,              // unreachable instruction executed
    IndirectCallTypeMismatch, // call_indirect signature check failed
    TableOutOfBounds,         // Table access out of bounds
    UndefinedElement,         // Undefined element in table
}

type WasmResult<T> = Result<T, WasmTrap>;
```

No panics, no unwinding. The `?` operator propagates traps up the call stack.

### 4.4 Arithmetic Operations

```{spec} Arithmetic Operations
:id: SPEC_ARITHMETIC
:satisfies: REQ_TRANS_FUNCTIONS
:tags: transpilation, arithmetic

Wasm arithmetic semantics (wrapping, trapping division, IEEE 754 floats).
```

> Implementation: [crates/herkos-runtime/src/ops.rs](../crates/herkos-runtime/src/ops.rs)

Wasm arithmetic operations that can trap (division, remainder, truncation) return `WasmResult`:

```rust
fn i32_div_s(a: i32, b: i32) -> WasmResult<i32>;  // traps on /0 or overflow
fn i32_rem_u(a: i32, b: i32) -> WasmResult<i32>;   // traps on /0
fn i32_trunc_f32_s(a: f32) -> WasmResult<i32>;     // traps on out-of-range
```

Non-trapping arithmetic uses Rust's wrapping operations (`wrapping_add`, `wrapping_mul`, etc.) per the Wasm spec.

### 4.5 Function Calls

```{spec} Function Calls
:id: SPEC_FUNCTION_CALLS
:satisfies: REQ_TRANS_INDIRECT_CALLS, REQ_TRANS_TYPE_EQUIVALENCE
:tags: transpilation, call

Direct calls, indirect calls via table, structural type equivalence.
```

#### 4.5.1 Direct Calls (`call`)

Direct calls transpile to regular Rust function calls with state threaded through:

```rust
// Wasm: call $func_3 (with 2 args on the stack)
v5 = func_3(memory, globals, table, v3, v4)?;
```

#### 4.5.2 Indirect Calls (`call_indirect`)

`call_indirect` implements function pointers. The transpiler emits a static match dispatch:

```rust
// Wasm: call_indirect (type $binop)  ; expects (i32, i32) -> i32
let __entry = table.get(v2 as u32)?;                    // lookup + bounds check
if __entry.type_index != 0 {                              // type signature check
    return Err(WasmTrap::IndirectCallTypeMismatch);
}
v4 = match __entry.func_index {                           // static dispatch
    0 => func_0(v0, v1, table)?,    // add
    1 => func_1(v0, v1, table)?,    // sub
    2 => func_2(v0, v1, table)?,    // mul
    _ => return Err(WasmTrap::UndefinedElement),
};
```

**Why match-based dispatch?** Function pointer arrays, `dyn Fn` trait objects, or computed gotos all require `unsafe`, heap allocation, or break `no_std` compatibility. A match statement is 100% safe, `no_std` compatible, and LLVM optimizes it to a jump table when arms are dense.

The `_ =>` arm handles func_index values that don't match any function of the right type — a safety net for corrupted table entries.

#### 4.5.3 Structural Type Equivalence

The Wasm spec requires `call_indirect` to use **structural equivalence**: two type indices match if they have identical parameter and result types, regardless of index.

```
Type 0: (i32, i32) → i32  →  canonical = 0
Type 1: (i32, i32) → i32  →  canonical = 0  (same signature as type 0)
Type 2: (i32) → i32       →  canonical = 2  (new signature)
```

The transpiler builds a canonical type index mapping at transpile time. Both `FuncRef.type_index` and the type check use canonical indices. At runtime, the check is a simple integer comparison.

### 4.6 Bulk Memory Operations

```{spec} Bulk Memory Operations
:id: SPEC_BULK_MEMORY
:satisfies: REQ_MEM_BULK_OPS, REQ_MEM_DATA_SEGMENTS
:tags: transpilation, memory, bulk

memory.fill, memory.init, data.drop, memory.copy.
```

> Implementation: [crates/herkos-runtime/src/memory.rs](../crates/herkos-runtime/src/memory.rs) lines 149–174

The WebAssembly bulk memory operations allow efficient copying and initialization of memory regions without scalar load/store loops.

#### 4.6.1 `memory.fill`

Fills a region of memory with a byte value. Per Wasm spec, only the low 8 bits of the value are used.

```rust
impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
    pub fn fill(&mut self, dst: usize, val: u8, len: usize) -> WasmResult<()>;
}
```

Generated code:
```rust
// Wasm: memory.fill $dst $val $len
memory.fill(dst as usize, val as u8, len as usize)?;
```

Traps `OutOfBounds` if `[dst, dst + len)` exceeds active memory. Length zero is a no-op.

#### 4.6.2 `memory.init`

Copies data from a passive data segment into memory at runtime. Each data segment is stored as a constant `&'static [u8]` in the generated code.

```rust
impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
    pub fn init_data_partial(&mut self, dst: usize, data: &[u8], src_offset: usize, len: usize) -> WasmResult<()>;
}
```

Generated code:
```rust
// Wasm: memory.init $data_segment $dst $src_offset $len
memory.init_data_partial(dst as usize, &DATA_SEGMENT_0, src_offset as usize, len as usize)?;
```

Traps `OutOfBounds` if either region (source or destination) exceeds bounds:
- Source: `[src_offset, src_offset + len)` must be within the data segment
- Destination: `[dst, dst + len)` must be within active memory

#### 4.6.3 `data.drop`

Marks a data segment as dropped (per Wasm spec). In the safe backend this is a no-op because data segments are stored as constant references and cannot actually be deallocated.

```rust
// Wasm: data.drop $segment
// (no-op in safe backend — const slices persist)
```

In future verified and hybrid backends, `data.drop` may enable optimizations: proving that dropped segments are never accessed again could allow proving certain addresses as never-in-bounds.

---

## 5. Integration

### 5.1 Trait-Based Integration (Primary)

The host instantiates modules and provides capabilities through trait implementations:

```rust
struct MyHost { /* platform resources */ }
impl SocketOps for MyHost { /* ... */ }
impl WasiFd for MyHost { /* ... */ }

let mut module = Module::<MyGlobals, 256, 4>::new(16, MyGlobals::default(), table)?;
let mut host = MyHost::new();
let result = module.process_data(&mut host, ptr, len)?;
```

Full type safety, zero `unsafe`, zero-cost dispatch via monomorphization.

### 5.2 The Env<H> Context Pattern

> Implementation: [crates/herkos-core/src/codegen/env.rs](../crates/herkos-core/src/codegen/env.rs)

Generated modules use a unified **Env<H>** context struct that bundles the host (generic parameter `H`) and mutable globals, simplifying parameter threading throughout function calls.

```rust
// Generated by transpiler
pub struct Env<'a, H: ModuleHostTrait + ?Sized> {
    pub host: &'a mut H,
    pub globals: &'a mut Globals,
}

// Every function that needs imports or mutable state receives Env<H>
fn process<H: ModuleHostTrait>(
    memory: &mut IsolatedMemory<MAX_PAGES>,
    env: &mut Env<H>,
    input: i32,
) -> WasmResult<i32> {
    // Call imported function via trait
    let result = env.host.some_import(input)?;
    // Read/write mutable global
    env.globals.my_global += 1;
    Ok(result)
}
```

**Design rationale:**
- **Unified state**: Avoids threading `host`, `globals`, and other mutable state as separate parameters
- **Type safety**: All imports must be present in the host's trait implementation — checked at compile time
- **Zero overhead**: The Env struct is a thin wrapper; LLVM inlines and optimizes away the indirection
- **Extensibility**: Adding new imports or globals requires only modifying the trait, not all function signatures

**Generated trait:**

```rust
pub trait ModuleHostTrait {
    // One method per function import
    fn imported_function(&mut self, arg: i32) -> WasmResult<i32>;

    // Getter/setter methods for each imported global
    fn get_imported_global(&self) -> i32;
    fn set_imported_global(&mut self, value: i32);
}
```

### 5.4 C-Compatible ABI (Optional)

For integration with non-Rust systems, an optional `extern "C"` wrapper erases generics:

```rust
#[no_mangle]
pub extern "C" fn module_new(initial_pages: u32) -> *mut OpaqueModule { /* ... */ }

#[no_mangle]
pub extern "C" fn module_call(
    instance: *mut OpaqueModule,
    function_index: u32,
    args: *const i64,
    args_len: usize,
    result: *mut i64,
) -> i32 { /* 0 = success, non-zero = WasmTrap discriminant */ }
```

The C ABI wrapper uses `unsafe` and raw pointers. Capability enforcement still applies inside — the wrapper calls through trait-bounded functions. This is an escape hatch, not the default.

### 5.5 Native Rust Integration

Native Rust code integrates by implementing import traits directly:

```rust
trait GpioOps {
    fn gpio_set(&mut self, pin: u32, value: bool) -> WasmResult<()>;
    fn gpio_read(&self, pin: u32) -> WasmResult<bool>;
}

struct EmbeddedHost { /* ... */ }
impl GpioOps for EmbeddedHost { /* ... */ }
```

---

## 6. Performance

### 6.1 Overhead

| Backend | Overhead | Source | Status |
|---------|----------|--------|--------|
| Safe | 15–30% | Runtime bounds check on every memory access | Implemented |
| Verified | 0–5% | Function call indirection only | Planned ([FUTURE.md](FUTURE.md)) |
| Hybrid | 5–15% | Mix of checked and proven accesses | Planned ([FUTURE.md](FUTURE.md)) |

### 6.2 Monomorphization Bloat Mitigation

Each distinct `MAX_PAGES` and trait bound combination generates separate code. Mitigation strategies:

#### 1. Outline pattern (mandatory for runtime)

Move logic into non-generic inner functions. Generic wrapper is a thin shell:

```rust
#[inline(never)]
fn load_i32_inner(memory: &[u8], active_bytes: usize, offset: usize) -> WasmResult<i32> {
    // ONE copy in the binary
}

impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
    #[inline(always)]
    fn load_i32(&self, offset: usize) -> WasmResult<i32> {
        load_i32_inner(self.pages.as_flattened(), self.active_pages * PAGE_SIZE, offset)
    }
}
```

#### 2. `MAX_PAGES` normalization

Use standard sizes (16, 64, 256, 1024) instead of exact declared maximums. Two modules with `MAX_PAGES=253` and `MAX_PAGES=260` both use `MAX_PAGES=256`.

#### 3. Trait objects for cold paths

Use `&mut dyn Trait` instead of generics for rarely-called code (error handling, initialization).

#### 4. LTO

Link-time optimization eliminates unreachable monomorphized copies.

### 6.3 Transpiler Parallelization

IR building and code generation are embarrassingly parallel — each function is independent:

```
                  ┌──────────┐
                  │  Parse   │  (sequential)
                  └────┬─────┘
                       │
         ┌─────────────┼───────────┐
         ▼             ▼           ▼
    ┌──────────┐ ┌──────────┐ ┌──────────┐
    │ IR Build │ │ IR Build │ │ IR Build │  (parallel)
    │ func_0   │ │ func_1   │ │ func_N   │
    └────┬─────┘ └────┬─────┘ └────┬─────┘
         │            │            │
         ▼            ▼            ▼
    ┌──────────┐ ┌──────────┐ ┌──────────┐
    │ Codegen  │ │ Codegen  │ │ Codegen  │  (parallel)
    │ func_0   │ │ func_1   │ │ func_N   │
    └────┬─────┘ └────┬─────┘ └────┬─────┘
         │            │            │
         └────────────┼────────────┘
                      ▼
                  ┌──────────┐
                  │ Assemble │  (sequential)
                  └──────────┘
```

Activation heuristic: use `rayon` parallel iterators when the module has 20+ functions. Output is deterministic regardless of thread count (`par_iter().enumerate()` preserves order).

### 6.4 Comparison to Alternatives

| Approach | Runtime Overhead | Isolation Strength | `unsafe` in output |
|----------|-----------------|-------------------|--------------------|
| MMU/MPU | 10–50% (context switches) | Strong (hardware) | N/A |
| herkos (safe) | 15–30% | Strong (runtime checks) | None |
| herkos (verified, planned) | 0–5% | Strong (formal proofs) | Yes — proof-justified |
| WebAssembly runtime | 20–100% | Strong (runtime sandbox) | N/A |
| Software fault isolation | 10–30% | Medium (runtime) | N/A |

---

## 7. Security Properties

### 7.1 Protected Against

- **Memory corruption**: buffer overflows, use-after-free — prevented by bounds-checked access and Rust's ownership system
- **Unauthorized resource access**: files, network, system calls — prevented by trait-based capability enforcement
- **Cross-module interference**: freedom from interference — enforced by memory ownership isolation
- **ROP attacks**: no function pointers in generated code — all dispatch is static match

### 7.2 Not Protected Against (current scope)

- Logic bugs in the original C/C++ code
- Side-channel attacks (timing, cache)
- Resource exhaustion (infinite loops, memory leaks within bounds) — see [FUTURE.md](FUTURE.md) §3 for temporal isolation plans
- Timing interference — spatial isolation only, not temporal

### 7.3 Relationship to Safety Standards

This pipeline produces **evidence** for a freedom-from-interference argument:
- Transpiled Rust source is auditable
- Isolation boundary is the Rust type system — well-understood, no runtime configuration dependency
- **This tool does not replace a formal safety case.** It provides a compile-time isolation mechanism and associated evidence that can be used as part of one.

---

## 8. Open Questions

1. How to handle C++ exceptions in WebAssembly?
2. How to represent and verify concurrent access patterns?
3. Should we support dynamic linking of transpiled modules?
4. What level of C/C++ standard library should be supported?

---

## 9. References

- WebAssembly Specification: https://webassembly.github.io/spec/
- Rust Reference: https://doc.rust-lang.org/reference/
- Software Fault Isolation: Wahbe et al., 1993
- Proof-Carrying Code: Necula & Lee, 1996
