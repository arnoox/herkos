//! Function code generation from IR.
//!
//! Converts IR functions to complete Rust functions,
//! including signature generation, variable declarations,
//! and block-to-code translation.

use crate::backend::Backend;
use crate::ir::*;
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Returns the successor block IDs for a terminator.
fn terminator_successors(term: &IrTerminator) -> Vec<BlockId> {
    match term {
        IrTerminator::Return { .. } | IrTerminator::Unreachable => vec![],
        IrTerminator::Jump { target } => vec![*target],
        IrTerminator::BranchIf {
            if_true, if_false, ..
        } => vec![*if_true, *if_false],
        IrTerminator::BranchTable {
            targets, default, ..
        } => targets
            .iter()
            .chain(std::iter::once(default))
            .copied()
            .collect(),
    }
}

/// Compute the set of blocks that can be inlined into their sole predecessor's
/// conditional arm. A block is inlinable when:
/// - It has exactly one *distinct* block-predecessor
/// - It is not the entry block
/// - Its predecessor reaches it via `BranchIf` (not `Jump` — those are handled
///   by the `merge_blocks` IR pass)
///
/// Inlining is applied recursively: if block B is inlined into P, and B's
/// terminator targets block C which is also inlinable, C is inlined into the
/// same arm.
fn compute_inlinable_blocks(ir_func: &IrFunction) -> HashSet<BlockId> {
    // Build predecessor map: BlockId → set of predecessor BlockIds.
    let mut preds: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
    for block in &ir_func.blocks {
        preds.entry(block.id).or_default();
        for succ in terminator_successors(&block.terminator) {
            preds.entry(succ).or_default().insert(block.id);
        }
    }

    let mut inlinable = HashSet::new();
    for block in &ir_func.blocks {
        if block.id == ir_func.entry_block {
            continue;
        }
        if let Some(pred_set) = preds.get(&block.id) {
            if pred_set.len() == 1 {
                inlinable.insert(block.id);
            }
        }
    }
    inlinable
}

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
    let mut block_id_to_index = HashMap::new();
    for (idx, block) in ir_func.blocks.iter().enumerate() {
        block_id_to_index.insert(block.id, idx);
    }

    // Index blocks by BlockId for O(1) lookup during inline emission.
    let block_by_id: HashMap<BlockId, &IrBlock> =
        ir_func.blocks.iter().map(|b| (b.id, b)).collect();

    // Compute which blocks can be inlined into their predecessor's conditional arm.
    let inlinable = compute_inlinable_blocks(ir_func);

    // Compute trivial return blocks: no instructions, terminator is Return,
    // and not the entry block. These can be inlined at every use site
    // regardless of predecessor count.
    let trivial_returns: HashSet<BlockId> = ir_func
        .blocks
        .iter()
        .filter(|b| {
            b.id != ir_func.entry_block
                && b.instructions.is_empty()
                && matches!(b.terminator, IrTerminator::Return { .. })
        })
        .map(|b| b.id)
        .collect();

    // Collect all variables and their types from instructions.
    let mut var_types: HashMap<VarId, WasmType> = HashMap::new();

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
                    // func_idx is in local space (imports already excluded)
                    let ty = info
                        .ir_function(*func_idx)
                        .and_then(|f| f.return_type)
                        .unwrap_or(WasmType::I32);
                    var_types.insert(*dest, ty);
                }
                IrInstr::CallImport {
                    dest: Some(dest),
                    import_idx,
                    ..
                } => {
                    // Look up import signature from func_imports
                    let ty = info
                        .func_import(import_idx.clone())
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
                    let ty = match info.resolve_global(*index) {
                        ResolvedGlobal::Imported(_idx, g) => g.wasm_type,
                        ResolvedGlobal::Local(_idx, g) => g.init_value.ty(),
                    };
                    var_types.insert(*dest, ty);
                }
                IrInstr::CallIndirect {
                    dest: Some(dest),
                    type_idx,
                    ..
                } => {
                    let ty = info
                        .type_signature(type_idx.clone())
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

    // Determine which blocks are emitted as match arms (non-inlined, non-trivial-return blocks).
    let emitted_blocks: Vec<(usize, &IrBlock)> = ir_func
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| !inlinable.contains(&b.id) && !trivial_returns.contains(&b.id))
        .collect();

    // Multi-block: state machine with per-function Block enum
    output.push_str("    #[derive(Clone, Copy)]\n    #[allow(dead_code)]\n");
    output.push_str("    enum Block { ");
    for (i, (idx, _)) in emitted_blocks.iter().enumerate() {
        if i > 0 {
            output.push_str(", ");
        }
        output.push_str(&format!("B{}", idx));
    }
    output.push_str(" }\n");

    // Entry block is always at index 0 in emitted_blocks.
    let entry_idx = block_id_to_index[&ir_func.entry_block];
    output.push_str(&format!(
        "    let mut __current_block = Block::B{};\n",
        entry_idx
    ));
    output.push_str("    loop {\n");
    output.push_str("        match __current_block {\n");

    let ctx = EmitCtx {
        backend,
        info,
        block_id_to_index: &block_id_to_index,
        block_by_id: &block_by_id,
        inlinable: &inlinable,
        trivial_returns: &trivial_returns,
        func_return_type: ir_func.return_type,
    };

    for (idx, block) in &emitted_blocks {
        output.push_str(&format!("            Block::B{} => {{\n", idx));
        ctx.emit_block_body(block, &mut output, 0)?;
        output.push_str("            }\n");
    }

    // No catch-all needed — match is exhaustive over Block enum
    output.push_str("        }\n");
    output.push_str("    }\n");

    output.push_str("}\n");
    Ok(output)
}

/// Maximum inline depth to prevent unbounded recursion on pathological CFGs.
const MAX_INLINE_DEPTH: usize = 16;

/// Shared context for recursive block emission, avoiding excessive parameters.
struct EmitCtx<'a, B: Backend> {
    backend: &'a B,
    info: &'a ModuleInfo,
    block_id_to_index: &'a HashMap<BlockId, usize>,
    block_by_id: &'a HashMap<BlockId, &'a IrBlock>,
    inlinable: &'a HashSet<BlockId>,
    trivial_returns: &'a HashSet<BlockId>,
    func_return_type: Option<WasmType>,
}

impl<B: Backend> EmitCtx<'_, B> {
    /// Try to emit a target block inline. Returns `true` if the target was
    /// emitted (either as a single-predecessor inline or a trivial return).
    fn try_emit_inline_target(
        &self,
        target: &BlockId,
        output: &mut String,
        depth: usize,
    ) -> Result<bool> {
        if depth < MAX_INLINE_DEPTH && self.inlinable.contains(target) {
            self.emit_block_body(self.block_by_id[target], output, depth + 1)?;
            return Ok(true);
        }
        if self.trivial_returns.contains(target) {
            let block = self.block_by_id[target];
            let term_code =
                crate::codegen::instruction::generate_terminator_with_mapping(
                    self.backend,
                    &block.terminator,
                    self.block_id_to_index,
                    self.func_return_type,
                );
            output.push_str(&term_code);
            output.push('\n');
            return Ok(true);
        }
        Ok(false)
    }

    /// Emit the body of a block (instructions + terminator), inlining single-
    /// predecessor target blocks into conditional arms recursively.
    fn emit_block_body(
        &self,
        block: &IrBlock,
        output: &mut String,
        depth: usize,
    ) -> Result<()> {
        for instr in &block.instructions {
            let code = crate::codegen::instruction::generate_instruction_with_info(
                self.backend,
                instr,
                self.info,
            )?;
            output.push_str(&code);
            output.push('\n');
        }
        self.emit_terminator(&block.terminator, output, depth)
    }

    /// Emit a terminator, inlining target blocks that are in the `inlinable` set
    /// or the `trivial_returns` set.
    fn emit_terminator(
        &self,
        term: &IrTerminator,
        output: &mut String,
        depth: usize,
    ) -> Result<()> {
        match term {
            IrTerminator::BranchIf {
                condition,
                if_true,
                if_false,
            } => {
                output.push_str(&format!("                if {} != 0 {{\n", condition));

                let true_inlined = self.try_emit_inline_target(if_true, output, depth)?;
                if !true_inlined {
                    let idx = self.block_id_to_index[if_true];
                    output.push_str(&format!(
                        "                    __current_block = Block::B{};\n",
                        idx
                    ));
                }

                output.push_str("                } else {\n");

                let false_inlined = self.try_emit_inline_target(if_false, output, depth)?;
                if !false_inlined {
                    let idx = self.block_id_to_index[if_false];
                    output.push_str(&format!(
                        "                    __current_block = Block::B{};\n",
                        idx
                    ));
                }

                output.push_str("                }\n");
                if !true_inlined || !false_inlined {
                    output.push_str("                continue;\n");
                }
            }

            // Jump to a trivial return block — emit the return directly.
            IrTerminator::Jump { target } if self.trivial_returns.contains(target) => {
                let block = self.block_by_id[target];
                let term_code =
                    crate::codegen::instruction::generate_terminator_with_mapping(
                        self.backend,
                        &block.terminator,
                        self.block_id_to_index,
                        self.func_return_type,
                    );
                output.push_str(&term_code);
                output.push('\n');
            }

            // For other terminators, fall back to the standard generation.
            _ => {
                let term_code =
                    crate::codegen::instruction::generate_terminator_with_mapping(
                        self.backend,
                        term,
                        self.block_id_to_index,
                        self.func_return_type,
                    );
                output.push_str(&term_code);
                output.push('\n');
            }
        }

        Ok(())
    }
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
            matches!(instr, IrInstr::GlobalGet { index, .. } | IrInstr::GlobalSet { index, .. } if index.as_usize() < num_imported_globals)
        })
    })
}
