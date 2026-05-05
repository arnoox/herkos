.. _architecture:

Architecture
============

Component Overview
------------------

.. code-block:: text

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

Runtime (``herkos-runtime``)
----------------------------

*Source:* ``crates/herkos-runtime/src/``

The runtime is a ``#![no_std]`` crate providing the types that all transpiled code depends on. It has **zero external dependencies** in the default configuration.

.. list-table::
   :header-rows: 1

   * - Module
     - Provides
     - Reference
   * - ``memory.rs``
     - ``IsolatedMemory<MAX_PAGES>``, load/store methods, ``memory.grow``/``memory.size``
     - §2.1
   * - ``table.rs``
     - ``Table<MAX_SIZE>``, ``FuncRef``
     - §2.3
   * - ``module.rs``
     - ``Module<G, MAX_PAGES, TABLE_SIZE>``, ``LibraryModule<G, TABLE_SIZE>``
     - §2.2
   * - ``ops.rs``
     - Wasm arithmetic operations (``i32_div_s``, ``i32_trunc_f32_s``, etc.)
     - §4.4
   * - ``lib.rs``
     - ``WasmTrap``, ``WasmResult<T>``, ``ConstructionError``, ``PAGE_SIZE``
     - §4.3

**Constraints** (see :doc:`/REQUIREMENTS`):

- No heap allocation without the optional ``alloc`` feature gate
- No panics, no ``format!``, no ``String``
- Errors are ``Result<T, WasmTrap>`` only
- Optional ``alloc`` feature gate for targets with a global allocator

**Runtime verification with Kani**: The runtime includes ``#[kani::proof]`` harnesses that verify core invariants (no panics on any input, correct grow semantics, load/store roundtrip). Run via ``cargo kani``. See ``crates/herkos-runtime/KANI.md`` in the repository root.

Transpiler (``herkos-core``)
----------------------------

*Source:* ``crates/herkos-core/src/``

``herkos-core`` is the transpiler library. The ``herkos`` crate is a thin CLI wrapper around it. The pipeline:

.. code-block:: text

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

Parser
~~~~~~

*Source:* ``crates/herkos-core/src/parser/``

Uses the ``wasmparser`` crate to extract module structure: types, functions, memories, tables, globals, imports, exports, data segments, element segments.

**Design choice**: ``wasmparser`` only, not ``wasm-tools`` or ``walrus``. Keeps the dependency tree small and avoids pulling in a full Wasm runtime.

IR (Intermediate Representation)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

*Source:* ``crates/herkos-core/src/ir/``

A pure SSA-form IR that sits between Wasm bytecode and Rust source. Every variable is defined exactly once (``DefVar`` token, non-``Copy``) and may be read many times (``UseVar`` token, ``Copy``).

.. code-block:: text

          Wasm bytecode                  SSA IR                      Rust source
   ┌──────────────────────┐   ┌──────────────────────────┐   ┌──────────────────────────┐
   │ i32.const 5          │   │ v0 = Const(I32(5))       │   │ let mut v0: i32 = 0i32;  │
   │ i32.const 3          │   │ v1 = Const(I32(3))       │   │ let mut v1: i32 = 0i32;  │
   │ i32.add              │   │ v2 = BinOp(Add, v0, v1)  │   │ let mut v2: i32 = 0i32;  │
   │                      │   │                          │   │ v0 = 5i32;               │
   │                      │   │                          │   │ v1 = 3i32;               │
   │                      │   │                          │   │ v2 = v0.wrapping_add(v1);│
   └──────────────────────┘   └──────────────────────────┘   └──────────────────────────┘

Key types (defined in ``ir/types.rs``):

- ``ModuleInfo`` — complete module metadata (types, functions, memories, globals, imports, exports)
- ``LoweredModuleInfo`` — newtype wrapper around ``ModuleInfo`` guaranteeing no ``IrInstr::Phi`` nodes remain; only produced by ``lower_phis::lower()``
- ``IrFunction`` — one function's IR: entry block, all basic blocks, locals, return type, type index
- ``IrBlock`` — basic block with a ``Vec<IrInstr>`` body and an ``IrTerminator``
- ``IrInstr`` — a single SSA instruction: ``Const``, ``BinOp``, ``UnOp``, ``Load``, ``Store``, ``Call``, ``CallImport``, ``CallIndirect``, ``Assign``, ``GlobalGet``, ``GlobalSet``, ``MemorySize``, ``MemoryGrow``, ``MemoryCopy``, ``MemoryFill``, ``MemoryInit``, ``DataDrop``, ``Select``, ``Phi``
- ``IrTerminator`` — block exit: ``Return``, ``Jump``, ``BranchIf``, ``BranchTable``, ``Unreachable``
- ``VarId`` — SSA variable identifier (displayed as ``v0``, ``v1``, ...)
- ``BlockId`` — basic block identifier (displayed as ``block_0``, ``block_1``, ...)
- ``DefVar`` / ``UseVar`` — single-use definition / multi-use read tokens enforcing SSA invariants at build time

Index types use a phantom-typed ``Idx<TAG>`` generic to prevent mixing ``LocalFuncIdx``, ``ImportIdx``, ``GlobalIdx``, ``TypeIdx``, etc.

The builder (``ir/builder/``) translates Wasm stack-based instructions to SSA IR by maintaining an explicit value stack of ``UseVar``. Each function is independent — enabling future parallelization (see §6.2).

SSA Phi Lowering
~~~~~~~~~~~~~~~~

*Source:* ``crates/herkos-core/src/ir/lower_phis.rs``

SSA phi nodes are inserted by the builder at join points (if/else merges, loop headers). Before codegen they must be *destroyed* — converted to ordinary assignments in predecessor blocks.

.. code-block:: text

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

The pass:

1. **Prunes stale sources**: removes ``(pred, var)`` entries whose predecessor was eliminated by dead block removal
2. **Simplifies trivial phis**: single-source or all-same-source phis become ``Assign`` in-place
3. **Lowers non-trivial phis**: inserts ``Assign { dest, src }`` at the end of each predecessor block, then removes the ``Phi``

Optimizer
~~~~~~~~~

*Source:* ``crates/herkos-core/src/optimizer/``

The optimizer is split into two phases separated by phi lowering:

**Pre-lowering passes** (operate on SSA IR with phi nodes intact):

.. list-table::
   :header-rows: 1

   * - Pass
     - What it does
   * - ``dead_blocks``
     - Removes basic blocks unreachable from the entry block
   * - ``const_prop``
     - Propagates constant values through assignments and binary ops
   * - ``algebraic``
     - Algebraic simplifications (e.g., ``x + 0 → x``, ``x * 1 → x``)
   * - ``copy_prop``
     - Replaces uses of copy vars with their sources (``v1 = v0; use(v1)`` → ``use(v0)``)

**Post-lowering passes** (operate on phi-free ``LoweredModuleInfo``):

.. list-table::
   :header-rows: 1

   * - Pass
     - What it does
   * - ``empty_blocks``
     - Removes blocks with no instructions and a single unconditional jump
   * - ``dead_blocks``
     - Second dead block pass after structural changes
   * - ``merge_blocks``
     - Merges a block with its sole successor when no other predecessor exists
   * - ``copy_prop``
     - Copy propagation on lowered IR
   * - ``local_cse``
     - Local common subexpression elimination within each block
   * - ``gvn``
     - Global value numbering across blocks
   * - ``dead_instrs``
     - Removes instructions whose results are never used
   * - ``branch_fold``
     - Folds constant-condition branches (``br_if true → jump``)
   * - ``licm``
     - Loop-invariant code motion: hoists invariant computations out of loops

Both phases run up to 2 iterations until fixed point. Passes run only when ``--optimize`` / ``-O`` is passed.

Backend
~~~~~~~

*Source:* ``crates/herkos-core/src/backend/``

The ``Backend`` trait abstracts the code emission strategy. Currently only ``SafeBackend`` is implemented:

- Emits 100% safe Rust
- Every memory access goes through bounds-checked wrappers returning ``WasmResult<T>``
- No verification metadata required

For the planned verified and hybrid backends, see :doc:`/FUTURE`.

Code Generator
~~~~~~~~~~~~~~

*Source:* ``crates/herkos-core/src/codegen/``

Walks the ``LoweredModuleInfo`` and emits Rust source code via the configured ``Backend``:

.. list-table::
   :header-rows: 1

   * - Codegen module
     - Responsibility
   * - ``module.rs``
     - Top-level orchestration; module struct definition
   * - ``function.rs``
     - Function signatures, local declarations, block state machines
   * - ``instruction.rs``
     - Individual ``IrInstr`` → Rust expression
   * - ``traits.rs``
     - ``ModuleHostTrait`` generation from function imports
   * - ``export.rs``
     - Export method generation (forwarding to internal functions)
   * - ``constructor.rs``
     - ``new()`` with data segment and element segment initialization
   * - ``env.rs``
     - ``Env<H>`` context struct bundling host + globals
   * - ``types.rs``
     - Wasm → Rust type name mapping
   * - ``utils.rs``
     - Call arg building, import grouping

**Multi-block control flow** uses a local ``Block`` enum and a ``loop { match __block { … } }`` state machine. Single-block functions optimize to flat inline code.

Tests (``herkos-tests``)
------------------------

*Source:* ``crates/herkos-tests/``

End-to-end test crate that compiles WAT/C/Rust sources to ``.wasm``, transpiles them, and runs the output.

Test pipeline
~~~~~~~~~~~~~

.. code-block:: text

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

Test categories
~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1

   * - Category
     - Test files
     - What's tested
   * - Arithmetic
     - ``arithmetic.rs``, ``numeric_ops.rs``
     - Wasm arithmetic, bitwise, comparison ops
   * - Memory
     - ``memory.rs``, ``memory_grow.rs``, ``subwidth_mem.rs``, ``bulk_memory.rs``
     - Load/store, memory.grow, sub-width access, memory.copy/fill/init
   * - Control flow
     - ``control_flow.rs``, ``early_return.rs``, ``select.rs``, ``unreachable.rs``
     - Block, loop, if, br, br_table, select
   * - Functions
     - ``function_calls.rs``, ``indirect_calls.rs``, ``indirect_call_import.rs``
     - Direct calls, call_indirect dispatch, indirect calls through imports
   * - Imports/Exports
     - ``import_traits.rs``, ``import_memory.rs``, ``import_multi.rs``, ``module_wrapper.rs``, ``call_import_transitive.rs``
     - Trait-based imports, module wrapper, transitive import calls
   * - Locals
     - ``locals.rs``, ``locals_aliasing.rs``
     - Local variable handling
   * - Inter-module
     - ``inter_module_lending.rs``
     - Memory lending between modules
   * - E2E (C)
     - ``c_e2e.rs``, ``c_e2e_i64.rs``, ``c_e2e_loops.rs``, ``c_e2e_memory.rs``
     - Full C → Wasm → Rust pipeline
   * - E2E (Rust)
     - ``rust_e2e.rs``, ``rust_e2e_control.rs``, ``rust_e2e_i64.rs``, ``rust_e2e_heavy_fibo.rs``, ``rust_e2e_memory_bench.rs``
     - Pre-generated Rust modules

Running tests
~~~~~~~~~~~~~

.. code-block:: bash

   cargo test -p herkos-core     # transpiler unit tests (IR, optimizer, codegen)
   cargo test -p herkos-runtime  # runtime unit tests

   # herkos-tests must always be run twice:
   HERKOS_OPTIMIZE=0 cargo test -p herkos-tests   # unoptimized transpiler output
   HERKOS_OPTIMIZE=1 cargo test -p herkos-tests   # optimized transpiler output

``HERKOS_OPTIMIZE`` is read by ``herkos-tests/build.rs`` at compile time. When set to ``"1"``, the test suite's ``.wasm`` sources are transpiled with the IR optimization pipeline enabled; any other value (or unset) disables it. Running both variants is required — it verifies that the optimizer is *semantics-preserving*: the same inputs must produce the same outputs regardless of optimization level. CI enforces both runs as separate steps. The variable has no effect on ``herkos-core``, ``herkos-runtime``, or any production code.

Benchmarks
----------

*Source:* ``crates/herkos-tests/benches/``

Performance benchmarks using Criterion. Currently includes Fibonacci benchmarks comparing transpiled Wasm execution against native Rust.

.. code-block:: bash

   cargo bench -p herkos-tests
