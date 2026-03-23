//! IR optimization passes.
//!
//! This module implements optimizations on the intermediate representation (IR)
//! to improve code generation quality and runtime performance.
//!
//! Passes are split into two phases:
//! - **Pre-lowering** ([`optimize_ir`]): operates on SSA IR with phi nodes
//! - **Post-lowering** ([`optimize_lowered_ir`]): operates on lowered IR after phi destruction

use crate::ir::{LoweredModuleInfo, ModuleInfo};
use anyhow::Result;

// ── Shared utilities ─────────────────────────────────────────────────────────
pub(crate) mod utils;

// ── Pre-lowering passes ──────────────────────────────────────────────────────
mod dead_blocks;

// ── Post-lowering passes ─────────────────────────────────────────────────────
mod dead_instrs;
mod empty_blocks;
mod merge_blocks;
mod branch_fold;
mod gvn;
mod licm;
mod local_cse;

/// Optimizes the pure SSA IR before phi lowering.
///
/// Passes here operate on [`ModuleInfo`] with phi nodes still intact.
pub fn optimize_ir(module_info: ModuleInfo, do_opt: bool) -> Result<ModuleInfo> {
    let mut module_info = module_info;
    if do_opt {
        for func in &mut module_info.ir_functions {
            dead_blocks::eliminate(func)?;
        }
    }
    Ok(module_info)
}

/// Optimizes the lowered IR after phi nodes have been eliminated.
///
/// Runs all post-lowering optimization passes: structural cleanup, redundancy
/// elimination (CSE/GVN), loop optimizations (LICM), and branch simplification.
pub fn optimize_lowered_ir(
    module_info: LoweredModuleInfo,
    do_opt: bool,
) -> Result<LoweredModuleInfo> {
    let mut module_info = module_info;
    if do_opt {
        for func in &mut module_info.ir_functions {
            for _ in 0..2 {
                empty_blocks::eliminate(func);
                dead_blocks::eliminate(func)?;
                merge_blocks::eliminate(func);
                dead_blocks::eliminate(func)?;
                local_cse::eliminate(func);
                gvn::eliminate(func);
                dead_instrs::eliminate(func);
                branch_fold::eliminate(func);
                dead_instrs::eliminate(func);
                licm::eliminate(func);
                dead_instrs::eliminate(func);
            }
        }
    }
    Ok(module_info)
}

// ── optimize_ir integration tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::ir::{BlockId, IrBlock, IrFunction, IrTerminator, ModuleInfo, TypeIdx};

    #[test]
    fn optimize_ir_eliminates_dead_blocks_across_functions() {
        let make_ir_func = |blocks: Vec<IrBlock>| IrFunction {
            params: vec![],
            locals: vec![],
            blocks,
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
        };

        let module = ModuleInfo {
            ir_functions: vec![
                // func 0: block_0 → Return, block_1 dead
                make_ir_func(vec![
                    IrBlock {
                        id: BlockId(0),
                        instructions: vec![],
                        terminator: IrTerminator::Return { value: None },
                    },
                    IrBlock {
                        id: BlockId(1),
                        instructions: vec![],
                        terminator: IrTerminator::Return { value: None },
                    },
                ]),
                // func 1: block_0 → Jump → block_1 → Return (all live)
                make_ir_func(vec![
                    IrBlock {
                        id: BlockId(0),
                        instructions: vec![],
                        terminator: IrTerminator::Jump { target: BlockId(1) },
                    },
                    IrBlock {
                        id: BlockId(1),
                        instructions: vec![],
                        terminator: IrTerminator::Return { value: None },
                    },
                ]),
            ],
            ..Default::default()
        };

        let result = super::optimize_ir(module, true).unwrap();
        assert_eq!(
            result.ir_functions[0].blocks.len(),
            1,
            "dead block in func 0 should be removed"
        );
        assert_eq!(
            result.ir_functions[1].blocks.len(),
            2,
            "both blocks in func 1 should be kept"
        );
    }
}
