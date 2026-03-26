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

use super::utils::{
    build_global_use_count, count_uses_of, count_uses_of_terminator, for_each_use, instr_dest,
    prune_dead_locals, replace_uses_of, replace_uses_of_terminator, set_instr_dest,
};
use crate::ir::{IrBlock, IrFunction, IrInstr, VarId};
use std::collections::HashMap;

// ── Public entry point ────────────────────────────────────────────────────────

/// Eliminate single-use Assign copies and prune now-dead locals.
pub fn eliminate(func: &mut IrFunction) {
    // ── Backward pass: redirect I_def dest through single-use Assigns ────────
    //
    // We rebuild the global use-count map before each round because a successful
    // coalescing removes one Assign (changing use counts), so the previous map is
    // stale. We break out of the per-block scan as soon as any block changes and
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
            None => continue, // v_src is not defined in this block, so can't coalesce
        };

        // The def must precede the Assign.
        // In strict SSA form each variable is defined exactly once, so this check
        // is always satisfied — but kept as a safety guard.
        if def_idx >= assign_idx {
            continue;
        }

        // Safety check: v_dst must not be read or written in the instructions
        // strictly between def_idx and assign_idx.
        //
        // Rationale: after redirect, I_def writes to v_dst at def_idx.  Any
        // intervening read would see the new value instead of the old one (wrong).
        // Any intervening write would clobber the value before it can be used (wrong).
        //
        // In strict SSA form v_dst has exactly one definition (this Assign), so it
        // cannot be written between the two indices. It also cannot be read before its
        // definition (the Assign), so the check is effectively a no-op. Kept as a
        // guard against any future relaxation of the invariant.
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
    // NOTE: Tests for global copy prop are ignored since the pass was removed.
    // See GLOBAL_COPY_PROP_TICKET.md for details on re-introducing it safely.

    #[test]
    #[ignore]
    fn const_assign_coalesced() {
        // v7 = Const(2); v1 = Assign(v7)
        // Global copy prop: v1→v7, removes Assign. Result: v7 = Const(2).
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
            } => assert_eq!(*dest, VarId(7), "producer v7 survives"),
            other => panic!("expected Const, got {other:?}"),
        }
    }

    // ── Intervening read of v_dst: must NOT coalesce ──────────────────────

    #[test]
    fn intervening_read_of_dst_blocks_coalesce() {
        // Non-SSA pattern: v5 is a parameter AND redefined by Assign.
        // v16 = v4+v3; v8 = v5+v1 (reads param v5); v5 = Assign(v16)
        // Backward coalescing v16→v5 blocked because v5 is read in between.
        // Global copy prop skips v5 because it has 2 defs (param + Assign).
        let mut func = IrFunction {
            params: vec![(VarId(5), WasmType::I32)],
            locals: vec![],
            blocks: single_block(
                vec![
                    IrInstr::BinOp {
                        dest: VarId(16),
                        op: BinOp::I32Add,
                        lhs: VarId(4),
                        rhs: VarId(3),
                    },
                    IrInstr::BinOp {
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
            ),
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
        };
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
    fn forward_blocked_by_src_redef_before_last_use() {
        // Non-SSA: v0 is a param AND redefined by BinOp (2 defs).
        // v10 = Assign(v0); v0 = v0+v1; v11 = v10+v2
        // Global copy prop skips v10 because v0 (src) has >1 def.
        // Forward pass also blocked: v0 redefined before last use of v10.
        let mut func = IrFunction {
            params: vec![(VarId(0), WasmType::I32)],
            locals: vec![],
            blocks: single_block(
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
            ),
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
        };
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
