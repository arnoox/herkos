//! Copy propagation: backward coalescing and forward substitution.
//!
//! ## Backward pass — single-use Assign coalescing
//!
//! When an instruction I_def defines variable `v_src`, and `v_src` is used
//! exactly once in the same block — by `Assign { dest: v_dst, src: v_src }` —
//! and `v_dst` is neither read nor written in instructions between I_def and the
//! Assign, we can:
//!
//! 1. Change I_def to write directly to `v_dst` (eliminating the copy-through `v_src`).
//! 2. Remove the `Assign` instruction.
//!
//! This eliminates the single-use temporaries that arise from Wasm's stack-based
//! evaluation model. For example:
//!
//! ```text
//! v7  = Const(2)         →  v1 = Const(2)
//! v1  = Assign(v7)       →  (removed)
//!
//! v16 = v4.add(v3)       →  v5 = v4.add(v3)
//! v5  = Assign(v16)      →  (removed)
//! ```
//!
//! ## Forward pass — Assign substitution
//!
//! When `v_dst = Assign(v_src)` and all uses of `v_dst` are within the same
//! block after the Assign, and `v_src` is not redefined before those uses,
//! replace every read of `v_dst` with `v_src` and remove the Assign.
//!
//! This eliminates the `local.get` temporaries that Wasm emits when reading a
//! parameter or loop variable:
//!
//! ```text
//! v20 = Assign(v1)              →  (removed)
//! v24 = v20.wrapping_add(v23)   →  v24 = v1.wrapping_add(v23)
//! ```
//!
//! ## Fixpoint and ordering
//!
//! Both passes run to fixpoint.  The backward pass runs first (it creates no
//! new forward opportunities), then the forward pass runs.  After both passes
//! settle, dead variables are pruned from `IrFunction::locals`.

use crate::ir::{IrBlock, IrFunction, IrInstr, IrTerminator, VarId};
use std::collections::{HashMap, HashSet};

// ── Public entry point ────────────────────────────────────────────────────────

/// Eliminate single-use Assign copies and prune now-dead locals.
pub fn eliminate(func: &mut IrFunction) {
    // ── Backward pass: redirect I_def dest through single-use Assigns ────────
    //
    // We rebuild the global use-count map before each round because a successful
    // coalescing removes one Assign (changing use counts), so the previous map is
    // stale.  We break out of the per-block scan as soon as any block changes and
    // restart the outer loop so we always work from a fresh global count.
    loop {
        let global_uses = build_global_use_count(func);
        let mut any_changed = false;
        for block in &mut func.blocks {
            if coalesce_one(block, &global_uses) {
                any_changed = true;
                break; // global_uses is now stale; rebuild before continuing
            }
        }
        if !any_changed {
            break;
        }
    }

    // ── Forward pass: substitute v_src for v_dst at each use site ────────────
    //
    // Runs after the backward pass because backward creates no new forward
    // opportunities (it only removes Assigns, never adds them).  The forward
    // pass eliminates the `local.get` snapshots that Wasm emits when reading
    // parameters or loop-carried variables, e.g.:
    //
    //   v20 = Assign(v1)             →  (removed)
    //   v24 = v20.wrapping_add(v23)  →  v24 = v1.wrapping_add(v23)
    loop {
        let global_uses = build_global_use_count(func);
        let mut any_changed = false;
        for block in &mut func.blocks {
            if forward_propagate_one(block, &global_uses) {
                any_changed = true;
                break;
            }
        }
        if !any_changed {
            break;
        }
    }

    // Prune locals that are no longer referenced anywhere.
    prune_dead_locals(func);
}

// ── Global use-count ──────────────────────────────────────────────────────────

/// Counts how many times each variable is *read* across the entire function
/// (all blocks, all instructions, all terminators).
///
/// This is used for the single-use check in `coalesce_one`: a variable is safe
/// to coalesce only when its global read-count is exactly 1, ensuring it is not
/// read in any other block.
fn build_global_use_count(func: &IrFunction) -> HashMap<VarId, usize> {
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

// ── Core coalescing logic ─────────────────────────────────────────────────────

/// Tries to perform a single Assign coalescing in `block`.
/// Returns `true` if a coalescing was performed.
///
/// `global_uses` is the function-wide read-count for every variable (built by
/// `build_global_use_count`).  We use it — rather than a per-block count — for
/// the single-use check so that variables read in *other* blocks are not
/// incorrectly considered single-use.
fn coalesce_one(block: &mut IrBlock, global_uses: &HashMap<VarId, usize>) -> bool {
    // ── Step 1: build per-block def-site map ─────────────────────────────
    let mut def_site: HashMap<VarId, usize> = HashMap::new(); // var → instruction index

    for (i, instr) in block.instructions.iter().enumerate() {
        if let Some(dest) = instr_dest(instr) {
            def_site.insert(dest, i);
        }
    }

    // ── Step 2: find a coalesceable Assign ───────────────────────────────
    for assign_idx in 0..block.instructions.len() {
        let (v_dst, v_src) = match &block.instructions[assign_idx] {
            IrInstr::Assign { dest, src } => (*dest, *src),
            _ => continue,
        };

        // Skip self-assignments (v_dst = v_src where they are the same).
        if v_dst == v_src {
            // Self-assignment: just remove it.
            block.instructions.remove(assign_idx);
            return true;
        }

        // v_src must have exactly one use *globally* (this Assign).
        //
        // Using the global count (not a per-block count) is the key safety
        // invariant: if v_src is also read in another block, coalescing would
        // redirect v_src's definition to write v_dst instead, leaving v_src
        // undefined for those other-block reads.
        if global_uses.get(&v_src).copied().unwrap_or(0) != 1 {
            continue;
        }

        // v_src must be defined by an instruction in this block.
        let def_idx = match def_site.get(&v_src) {
            Some(&i) => i,
            None => continue,
        };

        // The def must precede the Assign.
        if def_idx >= assign_idx {
            continue;
        }

        // Safety check: v_dst must not be read or written in the instructions
        // strictly between def_idx and assign_idx.
        //
        // Rationale: after redirect, I_def writes to v_dst at def_idx.  Any
        // intervening read would see the new value instead of the old one (wrong).
        // Any intervening write would clobber the value before it can be used (wrong).
        let conflict = block.instructions[def_idx + 1..assign_idx].iter().any(|i| {
            let mut found = false;
            for_each_use(i, |v| {
                if v == v_dst {
                    found = true;
                }
            });
            if instr_dest(i) == Some(v_dst) {
                found = true;
            }
            found
        });
        if conflict {
            continue;
        }

        // ── Safe: perform the redirect ────────────────────────────────────
        set_instr_dest(&mut block.instructions[def_idx], v_dst);
        block.instructions.remove(assign_idx);
        return true;
    }

    false
}

// ── Dead-local pruning ────────────────────────────────────────────────────────

/// Remove from `func.locals` any variable that no longer appears in any
/// instruction or terminator of any block.
fn prune_dead_locals(func: &mut IrFunction) {
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
        // Terminators that reference block IDs don't reference variables, but
        // the BranchIf condition and BranchTable index are already handled above.
    }

    // Keep params unconditionally; prune locals that are not in `live`.
    func.locals.retain(|(var, _)| live.contains(var));
}

// ── Instruction helpers ───────────────────────────────────────────────────────

/// Calls `f` with every variable read by `instr`.
fn for_each_use<F: FnMut(VarId)>(instr: &IrInstr, mut f: F) {
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
fn for_each_use_terminator<F: FnMut(VarId)>(term: &IrTerminator, mut f: F) {
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

/// Returns the variable written by `instr`, or `None` for side-effect-only instructions.
fn instr_dest(instr: &IrInstr) -> Option<VarId> {
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
fn set_instr_dest(instr: &mut IrInstr, new_dest: VarId) {
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

// ── Forward propagation ───────────────────────────────────────────────────────

/// Tries to perform a single forward substitution in `block`.
/// Returns `true` if any substitution was performed.
///
/// For each `Assign { dest: v_dst, src: v_src }` at position `assign_idx`:
///
/// 1. All global reads of `v_dst` must occur within this block, strictly after
///    `assign_idx` (ensures no cross-block uses, no pre-Assign uses in this block).
/// 2. `v_dst` must not be redefined after `assign_idx` within this block (avoids
///    incorrectly replacing uses that read a later definition).
/// 3. `v_src` must not be redefined between `assign_idx` (exclusive) and the
///    last use of `v_dst` (exclusive), preserving the value the Assign captured.
///
/// When all conditions hold, every use of `v_dst` after `assign_idx` is replaced
/// by `v_src`, and the Assign is removed.
fn forward_propagate_one(block: &mut IrBlock, global_uses: &HashMap<VarId, usize>) -> bool {
    for assign_idx in 0..block.instructions.len() {
        let (v_dst, v_src) = match &block.instructions[assign_idx] {
            IrInstr::Assign { dest, src } => (*dest, *src),
            _ => continue,
        };

        // Self-assignments are handled by the backward pass.
        if v_dst == v_src {
            continue;
        }

        // Count uses of v_dst in this block strictly after assign_idx.
        let uses_after_instrs: usize = block.instructions[assign_idx + 1..]
            .iter()
            .map(|i| count_uses_of(i, v_dst))
            .sum();
        let uses_in_term = count_uses_of_terminator(&block.terminator, v_dst);
        let local_uses_after = uses_after_instrs + uses_in_term;

        // All global reads of v_dst must be accounted for by local_uses_after.
        // Any excess means v_dst is used in another block, or before assign_idx
        // in this block — both unsafe to substitute.
        let global_count = global_uses.get(&v_dst).copied().unwrap_or(0);
        if global_count != local_uses_after {
            continue;
        }

        // Nothing to do if v_dst is never read after the Assign.
        if local_uses_after == 0 {
            continue;
        }

        // v_dst must not be redefined in instructions after assign_idx.
        // If it were, uses past the redefinition would read a different value.
        if block.instructions[assign_idx + 1..]
            .iter()
            .any(|i| instr_dest(i) == Some(v_dst))
        {
            continue;
        }

        // Determine the range of instructions in which v_src must remain stable.
        //
        // If the terminator reads v_dst, v_src must survive all instructions
        // after assign_idx.  Otherwise, only up to (but not including) the last
        // instruction that reads v_dst: reads happen before the dest-write in
        // the same instruction, so a same-position redefinition of v_src is safe.
        let check_end = if uses_in_term > 0 {
            block.instructions.len()
        } else {
            // last instruction index (0-based into the full block) that reads v_dst
            block.instructions[assign_idx + 1..]
                .iter()
                .enumerate()
                .filter(|(_, i)| count_uses_of(i, v_dst) > 0)
                .map(|(rel, _)| assign_idx + 1 + rel)
                .next_back()
                .unwrap_or(assign_idx) // unreachable: local_uses_after > 0
        };

        // Check v_src is not written in [assign_idx+1, check_end).
        if block.instructions[assign_idx + 1..check_end]
            .iter()
            .any(|i| instr_dest(i) == Some(v_src))
        {
            continue;
        }

        // Safe: substitute v_src for every read of v_dst after assign_idx.
        for instr in &mut block.instructions[assign_idx + 1..] {
            replace_uses_of(instr, v_dst, v_src);
        }
        replace_uses_of_terminator(&mut block.terminator, v_dst, v_src);
        block.instructions.remove(assign_idx);
        return true;
    }
    false
}

// ── Use-count helpers ─────────────────────────────────────────────────────────

/// Count how many times `var` appears as an operand (read) in `instr`.
fn count_uses_of(instr: &IrInstr, var: VarId) -> usize {
    let mut count = 0usize;
    for_each_use(instr, |v| {
        if v == var {
            count += 1;
        }
    });
    count
}

/// Count how many times `var` appears as an operand in `term`.
fn count_uses_of_terminator(term: &IrTerminator, var: VarId) -> usize {
    let mut count = 0usize;
    for_each_use_terminator(term, |v| {
        if v == var {
            count += 1;
        }
    });
    count
}

// ── Mutable use-replacement helpers ──────────────────────────────────────────

/// Replace every read-occurrence of `old` with `new` in `instr`.
/// Only touches operand (source) slots; the destination slot is never modified.
fn replace_uses_of(instr: &mut IrInstr, old: VarId, new: VarId) {
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
fn replace_uses_of_terminator(term: &mut IrTerminator, old: VarId, new: VarId) {
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{
        BinOp, BlockId, IrBlock, IrFunction, IrTerminator, IrValue, TypeIdx, WasmType,
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

    // ── Basic: Const → Assign ─────────────────────────────────────────────

    #[test]
    fn const_assign_coalesced() {
        // v7 = Const(2); v1 = Assign(v7)  →  v1 = Const(2)
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(7),
                    value: IrValue::I32(2),
                },
                IrInstr::Assign {
                    dest: VarId(1),
                    src: VarId(7),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        let block = &func.blocks[0];
        assert_eq!(block.instructions.len(), 1, "Assign should be removed");
        match &block.instructions[0] {
            IrInstr::Const {
                dest,
                value: IrValue::I32(2),
            } => assert_eq!(*dest, VarId(1), "dest should be redirected to v1"),
            other => panic!("expected Const, got {other:?}"),
        }
    }

    // ── Basic: BinOp → Assign ─────────────────────────────────────────────

    #[test]
    fn binop_assign_coalesced() {
        // v16 = v4 + v3; v5 = Assign(v16)  →  v5 = v4 + v3
        let mut func = make_func(single_block(
            vec![
                IrInstr::BinOp {
                    dest: VarId(16),
                    op: BinOp::I32Add,
                    lhs: VarId(4),
                    rhs: VarId(3),
                },
                IrInstr::Assign {
                    dest: VarId(5),
                    src: VarId(16),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        let block = &func.blocks[0];
        assert_eq!(block.instructions.len(), 1);
        match &block.instructions[0] {
            IrInstr::BinOp { dest, .. } => assert_eq!(*dest, VarId(5)),
            other => panic!("expected BinOp, got {other:?}"),
        }
    }

    // ── Multi-use src: must NOT coalesce ─────────────────────────────────

    #[test]
    fn multi_use_src_not_coalesced() {
        // v7 = Const(2); v1 = Assign(v7); v2 = Assign(v7)  — v7 used twice
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(7),
                    value: IrValue::I32(2),
                },
                IrInstr::Assign {
                    dest: VarId(1),
                    src: VarId(7),
                },
                IrInstr::Assign {
                    dest: VarId(2),
                    src: VarId(7),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        // Nothing should change: v7 has 2 uses.
        assert_eq!(func.blocks[0].instructions.len(), 3);
    }

    // ── Intervening read of v_dst: must NOT coalesce ──────────────────────

    #[test]
    fn intervening_read_of_dst_blocks_coalesce() {
        // v5 = v4 + v3
        // v3 = Assign(v4)   ← reads v4 (not v_dst=v5), so no conflict for v5→v_dst5
        // BUT let's test a case where v_dst IS read in between:
        // v5 = v4 + v3
        // use_v5 = v5 + 1   ← reads v5 (which IS v_src here)... wait v_src=v16, v_dst=v5
        // v5 = Assign(v16)  ← can't coalesce because... let me reconsider.
        //
        // Pattern: v16 = v4+v3 (def_idx=0), intervening: v8 = v5+1 (reads v5=v_dst), Assign(v16→v5)
        // v5 is read in between → conflict → do NOT coalesce.
        let mut func = make_func(single_block(
            vec![
                IrInstr::BinOp {
                    dest: VarId(16),
                    op: BinOp::I32Add,
                    lhs: VarId(4),
                    rhs: VarId(3),
                },
                IrInstr::BinOp {
                    // reads v5 (= v_dst of the upcoming Assign)
                    dest: VarId(8),
                    op: BinOp::I32Add,
                    lhs: VarId(5),
                    rhs: VarId(1),
                },
                IrInstr::Assign {
                    dest: VarId(5),
                    src: VarId(16),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        // Coalescing v16→v5 is blocked because v5 is read between def(v16) and Assign.
        assert_eq!(func.blocks[0].instructions.len(), 3);
    }

    // ── Intervening write of v_dst: must NOT coalesce ─────────────────────

    #[test]
    fn intervening_write_of_dst_blocks_coalesce() {
        // v5 = v4+v3; v5 = Assign(v0) [write to v5 in between]; v4 = Assign(v5)
        // v5 is written between def(v5_tmp) and Assign → conflict.
        let mut func = make_func(single_block(
            vec![
                IrInstr::BinOp {
                    dest: VarId(99), // temp
                    op: BinOp::I32Add,
                    lhs: VarId(4),
                    rhs: VarId(3),
                },
                IrInstr::Assign {
                    // writes v4 (= v_dst of next Assign)
                    dest: VarId(4),
                    src: VarId(0),
                },
                IrInstr::Assign {
                    dest: VarId(4),
                    src: VarId(99),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        // Coalescing v99→v4 is blocked because v4 is written between def(v99) and Assign.
        assert_eq!(func.blocks[0].instructions.len(), 3);
    }

    // ── Self-assignment removal ───────────────────────────────────────────

    #[test]
    fn self_assign_removed() {
        // v1 = Assign(v1) is a no-op and should be removed.
        let mut func = make_func(single_block(
            vec![IrInstr::Assign {
                dest: VarId(1),
                src: VarId(1),
            }],
            ret_none(),
        ));
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 0);
    }

    // ── Chain coalescing ──────────────────────────────────────────────────

    #[test]
    fn chain_coalesced() {
        // v7 = Const(2); v_a = Assign(v7); v1 = Assign(v_a)
        // Round 1: v7 single-use → coalesce v_a=Const(2), remove first Assign.
        // Round 2: v_a single-use → coalesce v1=Const(2), remove second Assign.
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(7),
                    value: IrValue::I32(2),
                },
                IrInstr::Assign {
                    dest: VarId(10),
                    src: VarId(7),
                },
                IrInstr::Assign {
                    dest: VarId(1),
                    src: VarId(10),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
        match &func.blocks[0].instructions[0] {
            IrInstr::Const {
                dest,
                value: IrValue::I32(2),
            } => assert_eq!(*dest, VarId(1)),
            other => panic!("expected Const(v1,2), got {other:?}"),
        }
    }

    // ── Dead local pruning ────────────────────────────────────────────────

    #[test]
    fn dead_local_pruned_after_coalesce() {
        // v7 is in locals; after coalescing v7 is gone from instructions.
        let mut func = make_func_with_locals(
            single_block(
                vec![
                    IrInstr::Const {
                        dest: VarId(7),
                        value: IrValue::I32(2),
                    },
                    IrInstr::Assign {
                        dest: VarId(1),
                        src: VarId(7),
                    },
                ],
                ret_none(),
            ),
            vec![(VarId(7), WasmType::I32), (VarId(1), WasmType::I32)],
        );
        eliminate(&mut func);
        // v7 should be pruned; v1 (now has the Const) should survive.
        assert!(
            !func.locals.iter().any(|(v, _)| *v == VarId(7)),
            "v7 should be pruned from locals"
        );
        assert!(
            func.locals.iter().any(|(v, _)| *v == VarId(1)),
            "v1 should remain in locals"
        );
    }

    // ── No-op: no Assigns → nothing changes ──────────────────────────────

    #[test]
    fn no_assigns_unchanged() {
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(42),
                },
                IrInstr::BinOp {
                    dest: VarId(1),
                    op: BinOp::I32Add,
                    lhs: VarId(0),
                    rhs: VarId(0),
                },
            ],
            IrTerminator::Return {
                value: Some(VarId(1)),
            },
        ));
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 2);
    }

    // ── Realistic fibo B0 pattern ─────────────────────────────────────────

    #[test]
    fn fibo_b0_pattern() {
        // v7 = Const(2); v1 = Assign(v7)   →  v1 = Const(2)
        // v8 = Const(2); BranchIf using v8  →  v8 stays (used in BranchIf, not Assign)
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(7),
                    value: IrValue::I32(2),
                },
                IrInstr::Assign {
                    dest: VarId(1),
                    src: VarId(7),
                },
                IrInstr::Const {
                    dest: VarId(8),
                    value: IrValue::I32(2),
                },
                IrInstr::BinOp {
                    dest: VarId(9),
                    op: BinOp::I32LtS,
                    lhs: VarId(0),
                    rhs: VarId(8),
                },
            ],
            IrTerminator::BranchIf {
                condition: VarId(9),
                if_true: BlockId(1),
                if_false: BlockId(2),
            },
        ));
        eliminate(&mut func);
        // v7 (single-use in Assign) coalesced into v1; v8, v9 remain.
        let instrs = &func.blocks[0].instructions;
        assert_eq!(
            instrs.len(),
            3,
            "only v7+Assign pair removed, leaving 3 instrs"
        );
        // First instruction should now define v1 directly.
        match &instrs[0] {
            IrInstr::Const {
                dest,
                value: IrValue::I32(2),
            } => assert_eq!(*dest, VarId(1)),
            other => panic!("expected Const(v1,2), got {other:?}"),
        }
    }

    // ── Forward pass tests ────────────────────────────────────────────────

    #[test]
    fn forward_basic_param_snapshot() {
        // v_dst = Assign(v_src); v_out = BinOp(v_dst, v1) → v_out = BinOp(v_src, v1)
        let mut func = make_func(single_block(
            vec![
                IrInstr::Assign {
                    dest: VarId(10),
                    src: VarId(0), // parameter — not defined by any in-block instruction
                },
                IrInstr::BinOp {
                    dest: VarId(11),
                    op: BinOp::I32Add,
                    lhs: VarId(10),
                    rhs: VarId(1),
                },
            ],
            IrTerminator::Return {
                value: Some(VarId(11)),
            },
        ));
        eliminate(&mut func);
        let instrs = &func.blocks[0].instructions;
        assert_eq!(instrs.len(), 1, "Assign should be removed");
        match &instrs[0] {
            IrInstr::BinOp { lhs, .. } => assert_eq!(*lhs, VarId(0), "lhs should be v0"),
            other => panic!("expected BinOp, got {other:?}"),
        }
    }

    #[test]
    fn forward_multi_use_all_in_block() {
        // v_dst = Assign(v_src); use(v_dst) twice in same block → both replaced
        let mut func = make_func(single_block(
            vec![
                IrInstr::Assign {
                    dest: VarId(10),
                    src: VarId(0),
                },
                IrInstr::BinOp {
                    dest: VarId(11),
                    op: BinOp::I32Add,
                    lhs: VarId(10),
                    rhs: VarId(1),
                },
                IrInstr::BinOp {
                    dest: VarId(12),
                    op: BinOp::I32Add,
                    lhs: VarId(10),
                    rhs: VarId(2),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        let instrs = &func.blocks[0].instructions;
        assert_eq!(instrs.len(), 2, "Assign should be removed");
        // Both BinOps should now reference v0 directly
        for instr in instrs {
            match instr {
                IrInstr::BinOp { lhs, .. } => assert_eq!(*lhs, VarId(0)),
                other => panic!("expected BinOp, got {other:?}"),
            }
        }
    }

    #[test]
    fn forward_use_in_terminator() {
        // v_dst = Assign(v_src); Return(v_dst) → Return(v_src)
        let mut func = make_func(single_block(
            vec![IrInstr::Assign {
                dest: VarId(10),
                src: VarId(0),
            }],
            IrTerminator::Return {
                value: Some(VarId(10)),
            },
        ));
        eliminate(&mut func);
        assert_eq!(
            func.blocks[0].instructions.len(),
            0,
            "Assign should be removed"
        );
        match &func.blocks[0].terminator {
            IrTerminator::Return { value: Some(v) } => assert_eq!(*v, VarId(0)),
            other => panic!("expected Return(v0), got {other:?}"),
        }
    }

    #[test]
    fn forward_blocked_by_cross_block_use() {
        // v_dst used in another block → do NOT forward
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Assign {
                    dest: VarId(10),
                    src: VarId(0),
                }],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::BinOp {
                    dest: VarId(11),
                    op: BinOp::I32Add,
                    lhs: VarId(10), // reads v10 from B0 — cross-block use
                    rhs: VarId(1),
                }],
                terminator: IrTerminator::Return {
                    value: Some(VarId(11)),
                },
            },
        ]);
        eliminate(&mut func);
        // Assign must remain in B0 because v10 is used in B1
        assert_eq!(func.blocks[0].instructions.len(), 1);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Assign {
                dest: VarId(10),
                src: VarId(0)
            }
        ));
    }

    #[test]
    fn forward_blocked_by_src_redef_before_last_use() {
        // v_dst = Assign(v_src)
        // v_src = v_src + 1      ← redefines v_src
        // use(v_dst)             ← last use — would see wrong v_src value
        let mut func = make_func(single_block(
            vec![
                IrInstr::Assign {
                    dest: VarId(10),
                    src: VarId(0),
                },
                IrInstr::BinOp {
                    dest: VarId(0), // redefines v_src = v0
                    op: BinOp::I32Add,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
                IrInstr::BinOp {
                    dest: VarId(11),
                    op: BinOp::I32Add,
                    lhs: VarId(10), // last use of v_dst
                    rhs: VarId(2),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        // v10 = Assign(v0) must NOT be eliminated: v0 is redefined before v10's last use
        assert_eq!(func.blocks[0].instructions.len(), 3);
    }

    #[test]
    fn forward_safe_when_src_redef_at_last_use() {
        // v_dst = Assign(v_src)
        // v_src = v_dst + 5   ← uses v_dst AND redefines v_src at the same position
        //
        // v_src is redefined at check_end (exclusive), not before it, so the
        // substitution is safe: v_src = (old v_src) + 5.
        let mut func = make_func(single_block(
            vec![
                IrInstr::Assign {
                    dest: VarId(10),
                    src: VarId(0),
                },
                IrInstr::BinOp {
                    dest: VarId(0), // redefines v0 (v_src) — but this is also the last use of v10
                    op: BinOp::I32Add,
                    lhs: VarId(10), // reads v10 (v_dst)
                    rhs: VarId(5),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        // Assign should be removed; v0 = BinOp(v0, v5) is the result
        let instrs = &func.blocks[0].instructions;
        assert_eq!(instrs.len(), 1, "Assign should be removed");
        match &instrs[0] {
            IrInstr::BinOp { dest, lhs, .. } => {
                assert_eq!(*dest, VarId(0));
                assert_eq!(*lhs, VarId(0), "lhs should be v0 (substituted from v10)");
            }
            other => panic!("expected BinOp, got {other:?}"),
        }
    }

    #[test]
    fn forward_blocked_by_dst_redef() {
        // v_dst = Assign(v_src)
        // v_dst = BinOp(...)     ← redefines v_dst: later uses read the new value
        // use(v_dst)
        let mut func = make_func(single_block(
            vec![
                IrInstr::Assign {
                    dest: VarId(10),
                    src: VarId(0),
                },
                IrInstr::BinOp {
                    dest: VarId(10), // redefines v_dst
                    op: BinOp::I32Add,
                    lhs: VarId(1),
                    rhs: VarId(2),
                },
                IrInstr::BinOp {
                    dest: VarId(11),
                    op: BinOp::I32Add,
                    lhs: VarId(10),
                    rhs: VarId(3),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 3);
    }

    #[test]
    fn forward_fibo_b3_local_get_chain() {
        // Mirrors the v16/v17 pattern from func_7 B3:
        //   v16 = Assign(v1)
        //   v17 = Assign(v0)
        //   v18 = BinOp(I32GeS, v16, v17)
        //   BranchIf(v18, B5, B4)
        //
        // After forward pass: v18 = BinOp(v1, v0), no Assigns.
        let mut func = make_func(vec![IrBlock {
            id: BlockId(0),
            instructions: vec![
                IrInstr::Assign {
                    dest: VarId(16),
                    src: VarId(1),
                },
                IrInstr::Assign {
                    dest: VarId(17),
                    src: VarId(0),
                },
                IrInstr::BinOp {
                    dest: VarId(18),
                    op: BinOp::I32GeS,
                    lhs: VarId(16),
                    rhs: VarId(17),
                },
            ],
            terminator: IrTerminator::BranchIf {
                condition: VarId(18),
                if_true: BlockId(1),
                if_false: BlockId(2),
            },
        }]);
        eliminate(&mut func);
        let instrs = &func.blocks[0].instructions;
        assert_eq!(instrs.len(), 1, "both Assigns should be removed");
        match &instrs[0] {
            IrInstr::BinOp {
                lhs,
                rhs,
                op: BinOp::I32GeS,
                ..
            } => {
                assert_eq!(*lhs, VarId(1), "lhs should be v1");
                assert_eq!(*rhs, VarId(0), "rhs should be v0");
            }
            other => panic!("expected BinOp(I32GeS), got {other:?}"),
        }
    }

    #[test]
    fn forward_fibo_b4_multi_snapshot() {
        // Mirrors the v20/v21/v22/v25 pattern from func_7 B4.
        // v20 = Assign(v1); v21 = Assign(v1); v22 = Assign(v0)
        // v23 = BinOp(I32LtS, v21, v22)
        // v24 = BinOp(I32Add, v20, v23)
        // v25 = Assign(v0)
        // v26 = BinOp(I32LeS, v24, v25)
        // BranchIf(v26, ...)
        //
        // After forward pass:
        //   v23 = BinOp(I32LtS, v1, v0)
        //   v24 = BinOp(I32Add, v1, v23)
        //   v26 = BinOp(I32LeS, v24, v0)
        let mut func = make_func(vec![IrBlock {
            id: BlockId(0),
            instructions: vec![
                IrInstr::Assign {
                    dest: VarId(20),
                    src: VarId(1),
                },
                IrInstr::Assign {
                    dest: VarId(21),
                    src: VarId(1),
                },
                IrInstr::Assign {
                    dest: VarId(22),
                    src: VarId(0),
                },
                IrInstr::BinOp {
                    dest: VarId(23),
                    op: BinOp::I32LtS,
                    lhs: VarId(21),
                    rhs: VarId(22),
                },
                IrInstr::BinOp {
                    dest: VarId(24),
                    op: BinOp::I32Add,
                    lhs: VarId(20),
                    rhs: VarId(23),
                },
                IrInstr::Assign {
                    dest: VarId(25),
                    src: VarId(0),
                },
                IrInstr::BinOp {
                    dest: VarId(26),
                    op: BinOp::I32LeS,
                    lhs: VarId(24),
                    rhs: VarId(25),
                },
            ],
            terminator: IrTerminator::BranchIf {
                condition: VarId(26),
                if_true: BlockId(1),
                if_false: BlockId(2),
            },
        }]);
        eliminate(&mut func);

        let instrs = &func.blocks[0].instructions;
        // Only the 3 BinOps remain; all 4 Assigns are gone.
        assert_eq!(
            instrs.len(),
            3,
            "all 4 Assigns should be removed, leaving 3 BinOps"
        );

        // v23 = BinOp(I32LtS, v1, v0)
        match &instrs[0] {
            IrInstr::BinOp {
                dest,
                op: BinOp::I32LtS,
                lhs,
                rhs,
            } => {
                assert_eq!(*dest, VarId(23));
                assert_eq!(*lhs, VarId(1));
                assert_eq!(*rhs, VarId(0));
            }
            other => panic!("instrs[0]: expected BinOp(I32LtS, v1, v0), got {other:?}"),
        }

        // v24 = BinOp(I32Add, v1, v23)
        match &instrs[1] {
            IrInstr::BinOp {
                dest,
                op: BinOp::I32Add,
                lhs,
                rhs,
            } => {
                assert_eq!(*dest, VarId(24));
                assert_eq!(*lhs, VarId(1));
                assert_eq!(*rhs, VarId(23));
            }
            other => panic!("instrs[1]: expected BinOp(I32Add, v1, v23), got {other:?}"),
        }

        // v26 = BinOp(I32LeS, v24, v0)
        match &instrs[2] {
            IrInstr::BinOp {
                dest,
                op: BinOp::I32LeS,
                lhs,
                rhs,
            } => {
                assert_eq!(*dest, VarId(26));
                assert_eq!(*lhs, VarId(24));
                assert_eq!(*rhs, VarId(0));
            }
            other => panic!("instrs[2]: expected BinOp(I32LeS, v24, v0), got {other:?}"),
        }
    }

    // ── Regression: cross-block v_src must NOT be coalesced ──────────────
    //
    // Bug: the old per-block use_count counted v_src uses only within the
    // current block.  If v_src had exactly 1 use in the current block (the
    // Assign) but was also read in another block, copy_prop would incorrectly
    // coalesce, redirecting v_src's definition to v_dst and leaving v_src
    // undefined in the other block.  Functions like `lcm` and `isqrt` then
    // returned 0 (the default initial value) instead of the correct result.

    #[test]
    fn cross_block_src_not_coalesced() {
        // Block 0: v3 = Const(1)
        //          v0 = Assign(v3)    ← local.set; v3 single-use globally → OK to coalesce
        //          v10 = Assign(v0)   ← local.get snapshot (1 use in B0)
        //          Jump(B1)
        //
        // Block 1: v20 = Assign(v0)   ← local.get snapshot (1 use in B1)
        //          Return(v20)
        //
        // Global use_count[v0] = 2 (B0 index 2 AND B1 index 0) → must NOT coalesce v10=Assign(v0).
        // If coalesced (buggy), v0 would vanish from B0 and B1 would read an undefined variable.
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![
                    IrInstr::Const {
                        dest: VarId(3),
                        value: IrValue::I32(1),
                    },
                    IrInstr::Assign {
                        dest: VarId(0),
                        src: VarId(3),
                    },
                    IrInstr::Assign {
                        dest: VarId(10),
                        src: VarId(0),
                    },
                ],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::Assign {
                    dest: VarId(20),
                    src: VarId(0),
                }],
                terminator: IrTerminator::Return {
                    value: Some(VarId(20)),
                },
            },
        ]);
        eliminate(&mut func);

        // v3 is single-use globally → coalesced into v0: B0[0] becomes `v0 = Const(1)`.
        // v0 has 2 global uses → NOT coalesced; v0 must still be defined in Block 0.
        let b0_dests: Vec<VarId> = func.blocks[0]
            .instructions
            .iter()
            .filter_map(instr_dest)
            .collect();
        assert!(
            b0_dests.contains(&VarId(0)),
            "v0 must still be defined in Block 0 (it is read in Block 1); got: {b0_dests:?}"
        );
    }

    // ── Multi-block: each block is independent ────────────────────────────

    #[test]
    fn multi_block_each_coalesced_independently() {
        // Block 0: v7 = Const(1); v1 = Assign(v7) → v1 = Const(1)
        // Block 1: v8 = Const(2); v2 = Assign(v8) → v2 = Const(2)
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![
                    IrInstr::Const {
                        dest: VarId(7),
                        value: IrValue::I32(1),
                    },
                    IrInstr::Assign {
                        dest: VarId(1),
                        src: VarId(7),
                    },
                ],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![
                    IrInstr::Const {
                        dest: VarId(8),
                        value: IrValue::I32(2),
                    },
                    IrInstr::Assign {
                        dest: VarId(2),
                        src: VarId(8),
                    },
                ],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func);
        assert_eq!(func.blocks[0].instructions.len(), 1);
        assert_eq!(func.blocks[1].instructions.len(), 1);
    }
}
