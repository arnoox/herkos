//! herkos — WebAssembly to Rust transpiler.
//!
//! This crate provides the core transpilation pipeline that converts WebAssembly
//! modules into memory-safe Rust source code.

pub mod backend;
pub mod codegen;
pub mod ir;
pub mod parser;

// Re-export key types for convenience
pub use backend::{Backend, SafeBackend};
pub use codegen::{
    CodeGenerator, DataSegmentDef, ElementSegmentDef, FuncExport, FuncImport, FuncSignature,
    GlobalDef, GlobalInit, ModuleInfo,
};
pub use ir::{IrBuilder, IrFunction};
pub use parser::{parse_wasm, ExportKind, ImportKind, ParsedModule};

use anyhow::{Context, Result};

/// Configuration options for transpilation
#[derive(Debug, Clone)]
pub struct TranspileOptions {
    /// Code generation backend mode ("safe", "verified", "hybrid")
    pub mode: String,
    /// Maximum memory pages (used when Wasm module declares no maximum)
    pub max_pages: usize,
}

impl Default for TranspileOptions {
    fn default() -> Self {
        Self {
            mode: "safe".to_string(),
            max_pages: 256,
        }
    }
}

/// Transpile a WebAssembly module to Rust source code.
///
/// This is the main entry point for the transpilation pipeline.
/// It takes raw WASM bytes and returns generated Rust code as a String.
///
/// # Arguments
/// * `wasm_bytes` - Raw WebAssembly binary data
/// * `options` - Transpilation configuration options
///
/// # Returns
/// Generated Rust source code ready to be compiled
///
/// # Example
/// ```no_run
/// use herkos::{transpile, TranspileOptions};
///
/// let wasm_bytes = std::fs::read("input.wasm").unwrap();
/// let options = TranspileOptions::default();
/// let rust_code = transpile(&wasm_bytes, &options).unwrap();
/// std::fs::write("output.rs", rust_code).unwrap();
/// ```
pub fn transpile(wasm_bytes: &[u8], options: &TranspileOptions) -> Result<String> {
    // Step 1: Parse WASM
    let parsed = parse_wasm(wasm_bytes).context("failed to parse WebAssembly module")?;

    // Extract memory information
    let has_memory = parsed.memory.is_some();
    let max_pages = if let Some(ref mem) = parsed.memory {
        mem.maximum_pages.unwrap_or(options.max_pages as u32)
    } else {
        options.max_pages as u32
    };
    let initial_pages = parsed.memory.as_ref().map(|m| m.initial_pages).unwrap_or(0);

    // Extract table information
    let (table_initial, table_max) = if let Some(ref tbl) = parsed.table {
        (tbl.initial_size, tbl.max_size.unwrap_or(tbl.initial_size))
    } else {
        (0, 0)
    };

    // Build canonical type index mapping for structural equivalence.
    // The Wasm spec requires call_indirect type checks to be structural:
    // two different type indices with identical (params, results) must match.
    // We canonicalize by mapping each type_idx to the smallest index with
    // the same structural signature.
    let canonical_type: Vec<u32> = {
        let mut mapping = Vec::with_capacity(parsed.types.len());
        for (i, ty) in parsed.types.iter().enumerate() {
            let canon = parsed.types[..i]
                .iter()
                .position(|earlier| {
                    earlier.params() == ty.params() && earlier.results() == ty.results()
                })
                .map(|pos| mapping[pos])
                .unwrap_or(i as u32);
            mapping.push(canon);
        }
        mapping
    };

    // Build type section signatures for call_indirect resolution
    let type_sigs: Vec<(usize, Option<ir::WasmType>)> = parsed
        .types
        .iter()
        .map(|ty| {
            let param_count = ty.params().len();
            let ret = ty
                .results()
                .first()
                .map(|vt| ir::WasmType::from_wasmparser(*vt));
            (param_count, ret)
        })
        .collect();

    let num_imported_functions = parsed.num_imported_functions;

    // Step 2: Build IR for each function
    let mut ir_builder = IrBuilder::new();
    let mut ir_functions = Vec::new();

    // Build callee signature list for resolving calls.
    // Wasm index space: imported functions at 0..N-1, local functions at N..N+M-1.
    let mut func_sigs: Vec<(usize, Option<ir::WasmType>)> = Vec::new();

    // First: imported function signatures (indices 0..num_imported_functions-1)
    for import in &parsed.imports {
        if let parser::ImportKind::Function(type_idx) = &import.kind {
            let func_type = &parsed.types[*type_idx as usize];
            let param_count = func_type.params().len();
            let ret = func_type
                .results()
                .first()
                .map(|vt| ir::WasmType::from_wasmparser(*vt));
            func_sigs.push((param_count, ret));
        }
    }

    // Then: local function signatures (indices num_imported_functions..)
    for func in &parsed.functions {
        let func_type = &parsed.types[func.type_idx as usize];
        let param_count = func_type.params().len();
        let ret = func_type
            .results()
            .first()
            .map(|vt| ir::WasmType::from_wasmparser(*vt));
        func_sigs.push((param_count, ret));
    }

    // Build function import list (module_name, func_name) for IR builder
    let func_imports: Vec<(String, String)> = parsed
        .imports
        .iter()
        .filter_map(|imp| match &imp.kind {
            parser::ImportKind::Function(_) => Some((imp.module_name.clone(), imp.name.clone())),
            _ => None,
        })
        .collect();

    // Build module context for IR translation
    let module_ctx = ir::ModuleContext {
        func_signatures: func_sigs,
        type_signatures: type_sigs,
        num_imported_functions,
        func_imports,
    };

    for (func_idx, func) in parsed.functions.iter().enumerate() {
        let func_type = &parsed.types[func.type_idx as usize];

        // Convert wasmparser::ValType to our WasmType
        let params: Vec<_> = func_type
            .params()
            .iter()
            .map(|vt| (*vt, ir::WasmType::from_wasmparser(*vt)))
            .collect();

        let return_type = func_type
            .results()
            .first()
            .map(|vt| ir::WasmType::from_wasmparser(*vt));

        // Parse operators from body bytes
        let mut operators = Vec::new();
        let mut binary_reader = wasmparser::BinaryReader::new(&func.body, 0);

        while !binary_reader.eof() {
            let op = binary_reader
                .read_operator()
                .context("failed to read operator")?;
            operators.push(op);
        }

        let ir_func = ir_builder
            .translate_function(&params, &func.locals, return_type, &operators, &module_ctx)
            .with_context(|| format!("failed to build IR for function {}", func_idx))?;

        ir_functions.push(ir_func);
    }

    // Step 3: Build ModuleInfo from parsed data
    let globals: Vec<GlobalDef> = parsed
        .globals
        .iter()
        .map(|g| {
            let wasm_type = ir::WasmType::from_wasmparser(g.val_type);
            let init_value = match g.init_value {
                parser::InitValue::I32(v) => GlobalInit::I32(v),
                parser::InitValue::I64(v) => GlobalInit::I64(v),
                parser::InitValue::F32(v) => GlobalInit::F32(v),
                parser::InitValue::F64(v) => GlobalInit::F64(v),
            };
            GlobalDef {
                wasm_type,
                mutable: g.mutable,
                init_value,
            }
        })
        .collect();

    let data_segments: Vec<DataSegmentDef> = parsed
        .data_segments
        .iter()
        .map(|ds| DataSegmentDef {
            offset: ds.offset,
            data: ds.data.clone(),
        })
        .collect();

    let element_segments: Vec<ElementSegmentDef> = parsed
        .element_segments
        .iter()
        .map(|es| ElementSegmentDef {
            offset: es.offset,
            func_indices: es.func_indices.clone(),
        })
        .collect();

    // Export indices use global numbering (imports + locals). Filter to only
    // local functions (not imported), then offset to local function index space
    // for codegen (func_0, func_1, ...).
    let func_exports: Vec<FuncExport> = parsed
        .exports
        .iter()
        .filter(|e| e.kind == ExportKind::Func && e.index >= num_imported_functions)
        .map(|e| FuncExport {
            name: e.name.clone(),
            func_index: e.index - num_imported_functions,
        })
        .collect();

    // func_signatures: indexed by local function index (0-based, excluding imports).
    // This is used by codegen for call_indirect dispatch and export method generation.
    let func_signatures: Vec<FuncSignature> = parsed
        .functions
        .iter()
        .enumerate()
        .map(|(func_idx, func)| {
            let func_type = &parsed.types[func.type_idx as usize];
            let params = func_type
                .params()
                .iter()
                .map(|vt| ir::WasmType::from_wasmparser(*vt))
                .collect();
            let return_type = func_type
                .results()
                .first()
                .map(|vt| ir::WasmType::from_wasmparser(*vt));

            // Check if this function calls imports (needs host parameter)
            let needs_host = ir_functions
                .get(func_idx)
                .map(|ir_func| {
                    ir_func.blocks.iter().any(|block| {
                        block
                            .instructions
                            .iter()
                            .any(|instr| matches!(instr, ir::IrInstr::CallImport { .. }))
                    })
                })
                .unwrap_or(false);

            FuncSignature {
                params,
                return_type,
                type_idx: canonical_type[func.type_idx as usize],
                needs_host,
            }
        })
        .collect();

    let type_signatures: Vec<FuncSignature> = parsed
        .types
        .iter()
        .map(|ty| {
            let params = ty
                .params()
                .iter()
                .map(|vt| ir::WasmType::from_wasmparser(*vt))
                .collect();
            let return_type = ty
                .results()
                .first()
                .map(|vt| ir::WasmType::from_wasmparser(*vt));
            FuncSignature {
                params,
                return_type,
                type_idx: 0,       // Not meaningful for type_signatures
                needs_host: false, // Type signatures are for call_indirect type checking only
            }
        })
        .collect();

    // Build func_imports for trait generation (Milestone 3)
    let func_imports: Vec<FuncImport> = parsed
        .imports
        .iter()
        .filter_map(|imp| match &imp.kind {
            parser::ImportKind::Function(type_idx) => {
                let func_type = &parsed.types[*type_idx as usize];
                let params = func_type
                    .params()
                    .iter()
                    .map(|vt| ir::WasmType::from_wasmparser(*vt))
                    .collect();
                let return_type = func_type
                    .results()
                    .first()
                    .map(|vt| ir::WasmType::from_wasmparser(*vt));
                Some(FuncImport {
                    module_name: imp.module_name.clone(),
                    func_name: imp.name.clone(),
                    params,
                    return_type,
                })
            }
            _ => None,
        })
        .collect();

    let module_info = ModuleInfo {
        has_memory,
        max_pages,
        initial_pages,
        table_initial,
        table_max,
        element_segments,
        globals,
        data_segments,
        func_exports,
        func_signatures,
        type_signatures,
        canonical_type,
        num_imported_functions,
        func_imports,
    };

    // Step 4: Generate Rust code
    let backend = SafeBackend::new();
    let codegen = CodeGenerator::new(&backend);

    let rust_code = codegen
        .generate_module_with_info(&ir_functions, &module_info)
        .context("failed to generate Rust code")?;

    Ok(rust_code)
}
