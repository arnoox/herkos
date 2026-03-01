//! Local common subexpression elimination (CSE) via value numbering.
//!
//! Within each block, identifies identical computations and replaces duplicates
//! with references to the first result. Only side-effect-free instructions are
//! considered (`BinOp`, `UnOp`, `Const`). Duplicates are replaced with
//! `Assign { dest, src: previous_result }`, which copy propagation cleans up.

use crate::ir::{BinOp, IrFunction, IrInstr, IrValue, UnOp, VarId};
use std::collections::HashMap;

use super::utils::prune_dead_locals;

// ── Value key ────────────────────────────────────────────────────────────────

/// Hashable representation of a pure computation for deduplication.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ValueKey {
    /// Constant value (using bit-level equality for floats).
    Const(ConstKey),

    /// Binary operation with operand variable IDs.
    BinOp { op: BinOp, lhs: VarId, rhs: VarId },

    /// Unary operation with operand variable ID.
    UnOp { op: UnOp, operand: VarId },
}

/// Bit-level constant key that implements Eq/Hash correctly for floats.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConstKey {
    I32(i32),
    I64(i64),
    F32(u32),
    F64(u64),
}

impl From<IrValue> for ConstKey {
    fn from(v: IrValue) -> Self {
        match v {
            IrValue::I32(x) => ConstKey::I32(x),
            IrValue::I64(x) => ConstKey::I64(x),
            IrValue::F32(x) => ConstKey::F32(x.to_bits()),
            IrValue::F64(x) => ConstKey::F64(x.to_bits()),
        }
    }
}

// ── Commutative op detection ─────────────────────────────────────────────────

/// Returns true for operations where `op(a, b) == op(b, a)`.
fn is_commutative(op: &BinOp) -> bool {
    matches!(
        op,
        BinOp::I32Add
            | BinOp::I32Mul
            | BinOp::I32And
            | BinOp::I32Or
            | BinOp::I32Xor
            | BinOp::I32Eq
            | BinOp::I32Ne
            | BinOp::I64Add
            | BinOp::I64Mul
            | BinOp::I64And
            | BinOp::I64Or
            | BinOp::I64Xor
            | BinOp::I64Eq
            | BinOp::I64Ne
            | BinOp::F32Add
            | BinOp::F32Mul
            | BinOp::F32Eq
            | BinOp::F32Ne
            | BinOp::F64Add
            | BinOp::F64Mul
            | BinOp::F64Eq
            | BinOp::F64Ne
    )
}

/// Build a `ValueKey` for a `BinOp`, normalizing operand order for commutative ops.
fn binop_key(op: BinOp, lhs: VarId, rhs: VarId) -> ValueKey {
    let (lhs, rhs) = if is_commutative(&op) && lhs.0 > rhs.0 {
        (rhs, lhs)
    } else {
        (lhs, rhs)
    };
    ValueKey::BinOp { op, lhs, rhs }
}

// ── Pass entry point ─────────────────────────────────────────────────────────

/// Eliminates common subexpressions within each block of `func`.
pub fn eliminate(func: &mut IrFunction) {
    let mut changed = false;

    for block in &mut func.blocks {
        // Maps a pure computation to the first VarId that computed it.
        let mut value_map: HashMap<ValueKey, VarId> = HashMap::new();

        for instr in &mut block.instructions {
            match instr {
                IrInstr::Const { dest, value } => {
                    let key = ValueKey::Const(ConstKey::from(*value));
                    if let Some(&first) = value_map.get(&key) {
                        *instr = IrInstr::Assign {
                            dest: *dest,
                            src: first,
                        };
                        changed = true;
                    } else {
                        value_map.insert(key, *dest);
                    }
                }

                IrInstr::BinOp {
                    dest, op, lhs, rhs, ..
                } => {
                    let key = binop_key(*op, *lhs, *rhs);
                    if let Some(&first) = value_map.get(&key) {
                        *instr = IrInstr::Assign {
                            dest: *dest,
                            src: first,
                        };
                        changed = true;
                    } else {
                        value_map.insert(key, *dest);
                    }
                }

                IrInstr::UnOp { dest, op, operand } => {
                    let key = ValueKey::UnOp {
                        op: *op,
                        operand: *operand,
                    };
                    if let Some(&first) = value_map.get(&key) {
                        *instr = IrInstr::Assign {
                            dest: *dest,
                            src: first,
                        };
                        changed = true;
                    } else {
                        value_map.insert(key, *dest);
                    }
                }

                // All other instructions are not eligible for CSE.
                _ => {}
            }
        }
    }

    if changed {
        prune_dead_locals(func);
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{BlockId, IrBlock, IrTerminator, TypeIdx, WasmType};

    /// Helper: create a minimal IrFunction with the given blocks.
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

    /// Helper: create a block with given instructions and a simple return terminator.
    fn make_block(id: u32, instructions: Vec<IrInstr>) -> IrBlock {
        IrBlock {
            id: BlockId(id),
            instructions,
            terminator: IrTerminator::Return { value: None },
        }
    }

    #[test]
    fn duplicate_binop_is_eliminated() {
        let instrs = vec![
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            },
            IrInstr::BinOp {
                dest: VarId(3),
                op: BinOp::I32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ];

        let mut func = make_func(vec![make_block(0, instrs)]);
        func.locals = vec![(VarId(2), WasmType::I32), (VarId(3), WasmType::I32)];
        eliminate(&mut func);

        let block = &func.blocks[0];
        assert!(matches!(block.instructions[0], IrInstr::BinOp { .. }));
        assert!(
            matches!(
                block.instructions[1],
                IrInstr::Assign {
                    dest: VarId(3),
                    src: VarId(2)
                }
            ),
            "Duplicate BinOp should be replaced with Assign"
        );
    }

    #[test]
    fn commutative_binop_is_deduplicated() {
        // v2 = v0 + v1, v3 = v1 + v0  →  v3 should become Assign from v2
        let instrs = vec![
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            },
            IrInstr::BinOp {
                dest: VarId(3),
                op: BinOp::I32Add,
                lhs: VarId(1),
                rhs: VarId(0),
            },
        ];

        let mut func = make_func(vec![make_block(0, instrs)]);
        func.locals = vec![(VarId(2), WasmType::I32), (VarId(3), WasmType::I32)];
        eliminate(&mut func);

        assert!(
            matches!(
                func.blocks[0].instructions[1],
                IrInstr::Assign {
                    dest: VarId(3),
                    src: VarId(2)
                }
            ),
            "Commutative BinOp with swapped operands should be deduplicated"
        );
    }

    #[test]
    fn non_commutative_binop_not_deduplicated() {
        // v2 = v0 - v1, v3 = v1 - v0  →  different computations, keep both
        let instrs = vec![
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Sub,
                lhs: VarId(0),
                rhs: VarId(1),
            },
            IrInstr::BinOp {
                dest: VarId(3),
                op: BinOp::I32Sub,
                lhs: VarId(1),
                rhs: VarId(0),
            },
        ];

        let mut func = make_func(vec![make_block(0, instrs)]);
        func.locals = vec![(VarId(2), WasmType::I32), (VarId(3), WasmType::I32)];
        eliminate(&mut func);

        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::BinOp { .. }
        ));
        assert!(
            matches!(func.blocks[0].instructions[1], IrInstr::BinOp { .. }),
            "Non-commutative BinOp with swapped operands should NOT be deduplicated"
        );
    }

    #[test]
    fn duplicate_unop_is_eliminated() {
        let instrs = vec![
            IrInstr::UnOp {
                dest: VarId(1),
                op: UnOp::I32Clz,
                operand: VarId(0),
            },
            IrInstr::UnOp {
                dest: VarId(2),
                op: UnOp::I32Clz,
                operand: VarId(0),
            },
        ];

        let mut func = make_func(vec![make_block(0, instrs)]);
        func.locals = vec![(VarId(1), WasmType::I32), (VarId(2), WasmType::I32)];
        eliminate(&mut func);

        assert!(
            matches!(
                func.blocks[0].instructions[1],
                IrInstr::Assign {
                    dest: VarId(2),
                    src: VarId(1)
                }
            ),
            "Duplicate UnOp should be replaced with Assign"
        );
    }

    #[test]
    fn duplicate_const_is_eliminated() {
        let instrs = vec![
            IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(42),
            },
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(42),
            },
        ];

        let mut func = make_func(vec![make_block(0, instrs)]);
        func.locals = vec![(VarId(0), WasmType::I32), (VarId(1), WasmType::I32)];
        eliminate(&mut func);

        assert!(
            matches!(
                func.blocks[0].instructions[1],
                IrInstr::Assign {
                    dest: VarId(1),
                    src: VarId(0)
                }
            ),
            "Duplicate Const should be replaced with Assign"
        );
    }

    #[test]
    fn float_const_nan_bits_handled() {
        // Two NaN constants with the same bit pattern should be deduplicated.
        let instrs = vec![
            IrInstr::Const {
                dest: VarId(0),
                value: IrValue::F32(f32::NAN),
            },
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::F32(f32::NAN),
            },
        ];

        let mut func = make_func(vec![make_block(0, instrs)]);
        func.locals = vec![(VarId(0), WasmType::F32), (VarId(1), WasmType::F32)];
        eliminate(&mut func);

        assert!(
            matches!(
                func.blocks[0].instructions[1],
                IrInstr::Assign {
                    dest: VarId(1),
                    src: VarId(0)
                }
            ),
            "NaN constants with same bit pattern should be deduplicated"
        );
    }

    #[test]
    fn different_ops_not_deduplicated() {
        let instrs = vec![
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            },
            IrInstr::BinOp {
                dest: VarId(3),
                op: BinOp::I32Sub,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ];

        let mut func = make_func(vec![make_block(0, instrs)]);
        func.locals = vec![(VarId(2), WasmType::I32), (VarId(3), WasmType::I32)];
        eliminate(&mut func);

        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::BinOp { .. }
        ));
        assert!(
            matches!(func.blocks[0].instructions[1], IrInstr::BinOp { .. }),
            "Different operations should not be deduplicated"
        );
    }

    #[test]
    fn cross_block_not_deduplicated() {
        // Each block should have its own value map — no cross-block CSE.
        let block0 = IrBlock {
            id: BlockId(0),
            instructions: vec![IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            }],
            terminator: IrTerminator::Jump { target: BlockId(1) },
        };
        let block1 = IrBlock {
            id: BlockId(1),
            instructions: vec![IrInstr::BinOp {
                dest: VarId(3),
                op: BinOp::I32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            }],
            terminator: IrTerminator::Return { value: None },
        };

        let mut func = make_func(vec![block0, block1]);
        func.locals = vec![(VarId(2), WasmType::I32), (VarId(3), WasmType::I32)];
        eliminate(&mut func);

        // Both should remain as BinOp (no cross-block elimination).
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::BinOp { .. }
        ));
        assert!(
            matches!(func.blocks[1].instructions[0], IrInstr::BinOp { .. }),
            "Cross-block duplicate should NOT be eliminated"
        );
    }

    #[test]
    fn side_effect_instructions_not_eliminated() {
        // Load, Store, Call, etc. should never be CSE'd.
        use crate::ir::MemoryAccessWidth;

        let instrs = vec![
            IrInstr::Load {
                dest: VarId(1),
                ty: WasmType::I32,
                addr: VarId(0),
                offset: 0,
                width: MemoryAccessWidth::Full,
                sign: None,
            },
            IrInstr::Load {
                dest: VarId(2),
                ty: WasmType::I32,
                addr: VarId(0),
                offset: 0,
                width: MemoryAccessWidth::Full,
                sign: None,
            },
        ];

        let mut func = make_func(vec![make_block(0, instrs)]);
        func.locals = vec![(VarId(1), WasmType::I32), (VarId(2), WasmType::I32)];
        eliminate(&mut func);

        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Load { .. }
        ));
        assert!(
            matches!(func.blocks[0].instructions[1], IrInstr::Load { .. }),
            "Load instructions should not be CSE'd"
        );
    }

    #[test]
    fn triple_duplicate_eliminates_both() {
        // Three identical BinOps: second and third should become Assigns to first.
        let instrs = vec![
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Mul,
                lhs: VarId(0),
                rhs: VarId(1),
            },
            IrInstr::BinOp {
                dest: VarId(3),
                op: BinOp::I32Mul,
                lhs: VarId(0),
                rhs: VarId(1),
            },
            IrInstr::BinOp {
                dest: VarId(4),
                op: BinOp::I32Mul,
                lhs: VarId(0),
                rhs: VarId(1),
            },
        ];

        let mut func = make_func(vec![make_block(0, instrs)]);
        func.locals = vec![
            (VarId(2), WasmType::I32),
            (VarId(3), WasmType::I32),
            (VarId(4), WasmType::I32),
        ];
        eliminate(&mut func);

        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::BinOp { .. }
        ));
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Assign {
                dest: VarId(3),
                src: VarId(2)
            }
        ));
        assert!(matches!(
            func.blocks[0].instructions[2],
            IrInstr::Assign {
                dest: VarId(4),
                src: VarId(2)
            }
        ));
    }
}
