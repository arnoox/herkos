Implementation Modules
======================

Key implementation modules in herkos-core and herkos-runtime.

.. impl:: Wasm binary parser
   :id: IMPL_PARSER
   :source_file: crates/herkos-core/src/parser/mod.rs
   :tags: parser, binary
   :satisfies: REQ_TRANS_FUNCTIONS
   :implements: WASM_BIN_MODULES, WASM_BIN_SECTIONS, WASM_BIN_TYPES, WASM_BIN_INSTRUCTIONS

   ``crates/herkos-core/src/parser/mod.rs``

.. impl:: Wasm-to-IR translation
   :id: IMPL_IR_BUILDER
   :source_file: crates/herkos-core/src/ir/builder/
   :tags: ir, builder
   :satisfies: REQ_TRANS_FUNCTIONS, REQ_TRANS_CONTROL_FLOW
   :implements: WASM_MOD_FUNCTIONS, WASM_EXEC_CONTROL, WASM_EXEC_CALLS

   ``crates/herkos-core/src/ir/builder/``

.. impl:: Safe backend (bounds-checked codegen)
   :id: IMPL_BACKEND_SAFE
   :source_file: crates/herkos-core/src/backend/safe.rs
   :tags: backend, safe, codegen
   :satisfies: REQ_MEM_BOUNDS_CHECKED, REQ_TRANS_SELF_CONTAINED
   :implements: WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_MEMORY

   ``crates/herkos-core/src/backend/safe.rs``

.. impl:: Instruction code generation
   :id: IMPL_CODEGEN_INSTR
   :source_file: crates/herkos-core/src/codegen/instruction.rs
   :tags: codegen, instruction
   :satisfies: REQ_TRANS_FUNCTIONS
   :implements: WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS, WASM_EXEC_MEMORY, WASM_EXEC_CALLS

   ``crates/herkos-core/src/codegen/instruction.rs``

.. impl:: Module code generation
   :id: IMPL_CODEGEN_MODULE
   :source_file: crates/herkos-core/src/codegen/module.rs
   :tags: codegen, module
   :satisfies: REQ_MOD_TWO_TYPES, REQ_TRANS_SELF_CONTAINED, REQ_TRANS_VERSION_INFO
   :implements: WASM_MOD_FUNCTIONS, WASM_MOD_EXPORTS, WASM_MOD_IMPORTS, WASM_MOD_GLOBALS, WASM_MOD_TABLES

   ``crates/herkos-core/src/codegen/module.rs``

.. impl:: IsolatedMemory runtime
   :id: IMPL_RUNTIME_MEMORY
   :source_file: crates/herkos-runtime/src/memory.rs
   :tags: runtime, memory
   :satisfies: REQ_MEM_PAGE_MODEL, REQ_MEM_COMPILE_TIME_SIZE, REQ_MEM_BOUNDS_CHECKED, REQ_MEM_GROW_NO_ALLOC, REQ_MEM_BULK_OPS
   :implements: WASM_MEMORY_TYPE, WASM_MOD_MEMORIES, WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_EXEC_MEMORY

   ``crates/herkos-runtime/src/memory.rs``

.. impl:: Table runtime (indirect calls)
   :id: IMPL_RUNTIME_TABLE
   :source_file: crates/herkos-runtime/src/table.rs
   :tags: runtime, table
   :satisfies: REQ_MOD_TABLE, REQ_TRANS_INDIRECT_CALLS
   :implements: WASM_TABLE_TYPE, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_CALL_INDIRECT

   ``crates/herkos-runtime/src/table.rs``

.. impl:: Wasm arithmetic operations
   :id: IMPL_RUNTIME_OPS
   :source_file: crates/herkos-runtime/src/ops.rs
   :tags: runtime, ops, arithmetic
   :satisfies: REQ_ERR_TRAPS
   :implements: WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

   ``crates/herkos-runtime/src/ops.rs``
