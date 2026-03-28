//! Shared utility functions for IR optimization passes.
//!
//! Provides common operations on IR instructions, terminators, and control flow
//! that are needed by multiple optimization passes.
//!
//! Some functions are not yet used by existing passes but are provided for
//! upcoming optimization passes (const_prop, dead_instrs, local_cse, licm).
#![allow(dead_code)]

use crate::ir::{BinOp, BlockId, IrFunction, IrInstr, IrTerminator, IrValue, UnOp, VarId};
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
        IrInstr::Call { args, .. } | IrInstr::CallImport { args, .. } => {
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
        IrInstr::MemoryFill { dst, val, len } => {
            f(*dst);
            f(*val);
            f(*len);
        }
        IrInstr::MemoryInit {
            dst,
            src_offset,
            len,
            ..
        } => {
            f(*dst);
            f(*src_offset);
            f(*len);
        }
        IrInstr::DataDrop { .. } => {}
        IrInstr::Phi { srcs, .. } => {
            for (_, src) in srcs {
                f(*src);
            }
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

        IrInstr::Store { .. }
        | IrInstr::GlobalSet { .. }
        | IrInstr::MemoryCopy { .. }
        | IrInstr::MemoryFill { .. }
        | IrInstr::MemoryInit { .. }
        | IrInstr::DataDrop { .. }
        | IrInstr::Phi { .. } => None,
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
        IrInstr::Store { .. }
        | IrInstr::GlobalSet { .. }
        | IrInstr::MemoryCopy { .. }
        | IrInstr::MemoryFill { .. }
        | IrInstr::MemoryInit { .. }
        | IrInstr::DataDrop { .. }
        | IrInstr::Phi { .. } => {}
    }
}

// ── Instruction iteration ───────────────────────────────────────────────────

/// Call `f` for each instruction across all blocks in the function.
pub fn for_each_instr<F: FnMut(&IrInstr)>(func: &IrFunction, mut f: F) {
    for block in &func.blocks {
        for instr in &block.instructions {
            f(instr);
        }
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
        IrInstr::MemoryFill { dst, val, len } => {
            sub(dst);
            sub(val);
            sub(len);
        }
        IrInstr::MemoryInit {
            dst,
            src_offset,
            len,
            ..
        } => {
            sub(dst);
            sub(src_offset);
            sub(len);
        }
        IrInstr::DataDrop { .. } => {}
        IrInstr::Phi { srcs, .. } => {
            for (_, src) in srcs.iter_mut() {
                sub(src);
            }
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

/// Counts how many times each variable is *defined* across the entire function.
/// Function parameters count as one definition each.
pub fn build_global_def_count(func: &IrFunction) -> HashMap<VarId, usize> {
    let mut counts: HashMap<VarId, usize> = HashMap::new();
    // Params count as definitions.
    for (param_var, _) in &func.params {
        *counts.entry(*param_var).or_insert(0) += 1;
    }
    // Each instruction that produces a value is a definition.
    for_each_instr(func, |instr| {
        if let Some(dest) = instr_dest(instr) {
            *counts.entry(dest).or_insert(0) += 1;
        }
    });
    counts
}

/// Counts how many times each variable is *read* across the entire function
/// (all blocks, all instructions, all terminators).
pub fn build_global_use_count(func: &IrFunction) -> HashMap<VarId, usize> {
    let mut counts: HashMap<VarId, usize> = HashMap::new();
    for_each_instr(func, |instr| {
        for_each_use(instr, |v| {
            *counts.entry(v).or_insert(0) += 1;
        });
    });
    for block in &func.blocks {
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

    for_each_instr(func, |instr| {
        for_each_use(instr, |v| {
            live.insert(v);
        });
        if let Some(dest) = instr_dest(instr) {
            live.insert(dest);
        }
    });

    for block in &func.blocks {
        for_each_use_terminator(&block.terminator, |v| {
            live.insert(v);
        });
    }

    // Keep params unconditionally; prune locals that are not in `live`.
    func.locals.retain(|(var, _)| live.contains(var));
}

// ── Value key ────────────────────────────────────────────────────────────────

/// Hashable representation of a pure computation for deduplication.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueKey {
    /// Constant value (using bit-level equality for floats).
    Const(ConstKey),
    /// Binary operation with operand variable IDs.
    BinOp { op: BinOp, lhs: VarId, rhs: VarId },
    /// Unary operation with operand variable ID.
    UnOp { op: UnOp, operand: VarId },
}

/// Build a [`ValueKey`] for a `BinOp`, normalizing operand order for commutative ops.
pub fn binop_key(op: BinOp, lhs: VarId, rhs: VarId) -> ValueKey {
    let (lhs, rhs) = if is_commutative(&op) && lhs.0 > rhs.0 {
        (rhs, lhs)
    } else {
        (lhs, rhs)
    };
    ValueKey::BinOp { op, lhs, rhs }
}

/// Bit-level constant key that implements `Eq`/`Hash` correctly for floats.
///
/// Floats have no `Eq` or `Hash` impl in Rust because NaN != NaN.  This type
/// stores the raw bit pattern instead, so two constants with identical bits
/// hash to the same bucket and compare equal — matching Wasm's value semantics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstKey {
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

// ── Multi-definition detection ────────────────────────────────────────────────

/// Build the set of variables defined more than once across the function.
///
/// After phi lowering the code is no longer in strict SSA form: loop phi
/// variables receive an initial assignment in the pre-loop block and a
/// back-edge update at the end of each iteration.  These variables carry
/// different values at different program points, so any `BinOp`/`UnOp` that
/// uses them cannot be safely deduplicated across blocks.
///
/// `Const` instructions are always safe to deduplicate (they have no operands).
pub fn build_multi_def_vars(func: &IrFunction) -> HashSet<VarId> {
    build_global_def_count(func)
        .into_iter()
        .filter(|&(_, count)| count > 1)
        .map(|(v, _)| v)
        .collect()
}

// ── Dominator tree ────────────────────────────────────────────────────────────

/// Compute the reverse-postorder (RPO) traversal of the CFG from `entry_block`.
///
/// RPO is the reverse of the DFS postorder: nodes are visited only after all
/// their DFS-tree predecessors, so every dominator appears before the blocks it
/// dominates.  This ordering is required by the Cooper/Harvey/Kennedy iterative
/// dominator algorithm, which relies on processing a block's dominators before
/// the block itself.
///
/// For example, given the CFG:
///
/// ```text
///   entry
///   /   \
///  A     B
///   \   /
///     C
/// ```
///
/// DFS visits entry → A → C → B (postorder: C, A, B, entry or C, B, A, entry
/// depending on successor order).  Reversing gives RPO:
///
/// ```text
/// [entry, A, B, C]
/// ```
///
/// `entry` is always first; `C` is always last because it is reachable only
/// after both `A` and `B`.
pub fn compute_rpo(func: &IrFunction) -> Vec<BlockId> {
    let block_idx: HashMap<BlockId, usize> = func
        .blocks
        .iter()
        .enumerate()
        .map(|(i, b)| (b.id, i))
        .collect();

    let mut visited = vec![false; func.blocks.len()];
    let mut postorder = Vec::with_capacity(func.blocks.len());

    dfs_postorder(
        func,
        func.entry_block,
        &block_idx,
        &mut visited,
        &mut postorder,
    );

    postorder.reverse();
    postorder
}

/// Recursive DFS helper that appends `block_id` to `postorder` after visiting
/// all of its successors.
///
/// Each block is visited at most once (guarded by `visited`).  Unreachable
/// blocks — those not reachable from the entry — are never pushed and therefore
/// absent from the final RPO, which is the desired behaviour: the dominator
/// algorithm only needs to reason about reachable blocks.
fn dfs_postorder(
    func: &IrFunction,
    block_id: BlockId,
    block_idx: &HashMap<BlockId, usize>,
    visited: &mut Vec<bool>,
    postorder: &mut Vec<BlockId>,
) {
    let idx = match block_idx.get(&block_id) {
        Some(&i) => i,
        None => return,
    };
    if visited[idx] {
        return;
    }
    visited[idx] = true;

    for succ in terminator_successors(&func.blocks[idx].terminator) {
        dfs_postorder(func, succ, block_idx, visited, postorder);
    }
    postorder.push(block_id);
}

/// Compute the immediate dominator of each block using the Cooper/Harvey/Kennedy
/// iterative algorithm.
///
/// Returns a map `idom` where `idom[b]` is the immediate dominator of `b`.
/// The entry block is its own immediate dominator: `idom[entry] = entry`.
///
/// A block `d` dominates block `b` if `d` appears on *every* path from the
/// entry block to `b`.  For example, given:
///
/// ```text
/// entry → A → B
/// entry → C → B
/// ```
///
/// `entry` dominates `B` (it is on every path), but `A` and `C` do not.
/// The immediate dominator is the closest such dominator — the last one
/// before `b` in the dominator tree.
///
/// The algorithm works by repeatedly intersecting predecessor dominators in RPO
/// until a fixed point is reached.  Because blocks are processed in RPO order,
/// most dominators converge in a single pass over the function.
pub fn compute_idoms(func: &IrFunction) -> HashMap<BlockId, BlockId> {
    let rpo = compute_rpo(func);
    // rpo_num[b] = index in RPO order (entry = 0, smallest = processed first).
    let rpo_num: HashMap<BlockId, usize> = rpo.iter().enumerate().map(|(i, &b)| (b, i)).collect();

    let preds = build_predecessors(func);
    let entry = func.entry_block;

    let mut idom: HashMap<BlockId, BlockId> = HashMap::new();
    idom.insert(entry, entry);

    let mut changed = true;
    while changed {
        changed = false;
        // Process blocks in RPO order, skipping the entry.
        for &b in rpo.iter().skip(1) {
            let block_preds = &preds[&b];

            // Start with the first predecessor that already has an idom assigned.
            let mut new_idom = match block_preds
                .iter()
                .filter(|&&p| idom.contains_key(&p))
                .min_by_key(|&&p| rpo_num[&p])
            {
                Some(&p) => p,
                None => continue, // unreachable block — skip
            };

            // Intersect (walk up dom tree) with all other processed predecessors.
            for &p in block_preds {
                if p != new_idom && idom.contains_key(&p) {
                    new_idom = intersect(p, new_idom, &idom, &rpo_num);
                }
            }

            if idom.get(&b) != Some(&new_idom) {
                idom.insert(b, new_idom);
                changed = true;
            }
        }
    }

    idom
}

/// Walk up both dom-tree fingers until they meet — the standard Cooper intersect.
fn intersect(
    mut a: BlockId,
    mut b: BlockId,
    idom: &HashMap<BlockId, BlockId>,
    rpo_num: &HashMap<BlockId, usize>,
) -> BlockId {
    while a != b {
        while rpo_num[&a] > rpo_num[&b] {
            a = idom[&a];
        }
        while rpo_num[&b] > rpo_num[&a] {
            b = idom[&b];
        }
    }
    a
}

/// Build the dominator-tree children map from the `idom` map.
///
/// For each block `b` (except the entry), `children[idom[b]]` gains `b` as a
/// child.  Children are sorted by block ID for deterministic traversal order.
///
/// For example, given the CFG:
///
/// ```text
///       entry
///      /     \
///     A       B
///    / \
///   C   D
/// ```
///
/// The `idom` map is `{ A → entry, B → entry, C → A, D → A }`, and the
/// resulting children map is:
///
/// ```text
/// entry → [A, B]
/// A     → [C, D]
/// ```
pub fn build_dom_children(
    idom: &HashMap<BlockId, BlockId>,
    entry: BlockId,
) -> HashMap<BlockId, Vec<BlockId>> {
    let mut children: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    for (&b, &d) in idom {
        if b != entry {
            children.entry(d).or_default().push(b);
        }
    }
    // Sort children for deterministic output.
    for v in children.values_mut() {
        v.sort_unstable_by_key(|id| id.0);
    }
    children
}

// ── Side-effect classification ───────────────────────────────────────────────

/// Returns `true` if the instruction is side-effect-free and can be safely
/// removed when its result is unused.
///
/// Instructions that may trap (Load, MemoryGrow, integer div/rem, float-to-int
/// truncation), modify external state (Store, GlobalSet, MemoryCopy), or have
/// unknown effects (Call*) are considered side-effectful and must be retained
/// even if their result is unused — removing them would suppress a Wasm trap.
pub fn is_side_effect_free(instr: &IrInstr) -> bool {
    match instr {
        // Integer division and remainder trap on divisor == 0 (and i*::MIN / -1
        // for signed division). Must be preserved even when the result is dead.
        IrInstr::BinOp { op, .. } => !matches!(
            op,
            BinOp::I32DivS
                | BinOp::I32DivU
                | BinOp::I32RemS
                | BinOp::I32RemU
                | BinOp::I64DivS
                | BinOp::I64DivU
                | BinOp::I64RemS
                | BinOp::I64RemU
        ),
        // Float-to-integer truncations trap on NaN or out-of-range inputs.
        IrInstr::UnOp { op, .. } => !matches!(
            op,
            UnOp::I32TruncF32S
                | UnOp::I32TruncF32U
                | UnOp::I32TruncF64S
                | UnOp::I32TruncF64U
                | UnOp::I64TruncF32S
                | UnOp::I64TruncF32U
                | UnOp::I64TruncF64S
                | UnOp::I64TruncF64U
        ),
        IrInstr::Const { .. }
        | IrInstr::Assign { .. }
        | IrInstr::Select { .. }
        | IrInstr::GlobalGet { .. }
        | IrInstr::MemorySize { .. } => true,
        IrInstr::Phi { .. } => false,
        _ => false,
    }
}

// ── Commutative op detection ─────────────────────────────────────────────────

/// Returns true for operations where `op(a, b) == op(b, a)`.
pub fn is_commutative(op: &BinOp) -> bool {
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

// ── Rewrite terminator block targets ─────────────────────────────────────────

/// Rewrite all block-ID references in a terminator from `old` to `new`.
pub fn rewrite_terminator_target(term: &mut IrTerminator, old: BlockId, new: BlockId) {
    let replace = |b: &mut BlockId| {
        if *b == old {
            *b = new;
        }
    };
    match term {
        IrTerminator::Jump { target } => replace(target),
        IrTerminator::BranchIf {
            if_true, if_false, ..
        } => {
            replace(if_true);
            replace(if_false);
        }
        IrTerminator::BranchTable {
            targets, default, ..
        } => {
            for t in targets.iter_mut() {
                replace(t);
            }
            replace(default);
        }
        IrTerminator::Return { .. } | IrTerminator::Unreachable => {}
    }
}

/// Returns `true` if `var` is known to be zero according to `consts`.
pub fn is_zero(var: VarId, consts: &HashMap<VarId, IrValue>) -> bool {
    matches!(
        consts.get(&var),
        Some(IrValue::I32(0)) | Some(IrValue::I64(0))
    )
}

/// Variables with exactly one definition across the function that is a `Const`
/// instruction. These can be treated as constants in any block that uses them.
pub fn build_global_const_map(func: &IrFunction) -> HashMap<VarId, IrValue> {
    // Count total definitions per variable (any instruction with a dest).
    let mut total_defs: HashMap<VarId, usize> = HashMap::new();
    let mut const_defs: HashMap<VarId, IrValue> = HashMap::new();

    for_each_instr(func, |instr| {
        if let Some(dest) = instr_dest(instr) {
            *total_defs.entry(dest).or_insert(0) += 1;
            if let IrInstr::Const { dest, value } = instr {
                const_defs.insert(*dest, *value);
            }
        }
    });

    // In strict SSA form, each variable is defined at most once, so count should
    // be 0 or 1. We check count == 1 defensively: only include variables whose
    // sole definition is a Const instruction, ensuring we don't propagate a
    // variable that is never defined or has multiple definitions (which would
    // violate SSA invariants).

    // Only include variables whose sole definition is a Const instruction.
    const_defs
        .into_iter()
        .filter(|(v, _)| total_defs.get(v).copied().unwrap_or(0) == 1)
        .collect()
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
    fn trapping_binops_not_side_effect_free() {
        // Integer div/rem must NOT be classified as side-effect-free because they
        // can trap at runtime (division by zero, i*::MIN / -1 for signed div).
        for op in [
            BinOp::I32DivS,
            BinOp::I32DivU,
            BinOp::I32RemS,
            BinOp::I32RemU,
            BinOp::I64DivS,
            BinOp::I64DivU,
            BinOp::I64RemS,
            BinOp::I64RemU,
        ] {
            let instr = IrInstr::BinOp {
                dest: VarId(0),
                op,
                lhs: VarId(1),
                rhs: VarId(2),
            };
            assert!(
                !is_side_effect_free(&instr),
                "{op:?} should NOT be side-effect-free"
            );
        }
        // Non-trapping BinOps remain side-effect-free.
        assert!(is_side_effect_free(&IrInstr::BinOp {
            dest: VarId(0),
            op: BinOp::I32Mul,
            lhs: VarId(1),
            rhs: VarId(2),
        }));
    }

    #[test]
    fn trapping_unops_not_side_effect_free() {
        use crate::ir::UnOp;
        // Float-to-integer truncations trap on NaN or out-of-range values.
        for op in [
            UnOp::I32TruncF32S,
            UnOp::I32TruncF32U,
            UnOp::I32TruncF64S,
            UnOp::I32TruncF64U,
            UnOp::I64TruncF32S,
            UnOp::I64TruncF32U,
            UnOp::I64TruncF64S,
            UnOp::I64TruncF64U,
        ] {
            let instr = IrInstr::UnOp {
                dest: VarId(0),
                op,
                operand: VarId(1),
            };
            assert!(
                !is_side_effect_free(&instr),
                "{op:?} should NOT be side-effect-free"
            );
        }
        // Non-trapping UnOp remains side-effect-free.
        assert!(is_side_effect_free(&IrInstr::UnOp {
            dest: VarId(0),
            op: UnOp::I32Clz,
            operand: VarId(1),
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
