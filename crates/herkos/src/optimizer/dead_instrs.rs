//! Dead instruction elimination.
//!
//! Removes instructions whose destination `VarId` has zero uses across the
//! entire function and whose operation is side-effect-free.
//!
//! ## Algorithm
//!
//! 1. Build the global use-count map (`VarId → number of reads`).
//! 2. For each instruction that produces a value (`instr_dest` returns `Some`):
//!    if the use count is zero **and** the instruction is side-effect-free,
//!    mark it for removal.
//! 3. Remove all marked instructions.
//! 4. Repeat to fixpoint — removing an instruction may make its operands'
//!    definitions unused.
//! 5. Prune dead locals from `IrFunction::locals`.

use super::utils::{build_global_use_count, instr_dest, is_side_effect_free, prune_dead_locals};
use crate::ir::IrFunction;

/// Run dead instruction elimination to fixpoint, then prune dead locals.
pub fn eliminate(func: &mut IrFunction) {
    loop {
        let uses = build_global_use_count(func);
        let mut changed = false;

        for block in &mut func.blocks {
            block.instructions.retain(|instr| {
                if let Some(dest) = instr_dest(instr) {
                    if uses.get(&dest).copied().unwrap_or(0) == 0 && is_side_effect_free(instr) {
                        changed = true;
                        return false; // remove
                    }
                }
                true // keep
            });
        }

        if !changed {
            break;
        }
    }

    prune_dead_locals(func);
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{
        BinOp, BlockId, GlobalIdx, IrBlock, IrFunction, IrInstr, IrTerminator, IrValue,
        MemoryAccessWidth, TypeIdx, VarId, WasmType,
    };

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

    fn make_func_with_locals(blocks: Vec<IrBlock>, locals: Vec<(VarId, WasmType)>) -> IrFunction {
        IrFunction {
            params: vec![],
            locals,
            blocks,
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
            needs_host: false,
        }
    }

    fn single_block(instrs: Vec<IrInstr>, term: IrTerminator) -> Vec<IrBlock> {
        vec![IrBlock {
            id: BlockId(0),
            instructions: instrs,
            terminator: term,
        }]
    }

    fn ret_none() -> IrTerminator {
        IrTerminator::Return { value: None }
    }

    // ── Basic: unused side-effect-free instruction is removed ─────────────

    #[test]
    fn unused_const_removed() {
        let mut func = make_func(single_block(
            vec![IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(42),
            }],
            ret_none(),
        ));
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 0);
    }

    #[test]
    fn unused_binop_removed() {
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(2),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Add,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        // v2 unused → removed; then v0, v1 become unused → removed
        assert_eq!(func.blocks[0].instructions.len(), 0);
    }

    // ── Used instruction is kept ─────────────────────────────────────────

    #[test]
    fn used_const_kept() {
        let mut func = make_func(single_block(
            vec![IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(42),
            }],
            IrTerminator::Return {
                value: Some(VarId(0)),
            },
        ));
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    // ── Side-effectful instructions are kept even when unused ─────────────

    #[test]
    fn unused_load_kept() {
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(0),
                },
                IrInstr::Load {
                    dest: VarId(1),
                    ty: WasmType::I32,
                    addr: VarId(0),
                    offset: 0,
                    width: MemoryAccessWidth::Full,
                    sign: None,
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        // Load may trap → kept; v0 is used by Load → kept
        assert_eq!(func.blocks[0].instructions.len(), 2);
    }

    #[test]
    fn store_kept() {
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(0),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(99),
                },
                IrInstr::Store {
                    ty: WasmType::I32,
                    addr: VarId(0),
                    value: VarId(1),
                    offset: 0,
                    width: MemoryAccessWidth::Full,
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        // Store has side effects → kept; v0, v1 used by Store → kept
        assert_eq!(func.blocks[0].instructions.len(), 3);
    }

    // ── Fixpoint: cascading removal ──────────────────────────────────────

    #[test]
    fn fixpoint_cascading_removal() {
        // v0 = Const(1)
        // v1 = Const(2)
        // v2 = BinOp(v0, v1)    ← only use of v0, v1
        // v3 = BinOp(v2, v2)    ← only use of v2
        // Return(None)           ← v3 unused
        //
        // Round 1: v3 unused → remove v3's BinOp
        // Round 2: v2 unused → remove v2's BinOp
        // Round 3: v0, v1 unused → remove both Consts
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(2),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Add,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
                IrInstr::BinOp {
                    dest: VarId(3),
                    op: BinOp::I32Mul,
                    lhs: VarId(2),
                    rhs: VarId(2),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 0);
    }

    // ── Mixed: some dead, some live ──────────────────────────────────────

    #[test]
    fn mixed_dead_and_live() {
        // v0 = Const(1)       ← used by Return
        // v1 = Const(2)       ← unused → dead
        // v2 = BinOp(v1, v1)  ← unused → dead
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(2),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Add,
                    lhs: VarId(1),
                    rhs: VarId(1),
                },
            ],
            IrTerminator::Return {
                value: Some(VarId(0)),
            },
        ));
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
        match &func.blocks[0].instructions[0] {
            IrInstr::Const {
                dest,
                value: IrValue::I32(1),
            } => assert_eq!(*dest, VarId(0)),
            other => panic!("expected Const(v0, 1), got {other:?}"),
        }
    }

    // ── Dead locals are pruned ───────────────────────────────────────────

    #[test]
    fn dead_locals_pruned() {
        let mut func = make_func_with_locals(
            single_block(
                vec![
                    IrInstr::Const {
                        dest: VarId(0),
                        value: IrValue::I32(1),
                    },
                    IrInstr::Const {
                        dest: VarId(1),
                        value: IrValue::I32(2),
                    },
                ],
                IrTerminator::Return {
                    value: Some(VarId(0)),
                },
            ),
            vec![(VarId(0), WasmType::I32), (VarId(1), WasmType::I32)],
        );
        eliminate(&mut func);
        // v1 is dead → removed from instructions and locals
        assert!(!func.locals.iter().any(|(v, _)| *v == VarId(1)));
        assert!(func.locals.iter().any(|(v, _)| *v == VarId(0)));
    }

    // ── Multi-block: dead in one, live in another ────────────────────────

    #[test]
    fn multi_block_cross_reference_kept() {
        // Block 0: v0 = Const(1); Jump(B1)
        // Block 1: Return(v0)
        // v0 is used in B1 → must NOT be removed from B0
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                }],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Return {
                    value: Some(VarId(0)),
                },
            },
        ]);
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    // ── GlobalGet (side-effect-free) is removed when unused ──────────────

    #[test]
    fn unused_global_get_removed() {
        let mut func = make_func(single_block(
            vec![IrInstr::GlobalGet {
                dest: VarId(0),
                index: GlobalIdx::new(0),
            }],
            ret_none(),
        ));
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 0);
    }

    // ── No-op: empty function ────────────────────────────────────────────

    #[test]
    fn empty_function_unchanged() {
        let mut func = make_func(single_block(vec![], ret_none()));
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 0);
    }
}
