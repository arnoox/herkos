//! Function code generation from IR.
//!
//! Converts IR functions to complete Rust functions,
//! including signature generation, variable declarations,
//! and block-to-code translation.

use crate::backend::Backend;
use crate::ir::*;
use anyhow::Result;

/// Generate a complete Rust function from IR with module info.
///
/// `is_public` controls whether the function is `pub fn` or `fn`.
pub fn generate_function_with_info<B: Backend>(
    backend: &B,
    ir_func: &IrFunction,
    func_name: &str,
    info: &ModuleInfo,
    is_public: bool,
) -> Result<String> {
    let mut output = String::new();

    // Suppress warnings for generated code patterns that are hard to avoid
    output.push_str("#[allow(unused_mut, unused_variables, unused_assignments, clippy::needless_return, clippy::manual_range_contains, clippy::never_loop)]\n");

    // Generate function signature
    output.push_str(&generate_signature_with_info(
        backend, ir_func, func_name, info, is_public,
    ));
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

    // Seed with declared local variable types
    for (var, ty) in &ir_func.locals {
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
                    let ty = if *func_idx >= info.num_imported_functions() {
                        let local_idx = func_idx - info.num_imported_functions();
                        info.ir_functions
                            .get(local_idx)
                            .and_then(|f| f.return_type)
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
                        .get(*import_idx)
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
                    // Distinguish imported globals (lower indices) from local globals
                    let ty = if *index < info.imported_globals.len() {
                        // Imported global
                        info.imported_globals[*index].wasm_type
                    } else {
                        // Local global — adjust index by removing imported count
                        let local_idx = *index - info.imported_globals.len();
                        info.globals
                            .get(local_idx)
                            .map(|g| g.init_value.ty())
                            .unwrap_or(WasmType::I32)
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
                        .get(*type_idx)
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
        let rust_ty = crate::codegen::types::wasm_type_to_rust(ty);
        let default = ty.default_value_literal();
        output.push_str(&format!("    let mut {var}: {rust_ty} = {default};\n"));
    }

    if ir_func.blocks.len() == 1 {
        // Single-block optimization: emit flat body without loop/match
        let block = &ir_func.blocks[0];
        for instr in &block.instructions {
            let code =
                crate::codegen::instruction::generate_instruction_with_info(backend, instr, info);
            output.push_str(&code);
            output.push('\n');
        }
        let term_code = crate::codegen::instruction::generate_terminator_with_mapping(
            backend,
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
                let code = crate::codegen::instruction::generate_instruction_with_info(
                    backend, instr, info,
                );
                output.push_str(&code);
                output.push('\n');
            }

            let term_code = crate::codegen::instruction::generate_terminator_with_mapping(
                backend,
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
fn generate_signature_with_info<B: Backend>(
    _backend: &B,
    ir_func: &IrFunction,
    func_name: &str,
    info: &ModuleInfo,
    is_public: bool,
) -> String {
    let visibility = if is_public { "pub " } else { "" };

    // Check if function needs host parameter (imports or global imports)
    let needs_host =
        has_import_calls(ir_func) || has_global_import_access(ir_func, info.imported_globals.len());
    let trait_bounds_opt = if needs_host {
        crate::codegen::traits::build_trait_bounds(info)
    } else {
        None
    };

    let has_multiple_bounds = trait_bounds_opt.as_ref().is_some_and(|b| b.contains(" + "));

    // Build generics: handle both H (host) and MP (imported memory size)
    let mut generics: Vec<String> = Vec::new();
    if info.has_memory_import {
        generics.push("const MP: usize".to_string());
    }
    if has_multiple_bounds {
        generics.push(format!("H: {}", trait_bounds_opt.as_ref().unwrap()));
    }

    let generic_part = if generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", generics.join(", "))
    };

    let mut sig = format!("{visibility}fn {func_name}{generic_part}(");

    // Parameters (mutable, as in WebAssembly all locals are mutable)
    let mut param_parts: Vec<String> = ir_func
        .params
        .iter()
        .map(|(var_id, ty)| {
            let rust_ty = crate::codegen::types::wasm_type_to_rust(ty);
            format!("mut {}: {}", var_id, rust_ty)
        })
        .collect();

    // Add host parameter if function needs imports or global imports
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

    // Add globals parameter if module has mutable globals
    if info.has_mutable_globals() {
        param_parts.push("globals: &mut Globals".to_string());
    }

    // Add memory parameter — either const MAX_PAGES or generic MP
    if info.has_memory {
        param_parts.push("memory: &mut IsolatedMemory<MAX_PAGES>".to_string());
    } else if info.has_memory_import {
        param_parts.push("memory: &mut IsolatedMemory<MP>".to_string());
    }

    // Add table parameter if module has a table
    if info.has_table() {
        param_parts.push("table: &Table<TABLE_MAX>".to_string());
    }

    sig.push_str(&param_parts.join(", "));
    sig.push(')');

    // Return type
    sig.push_str(&format!(
        " -> {}",
        crate::codegen::types::format_return_type(ir_func.return_type.as_ref())
    ));

    sig
}

/// Check if an IR function has any import calls.
fn has_import_calls(ir_func: &IrFunction) -> bool {
    ir_func.blocks.iter().any(|block| {
        block
            .instructions
            .iter()
            .any(|instr| matches!(instr, IrInstr::CallImport { .. }))
    })
}

/// Check if an IR function accesses any imported globals.
fn has_global_import_access(ir_func: &IrFunction, num_imported_globals: usize) -> bool {
    if num_imported_globals == 0 {
        return false;
    }
    ir_func.blocks.iter().any(|block| {
        block.instructions.iter().any(|instr| {
            matches!(instr, IrInstr::GlobalGet { index, .. } | IrInstr::GlobalSet { index, .. } if *index < num_imported_globals)
        })
    })
}
