herkos Documentation
====================

herkos is a compilation pipeline that transforms WebAssembly modules into memory-safe Rust code with compile-time isolation guarantees, replacing runtime hardware-based memory protection (MMU/MPU) with type-system-enforced safety.

.. mermaid::

   flowchart TD
       A["C/C++ Source"] --> B["WebAssembly (Wasm)"]
       B --> C["Rust Transpiler + Runtime"]
       C --> D["Safe Rust Binary"]

Systems that mix components of different trust or criticality levels require **freedom from interference**: the guarantee that one module cannot corrupt the state of another. Hardware isolation mechanisms (MMU, MPU, hypervisors) provide this today, but at a cost in performance, energy, and certification complexity.

herkos takes a different approach: **move the isolation guarantee from runtime hardware to compile-time type system enforcement**. If the Rust compiler accepts the transpiled code, isolation is guaranteed — no MMU, no context switches, no runtime overhead for proven accesses.

Goals
-----

- Achieve memory safety and inter-module isolation guarantees at compile time rather than runtime
- Provide performance competitive with or better than hardware-based isolation
- Provide a migration path for existing C/C++ codebases toward provable isolation without full rewrites
- Enable capability-based security enforced through the type system (freedom from interference by construction)
- Support incremental adoption: start with the safe backend (runtime checks, no proofs needed), progressively move to verified as proof coverage improves

Sections
--------

.. toctree::
   :maxdepth: 2

   GETTING_STARTED
   REQUIREMENTS
   SPECIFICATION
   FUTURE
   traceability/index
