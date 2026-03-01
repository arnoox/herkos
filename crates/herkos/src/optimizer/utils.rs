//! Shared utility functions for IR optimization passes.
//!
//! Provides common operations on IR instructions, terminators, and control flow
//! that are needed by multiple optimization passes.
//!
//! Some functions are not yet used by existing passes but are provided for
//! upcoming optimization passes (const_prop, dead_instrs, local_cse, licm).
#![allow(dead_code)]

use crate::ir::{BlockId, IrFunction, IrInstr, IrTerminator, VarId};
use std::collections::{HashMap, HashSet};

// ── Terminator successors ────────────────────────────────────────────────────

/// Returns the successor block IDs for a terminator.
pub fn terminator_successors(term: &IrTerminator) -> Vec<BlockId> {
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

// ── Predecessor map ──────────────────────────────────────────────────────────

/// Build a map from each block ID to the set of *distinct* predecessor block IDs.
pub fn build_predecessors(func: &IrFunction) -> HashMap<BlockId, HashSet<BlockId>> {
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

// ── Instruction variable traversal ───────────────────────────────────────────

/// Calls `f` with every variable read by `instr`.
pub fn for_each_use<F: FnMut(VarId)>(instr: &IrInstr, mut f: F) {
    match instr {
        IrInstr::Const { .. } => {}
        IrInstr::BinOp { lhs, rhs, .. } => {
            f(*lhs);
            f(*rhs);
        }
        IrInstr::UnOp { operand, .. } => {
            f(*operand);
        }
        IrInstr::Load { addr, .. } => {
            f(*addr);
        }
        IrInstr::Store { addr, value, .. } => {
            f(*addr);
            f(*value);
        }
        IrInstr::Call { args, .. } => {
            for a in args {
                f(*a);
            }
        }
        IrInstr::CallImport { args, .. } => {
            for a in args {
                f(*a);
            }
        }
        IrInstr::CallIndirect {
            table_idx, args, ..
        } => {
            f(*table_idx);
            for a in args {
                f(*a);
            }
        }
        IrInstr::Assign { src, .. } => {
            f(*src);
        }
        IrInstr::GlobalGet { .. } => {}
        IrInstr::GlobalSet { value, .. } => {
            f(*value);
        }
        IrInstr::MemorySize { .. } => {}
        IrInstr::MemoryGrow { delta, .. } => {
            f(*delta);
        }
        IrInstr::MemoryCopy { dst, src, len } => {
            f(*dst);
            f(*src);
            f(*len);
        }
        IrInstr::Select {
            val1,
            val2,
            condition,
            ..
        } => {
            f(*val1);
            f(*val2);
            f(*condition);
        }
    }
}

/// Calls `f` with every variable read by a block terminator.
pub fn for_each_use_terminator<F: FnMut(VarId)>(term: &IrTerminator, mut f: F) {
    match term {
        IrTerminator::Return { value: Some(v) } => {
            f(*v);
        }
        IrTerminator::Return { value: None }
        | IrTerminator::Jump { .. }
        | IrTerminator::Unreachable => {}
        IrTerminator::BranchIf { condition, .. } => {
            f(*condition);
        }
        IrTerminator::BranchTable { index, .. } => {
            f(*index);
        }
    }
}

// ── Instruction destination ──────────────────────────────────────────────────

/// Returns the variable written by `instr`, or `None` for side-effect-only instructions.
pub fn instr_dest(instr: &IrInstr) -> Option<VarId> {
    match instr {
        IrInstr::Const { dest, .. }
        | IrInstr::BinOp { dest, .. }
        | IrInstr::UnOp { dest, .. }
        | IrInstr::Load { dest, .. }
        | IrInstr::Assign { dest, .. }
        | IrInstr::GlobalGet { dest, .. }
        | IrInstr::MemorySize { dest }
        | IrInstr::MemoryGrow { dest, .. }
        | IrInstr::Select { dest, .. } => Some(*dest),

        IrInstr::Call { dest, .. }
        | IrInstr::CallImport { dest, .. }
        | IrInstr::CallIndirect { dest, .. } => *dest,

        IrInstr::Store { .. } | IrInstr::GlobalSet { .. } | IrInstr::MemoryCopy { .. } => None,
    }
}

/// Redirects the destination variable of `instr` to `new_dest`.
///
/// Only called when `instr_dest(instr)` is `Some(_)`, i.e. the instruction
/// produces a value.  Instructions without a dest are left unchanged.
pub fn set_instr_dest(instr: &mut IrInstr, new_dest: VarId) {
    match instr {
        IrInstr::Const { dest, .. }
        | IrInstr::BinOp { dest, .. }
        | IrInstr::UnOp { dest, .. }
        | IrInstr::Load { dest, .. }
        | IrInstr::Assign { dest, .. }
        | IrInstr::GlobalGet { dest, .. }
        | IrInstr::MemorySize { dest }
        | IrInstr::MemoryGrow { dest, .. }
        | IrInstr::Select { dest, .. } => {
            *dest = new_dest;
        }
        IrInstr::Call { dest, .. }
        | IrInstr::CallImport { dest, .. }
        | IrInstr::CallIndirect { dest, .. } => {
            *dest = Some(new_dest);
        }
        // No dest — unreachable given precondition, but harmless to ignore.
        IrInstr::Store { .. } | IrInstr::GlobalSet { .. } | IrInstr::MemoryCopy { .. } => {}
    }
}

// ── Use-count helpers ────────────────────────────────────────────────────────

/// Count how many times `var` appears as an operand (read) in `instr`.
pub fn count_uses_of(instr: &IrInstr, var: VarId) -> usize {
    let mut count = 0usize;
    for_each_use(instr, |v| {
        if v == var {
            count += 1;
        }
    });
    count
}

/// Count how many times `var` appears as an operand in `term`.
pub fn count_uses_of_terminator(term: &IrTerminator, var: VarId) -> usize {
    let mut count = 0usize;
    for_each_use_terminator(term, |v| {
        if v == var {
            count += 1;
        }
    });
    count
}

// ── Use-replacement helpers ──────────────────────────────────────────────────

/// Replace every read-occurrence of `old` with `new` in `instr`.
/// Only touches operand (source) slots; the destination slot is never modified.
pub fn replace_uses_of(instr: &mut IrInstr, old: VarId, new: VarId) {
    let sub = |v: &mut VarId| {
        if *v == old {
            *v = new;
        }
    };
    match instr {
        IrInstr::Const { .. } => {}
        IrInstr::BinOp { lhs, rhs, .. } => {
            sub(lhs);
            sub(rhs);
        }
        IrInstr::UnOp { operand, .. } => {
            sub(operand);
        }
        IrInstr::Load { addr, .. } => {
            sub(addr);
        }
        IrInstr::Store { addr, value, .. } => {
            sub(addr);
            sub(value);
        }
        IrInstr::Call { args, .. } | IrInstr::CallImport { args, .. } => {
            for a in args {
                sub(a);
            }
        }
        IrInstr::CallIndirect {
            table_idx, args, ..
        } => {
            sub(table_idx);
            for a in args {
                sub(a);
            }
        }
        IrInstr::Assign { src, .. } => {
            sub(src);
        }
        IrInstr::GlobalGet { .. } => {}
        IrInstr::GlobalSet { value, .. } => {
            sub(value);
        }
        IrInstr::MemorySize { .. } => {}
        IrInstr::MemoryGrow { delta, .. } => {
            sub(delta);
        }
        IrInstr::MemoryCopy { dst, src, len } => {
            sub(dst);
            sub(src);
            sub(len);
        }
        IrInstr::Select {
            val1,
            val2,
            condition,
            ..
        } => {
            sub(val1);
            sub(val2);
            sub(condition);
        }
    }
}

/// Replace every read-occurrence of `old` with `new` in `term`.
pub fn replace_uses_of_terminator(term: &mut IrTerminator, old: VarId, new: VarId) {
    let sub = |v: &mut VarId| {
        if *v == old {
            *v = new;
        }
    };
    match term {
        IrTerminator::Return { value: Some(v) } => {
            sub(v);
        }
        IrTerminator::Return { value: None }
        | IrTerminator::Jump { .. }
        | IrTerminator::Unreachable => {}
        IrTerminator::BranchIf { condition, .. } => {
            sub(condition);
        }
        IrTerminator::BranchTable { index, .. } => {
            sub(index);
        }
    }
}

// ── Global use-count ─────────────────────────────────────────────────────────

/// Counts how many times each variable is *read* across the entire function
/// (all blocks, all instructions, all terminators).
pub fn build_global_use_count(func: &IrFunction) -> HashMap<VarId, usize> {
    let mut counts: HashMap<VarId, usize> = HashMap::new();
    for block in &func.blocks {
        for instr in &block.instructions {
            for_each_use(instr, |v| {
                *counts.entry(v).or_insert(0) += 1;
            });
        }
        for_each_use_terminator(&block.terminator, |v| {
            *counts.entry(v).or_insert(0) += 1;
        });
    }
    counts
}

// ── Dead-local pruning ───────────────────────────────────────────────────────

/// Remove from `func.locals` any variable that no longer appears in any
/// instruction or terminator of any block.
pub fn prune_dead_locals(func: &mut IrFunction) {
    // Collect all variables still referenced anywhere in the function.
    let mut live: HashSet<VarId> = HashSet::new();

    for block in &func.blocks {
        for instr in &block.instructions {
            for_each_use(instr, |v| {
                live.insert(v);
            });
            if let Some(dest) = instr_dest(instr) {
                live.insert(dest);
            }
        }
        for_each_use_terminator(&block.terminator, |v| {
            live.insert(v);
        });
    }

    // Keep params unconditionally; prune locals that are not in `live`.
    func.locals.retain(|(var, _)| live.contains(var));
}

// ── Side-effect classification ───────────────────────────────────────────────

/// Returns `true` if the instruction is side-effect-free and can be safely
/// removed when its result is unused.
///
/// Instructions that may trap (Load, MemoryGrow), modify external state
/// (Store, GlobalSet, MemoryCopy), or have unknown effects (Call*) are
/// considered side-effectful and must be retained even if unused.
pub fn is_side_effect_free(instr: &IrInstr) -> bool {
    matches!(
        instr,
        IrInstr::Const { .. }
            | IrInstr::BinOp { .. }
            | IrInstr::UnOp { .. }
            | IrInstr::Assign { .. }
            | IrInstr::Select { .. }
            | IrInstr::GlobalGet { .. }
            | IrInstr::MemorySize { .. }
    )
}

// ── Rewrite terminator block targets ─────────────────────────────────────────

/// Rewrite all block-ID references in a terminator from `old` to `new`.
pub fn rewrite_terminator_target(term: &mut IrTerminator, old: BlockId, new: BlockId) {
    match term {
        IrTerminator::Jump { target } => {
            if *target == old {
                *target = new;
            }
        }
        IrTerminator::BranchIf {
            if_true, if_false, ..
        } => {
            if *if_true == old {
                *if_true = new;
            }
            if *if_false == old {
                *if_false = new;
            }
        }
        IrTerminator::BranchTable {
            targets, default, ..
        } => {
            for t in targets.iter_mut() {
                if *t == old {
                    *t = new;
                }
            }
            if *default == old {
                *default = new;
            }
        }
        IrTerminator::Return { .. } | IrTerminator::Unreachable => {}
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{BinOp, IrBlock, IrValue, WasmType};

    #[test]
    fn for_each_use_covers_binop() {
        let instr = IrInstr::BinOp {
            dest: VarId(2),
            op: BinOp::I32Add,
            lhs: VarId(0),
            rhs: VarId(1),
        };
        let mut uses = vec![];
        for_each_use(&instr, |v| uses.push(v));
        assert_eq!(uses, vec![VarId(0), VarId(1)]);
    }

    #[test]
    fn instr_dest_returns_none_for_store() {
        let instr = IrInstr::Store {
            ty: WasmType::I32,
            addr: VarId(0),
            value: VarId(1),
            offset: 0,
            width: crate::ir::MemoryAccessWidth::Full,
        };
        assert_eq!(instr_dest(&instr), None);
    }

    #[test]
    fn instr_dest_returns_some_for_const() {
        let instr = IrInstr::Const {
            dest: VarId(5),
            value: IrValue::I32(42),
        };
        assert_eq!(instr_dest(&instr), Some(VarId(5)));
    }

    #[test]
    fn is_side_effect_free_classification() {
        assert!(is_side_effect_free(&IrInstr::Const {
            dest: VarId(0),
            value: IrValue::I32(1),
        }));
        assert!(is_side_effect_free(&IrInstr::BinOp {
            dest: VarId(0),
            op: BinOp::I32Add,
            lhs: VarId(1),
            rhs: VarId(2),
        }));
        assert!(is_side_effect_free(&IrInstr::Assign {
            dest: VarId(0),
            src: VarId(1),
        }));
        assert!(!is_side_effect_free(&IrInstr::Store {
            ty: WasmType::I32,
            addr: VarId(0),
            value: VarId(1),
            offset: 0,
            width: crate::ir::MemoryAccessWidth::Full,
        }));
        assert!(!is_side_effect_free(&IrInstr::Load {
            dest: VarId(0),
            ty: WasmType::I32,
            addr: VarId(1),
            offset: 0,
            width: crate::ir::MemoryAccessWidth::Full,
            sign: None,
        }));
    }

    #[test]
    fn terminator_successors_coverage() {
        assert_eq!(
            terminator_successors(&IrTerminator::Return { value: None }),
            vec![]
        );
        assert_eq!(
            terminator_successors(&IrTerminator::Jump { target: BlockId(3) }),
            vec![BlockId(3)]
        );
        assert_eq!(
            terminator_successors(&IrTerminator::BranchIf {
                condition: VarId(0),
                if_true: BlockId(1),
                if_false: BlockId(2),
            }),
            vec![BlockId(1), BlockId(2)]
        );
    }

    #[test]
    fn build_predecessors_simple() {
        let func = IrFunction {
            params: vec![],
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
                    terminator: IrTerminator::Return { value: None },
                },
            ],
            entry_block: BlockId(0),
            return_type: None,
            type_idx: crate::ir::TypeIdx::new(0),
            needs_host: false,
        };
        let preds = build_predecessors(&func);
        assert!(preds[&BlockId(0)].is_empty());
        assert_eq!(preds[&BlockId(1)], HashSet::from([BlockId(0)]));
    }

    #[test]
    fn replace_uses_of_substitutes_correctly() {
        let mut instr = IrInstr::BinOp {
            dest: VarId(2),
            op: BinOp::I32Add,
            lhs: VarId(0),
            rhs: VarId(0),
        };
        replace_uses_of(&mut instr, VarId(0), VarId(5));
        match &instr {
            IrInstr::BinOp { lhs, rhs, .. } => {
                assert_eq!(*lhs, VarId(5));
                assert_eq!(*rhs, VarId(5));
            }
            _ => panic!("expected BinOp"),
        }
    }

    #[test]
    fn rewrite_terminator_target_works() {
        let mut term = IrTerminator::BranchIf {
            condition: VarId(0),
            if_true: BlockId(1),
            if_false: BlockId(2),
        };
        rewrite_terminator_target(&mut term, BlockId(1), BlockId(5));
        match &term {
            IrTerminator::BranchIf {
                if_true, if_false, ..
            } => {
                assert_eq!(*if_true, BlockId(5));
                assert_eq!(*if_false, BlockId(2));
            }
            _ => panic!("expected BranchIf"),
        }
    }
}
