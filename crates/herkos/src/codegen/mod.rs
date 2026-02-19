//! Code generation — emits Rust source code from IR.
//!
//! # Overview
//!
//! This module walks the IR and uses a Backend to emit complete Rust functions and module
//! structures. It generates a `Module<Globals, MAX_PAGES, 0>` struct with constructor,
//! internal functions, and exported methods.
//!
//! # Architecture
//!
//! The code generation pipeline is organized into focused sub-modules:
//!
//! ```text
//!                          ┌─────────────────────────────────────┐
//!                          │      ModuleInfo (IR input)          │
//!                          │  ┌─ IR functions                    │
//!                          │  ├─ Imports/Exports                 │
//!                          │  ├─ Globals, Memory, Table          │
//!                          │  └─ Data/Element segments           │
//!                          └─────────────────────────────────────┘
//!                                            │
//!                                            ▼
//!                          ┌─────────────────────────────────────┐
//!                          │   generate_module_with_info()       │
//!                          │   (Main entry point)                │
//!                          └─────────────────────────────────────┘
//!                                            │
//!                                            ▼
//!                      ┌──────────────────────────────────────┐
//!                      │      MODULE WRAPPER GENERATION      │
//!                      ├─ Preamble                           │
//!                      ├─ Host traits                        │
//!                      ├─ Const globals                      │
//!                      ├─ Globals struct                     │
//!                      ├─ WasmModule newtype                 │
//!                      ├─ Constructor (new())                │
//!                      ├─ Private functions                  │
//!                      ├─ Export impl block                  │
//!                      └──────────────────────────────────────┘
//!                                            │
//!                                            ▼
//!                ┌──────────────────────┐
//!                │   Rust Source Code   │
//!                │   (ready to compile) │
//!                └──────────────────────┘
//!
//!
//! # Sub-modules
//!
//! Each sub-module handles a specific aspect of code generation:
//!
//! - **`module`**: Main generation orchestration (`generate_module_with_info`, standalone vs wrapper)
//! - **`traits`**: Host trait definitions from imports (`EnvImports`, `WasiImports`, etc.)
//! - **`constructor`**: Module initialization (`new()`, data/element segments, const globals)
//! - **`function`**: IR function translation (signatures, blocks, variables, SSA)
//! - **`instruction`**: Individual instruction code generation and terminators
//! - **`export`**: Export method generation (forwarding to internal functions)
//! - **`types`**: Type conversions (Wasm→Rust, WasmResult formatting)
//! - **`utils`**: Utility functions (call arg building, grouping)
//!
//! # Control Flow Example
//!
//! When transpiling a module with a memory and an export:
//!
//! ```text
//! ModuleInfo {
//!   has_memory: true,
//!   max_pages: 16,
//!   func_exports: [FuncExport { name: "process", func_index: 0 }],
//!   ...
//! }
//!    │
//!    ├─→ generate_module_with_info()
//!    │     └─→ generate_wrapper_module()
//!    │
//!    ├─→ [Constructor generation]
//!    │   └─ emit_element_segments() (if table)
//!    │   └─ Data segment init (byte-by-byte)
//!    │
//!    ├─→ [Function generation per func in IR]
//!    │   └─→ generate_function_with_info("func_0", ...)
//!    │       ├─→ generate_signature_with_info()
//!    │       │   ├─ Collect trait bounds if needs_host
//!    │       │   ├─ Add globals/memory/table/host parameters
//!    │       │   └─ Build generic param H (if multiple trait bounds)
//!    │       │
//!    │       ├─→ [Variable type inference from instructions]
//!    │       │
//!    │       └─→ [Block translation]
//!    │           ├─ Single-block: flat code emission
//!    │           └─ Multi-block: state machine with Block enum + loop/match
//!    │
//!    ├─→ [Per instruction]
//!    │   └─→ generate_instruction_with_info()
//!    │       ├─ Delegates to backend.emit_*() for most operations
//!    │       ├─ CallImport → host.func_name()
//!    │       ├─ CallIndirect → dispatch match on func_index
//!    │       └─ GlobalGet/Set → redirect to imported globals via host traits
//!    │
//!    └─→ [Export impl generation]
//!        └─→ generate_export_impl()
//!            └─ pub fn process(&mut self, ...) { func_0(...) }
//!
//! ```
//!
//! # Key Design Decisions
//!
//! 1. **Backend Delegation**: All instruction emission is delegated to a `Backend` trait
//!    (SafeBackend, VerifiedBackend, etc.). This module orchestrates structure;
//!    the backend handles the actual Rust code patterns.
//!
//! 2. **Trait-Based Imports**: Imported functions become trait bounds on a generic `H`
//!    parameter. Each import module gets its own trait (e.g., `EnvImports`, `WasiImports`).
//!    This ensures zero-cost dispatch and type safety.
//!
//! 3. **SSA Variable Inference**: Types are inferred from instructions using a HashMap,
//!    ensuring correct Rust type declarations for all intermediate values.
//!
//! 4. **State Machine for Multi-Block Functions**: Functions with multiple blocks emit
//!    a local `Block` enum and a `loop { match }` structure. Single-block functions
//!    optimize to flat code.
//!
//! 5. **Const Generics Over Runtime Sizes**: `MAX_PAGES` and `TABLE_MAX` are const
//!    generics, not runtime values. This enables monomorphization and zero-cost memory
//!    bounds checking.
//!
//! # Integration Points
//!
//! - **Input**: [`ModuleInfo`] from IR builder, [`Backend`] trait for emission rules
//! - **Output**: Formatted Rust source code (typically passed through `rustfmt`)
//! - **Error Handling**: Uses `anyhow::Result` for context on generation failures

pub mod constructor;
pub mod export;
pub mod function;
pub mod instruction;
pub mod module;
pub mod traits;
pub mod types;
pub mod utils;

use crate::backend::Backend;
use crate::ir::*;
use anyhow::Result;

/// Main code generator struct that orchestrates emission of Rust code from IR.
///
/// # Example
///
/// ```ignore
/// let backend = SafeBackend::new();
/// let codegen = CodeGenerator::new(&backend);
/// let rust_code = codegen.generate_module_with_info(&module_info)?;
/// ```
pub struct CodeGenerator<'a, B: Backend> {
    backend: &'a B,
}

impl<'a, B: Backend> CodeGenerator<'a, B> {
    /// Create a new code generator with a given backend.
    pub fn new(backend: &'a B) -> Self {
        CodeGenerator { backend }
    }

    /// Generate a complete Rust module from IR with full module info.
    ///
    /// This is the main entry point. It generates a module wrapper structure.
    pub fn generate_module_with_info(&self, info: &ModuleInfo) -> Result<String> {
        module::generate_module_with_info(self.backend, info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::SafeBackend;

    #[test]
    fn generate_simple_function() {
        // Build a simple IR function: fn add(v0: i32, v1: i32) -> i32 { return v0 + v1; }
        let ir_func = IrFunction {
            params: vec![(VarId(0), WasmType::I32), (VarId(1), WasmType::I32)],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                label: "block0".to_string(),
                instructions: vec![IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Add,
                    lhs: VarId(0),
                    rhs: VarId(1),
                }],
                terminator: IrTerminator::Return {
                    value: Some(VarId(2)),
                },
            }],
            entry_block: BlockId(0),
            return_type: Some(WasmType::I32),
        };

        let backend = SafeBackend::new();
        let _codegen = CodeGenerator::new(&backend);
        let info = ModuleInfo {
            has_memory: false,
            has_memory_import: false,
            max_pages: 0,
            initial_pages: 0,
            table_initial: 0,
            table_max: 0,
            element_segments: Vec::new(),
            globals: Vec::new(),
            data_segments: Vec::new(),
            func_exports: Vec::new(),
            func_signatures: Vec::new(),
            type_signatures: Vec::new(),
            canonical_type: Vec::new(),
            func_imports: Vec::new(),
            imported_globals: Vec::new(),
            ir_functions: Vec::new(),
        };
        let code =
            function::generate_function_with_info(&backend, &ir_func, "add", &info, true).unwrap();

        println!("Generated code:\n{}", code);

        // Basic checks
        assert!(code.contains("pub fn add("));
        assert!(code.contains("v0: i32"));
        assert!(code.contains("v1: i32"));
        assert!(code.contains("-> WasmResult<i32>"));
        assert!(code.contains("wrapping_add"));
        assert!(code.contains("return Ok(v2)"));
    }

    #[test]
    fn generate_void_function() {
        // fn noop() -> () { return; }
        let ir_func = IrFunction {
            params: vec![],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                label: "block0".to_string(),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            }],
            entry_block: BlockId(0),
            return_type: None,
        };

        let backend = SafeBackend::new();
        let _codegen = CodeGenerator::new(&backend);
        let info = ModuleInfo {
            has_memory: false,
            has_memory_import: false,
            max_pages: 0,
            initial_pages: 0,
            table_initial: 0,
            table_max: 0,
            element_segments: Vec::new(),
            globals: Vec::new(),
            data_segments: Vec::new(),
            func_exports: Vec::new(),
            func_signatures: Vec::new(),
            type_signatures: Vec::new(),
            canonical_type: Vec::new(),
            func_imports: Vec::new(),
            imported_globals: Vec::new(),
            ir_functions: Vec::new(),
        };
        let code =
            function::generate_function_with_info(&backend, &ir_func, "noop", &info, true).unwrap();

        assert!(code.contains("pub fn noop()"));
        assert!(code.contains("-> WasmResult<()>"));
        assert!(code.contains("return Ok(())"));
    }

    #[test]
    fn generate_function_with_import_call() {
        use crate::TranspileOptions;

        // WAT module that imports and calls a function
        // Include a mutable global to trigger wrapper generation
        let wat = r#"
            (module
                (import "env" "log" (func $log (param i32)))
                (global $counter (mut i32) (i32.const 0))
                (func (export "test") (param i32)
                    local.get 0
                    call $log
                )
            )
        "#;

        let wasm = wat::parse_str(wat).unwrap();
        let rust_code = crate::transpile(&wasm, &TranspileOptions::default()).unwrap();

        println!("Generated code:\n{}", rust_code);

        // Verify the generated code contains:
        // 1. Trait definition for imports
        assert!(
            rust_code.contains("pub trait EnvImports"),
            "Should generate EnvImports trait"
        );

        // 2. Trait method signature
        assert!(
            rust_code.contains("fn log(&mut self, arg0: i32) -> WasmResult<()>"),
            "Trait should have log method"
        );

        // 3. Host parameter with trait bound in function signature
        assert!(
            rust_code.contains("host: &mut impl EnvImports"),
            "Function should have host parameter with EnvImports trait bound"
        );

        // 4. Call to host.log()
        assert!(
            rust_code.contains("host.log("),
            "Function should call host.log()"
        );

        // 5. Export method should also have host parameter and forward it
        assert!(
            rust_code.contains("pub fn test(") && rust_code.contains("host: &mut impl EnvImports"),
            "Export method should have host parameter with trait bound"
        );
    }

    #[test]
    fn generate_i64_variables_with_correct_types() {
        // fn add64(v0: i64, v1: i64) -> i64 { return v0 + v1; }
        let ir_func = IrFunction {
            params: vec![(VarId(0), WasmType::I64), (VarId(1), WasmType::I64)],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                label: "block0".to_string(),
                instructions: vec![IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I64Add,
                    lhs: VarId(0),
                    rhs: VarId(1),
                }],
                terminator: IrTerminator::Return {
                    value: Some(VarId(2)),
                },
            }],
            entry_block: BlockId(0),
            return_type: Some(WasmType::I64),
        };

        let backend = SafeBackend::new();
        let _codegen = CodeGenerator::new(&backend);
        let info = ModuleInfo {
            has_memory: false,
            has_memory_import: false,
            max_pages: 0,
            initial_pages: 0,
            table_initial: 0,
            table_max: 0,
            element_segments: Vec::new(),
            globals: Vec::new(),
            data_segments: Vec::new(),
            func_exports: Vec::new(),
            func_signatures: Vec::new(),
            type_signatures: Vec::new(),
            canonical_type: Vec::new(),
            func_imports: Vec::new(),
            imported_globals: Vec::new(),
            ir_functions: Vec::new(),
        };
        let code = function::generate_function_with_info(&backend, &ir_func, "add64", &info, true)
            .unwrap();

        println!("Generated code:\n{}", code);

        assert!(code.contains("v0: i64"));
        assert!(code.contains("v1: i64"));
        assert!(code.contains("-> WasmResult<i64>"));
        // v2 should be declared as i64, not i32
        assert!(code.contains("let mut v2: i64 = 0i64;"));
        assert!(!code.contains("let mut v2: i32"));
    }

    #[test]
    fn generate_mixed_types() {
        // A function that uses i64 const and an i64 comparison (which returns i32)
        let ir_func = IrFunction {
            params: vec![(VarId(0), WasmType::I64)],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                label: "block0".to_string(),
                instructions: vec![
                    IrInstr::Const {
                        dest: VarId(1),
                        value: IrValue::I64(42),
                    },
                    // i64 comparison produces i32 result
                    IrInstr::BinOp {
                        dest: VarId(2),
                        op: BinOp::I64Eq,
                        lhs: VarId(0),
                        rhs: VarId(1),
                    },
                ],
                terminator: IrTerminator::Return {
                    value: Some(VarId(2)),
                },
            }],
            entry_block: BlockId(0),
            return_type: Some(WasmType::I32),
        };

        let backend = SafeBackend::new();
        let _codegen = CodeGenerator::new(&backend);
        let info = ModuleInfo {
            has_memory: false,
            has_memory_import: false,
            max_pages: 0,
            initial_pages: 0,
            table_initial: 0,
            table_max: 0,
            element_segments: Vec::new(),
            globals: Vec::new(),
            data_segments: Vec::new(),
            func_exports: Vec::new(),
            func_signatures: Vec::new(),
            type_signatures: Vec::new(),
            canonical_type: Vec::new(),
            func_imports: Vec::new(),
            imported_globals: Vec::new(),
            ir_functions: Vec::new(),
        };
        let code =
            function::generate_function_with_info(&backend, &ir_func, "eq64", &info, true).unwrap();

        println!("Generated code:\n{}", code);

        assert!(code.contains("v0: i64"));
        // v1 is an i64 constant
        assert!(code.contains("let mut v1: i64 = 0i64;"));
        // v2 is the result of i64.eq, which is i32
        assert!(code.contains("let mut v2: i32 = 0i32;"));
    }

    #[test]
    fn generate_module_wrapper_with_mutable_global() {
        let ir_func = IrFunction {
            params: vec![],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                label: "block0".to_string(),
                instructions: vec![IrInstr::GlobalGet {
                    dest: VarId(0),
                    index: 0,
                }],
                terminator: IrTerminator::Return {
                    value: Some(VarId(0)),
                },
            }],
            entry_block: BlockId(0),
            return_type: Some(WasmType::I32),
        };

        let info = ModuleInfo {
            has_memory: false,
            has_memory_import: false,
            max_pages: 0,
            initial_pages: 0,
            table_initial: 0,
            table_max: 0,
            element_segments: Vec::new(),
            globals: vec![GlobalDef {
                wasm_type: WasmType::I32,
                mutable: true,
                init_value: GlobalInit::I32(0),
            }],
            data_segments: Vec::new(),
            func_exports: vec![FuncExport {
                name: "get_value".to_string(),
                func_index: 0,
            }],
            func_signatures: vec![FuncSignature {
                params: vec![],
                return_type: Some(WasmType::I32),
                type_idx: 0,
                needs_host: false,
            }],
            type_signatures: Vec::new(),
            canonical_type: Vec::new(),
            func_imports: Vec::new(),
            imported_globals: Vec::new(),
            ir_functions: vec![ir_func],
        };

        let backend = SafeBackend::new();
        let codegen = CodeGenerator::new(&backend);
        let code = codegen.generate_module_with_info(&info).unwrap();

        println!("Generated wrapper code:\n{}", code);

        assert!(code.contains("pub struct Globals"));
        assert!(code.contains("pub g0: i32"));
        assert!(code.contains("pub struct WasmModule(pub LibraryModule<Globals, 0>)"));
        assert!(code.contains("pub fn new() -> WasmResult<WasmModule>"));
        assert!(code.contains("Globals { g0: 0i32 }"));
        assert!(code.contains("impl WasmModule"));
        assert!(code.contains("pub fn get_value(&mut self) -> WasmResult<i32>"));
        assert!(code.contains("globals.g0"));
    }

    #[test]
    fn generate_module_wrapper_with_memory_and_data() {
        let ir_func = IrFunction {
            params: vec![(VarId(0), WasmType::I32)],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                label: "block0".to_string(),
                instructions: vec![IrInstr::Load {
                    dest: VarId(1),
                    ty: WasmType::I32,
                    addr: VarId(0),
                    offset: 0,
                    width: MemoryAccessWidth::Full,
                    sign: None,
                }],
                terminator: IrTerminator::Return {
                    value: Some(VarId(1)),
                },
            }],
            entry_block: BlockId(0),
            return_type: Some(WasmType::I32),
        };

        let info = ModuleInfo {
            has_memory: true,
            has_memory_import: false,
            max_pages: 1,
            initial_pages: 1,
            table_initial: 0,
            table_max: 0,
            element_segments: Vec::new(),
            globals: Vec::new(),
            data_segments: vec![DataSegmentDef {
                offset: 0,
                data: vec![72, 101, 108, 108, 111], // "Hello"
            }],
            func_exports: vec![FuncExport {
                name: "load_word".to_string(),
                func_index: 0,
            }],
            func_signatures: vec![FuncSignature {
                params: vec![WasmType::I32],
                return_type: Some(WasmType::I32),
                type_idx: 0,
                needs_host: false,
            }],
            type_signatures: Vec::new(),
            canonical_type: Vec::new(),
            func_imports: Vec::new(),
            imported_globals: Vec::new(),
            ir_functions: vec![ir_func],
        };

        let backend = SafeBackend::new();
        let codegen = CodeGenerator::new(&backend);
        let code = codegen.generate_module_with_info(&info).unwrap();

        println!("Generated wrapper code:\n{}", code);

        assert!(code.contains("pub struct WasmModule(pub Module<(), MAX_PAGES, 0>)"));
        assert!(code.contains("pub fn new() -> WasmResult<WasmModule>"));
        assert!(code.contains(
            "Module::try_new(1, (), Table::try_new(0)?).map_err(|_| WasmTrap::OutOfBounds)?"
        ));
        // Data segment init
        assert!(code.contains("module.memory.store_u8(0, 72)?"));
        assert!(code.contains("module.memory.store_u8(4, 111)?"));
        // Export impl
        assert!(code.contains("impl WasmModule"));
        assert!(code.contains("pub fn load_word(&mut self, v0: i32) -> WasmResult<i32>"));
        assert!(code.contains("&mut self.0.memory"));
    }

    #[test]
    fn generate_immutable_global_as_const() {
        let ir_func = IrFunction {
            params: vec![],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                label: "block0".to_string(),
                instructions: vec![IrInstr::GlobalGet {
                    dest: VarId(0),
                    index: 0,
                }],
                terminator: IrTerminator::Return {
                    value: Some(VarId(0)),
                },
            }],
            entry_block: BlockId(0),
            return_type: Some(WasmType::I32),
        };

        let info = ModuleInfo {
            has_memory: false,
            has_memory_import: false,
            max_pages: 0,
            initial_pages: 0,
            table_initial: 0,
            table_max: 0,
            element_segments: Vec::new(),
            globals: vec![GlobalDef {
                wasm_type: WasmType::I32,
                mutable: false,
                init_value: GlobalInit::I32(42),
            }],
            data_segments: Vec::new(),
            func_exports: vec![FuncExport {
                name: "get_const".to_string(),
                func_index: 0,
            }],
            func_signatures: vec![FuncSignature {
                params: vec![],
                return_type: Some(WasmType::I32),
                type_idx: 0,
                needs_host: false,
            }],
            type_signatures: Vec::new(),
            canonical_type: Vec::new(),
            func_imports: Vec::new(),
            imported_globals: Vec::new(),
            ir_functions: vec![ir_func],
        };

        let backend = SafeBackend::new();
        let codegen = CodeGenerator::new(&backend);
        let code = codegen.generate_module_with_info(&info).unwrap();

        println!("Generated code with immutable global:\n{}", code);

        // Should use wrapper mode
        assert!(code.contains("pub const G0: i32 = 42i32;"));
        assert!(code.contains("pub struct WasmModule"));
        assert!(code.contains("pub fn new()"));
        assert!(code.contains("pub fn get_const"));
        // GlobalGet for immutable should use const name
        assert!(code.contains("G0"));
    }
}
