//! # IR Builder
//!
//! Translates a `ParsedModule` (parsed WebAssembly) into a `ModuleInfo` (SSA IR).
//!
//! ## Pipeline overview
//!
//! ```text
//! ParsedModule
//!      │
//!      ├─[analysis]──────────────────────────────────────────────┐
//!      │  extract_memory_info()  ─► MemoryInfo                   │
//!      │  extract_table_info()   ─► TableInfo                    │
//!      │  build_type_mappings()  ─► canonical_type, type_sigs    │
//!      │  build_imported_globals() ─► Vec<ImportedGlobalDef>     │
//!      │                                                         │
//!      └─[translate]─────────────────────────────────────────┐   │
//!         build_ir_functions()                               │   │
//!           └── for each local function:                     │   │
//!                 IrBuilder::translate_function()            │   │
//!                   └── for each Operator:                   │   │
//!                         translate_operator()               │   │
//!                           ├── emit_binop/emit_unop         │   │
//!                           ├── emit_load/emit_store         │   │
//!                           └── control-flow ops             │   │
//!                ─► Vec<IrFunction>                          │   │
//!                                                            │   │
//! ◄───────────────────────────────[assembly]─────────────────┘───┘
//!   assemble_module_metadata()
//!     ├── build_globals()
//!     ├── build_data_segments()
//!     ├── build_element_segments()
//!     ├── build_function_exports()
//!     ├── build_function_type_signatures()
//!     ├── build_call_indirect_signatures()
//!     └── build_function_imports()
//!          ─► ModuleInfo  ──► codegen
//! ```
//!
//! ## Architecture
//!
//! The builder is split into four sub-modules:
//!
//! | Module       | Responsibility                                               |
//! |--------------|--------------------------------------------------------------|
//! | [`core`]     | `IrBuilder` state machine, SSA allocation, control stack     |
//! | [`translate`]| Wasm operator → IR instruction dispatch                      |
//! | [`analysis`] | Extract per-section metadata from `ParsedModule`             |
//! | [`assembly`] | Assemble extracted pieces into a final `ModuleInfo`          |
//!
//! ### Flow
//!
//! 1. **Analysis phase**: Module structure is examined to extract fixed information
//!    (memory size, imported functions, type signatures, etc.)
//! 2. **Translation phase**: For each function, an `IrBuilder` walks the Wasm bytecode,
//!    maintaining a simulated Wasm value stack in SSA form, converting each operator
//!    to equivalent IR instructions and control flow.
//! 3. **Assembly phase**: All pieces are combined into the final `ModuleInfo`.
//!
//! The `IrBuilder` is a stack machine interpreter that:
//! - Simulates the WebAssembly evaluation stack using SSA variables (`VarId`)
//! - Maintains a control-flow stack tracking nested `block`/`loop`/`if` frames
//! - Allocates new basic blocks as needed (loops, branches, control structures)
//! - Emits IR instructions to the current block
//! - Terminates blocks with control-flow terminators (`Jump`, `BranchIf`, `Return`, etc.)

mod analysis;
mod assembly;
pub mod core;
mod translate;

pub use core::ModuleContext;

use super::types::ModuleInfo;
use crate::parser::ParsedModule;
use crate::TranspileOptions;
use anyhow::Result;

/// Build complete module metadata from a parsed WebAssembly module.
///
/// This is the main entry point for IR construction, coordinating all
/// the intermediate steps needed to produce a fully-formed `ModuleInfo`.
pub fn build_module_info(parsed: &ParsedModule, options: &TranspileOptions) -> Result<ModuleInfo> {
    // Analyze module structure (memory, table, types)
    let mem_info = analysis::extract_memory_info(parsed, options)?;
    let table_info = analysis::extract_table_info(parsed);
    let (canonical_type, type_sigs) = analysis::build_type_mappings(parsed); // TODO: could we maybe just build 2 different functions for canonical-types and type-sigs instead of returning a tuple?

    // Analyze imports
    let imported_globals = analysis::build_imported_globals(parsed);
    let num_imported_functions = parsed.num_imported_functions;

    // Translate WebAssembly to intermediate representation
    let ir_functions = analysis::build_ir_functions(parsed, &type_sigs, num_imported_functions)?;

    // Assemble module metadata for code generation
    assembly::assemble_module_metadata(
        parsed,
        &mem_info,
        &table_info,
        &canonical_type,
        ir_functions,
        num_imported_functions as usize,
        &imported_globals,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::types::WasmType;
    use wasmparser::ValType;

    /// Test the invariant: entry_block is always BlockId(0)
    #[test]
    fn entry_block_is_always_block_zero() {
        let mut builder = core::IrBuilder::new();

        // Simple function: fn add(a: i32, b: i32) -> i32 { a + b }
        let params = vec![(ValType::I32, WasmType::I32), (ValType::I32, WasmType::I32)];
        let operators = vec![
            wasmparser::Operator::LocalGet { local_index: 0 },
            wasmparser::Operator::LocalGet { local_index: 1 },
            wasmparser::Operator::I32Add,
            wasmparser::Operator::End,
        ];

        let module_ctx = ModuleContext {
            func_signatures: vec![],
            type_signatures: vec![],
            num_imported_functions: 0,
            func_imports: vec![],
        };

        let ir_func = builder
            .translate_function(&params, &[], Some(WasmType::I32), &operators, &module_ctx)
            .expect("translation should succeed");

        // INVARIANT CHECK: entry_block must be BlockId(0)
        assert_eq!(
            ir_func.entry_block,
            crate::ir::types::BlockId(0),
            "entry_block must always be BlockId(0)"
        );

        // Additional sanity checks
        assert!(
            !ir_func.blocks.is_empty(),
            "function must have at least one block"
        );
        assert_eq!(
            ir_func.blocks[0].id,
            crate::ir::types::BlockId(0),
            "first block in the blocks vector must be BlockId(0)"
        );
    }

    /// Test that entry_block == BlockId(0) even for void functions
    #[test]
    fn entry_block_is_zero_for_void_function() {
        let mut builder = core::IrBuilder::new();

        // Void function: fn noop() { }
        let operators = vec![wasmparser::Operator::Nop, wasmparser::Operator::End];

        let module_ctx = ModuleContext {
            func_signatures: vec![],
            type_signatures: vec![],
            num_imported_functions: 0,
            func_imports: vec![],
        };

        let ir_func = builder
            .translate_function(&[], &[], None, &operators, &module_ctx)
            .expect("translation should succeed");

        assert_eq!(
            ir_func.entry_block,
            crate::ir::types::BlockId(0),
            "entry_block must be BlockId(0) even for void functions"
        );
    }

    /// Test that entry_block == BlockId(0) for functions with locals
    #[test]
    fn entry_block_is_zero_with_locals() {
        let mut builder = core::IrBuilder::new();

        let params = vec![(ValType::I32, WasmType::I32)];
        let locals = vec![ValType::I32, ValType::I32];
        let operators = vec![
            wasmparser::Operator::I32Const { value: 42 },
            wasmparser::Operator::End,
        ];

        let module_ctx = ModuleContext {
            func_signatures: vec![],
            type_signatures: vec![],
            num_imported_functions: 0,
            func_imports: vec![],
        };

        let ir_func = builder
            .translate_function(
                &params,
                &locals,
                Some(WasmType::I32),
                &operators,
                &module_ctx,
            )
            .expect("translation should succeed");

        assert_eq!(
            ir_func.entry_block,
            crate::ir::types::BlockId(0),
            "entry_block must be BlockId(0) regardless of locals"
        );
    }

    /// Test that local variables are properly tracked and distinguished from parameters
    #[test]
    fn local_variables_separate_from_params() {
        let mut builder = core::IrBuilder::new();

        // Function: (param i32) (result i32)
        //   (local i32)
        //   local.get 0      ;; param
        //   local.set 1      ;; store to local (index 1)
        //   local.get 1      ;; get local back
        let params = vec![(ValType::I32, WasmType::I32)];
        let locals = vec![ValType::I32];
        let operators = vec![
            wasmparser::Operator::LocalGet { local_index: 0 },
            wasmparser::Operator::LocalSet { local_index: 1 },
            wasmparser::Operator::LocalGet { local_index: 1 },
            wasmparser::Operator::End,
        ];

        let module_ctx = ModuleContext {
            func_signatures: vec![],
            type_signatures: vec![],
            num_imported_functions: 0,
            func_imports: vec![],
        };

        let ir_func = builder
            .translate_function(
                &params,
                &locals,
                Some(WasmType::I32),
                &operators,
                &module_ctx,
            )
            .expect("translation should succeed");

        // Parameter 0 and local 1 must be different VarIds
        let param_var = ir_func.params[0].0;
        let local_var = ir_func.locals[0].0;
        assert_ne!(
            param_var, local_var,
            "param and local must have distinct VarIds"
        );

        // Verify local tracking
        assert_eq!(
            ir_func.locals.len(),
            1,
            "should have exactly 1 declared local"
        );
        assert_eq!(
            ir_func.locals[0].1,
            WasmType::I32,
            "local should have type i32"
        );

        // Verify params are still tracked
        assert_eq!(ir_func.params.len(), 1, "should have exactly 1 param");
        assert_eq!(
            ir_func.params[0].1,
            WasmType::I32,
            "param should have type i32"
        );
    }

    /// Test with multiple locals of different types
    #[test]
    fn multiple_locals_different_types() {
        let mut builder = core::IrBuilder::new();

        // Function: (param i32) (result i32)
        //   (local i32 i64 f32)
        //   local.get 0
        let params = vec![(ValType::I32, WasmType::I32)];
        let locals = vec![ValType::I32, ValType::I64, ValType::F32];
        let operators = vec![
            wasmparser::Operator::LocalGet { local_index: 0 },
            wasmparser::Operator::End,
        ];

        let module_ctx = ModuleContext {
            func_signatures: vec![],
            type_signatures: vec![],
            num_imported_functions: 0,
            func_imports: vec![],
        };

        let ir_func = builder
            .translate_function(
                &params,
                &locals,
                Some(WasmType::I32),
                &operators,
                &module_ctx,
            )
            .expect("translation should succeed");

        // Verify all locals are tracked with correct types
        assert_eq!(ir_func.locals.len(), 3);
        assert_eq!(ir_func.locals[0].1, WasmType::I32);
        assert_eq!(ir_func.locals[1].1, WasmType::I64);
        assert_eq!(ir_func.locals[2].1, WasmType::F32);

        // All locals should have different VarIds
        let var_ids: Vec<crate::ir::types::VarId> =
            ir_func.locals.iter().map(|(v, _)| *v).collect();
        assert_eq!(var_ids[0], var_ids[0]);
        assert_ne!(var_ids[0], var_ids[1]);
        assert_ne!(var_ids[1], var_ids[2]);
    }
}
