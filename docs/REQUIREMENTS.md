# Requirements

## 1.1 Purpose
This specification defines a compilation pipeline that transforms WebAssembly modules (compiled from C/C++ or other languages) into memory-safe Rust code with compile-time isolation guarantees, eliminating the runtime overhead of hardware-based memory protection mechanisms (MMU/MPU).

## 1.2 Problem Statement

Industry standards for functional safety (e.g. ISO 26262 with use case below) and security require **freedom from interference** between software modules of different criticality levels. In practice this means:

- An ASIL-B rated component must not be able to corrupt the memory of an ASIL-D component
- An untrusted third-party library must be contained so it cannot reach outside its sandbox
- Modules at different security or safety levels must have provable isolation boundaries

This isolation is typically achieved through hardware mechanisms (MMU, MPU) or hypervisors. While effective, these approaches are:

- **Expensive in performance**: context switches, TLB flushes, and memory barrier overhead
- **Expensive in energy**: critical for battery-powered and thermally constrained embedded systems
- **Complex to implement**: MPU region configuration, partition scheduling, and efficient inter-partition communication require significant engineering effort
- **Hard to certify**: the isolation argument depends on correct hardware configuration, OS behavior, and linker scripts: all of which must be verified together

This project takes a different approach: **move the isolation guarantee from runtime hardware to compile-time type system enforcement**. If the Rust compiler accepts the transpiled code, isolation is guaranteed: no MMU, no context switches, no runtime overhead for proven accesses.

> **Note on Hardware Isolation:**
>
> herkos does *not* claim that MPU/MMU isolation is obsolete. Hardware isolation remains essential for:
> - **Untrusted kernels and hypervisors** — compile-time safety assumes trusted compilation (memory safe language like Rust) and runtime
> - **Defense in depth** — multiple isolation layers reduce risk from both compiler bugs and runtime exploits
> - **Legacy systems** — many existing codebases cannot be rewritten in Rust
> - **Dynamic code** — dynamically loaded code cannot benefit from static Rust type safety
> - **Cross-language systems** — mixed C/C++/Rust systems need runtime isolation
>
> herkos is positioned as a **complementary approach**: replace runtime isolation *where* compile-time safety is achievable, and use hardware isolation for the rest. The two strategies work well together.

## 1.3 Goals
- Achieve memory safety and inter-module isolation guarantees at compile time rather than runtime
- Provide performance competitive with or better than hardware-based isolation, especially for the verified backend (0–5% overhead)
- Provide a migration path for existing C/C++ codebases toward provable isolation without full rewrites
- Enable capability-based security enforced through the type system (freedom from interference by construction)
- Support incremental adoption: start with the safe backend (runtime checks, no proofs needed), progressively move to verified as proof coverage improves

```{req} Compile-Time Isolation Enforcement
:id: REQ_ISOLATION_COMPILE_TIME
:status: open
:tags: architecture, isolation, compile-time, safety
Isolation properties are verified at compile time via Rust's type system. The safety
argument does not depend on correct hardware configuration, OS behavior, or runtime
state. If the Rust compiler accepts the transpiled code, isolation is guaranteed.
```

```{req} Freedom from Interference Guarantee
:id: REQ_FREEDOM_FROM_INTERFERENCE
:status: open
:tags: architecture, isolation, safety-critical, iso26262
No module at a lower criticality level (e.g., ASIL-B) can corrupt the state of a
module at a higher criticality level (e.g., ASIL-D), per ISO 26262 Part 6 and IEC 61508.
This is enforced via spatial isolation (memory ownership) and capability enforcement
(trait bounds).
```

```{spec} Spatial Isolation via Memory Ownership
:id: REQ_ISOLATION_SPATIAL
:status: open
:tags: architecture, memory, isolation, type-system
Each module operates on its own IsolatedMemory instance. The Rust type system
structurally prevents any cross-module memory access—there is no pointer, offset, or API
that allows one module to reach another module's linear memory.
```

```{req} Capability Enforcement via Traits
:id: REQ_ISOLATION_CAPABILITY
:status: open
:tags: architecture, capabilities, imports, exports
Capabilities (what a module can do) are enforced via Rust trait bounds on the host
parameter. A module can only perform operations (file I/O, network, system calls)
that it was explicitly granted at instantiation. Missing capabilities cause compile errors,
not runtime failures.
```

### Determinism and Reproducibility

WebAssembly has fully deterministic semantics: given the same inputs, a Wasm module always produces the same outputs (okay, probably there are a few exeptions like floating point calculation). herkos preserves this property in the transpiled Rust code. Each transpiled function is **pure** with respect to its explicit state: it takes parameters, globals, and memory as input, and its output is entirely determined by those inputs.

This means that if you can capture the state of a module (memory contents, globals, function arguments) at any point, you can **replay that exact execution** and get identical results. There are no hidden dependencies on thread scheduling, memory layout, allocation order, or system state.

This determinism is a powerful property for:
- **Debugging**: Capture the module state when a bug occurs and replay it locally to reproduce the issue exactly, every time
- **Testing**: Write tests against concrete state snapshots; no flaky tests from non-deterministic behavior
- **Fuzzing**: Feed random inputs into pure functions with full confidence that crashes are reproducible. Corpus minimization works reliably because executions are deterministic
- **Record and replay**: Log function inputs during production and replay them offline for post-mortem analysis
- **Differential testing**: Compare transpiled output against a reference Wasm interpreter to validate correctness — both must produce identical results

Host calls (imports) are the only source of non-determinism. Since these are explicit trait-bounded parameters, they can be easily mocked or recorded, keeping the determinism boundary clear and controllable.

```{req} Deterministic Execution Semantics
:id: REQ_DETERMINISM
:status: open
:tags: architecture, determinism, testing, debugging, fuzzing
Transpiled modules preserve WebAssembly's deterministic semantics. Each function is pure
with respect to its explicit state (parameters, globals, memory). Given identical inputs,
execution always produces identical outputs. Host imports are the sole source of
non-determinism and are isolated behind trait bounds, allowing them to be mocked or
recorded for full reproducibility.
```

## 1.4 Non-Goals
- Complete automation of unsafe code to safe Rust transformation (some manual intervention may be required)
- 100% preservation of C/C++ performance characteristics
- Support for all possible C/C++ undefined behaviors
- Replacing formal safety cases; this tool (not certified for any standard) provides pseudo-evidence for isolation arguments, not a complete safety case
