# Requirements

## 1. Purpose

This document defines **what** the herkos compilation pipeline shall achieve: its goals, constraints, and formal requirements. For **how** these are implemented, see [SPECIFICATION.md](SPECIFICATION.md).

herkos is a compilation pipeline that transforms WebAssembly modules into memory-safe Rust code with compile-time isolation guarantees, replacing runtime hardware-based memory protection (MMU/MPU) with type-system-enforced safety.

```
┌─────────────┐      ┌──────────────┐      ┌─────────────────┐      ┌─────────────┐
│   C/C++     │ ───> │  WebAssembly │ ───> │ Rust Transpiler │ ───> │ Safe Rust   │
│   Source    │      │    (Wasm)    │      │   + Runtime     │      │   Binary    │
└─────────────┘      └──────────────┘      └─────────────────┘      └─────────────┘
```

## 2. Problem Statement

Systems that mix components of different trust or criticality levels require **freedom from interference** — the guarantee that one module cannot corrupt the state of another. In practice this means:

- A higher-criticality component shall not be corruptible by a lower-criticality one
- An untrusted third-party library shall be contained so it cannot reach outside its sandbox
- Modules at different trust levels shall have provable isolation boundaries

This isolation is typically achieved through hardware mechanisms (MMU, MPU) or hypervisors. While effective, these approaches are:

- **Expensive in performance**: context switches, TLB flushes, and memory barrier overhead
- **Expensive in energy**: critical for battery-powered and thermally constrained embedded systems
- **Complex to implement**: MPU region configuration, partition scheduling, and efficient inter-partition communication require significant engineering effort
- **Hard to certify**: the isolation argument depends on correct hardware configuration, OS behavior, and linker scripts — all of which shall be verified together

herkos takes a different approach: **move the isolation guarantee from runtime hardware to compile-time type system enforcement**. If the Rust compiler accepts the transpiled code, isolation is guaranteed — no MMU, no context switches, no runtime overhead for proven accesses.

> **Note on Hardware Isolation:**
>
> herkos does *not* claim that MPU/MMU isolation is obsolete. Hardware isolation remains essential for:
> - **Untrusted kernels and hypervisors** — compile-time safety assumes trusted compilation and runtime
> - **Defense in depth** — multiple isolation layers reduce risk from both compiler bugs and runtime exploits
> - **Legacy systems** — many existing codebases cannot be rewritten in Rust
> - **Dynamic code** — dynamically loaded code cannot benefit from static Rust type safety
> - **Cross-language systems** — mixed C/C++/Rust systems need runtime isolation
>
> herkos is positioned as a **complementary approach**: replace runtime isolation *where* compile-time safety is achievable, and use hardware isolation for the rest.

## 3. Goals

- Achieve memory safety and inter-module isolation guarantees at compile time rather than runtime
- Provide performance competitive with or better than hardware-based isolation
- Provide a migration path for existing C/C++ codebases toward provable isolation without full rewrites
- Enable capability-based security enforced through the type system (freedom from interference by construction)
- Support incremental adoption: start with the safe backend (runtime checks, no proofs needed), progressively move to verified as proof coverage improves

## 4. Functional Requirements

### 4.1 Memory Model

```{req} Wasm Page-Based Memory Model
:id: REQ_MEM_PAGE_MODEL
:status: open
:tags: memory, wasm-spec
Linear memory shall be organized in pages of 64 KiB (per the WebAssembly specification).
Each module declares an initial page count and an optional maximum page count. The
memory.grow instruction adds pages at runtime up to the declared maximum.
```

```{req} Compile-Time Memory Sizing
:id: REQ_MEM_COMPILE_TIME_SIZE
:status: open
:tags: memory, no_std, static-sizing
The maximum memory size for each module shall be fixed at compile time. No heap
allocation is permitted for memory backing storage. All memory shall be statically
sized.
```

```{req} Bounds-Checked Memory Access
:id: REQ_MEM_BOUNDS_CHECKED
:status: open
:tags: memory, safety
All memory accesses in the safe backend shall be bounds-checked against the current
active memory size (active_pages * PAGE_SIZE). Out-of-bounds accesses shall return
an error (WasmTrap::OutOfBounds), never panic or invoke undefined behavior.
```

```{req} Memory Growth Without Allocation
:id: REQ_MEM_GROW_NO_ALLOC
:status: open
:tags: memory, memory.grow, no_std
memory.grow shall not perform heap allocation. New pages shall be zero-initialized
within pre-allocated storage. Returns previous page count on success, -1 on failure.
```

### 4.2 Module Representation

```{req} Two Module Types
:id: REQ_MOD_TWO_TYPES
:status: open
:tags: modules, memory-ownership
The system shall support two module types: (1) modules that own their own memory
(process-like), and (2) modules that borrow memory from a caller (library-like).
This distinction is the primary mechanism for spatial isolation.
```

```{req} Globals as Typed Struct Fields
:id: REQ_MOD_GLOBALS
:status: open
:tags: modules, globals
Mutable Wasm globals shall have statically typed, per-instance storage. Immutable
globals shall be compile-time constants. Global access shall be resolved statically,
with no dynamic lookup.
```

```{req} Indirect Call Table
:id: REQ_MOD_TABLE
:status: open
:tags: modules, table, call_indirect
Each module shall have a table for indirect call dispatch. Table entries store
function references with type index and function index. Indirect calls shall
validate the type signature before dispatch.
```

### 4.3 Imports, Exports, and Capabilities

```{req} Imports as Trait Bounds
:id: REQ_CAP_IMPORTS
:status: open
:tags: imports, traits, capabilities
Wasm module imports shall be statically checked capabilities. Related imports
shall be grouped into discrete capability sets. If a module does not import a
capability, no code path to invoke it shall exist.
```

```{req} Exports as Trait Implementations
:id: REQ_CAP_EXPORTS
:status: open
:tags: exports, traits
Wasm module exports shall be exposed as statically typed interfaces on the
transpiled module. This shall enable inter-module linking via interface composition.
```

```{req} Zero-Cost Dispatch
:id: REQ_CAP_ZERO_COST
:status: open
:tags: traits, dispatch, performance
Capability dispatch shall incur zero runtime overhead compared to direct function
calls. If a module does not import a capability, no code for that capability shall
be generated.
```

```{req} WASI Support via Standard Traits
:id: REQ_CAP_WASI
:status: open
:tags: wasi, imports, traits
WASI (WebAssembly System Interface) support shall be provided as a standard set
of capability interfaces shipped with the runtime. The host provides whichever
subset it supports.
```

### 4.4 Transpilation

```{req} Wasm-to-Rust Function Translation
:id: REQ_TRANS_FUNCTIONS
:status: open
:tags: transpilation, functions
Each Wasm function shall be transpiled to a Rust function with explicit access to
module state (memory, globals, table) and granted capabilities.
```

```{req} Control Flow Mapping
:id: REQ_TRANS_CONTROL_FLOW
:status: open
:tags: transpilation, control-flow
Wasm control flow (block, loop, if, br, br_if, br_table) shall map to safe Rust
control flow structures. No goto or unsafe control flow.
```

```{req} Safe Indirect Call Dispatch
:id: REQ_TRANS_INDIRECT_CALLS
:status: open
:tags: transpilation, indirect-calls, safety
Indirect calls (call_indirect) shall be dispatched using only safe Rust — no
function pointers, no unsafe dispatch. The dispatch mechanism shall validate type
signatures and enumerate only functions matching the expected type.
```

```{req} Structural Type Equivalence
:id: REQ_TRANS_TYPE_EQUIVALENCE
:status: open
:tags: transpilation, types
Type checks in call_indirect shall use structural equivalence: two type indices
match if they have identical parameter and result types, regardless of index.
Type equivalence shall be resolved at transpile time.
```

```{req} Self-Contained Output
:id: REQ_TRANS_SELF_CONTAINED
:status: open
:tags: transpilation, output
Transpiled code shall be self-contained, depending only on herkos-runtime. Output
shall be formatted (rustfmt), readable, and auditable. No panics, no unwinding —
only Result<T, WasmTrap> for error handling.
```

```{req} Deterministic Code Generation
:id: REQ_TRANS_DETERMINISTIC
:status: open
:tags: transpilation, determinism
Generated output shall be identical regardless of CPU, thread count, execution order,
or random seed. Enables reproducible builds and auditable output.
```

### 4.5 Error Handling

```{req} Trap-Based Error Handling
:id: REQ_ERR_TRAPS
:status: open
:tags: error-handling, traps
Wasm traps shall be reported as typed, structured errors. The following trap
categories shall be distinguished: out-of-bounds memory access, division by zero,
integer overflow, unreachable code, indirect call type mismatch, table out-of-bounds,
and undefined table element. No exceptions, no panics, no unwinding.
```

### 4.6 Platform Constraints

```{req} no_std Compatibility
:id: REQ_PLATFORM_NO_STD
:status: open
:tags: no_std, embedded
herkos-runtime and all transpiled output shall be #![no_std]. No heap allocation
without the optional alloc feature gate. No panics, no format!, no String in the
runtime or generated code. Enables resource-constrained and embedded targets.
```

## 5. Non-Functional Requirements

### 5.1 Safety and Isolation

```{req} Compile-Time Isolation Enforcement
:id: REQ_ISOLATION_COMPILE_TIME
:status: open
:tags: isolation, compile-time, safety
Isolation properties shall be verified at compile time via Rust's type system.
The safety argument shall not depend on correct hardware configuration, OS behavior,
or runtime state. If the Rust compiler accepts the transpiled code, isolation is
guaranteed.
```

```{req} Freedom from Interference
:id: REQ_FREEDOM_FROM_INTERFERENCE
:status: open
:tags: isolation, freedom-from-interference
No module shall be able to corrupt the state of another module. This property
— commonly known as "freedom from interference" — shall be enforced via spatial
isolation and capability enforcement. Note: herkos is not qualified to any
safety standard. It provides an isolation mechanism, not a certified safety case.
```

```{req} Spatial Isolation via Memory Ownership
:id: REQ_ISOLATION_SPATIAL
:status: open
:tags: isolation, memory, type-system
Each module shall operate on its own isolated memory. The type system shall
structurally prevent any cross-module memory access — there shall be no pointer,
offset, or API that allows one module to reach another module's linear memory.
```

```{req} Capability Enforcement via Traits
:id: REQ_ISOLATION_CAPABILITY
:status: open
:tags: isolation, capabilities
Capabilities shall be statically enforced at compile time. A module can only perform
operations that it was explicitly granted at instantiation. Missing capabilities
shall cause compile errors, not runtime failures.
```

### 5.2 Determinism

```{req} Deterministic Execution Semantics
:id: REQ_DETERMINISM
:status: open
:tags: determinism, testing, debugging
Transpiled modules shall preserve WebAssembly's deterministic semantics. Each function
shall be pure with respect to its explicit state (parameters, globals, memory). Given
identical inputs, execution shall always produce identical outputs. Host imports are
the sole source of non-determinism and are isolated behind trait bounds.
```

This determinism enables:
- **Debugging**: Capture module state when a bug occurs, replay it locally
- **Testing**: Tests against concrete state snapshots — no flaky tests
- **Fuzzing**: Random inputs with confidence that crashes are reproducible
- **Record and replay**: Log function inputs in production, replay offline
- **Differential testing**: Compare transpiled output against a reference Wasm interpreter

### 5.3 Performance

```{req} Safe Backend Overhead
:id: REQ_PERF_SAFE_OVERHEAD
:status: open
:tags: performance, safe-backend
The safe backend (runtime bounds checking on every memory access) shall achieve
overhead of 15–30% compared to native execution. This is the baseline for all
modules.
```

```{req} Monomorphization Bloat Mitigation
:id: REQ_PERF_MONO_BLOAT
:status: open
:tags: performance, monomorphization, binary-size
The runtime and transpiler shall mitigate binary size explosion from generic code
specialization. Binary size shall be a tracked metric.
```

### 5.4 Security

```{req} Threat Model — Protected Against
:id: REQ_SEC_PROTECTED
:status: open
:tags: security, threat-model
The system shall protect against: memory corruption (buffer overflows, use-after-free),
unauthorized resource access (files, network, system calls), cross-module interference,
and return-oriented programming (ROP) attacks.
```

```{req} Threat Model — Not Protected Against
:id: REQ_SEC_NOT_PROTECTED
:status: open
:tags: security, threat-model
The system does not protect against (current scope): logic bugs in the original source
code, side-channel attacks (timing, cache), resource exhaustion (infinite loops, memory
leaks within bounds), or timing interference. See FUTURE.md for temporal isolation plans.
```

## 6. Non-Goals

- Complete automation of unsafe code to safe Rust transformation (some manual intervention may be required)
- 100% preservation of C/C++ performance characteristics
- Support for all possible C/C++ undefined behaviors
- Replacing formal safety cases — this tool provides evidence for isolation arguments, not a complete safety case
