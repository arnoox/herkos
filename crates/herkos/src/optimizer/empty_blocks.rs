//! Empty block / passthrough elimination.
//!
//! A passthrough block contains no instructions and ends with an unconditional
//! `Jump`. All references to such a block can be replaced by references to its
//! ultimate target, eliminating the block entirely.
//!
//! Example (from the fibo transpilation):
//!   B5: {} → Jump(B6)
//!   B6: {} → Jump(B7)
//!
//! After this pass B4's branch-false target is rewritten from B5 to B7, and
//! B5/B6 become unreferenced dead blocks, removed by `dead_blocks::eliminate`
//! in the next pass.

use crate::ir::{BlockId, IrFunction, IrTerminator};
use std::collections::HashMap;

/// Replace every reference to a passthrough block with its ultimate target.
///
/// After this call, all passthrough blocks are unreferenced and will be
/// removed by the subsequent `dead_blocks::eliminate` pass.
pub fn eliminate(func: &mut IrFunction) {
    // ── Step 1: Build the raw forwarding map ────────────────────────────
    // A block is a passthrough if it has no instructions and its terminator
    // is an unconditional Jump.
    let mut forward: HashMap<BlockId, BlockId> = HashMap::new();
    for block in &func.blocks {
        if block.instructions.is_empty() {
            if let IrTerminator::Jump { target } = block.terminator {
                forward.insert(block.id, target);
            }
        }
    }

    if forward.is_empty() {
        return;
    }

    // ── Step 2: Resolve chains, cycle-safe ──────────────────────────────
    // Collapse A → B → C chains into A → C.
    // Bound hop count to func.blocks.len() to handle cycles (e.g. A→B→A).
    let max_hops = func.blocks.len();
    let resolved: HashMap<BlockId, BlockId> = forward
        .keys()
        .copied()
        .map(|start| {
            let mut cur = start;
            for _ in 0..max_hops {
                match forward.get(&cur) {
                    Some(&next) => cur = next,
                    None => break,
                }
            }
            (start, cur)
        })
        .collect();

    // ── Step 3: Rewrite all terminator targets ───────────────────────────
    let fwd = |id: BlockId| resolved.get(&id).copied().unwrap_or(id);

    for block in &mut func.blocks {
        match &mut block.terminator {
            IrTerminator::Jump { target } => {
                *target = fwd(*target);
            }
            IrTerminator::BranchIf {
                if_true, if_false, ..
            } => {
                *if_true = fwd(*if_true);
                *if_false = fwd(*if_false);
            }
            IrTerminator::BranchTable {
                targets, default, ..
            } => {
                for t in targets.iter_mut() {
                    *t = fwd(*t);
                }
                *default = fwd(*default);
            }
            IrTerminator::Return { .. } | IrTerminator::Unreachable => {}
        }
    }
    // Passthrough blocks are now unreferenced; dead_blocks::eliminate will
    // remove them in the next pass.
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

    fn jump(id: u32, target: u32) -> IrBlock {
        IrBlock {
            id: BlockId(id),
            instructions: vec![],
            terminator: IrTerminator::Jump {
                target: BlockId(target),
            },
        }
    }

    fn ret(id: u32) -> IrBlock {
        IrBlock {
            id: BlockId(id),
            instructions: vec![],
            terminator: IrTerminator::Return { value: None },
        }
    }

    fn branch(id: u32, cond: u32, if_true: u32, if_false: u32) -> IrBlock {
        IrBlock {
            id: BlockId(id),
            instructions: vec![],
            terminator: IrTerminator::BranchIf {
                condition: VarId(cond),
                if_true: BlockId(if_true),
                if_false: BlockId(if_false),
            },
        }
    }

    fn target_of(func: &IrFunction, id: u32) -> Option<BlockId> {
        func.blocks
            .iter()
            .find(|b| b.id == BlockId(id))
            .and_then(|b| match b.terminator {
                IrTerminator::Jump { target } => Some(target),
                _ => None,
            })
    }

    fn branch_targets(func: &IrFunction, id: u32) -> Option<(BlockId, BlockId)> {
        func.blocks
            .iter()
            .find(|b| b.id == BlockId(id))
            .and_then(|b| match b.terminator {
                IrTerminator::BranchIf {
                    if_true, if_false, ..
                } => Some((if_true, if_false)),
                _ => None,
            })
    }

    // ── Basic cases ──────────────────────────────────────────────────────

    #[test]
    fn no_passthrough_unchanged() {
        // B0: instr → Jump(B1), B1: Return — no passthrough, nothing changes
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                }],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            ret(1),
        ]);
        eliminate(&mut func);
        assert_eq!(target_of(&func, 0), Some(BlockId(1)));
        assert_eq!(func.blocks.len(), 2);
    }

    #[test]
    fn single_passthrough_redirected() {
        // B0 → B1(pass) → B2: B0's target becomes B2
        let mut func = make_func(vec![jump(0, 1), jump(1, 2), ret(2)]);
        eliminate(&mut func);
        assert_eq!(target_of(&func, 0), Some(BlockId(2)));
    }

    #[test]
    fn chain_collapsed() {
        // B0 → B1(pass) → B2(pass) → B3: B0's target becomes B3
        let mut func = make_func(vec![jump(0, 1), jump(1, 2), jump(2, 3), ret(3)]);
        eliminate(&mut func);
        assert_eq!(target_of(&func, 0), Some(BlockId(3)));
        // B1 should also forward to B3
        assert_eq!(target_of(&func, 1), Some(BlockId(3)));
    }

    // ── BranchIf ────────────────────────────────────────────────────────

    #[test]
    fn branch_if_both_arms_redirected() {
        // B0: BranchIf(true→B1(pass)→B3, false→B2(pass)→B4)
        let mut func = make_func(vec![
            branch(0, 0, 1, 2),
            jump(1, 3),
            jump(2, 4),
            ret(3),
            ret(4),
        ]);
        eliminate(&mut func);
        let (t, f) = branch_targets(&func, 0).unwrap();
        assert_eq!(t, BlockId(3));
        assert_eq!(f, BlockId(4));
    }

    #[test]
    fn branch_if_one_arm_redirected() {
        // B0: BranchIf(true→B1(non-pass), false→B2(pass)→B3)
        let mut func = make_func(vec![
            branch(0, 0, 1, 2),
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(42),
                }],
                terminator: IrTerminator::Return { value: None },
            },
            jump(2, 3),
            ret(3),
        ]);
        eliminate(&mut func);
        let (t, f) = branch_targets(&func, 0).unwrap();
        assert_eq!(t, BlockId(1)); // unchanged
        assert_eq!(f, BlockId(3)); // forwarded
    }

    // ── BranchTable ──────────────────────────────────────────────────────

    #[test]
    fn branch_table_redirected() {
        // B0: BranchTable(targets:[B1(pass)→B3, B2(pass)→B4], default:B5)
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::BranchTable {
                    index: VarId(0),
                    targets: vec![BlockId(1), BlockId(2)],
                    default: BlockId(5),
                },
            },
            jump(1, 3),
            jump(2, 4),
            ret(3),
            ret(4),
            ret(5),
        ]);
        eliminate(&mut func);
        let b = func.blocks.iter().find(|b| b.id == BlockId(0)).unwrap();
        match &b.terminator {
            IrTerminator::BranchTable {
                targets, default, ..
            } => {
                assert_eq!(targets[0], BlockId(3));
                assert_eq!(targets[1], BlockId(4));
                assert_eq!(*default, BlockId(5)); // non-passthrough, unchanged
            }
            _ => panic!("expected BranchTable"),
        }
    }

    // ── Edge cases ───────────────────────────────────────────────────────

    #[test]
    fn cycle_safe() {
        // B0 → B1(pass) → B2(pass) → B1 (cycle)
        // Should not infinite loop; B0 ends up pointing somewhere in the cycle
        let mut func = make_func(vec![jump(0, 1), jump(1, 2), jump(2, 1)]);
        // Must complete without hanging; exact target is unspecified for cycles
        eliminate(&mut func);
    }

    #[test]
    fn entry_passthrough_not_removed() {
        // Entry block B0 is itself a passthrough: B0(pass) → B1 → Return
        // After pass B0's jump stays (it's a passthrough of a passthrough pointing at B1),
        // dead_blocks won't remove B0 (it starts BFS from entry).
        let mut func = make_func(vec![jump(0, 1), ret(1)]);
        eliminate(&mut func);
        // B0 is a passthrough pointing to B1; resolve(B0)=B1 but nobody *jumps to* B0,
        // so B0's own terminator remains Jump(B1) (forwarded to itself, i.e. B1).
        assert_eq!(target_of(&func, 0), Some(BlockId(1)));
        assert_eq!(func.blocks.len(), 2); // dead_blocks not called here, both still present
    }

    // ── Realistic fibo pattern ────────────────────────────────────────────

    #[test]
    fn fibo_pattern() {
        // Mirrors the B3/B4/B5/B6/B7 structure from func_7 (release build):
        //   B3: BranchIf(cond→B7, else→B4)
        //   B4: BranchIf(cond→B3, else→B5)
        //   B5: {} → Jump(B6)    ← passthrough
        //   B6: {} → Jump(B7)    ← passthrough
        //   B7: Return
        //
        // After eliminate():
        //   B4's false-arm should be B7 (not B5)
        //   B3 and B7 are unchanged
        let mut func = make_func(vec![
            branch(3, 0, 7, 4),
            branch(4, 1, 3, 5),
            jump(5, 6), // passthrough
            jump(6, 7), // passthrough
            ret(7),
        ]);
        func.entry_block = BlockId(3);

        eliminate(&mut func);

        // B4's false-arm: was B5, must now be B7
        let (true_arm, false_arm) = branch_targets(&func, 4).unwrap();
        assert_eq!(true_arm, BlockId(3)); // back-edge unchanged
        assert_eq!(false_arm, BlockId(7)); // forwarded through B5→B6→B7

        // B3's true-arm was already B7 — still B7
        let (t3, f3) = branch_targets(&func, 3).unwrap();
        assert_eq!(t3, BlockId(7));
        assert_eq!(f3, BlockId(4));

        // B5 and B6 themselves now point to B7 (resolved)
        assert_eq!(target_of(&func, 5), Some(BlockId(7)));
        assert_eq!(target_of(&func, 6), Some(BlockId(7)));
    }
}
