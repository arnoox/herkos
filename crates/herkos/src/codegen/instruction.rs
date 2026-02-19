//! Instruction code generation and terminator handling.
//!
//! Converts IR instructions and terminators into Rust code,
//! delegating to the backend for most operations.

use crate::backend::Backend;
use crate::ir::*;
use std::collections::HashMap;

/// Generate code for a single instruction with module info.
pub fn generate_instruction_with_info<B: Backend>(
    backend: &B,
    instr: &IrInstr,
    info: &ModuleInfo,
) -> String {
    match instr {
        IrInstr::Const { dest, value } => backend.emit_const(*dest, value),

        IrInstr::BinOp { dest, op, lhs, rhs } => backend.emit_binop(*dest, *op, *lhs, *rhs),

        IrInstr::UnOp { dest, op, operand } => backend.emit_unop(*dest, *op, *operand),

        IrInstr::Load {
            dest,
            ty,
            addr,
            offset,
            width,
            sign,
        } => backend.emit_load(*dest, *ty, *addr, *offset, *width, *sign),

        IrInstr::Store {
            ty,
            addr,
            value,
            offset,
            width,
        } => backend.emit_store(*ty, *addr, *value, *offset, *width),

        IrInstr::Call {
            dest,
            func_idx,
            args,
        } => {
            // Call to local function (imports are handled by CallImport)
            let has_globals = info.has_mutable_globals();
            let has_memory = info.has_memory;
            let has_table = info.has_table();
            backend.emit_call(*dest, *func_idx, args, has_globals, has_memory, has_table)
        }

        IrInstr::CallImport {
            dest,
            module_name,
            func_name,
            args,
            ..
        } => backend.emit_call_import(*dest, module_name, func_name, args),

        IrInstr::CallIndirect {
            dest,
            type_idx,
            table_idx,
            args,
        } => generate_call_indirect(*dest, *type_idx, *table_idx, args, info),

        IrInstr::Assign { dest, src } => backend.emit_assign(*dest, *src),

        IrInstr::GlobalGet { dest, index } => {
            if *index < info.imported_globals.len() {
                // Imported global — access via host trait getter
                let g = &info.imported_globals[*index];
                format!("                {} = host.get_{}();", dest, g.name)
            } else {
                // Local global — use corrected index and backend
                let local_idx = index - info.imported_globals.len();
                let is_mutable = info
                    .globals
                    .get(local_idx)
                    .map(|g| g.mutable)
                    .unwrap_or(true);
                backend.emit_global_get(*dest, local_idx, is_mutable)
            }
        }

        IrInstr::GlobalSet { index, value } => {
            if *index < info.imported_globals.len() {
                // Imported global — access via host trait setter
                let g = &info.imported_globals[*index];
                format!("                host.set_{}({});", g.name, value)
            } else {
                // Local global — use corrected index and backend
                let local_idx = index - info.imported_globals.len();
                backend.emit_global_set(local_idx, *value)
            }
        }

        IrInstr::MemorySize { dest } => backend.emit_memory_size(*dest),

        IrInstr::MemoryGrow { dest, delta } => backend.emit_memory_grow(*dest, *delta),

        IrInstr::Select {
            dest,
            val1,
            val2,
            condition,
        } => backend.emit_select(*dest, *val1, *val2, *condition),
    }
}

/// Generate code for a terminator with BlockId to index mapping.
pub fn generate_terminator_with_mapping<B: Backend>(
    backend: &B,
    term: &IrTerminator,
    block_id_to_index: &HashMap<BlockId, usize>,
    func_return_type: Option<WasmType>,
) -> String {
    match term {
        IrTerminator::Return { value } => {
            // If the function has a return type but the return has no value,
            // this is dead code after `unreachable` — emit a trap instead
            // of `return Ok(())` which would be a type mismatch.
            if value.is_none() && func_return_type.is_some() {
                return backend.emit_unreachable();
            }
            backend.emit_return(*value)
        }

        IrTerminator::Jump { target } => {
            let idx = block_id_to_index[target];
            backend.emit_jump_to_index(idx)
        }

        IrTerminator::BranchIf {
            condition,
            if_true,
            if_false,
        } => {
            let true_idx = block_id_to_index[if_true];
            let false_idx = block_id_to_index[if_false];
            backend.emit_branch_if_to_index(*condition, true_idx, false_idx)
        }

        IrTerminator::BranchTable {
            index,
            targets,
            default,
        } => {
            let target_indices: Vec<usize> = targets.iter().map(|t| block_id_to_index[t]).collect();
            let default_idx = block_id_to_index[default];
            backend.emit_branch_table_to_index(*index, &target_indices, default_idx)
        }

        IrTerminator::Unreachable => backend.emit_unreachable(),
    }
}

/// Generate inline dispatch code for `call_indirect`.
///
/// The generated code:
/// 1. Looks up the table entry by index
/// 2. Checks the type signature matches
/// 3. Dispatches to the matching function via a match on func_index
fn generate_call_indirect(
    dest: Option<VarId>,
    type_idx: usize,
    table_idx: VarId,
    args: &[VarId],
    info: &ModuleInfo,
) -> String {
    let has_globals = info.has_mutable_globals();
    let has_memory = info.has_memory;
    let has_table = info.has_table();

    // Canonicalize the type index for structural equivalence (Wasm spec §4.4.9).
    // Two different type indices with identical (params, results) must match.
    let canon_idx = info
        .canonical_type
        .get(type_idx)
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
    let base_args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    let call_args = crate::codegen::utils::build_inner_call_args(
        &base_args,
        has_globals,
        "globals",
        has_memory,
        "memory",
        has_table,
        "table",
    );
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
