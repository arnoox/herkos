//! IR optimization passes.
//!
//! This module implements optimizations on the intermediate representation (IR)
//! to improve code generation quality and runtime performance.
//!
//! Each optimization is a self-contained sub-module. The top-level
//! [`optimize_ir`] function runs all passes in order.

use crate::ir::{LoweredModuleInfo, ModuleInfo};
use anyhow::Result;

// ── Passes ───────────────────────────────────────────────────────────────────
mod dead_blocks;

/// Optimizes the pure SSA IR before phi lowering.
///
/// Passes here operate on [`ModuleInfo`] with phi nodes still intact.
/// Control-flow based passes (e.g. dead block elimination) belong here
/// because reachability is identical in SSA and lowered form.
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
/// Passes here operate on [`LoweredModuleInfo`] where all `IrInstr::Phi`
/// nodes have been replaced by `IrInstr::Assign` in predecessor blocks.
pub fn optimize_lowered_ir(
    module_info: LoweredModuleInfo,
    _do_opt: bool,
) -> Result<LoweredModuleInfo> {
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
            needs_host: false,
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
