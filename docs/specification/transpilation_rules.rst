.. _transpilation-rules:

Transpilation Rules
===================

This section describes how Wasm constructs map to Rust code in the safe backend.

Function Translation
--------------------

.. spec:: Function Translation
   :id: SPEC_FUNCTION_TRANSLATION
   :satisfies: REQ_TRANS_FUNCTIONS, REQ_TRANS_DETERMINISTIC
   :tags: transpilation, function

   Wasm functions to Rust functions via SSA IR.

Wasm functions become Rust functions. Module state is threaded through as parameters. Functions that call imports or touch mutable globals receive an ``Env<H>`` context struct; pure computation functions omit it.

.. code-block:: rust

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

Only state that the function actually uses is passed. Memory is omitted for memory-free functions; ``env`` is omitted when there are no imports and no mutable globals.

Control Flow
------------

.. spec:: Control Flow Mapping
   :id: SPEC_CONTROL_FLOW
   :satisfies: REQ_TRANS_CONTROL_FLOW
   :tags: transpilation, control

   Wasm structured control flow mapped to safe Rust (loop/break/if).

Wasm structured control flow is lowered to basic blocks in the IR, then emitted as a state-machine loop in Rust:

.. code-block:: rust

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

.. list-table::
   :header-rows: 1

   * - Wasm
     - IR
     - Rust (state machine)
   * - ``block`` / ``end``
     - jump to successor block
     - ``__block = __Block::BN``
   * - ``loop``
     - back-edge jump to loop header
     - ``__block = __Block::BHeader``
   * - ``if / else / end``
     - ``BranchIf { cond, if_true, if_false }``
     - ``if cond { __block = BT } else { __block = BF }``
   * - ``br $label``
     - ``Jump { target }``
     - ``__block = __Block::BN``
   * - ``br_if $label``
     - ``BranchIf``
     - ``if cond { __block = BT } else { __block = BF }``
   * - ``br_table``
     - ``BranchTable { index, targets, default }``
     - ``match index { 0 => __block = B0, … _ => __block = BD }``
   * - ``unreachable``
     - ``Unreachable`` terminator
     - ``return Err(WasmTrap::Unreachable)``

Single-block functions (no branches) skip the state machine entirely and emit flat inline code.

Error Handling
--------------

.. spec:: Error Handling
   :id: SPEC_ERROR_HANDLING
   :satisfies: REQ_ERR_TRAPS
   :tags: transpilation, error

   Trap-based error handling via WasmTrap/WasmResult.

*Implementation:* ``crates/herkos-runtime/src/lib.rs``

.. code-block:: rust

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

No panics, no unwinding. The ``?`` operator propagates traps up the call stack.

Arithmetic Operations
---------------------

.. spec:: Arithmetic Operations
   :id: SPEC_ARITHMETIC
   :satisfies: REQ_TRANS_FUNCTIONS
   :tags: transpilation, arithmetic

   Wasm arithmetic semantics (wrapping, trapping division, IEEE 754 floats).

*Implementation:* ``crates/herkos-runtime/src/ops.rs``

Wasm arithmetic operations that can trap (division, remainder, truncation) return ``WasmResult``:

.. code-block:: rust

   fn i32_div_s(a: i32, b: i32) -> WasmResult<i32>;  // traps on /0 or overflow
   fn i32_rem_u(a: i32, b: i32) -> WasmResult<i32>;   // traps on /0
   fn i32_trunc_f32_s(a: f32) -> WasmResult<i32>;     // traps on out-of-range

Non-trapping arithmetic uses Rust's wrapping operations (``wrapping_add``, ``wrapping_mul``, etc.) per the Wasm spec.

Function Calls
--------------

.. spec:: Function Calls
   :id: SPEC_FUNCTION_CALLS
   :satisfies: REQ_TRANS_INDIRECT_CALLS, REQ_TRANS_TYPE_EQUIVALENCE
   :tags: transpilation, call

   Direct calls, indirect calls via table, structural type equivalence.

Direct Calls (``call``)
~~~~~~~~~~~~~~~~~~~~~~~

Direct calls transpile to regular Rust function calls with state threaded through:

.. code-block:: rust

   // Wasm: call $func_3 (with 2 args on the stack)
   v5 = func_3(memory, globals, table, v3, v4)?;

Indirect Calls (``call_indirect``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``call_indirect`` implements function pointers. The transpiler emits a static match dispatch:

.. code-block:: rust

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

**Why match-based dispatch?** Function pointer arrays, ``dyn Fn`` trait objects, or computed gotos all require ``unsafe``, heap allocation, or break ``no_std`` compatibility. A match statement is 100% safe, ``no_std`` compatible, and LLVM optimizes it to a jump table when arms are dense.

The ``_ =>`` arm handles func_index values that don't match any function of the right type — a safety net for corrupted table entries.

Structural Type Equivalence
~~~~~~~~~~~~~~~~~~~~~~~~~~~

The Wasm spec requires ``call_indirect`` to use **structural equivalence**: two type indices match if they have identical parameter and result types, regardless of index.

.. code-block:: text

   Type 0: (i32, i32) → i32  →  canonical = 0
   Type 1: (i32, i32) → i32  →  canonical = 0  (same signature as type 0)
   Type 2: (i32) → i32       →  canonical = 2  (new signature)

The transpiler builds a canonical type index mapping at transpile time. Both ``FuncRef.type_index`` and the type check use canonical indices. At runtime, the check is a simple integer comparison.

Bulk Memory Operations
----------------------

.. spec:: Bulk Memory Operations
   :id: SPEC_BULK_MEMORY
   :satisfies: REQ_MEM_BULK_OPS, REQ_MEM_DATA_SEGMENTS
   :tags: transpilation, memory, bulk

   memory.fill, memory.init, data.drop, memory.copy.

*Implementation:* ``crates/herkos-runtime/src/memory.rs`` lines 149–174

The WebAssembly bulk memory operations allow efficient copying and initialization of memory regions without scalar load/store loops.

``memory.fill``
~~~~~~~~~~~~~~~

Fills a region of memory with a byte value. Per Wasm spec, only the low 8 bits of the value are used.

.. code-block:: rust

   impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
       pub fn fill(&mut self, dst: usize, val: u8, len: usize) -> WasmResult<()>;
   }

Generated code:

.. code-block:: rust

   // Wasm: memory.fill $dst $val $len
   memory.fill(dst as usize, val as u8, len as usize)?;

Traps ``OutOfBounds`` if ``[dst, dst + len)`` exceeds active memory. Length zero is a no-op.

``memory.init``
~~~~~~~~~~~~~~~

Copies data from a passive data segment into memory at runtime. Each data segment is stored as a constant ``&'static [u8]`` in the generated code.

.. code-block:: rust

   impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
       pub fn init_data_partial(&mut self, dst: usize, data: &[u8], src_offset: usize, len: usize) -> WasmResult<()>;
   }

Generated code:

.. code-block:: rust

   // Wasm: memory.init $data_segment $dst $src_offset $len
   memory.init_data_partial(dst as usize, &DATA_SEGMENT_0, src_offset as usize, len as usize)?;

Traps ``OutOfBounds`` if either region (source or destination) exceeds bounds:

- Source: ``[src_offset, src_offset + len)`` must be within the data segment
- Destination: ``[dst, dst + len)`` must be within active memory

``data.drop``
~~~~~~~~~~~~~

Marks a data segment as dropped (per Wasm spec). In the safe backend this is a no-op because data segments are stored as constant references and cannot actually be deallocated.

.. code-block:: rust

   // Wasm: data.drop $segment
   // (no-op in safe backend — const slices persist)

In future verified and hybrid backends, ``data.drop`` may enable optimizations: proving that dropped segments are never accessed again could allow proving certain addresses as never-in-bounds.
