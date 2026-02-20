//! Metadata assembly - builds the final ModuleInfo from analyzed pieces.
//!
//! This module takes the results of module analysis (extracted memory, table,
//! type, and function information) and assembles them into the final `ModuleInfo`
//! structure that is passed to code generation.

use super::super::types::*;
use super::analysis::{MemoryInfo, TableInfo};
use crate::parser::{ExportKind, ImportKind, ParsedModule};
use anyhow::Result;

/// Assembles module metadata for code generation.
#[allow(clippy::too_many_arguments)]
pub(super) fn assemble_module_metadata(
    parsed: &ParsedModule,
    mem_info: &MemoryInfo,
    table_info: &TableInfo,
    canonical_type: &[usize],
    mut ir_functions: Vec<IrFunction>,
    num_imported_functions: usize,
    imported_globals: &[ImportedGlobalDef],
) -> Result<ModuleInfo> {
    let globals = build_globals(parsed);
    let data_segments = build_data_segments(parsed);
    let element_segments = build_element_segments(parsed);
    let func_exports = build_function_exports(parsed, num_imported_functions);
    let type_signatures = build_call_indirect_signatures(parsed);
    let func_imports = build_function_imports(parsed);

    // Enrich IR functions with signature metadata (type_idx and needs_host)
    enrich_ir_functions(parsed, canonical_type, &mut ir_functions, imported_globals);

    Ok(ModuleInfo {
        has_memory: mem_info.has_memory,
        has_memory_import: mem_info.has_memory_import,
        max_pages: mem_info.max_pages,
        initial_pages: mem_info.initial_pages,
        table_initial: table_info.initial,
        table_max: table_info.max,
        element_segments,
        globals,
        data_segments,
        func_exports,
        type_signatures,
        canonical_type: canonical_type.to_vec(),
        func_imports,
        imported_globals: imported_globals.to_vec(),
        ir_functions,
    })
}

/// Builds global variable definitions.
fn build_globals(parsed: &ParsedModule) -> Vec<GlobalDef> {
    parsed
        .globals
        .iter()
        .map(|g| {
            let init_value = match g.init_value {
                crate::parser::InitValue::I32(v) => GlobalInit::I32(v),
                crate::parser::InitValue::I64(v) => GlobalInit::I64(v),
                crate::parser::InitValue::F32(v) => GlobalInit::F32(v),
                crate::parser::InitValue::F64(v) => GlobalInit::F64(v),
            };
            GlobalDef {
                mutable: g.mutable,
                init_value,
            }
        })
        .collect()
}

/// Builds data segment definitions.
fn build_data_segments(parsed: &ParsedModule) -> Vec<DataSegmentDef> {
    parsed
        .data_segments
        .iter()
        .map(|ds| DataSegmentDef {
            offset: ds.offset,
            data: ds.data.clone(),
        })
        .collect()
}

/// Builds element segment (table initialization) definitions.
fn build_element_segments(parsed: &ParsedModule) -> Vec<ElementSegmentDef> {
    parsed
        .element_segments
        .iter()
        .map(|es| ElementSegmentDef {
            offset: es.offset as usize,
            func_indices: es
                .func_indices
                .iter()
                .map(|idx| GlobalFuncIdx::new(*idx as usize))
                .collect(),
        })
        .collect()
}

/// Builds exported function definitions.
///
/// Export indices use global numbering (imports + locals). We filter to local
/// functions and offset to local function index space for codegen (func_0, func_1, ...).
fn build_function_exports(parsed: &ParsedModule, num_imported_functions: usize) -> Vec<FuncExport> {
    parsed
        .exports
        .iter()
        .filter(|e| e.kind == ExportKind::Func && (e.index as usize) >= num_imported_functions)
        .map(|e| FuncExport {
            name: e.name.clone(),
            func_index: LocalFuncIdx::new((e.index as usize) - num_imported_functions),
        })
        .collect()
}

/// Enriches IR functions with signature metadata (type_idx and needs_host).
///
/// This iterates through the parsed functions and sets the type_idx and needs_host
/// fields in the corresponding IR functions.
fn enrich_ir_functions(
    parsed: &ParsedModule,
    canonical_type: &[usize],
    ir_functions: &mut [IrFunction],
    imported_globals: &[ImportedGlobalDef],
) {
    let num_imported_globals = imported_globals.len();
    for (func_idx, func) in parsed.functions.iter().enumerate() {
        if let Some(ir_func) = ir_functions.get_mut(func_idx) {
            ir_func.type_idx = TypeIdx::new(canonical_type[func.type_idx as usize]);
            ir_func.needs_host = function_calls_imports(ir_func, num_imported_globals);
        }
    }
}

/// Determines if a function calls imports or accesses imported globals.
fn function_calls_imports(ir_func: &IrFunction, num_imported_globals: usize) -> bool {
    ir_func.blocks.iter().any(|block| {
        block.instructions.iter().any(|instr| {
            matches!(instr, IrInstr::CallImport { .. })
                || (num_imported_globals > 0
                    && matches!(
                        instr,
                        IrInstr::GlobalGet { index, .. }
                            | IrInstr::GlobalSet { index, .. }
                            if index.as_usize() < num_imported_globals
                    ))
        })
    })
}

/// Builds type signatures for call_indirect type checking.
fn build_call_indirect_signatures(parsed: &ParsedModule) -> Vec<FuncSignature> {
    parsed
        .types
        .iter()
        .map(|ty| {
            let params = ty
                .params()
                .iter()
                .map(|vt| WasmType::from_wasmparser(*vt))
                .collect();
            let return_type = ty
                .results()
                .first()
                .map(|vt| WasmType::from_wasmparser(*vt));
            FuncSignature {
                params,
                return_type,
                type_idx: TypeIdx::new(0),
                needs_host: false,
            }
        })
        .collect()
}

/// Builds function import trait definitions.
fn build_function_imports(parsed: &ParsedModule) -> Vec<FuncImport> {
    parsed
        .imports
        .iter()
        .filter_map(|imp| match &imp.kind {
            ImportKind::Function(type_idx) => {
                let func_type = &parsed.types[*type_idx as usize];
                let params = func_type
                    .params()
                    .iter()
                    .map(|vt| WasmType::from_wasmparser(*vt))
                    .collect();
                let return_type = func_type
                    .results()
                    .first()
                    .map(|vt| WasmType::from_wasmparser(*vt));
                Some(FuncImport {
                    module_name: imp.module_name.clone(),
                    func_name: imp.name.clone(),
                    params,
                    return_type,
                })
            }
            _ => None,
        })
        .collect()
}
