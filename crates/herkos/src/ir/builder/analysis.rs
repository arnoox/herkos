//! Module-level analysis - extracts metadata from parsed WebAssembly modules.
//!
//! This module performs structural analysis on a `ParsedModule` to extract
//! memory, table, type, import, and function signature information needed
//! for IR construction and code generation.

use super::super::types::*;
use crate::parser::{ImportKind, ParsedModule};
use crate::TranspileOptions;
use anyhow::{Context, Result};

/// Memory information extracted from the module.
pub(super) struct MemoryInfo {
    pub(super) has_memory: bool,
    pub(super) has_memory_import: bool,
    pub(super) max_pages: usize,
    pub(super) initial_pages: usize,
}

/// Table information extracted from the module.
pub(super) struct TableInfo {
    pub(super) initial: usize,
    pub(super) max: usize,
}

/// Extracts memory information from a parsed WASM module.
pub(super) fn extract_memory_info(
    parsed: &ParsedModule,
    options: &TranspileOptions,
) -> Result<MemoryInfo> {
    let has_memory = parsed.memory.is_some();
    let has_memory_import = parsed
        .imports
        .iter()
        .any(|imp| matches!(imp.kind, ImportKind::Memory { .. }));
    let max_pages = if let Some(ref mem) = parsed.memory {
        mem.maximum_pages
            .map(|p| p as usize)
            .unwrap_or(options.max_pages)
    } else {
        options.max_pages
    };
    let initial_pages = parsed
        .memory
        .as_ref()
        .map(|m| m.initial_pages as usize)
        .unwrap_or(0);

    Ok(MemoryInfo {
        has_memory,
        has_memory_import,
        max_pages,
        initial_pages,
    })
}

/// Extracts table information from a parsed WASM module.
pub(super) fn extract_table_info(parsed: &ParsedModule) -> TableInfo {
    if let Some(ref tbl) = parsed.table {
        TableInfo {
            initial: tbl.initial_size as usize,
            max: (tbl.max_size.unwrap_or(tbl.initial_size) as usize),
        }
    } else {
        TableInfo { initial: 0, max: 0 }
    }
}

/// Builds canonical type index mapping and type signatures.
///
/// Canonical mapping ensures that call_indirect type checks follow the Wasm spec:
/// two different type indices with identical (params, results) must match.
/// We map each type_idx to the smallest index with the same structural signature.
pub(super) fn build_type_mappings(
    parsed: &ParsedModule,
) -> (Vec<usize>, Vec<(usize, Option<WasmType>)>) {
    let canonical_type: Vec<usize> = {
        let mut mapping = Vec::with_capacity(parsed.types.len());
        for (i, ty) in parsed.types.iter().enumerate() {
            let canon = parsed.types[..i]
                .iter()
                .position(|earlier| {
                    earlier.params() == ty.params() && earlier.results() == ty.results()
                })
                .map(|pos| mapping[pos])
                .unwrap_or(i);
            mapping.push(canon);
        }
        mapping
    };

    let type_sigs: Vec<(usize, Option<WasmType>)> = parsed
        .types
        .iter()
        .map(|ty| {
            let param_count = ty.params().len();
            let ret = ty
                .results()
                .first()
                .map(|vt| WasmType::from_wasmparser(*vt));
            (param_count, ret)
        })
        .collect();

    (canonical_type, type_sigs)
}

/// Extracts imported globals from a parsed WASM module.
pub(super) fn build_imported_globals(parsed: &ParsedModule) -> Vec<ImportedGlobalDef> {
    parsed
        .imports
        .iter()
        .filter_map(|imp| {
            if let ImportKind::Global { val_type, mutable } = &imp.kind {
                Some(ImportedGlobalDef {
                    module_name: imp.module_name.clone(),
                    name: imp.name.clone(),
                    wasm_type: WasmType::from_wasmparser(*val_type),
                    mutable: *mutable,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Builds the function signature list (imported functions followed by local functions).
pub(super) fn build_function_signatures(parsed: &ParsedModule) -> Vec<(usize, Option<WasmType>)> {
    let mut func_sigs: Vec<(usize, Option<WasmType>)> = Vec::new();

    // Imported function signatures
    for import in &parsed.imports {
        if let ImportKind::Function(type_idx) = &import.kind {
            let func_type = &parsed.types[*type_idx as usize];
            let param_count = func_type.params().len();
            let ret = func_type
                .results()
                .first()
                .map(|vt| WasmType::from_wasmparser(*vt));
            func_sigs.push((param_count, ret));
        }
    }

    // Local function signatures
    for func in &parsed.functions {
        let func_type = &parsed.types[func.type_idx as usize];
        let param_count = func_type.params().len();
        let ret = func_type
            .results()
            .first()
            .map(|vt| WasmType::from_wasmparser(*vt));
        func_sigs.push((param_count, ret));
    }

    func_sigs
}

/// Parses Wasm operators from a function body.
pub(super) fn parse_function_operators(body: &[u8]) -> Result<Vec<wasmparser::Operator<'_>>> {
    let mut operators = Vec::new();
    let mut binary_reader = wasmparser::BinaryReader::new(body, 0);

    while !binary_reader.eof() {
        let op = binary_reader
            .read_operator()
            .context("failed to read operator")?;
        operators.push(op);
    }

    Ok(operators)
}

/// Translates all functions in the module to intermediate representation.
pub(super) fn build_ir_functions(
    parsed: &ParsedModule,
    type_sigs: &[(usize, Option<WasmType>)],
    num_imported_functions: u32,
) -> Result<Vec<IrFunction>> {
    use super::core::{IrBuilder, ModuleContext};
    use crate::parser::ImportKind;

    let mut ir_builder = IrBuilder::new();
    let mut ir_functions = Vec::new();

    // Build function signature list (imported + local)
    let func_sigs = build_function_signatures(parsed);

    // Build function import list for IR builder
    let func_imports: Vec<(String, String)> = parsed
        .imports
        .iter()
        .filter_map(|imp| match &imp.kind {
            ImportKind::Function(_) => Some((imp.module_name.clone(), imp.name.clone())),
            _ => None,
        })
        .collect();

    let module_ctx = ModuleContext {
        func_signatures: func_sigs,
        type_signatures: type_sigs.to_vec(),
        num_imported_functions: num_imported_functions as usize,
        func_imports,
    };

    for (func_idx, func) in parsed.functions.iter().enumerate() {
        let func_type = &parsed.types[func.type_idx as usize];

        let params: Vec<_> = func_type
            .params()
            .iter()
            .map(|vt| (*vt, WasmType::from_wasmparser(*vt)))
            .collect();

        let return_type = func_type
            .results()
            .first()
            .map(|vt| WasmType::from_wasmparser(*vt));

        let operators = parse_function_operators(&func.body)?;

        let ir_func = ir_builder
            .translate_function(&params, &func.locals, return_type, &operators, &module_ctx)
            .with_context(|| format!("failed to build IR for function {}", func_idx))?;

        ir_functions.push(ir_func);
    }

    Ok(ir_functions)
}
