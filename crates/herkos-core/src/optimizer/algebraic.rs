//! Algebraic simplifications.
//!
//! Rewrites `BinOp` instructions when one operand is a known constant and an
//! identity or annihilator rule applies. Runs after `const_prop` so that
//! constant operands are already resolved.
//!
//! ## Rules
//!
//! | Pattern             | Result       |
//! |---------------------|--------------|
//! | `x + 0`, `0 + x`   | `x`          |
//! | `x - 0`            | `x`          |
//! | `x * 1`, `1 * x`   | `x`          |
//! | `x * 0`, `0 * x`   | `0`          |
//! | `x & 0`            | `0`          |
//! | `x & -1`           | `x`          |
//! | `x | 0`            | `x`          |
//! | `x | -1`           | `-1`         |
//! | `x ^ 0`            | `x`          |
//! | `x ^ x`            | `0`          |
//! | `x << 0`, `x >> 0` | `x`          |
//! | `x == x`           | `1`          |
//! | `x != x`           | `0`          |

use crate::{
    ir::{BinOp, IrFunction, IrInstr, IrValue, VarId},
    optimizer::utils::build_global_const_map,
};

// ── Public entry point ────────────────────────────────────────────────────────

/// Eliminates algebraic identities and annihilators until reaching a fixed point.
///
/// Runs multiple passes because simplifications from one pass can expose new
/// optimization opportunities. For example, `(x * 1) + 0` requires two passes:
/// first `x * 1 → x`, then `x + 0 → x`.
pub fn eliminate(func: &mut IrFunction) {
    // Run passes until reaching a fixed point (no more changes)
    loop {
        if !eliminate_once(func) {
            break;
        }
    }
}

fn eliminate_once(func: &mut IrFunction) -> bool {
    let global_consts = build_global_const_map(func);
    let mut changed = false;

    for block in &mut func.blocks {
        let mut local_consts = global_consts.clone();

        for instr in &mut block.instructions {
            // Track constants defined in this block. We walk forward and update
            // local_consts as we encounter Const instructions, so that later BinOp
            // instructions in the same block can see newly-defined constants.
            // (global_consts only captures constants with a single def across the
            // entire function, not block-local defs discovered during forward walk.)
            if let IrInstr::Const { dest, value } = instr {
                local_consts.insert(*dest, *value);
                continue;
            }

            let (dest, op, lhs, rhs) = match instr {
                IrInstr::BinOp { dest, op, lhs, rhs } => (*dest, *op, *lhs, *rhs),
                _ => continue,
            };

            // Same-operand rules (no constant needed).
            if lhs == rhs {
                if let Some(replacement) = same_operand_rule(op, dest, lhs) {
                    *instr = replacement;
                    changed = true;
                    if let IrInstr::Const { dest, value } = instr {
                        local_consts.insert(*dest, *value);
                    }
                    continue;
                }
            }

            let lhs_val = local_consts.get(&lhs).copied();
            let rhs_val = local_consts.get(&rhs).copied();

            if let Some(replacement) = constant_operand_rule(op, dest, lhs, rhs, lhs_val, rhs_val) {
                *instr = replacement;
                changed = true;
                if let IrInstr::Const { dest, value } = instr {
                    local_consts.insert(*dest, *value);
                }
            }
        }
    }

    changed
}

// ── Same-operand rules ────────────────────────────────────────────────────────

fn same_operand_rule(op: BinOp, dest: VarId, operand: VarId) -> Option<IrInstr> {
    match op {
        // x ^ x → 0
        BinOp::I32Xor => Some(IrInstr::Const {
            dest,
            value: IrValue::I32(0),
        }),
        BinOp::I64Xor => Some(IrInstr::Const {
            dest,
            value: IrValue::I64(0),
        }),

        // x == x → 1 (integers only; floats have NaN)
        BinOp::I32Eq
        | BinOp::I32LeS
        | BinOp::I32LeU
        | BinOp::I32GeS
        | BinOp::I32GeU
        | BinOp::I64Eq
        | BinOp::I64LeS
        | BinOp::I64LeU
        | BinOp::I64GeS
        | BinOp::I64GeU => Some(IrInstr::Const {
            dest,
            value: IrValue::I32(1),
        }),

        // x != x → 0 (integers only)
        BinOp::I32Ne
        | BinOp::I32LtS
        | BinOp::I32LtU
        | BinOp::I32GtS
        | BinOp::I32GtU
        | BinOp::I64Ne
        | BinOp::I64LtS
        | BinOp::I64LtU
        | BinOp::I64GtS
        | BinOp::I64GtU => Some(IrInstr::Const {
            dest,
            value: IrValue::I32(0),
        }),

        // x - x → 0 (integers only; floats have Inf - Inf = NaN)
        BinOp::I32Sub => Some(IrInstr::Const {
            dest,
            value: IrValue::I32(0),
        }),
        BinOp::I64Sub => Some(IrInstr::Const {
            dest,
            value: IrValue::I64(0),
        }),

        // x & x → x, x | x → x
        BinOp::I32And | BinOp::I32Or | BinOp::I64And | BinOp::I64Or => {
            Some(IrInstr::Assign { dest, src: operand })
        }

        _ => None,
    }
}

// ── Constant-operand rules ────────────────────────────────────────────────────

fn constant_operand_rule(
    op: BinOp,
    dest: VarId,
    lhs: VarId,
    rhs: VarId,
    lhs_val: Option<IrValue>,
    rhs_val: Option<IrValue>,
) -> Option<IrInstr> {
    match op {
        // ── Add ──────────────────────────────────────────────────────────
        BinOp::I32Add => match (lhs_val, rhs_val) {
            (_, Some(IrValue::I32(0))) => Some(IrInstr::Assign { dest, src: lhs }),
            (Some(IrValue::I32(0)), _) => Some(IrInstr::Assign { dest, src: rhs }),
            _ => None,
        },
        BinOp::I64Add => match (lhs_val, rhs_val) {
            (_, Some(IrValue::I64(0))) => Some(IrInstr::Assign { dest, src: lhs }),
            (Some(IrValue::I64(0)), _) => Some(IrInstr::Assign { dest, src: rhs }),
            _ => None,
        },

        // ── Sub ──────────────────────────────────────────────────────────
        BinOp::I32Sub => match rhs_val {
            Some(IrValue::I32(0)) => Some(IrInstr::Assign { dest, src: lhs }),
            _ => None,
        },
        BinOp::I64Sub => match rhs_val {
            Some(IrValue::I64(0)) => Some(IrInstr::Assign { dest, src: lhs }),
            _ => None,
        },

        // ── Mul ──────────────────────────────────────────────────────────
        BinOp::I32Mul => match (lhs_val, rhs_val) {
            (_, Some(IrValue::I32(1))) => Some(IrInstr::Assign { dest, src: lhs }),
            (Some(IrValue::I32(1)), _) => Some(IrInstr::Assign { dest, src: rhs }),
            (_, Some(IrValue::I32(0))) | (Some(IrValue::I32(0)), _) => Some(IrInstr::Const {
                dest,
                value: IrValue::I32(0),
            }),
            _ => None,
        },
        BinOp::I64Mul => match (lhs_val, rhs_val) {
            (_, Some(IrValue::I64(1))) => Some(IrInstr::Assign { dest, src: lhs }),
            (Some(IrValue::I64(1)), _) => Some(IrInstr::Assign { dest, src: rhs }),
            (_, Some(IrValue::I64(0))) => Some(IrInstr::Const {
                dest,
                value: IrValue::I64(0),
            }),
            (Some(IrValue::I64(0)), _) => Some(IrInstr::Const {
                dest,
                value: IrValue::I64(0),
            }),
            _ => None,
        },

        // ── And ──────────────────────────────────────────────────────────
        BinOp::I32And => match (lhs_val, rhs_val) {
            (_, Some(IrValue::I32(0))) | (Some(IrValue::I32(0)), _) => Some(IrInstr::Const {
                dest,
                value: IrValue::I32(0),
            }),
            (_, Some(IrValue::I32(-1))) => Some(IrInstr::Assign { dest, src: lhs }),
            (Some(IrValue::I32(-1)), _) => Some(IrInstr::Assign { dest, src: rhs }),
            _ => None,
        },
        BinOp::I64And => match (lhs_val, rhs_val) {
            (_, Some(IrValue::I64(0))) | (Some(IrValue::I64(0)), _) => Some(IrInstr::Const {
                dest,
                value: IrValue::I64(0),
            }),
            (_, Some(IrValue::I64(-1))) => Some(IrInstr::Assign { dest, src: lhs }),
            (Some(IrValue::I64(-1)), _) => Some(IrInstr::Assign { dest, src: rhs }),
            _ => None,
        },

        // ── Or ───────────────────────────────────────────────────────────
        BinOp::I32Or => match (lhs_val, rhs_val) {
            (_, Some(IrValue::I32(0))) => Some(IrInstr::Assign { dest, src: lhs }),
            (Some(IrValue::I32(0)), _) => Some(IrInstr::Assign { dest, src: rhs }),
            (_, Some(IrValue::I32(-1))) | (Some(IrValue::I32(-1)), _) => Some(IrInstr::Const {
                dest,
                value: IrValue::I32(-1),
            }),
            _ => None,
        },
        BinOp::I64Or => match (lhs_val, rhs_val) {
            (_, Some(IrValue::I64(0))) => Some(IrInstr::Assign { dest, src: lhs }),
            (Some(IrValue::I64(0)), _) => Some(IrInstr::Assign { dest, src: rhs }),
            (_, Some(IrValue::I64(-1))) | (Some(IrValue::I64(-1)), _) => Some(IrInstr::Const {
                dest,
                value: IrValue::I64(-1),
            }),
            _ => None,
        },

        // ── Xor ──────────────────────────────────────────────────────────
        BinOp::I32Xor => match (lhs_val, rhs_val) {
            (_, Some(IrValue::I32(0))) => Some(IrInstr::Assign { dest, src: lhs }),
            (Some(IrValue::I32(0)), _) => Some(IrInstr::Assign { dest, src: rhs }),
            _ => None,
        },
        BinOp::I64Xor => match (lhs_val, rhs_val) {
            (_, Some(IrValue::I64(0))) => Some(IrInstr::Assign { dest, src: lhs }),
            (Some(IrValue::I64(0)), _) => Some(IrInstr::Assign { dest, src: rhs }),
            _ => None,
        },

        // ── Shifts / Rotates ─────────────────────────────────────────────
        BinOp::I32Shl | BinOp::I32ShrS | BinOp::I32ShrU | BinOp::I32Rotl | BinOp::I32Rotr => {
            match rhs_val {
                Some(IrValue::I32(0)) => Some(IrInstr::Assign { dest, src: lhs }),
                _ => None,
            }
        }
        BinOp::I64Shl | BinOp::I64ShrS | BinOp::I64ShrU | BinOp::I64Rotl | BinOp::I64Rotr => {
            match rhs_val {
                Some(IrValue::I64(0)) => Some(IrInstr::Assign { dest, src: lhs }),
                _ => None,
            }
        }

        _ => None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{BlockId, IrBlock, IrTerminator, TypeIdx};

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

    fn single_block(instrs: Vec<IrInstr>) -> Vec<IrBlock> {
        vec![IrBlock {
            id: BlockId(0),
            instructions: instrs,
            terminator: IrTerminator::Return { value: None },
        }]
    }

    // ── Additive identity ────────────────────────────────────────────────

    #[test]
    fn add_zero_rhs() {
        // v1 = 0; v2 = v0 + v1 → v2 = Assign(v0)
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(0),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Assign {
                dest: VarId(2),
                src: VarId(0)
            }
        ));
    }

    #[test]
    fn add_zero_lhs() {
        // v1 = 0; v2 = v1 + v0 → v2 = Assign(v0)
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(0),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Add,
                lhs: VarId(1),
                rhs: VarId(0),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Assign {
                dest: VarId(2),
                src: VarId(0)
            }
        ));
    }

    // ── Multiplicative identity ──────────────────────────────────────────

    #[test]
    fn mul_one() {
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(1),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Mul,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Assign {
                dest: VarId(2),
                src: VarId(0)
            }
        ));
    }

    #[test]
    fn mul_zero() {
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(0),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Mul,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Const {
                dest: VarId(2),
                value: IrValue::I32(0)
            }
        ));
    }

    // ── XOR same operand ─────────────────────────────────────────────────

    #[test]
    fn xor_self() {
        let mut func = make_func(single_block(vec![IrInstr::BinOp {
            dest: VarId(1),
            op: BinOp::I32Xor,
            lhs: VarId(0),
            rhs: VarId(0),
        }]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(0)
            }
        ));
    }

    // ── Equality same operand ────────────────────────────────────────────

    #[test]
    fn eq_self() {
        let mut func = make_func(single_block(vec![IrInstr::BinOp {
            dest: VarId(1),
            op: BinOp::I32Eq,
            lhs: VarId(0),
            rhs: VarId(0),
        }]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(1)
            }
        ));
    }

    #[test]
    fn ne_self() {
        let mut func = make_func(single_block(vec![IrInstr::BinOp {
            dest: VarId(1),
            op: BinOp::I32Ne,
            lhs: VarId(0),
            rhs: VarId(0),
        }]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(0)
            }
        ));
    }

    // ── AND / OR with constants ──────────────────────────────────────────

    #[test]
    fn and_zero() {
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(0),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32And,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Const {
                dest: VarId(2),
                value: IrValue::I32(0)
            }
        ));
    }

    #[test]
    fn and_all_ones() {
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(-1),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32And,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Assign {
                dest: VarId(2),
                src: VarId(0)
            }
        ));
    }

    #[test]
    fn or_all_ones() {
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(-1),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Or,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Const {
                dest: VarId(2),
                value: IrValue::I32(-1)
            }
        ));
    }

    // ── Shift by zero ────────────────────────────────────────────────────

    #[test]
    fn shl_zero() {
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(0),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Shl,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Assign {
                dest: VarId(2),
                src: VarId(0)
            }
        ));
    }

    // ── Sub self ─────────────────────────────────────────────────────────

    #[test]
    fn sub_self() {
        let mut func = make_func(single_block(vec![IrInstr::BinOp {
            dest: VarId(1),
            op: BinOp::I32Sub,
            lhs: VarId(0),
            rhs: VarId(0),
        }]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(0)
            }
        ));
    }

    // ── Sub zero ─────────────────────────────────────────────────────────

    #[test]
    fn sub_zero() {
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(0),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Sub,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Assign {
                dest: VarId(2),
                src: VarId(0)
            }
        ));
    }

    // ── i64 variants ─────────────────────────────────────────────────────

    #[test]
    fn i64_add_zero() {
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I64(0),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I64Add,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Assign {
                dest: VarId(2),
                src: VarId(0)
            }
        ));
    }

    #[test]
    fn i64_xor_self() {
        let mut func = make_func(single_block(vec![IrInstr::BinOp {
            dest: VarId(1),
            op: BinOp::I64Xor,
            lhs: VarId(0),
            rhs: VarId(0),
        }]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I64(0)
            }
        ));
    }

    // ── Cross-block constant ─────────────────────────────────────────────

    #[test]
    fn cross_block_const_simplification() {
        // B0: v1 = 0  →  B1: v2 = v0 + v1 → v2 = Assign(v0)
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
                    op: BinOp::I32Add,
                    lhs: VarId(0),
                    rhs: VarId(1),
                }],
                terminator: IrTerminator::Return {
                    value: Some(VarId(2)),
                },
            },
        ]);
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[1].instructions[0],
            IrInstr::Assign {
                dest: VarId(2),
                src: VarId(0)
            }
        ));
    }

    // ── No-op: non-identity constant unchanged ───────────────────────────

    #[test]
    fn add_nonzero_unchanged() {
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(5),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::BinOp { .. }
        ));
    }

    // ── Float ops are NOT simplified (NaN concerns) ──────────────────────

    #[test]
    fn float_add_zero_unchanged() {
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::F32(0.0),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::F32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ]));
        eliminate(&mut func);
        // Float add with 0 is NOT simplified because -0.0 + 0.0 = 0.0 ≠ -0.0
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::BinOp { .. }
        ));
    }

    // ── AND/OR self ──────────────────────────────────────────────────────

    #[test]
    fn and_self() {
        let mut func = make_func(single_block(vec![IrInstr::BinOp {
            dest: VarId(1),
            op: BinOp::I32And,
            lhs: VarId(0),
            rhs: VarId(0),
        }]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Assign {
                dest: VarId(1),
                src: VarId(0)
            }
        ));
    }

    #[test]
    fn or_self() {
        let mut func = make_func(single_block(vec![IrInstr::BinOp {
            dest: VarId(1),
            op: BinOp::I32Or,
            lhs: VarId(0),
            rhs: VarId(0),
        }]));
        eliminate(&mut func);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Assign {
                dest: VarId(1),
                src: VarId(0)
            }
        ));
    }

    // ── Cascading optimizations (multiple passes needed) ───────────────────

    #[test]
    fn cascading_mul_one_and_add_zero() {
        // v1 = 1; v2 = v0 * v1; v3 = 0; v4 = v2 + v3
        // Pass 1: v2 = v0 * 1 → v2 = Assign(v0)
        //         v4 = v2 + 0 → NOT YET (v2 is not recognized as const)
        // Pass 2: v4 = v2 + 0 → v4 = Assign(v2) → v4 = Assign(v0)
        let mut func = make_func(single_block(vec![
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(1),
            },
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Mul,
                lhs: VarId(0),
                rhs: VarId(1),
            },
            IrInstr::Const {
                dest: VarId(3),
                value: IrValue::I32(0),
            },
            IrInstr::BinOp {
                dest: VarId(4),
                op: BinOp::I32Add,
                lhs: VarId(2),
                rhs: VarId(3),
            },
        ]));
        eliminate(&mut func);
        // After multiple passes, both simplifications should be applied.
        // v2 should be Assign(v0) and v4 should be Assign(v0) (through v2).
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Assign {
                dest: VarId(2),
                src: VarId(0)
            }
        ));
        assert!(matches!(
            func.blocks[0].instructions[3],
            IrInstr::Assign {
                dest: VarId(4),
                src: VarId(2)
            }
        ));
    }
}
