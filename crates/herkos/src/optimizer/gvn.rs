//! Global value numbering (GVN) — cross-block CSE using the dominator tree.
//!
//! Extends block-local CSE ([`super::local_cse`]) to work across basic blocks.
//! If block A dominates block B (every path to B passes through A), then any
//! pure computation defined in A with the same value key as one in B can be
//! reused in B instead of recomputing.
//!
//! ## Algorithm
//!
//! 1. Compute the immediate dominator of each block (Cooper/Harvey/Kennedy
//!    iterative algorithm) to build the dominator tree.
//! 2. Walk the dominator tree in preorder using a scoped value-number table.
//!    On entry to a block, push a new scope; on exit, pop it.
//! 3. For each pure instruction (`Const`, `BinOp`, `UnOp`) in the current
//!    block, compute a value key.  If the key already exists in any enclosing
//!    scope (meaning it was computed in a dominating block), record a
//!    replacement: `dest → first_var`.  Otherwise insert the key into the
//!    current scope.
//! 4. After the walk, rewrite all recorded destinations to
//!    `Assign { dest, src: first_var }` and let copy-propagation clean up.
//!
//! **Only pure instructions are eligible.**  Loads, calls, and memory ops are
//! never deduplicated (they may trap or have observable side effects).

use super::utils::{build_predecessors, instr_dest, prune_dead_locals, terminator_successors};
use crate::ir::{BinOp, BlockId, IrFunction, IrInstr, IrValue, UnOp, VarId};
use std::collections::{HashMap, HashSet};

// ── Value key ────────────────────────────────────────────────────────────────

/// Hashable representation of a pure computation for deduplication.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ValueKey {
    Const(ConstKey),
    BinOp { op: BinOp, lhs: VarId, rhs: VarId },
    UnOp { op: UnOp, operand: VarId },
}

/// Bit-level constant key that implements `Eq`/`Hash` correctly for floats.
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

fn binop_key(op: BinOp, lhs: VarId, rhs: VarId) -> ValueKey {
    let (lhs, rhs) = if is_commutative(&op) && lhs.0 > rhs.0 {
        (rhs, lhs)
    } else {
        (lhs, rhs)
    };
    ValueKey::BinOp { op, lhs, rhs }
}

// ── Multi-definition detection ───────────────────────────────────────────────

/// Build the set of variables defined more than once across the function.
///
/// After phi lowering the code is no longer in strict SSA form: loop phi
/// variables receive an initial assignment in the pre-loop block and a
/// back-edge update at the end of each iteration.  These variables carry
/// different values at different program points, so any BinOp/UnOp that uses
/// them cannot be safely hoisted or deduplicated across blocks.
///
/// `Const` instructions are always safe (they have no operands).
fn build_multi_def_vars(func: &IrFunction) -> HashSet<VarId> {
    let mut def_count: HashMap<VarId, usize> = HashMap::new();
    for block in &func.blocks {
        for instr in &block.instructions {
            if let Some(dest) = instr_dest(instr) {
                *def_count.entry(dest).or_insert(0) += 1;
            }
        }
    }
    def_count
        .into_iter()
        .filter(|&(_, count)| count > 1)
        .map(|(v, _)| v)
        .collect()
}

// ── Dominator tree ───────────────────────────────────────────────────────────

/// Compute the reverse-postorder traversal of the CFG starting from `entry`.
fn compute_rpo(func: &IrFunction) -> Vec<BlockId> {
    let block_idx: HashMap<BlockId, usize> =
        func.blocks.iter().enumerate().map(|(i, b)| (b.id, i)).collect();

    let mut visited = vec![false; func.blocks.len()];
    let mut postorder = Vec::with_capacity(func.blocks.len());

    dfs_postorder(func, func.entry_block, &block_idx, &mut visited, &mut postorder);

    postorder.reverse();
    postorder
}

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

/// Compute the immediate dominator of each block using Cooper/Harvey/Kennedy.
///
/// Returns `idom[b] = immediate dominator of b`, with `idom[entry] = entry`.
fn compute_idoms(func: &IrFunction) -> HashMap<BlockId, BlockId> {
    let rpo = compute_rpo(func);
    // rpo_num[b] = index in RPO order (entry = 0, smallest index = processed first)
    let rpo_num: HashMap<BlockId, usize> =
        rpo.iter().enumerate().map(|(i, &b)| (b, i)).collect();

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

/// Walk up both fingers until they meet — the standard Cooper intersect.
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

/// Build dominator-tree children from the `idom` map.
fn build_dom_children(
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

// ── GVN walk ─────────────────────────────────────────────────────────────────

/// Recursively walk the dominator tree in preorder.
///
/// `value_map` is a flat map that acts as a scoped table: on entry we insert
/// new keys (recording them in `frame_keys`), on exit we remove them, restoring
/// the parent scope.  Any key already present in `value_map` when we visit a
/// block was computed in a dominating block — safe to reuse.
fn collect_replacements(
    func: &IrFunction,
    block_id: BlockId,
    dom_children: &HashMap<BlockId, Vec<BlockId>>,
    block_idx: &HashMap<BlockId, usize>,
    multi_def_vars: &HashSet<VarId>,
    value_map: &mut HashMap<ValueKey, VarId>,
    replacements: &mut HashMap<VarId, VarId>,
) {
    let idx = match block_idx.get(&block_id) {
        Some(&i) => i,
        None => return,
    };

    let mut frame_keys: Vec<ValueKey> = Vec::new();

    for instr in &func.blocks[idx].instructions {
        match instr {
            IrInstr::Const { dest, value } => {
                // A multiply-defined dest (loop phi var) must be skipped
                // entirely: adding it to replacements would replace ALL of
                // its definitions with Assign(first), clobbering back-edge
                // updates; inserting it into value_map would let dominated
                // blocks wrongly reuse a value that changes each iteration.
                if multi_def_vars.contains(dest) {
                    continue;
                }
                let key = ValueKey::Const(ConstKey::from(*value));
                if let Some(&first) = value_map.get(&key) {
                    replacements.insert(*dest, first);
                } else {
                    value_map.insert(key.clone(), *dest);
                    frame_keys.push(key);
                }
            }

            IrInstr::BinOp { dest, op, lhs, rhs, .. } => {
                // Skip if dest is multiply-defined (same reason as Const).
                // Also skip if any operand is multiply-defined: a loop phi
                // var carries different values per iteration, so the same
                // BinOp in two dominated blocks can produce different results.
                if multi_def_vars.contains(dest)
                    || multi_def_vars.contains(lhs)
                    || multi_def_vars.contains(rhs)
                {
                    continue;
                }
                let key = binop_key(*op, *lhs, *rhs);
                if let Some(&first) = value_map.get(&key) {
                    replacements.insert(*dest, first);
                } else {
                    value_map.insert(key.clone(), *dest);
                    frame_keys.push(key);
                }
            }

            IrInstr::UnOp { dest, op, operand } => {
                if multi_def_vars.contains(dest) || multi_def_vars.contains(operand) {
                    continue;
                }
                let key = ValueKey::UnOp { op: *op, operand: *operand };
                if let Some(&first) = value_map.get(&key) {
                    replacements.insert(*dest, first);
                } else {
                    value_map.insert(key.clone(), *dest);
                    frame_keys.push(key);
                }
            }

            _ => {}
        }
    }

    // Recurse into dominated children.
    if let Some(children) = dom_children.get(&block_id) {
        for &child in children {
            collect_replacements(
                func,
                child,
                dom_children,
                block_idx,
                multi_def_vars,
                value_map,
                replacements,
            );
        }
    }

    // Pop this block's scope.
    for key in frame_keys {
        value_map.remove(&key);
    }
}

// ── Pass entry point ─────────────────────────────────────────────────────────

/// Eliminates common subexpressions across basic blocks using the dominator tree.
pub fn eliminate(func: &mut IrFunction) {
    if func.blocks.len() < 2 {
        return; // nothing to do for single-block functions (local_cse covers those)
    }

    let idom = compute_idoms(func);
    let dom_children = build_dom_children(&idom, func.entry_block);
    let block_idx: HashMap<BlockId, usize> =
        func.blocks.iter().enumerate().map(|(i, b)| (b.id, i)).collect();

    let multi_def_vars = build_multi_def_vars(func);
    let mut value_map: HashMap<ValueKey, VarId> = HashMap::new();
    let mut replacements: HashMap<VarId, VarId> = HashMap::new();

    collect_replacements(
        func,
        func.entry_block,
        &dom_children,
        &block_idx,
        &multi_def_vars,
        &mut value_map,
        &mut replacements,
    );

    if replacements.is_empty() {
        return;
    }

    for block in &mut func.blocks {
        for instr in &mut block.instructions {
            if let Some(dest) = instr_dest(instr) {
                if let Some(&src) = replacements.get(&dest) {
                    *instr = IrInstr::Assign { dest, src };
                }
            }
        }
    }

    prune_dead_locals(func);
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IrBlock, IrTerminator, IrValue, TypeIdx, WasmType};

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

    /// Entry (B0) → B1: const duplicated across the edge.
    /// B0 dominates B1, so the duplicate in B1 should be replaced with Assign.
    #[test]
    fn cross_block_const_deduplication() {
        let b0 = IrBlock {
            id: BlockId(0),
            instructions: vec![IrInstr::Const { dest: VarId(0), value: IrValue::I32(42) }],
            terminator: IrTerminator::Jump { target: BlockId(1) },
        };
        let b1 = IrBlock {
            id: BlockId(1),
            instructions: vec![IrInstr::Const { dest: VarId(1), value: IrValue::I32(42) }],
            terminator: IrTerminator::Return { value: Some(VarId(1)) },
        };
        let mut func = make_func(vec![b0, b1]);
        func.locals = vec![(VarId(0), WasmType::I32), (VarId(1), WasmType::I32)];

        eliminate(&mut func);

        assert!(
            matches!(func.blocks[0].instructions[0], IrInstr::Const { dest: VarId(0), .. }),
            "first definition should stay as Const"
        );
        assert!(
            matches!(
                func.blocks[1].instructions[0],
                IrInstr::Assign { dest: VarId(1), src: VarId(0) }
            ),
            "dominated duplicate should become Assign"
        );
    }

    /// Entry (B0) → B1: BinOp duplicated across the edge.
    #[test]
    fn cross_block_binop_deduplication() {
        let b0 = IrBlock {
            id: BlockId(0),
            instructions: vec![IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            }],
            terminator: IrTerminator::Jump { target: BlockId(1) },
        };
        let b1 = IrBlock {
            id: BlockId(1),
            instructions: vec![IrInstr::BinOp {
                dest: VarId(3),
                op: BinOp::I32Add,
                lhs: VarId(0),
                rhs: VarId(1),
            }],
            terminator: IrTerminator::Return { value: Some(VarId(3)) },
        };
        let mut func = make_func(vec![b0, b1]);
        func.locals = vec![(VarId(2), WasmType::I32), (VarId(3), WasmType::I32)];

        eliminate(&mut func);

        assert!(matches!(func.blocks[0].instructions[0], IrInstr::BinOp { .. }));
        assert!(
            matches!(
                func.blocks[1].instructions[0],
                IrInstr::Assign { dest: VarId(3), src: VarId(2) }
            ),
            "dominated duplicate BinOp should become Assign"
        );
    }

    /// B0 branches to B1 and B2 (diamond). B1 and B2 don't dominate each other,
    /// so a const in B1 should NOT eliminate the same const in B2.
    #[test]
    fn sibling_blocks_not_deduplicated() {
        // B0 → B1, B0 → B2, both converge to B3
        let b0 = IrBlock {
            id: BlockId(0),
            instructions: vec![],
            terminator: IrTerminator::BranchIf {
                condition: VarId(0),
                if_true: BlockId(1),
                if_false: BlockId(2),
            },
        };
        let b1 = IrBlock {
            id: BlockId(1),
            instructions: vec![IrInstr::Const { dest: VarId(1), value: IrValue::I32(7) }],
            terminator: IrTerminator::Jump { target: BlockId(3) },
        };
        let b2 = IrBlock {
            id: BlockId(2),
            instructions: vec![IrInstr::Const { dest: VarId(2), value: IrValue::I32(7) }],
            terminator: IrTerminator::Jump { target: BlockId(3) },
        };
        let b3 = IrBlock {
            id: BlockId(3),
            instructions: vec![],
            terminator: IrTerminator::Return { value: None },
        };
        let mut func = make_func(vec![b0, b1, b2, b3]);
        func.locals = vec![(VarId(1), WasmType::I32), (VarId(2), WasmType::I32)];

        eliminate(&mut func);

        // Both consts should remain — neither block dominates the other.
        assert!(
            matches!(func.blocks[1].instructions[0], IrInstr::Const { dest: VarId(1), .. }),
            "const in B1 must not be eliminated"
        );
        assert!(
            matches!(func.blocks[2].instructions[0], IrInstr::Const { dest: VarId(2), .. }),
            "const in B2 must not be eliminated"
        );
    }

    /// A const defined in B0 (entry) should be reused in a deeply dominated block.
    #[test]
    fn deep_domination_chain() {
        // B0 → B1 → B2: const defined in B0, duplicated in B2
        let b0 = IrBlock {
            id: BlockId(0),
            instructions: vec![IrInstr::Const { dest: VarId(0), value: IrValue::I32(99) }],
            terminator: IrTerminator::Jump { target: BlockId(1) },
        };
        let b1 = IrBlock {
            id: BlockId(1),
            instructions: vec![],
            terminator: IrTerminator::Jump { target: BlockId(2) },
        };
        let b2 = IrBlock {
            id: BlockId(2),
            instructions: vec![IrInstr::Const { dest: VarId(1), value: IrValue::I32(99) }],
            terminator: IrTerminator::Return { value: Some(VarId(1)) },
        };
        let mut func = make_func(vec![b0, b1, b2]);
        func.locals = vec![(VarId(0), WasmType::I32), (VarId(1), WasmType::I32)];

        eliminate(&mut func);

        assert!(
            matches!(func.blocks[0].instructions[0], IrInstr::Const { dest: VarId(0), .. })
        );
        assert!(
            matches!(
                func.blocks[2].instructions[0],
                IrInstr::Assign { dest: VarId(1), src: VarId(0) }
            ),
            "deeply dominated duplicate should be eliminated"
        );
    }

    /// Commutative BinOps with swapped operands in a dominated block should be deduped.
    #[test]
    fn cross_block_commutative_deduplication() {
        let b0 = IrBlock {
            id: BlockId(0),
            instructions: vec![IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Mul,
                lhs: VarId(0),
                rhs: VarId(1),
            }],
            terminator: IrTerminator::Jump { target: BlockId(1) },
        };
        let b1 = IrBlock {
            id: BlockId(1),
            instructions: vec![IrInstr::BinOp {
                dest: VarId(3),
                op: BinOp::I32Mul,
                lhs: VarId(1), // swapped
                rhs: VarId(0),
            }],
            terminator: IrTerminator::Return { value: Some(VarId(3)) },
        };
        let mut func = make_func(vec![b0, b1]);
        func.locals = vec![(VarId(2), WasmType::I32), (VarId(3), WasmType::I32)];

        eliminate(&mut func);

        assert!(
            matches!(
                func.blocks[1].instructions[0],
                IrInstr::Assign { dest: VarId(3), src: VarId(2) }
            ),
            "commutative cross-block BinOp should be deduplicated"
        );
    }

    /// Single-block functions are skipped entirely (handled by local_cse).
    #[test]
    fn single_block_function_unchanged() {
        let b0 = IrBlock {
            id: BlockId(0),
            instructions: vec![
                IrInstr::Const { dest: VarId(0), value: IrValue::I32(1) },
                IrInstr::Const { dest: VarId(1), value: IrValue::I32(1) },
            ],
            terminator: IrTerminator::Return { value: None },
        };
        let mut func = make_func(vec![b0]);
        func.locals = vec![(VarId(0), WasmType::I32), (VarId(1), WasmType::I32)];

        eliminate(&mut func);

        // GVN skips single-block functions; duplicates remain (local_cse's job).
        assert!(matches!(func.blocks[0].instructions[0], IrInstr::Const { .. }));
        assert!(matches!(func.blocks[0].instructions[1], IrInstr::Const { .. }));
    }
}
