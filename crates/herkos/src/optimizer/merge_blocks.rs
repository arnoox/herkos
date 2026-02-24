//! Single-predecessor block merging.
//!
//! When a block `B` has exactly one predecessor `P`, and `P` reaches `B` via an
//! unconditional `Jump`, then `B` can be appended to `P` — its instructions are
//! concatenated and `P` inherits `B`'s terminator.
//!
//! The pass iterates to a fixed point so that chains like
//!   B0 → Jump → B1 → Jump → B2 → Return
//! collapse into a single block B0 → Return.
//!
//! After merging, absorbed blocks are removed from `func.blocks`.

use crate::ir::{BlockId, IrFunction, IrTerminator};
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

/// Build a map from each block ID to the set of *distinct* predecessor block IDs.
fn build_predecessors(func: &IrFunction) -> HashMap<BlockId, HashSet<BlockId>> {
    let mut preds: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
    // Ensure every block has an entry (even if no predecessors).
    for block in &func.blocks {
        preds.entry(block.id).or_default();
    }
    for block in &func.blocks {
        for succ in terminator_successors(&block.terminator) {
            preds.entry(succ).or_default().insert(block.id);
        }
    }
    preds
}

/// Merge single-predecessor blocks reached via unconditional `Jump`.
///
/// Iterates to a fixed point, then removes absorbed blocks.
pub fn eliminate(func: &mut IrFunction) {
    loop {
        let preds = build_predecessors(func);

        // Index blocks by ID for lookup during merging.
        let block_map: HashMap<BlockId, usize> = func
            .blocks
            .iter()
            .enumerate()
            .map(|(i, b)| (b.id, i))
            .collect();

        // Collect merge pairs: (predecessor_idx, target_idx) where target has
        // exactly one predecessor and that predecessor reaches it via Jump.
        let mut merges: Vec<(usize, usize)> = Vec::new();
        // Track which blocks are already involved in a merge this round to avoid
        // conflicting operations (a block can't be both a merge source and target
        // in the same round).
        let mut involved: HashSet<usize> = HashSet::new();

        for block in &func.blocks {
            if let IrTerminator::Jump { target } = block.terminator {
                // Skip self-loops.
                if target == block.id {
                    continue;
                }
                // Never merge away the entry block.
                if target == func.entry_block {
                    continue;
                }
                if let Some(pred_set) = preds.get(&target) {
                    if pred_set.len() == 1 {
                        let pred_idx = block_map[&block.id];
                        let target_idx = block_map[&target];
                        // Avoid conflicts: each block participates in at most one
                        // merge per round.
                        if !involved.contains(&pred_idx) && !involved.contains(&target_idx) {
                            merges.push((pred_idx, target_idx));
                            involved.insert(pred_idx);
                            involved.insert(target_idx);
                        }
                    }
                }
            }
        }

        if merges.is_empty() {
            break;
        }

        // Perform merges. We collect the target block data first to avoid borrow
        // conflicts on func.blocks.
        let absorbed: HashSet<usize> = merges.iter().map(|(_, t)| *t).collect();

        for (pred_idx, target_idx) in &merges {
            // Take target block's data out.
            let target_instrs = std::mem::take(&mut func.blocks[*target_idx].instructions);
            let target_term = std::mem::replace(
                &mut func.blocks[*target_idx].terminator,
                IrTerminator::Unreachable,
            );
            // Append to predecessor.
            func.blocks[*pred_idx].instructions.extend(target_instrs);
            func.blocks[*pred_idx].terminator = target_term;
        }

        // Remove absorbed blocks (iterate in reverse to preserve indices).
        let mut absorbed_sorted: Vec<usize> = absorbed.into_iter().collect();
        absorbed_sorted.sort_unstable_by(|a, b| b.cmp(a));
        for idx in absorbed_sorted {
            func.blocks.remove(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IrBlock, IrFunction, IrInstr, IrTerminator, IrValue, TypeIdx, VarId};

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
        func.blocks.iter().map(|b| b.id.0).collect()
    }

    fn instr_block(id: u32, dest: u32, val: i32, term: IrTerminator) -> IrBlock {
        IrBlock {
            id: BlockId(id),
            instructions: vec![IrInstr::Const {
                dest: VarId(dest),
                value: IrValue::I32(val),
            }],
            terminator: term,
        }
    }

    // ── Basic cases ──────────────────────────────────────────────────────

    #[test]
    fn single_block_unchanged() {
        let mut func = make_func(vec![IrBlock {
            id: BlockId(0),
            instructions: vec![],
            terminator: IrTerminator::Return { value: None },
        }]);
        eliminate(&mut func);
        assert_eq!(block_ids(&func), vec![0]);
    }

    #[test]
    fn linear_chain_collapses() {
        // B0 → Jump → B1 → Jump → B2 → Return
        let mut func = make_func(vec![
            instr_block(0, 0, 1, IrTerminator::Jump { target: BlockId(1) }),
            instr_block(1, 1, 2, IrTerminator::Jump { target: BlockId(2) }),
            instr_block(
                2,
                2,
                3,
                IrTerminator::Return {
                    value: Some(VarId(2)),
                },
            ),
        ]);
        eliminate(&mut func);
        // All merged into B0.
        assert_eq!(block_ids(&func), vec![0]);
        assert_eq!(func.blocks[0].instructions.len(), 3);
        assert!(matches!(
            func.blocks[0].terminator,
            IrTerminator::Return {
                value: Some(VarId(2))
            }
        ));
    }

    #[test]
    fn conditional_predecessor_not_merged() {
        // B0: BranchIf → B1 / B2 — both have 1 predecessor but via conditional
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
        eliminate(&mut func);
        // Nothing merged — conditional edges are not Jump.
        assert_eq!(block_ids(&func), vec![0, 1, 2]);
    }

    #[test]
    fn multiple_predecessors_not_merged() {
        // B0 → Jump → B2, B1 → Jump → B2 — B2 has 2 predecessors
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(2) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(2) },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        // B1 is dead (no predecessor), but merge_blocks doesn't remove dead blocks.
        // B2 has 2 predecessors (B0 and B1) → not merged.
        eliminate(&mut func);
        assert_eq!(block_ids(&func), vec![0, 1, 2]);
    }

    #[test]
    fn self_loop_not_merged() {
        // B0 → Jump → B0 (self-loop)
        let mut func = make_func(vec![IrBlock {
            id: BlockId(0),
            instructions: vec![],
            terminator: IrTerminator::Jump { target: BlockId(0) },
        }]);
        eliminate(&mut func);
        assert_eq!(block_ids(&func), vec![0]);
    }

    #[test]
    fn entry_block_not_absorbed() {
        // B1 → Jump → B0 (entry) — B0 has 1 predecessor but is entry
        let mut func = IrFunction {
            params: vec![],
            locals: vec![],
            blocks: vec![
                IrBlock {
                    id: BlockId(0),
                    instructions: vec![],
                    terminator: IrTerminator::Return { value: None },
                },
                IrBlock {
                    id: BlockId(1),
                    instructions: vec![],
                    terminator: IrTerminator::Jump { target: BlockId(0) },
                },
            ],
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
            needs_host: false,
        };
        eliminate(&mut func);
        // B0 is entry, must not be absorbed into B1.
        assert!(func.blocks.iter().any(|b| b.id == BlockId(0)));
    }

    // ── Fixed-point iteration ──────────────────────────────────────────

    #[test]
    fn fixed_point_three_block_chain() {
        // B0 → B1 → B2 → B3 → Return
        // Round 1: B1→B0, B3→B2. Round 2: B2→B0.
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(2) },
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
        eliminate(&mut func);
        assert_eq!(block_ids(&func), vec![0]);
        assert!(matches!(
            func.blocks[0].terminator,
            IrTerminator::Return { value: None }
        ));
    }

    // ── Realistic pattern ──────────────────────────────────────────────

    #[test]
    fn jump_then_branch_merges_prologue() {
        // B0 → Jump → B1 → BranchIf(B2, B3)
        // B2: Return, B3: Return
        // B1 has 1 predecessor (B0) via Jump → merge.
        let mut func = make_func(vec![
            instr_block(0, 0, 10, IrTerminator::Jump { target: BlockId(1) }),
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(20),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(0),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
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
        eliminate(&mut func);
        // B1 merged into B0.
        assert_eq!(block_ids(&func), vec![0, 2, 3]);
        assert_eq!(func.blocks[0].instructions.len(), 2);
        assert!(matches!(
            func.blocks[0].terminator,
            IrTerminator::BranchIf { .. }
        ));
    }

    #[test]
    fn loop_back_edge_prevents_merge() {
        // B0 → Jump → B1 → BranchIf(B2, B3)
        // B2 → Jump → B1 (back-edge, B1 now has 2 predecessors)
        // B3: Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(0),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func);
        // B1 has 2 predecessors (B0 and B2) → not merged.
        // No blocks are mergeable.
        assert_eq!(block_ids(&func), vec![0, 1, 2, 3]);
    }
}
