.. _performance:

Performance
===========

Overhead
--------

.. list-table::
   :header-rows: 1

   * - Backend
     - Overhead
     - Source
     - Status
   * - Safe
     - 15–30%
     - Runtime bounds check on every memory access
     - Implemented
   * - Verified
     - 0–5%
     - Function call indirection only
     - Planned (:doc:`/FUTURE`)
   * - Hybrid
     - 5–15%
     - Mix of checked and proven accesses
     - Planned (:doc:`/FUTURE`)

Monomorphization Bloat Mitigation
---------------------------------

Each distinct ``MAX_PAGES`` and trait bound combination generates separate code. Mitigation strategies:

Outline pattern (mandatory for runtime)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Move logic into non-generic inner functions. Generic wrapper is a thin shell:

.. code-block:: rust

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

``MAX_PAGES`` normalization
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Use standard sizes (16, 64, 256, 1024) instead of exact declared maximums. Two modules with ``MAX_PAGES=253`` and ``MAX_PAGES=260`` both use ``MAX_PAGES=256``.

Trait objects for cold paths
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Use ``&mut dyn Trait`` instead of generics for rarely-called code (error handling, initialization).

LTO
~~~

Link-time optimization eliminates unreachable monomorphized copies.

Transpiler Parallelization
--------------------------

IR building and code generation are embarrassingly parallel — each function is independent:

.. code-block:: text

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

Activation heuristic: use ``rayon`` parallel iterators when the module has 20+ functions. Output is deterministic regardless of thread count (``par_iter().enumerate()`` preserves order).

Comparison to Alternatives
--------------------------

.. list-table::
   :header-rows: 1

   * - Approach
     - Runtime Overhead
     - Isolation Strength
     - ``unsafe`` in output
   * - MMU/MPU
     - 10–50% (context switches)
     - Strong (hardware)
     - N/A
   * - herkos (safe)
     - 15–30%
     - Strong (runtime checks)
     - None
   * - herkos (verified, planned)
     - 0–5%
     - Strong (formal proofs)
     - Yes — proof-justified
   * - WebAssembly runtime
     - 20–100%
     - Strong (runtime sandbox)
     - N/A
   * - Software fault isolation
     - 10–30%
     - Medium (runtime)
     - N/A
