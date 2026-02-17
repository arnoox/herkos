Compile-Time Memory Isolation via WebAssembly and Rust Transpilation
====================================================================

A compilation pipeline that transpiles WebAssembly modules into memory-safe Rust code with compile-time isolation guarantees, replacing runtime hardware-based memory protection (MMU/MPU) with type-system-enforced safety.

**WebAssembly → Rust source → Safe binary**

Motivation
----------

Safety-critical standards (ISO 26262, IEC 61508, DO-178C) require **freedom from interference** between software modules of different criticality levels. Today this is achieved via MMU/MPU or hypervisors, approaches that are expensive in performance, energy, and certification effort.

herkos takes a different approach: if the Rust compiler accepts the transpiled code, isolation is guaranteed: no MMU, no context switches, no runtime overhead for proven accesses.


.. toctree::
   :maxdepth: 2
   :caption: Contents:

   GETTING_STARTED.md
   REQUIREMENTS.md
   WebAssemblyReferenceManual.md
   SPECIFICATION.md
   TRANSPILER_DESIGN.md
