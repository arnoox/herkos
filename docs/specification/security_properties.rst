.. _security-properties:

Security Properties
===================

Protected Against
-----------------

- **Memory corruption**: buffer overflows, use-after-free — prevented by bounds-checked access and Rust's ownership system
- **Unauthorized resource access**: files, network, system calls — prevented by trait-based capability enforcement
- **Cross-module interference**: freedom from interference — enforced by memory ownership isolation
- **ROP attacks**: no function pointers in generated code — all dispatch is static match

Not Protected Against (current scope)
-------------------------------------

- Logic bugs in the original C/C++ code
- Side-channel attacks (timing, cache)
- Resource exhaustion (infinite loops, memory leaks within bounds) — see :doc:`/FUTURE` §3 for temporal isolation plans
- Timing interference — spatial isolation only, not temporal

Relationship to Safety Standards
--------------------------------

This pipeline produces **evidence** for a freedom-from-interference argument:

- Transpiled Rust source is auditable
- Isolation boundary is the Rust type system — well-understood, no runtime configuration dependency
- **This tool does not replace a formal safety case.** It provides a compile-time isolation mechanism and associated evidence that can be used as part of one.
