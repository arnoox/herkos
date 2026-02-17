//! Code generation — emits Rust source code from IR.
//!
//! This module walks the IR and uses a Backend to emit complete Rust functions.
//! It supports two modes:
//! - **Standalone**: generates `pub fn func_N(...)` functions (backwards compatible)
//! - **Module wrapper**: generates a `Module<Globals, MAX_PAGES, 0>` struct with
//!   constructor, internal functions, and exported methods

use crate::backend::Backend;
use crate::ir::*;
use anyhow::{Context, Result};

// ─── ModuleInfo types ───────────────────────────────────────────────────────

/// Definition of a Wasm global variable for code generation.
#[derive(Debug, Clone)]
pub struct GlobalDef {
    /// The Wasm value type of the global.
    pub wasm_type: WasmType,
    /// Whether the global is mutable.
    pub mutable: bool,
    /// The constant initializer value.
    pub init_value: GlobalInit,
}

/// Constant initializer value for a global.
#[derive(Debug, Clone, Copy)]
pub enum GlobalInit {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

/// A data segment to initialize memory.
#[derive(Debug, Clone)]
pub struct DataSegmentDef {
    /// Byte offset into memory.
    pub offset: u32,
    /// Raw bytes to write.
    pub data: Vec<u8>,
}

/// An exported function mapping.
#[derive(Debug, Clone)]
pub struct FuncExport {
    /// The exported name (becomes a Rust method name).
    pub name: String,
    /// Index into the function index space.
    pub func_index: u32,
}

/// Signature of a function (for export method generation).
#[derive(Debug, Clone)]
pub struct FuncSignature {
    /// Parameter types.
    pub params: Vec<WasmType>,
    /// Return type (None for void).
    pub return_type: Option<WasmType>,
    /// Index into the Wasm type section (needed for call_indirect dispatch).
    pub type_idx: u32,
    /// Whether this function calls imported functions (needs host parameter).
    pub needs_host: bool,
}

/// An element segment to initialize a table.
#[derive(Debug, Clone)]
pub struct ElementSegmentDef {
    /// Starting offset in the table.
    pub offset: u32,
    /// Function indices to place into the table starting at `offset`.
    pub func_indices: Vec<u32>,
}

/// An imported function for trait generation.
#[derive(Debug, Clone)]
pub struct FuncImport {
    /// Import module name (e.g., "env").
    pub module_name: String,
    /// Import function name (e.g., "log").
    pub func_name: String,
    /// Parameter types.
    pub params: Vec<WasmType>,
    /// Return type (None for void).
    pub return_type: Option<WasmType>,
}

/// Module-level information for code generation.
///
/// When `needs_wrapper()` returns true, the codegen emits a `Module` struct
/// with constructor, globals, and export methods. Otherwise, it emits
/// standalone `pub fn func_N(...)` functions.
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Whether the module declares linear memory.
    pub has_memory: bool,
    /// Maximum memory pages (from Wasm memory section or default).
    pub max_pages: u32,
    /// Initial memory pages (from Wasm memory section).
    pub initial_pages: u32,
    /// Initial table size (number of entries).
    pub table_initial: u32,
    /// Maximum table size (for const generic TABLE_MAX).
    pub table_max: u32,
    /// Element segments for table initialization.
    pub element_segments: Vec<ElementSegmentDef>,
    /// Global variable definitions (mutable + immutable).
    pub globals: Vec<GlobalDef>,
    /// Data segments for memory initialization.
    pub data_segments: Vec<DataSegmentDef>,
    /// Exported functions.
    pub func_exports: Vec<FuncExport>,
    /// Signatures for all functions (index-aligned with IR functions).
    pub func_signatures: Vec<FuncSignature>,
    /// Type section signatures (for call_indirect dispatch).
    pub type_signatures: Vec<FuncSignature>,
    /// Canonical type index mapping: maps each Wasm type index to the
    /// smallest index with the same structural signature.
    /// Used for spec-compliant structural type equivalence in call_indirect.
    pub canonical_type: Vec<u32>,
    /// Number of imported functions (these occupy indices 0..N-1 in the
    /// function index space, before local functions).
    pub num_imported_functions: u32,
    /// Imported functions for trait generation (Milestone 3).
    pub func_imports: Vec<FuncImport>,
}

impl ModuleInfo {
    /// Whether the module needs a wrapper struct (Module/LibraryModule).
    ///
    /// A wrapper is generated when there are mutable globals, data segments,
    /// or a table (for indirect calls).
    pub fn needs_wrapper(&self) -> bool {
        self.globals.iter().any(|g| g.mutable)
            || !self.data_segments.is_empty()
            || !self.element_segments.is_empty()
    }

    /// Whether the module has any mutable globals.
    pub fn has_mutable_globals(&self) -> bool {
        self.globals.iter().any(|g| g.mutable)
    }

    /// Whether the module has a non-trivial table (for indirect calls).
    pub fn has_table(&self) -> bool {
        self.table_max > 0
    }
}

// ─── Helper functions ───────────────────────────────────────────────────────

/// Convert a module name to a Rust trait name.
///
/// Examples:
/// - "env" → "EnvImports"
/// - "wasi_snapshot_preview1" → "WasiSnapshotPreview1Imports"
fn module_name_to_trait_name(module_name: &str) -> String {
    // Convert to PascalCase and append "Imports"
    let pascal = module_name
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<String>();
    format!("{pascal}Imports")
}

/// Group function imports by module name.
fn group_imports_by_module(
    imports: &[FuncImport],
) -> std::collections::BTreeMap<String, Vec<&FuncImport>> {
    let mut grouped: std::collections::BTreeMap<String, Vec<&FuncImport>> =
        std::collections::BTreeMap::new();
    for imp in imports {
        grouped
            .entry(imp.module_name.clone())
            .or_default()
            .push(imp);
    }
    grouped
}

// ─── CodeGenerator ──────────────────────────────────────────────────────────

/// Generates a complete Rust function from IR.
pub struct CodeGenerator<'a, B: Backend> {
    backend: &'a B,
}

impl<'a, B: Backend> CodeGenerator<'a, B> {
    pub fn new(backend: &'a B) -> Self {
        CodeGenerator { backend }
    }

    /// Generate a complete Rust module from IR functions (legacy API).
    ///
    /// This is the backwards-compatible entry point. It generates standalone
    /// `pub fn func_N(...)` functions without a module wrapper.
    pub fn generate_module(
        &self,
        ir_functions: &[IrFunction],
        has_memory: bool,
        max_pages: u32,
    ) -> Result<String> {
        // Build a minimal ModuleInfo for backwards compatibility
        let info = ModuleInfo {
            has_memory,
            max_pages,
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
            num_imported_functions: 0,
            func_imports: Vec::new(),
        };
        self.generate_module_with_info(ir_functions, &info)
    }

    /// Generate a complete Rust module from IR functions with full module info.
    ///
    /// This is the main entry point for Milestone 4+. It decides between
    /// standalone functions and a module wrapper based on `info.needs_wrapper()`.
    pub fn generate_module_with_info(
        &self,
        ir_functions: &[IrFunction],
        info: &ModuleInfo,
    ) -> Result<String> {
        if info.needs_wrapper() {
            self.generate_wrapper_module(ir_functions, info)
        } else {
            self.generate_standalone_module(ir_functions, info)
        }
    }

    /// Generate host trait definitions from imports (Milestone 3).
    fn generate_host_traits(&self, info: &ModuleInfo) -> String {
        if info.func_imports.is_empty() {
            return String::new();
        }

        let mut code = String::new();
        let grouped = group_imports_by_module(&info.func_imports);

        for (module_name, imports) in grouped {
            let trait_name = module_name_to_trait_name(&module_name);
            code.push_str(&format!("pub trait {trait_name} {{\n"));

            for imp in imports {
                // Generate method signature
                let mut params: Vec<String> = Vec::new();
                params.push("&mut self".to_string());
                for (i, ty) in imp.params.iter().enumerate() {
                    let rust_ty = self.wasm_type_to_rust(ty);
                    params.push(format!("arg{i}: {rust_ty}"));
                }

                let return_ty = match &imp.return_type {
                    Some(ty) => {
                        let rust_ty = self.wasm_type_to_rust(ty);
                        format!("WasmResult<{rust_ty}>")
                    }
                    None => "WasmResult<()>".to_string(),
                };

                code.push_str(&format!(
                    "    fn {}({}) -> {};\n",
                    imp.func_name,
                    params.join(", "),
                    return_ty
                ));
            }

            code.push_str("}\n\n");
        }

        code
    }

    /// Build trait bounds string from imports (e.g., "EnvImports + WasiImports").
    fn build_trait_bounds(&self, info: &ModuleInfo) -> Option<String> {
        if info.func_imports.is_empty() {
            return None;
        }

        let grouped = group_imports_by_module(&info.func_imports);
        let trait_names: Vec<String> = grouped
            .keys()
            .map(|module_name| module_name_to_trait_name(module_name))
            .collect();

        Some(trait_names.join(" + "))
    }

    /// Generate standalone functions (no module wrapper).
    fn generate_standalone_module(
        &self,
        ir_functions: &[IrFunction],
        info: &ModuleInfo,
    ) -> Result<String> {
        let mut rust_code = String::new();

        // Preamble
        rust_code.push_str("// Generated by herkos\n");
        rust_code.push_str("// DO NOT EDIT\n\n");

        // Imports
        rust_code.push_str("#[allow(unused_imports)]\n");
        if info.has_memory {
            rust_code.push_str("use herkos_runtime::{WasmResult, WasmTrap, IsolatedMemory};\n\n");
            rust_code.push_str(&format!("const MAX_PAGES: usize = {};\n\n", info.max_pages));
        } else {
            rust_code.push_str("use herkos_runtime::{WasmResult, WasmTrap};\n\n");
        }

        // Host trait definitions (Milestone 3)
        rust_code.push_str(&self.generate_host_traits(info));

        // Const items for immutable globals (even in standalone mode)
        for (idx, g) in info.globals.iter().enumerate() {
            if !g.mutable {
                let (rust_ty, value_str) = self.global_init_to_rust(&g.init_value, &g.wasm_type);
                rust_code.push_str(&format!("pub const G{idx}: {rust_ty} = {value_str};\n"));
            }
        }
        if info.globals.iter().any(|g| !g.mutable) {
            rust_code.push('\n');
        }

        // Generate each function
        for (idx, ir_func) in ir_functions.iter().enumerate() {
            let func_name = format!("func_{}", idx);
            let code = self
                .generate_function_with_info(ir_func, &func_name, info, true)
                .with_context(|| format!("failed to generate code for function {}", idx))?;
            rust_code.push_str(&code);
            rust_code.push('\n');
        }

        Ok(rust_code)
    }

    /// Generate a module wrapper with Globals struct, constructor, and export methods.
    fn generate_wrapper_module(
        &self,
        ir_functions: &[IrFunction],
        info: &ModuleInfo,
    ) -> Result<String> {
        let mut rust_code = String::new();
        let has_mut_globals = info.has_mutable_globals();

        // Preamble
        rust_code.push_str("// Generated by herkos\n");
        rust_code.push_str("// DO NOT EDIT\n\n");

        // Imports
        rust_code.push_str("#[allow(unused_imports)]\n");
        let funcref_import = if !info.element_segments.is_empty() {
            ", FuncRef"
        } else {
            ""
        };
        if info.has_memory {
            rust_code.push_str(&format!(
                "use herkos_runtime::{{WasmResult, WasmTrap, IsolatedMemory, Module, Table{funcref_import}}};\n\n",
            ));
            rust_code.push_str(&format!("const MAX_PAGES: usize = {};\n", info.max_pages));
        } else {
            rust_code.push_str(&format!(
                "use herkos_runtime::{{WasmResult, WasmTrap, LibraryModule, Table{funcref_import}}};\n\n",
            ));
        }
        if info.has_table() {
            rust_code.push_str(&format!("const TABLE_MAX: usize = {};\n", info.table_max));
        }
        rust_code.push('\n');

        // Host trait definitions (Milestone 3)
        rust_code.push_str(&self.generate_host_traits(info));

        // Globals struct (mutable globals only)
        if has_mut_globals {
            rust_code.push_str("pub struct Globals {\n");
            for (idx, g) in info.globals.iter().enumerate() {
                if g.mutable {
                    let rust_ty = self.wasm_type_to_rust(&g.wasm_type);
                    rust_code.push_str(&format!("    pub g{idx}: {rust_ty},\n"));
                }
            }
            rust_code.push_str("}\n\n");
        }

        // Const items for immutable globals
        for (idx, g) in info.globals.iter().enumerate() {
            if !g.mutable {
                let (rust_ty, value_str) = self.global_init_to_rust(&g.init_value, &g.wasm_type);
                rust_code.push_str(&format!("pub const G{idx}: {rust_ty} = {value_str};\n"));
            }
        }
        if info.globals.iter().any(|g| !g.mutable) {
            rust_code.push('\n');
        }

        // Newtype wrapper struct (required to allow `impl WasmModule` on a foreign type)
        let globals_type = if has_mut_globals { "Globals" } else { "()" };
        let table_size_str = if info.has_table() { "TABLE_MAX" } else { "0" };
        if info.has_memory {
            rust_code.push_str(&format!(
                "pub struct WasmModule(pub Module<{globals_type}, MAX_PAGES, {table_size_str}>);\n\n"
            ));
        } else {
            rust_code.push_str(&format!(
                "pub struct WasmModule(pub LibraryModule<{globals_type}, {table_size_str}>);\n\n"
            ));
        }

        // Constructor (standalone for backwards compatibility)
        rust_code.push_str(&self.generate_constructor(info, has_mut_globals));
        rust_code.push('\n');

        // Internal functions (private)
        for (idx, ir_func) in ir_functions.iter().enumerate() {
            let func_name = format!("func_{}", idx);
            let code = self
                .generate_function_with_info(ir_func, &func_name, info, false)
                .with_context(|| format!("failed to generate code for function {}", idx))?;
            rust_code.push_str(&code);
            rust_code.push('\n');
        }

        // Export impl block
        if !info.func_exports.is_empty() {
            rust_code.push_str(&self.generate_export_impl(info));
            rust_code.push('\n');
        }

        Ok(rust_code)
    }

    /// Generate the `pub fn new() -> WasmModule` or `pub fn new() -> WasmResult<WasmModule>` constructor.
    fn generate_constructor(&self, info: &ModuleInfo, has_mut_globals: bool) -> String {
        let mut code = String::new();

        // Simple constructor for modules with no initialization
        if !info.has_memory
            && !has_mut_globals
            && info.data_segments.is_empty()
            && info.element_segments.is_empty()
        {
            code.push_str(
                "pub fn new() -> Result<WasmModule, herkos_runtime::ConstructionError> {\n",
            );
            code.push_str("    Ok(WasmModule(LibraryModule::new((), Table::try_new(0)?)))\n");
            code.push_str("}\n");
            return code;
        }

        code.push_str("pub fn new() -> WasmResult<WasmModule> {\n");

        // Build globals initializer
        let globals_init = if has_mut_globals {
            let mut fields = String::from("Globals { ");
            let mut first = true;
            for (idx, g) in info.globals.iter().enumerate() {
                if g.mutable {
                    if !first {
                        fields.push_str(", ");
                    }
                    let (_, value_str) = self.global_init_to_rust(&g.init_value, &g.wasm_type);
                    fields.push_str(&format!("g{idx}: {value_str}"));
                    first = false;
                }
            }
            fields.push_str(" }");
            fields
        } else {
            "()".to_string()
        };

        // Table initialization
        let table_init = if info.has_table() {
            format!("Table::try_new({})?", info.table_initial)
        } else {
            "Table::try_new(0)?".to_string()
        };

        if info.has_memory {
            let needs_mut = !info.data_segments.is_empty() || !info.element_segments.is_empty();
            let binding = if needs_mut {
                "let mut module"
            } else {
                "let module"
            };
            code.push_str(&format!(
                "    {} = Module::try_new({}, {}, {}).map_err(|_| WasmTrap::OutOfBounds)?;\n",
                binding, info.initial_pages, globals_init, table_init
            ));

            // Data segment initialization (byte-by-byte)
            for seg in &info.data_segments {
                for (i, byte) in seg.data.iter().enumerate() {
                    let addr = seg.offset as usize + i;
                    code.push_str(&format!(
                        "    module.memory.store_u8({}, {})?;\n",
                        addr, byte
                    ));
                }
            }

            // Element segment initialization
            for seg in &info.element_segments {
                for (i, func_idx) in seg.func_indices.iter().enumerate() {
                    let table_idx = seg.offset as usize + i;
                    // func_idx is in global space (imports + locals).
                    // Convert to local function index for lookup and storage.
                    let local_func_idx = *func_idx - info.num_imported_functions;
                    let type_idx = info
                        .func_signatures
                        .get(local_func_idx as usize)
                        .map(|s| s.type_idx)
                        .unwrap_or(0);
                    code.push_str(&format!(
                        "    module.table.set({}, Some(FuncRef {{ type_index: {}, func_index: {} }})).unwrap();\n",
                        table_idx, type_idx, local_func_idx
                    ));
                }
            }

            code.push_str("    Ok(WasmModule(module))\n");
        } else if !info.element_segments.is_empty() {
            // Need mutable table for element initialization
            code.push_str(&format!("    let mut table = {};\n", table_init));
            for seg in &info.element_segments {
                for (i, func_idx) in seg.func_indices.iter().enumerate() {
                    let table_idx = seg.offset as usize + i;
                    // func_idx is in global space (imports + locals).
                    // Convert to local function index for lookup and storage.
                    let local_func_idx = *func_idx - info.num_imported_functions;
                    let type_idx = info
                        .func_signatures
                        .get(local_func_idx as usize)
                        .map(|s| s.type_idx)
                        .unwrap_or(0);
                    code.push_str(&format!(
                        "    table.set({}, Some(FuncRef {{ type_index: {}, func_index: {} }})).unwrap();\n",
                        table_idx, type_idx, local_func_idx
                    ));
                }
            }
            code.push_str(&format!(
                "    Ok(WasmModule(LibraryModule::new({}, table)))\n",
                globals_init
            ));
        } else {
            code.push_str(&format!(
                "    Ok(WasmModule(LibraryModule::new({}, {})))\n",
                globals_init, table_init
            ));
        }

        code.push_str("}\n");
        code
    }

    /// Generate the `impl WasmModule { ... }` block with export methods.
    fn generate_export_impl(&self, info: &ModuleInfo) -> String {
        let mut code = String::new();
        let has_mut_globals = info.has_mutable_globals();

        code.push_str("impl WasmModule {\n");

        for export in &info.func_exports {
            let func_idx = export.func_index as usize;
            let sig = &info.func_signatures[func_idx];

            // Determine trait bounds for this export
            let trait_bounds_opt = if sig.needs_host {
                self.build_trait_bounds(info)
            } else {
                None
            };
            let has_multiple_bounds = trait_bounds_opt.as_ref().is_some_and(|b| b.contains(" + "));

            // Method signature with optional generic parameter
            let mut param_parts: Vec<String> = Vec::new();
            param_parts.push("&mut self".to_string());
            for (i, ty) in sig.params.iter().enumerate() {
                let rust_ty = self.wasm_type_to_rust(ty);
                param_parts.push(format!("v{i}: {rust_ty}"));
            }

            // Add host parameter if function needs it (Milestone 2/3)
            if sig.needs_host {
                if let Some(trait_bounds) = &trait_bounds_opt {
                    if has_multiple_bounds {
                        // Use generic parameter H
                        param_parts.push("host: &mut H".to_string());
                    } else {
                        // Single trait bound - use impl directly
                        param_parts.push(format!("host: &mut impl {trait_bounds}"));
                    }
                } else {
                    // Fallback for backwards compatibility
                    param_parts.push("host: &mut impl Host".to_string());
                }
            }

            let return_type = match &sig.return_type {
                Some(ty) => {
                    let rust_ty = self.wasm_type_to_rust(ty);
                    format!("WasmResult<{rust_ty}>")
                }
                None => "WasmResult<()>".to_string(),
            };

            // Generate method signature (with generic if needed)
            if has_multiple_bounds {
                code.push_str(&format!(
                    "    pub fn {}<H: {}>({}) -> {} {{\n",
                    export.name,
                    trait_bounds_opt.as_ref().unwrap(),
                    param_parts.join(", "),
                    return_type
                ));
            } else {
                code.push_str(&format!(
                    "    pub fn {}({}) -> {} {{\n",
                    export.name,
                    param_parts.join(", "),
                    return_type
                ));
            }

            // Forward call to internal function
            let mut call_args: Vec<String> =
                (0..sig.params.len()).map(|i| format!("v{i}")).collect();

            // Forward host parameter if needed
            if sig.needs_host {
                call_args.push("host".to_string());
            }

            if has_mut_globals {
                call_args.push("&mut self.0.globals".to_string());
            }
            if info.has_memory {
                call_args.push("&mut self.0.memory".to_string());
            }
            if info.has_table() {
                call_args.push("&self.0.table".to_string());
            }

            code.push_str(&format!(
                "        func_{}({})\n",
                export.func_index,
                call_args.join(", ")
            ));
            code.push_str("    }\n");
        }

        code.push_str("}\n");
        code
    }

    /// Generate a complete Rust function from IR (legacy API).
    pub fn generate_function(
        &self,
        ir_func: &IrFunction,
        func_name: &str,
        has_memory: bool,
    ) -> Result<String> {
        let info = ModuleInfo {
            has_memory,
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
            num_imported_functions: 0,
            func_imports: Vec::new(),
        };
        self.generate_function_with_info(ir_func, func_name, &info, true)
    }

    /// Check if an IR function contains any CallImport instructions.
    fn has_import_calls(ir_func: &IrFunction) -> bool {
        ir_func.blocks.iter().any(|block| {
            block
                .instructions
                .iter()
                .any(|instr| matches!(instr, IrInstr::CallImport { .. }))
        })
    }

    /// Generate a complete Rust function from IR with module info.
    ///
    /// `is_public` controls whether the function is `pub fn` or `fn`.
    fn generate_function_with_info(
        &self,
        ir_func: &IrFunction,
        func_name: &str,
        info: &ModuleInfo,
        is_public: bool,
    ) -> Result<String> {
        let mut output = String::new();

        // Suppress warnings for generated code patterns that are hard to avoid
        output.push_str("#[allow(unused_mut, unused_variables, unused_assignments, clippy::needless_return, clippy::manual_range_contains, clippy::never_loop)]\n");

        // Generate function signature
        output.push_str(&self.generate_signature_with_info(ir_func, func_name, info, is_public));
        output.push_str(" {\n");

        // Create mapping from BlockId to vector index
        let mut block_id_to_index = std::collections::HashMap::new();
        for (idx, block) in ir_func.blocks.iter().enumerate() {
            block_id_to_index.insert(block.id, idx);
        }

        // Collect all variables and their types from instructions.
        let mut var_types: std::collections::HashMap<VarId, WasmType> =
            std::collections::HashMap::new();

        // Seed with parameter types
        for (var, ty) in &ir_func.params {
            var_types.insert(*var, *ty);
        }

        // Infer types from instructions
        for block in &ir_func.blocks {
            for instr in &block.instructions {
                match instr {
                    IrInstr::Const { dest, value } => {
                        var_types.insert(*dest, value.wasm_type());
                    }
                    IrInstr::BinOp { dest, op, .. } => {
                        var_types.insert(*dest, op.result_type());
                    }
                    IrInstr::UnOp { dest, op, .. } => {
                        var_types.insert(*dest, op.result_type());
                    }
                    IrInstr::Load { dest, ty, .. } => {
                        var_types.insert(*dest, *ty);
                    }
                    IrInstr::Call {
                        dest: Some(dest),
                        func_idx,
                        ..
                    } => {
                        // func_idx is in global space (imports + locals).
                        // For Milestone 1, we error on imported functions during codegen,
                        // so just use a fallback type here if it's an import.
                        let ty = if *func_idx >= info.num_imported_functions {
                            let local_idx = func_idx - info.num_imported_functions;
                            info.func_signatures
                                .get(local_idx as usize)
                                .and_then(|s| s.return_type)
                                .unwrap_or(WasmType::I32)
                        } else {
                            // Call to imported function — will error during codegen.
                            // Use fallback type for now.
                            WasmType::I32
                        };
                        var_types.insert(*dest, ty);
                    }
                    IrInstr::CallImport {
                        dest: Some(dest),
                        import_idx,
                        ..
                    } => {
                        // Look up import signature from func_imports
                        let ty = info
                            .func_imports
                            .get(*import_idx as usize)
                            .and_then(|imp| imp.return_type)
                            .unwrap_or(WasmType::I32);
                        var_types.insert(*dest, ty);
                    }
                    IrInstr::Assign { dest, src } => {
                        if let Some(ty) = var_types.get(src) {
                            var_types.insert(*dest, *ty);
                        } else {
                            var_types.insert(*dest, WasmType::I32);
                        }
                    }
                    IrInstr::GlobalGet { dest, index } => {
                        // Look up actual type from ModuleInfo
                        let ty = if (*index as usize) < info.globals.len() {
                            info.globals[*index as usize].wasm_type
                        } else {
                            WasmType::I32 // fallback
                        };
                        var_types.insert(*dest, ty);
                    }
                    IrInstr::CallIndirect {
                        dest: Some(dest),
                        type_idx,
                        ..
                    } => {
                        let ty = info
                            .type_signatures
                            .get(*type_idx as usize)
                            .and_then(|s| s.return_type)
                            .unwrap_or(WasmType::I32);
                        var_types.insert(*dest, ty);
                    }
                    IrInstr::MemorySize { dest } | IrInstr::MemoryGrow { dest, .. } => {
                        var_types.insert(*dest, WasmType::I32);
                    }
                    IrInstr::Select { dest, val1, .. } => {
                        // Result type matches the operand type
                        let ty = var_types.get(val1).copied().unwrap_or(WasmType::I32);
                        var_types.insert(*dest, ty);
                    }
                    _ => {}
                }
            }

            // Also scan terminators for variable references (needed for
            // dead-code blocks after `unreachable` where the variable
            // was never assigned by an instruction).
            match &block.terminator {
                IrTerminator::Return { value: Some(var) } => {
                    var_types
                        .entry(*var)
                        .or_insert(ir_func.return_type.unwrap_or(WasmType::I32));
                }
                IrTerminator::BranchIf { condition, .. } => {
                    var_types.entry(*condition).or_insert(WasmType::I32);
                }
                IrTerminator::BranchTable { index, .. } => {
                    var_types.entry(*index).or_insert(WasmType::I32);
                }
                _ => {}
            }
        }

        // Declare all SSA variables with their inferred types
        let mut sorted_vars: Vec<_> = var_types
            .iter()
            .filter(|(var, _)| !ir_func.params.iter().any(|(p, _)| p == *var))
            .collect();
        sorted_vars.sort_by_key(|(var, _)| var.0);

        for (var, ty) in sorted_vars {
            let rust_ty = self.wasm_type_to_rust(ty);
            let default = match ty {
                WasmType::I32 => "0i32",
                WasmType::I64 => "0i64",
                WasmType::F32 => "0.0f32",
                WasmType::F64 => "0.0f64",
            };
            output.push_str(&format!("    let mut {var}: {rust_ty} = {default};\n"));
        }

        if ir_func.blocks.len() == 1 {
            // Single-block optimization: emit flat body without loop/match
            let block = &ir_func.blocks[0];
            for instr in &block.instructions {
                let code = self.generate_instruction_with_info(instr, info);
                output.push_str(&code);
                output.push('\n');
            }
            let term_code = self.generate_terminator_with_mapping(
                &block.terminator,
                &block_id_to_index,
                ir_func.return_type,
            );
            output.push_str(&term_code);
            output.push('\n');
        } else {
            // Multi-block: state machine with per-function Block enum
            output.push_str("    #[derive(Clone, Copy)]\n    #[allow(dead_code)]\n");
            output.push_str("    enum Block { ");
            for idx in 0..ir_func.blocks.len() {
                if idx > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("B{}", idx));
            }
            output.push_str(" }\n");
            output.push_str("    let mut __current_block = Block::B0;\n");
            output.push_str("    loop {\n");
            output.push_str("        match __current_block {\n");

            for (idx, block) in ir_func.blocks.iter().enumerate() {
                output.push_str(&format!("            Block::B{} => {{\n", idx));

                for instr in &block.instructions {
                    let code = self.generate_instruction_with_info(instr, info);
                    output.push_str(&code);
                    output.push('\n');
                }

                let term_code = self.generate_terminator_with_mapping(
                    &block.terminator,
                    &block_id_to_index,
                    ir_func.return_type,
                );
                output.push_str(&term_code);
                output.push('\n');

                output.push_str("            }\n");
            }

            // No catch-all needed — match is exhaustive over Block enum
            output.push_str("        }\n");
            output.push_str("    }\n");
        }

        output.push_str("}\n");
        Ok(output)
    }

    /// Generate function signature with module info.
    fn generate_signature_with_info(
        &self,
        ir_func: &IrFunction,
        func_name: &str,
        info: &ModuleInfo,
        is_public: bool,
    ) -> String {
        let visibility = if is_public { "pub " } else { "" };

        // Add generic type parameter if function needs host with multiple trait bounds
        let needs_host = Self::has_import_calls(ir_func);
        let trait_bounds_opt = if needs_host {
            self.build_trait_bounds(info)
        } else {
            None
        };

        let has_multiple_bounds = trait_bounds_opt.as_ref().is_some_and(|b| b.contains(" + "));

        let mut sig = if has_multiple_bounds {
            format!(
                "{visibility}fn {func_name}<H: {}>(",
                trait_bounds_opt.as_ref().unwrap()
            )
        } else {
            format!("{visibility}fn {func_name}(")
        };

        // Parameters (mutable, as in WebAssembly all locals are mutable)
        let mut param_parts: Vec<String> = ir_func
            .params
            .iter()
            .map(|(var_id, ty)| {
                let rust_ty = self.wasm_type_to_rust(ty);
                format!("mut {}: {}", var_id, rust_ty)
            })
            .collect();

        // Add host parameter if function calls imports (Milestone 2/3)
        if needs_host {
            if let Some(trait_bounds) = trait_bounds_opt {
                if has_multiple_bounds {
                    // Use generic parameter H
                    param_parts.push("host: &mut H".to_string());
                } else {
                    // Single trait bound - use impl directly
                    param_parts.push(format!("host: &mut impl {trait_bounds}"));
                }
            } else {
                // Fallback for backwards compatibility
                param_parts.push("host: &mut impl Host".to_string());
            }
        }

        // Add globals parameter if wrapper mode has mutable globals
        if info.needs_wrapper() && info.has_mutable_globals() {
            param_parts.push("globals: &mut Globals".to_string());
        }

        // Add memory parameter if module has memory
        if info.has_memory {
            param_parts.push("memory: &mut IsolatedMemory<MAX_PAGES>".to_string());
        }

        // Add table parameter if module has a table
        if info.has_table() {
            param_parts.push("table: &Table<TABLE_MAX>".to_string());
        }

        sig.push_str(&param_parts.join(", "));
        sig.push(')');

        // Return type
        match &ir_func.return_type {
            Some(ty) => {
                let rust_ty = self.wasm_type_to_rust(ty);
                sig.push_str(&format!(" -> WasmResult<{rust_ty}>"));
            }
            None => {
                sig.push_str(" -> WasmResult<()>");
            }
        }

        sig
    }

    /// Generate code for a single instruction with module info.
    fn generate_instruction_with_info(&self, instr: &IrInstr, info: &ModuleInfo) -> String {
        match instr {
            IrInstr::Const { dest, value } => self.backend.emit_const(*dest, value),

            IrInstr::BinOp { dest, op, lhs, rhs } => {
                self.backend.emit_binop(*dest, *op, *lhs, *rhs)
            }

            IrInstr::UnOp { dest, op, operand } => self.backend.emit_unop(*dest, *op, *operand),

            IrInstr::Load {
                dest,
                ty,
                addr,
                offset,
                width,
                sign,
            } => self
                .backend
                .emit_load(*dest, *ty, *addr, *offset, *width, *sign),

            IrInstr::Store {
                ty,
                addr,
                value,
                offset,
                width,
            } => self.backend.emit_store(*ty, *addr, *value, *offset, *width),

            IrInstr::Call {
                dest,
                func_idx,
                args,
            } => {
                // Call to local function (imports are handled by CallImport)
                let has_globals = info.needs_wrapper() && info.has_mutable_globals();
                let has_memory = info.has_memory;
                let has_table = info.has_table();
                self.backend
                    .emit_call(*dest, *func_idx, args, has_globals, has_memory, has_table)
            }

            IrInstr::CallImport {
                dest,
                module_name,
                func_name,
                args,
                ..
            } => self
                .backend
                .emit_call_import(*dest, module_name, func_name, args),

            IrInstr::CallIndirect {
                dest,
                type_idx,
                table_idx,
                args,
            } => self.generate_call_indirect(*dest, *type_idx, *table_idx, args, info),

            IrInstr::Assign { dest, src } => self.backend.emit_assign(*dest, *src),

            IrInstr::GlobalGet { dest, index } => {
                let is_mutable = if (*index as usize) < info.globals.len() {
                    info.globals[*index as usize].mutable
                } else {
                    true // fallback: assume mutable
                };
                self.backend.emit_global_get(*dest, *index, is_mutable)
            }

            IrInstr::GlobalSet { index, value } => self.backend.emit_global_set(*index, *value),

            IrInstr::MemorySize { dest } => self.backend.emit_memory_size(*dest),

            IrInstr::MemoryGrow { dest, delta } => self.backend.emit_memory_grow(*dest, *delta),

            IrInstr::Select {
                dest,
                val1,
                val2,
                condition,
            } => self.backend.emit_select(*dest, *val1, *val2, *condition),
        }
    }

    /// Generate code for a terminator with BlockId to index mapping.
    fn generate_terminator_with_mapping(
        &self,
        term: &IrTerminator,
        block_id_to_index: &std::collections::HashMap<BlockId, usize>,
        func_return_type: Option<WasmType>,
    ) -> String {
        match term {
            IrTerminator::Return { value } => {
                // If the function has a return type but the return has no value,
                // this is dead code after `unreachable` — emit a trap instead
                // of `return Ok(())` which would be a type mismatch.
                if value.is_none() && func_return_type.is_some() {
                    return self.backend.emit_unreachable();
                }
                self.backend.emit_return(*value)
            }

            IrTerminator::Jump { target } => {
                let idx = block_id_to_index[target];
                self.backend.emit_jump_to_index(idx)
            }

            IrTerminator::BranchIf {
                condition,
                if_true,
                if_false,
            } => {
                let true_idx = block_id_to_index[if_true];
                let false_idx = block_id_to_index[if_false];
                self.backend
                    .emit_branch_if_to_index(*condition, true_idx, false_idx)
            }

            IrTerminator::BranchTable {
                index,
                targets,
                default,
            } => {
                let target_indices: Vec<usize> =
                    targets.iter().map(|t| block_id_to_index[t]).collect();
                let default_idx = block_id_to_index[default];
                self.backend
                    .emit_branch_table_to_index(*index, &target_indices, default_idx)
            }

            IrTerminator::Unreachable => self.backend.emit_unreachable(),
        }
    }

    /// Convert WasmType to Rust type string.
    fn wasm_type_to_rust(&self, ty: &WasmType) -> &'static str {
        match ty {
            WasmType::I32 => "i32",
            WasmType::I64 => "i64",
            WasmType::F32 => "f32",
            WasmType::F64 => "f64",
        }
    }

    /// Convert a GlobalInit to (Rust type string, value literal string).
    fn global_init_to_rust(&self, init: &GlobalInit, ty: &WasmType) -> (&'static str, String) {
        let rust_ty = self.wasm_type_to_rust(ty);
        let value = match init {
            GlobalInit::I32(v) => format!("{v}i32"),
            GlobalInit::I64(v) => format!("{v}i64"),
            GlobalInit::F32(v) => format!("{v}f32"),
            GlobalInit::F64(v) => format!("{v}f64"),
        };
        (rust_ty, value)
    }

    /// Generate inline dispatch code for `call_indirect`.
    ///
    /// The generated code:
    /// 1. Looks up the table entry by index
    /// 2. Checks the type signature matches
    /// 3. Dispatches to the matching function via a match on func_index
    fn generate_call_indirect(
        &self,
        dest: Option<VarId>,
        type_idx: u32,
        table_idx: VarId,
        args: &[VarId],
        info: &ModuleInfo,
    ) -> String {
        let has_globals = info.needs_wrapper() && info.has_mutable_globals();
        let has_memory = info.has_memory;
        let has_table = info.has_table();

        // Canonicalize the type index for structural equivalence (Wasm spec §4.4.9).
        // Two different type indices with identical (params, results) must match.
        let canon_idx = info
            .canonical_type
            .get(type_idx as usize)
            .copied()
            .unwrap_or(type_idx);

        let mut code = String::new();

        // Look up the table entry
        code.push_str(&format!(
            "                let __entry = table.get({table_idx} as u32)?;\n"
        ));

        // Type check (compares canonical indices — FuncRef.type_index is
        // also stored as canonical during element segment initialization)
        code.push_str(&format!(
            "                if __entry.type_index != {canon_idx} {{ return Err(WasmTrap::IndirectCallTypeMismatch); }}\n"
        ));

        // Build the common args string for dispatch calls
        let mut call_args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        if has_globals {
            call_args.push("globals".to_string());
        }
        if has_memory {
            call_args.push("memory".to_string());
        }
        if has_table {
            call_args.push("table".to_string());
        }
        let args_str = call_args.join(", ");

        // Build dispatch match — only dispatch to functions with matching
        // canonical type (structural equivalence)
        let dest_prefix = match dest {
            Some(d) => format!("{d} = "),
            None => String::new(),
        };

        code.push_str(&format!(
            "                {dest_prefix}match __entry.func_index {{\n"
        ));

        for (func_idx, sig) in info.func_signatures.iter().enumerate() {
            if sig.type_idx == canon_idx {
                code.push_str(&format!(
                    "                    {} => func_{}({})?,\n",
                    func_idx, func_idx, args_str
                ));
            }
        }

        code.push_str("                    _ => return Err(WasmTrap::UndefinedElement),\n");
        code.push_str("                };");

        code
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
        let codegen = CodeGenerator::new(&backend);
        let code = codegen.generate_function(&ir_func, "add", false).unwrap();

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
        let codegen = CodeGenerator::new(&backend);
        let code = codegen.generate_function(&ir_func, "noop", false).unwrap();

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
        let codegen = CodeGenerator::new(&backend);
        let code = codegen.generate_function(&ir_func, "add64", false).unwrap();

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
        let codegen = CodeGenerator::new(&backend);
        let code = codegen.generate_function(&ir_func, "eq64", false).unwrap();

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
            num_imported_functions: 0,
            func_imports: Vec::new(),
        };

        let backend = SafeBackend::new();
        let codegen = CodeGenerator::new(&backend);
        let code = codegen
            .generate_module_with_info(&[ir_func], &info)
            .unwrap();

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
            num_imported_functions: 0,
            func_imports: Vec::new(),
        };

        let backend = SafeBackend::new();
        let codegen = CodeGenerator::new(&backend);
        let code = codegen
            .generate_module_with_info(&[ir_func], &info)
            .unwrap();

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
            func_exports: Vec::new(),
            func_signatures: vec![FuncSignature {
                params: vec![],
                return_type: Some(WasmType::I32),
                type_idx: 0,
                needs_host: false,
            }],
            type_signatures: Vec::new(),
            canonical_type: Vec::new(),
            num_imported_functions: 0,
            func_imports: Vec::new(),
        };

        let backend = SafeBackend::new();
        let codegen = CodeGenerator::new(&backend);
        let code = codegen
            .generate_module_with_info(&[ir_func], &info)
            .unwrap();

        println!("Generated standalone code:\n{}", code);

        // Immutable only → standalone (no wrapper needed)
        assert!(!info.needs_wrapper());
        assert!(code.contains("pub const G0: i32 = 42i32;"));
        assert!(code.contains("pub fn func_0"));
        // GlobalGet for immutable should use const name
        assert!(code.contains("G0"));
    }

    #[test]
    fn generate_backwards_compat_no_wrapper() {
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

        let info = ModuleInfo {
            has_memory: false,
            max_pages: 0,
            initial_pages: 0,
            table_initial: 0,
            table_max: 0,
            element_segments: Vec::new(),
            globals: Vec::new(),
            data_segments: Vec::new(),
            func_exports: vec![FuncExport {
                name: "add".to_string(),
                func_index: 0,
            }],
            func_signatures: vec![FuncSignature {
                params: vec![WasmType::I32, WasmType::I32],
                return_type: Some(WasmType::I32),
                type_idx: 0,
                needs_host: false,
            }],
            type_signatures: Vec::new(),
            canonical_type: Vec::new(),
            num_imported_functions: 0,
            func_imports: Vec::new(),
        };

        let backend = SafeBackend::new();
        let codegen = CodeGenerator::new(&backend);
        let code = codegen
            .generate_module_with_info(&[ir_func], &info)
            .unwrap();

        println!("Generated backwards compat code:\n{}", code);

        // No wrapper — standalone functions
        assert!(!info.needs_wrapper());
        assert!(!code.contains("pub struct Globals"));
        assert!(!code.contains("WasmModule"));
        assert!(!code.contains("impl"));
        assert!(code.contains("pub fn func_0("));
    }
}
