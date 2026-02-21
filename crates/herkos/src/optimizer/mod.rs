//! IR optimization passes.
//!
//! This module implements optimizations on the intermediate representation (IR)
//! to improve code generation quality and runtime performance.
//!
//! Each optimization is a self-contained sub-module. The top-level
//! [`optimize_ir`] function runs all passes in order.

use crate::ir::ModuleInfo;
use anyhow::Result;

// ── Passes ───────────────────────────────────────────────────────────────────
mod dead_blocks;

/// Optimizes the IR representation by running all passes in order.
pub fn optimize_ir(module_info: ModuleInfo) -> Result<ModuleInfo> {
    let mut module_info = module_info;
    for func in &mut module_info.ir_functions {
        dead_blocks::eliminate(func)?;
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

        let result = super::optimize_ir(module).unwrap();
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
