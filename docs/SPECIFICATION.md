# Specification

## 1. Intro

This document specifies the design and behavior of the `herkos` transpilation pipeline. It takes the goals and constraints defined in [REQUIREMENTS.md](REQUIREMENTS.md) as input and describes **how** they are achieved: the concrete data structures, algorithms, APIs, and code generation strategies that implement compile-time memory isolation, capability enforcement, deterministic execution, and the three code generation backends.

Where the requirements say *what* the system must do, this specification says *how* it does it.

## 2. Architecture

### 2.1 Compilation Pipeline

```
┌─────────────┐      ┌──────────────┐      ┌─────────────────┐      ┌─────────────┐
│   C/C++     │ ───> │  WebAssembly │ ───> │ Rust Transpiler │ ───> │ Safe Rust   │
│   Source    │      │    (Wasm)    │      │   + Runtime     │      │   Binary    │
└─────────────┘      └──────────────┘      └─────────────────┘      └─────────────┘
                            │                       │
                            │                       ▼
                            │              ┌─────────────────┐
                            └─────────────>│  Verification   │
                                           │   Metadata      │
                                           └─────────────────┘
```

### 2.2 Component Breakdown

#### 2.2.1 Stage 1: C/C++ to WebAssembly
- **Input**: C/C++ source code
- **Compiler**: LLVM/Clang with WebAssembly target
- **Output**: WebAssembly module (.wasm)
- **Key transformations**:
  - Undefined behavior → well-defined Wasm semantics
  - Raw pointers → Wasm linear memory offsets
  - Stack allocations → Wasm stack/linear memory

#### 2.2.2 Stage 2: WebAssembly Analysis
- **Input**: WebAssembly module
- **Analysis tasks**:
  - Extract memory access patterns
  - Identify capability requirements (file I/O, network, etc.)
  - Generate bounds checking metadata
  - Detect resource lifetime patterns
- **Output**: Verification metadata (TOML format, see §7)

#### 2.2.3 Stage 3: Rust Transpilation
- **Input**: WebAssembly module + verification metadata
- **Transpiler**: Custom tool (to be implemented)
- **Output**: Rust source code with:
  - Bounds-checked memory access via `IsolatedMemory` (see §3)
  - Import/export traits for capability enforcement (see §5)
  - Proof references from verification metadata (verified/hybrid backends, see §7)
  - Runtime support via `herkos-runtime` (see §6)

---

## 3. Memory Model

### 3.1 WebAssembly Page Model

WebAssembly linear memory is organized in **pages of 64 KiB** (65,536 bytes). A Wasm module declares an initial page count and an optional maximum page count. The `memory.grow` instruction adds pages at runtime.

```
Page size:    64 KiB (defined by the WebAssembly specification)
Initial size: declared in the Wasm module (e.g., 16 pages = 1 MiB)
Maximum size: declared in the Wasm module (e.g., 256 pages = 16 MiB)
```

### 3.2 Rust Representation

In the transpiled Rust code, linear memory is represented with a page-level const generic:

```rust
const PAGE_SIZE: usize = 65536; // 64 KiB, per Wasm spec

struct IsolatedMemory<const MAX_PAGES: usize> {
    /// Backing storage — MAX_PAGES pages of PAGE_SIZE bytes each.
    /// 2D array avoids `generic_const_exprs` (unstable); contiguous layout
    /// identical to `[u8; MAX_PAGES * PAGE_SIZE]`. Use `as_flattened()` for
    /// a flat `&[u8]` view (stable since Rust 1.80).
    pages: [[u8; PAGE_SIZE]; MAX_PAGES],

    /// Current number of active pages (starts at initial_pages).
    /// memory.grow increments this; accesses beyond active_pages * PAGE_SIZE trap.
    active_pages: usize,
}
```

Key design decisions:
- **`MAX_PAGES`** is a const generic derived from the Wasm module's declared maximum. The backing array is a 2D `[[u8; PAGE_SIZE]; MAX_PAGES]` — fully allocated at compile time, contiguous in memory, with `as_flattened()` for flat `&[u8]` views (stable Rust, no `generic_const_exprs`). This avoids any heap allocation and is `no_std` compatible.
- **`active_pages`** tracks the current live size. It starts at the module's declared initial page count and is incremented by `memory.grow`. Accesses beyond `active_pages * PAGE_SIZE` are out-of-bounds traps.
- If the Wasm module declares no maximum, the transpiler must choose a concrete maximum (configurable via CLI flag, e.g., `--max-pages 256`).
- **Monomorphization caution**: each distinct `MAX_PAGES` value generates a full copy of all `IsolatedMemory` methods. See §13.3 for mitigation strategies to control binary size and instruction cache pressure.

```{spec} Linear Memory as Const Generic
:id: SPEC_MEMORY_CONST_GENERIC
:status: open
:tags: memory, const-generic, no_std, monomorphization
Linear memory is represented as IsolatedMemory<const MAX_PAGES: usize>. The const
generic prevents any heap allocation, enables monomorphization for zero-cost dispatch,
and is no_std compatible. MAX_PAGES is derived from the Wasm module's declared maximum.
```

```{spec} 2D Memory Layout Avoiding generic_const_exprs
:id: SPEC_MEMORY_2D_LAYOUT
:status: open
:tags: memory, array-layout, stability
Memory backing is a 2D array [[u8; PAGE_SIZE]; MAX_PAGES], not 1D, to avoid the
unstable generic_const_exprs feature. The layout is contiguous in memory, and
as_flattened() provides flat &[u8] views for access operations (stable Rust 1.80+).
```

```{spec} Active Pages Tracking for memory.grow
:id: SPEC_MEMORY_ACTIVE_PAGES
:status: open
:tags: memory, memory.grow, bounds
Track active_pages—the current live page count. Starts at the module's initial page
count. Incremented by memory.grow instruction. All memory accesses beyond
(active_pages * PAGE_SIZE) are out-of-bounds traps.
```

```{spec} No Heap Allocation for Memory
:id: SPEC_MEMORY_NO_HEAP
:status: open
:tags: memory, no_std, allocation
IsolatedMemory uses only pre-allocated backing array—no Vec, no heap allocation, no
dynamic resizing. memory.grow is a counter increment plus zero-fill of new pages.
No allocation occurs. This enables no_std compatibility and embedded targets.
```

```{spec} WebAssembly Page Size Standard
:id: SPEC_MEMORY_PAGE_SIZE
:status: open
:tags: memory, wasm-spec
Page size is fixed at 64 KiB (65536 bytes) per the WebAssembly specification.
Not configurable. Immutable constant across all modules.
```

### 3.3 `memory.grow` Semantics

```rust
impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
    /// Wasm `memory.grow` — returns previous page count, or -1 on failure.
    /// No allocation occurs: the backing array is already sized to MAX_PAGES.
    /// New pages are zero-initialized (required by the Wasm spec).
    fn grow(&mut self, delta: u32) -> i32 {
        let old = self.active_pages;
        let new = old.wrapping_add(delta as usize);
        if new > MAX_PAGES {
            return -1; // growth refused
        }
        for page in &mut self.pages[old..new] {
            page.fill(0);
        }
        self.active_pages = new;
        old as i32
    }

    /// Wasm `memory.size` — returns current page count as i32 (Wasm convention).
    fn size(&self) -> i32 {
        self.active_pages as i32
    }
}
```

Because the backing array is pre-allocated to `MAX_PAGES`, `memory.grow` is a counter increment plus zero-fill of new pages — zero allocation, `no_std` safe. The zero-fill is required by the Wasm spec.

### 3.4 Memory Access API

WebAssembly linear memory is **flat and entirely read-write** — there is no concept of read-only pages or memory protection at the Wasm spec level. All accesses go through the same API:

```rust
impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
    // Safe backend: bounds-checked against active_pages * PAGE_SIZE.
    // One method per Wasm type — no WasmType trait, keeps things simple
    // and avoids monomorphization of the inner functions (outline pattern).
    fn load_i32(&self, offset: usize) -> WasmResult<i32>;
    fn load_i64(&self, offset: usize) -> WasmResult<i64>;
    fn load_u8(&self, offset: usize) -> WasmResult<u8>;
    fn load_u16(&self, offset: usize) -> WasmResult<u16>;
    fn load_f32(&self, offset: usize) -> WasmResult<f32>;
    fn load_f64(&self, offset: usize) -> WasmResult<f64>;
    fn store_i32(&mut self, offset: usize, value: i32) -> WasmResult<()>;
    fn store_i64(&mut self, offset: usize, value: i64) -> WasmResult<()>;
    // ... and store_u8, store_u16, store_f32, store_f64

    // Verified backend: unchecked, justified by proof
    unsafe fn load_i32_unchecked(&self, offset: usize) -> i32;
    unsafe fn load_i64_unchecked(&self, offset: usize) -> i64;
    unsafe fn store_i32_unchecked(&mut self, offset: usize, value: i32);
    unsafe fn store_i64_unchecked(&mut self, offset: usize, value: i64);
}
```

**Read-only guarantees are not a Wasm primitive** — they are an analysis result. The `wasm-verify` tool can prove that certain regions (e.g., data segment areas corresponding to C `.rodata`) are never targeted by any store instruction. This proof can be used by the verified backend to eliminate unnecessary store checks, but it's not a type-level distinction in the memory API itself.

### 3.5 Linear Memory Layout and the Shadow Stack

When C/C++ code compiles to Wasm, the compiler (e.g., Clang) organizes linear memory into conventional regions:

```
┌─────────────────────────────────────────┐ MAX_PAGES * PAGE_SIZE
│                                         │
│           (unused / growable)           │
│                                         │
├─────────────────────────────────────────┤ ← __stack_pointer (grows downward)
│                                         │
│           Shadow Stack                  │
│   (local variables, large structs,      │
│    spills, return values)               │
│                                         │
├─────────────────────────────────────────┤
│           Heap (grows upward)           │
│   (malloc / C++ new)                    │
├─────────────────────────────────────────┤
│           Data Segments                 │
│   (.data, .rodata, .bss)                │
├─────────────────────────────────────────┤ 0
```

Key points:
- **Wasm's value stack** only holds scalars (i32, i64, f32, f64). Local variables of scalar type live on the value stack and never touch linear memory.
- **Large structs, arrays, and address-taken locals** are placed in the **shadow stack** — a region of linear memory managed via a `__stack_pointer` global. The compiler adjusts this pointer on function entry/exit.
- **A "pure" C function** that returns a large struct (e.g., `struct BigStruct make_big()`) is actually writing to its shadow stack frame in linear memory. From the Wasm perspective, it performs `i32.store` instructions — it's not pure with respect to memory.

### 3.6 Stack Isolation in the Verified Backend

The shadow stack pattern is important for verification because it enables **stack frame isolation proofs**:

```rust
// wasm-verify can prove that a function only accesses its own stack frame:
//
// 1. Identify __stack_pointer adjustments at function entry/exit
// 2. Prove all memory accesses in the function are within
//    [__stack_pointer, __stack_pointer + frame_size)
// 3. Prove the stack pointer is restored on exit
//
// This lets the verified backend emit:

// PROOF: stack_frame_0x1000 — all accesses in this function target
//        [sp, sp+128), which is this function's stack frame
fn make_big<'mem, const MAX_PAGES: usize>(
    memory: &'mem mut IsolatedMemory<MAX_PAGES>,
    globals: &mut Globals,
    return_ptr: u32,
) -> WasmResult<()> {
    // Stack pointer adjustment
    let sp = globals.stack_pointer;
    globals.stack_pointer = sp - 128; // allocate 128 bytes of stack frame

    // All stores target [sp-128, sp) — provably within this frame
    // PROOF: bounds_0x1002 — sp-128+0 ∈ [sp-128, sp)
    unsafe { memory.store_unchecked::<i32>((sp - 128) as usize, 42) };

    // Copy result to return_ptr (caller's memory)
    // PROOF: bounds_0x1010 — return_ptr ∈ [caller_sp-N, caller_sp)
    unsafe { memory.store_unchecked::<i32>(return_ptr as usize, 42) };

    // Restore stack pointer
    globals.stack_pointer = sp;
    Ok(())
}
```

This analysis enables two optimizations:
- **Stack frame purity**: if a function only writes to its own stack frame and the return pointer, the verified backend can prove it has no side effects on heap or global data.
- **Stack bounds elimination**: if the stack pointer is proven to remain within the stack region, all stack-relative accesses can be proven in bounds without per-access checks.

### 3.7 Compile-Time Guarantees

- **Spatial safety**: All memory accesses bounds-checked against `active_pages * PAGE_SIZE` at runtime (safe backend) or proven within `MAX_PAGES * PAGE_SIZE` at compile time (verified backend)
- **Temporal safety**: Rust's lifetime system prevents use-after-free
- **Isolation**: Each module has its own `IsolatedMemory` instance — distinct types, distinct backing arrays, no cross-module access possible
- **Read-only regions**: Not a Wasm primitive, but provable via `wasm-verify` analysis (e.g., data segments never targeted by stores)
- **Stack frame isolation**: Provable via `wasm-verify` analysis of the `__stack_pointer` pattern

---

## 4. Module Representation

A WebAssembly module is composed of functions, globals, tables, and optionally a linear memory. Not all modules define their own memory — a module may import memory from another module, analogous to a shared library using its host process's address space.

### 4.1 Module Anatomy

A transpiled module maps to a Rust struct containing the module's own state:

```rust
/// A module that defines its own memory (like a POSIX process).
/// Capabilities are not encoded here — they are expressed as trait bounds
/// on the module's functions (see §5).
///
/// G is the transpiler-generated globals struct (one typed field per
/// mutable Wasm global). Immutable globals are emitted as Rust `const`.
struct Module<G, const MAX_PAGES: usize, const TABLE_SIZE: usize> {
    memory: IsolatedMemory<MAX_PAGES>,
    globals: G,
    table: Table<TABLE_SIZE>,
}

/// A module that does NOT define its own memory (like a shared library).
/// It has its own globals and table, but operates on borrowed memory.
struct LibraryModule<G, const TABLE_SIZE: usize> {
    globals: G,
    table: Table<TABLE_SIZE>,
    // no memory field — uses caller's memory
}
```

```{spec} Two Module Types: Ownership vs Borrowing
:id: SPEC_MODULE_TYPES
:status: open
:tags: modules, memory-ownership
Two module types exist: Module<G, MAX_PAGES, TABLE_SIZE> owns its own isolated memory
(process-like). LibraryModule<G, TABLE_SIZE> borrows memory from the caller (library-like).
This distinction enforces spatial isolation—one module's memory is inaccessible to others.
```

```{spec} Mutable Globals as Struct Fields
:id: SPEC_MODULE_GLOBALS_STRUCT
:status: open
:tags: modules, globals, transpilation
Mutable Wasm globals are transpiled to typed struct fields in a generated Globals struct.
Immutable globals are Rust const items (zero cost). One Globals struct per module,
generated at transpilation time. No dynamic lookup—direct field access.
```

```{spec} Table for Indirect Call Dispatch
:id: SPEC_MODULE_TABLE
:status: open
:tags: modules, table, call_indirect
Each module has a Table<TABLE_SIZE> containing FuncRef entries (type_index + func_index).
Used for call_indirect dispatch. Can be optimized to static match when never mutated
(const-table optimization).
```

```{spec} Memory Ownership Enforces Isolation
:id: SPEC_MODULE_OWNERSHIP_ISOLATION
:status: open
:tags: modules, isolation, memory-ownership
Memory ownership is the primary mechanism for spatial isolation. A module with its own
memory cannot access another module's memory. The Rust borrow checker and type system
guarantee this structurally—no runtime checks needed.
```

### 4.2 Globals and Table

```rust
/// Module-level global variables, transpiled from Wasm globals.
/// Each mutable global is a typed field — no dynamic lookup, no enum
/// indirection. The transpiler generates one such struct per module.
/// Immutable globals are emitted as Rust `const` items (zero cost).
struct Globals {
    g0: i32,      // (global (mut i32) ...) — mutable, lives in struct
    // g1 is immutable → emitted as `const G1: i64 = 42;` instead
    // ... one field per mutable Wasm global, named by index or export name
}

/// Indirect call table (for `call_indirect`).
/// Stores FuncRef entries (type_index + func_index); the transpiler
/// generates a match-based dispatch over func_index to call the
/// correct concrete Rust function after verifying the type_index.
///
/// For modules that never mutate the table (no `table.set`/`table.grow`),
/// the transpiler can bypass the runtime Table entirely and emit a
/// static match dispatch — see the const-table optimization.
struct Table<const MAX_SIZE: usize> {
    entries: [Option<FuncRef>; MAX_SIZE],
    active_size: usize,
}

struct FuncRef {
    type_index: u32,   // Wasm type section index for signature check
    func_index: u32,   // index into module function space → match dispatch
}
```

### 4.3 Memory Ownership Model

The key distinction is whether a module **owns** or **borrows** its linear memory:

| Wasm declaration | Rust representation | Analogy |
|------------------|---------------------|---------|
| Module defines memory | `Module` owns `IsolatedMemory` | POSIX process — has its own address space |
| Module imports memory | `LibraryModule` borrows `&mut IsolatedMemory` | Shared library — uses caller's address space |
| Module has no memory | `LibraryModule` with no memory parameter | Pure computation — no memory access |

#### Module with own memory

```rust
impl<G, const MAX_PAGES: usize, const TABLE_SIZE: usize> Module<G, MAX_PAGES, TABLE_SIZE> {
    /// Create a new module. The transpiler generates a wrapper that calls
    /// this with the correct initial values (data segments, element segments,
    /// global initializers).
    fn new(initial_pages: usize, globals: G, table: Table<TABLE_SIZE>) -> Self { /* ... */ }

    // The transpiler generates exported methods on the concrete module type:
    // fn call_export_add(&mut self, a: i32, b: i32) -> WasmResult<i32> {
    //     func_add(&mut self.memory, &mut self.globals, a, b)
    // }
}
```

#### Library module (borrows caller's memory)

```rust
impl<G, const TABLE_SIZE: usize> LibraryModule<G, TABLE_SIZE> {
    fn new(globals: G, table: Table<TABLE_SIZE>) -> Self { /* ... */ }

    // The transpiler generates exported methods that borrow the caller's memory:
    // fn call_export_transform<const MAX_PAGES: usize>(
    //     &mut self,
    //     memory: &mut IsolatedMemory<MAX_PAGES>,
    //     ptr: u32,
    //     len: u32,
    // ) -> WasmResult<i32> {
    //     func_transform(memory, &mut self.globals, ptr, len)
    // }
}
```

### 4.4 Inter-Module Calls

When a module that owns memory calls into a library module, it lends its memory for the duration of the call:

```rust
// Caller (owns memory) calls into library (borrows memory)
let mut app = Module::<AppGlobals, 256, 0>::new(16, AppGlobals::default(), Table::try_new(0));
let mut lib = LibraryModule::<LibGlobals, 0>::new(LibGlobals::default(), Table::try_new(0));

// The caller's memory is borrowed by the library for this call.
// Rust's borrow checker guarantees:
// - The library cannot store the memory reference beyond this call
// - The library cannot access memory from any other module
// - The caller cannot use its memory while the library holds it
let result = lib.call_export_transform(&mut app.memory, ptr, len)?;

// After the call, the caller has full ownership of its memory again.
// Capabilities are enforced via trait bounds on the functions, not on the struct (see §5).
```

### 4.5 Isolation Guarantees from the Ownership Model

This model enforces freedom from interference structurally:

1. **A module with its own memory** cannot access another module's memory — each `Module` owns a distinct `IsolatedMemory` instance, and there is no API to reach across.
2. **A library module** can only access the specific memory it was handed via `&mut` borrow. It cannot:
   - Hold onto the memory reference after the call returns (lifetime enforced)
   - Access a different module's memory (it only sees what it's given)
   - Access memory concurrently with the caller (`&mut` is exclusive)
3. **A pure module** (no memory at all) cannot perform any memory operations — the type system simply doesn't provide memory access methods.

This maps directly to the freedom-from-interference requirements in §12.2: the Rust type system makes cross-module memory access a **compile error**, not a runtime check.

---

## 5. Imports, Exports, and Capabilities as Traits

Capabilities are not encoded as bitflags — they are Rust **traits**. A Wasm module's imports become trait bounds on its functions; its exports become trait implementations. The host (kernel/OS layer) implements the system-level traits. If a required trait isn't provided, the code doesn't compile.

This maps directly to the WebAssembly import/export model and provides compile-time capability enforcement with zero runtime cost (monomorphization eliminates all indirection).

### 5.1 Imports as Trait Bounds

Each group of Wasm imports translates to a Rust trait. The transpiled module's functions are generic over the traits they need:

```rust
// Generated from: (import "env" "socket_open" (func ...))
//                 (import "env" "socket_read" (func ...))
trait SocketOps {
    fn socket_open(&mut self, domain: i32, sock_type: i32) -> WasmResult<i32>;
    fn socket_read(&mut self, fd: i32, buf_ptr: u32, len: u32) -> WasmResult<i32>;
}

// Generated from: (import "env" "fd_write" (func ...))
//                 (import "env" "fd_read"  (func ...))
trait FileOps {
    fn fd_write(&mut self, fd: i32, iovs: u32, iovs_len: u32, nwritten: u32) -> WasmResult<i32>;
    fn fd_read(&mut self, fd: i32, iovs: u32, iovs_len: u32, nread: u32) -> WasmResult<i32>;
}
```

A transpiled function that calls imported functions requires the corresponding traits:

```rust
impl<const MAX_PAGES: usize> MyModule<MAX_PAGES> {
    // This function calls socket_open and fd_write internally,
    // so it requires a host that provides both traits.
    fn send_to_file<H: SocketOps + FileOps>(
        &mut self,
        host: &mut H,
        addr: u32,
        fd: i32,
    ) -> WasmResult<i32> {
        let sock = host.socket_open(2, 1)?;
        let n = host.socket_read(sock, addr, 1024)?;
        host.fd_write(fd, addr, n as u32, 0)
    }
}
```

If the module doesn't import socket functions, there is no `SocketOps` bound — and no way to call socket functions. The capability is the trait bound itself.

```{spec} WebAssembly Imports as Trait Bounds
:id: SPEC_IMPORTS_AS_TRAIT_BOUNDS
:status: open
:tags: imports, traits, capabilities
Wasm module imports become Rust trait bounds on the generic host parameter H.
Each group of related imports (e.g., socket operations) maps to one trait.
The transpiler generates these traits from the module's import section.
```

```{spec} WebAssembly Exports as Trait Implementations
:id: SPEC_EXPORTS_AS_TRAIT_IMPLEMENTATIONS
:status: open
:tags: exports, traits, modules
Wasm module exports become trait implementations on the transpiled Module struct.
Each module's exported functions are expressed as a trait, and the transpiled module
implements it. Enables inter-module linking via trait composition.
```

```{spec} Zero-Cost Trait Dispatch via Monomorphization
:id: SPEC_TRAITS_ZERO_COST_DISPATCH
:status: open
:tags: traits, dispatch, monomorphization, performance
All capability dispatch is via monomorphization, not vtables or trait objects.
If a module doesn't import a capability, the trait bound doesn't exist and no code
path to call that capability is generated. Zero-cost abstraction.
```

```{spec} WASI as Standard Trait Set
:id: SPEC_WASI_STANDARD_TRAITS
:status: open
:tags: wasi, imports, traits, standard-library
WASI (WebAssembly System Interface) support is implemented as a standard set of
traits: WasiFd, WasiPath, WasiClock, WasiRandom, etc. Shipped by herkos-runtime.
The host implements whichever subset it supports.
```

```{spec} Natural Trait Composition for Capabilities
:id: SPEC_CAPABILITY_COMPOSITION
:status: open
:tags: traits, capabilities, composition
Multiple trait bounds compose naturally. A module requiring both file and network
access requires both bounds: fn module_fn<H: FileOps + SocketOps>(host: &mut H).
Enables fine-grained capability specification.
```

### 5.2 Exports as Trait Implementations

Each module's exported functions are expressed as a trait, and the transpiled module implements it:

```rust
// Generated from: (export "transform" (func $transform))
//                 (export "init"      (func $init))
trait ImageLibExports {
    fn transform(&mut self, ptr: u32, len: u32) -> WasmResult<i32>;
    fn init(&mut self) -> WasmResult<()>;
}

// The transpiled module implements its export trait
impl<const MAX_PAGES: usize> ImageLibExports for ImageModule<MAX_PAGES> {
    fn transform(&mut self, ptr: u32, len: u32) -> WasmResult<i32> {
        // generated function body
        func_transform(&mut self.memory, &mut self.globals, ptr, len)
    }

    fn init(&mut self) -> WasmResult<()> {
        func_init(&mut self.memory, &mut self.globals)
    }
}
```

### 5.3 Inter-Module Linking via Traits

When module A imports a function that module B exports, the wiring happens through trait bounds:

```rust
// Module A imports "transform" from "image_lib"
// Generated trait for this import:
trait ImageLibImports {
    fn transform(&mut self, ptr: u32, len: u32) -> WasmResult<i32>;
}

// Module A's functions require this trait:
impl<const MAX_PAGES: usize> ModuleA<MAX_PAGES> {
    fn process_image<H: ImageLibImports>(
        &mut self,
        host: &mut H,
        ptr: u32,
        len: u32,
    ) -> WasmResult<i32> {
        host.transform(ptr, len)
    }
}

// Module B already implements ImageLibExports (which has the same signature).
// The host wires them together:
impl ImageLibImports for ImageModule<256> {
    fn transform(&mut self, ptr: u32, len: u32) -> WasmResult<i32> {
        <Self as ImageLibExports>::transform(self, ptr, len)
    }
}
```

### 5.4 The Host as Kernel

The host is the component that instantiates modules and provides system-level capabilities. It implements the traits that modules need:

```rust
/// The host implements system-level traits for modules that require them.
struct Host {
    // OS resources, file descriptors, sockets, etc.
}

impl SocketOps for Host {
    fn socket_open(&mut self, domain: i32, sock_type: i32) -> WasmResult<i32> {
        // Actual syscall to the OS/RTOS — this is the only place
        // where real system interaction happens.
    }
    fn socket_read(&mut self, fd: i32, buf_ptr: u32, len: u32) -> WasmResult<i32> {
        // ...
    }
}

impl FileOps for Host {
    // ...
}
```

A module that doesn't import socket functions never gets `SocketOps` — there is no code path to call `socket_open`. The capability simply doesn't exist in the type system.

### 5.5 Capability Composition

Multiple traits compose naturally. A module that needs both file and network access requires both bounds:

```rust
// Module requires both — caller must provide both.
fn complex_operation<H: SocketOps + FileOps>(host: &mut H, ...) -> WasmResult<i32> { ... }

// Module requires neither — no host parameter at all, pure computation.
fn pure_math(a: i32, b: i32) -> i32 { a.wrapping_add(b) }
```

This replaces the bitflag approach entirely. The advantages:

| Aspect | Bitflags (`const CAPS: u64`) | Traits |
|--------|------------------------------|--------|
| Granularity | Coarse (1 bit = 1 capability class) | Fine (exact function signatures) |
| Compile-time checking | Fails if bit not set | Fails if trait not implemented |
| Error messages | Opaque bit mismatch | Clear: "trait `SocketOps` not implemented for `Host`" |
| Runtime cost | Zero (const generic) | Zero (monomorphization) — but see §13.3 for binary size impact |
| Extensibility | Limited to 64 bits | Unlimited — add new traits freely |
| Inter-module linking | Not supported | Natural — traits are the linking contract |

### 5.6 WASI Support

The [WebAssembly System Interface (WASI)](https://wasi.dev/) defines a standardized set of imports for system interaction (file I/O, clocks, random, sockets, etc.). In this model, WASI is simply a collection of pre-defined import traits shipped by `herkos-runtime`:

```rust
// Provided by herkos-runtime — these are the WASI "preview 1" imports
// expressed as traits. The host implements whichever subset it supports.

/// WASI file descriptor operations
trait WasiFd {
    fn fd_read(&mut self, fd: i32, iovs: u32, iovs_len: u32, nread: u32) -> WasmResult<i32>;
    fn fd_write(&mut self, fd: i32, iovs: u32, iovs_len: u32, nwritten: u32) -> WasmResult<i32>;
    fn fd_close(&mut self, fd: i32) -> WasmResult<i32>;
    fn fd_seek(&mut self, fd: i32, offset: i64, whence: i32, newoffset: u32) -> WasmResult<i32>;
    // ...
}

/// WASI path operations
trait WasiPath {
    fn path_open(&mut self, dirfd: i32, dirflags: i32, path: u32, path_len: u32,
                 oflags: i32, rights_base: i64, rights_inheriting: i64,
                 fdflags: i32, fd: u32) -> WasmResult<i32>;
    // ...
}

/// WASI clock operations
trait WasiClock {
    fn clock_time_get(&mut self, clock_id: i32, precision: i64, time: u32) -> WasmResult<i32>;
}

/// WASI random
trait WasiRandom {
    fn random_get(&mut self, buf: u32, len: u32) -> WasmResult<i32>;
}
```

A Wasm module compiled with WASI support (e.g., via `clang --target=wasm32-wasi`) imports functions like `fd_write` from the `wasi_snapshot_preview1` namespace. The transpiler maps these to the corresponding WASI trait bounds:

```rust
// Transpiled module that uses WASI fd_write and clock_time_get
impl<const MAX_PAGES: usize> MyWasiModule<MAX_PAGES> {
    fn main<H: WasiFd + WasiClock>(
        &mut self,
        host: &mut H,
    ) -> WasmResult<i32> {
        // Can call host.fd_write() — WasiFd is in bounds
        // Can call host.clock_time_get() — WasiClock is in bounds
        // Cannot call host.random_get() — WasiRandom is NOT in bounds
        // ...
    }
}
```

The host implements the WASI traits it supports:

```rust
// Bare-metal host: only supports fd_write (e.g., UART output) and clock
struct EmbeddedHost { /* ... */ }
impl WasiFd for EmbeddedHost { /* UART-backed fd_write, others return ENOSYS */ }
impl WasiClock for EmbeddedHost { /* hardware timer */ }
// WasiPath, WasiRandom: not implemented → modules that need them won't compile

// Full POSIX host: supports everything
struct PosixHost { /* ... */ }
impl WasiFd for PosixHost { /* real POSIX file ops */ }
impl WasiPath for PosixHost { /* real POSIX path ops */ }
impl WasiClock for PosixHost { /* clock_gettime */ }
impl WasiRandom for PosixHost { /* /dev/urandom */ }
```

This gives fine-grained, compile-time WASI support:
- **Partial WASI is natural** — just implement the traits you need. An embedded host that only supports `fd_write` over UART doesn't need to stub out hundreds of functions.
- **No WASI is also natural** — if the module doesn't import WASI functions, there are no WASI trait bounds. Pure computation modules have zero WASI overhead.
- **Custom extensions beyond WASI** — the host can define additional traits for platform-specific capabilities (e.g., `GpioOps`, `CanBusOps`) that coexist with WASI traits.

---

## 6. Runtime Implementation

### 6.1 `no_std` Constraint

The transpiled Rust code and the `herkos-runtime` library **must be `#![no_std]`**. This is non-negotiable — the primary targets are embedded and safety-critical systems where `std` is unavailable.

Implications:
- No heap allocation in the default configuration (`no_std` without `alloc`)
- `IsolatedMemory` uses a fixed-size backing array, not `Vec`
- Error handling via `Result<T, WasmTrap>` — no panics, no `format!`, no `String`
- An optional `alloc` feature gate may be provided for targets that have a global allocator (e.g., `memory.grow` support)

### 6.2 Minimal Runtime Requirements

The transpiled Rust code requires a minimal runtime providing:

- **Memory setup**: static or linker-provided backing memory for `IsolatedMemory`
- **System call interface**: mediated access to OS/RTOS services (optional — only if the module has capabilities that require it)
- **Trap handling**: Wasm traps map to `Result::Err(WasmTrap)` — no panics, no unwinding

### 6.3 Zero-Cost Abstractions

The runtime must compile away to minimal overhead:

- Inline all memory access wrappers
- Const-evaluate all compile-time checks
- Use link-time optimization to eliminate abstraction layers
- No vtables, no trait objects, no dynamic dispatch in the hot path

```{spec} Runtime Library Provides Core Types
:id: SPEC_RUNTIME_LIBRARY_PROVIDES
:status: open
:tags: runtime, herkos-runtime, library
herkos-runtime provides: IsolatedMemory type, import/export trait definitions,
standard Wasm runtime functions (memory.grow, memory.size), WasmTrap and WasmResult
types. All transpiled code depends on this single runtime crate.
```

```{spec} Wasm Traps Map to Result Errors
:id: SPEC_TRAP_MAPPING
:status: open
:tags: traps, error-handling, result
Wasm traps map to WasmTrap enum discriminants returned as Result::Err. Trap types:
OutOfBounds, DivisionByZero, IntegerOverflow, Unreachable, IndirectCallTypeMismatch,
TableOutOfBounds, UndefinedElement. No exceptions, no unwinding.
```

```{spec} Memory Growth Semantics
:id: SPEC_MEMORY_GROWTH
:status: open
:tags: memory, memory.grow, growth
memory.grow increments active_pages by delta. Returns previous page count (as i32) on
success, -1 on failure (would exceed MAX_PAGES). New pages are zero-initialized per
WebAssembly specification. No allocation occurs.
```

```{spec} Primary Integration: Trait-Based
:id: SPEC_PRIMARY_TRAIT_INTEGRATION
:status: open
:tags: integration, traits, api
Recommended integration pattern: host implements import traits, instantiates Module
with initial memory and globals, passes &mut Host to exported functions. Full type
safety, zero unsafe, zero-cost dispatch via monomorphization.
```

```{spec} Optional C-Compatible ABI Wrapper
:id: SPEC_C_ABI_OPTIONAL
:status: open
:tags: integration, c-abi, ffi, interoperability
Optional C-compatible ABI wrapper with extern "C" functions for integration with non-Rust
hosts. Erases generics using opaque types. Uses unsafe and raw pointers—escape hatch only,
not the default integration mechanism.
```

---

## 7. Verification Metadata Format

The verification metadata is the bridge between the analysis tool (`wasm-verify`) and the transpiler (`herkos`). Understanding the data flow is essential:

```
┌──────────────┐          ┌──────────────┐          ┌──────────────┐
│  .wasm file  │ ───────> │  wasm-verify │ ───────> │  metadata    │
│  (input)     │          │  (analyzer)  │          │  (TOML file) │
└──────────────┘          └──────────────┘          └──────┬───────┘
                                                          │
                                                          ▼
┌──────────────┐          ┌──────────────┐          ┌──────────────┐
│  .wasm file  │ ───────> │  herkos   │ <─────── │  metadata    │
│  (input)     │          │  (transpiler)│          │  (TOML file) │
└──────────────┘          └──────────────┘          └──────────────┘
                                │
                                ▼
                          ┌──────────────┐
                          │  .rs file    │
                          │  (output)    │
                          └──────────────┘
```

- **`wasm-verify` produces the metadata** (output). It statically analyzes the `.wasm` binary and generates proof artifacts: which memory accesses are provably in bounds, which regions are never written, which stack frames are isolated, etc.
- **`herkos` consumes the metadata** (input). It reads the proof artifacts and uses them to decide, per memory access, whether to emit a safe bounds-checked call or an `unsafe` unchecked call justified by the proof.
- **The safe backend does not require metadata at all** — it emits runtime checks everywhere.
- **The verified backend requires complete metadata** — every access must have a proof, or transpilation fails.
- **The hybrid backend accepts partial metadata** — proven accesses get `unsafe`, unproven accesses get runtime checks.

### 7.1 Metadata Schema

```toml
[module]
name = "example_module"
source_hash = "sha256:abcdef..."  # hash of the .wasm binary this metadata was generated from

[memory]
initial_pages = 16
maximum_pages = 256

# Regions identified by analysis — NOT a Wasm primitive.
# wasm-verify infers these from access pattern analysis.
[[memory.regions]]
offset = 0
size = 1024
access = "read-write"
purpose = "stack"

[[memory.regions]]
offset = 1024
size = 4096
access = "read-only"           # proven: no store instruction targets this range
purpose = "data_segment"

# Per-access proof artifacts.
# Each entry corresponds to a single memory operation in the Wasm bytecode.
[[proofs]]
instruction_offset = 0x1234
kind = "bounds"                 # what is proven
access = "load"                 # load or store
type = "i32"                    # Wasm type
proven_bounds = { min = 0, max = 65532 }  # address ∈ [min, max]
proof_method = "smt"            # how it was proven (smt, abstract_interp, trivial)

[[proofs]]
instruction_offset = 0x1238
kind = "bounds"
access = "store"
type = "i32"
proven_bounds = { min = 1024, max = 2044 }
proof_method = "abstract_interp"

[[proofs]]
instruction_offset = 0x1240
kind = "arithmetic"             # integer overflow proof
expression = "local.get 0 * 4"
proven_range = { min = 0, max = 65532 }
proof_method = "smt"

# Stack frame isolation proofs (see §3.6).
[[stack_frames]]
function_index = 12
function_name = "make_big"
frame_size = 128
stack_pointer_global = 0
proven_isolated = true          # all accesses within [sp-frame_size, sp)
proof_method = "abstract_interp"
```

### 7.2 What Gets Verified

`wasm-verify` performs the following analyses and records proof artifacts for each:

| Analysis | What it proves | Enables |
|----------|---------------|---------|
| **Memory bounds** | A load/store address is within `[0, active_pages * PAGE_SIZE)` | Replacing `memory.load()` with `memory.load_unchecked()` |
| **Arithmetic overflow** | An integer operation cannot overflow for the proven value ranges | Replacing `checked_add` with `wrapping_add` or plain `+` |
| **Read-only regions** | No store instruction targets a given address range (e.g., data segments) | Optimization: the transpiler can skip alias analysis for these regions |
| **Stack frame isolation** | A function only accesses memory within its own stack frame (§3.6) | Batch-proving all accesses in a function instead of one-by-one |
| **Stack bounds** | The stack pointer remains within the stack region across all paths | Eliminating per-access stack bounds checks |

### 7.3 Metadata Integrity

The metadata file includes a `source_hash` of the `.wasm` binary it was generated from. The transpiler **must** verify this hash matches the `.wasm` file it is transpiling — otherwise the proofs are meaningless (they were generated for a different binary).

```
herkos input.wasm --metadata verification.toml --mode verified
# Step 1: check sha256(input.wasm) == verification.toml[module.source_hash]
# Step 2: for each memory access in input.wasm, look up proof in verification.toml
# Step 3: emit unsafe (if proof found) or reject (verified mode) / fallback (hybrid mode)
```

```{spec} External TOML Verification Metadata
:id: SPEC_VERIFICATION_METADATA
:status: open
:tags: verification, metadata, proof-artifacts
External TOML metadata bridges wasm-verify (analyzer) and herkos (transpiler).
Contains proof artifacts (bounds, arithmetic, stack frame proofs) indexed by instruction
offset. See §7 for complete schema. Enables three-stage pipeline.
```

```{spec} Metadata Integrity via Source Hash
:id: SPEC_METADATA_INTEGRITY
:status: open
:tags: verification, metadata, hash, integrity
Metadata file includes source_hash (SHA256) of the .wasm binary it was generated from.
Transpiler must verify this hash matches the .wasm file being transpiled. Otherwise
proofs are for a different binary and are meaningless.
```

```{spec} no_std Constraint for Generated Code
:id: SPEC_NO_STD_CONSTRAINT
:status: open
:tags: no_std, embedded, safety
All transpiled output and herkos-runtime must be #![no_std]. No heap allocation in
default configuration. No panics, no format!, no String in generated code or runtime.
Enables embedded and safety-critical targets.
```

```{spec} Result-Based Error Handling
:id: SPEC_ERROR_HANDLING_RESULT
:status: open
:tags: error-handling, traps, result
All error paths use Result<T, WasmTrap>. Wasm traps (OutOfBounds, IntegerOverflow,
DivisionByZero, etc.) map to WasmTrap enum discriminants. No panics, no unwinding in
transpiled output. Deterministic error handling.
```

```{spec} Proof References in Unsafe Blocks
:id: SPEC_PROOF_REFERENCED
:status: open
:tags: unsafe, proofs, verification, comments
In verified and hybrid backends, every unsafe block carries a // PROOF: comment
referencing the verification metadata artifact. Proves why the unsafe is sound.
Enables auditing and proof traceability.
```

```{spec} Safety Comments in Runtime Unsafe
:id: SPEC_PROOF_SAFETY_COMMENT
:status: open
:tags: unsafe, safety, comments, runtime
In the herkos-runtime crate, every unsafe block carries a // SAFETY: comment
explaining the invariant being maintained. Documents why the unsafe code is correct.
```

---

## 8. Transpilation Rules

### 8.1 Function Translation

WebAssembly functions map to Rust functions. Capabilities (imports) become trait bounds on a generic host parameter (see §5):

```rust
// Wasm: (func $example (param i32) (result i32))
// This function calls no imports — no host parameter needed.
fn example<'mem, const MAX_PAGES: usize>(
    memory: &'mem mut IsolatedMemory<MAX_PAGES>,
    globals: &mut Globals,
    param0: i32,
) -> i32 {
    // Function body — pure computation over memory and globals
}

// Wasm: (func $send (param i32 i32) (result i32))
// This function calls imported socket and file functions.
fn send<'mem, const MAX_PAGES: usize, H: SocketOps + FileOps>(
    memory: &'mem mut IsolatedMemory<MAX_PAGES>,
    globals: &mut Globals,
    host: &mut H,
    param0: i32,
    param1: i32,
) -> WasmResult<i32> {
    // Function body — can call host.socket_open(), host.fd_write(), etc.
}
```

### 8.2 Control Flow

- Wasm `block`, `loop`, `if` → Rust control flow structures (all labeled)
- Wasm `br`, `br_if`, `br_table` → labeled breaks or match statements
- Wasm `call`, `call_indirect` → Rust function calls with capability propagation

### 8.3 Memory Operations

The emitted code depends on the selected backend (see §9):

```rust
// Safe / Hybrid fallback — bounds-checked, returns Result
let value = memory.load_i32(address)?;
memory.store_i32(address, value)?;

// Verified / Hybrid proven — unchecked, justified by proof artifact
// PROOF: bounds_check_0xABCD
let value = unsafe { memory.load_i32_unchecked(address) };
// PROOF: bounds_check_0xABCE
unsafe { memory.store_i32_unchecked(address, value) };
```

### 8.4 Error Handling

Wasm traps are translated to Rust `Result` types:

```rust
enum WasmTrap {
    OutOfBounds,
    DivisionByZero,
    IntegerOverflow,
    Unreachable,
    IndirectCallTypeMismatch,
    TableOutOfBounds,
    UndefinedElement,
}

type WasmResult<T> = Result<T, WasmTrap>;
```

### 8.5 Function Calls

WebAssembly has two calling mechanisms: **direct calls** (`call`) and **indirect calls** (`call_indirect`). Both are transpiled into safe Rust function calls with no dynamic dispatch overhead.

#### 8.5.1 Direct Calls (`call`)

A direct call invokes a function by its index in the module's function space. The transpiler emits a Rust function call to `func_{index}(...)`, forwarding the module's shared state (globals, memory, table) as needed:

```rust
// Wasm: call $func_3 (with 2 args on the stack)
// Transpiled (safe backend):
v5 = func_3(v3, v4, globals, memory, table)?;
```

Only the state parameters that the module actually uses are passed. A module with no memory omits the `memory` parameter; a module with no table omits `table`; a module with no mutable globals omits `globals`.

#### 8.5.2 Indirect Calls (`call_indirect`)

`call_indirect` implements function pointers in WebAssembly. It pops a table index from the stack, looks up a `FuncRef` entry in the module's table, verifies that the function's signature matches the expected type, and dispatches to the concrete function. This is used to implement C function pointers, vtables, and similar dynamic dispatch patterns.

**Runtime semantics** (per Wasm spec §4.4.9):

1. Pop the table index from the operand stack
2. Look up `table[index]` — trap with `TableOutOfBounds` if index ≥ table size
3. If the entry is `None` (uninitialized slot), trap with `UndefinedElement`
4. Compare the entry's type against the expected type — trap with `IndirectCallTypeMismatch` if they differ
5. Call the function with the arguments from the stack

**Transpiled pattern** (safe backend):

```rust
// Wasm: call_indirect (type $binop)  ; expects (i32, i32) -> i32
// Where table[0] = add, table[1] = sub, table[2] = mul
//
// Transpiled:
let __entry = table.get(v2 as u32)?;                          // Step 2-3
if __entry.type_index != 0 {                                   // Step 4
    return Err(WasmTrap::IndirectCallTypeMismatch);
}
v4 = match __entry.func_index {                                // Step 5
    0 => func_0(v0, v1, table)?,    // add
    1 => func_1(v0, v1, table)?,    // sub
    2 => func_2(v0, v1, table)?,    // mul
    _ => return Err(WasmTrap::UndefinedElement),
};
```

The `match` dispatch is **static** — the transpiler enumerates all functions in the module whose signature matches the expected type. This avoids function pointers, trait objects, or any form of dynamic dispatch. The `_ =>` arm is a safety net for func_index values that don't correspond to any function of the right type (which would indicate a corrupted or malicious table entry).

**Why match-based dispatch?** The alternatives — function pointer arrays, `dyn Fn` trait objects, or computed gotos — all require either `unsafe` code, heap allocation, or lose `no_std` compatibility. A match statement is 100% safe, `no_std` compatible, and optimizes to a jump table under LLVM when the arms are dense.

#### 8.5.3 Table Initialization

The module's `Table<TABLE_MAX>` is initialized from element segments during module construction. Each element segment specifies a table offset and a list of function indices to install:

```rust
// From: (elem (i32.const 0) $add $sub $mul $negate)
let mut table = Table::try_new(5);
table.set(0, Some(FuncRef { type_index: 0, func_index: 0 })).unwrap(); // add
table.set(1, Some(FuncRef { type_index: 0, func_index: 1 })).unwrap(); // sub
table.set(2, Some(FuncRef { type_index: 0, func_index: 2 })).unwrap(); // mul
table.set(3, Some(FuncRef { type_index: 1, func_index: 3 })).unwrap(); // negate
```

Each `FuncRef` stores two fields:
- **`type_index`**: the canonical type index of the function's signature (used for the type check in step 4)
- **`func_index`**: the function's index in the module (used for dispatch in step 5)

The table is passed as `&Table<TABLE_MAX>` to all functions in modules that use indirect calls. Functions that don't use `call_indirect` simply ignore it.

#### 8.5.4 Structural Type Equivalence

The Wasm spec (§4.4.9) requires `call_indirect` type checks to use **structural equivalence**: two type indices are considered matching if and only if they have the same parameter types and the same result types, regardless of whether they are the same type index. For example:

```text
(type $t0 (func (param i32 i32) (result i32)))  ;; type index 0
(type $t1 (func (param i32 i32) (result i32)))  ;; type index 1 — structurally identical to $t0
```

A function of type `$t0` placed in the table must pass the type check when `call_indirect (type $t1)` is used, because `$t0` and `$t1` are structurally equivalent.

**Implementation**: The transpiler builds a **canonical type index mapping** at transpile time. For each type index in the Wasm type section, it finds the smallest index with an identical structural signature:

```
Type 0: (i32, i32) → i32  →  canonical = 0
Type 1: (i32, i32) → i32  →  canonical = 0  (same as type 0)
Type 2: (i32) → i32       →  canonical = 2  (new signature)
```

Both the `FuncRef.type_index` stored in the table and the type check in `call_indirect` use canonical indices. This ensures that structurally equivalent types always compare equal, as required by the spec.

This canonicalization is performed once during transpilation. At runtime, the type check is a simple integer comparison (`__entry.type_index != canonical_idx`) — no signature introspection is needed.

```{spec} Direct Calls as Regular Function Calls
:id: SPEC_CALL_DIRECT
:status: open
:tags: calls, dispatch, transpilation
Direct calls (Wasm call instruction) transpile to regular Rust function calls with
shared state (memory, globals, table) threaded through as parameters. No indirection—
direct dispatch to the concrete function.
```

```{spec} Safe Call Dispatch via Static Match
:id: SPEC_CALL_INDIRECT_SAFE_DISPATCH
:status: open
:tags: calls, dispatch, indirect, safety
Indirect calls (Wasm call_indirect) use static match dispatch over func_index, not
function pointers, vtables, or trait objects. 100% safe Rust, no dynamic dispatch
overhead. Match arms enumerate all functions matching the expected signature.
```

```{spec} Type Checking in Indirect Calls
:id: SPEC_CALL_INDIRECT_TYPE_CHECK
:status: open
:tags: calls, dispatch, indirect, type-safety
call_indirect validates function signature before dispatch: looks up table entry,
compares type_index against the expected type, traps with IndirectCallTypeMismatch
if they differ. Prevents type confusion attacks.
```

```{spec} Structural Type Equivalence for Signatures
:id: SPEC_CALL_STRUCTURAL_TYPE_EQUIVALENCE
:status: open
:tags: calls, types, type-equivalence
Type checks use structural equivalence: two type indices match if they have identical
parameter and result types, regardless of index. Transpiler builds canonical type index
mapping at transpile time. Runtime check is simple integer comparison.
```

```{spec} Table Initialization from Element Segments
:id: SPEC_TABLE_INITIALIZATION
:status: open
:tags: table, initialization, element-segments
Table is initialized from element segments during module construction. Each element
segment specifies a table offset and list of function indices to install as FuncRef
entries (with type_index and func_index).
```

---

## 9. Code Generation Backends

The transpiler ships three backends. Each emits structurally different Rust code, trading off between auditability, performance, and proof requirements. The backend is selected per-module at transpilation time.

### 9.1 Safe Backend (Default)

- Emits **100% safe Rust** — no `unsafe` blocks
- Every memory access goes through bounds-checked wrappers that return `Result<T, WasmTrap>`
- Every integer arithmetic operation uses `checked_*` methods
- Suitable for initial migration, testing, and modules where performance is not critical
- **No verification metadata required** — correctness relies entirely on runtime checks

```rust
// Safe backend: bounds-checked load
let value = memory.load_i32(offset as usize)?;
```

```{spec} Three Code Generation Backends
:id: SPEC_BACKENDS_THREE
:status: open
:tags: backends, code-generation, transpilation
Three distinct backends exist: Safe (runtime checks, no proofs), Verified (unsafe +
formal proofs, complete metadata required), Hybrid (proven unsafe, unproven checked).
Enables trade-offs between auditability, performance, and proof requirements. Backend
selected per-module at transpilation time.
```

```{spec} Safe Backend: Runtime Bounds Checking
:id: SPEC_BACKEND_SAFE
:status: open
:tags: backends, safe, runtime-checks
Safe backend emits 100% safe Rust. Every memory access goes through bounds-checked
wrappers returning Result<T, WasmTrap>. Integer arithmetic uses checked_* methods.
No verification metadata required. Overhead: 15–30%.
```

```{spec} Verified Backend: Proof-Justified Unsafe
:id: SPEC_BACKEND_VERIFIED
:status: open
:tags: backends, verified, unsafe, proofs
Verified backend emits unsafe Rust for operations with formal proofs. Requires
complete verification metadata—transpilation fails if any operation lacks a proof.
Each unsafe block carries // PROOF: reference to proof artifact. Overhead: 0–5%.
```

```{spec} Hybrid Backend: Partial Proof Coverage
:id: SPEC_BACKEND_HYBRID
:status: open
:tags: backends, hybrid, unsafe, proofs
Hybrid backend accepts partial verification metadata. Proven accesses emit unsafe with
// PROOF: references. Unproven accesses fall back to runtime checks. Enables iterative
proof improvement. Overhead: 5–15%. Practical for production use.
```

```{spec} Backend Selection at Transpilation Time
:id: SPEC_BACKEND_SELECTION
:status: open
:tags: backends, transpilation, cli
Backend is selected per-module at transpilation time via CLI flag (--mode safe|verified|hybrid).
Different modules in the same project can use different backends. Enables gradual migration
and mixed-criticality scenarios.
```

```{spec} Deterministic Code Generation
:id: SPEC_BACKEND_DETERMINISTIC
:status: open
:tags: backends, determinism, reproducibility
Generated output must be identical regardless of CPU, thread count, execution order,
or random seed. No non-deterministic collection types (HashMap iteration order).
Enables reproducible builds and auditable output.
```

### 9.2 Verified Backend

- Emits `unsafe` Rust for memory accesses and arithmetic where **formal proofs** guarantee safety
- Requires **complete verification metadata** — every memory access and arithmetic operation must have an associated proof
- Proofs are generated by `wasm-verify` (backed by SMT solvers) and recorded in the verification metadata
- Each `unsafe` block carries a machine-checkable proof reference (e.g., `// PROOF: bounds_check_0x1234`)
- **Compilation fails** if any operation lacks a proof — no silent fallback
- Target overhead: **0–5%** (function call indirection only)

```rust
// Verified backend: unchecked load justified by formal proof
// PROOF: bounds_check_0x1234 — offset ∈ [0, MEM_SIZE - 4] proven by wasm-verify
let value = unsafe { memory.load_i32_unchecked(offset as usize) };
```

The `unsafe` here is **not** hand-waved — it is backed by a machine-generated, auditable proof artifact. The proof obligations are:
- **Spatial**: the access `[offset, offset + size)` is within the linear memory bounds
- **Alignment**: the offset satisfies the type's alignment requirement
- **Arithmetic**: integer operations cannot overflow in the proven value ranges

### 9.3 Hybrid Backend

- Emits `unsafe` where proofs exist, safe runtime checks where they don't
- Partial verification metadata is sufficient — unproven accesses fall back to bounds-checked wrappers
- The transpiler annotates each fallback site so developers can iteratively improve proof coverage
- Practical choice for production use: maximize performance where provable, stay safe elsewhere
- Target overhead: **5–15%** depending on proof coverage

```rust
// Hybrid backend: mix of proven and checked accesses
// PROOF: bounds_check_0x1234
let a = unsafe { memory.load_i32_unchecked(proven_offset as usize) };
// UNPROVEN: fallback to runtime check
let b = memory.load_i32(dynamic_offset as usize)?;
```

### 9.4 Contract-Based Verification (Alternative to External Metadata)

Instead of — or in addition to — the external `wasm-verify` → TOML → `herkos` pipeline (§7), the verified and hybrid backends can emit Rust code annotated with **formal verification contracts** that are checked during Rust compilation. This eliminates the separate metadata file entirely: the proofs live in the generated code and are verified by the Rust toolchain itself.

**Candidate tools for contract-based verification:**

| Tool | Technique | Contract style | `no_std` | Status |
|------|-----------|---------------|----------|--------|
| **Kani** (AWS) | Bounded model checking (CBMC) | `#[kani::proof]` harnesses + `kani::assume`/`kani::assert` | Yes | Production-ready |
| **Flux-rs** (UC San Diego) | Refinement types | Type-level predicates: `i32{v: 0 <= v && v < bound}` | Partial | Research prototype |
| **Creusot** (INRIA) | Deductive verification (Why3) | `#[requires]` / `#[ensures]` / `#[invariant]` | Partial | Research |
| **Prusti** (ETH Zurich) | Deductive verification (Viper) | `#[requires]` / `#[ensures]` macros | Partial | Research |

#### Kani: bounded model checking of generated code

Kani is the most practical choice today. The transpiler emits proof harnesses alongside the generated functions:

```rust
// Generated function (verified backend)
fn sum_array(
    memory: &IsolatedMemory<256>,
    arr: u32,
    len: u32,
) -> i32 {
    // ... (function body with unchecked accesses)
}

// Generated Kani proof harness
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(64)]  // bound on loop iterations
fn verify_sum_array() {
    let memory = IsolatedMemory::<256>::new(16);
    let arr: u32 = kani::any();
    let len: u32 = kani::any();

    // Precondition: arr + len*4 fits in active memory
    kani::assume(arr as u64 + len as u64 * 4 <= 16 * 65536);

    // Kani exhaustively checks: no out-of-bounds, no overflow
    let result = sum_array(&memory, arr, len);
}
```

Running `cargo kani` verifies the harness — if it passes, **all** memory accesses in `sum_array` are proven safe for all inputs satisfying the precondition. No `unsafe` needed in the function body; the bounds checks can be elided because Kani has proven they never fail.

#### Flux-rs: refinement types for zero-cost compile-time bounds

Flux-rs is more ambitious but more elegant. Instead of proof harnesses, bounds constraints are encoded directly in the type system:

```rust
// Flux-rs refinement: offset is proven in bounds at the type level
#[flux_rs::sig(fn(memory: &IsolatedMemory<MAX_PAGES>,
                   offset: u32{v: v + 3 < MAX_PAGES * 65536}) -> i32)]
fn load_i32_verified(memory: &IsolatedMemory<MAX_PAGES>, offset: u32) -> i32 {
    // No runtime check, no unsafe — Flux proves offset is valid
    memory.load_proven::<i32>(offset as usize)
}
```

The Flux type checker verifies at compile time that every caller of `load_i32_verified` passes an `offset` satisfying the refinement predicate. If any call site can't be proven, compilation fails.

#### Choosing between external metadata and contract-based verification

| Aspect | External metadata (§7) | Contract-based (Kani/Flux) |
|--------|----------------------|---------------------------|
| Proof location | Separate TOML file | Embedded in Rust source |
| Synchronization | Requires hash check | Inherently in sync |
| Toolchain | `wasm-verify` + SMT solver | `cargo kani` or Flux rustc plugin |
| Auditability | TOML + `// PROOF:` comments | Proof harnesses are readable Rust |
| `no_std` compat | N/A (metadata is offline) | Kani: yes; Flux: partial |
| Maturity | Custom tool (must be built) | Kani: production; Flux: research |
| Build time impact | None (offline analysis) | Kani: significant; Flux: moderate |

**Recommended strategy**: use **both** approaches complementarily:
1. **Kani** for verifying the `herkos-runtime` runtime crate itself (prove that `IsolatedMemory::load`, `memory.grow`, trap handling are correct)
2. **External metadata** (§7) for the transpiled code in the initial implementation — it's simpler to build and doesn't add Kani's build time to every compilation
3. **Contract-based** as a future evolution — when Flux-rs matures, the verified backend can emit refinement-typed code instead of `unsafe` + metadata references, achieving provable safety with zero `unsafe` and zero runtime overhead

### 9.5 Backend Selection Guidance

| Backend  | `unsafe` in output | Proof requirement | Overhead | Use case |
|----------|--------------------|-------------------|----------|----------|
| Safe     | None               | None              | 15–30%   | Migration, testing, non-critical modules |
| Verified | All accesses       | Complete (metadata or contracts) | 0–5% | Performance-critical, fully analyzed modules |
| Hybrid   | Proven accesses    | Partial           | 5–15%    | Production — iterative proof improvement |

---

## 10. Integration Points

### 10.1 Integration via Traits (Primary)

The primary integration mechanism is Rust's type system. The host instantiates modules and interacts through the import/export traits defined in §5:

```rust
// Host implements the import traits the module needs
struct MyHost { /* platform resources */ }
impl SocketOps for MyHost { /* ... */ }
impl WasiFd for MyHost { /* ... */ }

// Instantiate and call the module — fully type-safe, no raw pointers
let mut module = Module::<MyGlobals, 256, 4>::new(16, MyGlobals::default(), table);
let mut host = MyHost { /* ... */ };
let result = module.process_data(&mut host, ptr, len)?;
```

This is the recommended approach: zero `unsafe`, full capability enforcement at compile time, and zero-cost dispatch via monomorphization.

### 10.2 C-Compatible ABI (Optional)

For integration with non-Rust systems (C/C++ hosts, FFI boundaries), an optional `extern "C"` wrapper can be generated. This wrapper erases generics and uses opaque types:

```rust
#[no_mangle]
pub extern "C" fn module_new(initial_pages: u32) -> *mut OpaqueModule {
    // Allocates and initializes the module
}

#[no_mangle]
pub extern "C" fn module_call(
    instance: *mut OpaqueModule,
    function_index: u32,
    args: *const i64,   // Wasm values are i32/i64/f32/f64
    args_len: usize,
    result: *mut i64,
) -> i32 {
    // 0 = success, non-zero = WasmTrap discriminant
}
```

Note: the C ABI wrapper necessarily uses `unsafe` and raw pointers. Capability enforcement still applies inside — the wrapper calls through the same trait-bounded functions. The C API is an escape hatch, not the default.

### 10.3 Interoperability with Native Rust

Native Rust code integrates by implementing the import traits directly (see §5.4 "The Host as Kernel"). Custom platform-specific capabilities beyond WASI are just additional traits:

```rust
// Custom embedded capability
trait GpioOps {
    fn gpio_set(&mut self, pin: u32, value: bool) -> WasmResult<()>;
    fn gpio_read(&self, pin: u32) -> WasmResult<bool>;
}

// The host implements it; modules that import GPIO functions require it
struct EmbeddedHost { /* ... */ }
impl GpioOps for EmbeddedHost { /* ... */ }
```

---

## 11. Tooling Requirements

### 11.1 Transpiler Tool

**Name**: `herkos` (tentative)

**Architecture**: See [TRANSPILER_DESIGN.md](TRANSPILER_DESIGN.md) for complete design

**High-Level Pipeline**:
```
.wasm → Parser (wasmparser) → IR Builder → IR Optimizer → Backend Selection → Rust Codegen → rustfmt
                                                                   ↑
                                                          Verification Metadata
```

**Key Components**:
- **Parser**: Extracts Wasm module structure using `wasmparser` crate
- **IR (Intermediate Representation)**: SSA-form IR with structured control flow, separates Wasm semantics from Rust codegen
- **Backend Trait**: Abstracts safe/verified/hybrid differences
  - `SafeBackend`: Emits bounds-checked operations, no proofs needed
  - `VerifiedBackend`: Emits `unsafe` with proof references, fails if proof missing
  - `HybridBackend`: Mix of proven (unsafe) and unproven (checked) operations
- **Codegen**: Walks IR, emits Rust source using selected backend

**Features**:
- Parse WebAssembly binary format (`.wasm`)
- Generate auditable, formatted Rust code
- Support three backends (safe, verified, hybrid)
- Integrate with verification metadata (TOML)
- Emit `Module` structs with isolated memory

**CLI Interface**:
```bash
herkos input.wasm \
  --mode verified \
  --output output.rs \
  --metadata verification.toml \
  --max-pages 256        # override when Wasm module declares no maximum
# Note: capabilities are inferred from Wasm imports — no --capabilities flag needed.
# The transpiler generates trait bounds from the module's import section (see §5).
```

### 11.2 Verification Tool

**Name**: `wasm-verify` (tentative)

**Features**:
- Static analysis of Wasm modules
- Bounds checking proof generation
- Capability requirement extraction
- Integration with SMT solvers for complex proofs

**Analysis architecture**: two-phase approach:
1. **Abstract interpretation** (fast, first pass) — tracks value ranges and memory access patterns through the control flow graph. Resolves the majority of proof obligations (constant offsets, loop induction variables, stack frame accesses) without invoking an external solver.
2. **SMT solver** (precise, second pass) — unresolved obligations from phase 1 are encoded as bitvector constraints and dispatched to an SMT solver. The solver either produces a proof (the access is safe for all inputs) or a counterexample (a concrete input that violates the bound).

**Candidate SMT solvers**:

| Solver | License | Rust integration | Strengths |
|--------|---------|------------------|-----------|
| **Z3** (Microsoft Research) | MIT | `z3` crate (high-level bindings) | Industry standard; excellent bitvector and array theory support — directly models Wasm i32/i64 arithmetic and linear memory |
| **Bitwuzla** (Stanford/TU Wien) | MIT | `bitwuzla-sys` (FFI bindings) | Purpose-built for bitvector/array reasoning; often faster than Z3 for bounded memory proofs |
| **CVC5** (Stanford/Iowa) | BSD | `cvc5` crate | Strong quantifier support; useful for inter-procedural proofs involving universally quantified loop invariants |

Z3 is the recommended starting point: it has the most mature Rust bindings, the largest community, and handles the bitvector arithmetic that dominates Wasm proof obligations. Bitwuzla is a compelling alternative when Z3 is too slow on specific memory-heavy obligations.

**Implementation complexity by proof category**:

| Proof category | Difficulty | Approach | Notes |
|----------------|-----------|----------|-------|
| Constant-offset bounds | Trivial | Abstract interpretation only | `i32.load offset=8` with known base → no solver needed |
| Loop-induction bounds | Easy | Abstract interpretation + simple SMT | `arr[i]` where `i < len` — classic induction variable widening |
| Stack frame isolation | Medium | Pattern matching + abstract interpretation | Recognize Clang's `__stack_pointer` idiom, track frame size |
| Arithmetic overflow | Medium | SMT bitvector theory | Encode Wasm's wrapping semantics, prove no unintended overflow |
| Inter-procedural aliasing | Hard | SMT + manual annotations | Requires call-graph analysis; may need user-provided invariants |
| Indirect call targets (`call_indirect`) | Hard | Type-based analysis + SMT | Table contents are statically known in most Wasm modules, but dynamic dispatch complicates proofs |

### 11.3 Runtime Library

**Name**: `herkos-runtime` (tentative)

**Provides**:
- `IsolatedMemory` types
- Import/export trait definitions (WASI traits, capability traits)
- Standard Wasm runtime functions (memory.grow, memory.size, etc.)
- `WasmTrap` / `WasmResult<T>` error types
- Host call interface

**Runtime verification with Kani**:

The runtime crate itself should be verified using Kani proof harnesses. These prove that the building blocks used by all generated code are correct:

```rust
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn load_i32_never_panics() {
        let mem = IsolatedMemory::<4>::new(1); // 1 page = 64 KiB
        let offset: usize = kani::any();
        // Should return Err(OutOfBounds), never panic
        let _ = mem.load_i32(offset);
    }

    #[kani::proof]
    fn grow_respects_max_pages() {
        let mut mem = IsolatedMemory::<4>::new(1);
        let delta: u32 = kani::any();
        let result = mem.grow(delta);
        if result >= 0 {
            assert!(mem.page_count() <= 4);
        }
    }

    #[kani::proof]
    fn store_load_roundtrip() {
        let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let value: i32 = kani::any();
        if mem.store_i32(offset, value).is_ok() {
            assert_eq!(mem.load_i32(offset).unwrap(), value);
        }
    }
}
```

These harnesses run via `cargo kani` and exhaustively verify the runtime's core invariants — no panics, no silent corruption, correct grow semantics, load/store roundtrip. This provides a verified foundation that all three backends build on.

---

## 12. Security Properties

### 12.1 Threat Model

**Protected Against**:
- Memory corruption (buffer overflows, use-after-free)
- Unauthorized resource access (files, network, system calls)
- Cross-module interference (freedom from interference per ISO 26262 Part 6)
- Return-oriented programming (ROP) attacks

**Not Protected Against** (current scope):
- Logic bugs in the original C/C++ code
- Side-channel attacks (timing, cache)
- Resource exhaustion (infinite loops, memory leaks within bounds) — see §14.3 for a future extension addressing this
- Timing interference (this addresses spatial isolation, not temporal) — see §14.3 for a fuel-based approach

### 12.2 Freedom from Interference

The core safety property is **freedom from interference** as defined in functional safety standards: a module at a lower criticality level (e.g., ASIL-B) cannot corrupt the state of a module at a higher criticality level (e.g., ASIL-D). This pipeline enforces this through:

1. **Spatial isolation**: Each transpiled module operates on its own `IsolatedMemory` instance. The Rust type system prevents any cross-module memory access — there is no pointer, offset, or API that allows one module to reach another module's linear memory.
2. **Capability enforcement**: A module can only perform operations (file I/O, network, IPC) that it was explicitly granted at instantiation. Capabilities are encoded in the type system — missing capabilities cause compile errors, not runtime failures.
3. **Type safety**: All operations in the generated Rust code are type-safe. In the verified backend, `unsafe` blocks are justified by machine-checkable proofs.
4. **Compile-time verification**: In verified and hybrid modes, isolation properties are checked before the code ever runs. The safety argument does not depend on correct hardware configuration or OS behavior.

### 12.3 Relationship to Safety Standards

This pipeline produces **evidence** that can support a freedom-from-interference argument in a safety case:
- The transpiled Rust source is auditable (especially the safe backend output)
- Verification metadata provides a machine-checkable proof trail for the verified backend
- The isolation boundary is the Rust type system, which is well-understood and does not depend on runtime configuration

Note: this tool does not replace a formal safety case. It provides a compile-time isolation mechanism and associated evidence that can be used as part of one.

---

## 13. Performance Considerations

### 13.1 Expected Overhead by Backend

| Backend  | Overhead | Source of overhead |
|----------|----------|--------------------|
| Verified | 0–5%     | Function call indirection only; all checks eliminated by proofs |
| Hybrid   | 5–15%    | Runtime bounds checks on unproven accesses |
| Safe     | 15–30%   | Comprehensive runtime checking on every memory access and arithmetic op |

### 13.2 Optimization Strategies

- Aggressive inlining of memory access wrappers (critical for safe backend to reduce check cost)
- Const propagation for bounds checking
- LLVM LTO to eliminate abstraction layers between the transpiled code and `herkos-runtime`
- Profile-guided optimization for hot paths
- **Proof-guided optimization**: in hybrid mode, focus `wasm-verify` effort on hot paths first to maximize the performance benefit of moving accesses from checked to unchecked

### 13.3 Monomorphization Bloat and Instruction Cache

This design uses generics extensively: `IsolatedMemory<const MAX_PAGES: usize>`, trait bounds like `H: SocketOps + FileOps`, and `Module<const MAX_PAGES: usize>`. Rust monomorphizes all of these — each unique combination of type parameters generates a separate copy of the machine code. On embedded targets with limited flash and small instruction caches, this is a serious concern:

- **Binary size**: 5 modules with different `MAX_PAGES` values × 20 functions each = 100 monomorphized function copies instead of 20
- **Instruction cache**: cold icache from code duplication can negate the performance gains from eliminating bounds checks. A 0% overhead verified function that causes an icache miss on every call is slower than a 5% overhead checked function that stays hot in cache.

**Mitigation strategies** (the transpiler should apply these):

#### 1. Outline pattern: generic shell, non-generic core

The critical mitigation. Move the actual logic into a non-generic inner function that takes sizes as runtime parameters. The generic wrapper is a thin shell that passes const generics as runtime values:

```rust
// Non-generic inner function — ONE copy in the binary.
// No unwrap(), no panicking indexing — all error paths return Result.
#[inline(never)]
fn load_i32_inner(memory: &[u8], active_bytes: usize, offset: usize) -> WasmResult<i32> {
    let end = offset.checked_add(4).ok_or(WasmTrap::OutOfBounds)?;
    if end > active_bytes {
        return Err(WasmTrap::OutOfBounds);
    }
    let s = memory.get(offset..end).ok_or(WasmTrap::OutOfBounds)?;
    let arr: [u8; 4] = s.try_into().map_err(|_| WasmTrap::OutOfBounds)?;
    Ok(i32::from_le_bytes(arr))
}

// Generic wrapper — compiles to a single call instruction per MAX_PAGES
impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
    #[inline(always)]
    fn load_i32(&self, offset: usize) -> WasmResult<i32> {
        // Delegates to non-generic inner with runtime size
        load_i32_inner(self.pages.as_flattened(), self.active_pages * PAGE_SIZE, offset)
    }
}
```

This gives the type safety of const generics at the API boundary while keeping a single copy of the actual logic in the binary.

#### 2. Trait object fallback for cold paths

For host trait calls on cold paths (error handling, initialization), consider trait objects (`&mut dyn SocketOps`) instead of monomorphized generics. This trades a vtable indirection for a single code copy:

```rust
// Hot path: monomorphized, zero-cost dispatch
fn inner_loop<H: SocketOps>(host: &mut H, ...) { /* called millions of times */ }

// Cold path: dynamic dispatch, one copy in binary
fn error_report(host: &mut dyn SocketOps, ...) { /* called rarely */ }
```

#### 3. Unify `MAX_PAGES` where possible

The transpiler should normalize `MAX_PAGES` to a small set of standard sizes (e.g., 16, 64, 256, 1024) rather than emitting the exact declared maximum. Two modules with `MAX_PAGES=253` and `MAX_PAGES=260` should both use `MAX_PAGES=256` to share monomorphized code.

#### 4. `#[inline(never)]` on generated function bodies

Transpiled Wasm functions are typically large. The transpiler should mark them `#[inline(never)]` to prevent LLVM from inlining them into callers, which would duplicate the entire function body at each call site.

#### 5. LTO for dead code elimination

Link-time optimization can eliminate monomorphized copies that are unreachable after inlining. This is especially effective when combined with the outline pattern — the generic wrappers inline and disappear, leaving only the shared inner functions.

**Guidance**: the outline pattern (strategy 1) is mandatory for the runtime crate. Strategies 2–5 are recommended for the transpiler. Binary size should be a tracked metric in CI alongside correctness and performance.

```{spec} Outline Pattern: Generic Shell, Non-Generic Core
:id: SPEC_OUTLINE_PATTERN
:status: open
:tags: performance, monomorphization, binary-size, optimization
Move logic into non-generic inner functions (marked #[inline(never)]) taking sizes
as runtime parameters. Generic wrapper (inline(always)) is thin shell that passes const
generics as runtime values. Results in single binary copy of logic, not per-MAX_PAGES.
Critical mitigation for monomorphization bloat.
```

```{spec} MAX_PAGES Normalization to Standard Sizes
:id: SPEC_MAX_PAGES_NORMALIZATION
:status: open
:tags: performance, monomorphization, max-pages
Transpiler should normalize MAX_PAGES to standard sizes (16, 64, 256, 1024) rather
than exact declared maximum. Two modules with MAX_PAGES=253 and MAX_PAGES=260 both
use MAX_PAGES=256 and share monomorphized code.
```

```{spec} Monomorphization Bloat Awareness
:id: SPEC_MONOMORPHIZATION_BLOAT
:status: open
:tags: performance, monomorphization, optimization
Acknowledge that each distinct <MAX_PAGES, trait-bound> combination generates separate
machine code. This can cause binary bloat and instruction cache pressure. Apply
mitigation strategies: outline pattern (mandatory), trait objects on cold paths,
normalization, LTO.
```

```{spec} Transpiler Parallelization for Large Modules
:id: SPEC_TRANSPILER_PARALLELIZATION
:status: open
:tags: performance, parallelization, compilation
IR building and code generation are embarrassingly parallel—each function is independent
with no shared mutable state. Use rayon for parallel iteration when module has 20+
functions. Enables linear speedup on multi-core systems.
```

### 13.4 Comparison to Alternatives

| Approach | Runtime Overhead | Isolation Strength | `unsafe` in output |
|----------|-----------------|-------------------|--------------------|
| MMU/MPU | 10–50% (context switches) | Strong (hardware) | N/A |
| This approach (verified) | 0–5% | Strong (formal proofs) | Yes — proof-justified |
| This approach (safe) | 15–30% | Strong (runtime checks) | None |
| WebAssembly runtime | 20–100% | Strong (runtime sandbox) | N/A |
| Software fault isolation | 10–30% | Medium (runtime) | N/A |

### 13.5 Transpiler Parallelization

For large WebAssembly modules with hundreds or thousands of functions, the transpilation pipeline can become a bottleneck. Since function translation is an embarrassingly parallel problem (each function is independent), the transpiler should leverage multi-core processors to reduce transpilation time.

#### 13.5.1 Parallelization Opportunities

The transpilation pipeline has three main stages:

1. **Parsing** (~10–20% of time) — Sequential, hard to parallelize
   - Uses `wasmparser` crate which processes sections sequentially
   - WASM sections have dependencies (e.g., FunctionSection references TypeSection)
   - **Not parallelizable**

2. **IR Building** (~40% of time) — **Highly parallelizable**
   - Each function translation is independent
   - No shared mutable state between function translations
   - Linear speedup potential: 8x faster on 8 cores

3. **Code Generation** (~40% of time) — **Highly parallelizable**
   - Each function's code generation is independent
   - Backend implementations (`SafeBackend`, `VerifiedBackend`) are stateless
   - Order-preserving collection maintains function numbering

#### 13.5.2 Implementation Strategy

Use the `rayon` crate for data parallelism via parallel iterators:

```rust
use rayon::prelude::*;

// Sequential (current):
let ir_functions: Vec<_> = parsed.functions.iter().enumerate()
    .map(|(idx, func)| {
        let mut builder = IrBuilder::new();
        builder.translate_function(...)
    })
    .collect::<Result<Vec<_>>>()?;

// Parallel (proposed):
let ir_functions: Vec<_> = parsed.functions.par_iter().enumerate()
    .map(|(idx, func)| {
        let mut builder = IrBuilder::new();  // Each thread gets its own builder
        builder.translate_function(...)
    })
    .collect::<Result<Vec<_>>>()?;
```

**Key properties:**
- Each thread creates its own `IrBuilder` instance (no shared mutable state)
- Order is preserved by `par_iter().enumerate()` (maintains function numbering)
- Error handling propagates through `collect::<Result<Vec<_>>>()`
- Same applies to code generation loop

#### 13.5.3 Activation Strategy

Three approaches, in order of preference:

**Option 1: Heuristic-based (Recommended)**
```rust
if parsed.functions.len() >= 20 {
    // Use parallel iterators
} else {
    // Use sequential iterators
}
```
- **Pros**: Best of both worlds — no overhead for small modules, automatic speedup for large modules
- **Cons**: Magic number threshold needs tuning
- **Recommendation**: Start with threshold=20, adjust based on benchmarks

**Option 2: Cargo feature flag**
```toml
[features]
parallel = ["rayon"]

[dependencies]
rayon = { version = "1.10", optional = true }
```
- **Pros**: No forced dependency, explicit opt-in
- **Cons**: Users must know to enable it, loses benefit of zero-config speedup
- **Use case**: Suitable for embedded/deterministic builds that want sequential execution

**Option 3: Always-on parallelization**
- **Pros**: Simplest implementation, always fast for large modules
- **Cons**: Adds ~500KB dependency, small overhead (<1ms) for trivial modules
- **Use case**: If `herkos` is primarily used for large production modules

#### 13.5.4 Performance Expectations

Assuming 8-core CPU and large module (1000+ functions):

| Stage | Sequential Time | Parallel Time | Speedup |
|-------|----------------|---------------|---------|
| Parsing | 2s | 2s | 1.0x (sequential) |
| IR Building | 8s | 1.2s | 6.7x |
| Code Generation | 8s | 1.2s | 6.7x |
| **Total** | **18s** | **4.4s** | **4.1x overall** |

**Scaling efficiency**: ~85% (typical for embarrassingly parallel workloads with rayon)

#### 13.5.5 Implementation Considerations

**Thread safety requirements:**
- `IrBuilder` must not share mutable state across function translations
- Currently satisfied: each `IrBuilder::new()` is independent
- `Backend` implementations must be stateless or `Sync`
- Currently satisfied: `SafeBackend` has no fields, methods take `&self`

**Determinism:**
- Output must be identical regardless of thread count or scheduling
- Achieved by using `par_iter().enumerate()` which preserves order
- Function names are derived from index: `func_0`, `func_1`, etc.
- No non-deterministic collection types (HashMap iteration order doesn't affect output)

**Error handling:**
- First error short-circuits parallel iteration (rayon behavior)
- Error messages include function index for debugging
- Stack traces may reference worker thread IDs (cosmetic only)

**CI/Testing:**
- Run tests both sequentially and in parallel to verify determinism
- Benchmark suite should measure both single-threaded and multi-threaded performance
- Guard against regressions that break parallelizability (e.g., introducing shared mutable state)

#### 13.5.6 Future Optimization Opportunities

Once basic parallelization is implemented:

1. **Pipeline parallelism**: Parse next module while transpiling current one
2. **Work stealing within functions**: For very large functions (>10KB), parallelize basic block code generation
3. **Incremental transpilation**: Cache IR for unchanged functions (future optimization)
4. **GPU acceleration**: Use GPU shaders for proof checking in `wasm-verify` (speculative)

**Recommendation**: Implement Option 1 (heuristic-based) during Phase 9 (Hardening and Polish) of the development plan. Track transpilation time as a CI metric alongside correctness and output binary size.

---

## 14. Future Extensions

### 14.1 Planned Features

- Support for multi-threaded Wasm modules
- Automated refactoring suggestions for better Rust idioms
- DWARF debug info preservation for source-level debugging
- Proof coverage reports: show per-function and per-module percentage of accesses that are formally proven vs. runtime-checked

### 14.2 Research Directions

- Integration with proof assistants (Coq, Lean) for proofs beyond SMT solver reach
- Machine learning-assisted bounds proof generation
- Hardware-assisted verification (ARM PAC, CHERI capabilities)
- Integration with fuzzing tools for proof validation and counter-example discovery
- Gradual proof refinement: automatically identify high-impact unproven accesses (hot paths) and prioritize proof generation

### 14.3 Temporal Isolation (Fuel-Based Execution)

Spatial isolation (§12.2) prevents a module from corrupting another module's memory. **Temporal isolation** prevents a module from starving another of CPU time — equally critical for safety standards (ISO 26262 requires both).

Since we control code generation, we can instrument the transpiled code at every back edge (loop header) and function call. This is fundamentally cheaper than OS-level preemption because there are no context switches — the "yield" is a normal function return.

#### The fuel model

Each module call receives a **fuel budget** (instruction count). The transpiler inserts fuel decrements at loop headers and function calls. When fuel reaches zero, the function returns a `WasmTrap::FuelExhausted` — cooperative, deterministic, no preemption needed.

```rust
/// Fuel counter — passed through every function, decremented at loop headers.
/// When it reaches zero, the function returns WasmTrap::FuelExhausted.
struct Fuel {
    remaining: u64,
}

impl Fuel {
    #[inline(always)]
    fn consume(&mut self, cost: u64) -> WasmResult<()> {
        if self.remaining < cost {
            return Err(WasmTrap::FuelExhausted);
        }
        self.remaining -= cost;
        Ok(())
    }
}
```

#### Transpiled code with fuel instrumentation

```rust
// Safe backend: fuel check at every loop header
fn sum_array(
    memory: &IsolatedMemory<256>,
    globals: &mut Globals,
    fuel: &mut Fuel,
    arr: u32,
    len: u32,
) -> WasmResult<i32> {
    let mut sum: i32 = 0;
    let mut i: u32 = 0;

    'outer: loop {
        fuel.consume(1)?;           // ← inserted at loop back edge
        if i >= len { break 'outer; }

        let value = memory.load::<i32>((arr + i * 4) as usize)?;
        sum = sum.wrapping_add(value);
        i += 1;
    }
    Ok(sum)
}
```

#### Three levels of temporal guarantee (mirroring the three backends)

| Level | Mechanism | Overhead | What it guarantees |
|-------|-----------|----------|-------------------|
| **Fuel-checked** (safe) | Runtime fuel decrements at loop headers and calls | ~3–5% | Guaranteed termination within budget; no infinite loops |
| **WCET-proven** (verified) | `wasm-verify` statically bounds loop iterations and call depth | 0% | Proven worst-case execution time; fuel checks eliminated |
| **Hybrid** | Proven loops skip fuel checks; unproven loops keep them | 1–3% | Static bounds where provable, runtime fuel elsewhere |

#### WCET analysis via `wasm-verify`

Many loops in C-compiled Wasm have statically bounded iteration counts:

```rust
// wasm-verify can prove: this loop executes exactly `len` times.
// If len ≤ MAX_LEN (from function precondition or call-site analysis),
// then WCET = len * (cost_per_iteration) ≤ MAX_LEN * cost_per_iteration.

// With the proof, the verified backend emits NO fuel check:
fn sum_array_verified(
    memory: &IsolatedMemory<256>,
    globals: &mut Globals,
    // no fuel parameter — WCET is statically proven
    arr: u32,
    len: u32,
) -> i32 {
    // WCET_PROOF: loop_0x0010 — bounded by `len`, len ≤ 16384 (from call-site)
    // WCET: 16384 * 5 = 81920 instructions max
    let mut sum: i32 = 0;
    let mut i: u32 = 0;
    'outer: loop {
        if i >= len { break 'outer; }
        let value = unsafe { memory.load_unchecked::<i32>((arr + i * 4) as usize) };
        sum = sum.wrapping_add(value);
        i += 1;
    }
    sum
}
```

What `wasm-verify` analyzes for WCET:
- **Loop bounds**: identify induction variables, prove iteration counts are bounded
- **Recursion depth**: prove call depth is bounded (or reject unbounded recursion)
- **Call graph**: sum per-function WCET to get module-level WCET
- **Termination**: prove all control flow paths terminate (no infinite loops)

Loops that can't be statically bounded (e.g., `while (condition_from_input)`) keep the fuel check — same hybrid strategy as memory bounds.

#### Why this fits the architecture

The temporal isolation model is structurally identical to the spatial isolation model:

| Dimension | Spatial (memory) | Temporal (CPU time) |
|-----------|-----------------|-------------------|
| What's protected | Memory regions | CPU cycles |
| Safe backend | Bounds check at every access | Fuel check at every back edge |
| Verified backend | Proven in-bounds → unchecked | Proven bounded → no fuel check |
| Hybrid | Mix of checked and proven | Mix of fuel-checked and proven |
| Verification tool | `wasm-verify` bounds proofs | `wasm-verify` WCET proofs |
| Metadata format | `[[proofs]]` with `kind = "bounds"` | `[[proofs]]` with `kind = "wcet"` |
| Trap on violation | `WasmTrap::OutOfBounds` | `WasmTrap::FuelExhausted` |

This means the same `wasm-verify` infrastructure, the same backend selection model, and the same incremental proof improvement strategy apply to both dimensions of freedom from interference.

```{spec} Fuel-Based Temporal Isolation
:id: SPEC_TEMPORAL_ISOLATION_FUEL
:status: open
:tags: temporal-isolation, fuel, future, wcet
Future extension: fuel-based execution prevents modules from starving each other of CPU
time. Each module call receives an instruction budget. Transpiler inserts fuel decrements
at loop headers and function calls. Returns WasmTrap::FuelExhausted when budget exhausted.
Cooperative, deterministic, no OS preemption needed.
```

```{spec} Three Fuel Guarantee Levels
:id: SPEC_FUEL_THREE_LEVELS
:status: open
:tags: temporal-isolation, fuel, backends, wcet
Three fuel levels mirror spatial isolation backends: Fuel-checked (safe, 3–5% overhead),
WCET-proven (verified, 0% overhead), Hybrid (mix, 1–3% overhead). Enables gradual proof
coverage improvement for execution time, same as memory bounds.
```

```{spec} WCET Analysis via wasm-verify
:id: SPEC_WCET_ANALYSIS
:status: open
:tags: temporal-isolation, wcet, verification
wasm-verify performs worst-case execution time (WCET) analysis: proves loop bounds,
recursion depth, and termination. Enables verified backend to eliminate fuel checks on
proven paths. Same two-phase approach as bounds checking (abstract interpretation + SMT).
```

---

## 15. Example Use Cases

### 15.1 Safety-Critical Embedded Systems

Replace MPU/MMU-based isolation in systems where freedom from interference is required by standard:
- **Automotive (ISO 26262)**: mixed-criticality ECUs where ASIL-D and ASIL-B software share a processor — compile-time isolation eliminates the need for MPU partitioning and its associated performance/energy cost
- **Industrial (IEC 61508)**: PLCs and safety controllers running third-party function blocks alongside safety-certified logic
- **Avionics (DO-178C)**: partitioned architectures (ARINC 653) where compile-time isolation could reduce dependence on hypervisor-enforced spatial separation
- **Medical (IEC 62304)**: devices with mixed-criticality software where isolation evidence simplifies the safety argument

### 15.2 Security Containment

Isolate untrusted code without hardware overhead:
- Sandboxing third-party libraries (image decoders, parsers, codecs) so a vulnerability in the library cannot escalate
- Plugin systems where plugins are transpiled and run in isolated memory — no syscall access unless explicitly granted
- Supply chain risk mitigation: contain dependencies at compile time rather than trusting them

### 15.3 Legacy Code Migration

- Gradual migration of C codebases toward provable isolation without full rewrites
- Start with safe backend (no proofs needed), progressively move hot paths to verified
- Maintain interoperability with existing C/C++ codebases during transition

---

## 16. Open Questions

1. How to handle C++ exceptions in WebAssembly?
2. What is the best approach for generating bounds checking proofs automatically?
3. How to represent and verify concurrent access patterns?
4. Should we support dynamic linking of transpiled modules?
5. What level of C/C++ standard library should be supported?

---

## 17. References

- WebAssembly Specification: https://webassembly.github.io/spec/
- Rust Reference: https://doc.rust-lang.org/reference/
- Software Fault Isolation: Wahbe et al., 1993
- Proof-Carrying Code: Necula & Lee, 1996

---

## Appendix A: Example Transpilation

### Input C Code
```c
int sum_array(int* arr, size_t len) {
    int sum = 0;
    for (size_t i = 0; i < len; i++) {
        sum += arr[i];
    }
    return sum;
}
```

### WebAssembly (WAT format)
```wat
(func $sum_array (param $arr i32) (param $len i32) (result i32)
  (local $sum i32)
  (local $i i32)
  (local.set $sum (i32.const 0))
  (local.set $i (i32.const 0))
  (block $break
    (loop $continue
      (br_if $break (i32.ge_u (local.get $i) (local.get $len)))
      (local.set $sum
        (i32.add
          (local.get $sum)
          (i32.load (i32.add (local.get $arr) (i32.shl (local.get $i) (i32.const 2))))))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $continue)))
  (local.get $sum))
```

### Transpiled Rust Code — Safe Backend
```rust
pub fn sum_array<'mem, const MAX_PAGES: usize>(
    memory: &'mem IsolatedMemory<MAX_PAGES>,
    globals: &mut Globals,
    arr: u32,
    len: u32,
) -> WasmResult<i32> {
    let mut sum: i32 = 0;
    let mut i: u32 = 0;

    'outer: loop {
        if i >= len {
            break 'outer;
        }

        let offset = arr.checked_add(i.checked_mul(4).ok_or(WasmTrap::IntegerOverflow)?)
            .ok_or(WasmTrap::IntegerOverflow)?;

        let value = memory.load::<i32>(offset as usize)?;
        sum = sum.wrapping_add(value);

        i = i.checked_add(1).ok_or(WasmTrap::IntegerOverflow)?;
    }

    Ok(sum)
}
```

### Transpiled Rust Code — Verified Backend
```rust
/// All proofs reference verification metadata generated by `wasm-verify`.
pub fn sum_array<'mem, const MAX_PAGES: usize>(
    memory: &'mem IsolatedMemory<MAX_PAGES>,
    globals: &mut Globals,
    arr: u32,
    len: u32,
) -> i32 {
    let mut sum: i32 = 0;
    let mut i: u32 = 0;

    'outer: loop {
        if i >= len {
            break 'outer;
        }

        // PROOF: arith_0x0010 — i * 4 cannot overflow when i < len and len ≤ MAX_PAGES * PAGE_SIZE / 4
        let offset = arr + i * 4;

        // PROOF: bounds_0x0012 — offset ∈ [arr, arr + len*4) ⊆ [0, MAX_PAGES * PAGE_SIZE - 4]
        let value = unsafe { memory.load_unchecked::<i32>(offset as usize) };
        sum = sum.wrapping_add(value);

        i += 1;
    }

    sum
}
```

Note: in the verified backend the function returns `i32` directly instead of `WasmResult<i32>` — the proofs guarantee no trap can occur, so the `Result` wrapper is eliminated entirely. Wasm integer arithmetic semantics are wrapping, so `wrapping_add` is used in both backends. The `globals` parameter is present in both signatures per §8.1, even though this particular function doesn't access globals — it's part of the standard transpiled function signature.

---

**Document Status**: Draft for Review
**Version**: 0.1
**Date**: 2026-02-10
**Authors**: [To be filled]
**Reviewers**: [To be filled]