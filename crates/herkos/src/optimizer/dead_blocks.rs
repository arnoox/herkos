//! Dead basic block elimination.
//!
//! Removes blocks that are unreachable from the function entry. Dead blocks
//! arise naturally during IR translation when code follows an `unreachable` or
//! `return` instruction inside a Wasm structured control flow construct.

use super::utils::terminator_successors;
use crate::ir::{BlockId, IrBlock, IrFunction};
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

/// Computes the set of block IDs reachable from the entry block via BFS.
fn reachable_blocks(func: &IrFunction) -> Result<HashSet<BlockId>> {
    // Index blocks by ID for O(1) lookup during traversal.
    let block_map: HashMap<BlockId, &IrBlock> = func.blocks.iter().map(|b| (b.id, b)).collect();

    let mut reachable = HashSet::new();
    let mut worklist = vec![func.entry_block];

    while let Some(id) = worklist.pop() {
        // Skip blocks we have already visited to handle cycles and
        // diamond-shaped control flow (multiple predecessors).
        if !reachable.insert(id) {
            continue;
        }
        // Every BlockId that appears in a terminator must have been
        // pushed into func.blocks by the IR builder.
        let Some(block) = block_map.get(&id) else {
            bail!("IR invariant violated: terminator references BlockId({}) not present in func.blocks", id.0);
        };
        // Queue every block this one can transfer control to.
        worklist.extend(terminator_successors(&block.terminator));
    }
    Ok(reachable)
}

/// Removes unreachable basic blocks from a function in place.
pub fn eliminate(func: &mut IrFunction) -> Result<()> {
    let reachable = reachable_blocks(func)?;
    func.blocks.retain(|b| reachable.contains(&b.id));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IrInstr, IrTerminator, IrValue, TypeIdx, VarId, WasmType};

    /// Build a minimal `IrFunction` with the given blocks.
    /// Entry block is always `BlockId(0)`.
    fn make_func(blocks: Vec<IrBlock>) -> IrFunction {
        IrFunction {
            params: vec![],
            locals: vec![],
            blocks,
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
            needs_host: false,
        }
    }

    fn block_ids(func: &IrFunction) -> Vec<u32> {
        let mut ids: Vec<u32> = func.blocks.iter().map(|b| b.id.0).collect();
        ids.sort();
        ids
    }

    // ── Basic cases ──────────────────────────────────────────────────────

    #[test]
    fn single_block_return_kept() {
        let mut func = make_func(vec![IrBlock {
            id: BlockId(0),
            instructions: vec![],
            terminator: IrTerminator::Return { value: None },
        }]);
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), vec![0]);
    }

    #[test]
    fn all_reachable_linear_chain() {
        // block_0 → Jump → block_1 → Return
        let mut func = make_func(vec![
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
        ]);
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), vec![0, 1]);
    }

    // ── Simple dead block removal ────────────────────────────────────────

    #[test]
    fn dead_block_after_return_removed() {
        // block_0: Return  (entry)
        // block_1: Return  (dead — nobody jumps here)
        let mut func = make_func(vec![
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
        ]);
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), vec![0]);
    }

    #[test]
    fn dead_block_after_unconditional_jump_removed() {
        // block_0 → Jump → block_1 → Return
        // block_2: Return  (dead)
        let mut func = make_func(vec![
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
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), vec![0, 1]);
    }

    #[test]
    fn dead_block_after_unreachable_removed() {
        // block_0: Unreachable  (entry)
        // block_1: Return       (dead — unreachable has no successors)
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Unreachable,
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), vec![0]);
    }

    // ── Transitive unreachability ────────────────────────────────────────

    #[test]
    fn transitively_dead_blocks_removed() {
        // block_0 → Jump → block_1 → Return   (live)
        // block_2 → Jump → block_3             (dead)
        // block_3: Return  (dead — referenced only by dead block_2)
        let mut func = make_func(vec![
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
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(3) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), vec![0, 1]);
    }

    // ── BranchIf ────────────────────────────────────────────────────────

    #[test]
    fn branch_if_both_targets_reachable() {
        // block_0: BranchIf { if_true: block_1, if_false: block_2 }
        // block_1: Return
        // block_2: Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(0),
                    if_true: BlockId(1),
                    if_false: BlockId(2),
                },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), vec![0, 1, 2]);
    }

    #[test]
    fn branch_if_with_separate_dead_block() {
        // block_0: BranchIf → block_1 / block_2
        // block_3: Return  (dead)
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(0),
                    if_true: BlockId(1),
                    if_false: BlockId(2),
                },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), vec![0, 1, 2]);
    }

    // ── BranchTable ──────────────────────────────────────────────────────

    #[test]
    fn branch_table_all_targets_reachable() {
        // block_0: BranchTable { targets: [block_1, block_2], default: block_3 }
        // block_4: Return  (dead)
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(0),
                }],
                terminator: IrTerminator::BranchTable {
                    index: VarId(0),
                    targets: vec![BlockId(1), BlockId(2)],
                    default: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
            IrBlock {
                id: BlockId(4),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), vec![0, 1, 2, 3]);
    }

    // ── Idempotency ──────────────────────────────────────────────────────

    #[test]
    fn already_clean_function_unchanged() {
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(0),
                    if_true: BlockId(1),
                    if_false: BlockId(2),
                },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func).unwrap();
        let after_first = block_ids(&func);
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), after_first);
    }

    // ── Wasm-realistic pattern ───────────────────────────────────────────

    #[test]
    fn dead_code_after_early_return_in_wasm_block() {
        // Wasm: (block ... return ... end) emits blocks for unreachable tail.
        // block_0: entry → Jump → block_1
        // block_1: early Return (value)
        // block_2: dead tail (unreachable code inside wasm block)
        // block_3: dead continuation (block_2 never jumps here)
        let mut func = IrFunction {
            params: vec![(VarId(0), WasmType::I32)],
            locals: vec![],
            blocks: vec![
                IrBlock {
                    id: BlockId(0),
                    instructions: vec![],
                    terminator: IrTerminator::Jump { target: BlockId(1) },
                },
                IrBlock {
                    id: BlockId(1),
                    instructions: vec![],
                    terminator: IrTerminator::Return {
                        value: Some(VarId(0)),
                    },
                },
                IrBlock {
                    id: BlockId(2),
                    instructions: vec![IrInstr::Const {
                        dest: VarId(1),
                        value: IrValue::I32(99),
                    }],
                    terminator: IrTerminator::Jump { target: BlockId(3) },
                },
                IrBlock {
                    id: BlockId(3),
                    instructions: vec![],
                    terminator: IrTerminator::Return {
                        value: Some(VarId(1)),
                    },
                },
            ],
            entry_block: BlockId(0),
            return_type: Some(WasmType::I32),
            type_idx: TypeIdx::new(0),
            needs_host: false,
        };
        eliminate(&mut func).unwrap();
        assert_eq!(block_ids(&func), vec![0, 1]);
    }
}
