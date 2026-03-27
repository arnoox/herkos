//! Branch condition folding.
//!
//! Simplifies `BranchIf` terminators by looking at the instruction that
//! defines the condition variable:
//!
//! - `Eqz(x)` as condition → swap branch targets, use `x` directly
//! - `Ne(x, 0)` as condition → use `x` directly
//! - `Eq(x, 0)` as condition → swap branch targets, use `x` directly
//!
//! After substitution, the defining instruction becomes dead (single use was
//! the branch) and is cleaned up by `dead_instrs`.

use super::utils::{build_global_const_map, build_global_use_count, instr_dest, is_zero};
use crate::ir::{BinOp, IrFunction, IrInstr, IrTerminator, IrValue, UnOp, VarId};
use std::collections::HashMap;

pub fn eliminate(func: &mut IrFunction) {
    loop {
        let global_uses = build_global_use_count(func);
        let global_consts = build_global_const_map(func);
        if !fold_one(func, &global_uses, &global_consts) {
            break;
        }
    }
}

/// Attempt a single branch fold across the function. Returns `true` if a
/// change was made.
fn fold_one(
    func: &mut IrFunction,
    global_uses: &HashMap<VarId, usize>,
    global_consts: &HashMap<VarId, IrValue>,
) -> bool {
    // Build a map of VarId → defining instruction info.
    // We only care about single-use vars defined by Eqz, Ne(x,0), or Eq(x,0).
    let mut var_defs: HashMap<VarId, VarDef> = HashMap::new();

    for block in &func.blocks {
        for instr in &block.instructions {
            if let Some(dest) = instr_dest(instr) {
                match instr {
                    IrInstr::UnOp {
                        op: UnOp::I32Eqz | UnOp::I64Eqz,
                        operand,
                        ..
                    } => {
                        var_defs.insert(dest, VarDef::Eqz(*operand));
                    }
                    IrInstr::BinOp {
                        op: BinOp::I32Ne | BinOp::I64Ne,
                        lhs,
                        rhs,
                        ..
                    } => {
                        if is_zero(*rhs, global_consts) {
                            var_defs.insert(dest, VarDef::NeZero(*lhs));
                        } else if is_zero(*lhs, global_consts) {
                            var_defs.insert(dest, VarDef::NeZero(*rhs));
                        }
                    }
                    IrInstr::BinOp {
                        op: BinOp::I32Eq | BinOp::I64Eq,
                        lhs,
                        rhs,
                        ..
                    } => {
                        if is_zero(*rhs, global_consts) {
                            var_defs.insert(dest, VarDef::EqZero(*lhs));
                        } else if is_zero(*lhs, global_consts) {
                            var_defs.insert(dest, VarDef::EqZero(*rhs));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Now scan terminators for BranchIf with a foldable condition.
    for block in &mut func.blocks {
        let condition = match &block.terminator {
            IrTerminator::BranchIf { condition, .. } => *condition,
            _ => continue,
        };

        // Only fold if the condition has exactly one use (the BranchIf).
        if global_uses.get(&condition).copied().unwrap_or(0) != 1 {
            continue;
        }

        let def = match var_defs.get(&condition) {
            Some(d) => d,
            None => continue,
        };

        match def {
            VarDef::Eqz(inner) | VarDef::EqZero(inner) => {
                // eqz(x) != 0 ≡ x == 0, so swap targets and use x
                if let IrTerminator::BranchIf {
                    condition: cond,
                    if_true,
                    if_false,
                } = &mut block.terminator
                {
                    *cond = *inner;
                    std::mem::swap(if_true, if_false);
                }
                return true;
            }
            VarDef::NeZero(inner) => {
                // ne(x, 0) != 0 ≡ x != 0, so just use x
                if let IrTerminator::BranchIf {
                    condition: cond, ..
                } = &mut block.terminator
                {
                    *cond = *inner;
                }
                return true;
            }
        }
    }

    false
}

#[derive(Clone, Copy)]
enum VarDef {
    Eqz(VarId),
    NeZero(VarId),
    EqZero(VarId),
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{BlockId, IrBlock, TypeIdx};

    fn make_func(blocks: Vec<IrBlock>) -> IrFunction {
        IrFunction {
            params: vec![],
            locals: vec![],
            blocks,
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
        }
    }

    #[test]
    fn eqz_swaps_targets() {
        // v1 = Eqz(v0); BranchIf(v1, B1, B2) → BranchIf(v0, B2, B1)
        let mut func = make_func(vec![IrBlock {
            id: BlockId(0),
            instructions: vec![IrInstr::UnOp {
                dest: VarId(1),
                op: UnOp::I32Eqz,
                operand: VarId(0),
            }],
            terminator: IrTerminator::BranchIf {
                condition: VarId(1),
                if_true: BlockId(1),
                if_false: BlockId(2),
            },
        }]);
        eliminate(&mut func);
        match &func.blocks[0].terminator {
            IrTerminator::BranchIf {
                condition,
                if_true,
                if_false,
            } => {
                assert_eq!(*condition, VarId(0));
                assert_eq!(*if_true, BlockId(2), "targets should be swapped");
                assert_eq!(*if_false, BlockId(1));
            }
            other => panic!("expected BranchIf, got {other:?}"),
        }
    }

    #[test]
    fn ne_zero_simplifies() {
        // v1 = 0; v2 = Ne(v0, v1); BranchIf(v2, B1, B2) → BranchIf(v0, B1, B2)
        let mut func = make_func(vec![IrBlock {
            id: BlockId(0),
            instructions: vec![
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(0),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Ne,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            terminator: IrTerminator::BranchIf {
                condition: VarId(2),
                if_true: BlockId(1),
                if_false: BlockId(2),
            },
        }]);
        eliminate(&mut func);
        match &func.blocks[0].terminator {
            IrTerminator::BranchIf {
                condition,
                if_true,
                if_false,
            } => {
                assert_eq!(*condition, VarId(0));
                assert_eq!(*if_true, BlockId(1), "targets should NOT be swapped");
                assert_eq!(*if_false, BlockId(2));
            }
            other => panic!("expected BranchIf, got {other:?}"),
        }
    }

    #[test]
    fn eq_zero_swaps() {
        // v1 = 0; v2 = Eq(v0, v1); BranchIf(v2, B1, B2) → BranchIf(v0, B2, B1)
        let mut func = make_func(vec![IrBlock {
            id: BlockId(0),
            instructions: vec![
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(0),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Eq,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            terminator: IrTerminator::BranchIf {
                condition: VarId(2),
                if_true: BlockId(1),
                if_false: BlockId(2),
            },
        }]);
        eliminate(&mut func);
        match &func.blocks[0].terminator {
            IrTerminator::BranchIf {
                condition,
                if_true,
                if_false,
            } => {
                assert_eq!(*condition, VarId(0));
                assert_eq!(*if_true, BlockId(2), "targets should be swapped");
                assert_eq!(*if_false, BlockId(1));
            }
            other => panic!("expected BranchIf, got {other:?}"),
        }
    }

    #[test]
    fn multi_use_not_folded() {
        // v1 = Eqz(v0); use(v1) elsewhere → don't fold
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::UnOp {
                    dest: VarId(1),
                    op: UnOp::I32Eqz,
                    operand: VarId(0),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(1),
                    if_true: BlockId(1),
                    if_false: BlockId(2),
                },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Return {
                    value: Some(VarId(1)), // second use of v1
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func);
        // Should NOT fold — v1 has 2 uses
        match &func.blocks[0].terminator {
            IrTerminator::BranchIf { condition, .. } => {
                assert_eq!(*condition, VarId(1), "should not have been folded");
            }
            other => panic!("expected BranchIf, got {other:?}"),
        }
    }

    #[test]
    fn cross_block_zero_const() {
        // B0: v1 = 0; Jump(B1)
        // B1: v2 = Ne(v0, v1); BranchIf(v2, B2, B3) → BranchIf(v0, B2, B3)
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(0),
                }],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Ne,
                    lhs: VarId(0),
                    rhs: VarId(1),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(2),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
        ]);
        eliminate(&mut func);
        match &func.blocks[1].terminator {
            IrTerminator::BranchIf { condition, .. } => {
                assert_eq!(*condition, VarId(0));
            }
            other => panic!("expected BranchIf, got {other:?}"),
        }
    }
}
