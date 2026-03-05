//! SSA phi-node lowering.
//!
//! ## Overview
//!
//! This pass converts `IrInstr::Phi` nodes — which are SSA-form join-point
//! selectors — into ordinary `IrInstr::Assign` instructions placed at the end
//! of predecessor blocks (just before their terminators). After this pass, no
//! `IrInstr::Phi` instructions remain in the IR.
//!
//! This pass **must run before all optimizer passes** and **before codegen**.
//! It is a phase transition (SSA destruction), not an optimization.
//!
//! ## Algorithm
//!
//! For each function:
//!
//! 1. **Prune stale phi sources**: dead_blocks may have removed predecessor
//!    blocks. Remove any `(pred_block, _)` entry in a Phi's `srcs` whose
//!    `pred_block` no longer exists in `func.blocks`.
//!
//! 2. **Simplify trivial phis**: A phi is trivial if:
//!    - It has no sources → dead (replace with self-assign, will be removed by
//!      dead_instrs if unused).
//!    - Ignoring self-references (`src == dest`), all remaining sources resolve
//!      to the same single variable.
//!
//!    In both cases, replace `Phi { dest, srcs }` with `Assign { dest, src }`.
//!    Repeat until no more trivial phis can be simplified.
//!
//! 3. **Lower non-trivial phis**: For each remaining `Phi { dest, srcs }`:
//!    - In each predecessor block, insert `Assign { dest, src }` just before
//!      the block's terminator.
//!    - Remove the `Phi` instruction from the join block.
//!
//! ## Why predecessor assignments?
//!
//! A phi at a join point conceptually says "take the value from whichever
//! predecessor we came from". In the generated Rust state machine, the predecessor
//! block's code runs immediately before transitioning to the join block, so
//! assigning there is equivalent to selecting based on the taken path.

use crate::ir::{BlockId, IrFunction, IrInstr, ModuleInfo, VarId};
use std::collections::HashSet;

/// Lower all `IrInstr::Phi` nodes in `module_info`, returning a [`super::LoweredModuleInfo`].
///
/// After this call, no `IrInstr::Phi` instructions remain in any function.
/// The returned [`super::LoweredModuleInfo`] can be passed to the optimizer and codegen.
pub fn lower(module_info: ModuleInfo) -> super::LoweredModuleInfo {
    let mut module_info = module_info;
    for func in &mut module_info.ir_functions {
        lower_func(func);
    }
    super::LoweredModuleInfo(module_info)
}

/// Lower all `IrInstr::Phi` nodes in a single function.
fn lower_func(func: &mut IrFunction) {
    // Collect the set of live block IDs (blocks still present after dead-block elimination).
    let live_blocks: HashSet<BlockId> = func.blocks.iter().map(|b| b.id).collect();

    // Step 1: Prune phi sources that refer to removed (dead) predecessor blocks.
    for block in &mut func.blocks {
        for instr in &mut block.instructions {
            if let IrInstr::Phi { srcs, .. } = instr {
                srcs.retain(|(pred_id, _)| live_blocks.contains(pred_id));
            }
        }
    }

    // Step 2: Simplify trivial phis to Assign, iterating to fixpoint.
    //
    // A phi is trivial when — after removing self-references — all remaining
    // sources point to the same VarId.  We iterate because simplifying one phi
    // may allow another that referenced it to become trivial.
    loop {
        let mut changed = false;
        for block in &mut func.blocks {
            for instr in &mut block.instructions {
                if let IrInstr::Phi { dest, srcs } = instr {
                    let phi_dest = *dest;

                    // Collect unique non-self sources
                    let non_self: Vec<VarId> = srcs
                        .iter()
                        .map(|(_, v)| *v)
                        .filter(|&v| v != phi_dest)
                        .collect::<std::collections::HashSet<VarId>>()
                        .into_iter()
                        .collect();

                    let trivial_src = if non_self.is_empty() {
                        // All sources are self-references — phi is dead.
                        // Replace with an assign from itself (a no-op) so the
                        // dead_instrs pass can remove it if unused.
                        Some(phi_dest)
                    } else if non_self.len() == 1 {
                        // All non-self sources agree on one value.
                        Some(non_self[0])
                    } else {
                        None
                    };

                    if let Some(src) = trivial_src {
                        *instr = IrInstr::Assign {
                            dest: phi_dest,
                            src,
                        };
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break;
        }

        // After simplifying a phi, propagate the change: replace uses of the
        // former phi's dest with its now-known single source in the same block.
        // (Full global propagation is handled by copy_prop; we just do the local
        // fixpoint simplification of other phis in the same block here.)
        for block in &mut func.blocks {
            // Collect Assign(dest, src) replacements from this iteration's simplifications.
            let replacements: Vec<(VarId, VarId)> = block
                .instructions
                .iter()
                .filter_map(|i| {
                    if let IrInstr::Assign { dest, src } = i {
                        // Only propagate trivial-phi-turned-assigns where src != dest
                        if *dest != *src {
                            Some((*dest, *src))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            for (old, new) in replacements {
                for instr in &mut block.instructions {
                    replace_phi_src(instr, old, new);
                }
            }
        }
    }

    // Step 3: Lower non-trivial phis to predecessor-block assignments.
    //
    // We collect all phi nodes first, then mutate blocks.
    // `phi_assignments` maps `(dest, pred_block) -> src`.
    // `phi_locations` maps `block_id -> [phi_dest]` so we know which phis to remove.
    let mut phi_assignments: Vec<(BlockId, IrInstr)> = Vec::new();
    let mut phi_block_dests: Vec<(BlockId, VarId)> = Vec::new();

    for block in &func.blocks {
        for instr in &block.instructions {
            if let IrInstr::Phi { dest, srcs } = instr {
                phi_block_dests.push((block.id, *dest));
                for (pred_block, src) in srcs {
                    phi_assignments.push((
                        *pred_block,
                        IrInstr::Assign {
                            dest: *dest,
                            src: *src,
                        },
                    ));
                }
            }
        }
    }

    // Insert assignments into predecessor blocks (before the terminator).
    for (pred_id, assign) in phi_assignments {
        if let Some(block) = func.blocks.iter_mut().find(|b| b.id == pred_id) {
            // Insert just before the terminator (i.e., at the end of the instruction list).
            block.instructions.push(assign);
        }
    }

    // Remove phi instructions from their join blocks.
    let phi_dests: HashSet<VarId> = phi_block_dests.iter().map(|(_, d)| *d).collect();
    for block in &mut func.blocks {
        block
            .instructions
            .retain(|i| !matches!(i, IrInstr::Phi { dest, .. } if phi_dests.contains(dest)));
    }
}

/// Replace every read-occurrence of `old` with `new` in `instr`.
/// Used during trivial-phi simplification to propagate the resolved source.
fn replace_phi_src(instr: &mut IrInstr, old: VarId, new: VarId) {
    let sub = |v: &mut VarId| {
        if *v == old {
            *v = new;
        }
    };
    match instr {
        IrInstr::Phi { srcs, .. } => {
            for (_, src) in srcs {
                sub(src);
            }
        }
        IrInstr::Assign { src, .. } => sub(src),
        // Other instruction kinds don't appear before phi lowering completes
        // within the trivial-phi loop, so only phi/assign need handling here.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{BlockId, IrBlock, IrFunction, IrTerminator, IrValue, TypeIdx, VarId};

    fn make_module(blocks: Vec<IrBlock>) -> ModuleInfo {
        ModuleInfo {
            has_memory: false,
            has_memory_import: false,
            max_pages: 0,
            initial_pages: 0,
            table_initial: 0,
            table_max: 0,
            element_segments: Vec::new(),
            globals: Vec::new(),
            data_segments: Vec::new(),
            func_exports: Vec::new(),
            type_signatures: Vec::new(),
            canonical_type: Vec::new(),
            func_imports: Vec::new(),
            imported_globals: Vec::new(),
            ir_functions: vec![IrFunction {
                params: vec![],
                locals: vec![],
                blocks,
                entry_block: BlockId(0),
                return_type: None,
                type_idx: TypeIdx::new(0),
                needs_host: false,
            }],
            wasm_version: 1,
        }
    }

    /// A trivial phi where all non-self sources agree: phi(v0, v0) → Assign(dest, v0).
    #[test]
    fn test_trivial_phi_same_source() {
        // block_0: v0 = const 1; jump block_1
        // block_1: v1 = phi((block_0, v0), (block_0, v0)); return v1
        let block0 = IrBlock {
            id: BlockId(0),
            instructions: vec![IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(1),
            }],
            terminator: IrTerminator::Jump { target: BlockId(1) },
        };
        let block1 = IrBlock {
            id: BlockId(1),
            instructions: vec![IrInstr::Phi {
                dest: VarId(1),
                srcs: vec![(BlockId(0), VarId(0)), (BlockId(0), VarId(0))],
            }],
            terminator: IrTerminator::Return {
                value: Some(VarId(1)),
            },
        };

        let module = make_module(vec![block0, block1]);
        let lowered = lower(module);

        // phi should be simplified to Assign
        let instrs = &lowered.ir_functions[0].blocks[1].instructions;
        assert_eq!(instrs.len(), 1);
        assert!(matches!(
            instrs[0],
            IrInstr::Assign {
                dest: VarId(1),
                src: VarId(0)
            }
        ));
    }

    /// A non-trivial phi with two different sources gets lowered to predecessor assignments.
    #[test]
    fn test_non_trivial_phi_lowering() {
        // block_0: v0=1; br_if block_1 else block_2
        // block_1: v1=2; jump block_3
        // block_2: v2=3; jump block_3
        // block_3: v3=phi((block_1,v1),(block_2,v2)); return v3
        let block0 = IrBlock {
            id: BlockId(0),
            instructions: vec![IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(0),
            }],
            terminator: IrTerminator::BranchIf {
                condition: VarId(0),
                if_true: BlockId(1),
                if_false: BlockId(2),
            },
        };
        let block1 = IrBlock {
            id: BlockId(1),
            instructions: vec![IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(2),
            }],
            terminator: IrTerminator::Jump { target: BlockId(3) },
        };
        let block2 = IrBlock {
            id: BlockId(2),
            instructions: vec![IrInstr::Const {
                dest: VarId(2),
                value: IrValue::I32(3),
            }],
            terminator: IrTerminator::Jump { target: BlockId(3) },
        };
        let block3 = IrBlock {
            id: BlockId(3),
            instructions: vec![IrInstr::Phi {
                dest: VarId(3),
                srcs: vec![(BlockId(1), VarId(1)), (BlockId(2), VarId(2))],
            }],
            terminator: IrTerminator::Return {
                value: Some(VarId(3)),
            },
        };

        let module = make_module(vec![block0, block1, block2, block3]);
        let lowered = lower(module);
        let func = &lowered.ir_functions[0];

        // No phi in block3 after lowering
        assert!(!func.blocks[3]
            .instructions
            .iter()
            .any(|i| matches!(i, IrInstr::Phi { .. })));

        // block1 should have an Assign v3 = v1 appended
        assert!(func.blocks[1].instructions.iter().any(|i| matches!(
            i,
            IrInstr::Assign {
                dest: VarId(3),
                src: VarId(1)
            }
        )));

        // block2 should have an Assign v3 = v2 appended
        assert!(func.blocks[2].instructions.iter().any(|i| matches!(
            i,
            IrInstr::Assign {
                dest: VarId(3),
                src: VarId(2)
            }
        )));
    }
}
