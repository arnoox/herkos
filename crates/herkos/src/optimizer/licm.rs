//! Loop-invariant code motion (LICM).
//!
//! Identifies instructions in loop headers whose operands don't change across
//! iterations, and moves them to a preheader block.
//!
//! ## Algorithm
//!
//! 1. Compute dominators (iterative algorithm)
//! 2. Find back edges: edge (src → tgt) where tgt dominates src
//! 3. Find natural loops: for each back edge, collect all blocks that reach
//!    the source without going through the header
//! 4. For each loop, identify invariant instructions in the header (fixpoint):
//!    - `Const` — trivially invariant
//!    - `BinOp`, `UnOp`, `Assign`, `Select` — invariant if all operands are
//!      defined outside the loop or by other invariant instructions
//!    - Skip: `Load`, `Store`, `Call*`, `Global*`, `Memory*`
//! 5. Create or reuse a preheader block and move invariant instructions there
//!
//! **V1 simplification:** only hoists from the loop header block (which
//! dominates all loop blocks by definition).

use super::utils::{
    build_predecessors, for_each_use, instr_dest, rewrite_terminator_target, terminator_successors,
};
use crate::ir::{BlockId, IrBlock, IrFunction, IrInstr, IrTerminator, VarId};
use std::collections::{HashMap, HashSet};

/// Run loop-invariant code motion on `func`.
pub fn eliminate(func: &mut IrFunction) {
    if func.blocks.len() < 2 {
        return;
    }

    let preds = build_predecessors(func);
    let dominators = compute_dominators(func, &preds);
    let back_edges = find_back_edges(func, &dominators);

    if back_edges.is_empty() {
        return;
    }

    let loops = find_natural_loops(&back_edges, &preds);

    for (header, loop_blocks) in &loops {
        hoist_invariants(func, *header, loop_blocks);
    }
}

// ── Dominator computation ────────────────────────────────────────────────────

/// Compute the dominator set for each block using the iterative algorithm.
///
/// Returns a map from each block to the set of blocks that dominate it.
fn compute_dominators(
    func: &IrFunction,
    preds: &HashMap<BlockId, HashSet<BlockId>>,
) -> HashMap<BlockId, HashSet<BlockId>> {
    let entry = func.entry_block;
    let all_block_ids: HashSet<BlockId> = func.blocks.iter().map(|b| b.id).collect();

    let mut dom: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
    dom.insert(entry, HashSet::from([entry]));

    for block in &func.blocks {
        if block.id != entry {
            dom.insert(block.id, all_block_ids.clone());
        }
    }

    loop {
        let mut changed = false;
        for block in &func.blocks {
            if block.id == entry {
                continue;
            }
            let pred_set = &preds[&block.id];
            if pred_set.is_empty() {
                continue;
            }

            // new_dom = {self} ∪ ∩(dom[p] for p in preds)
            let mut new_dom: Option<HashSet<BlockId>> = None;
            for p in pred_set {
                if let Some(p_dom) = dom.get(p) {
                    new_dom = Some(match new_dom {
                        None => p_dom.clone(),
                        Some(current) => current.intersection(p_dom).copied().collect(),
                    });
                }
            }

            let mut new_dom = new_dom.unwrap_or_default();
            new_dom.insert(block.id);

            if new_dom != dom[&block.id] {
                dom.insert(block.id, new_dom);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    dom
}

// ── Back edge detection ──────────────────────────────────────────────────────

/// Find all back edges in the CFG.
///
/// A back edge is (src, tgt) where tgt dominates src.
fn find_back_edges(
    func: &IrFunction,
    dominators: &HashMap<BlockId, HashSet<BlockId>>,
) -> Vec<(BlockId, BlockId)> {
    let mut back_edges = Vec::new();
    for block in &func.blocks {
        for succ in terminator_successors(&block.terminator) {
            if dominators
                .get(&block.id)
                .is_some_and(|dom_set| dom_set.contains(&succ))
            {
                back_edges.push((block.id, succ));
            }
        }
    }
    back_edges
}

// ── Natural loop detection ───────────────────────────────────────────────────

/// Find natural loops from back edges.
///
/// For each back edge (src → header), collects all blocks that can reach `src`
/// without going through `header`. Multiple back edges with the same header
/// are merged into one loop.
fn find_natural_loops(
    back_edges: &[(BlockId, BlockId)],
    preds: &HashMap<BlockId, HashSet<BlockId>>,
) -> Vec<(BlockId, HashSet<BlockId>)> {
    let mut loops: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();

    for &(src, header) in back_edges {
        let loop_blocks = loops.entry(header).or_insert_with(|| {
            let mut set = HashSet::new();
            set.insert(header);
            set
        });

        let mut worklist = vec![src];
        while let Some(n) = worklist.pop() {
            if loop_blocks.insert(n) {
                if let Some(n_preds) = preds.get(&n) {
                    for &p in n_preds {
                        worklist.push(p);
                    }
                }
            }
        }
    }

    loops.into_iter().collect()
}

// ── Invariant identification & hoisting ──────────────────────────────────────

/// Returns `true` if the instruction type is eligible for LICM hoisting.
///
/// Only pure, side-effect-free computations are hoistable. Instructions that
/// depend on mutable state (`Global*`, `Memory*`) or have side effects
/// (`Load`, `Store`, `Call*`) are excluded.
fn is_licm_hoistable(instr: &IrInstr) -> bool {
    matches!(
        instr,
        IrInstr::Const { .. }
            | IrInstr::BinOp { .. }
            | IrInstr::UnOp { .. }
            | IrInstr::Assign { .. }
            | IrInstr::Select { .. }
    )
}

/// Identify loop-invariant instructions in the header and hoist them to a preheader.
fn hoist_invariants(func: &mut IrFunction, header: BlockId, loop_blocks: &HashSet<BlockId>) {
    let header_idx = match func.blocks.iter().position(|b| b.id == header) {
        Some(idx) => idx,
        None => return,
    };

    // Collect all VarIds defined in any loop block.
    let mut loop_defs: HashSet<VarId> = HashSet::new();
    for block in &func.blocks {
        if loop_blocks.contains(&block.id) {
            for instr in &block.instructions {
                if let Some(dest) = instr_dest(instr) {
                    loop_defs.insert(dest);
                }
            }
        }
    }

    // Fixpoint: identify invariant instructions in the header.
    let mut invariant_dests: HashSet<VarId> = HashSet::new();
    loop {
        let mut changed = false;
        for instr in &func.blocks[header_idx].instructions {
            if !is_licm_hoistable(instr) {
                continue;
            }
            let dest = match instr_dest(instr) {
                Some(d) => d,
                None => continue,
            };
            if invariant_dests.contains(&dest) {
                continue;
            }

            let mut all_ops_invariant = true;
            for_each_use(instr, |v| {
                if loop_defs.contains(&v) && !invariant_dests.contains(&v) {
                    all_ops_invariant = false;
                }
            });

            if all_ops_invariant {
                invariant_dests.insert(dest);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    if invariant_dests.is_empty() {
        return;
    }

    // Find or create preheader.
    let preheader_id = find_or_create_preheader(func, header, loop_blocks);

    // Re-lookup indices after possible block insertion.
    let header_idx = func.blocks.iter().position(|b| b.id == header).unwrap();
    let preheader_idx = func
        .blocks
        .iter()
        .position(|b| b.id == preheader_id)
        .unwrap();

    // Move invariant instructions from header to preheader (in order).
    let mut hoisted = Vec::new();
    let mut remaining = Vec::new();

    for instr in func.blocks[header_idx].instructions.drain(..) {
        if let Some(dest) = instr_dest(&instr) {
            if invariant_dests.contains(&dest) {
                hoisted.push(instr);
                continue;
            }
        }
        remaining.push(instr);
    }

    func.blocks[header_idx].instructions = remaining;
    func.blocks[preheader_idx].instructions.extend(hoisted);
}

/// Allocate a fresh `BlockId` that doesn't conflict with existing blocks.
fn fresh_block_id(func: &IrFunction) -> BlockId {
    let max_id = func.blocks.iter().map(|b| b.id.0).max().unwrap_or(0);
    BlockId(max_id + 1)
}

/// Find an existing preheader or create a new one.
///
/// A preheader is reused if it is the sole non-loop predecessor and ends
/// with an unconditional jump to the header. Otherwise a new preheader
/// block is created and non-loop predecessors are redirected to it.
fn find_or_create_preheader(
    func: &mut IrFunction,
    header: BlockId,
    loop_blocks: &HashSet<BlockId>,
) -> BlockId {
    let preds = build_predecessors(func);
    let header_preds = &preds[&header];
    let non_loop_preds: Vec<BlockId> = header_preds
        .iter()
        .filter(|p| !loop_blocks.contains(p))
        .copied()
        .collect();

    if non_loop_preds.is_empty() {
        // Header has no non-loop predecessors (entry block or unreachable from outside).
        let preheader_id = fresh_block_id(func);
        func.blocks.push(IrBlock {
            id: preheader_id,
            instructions: vec![],
            terminator: IrTerminator::Jump { target: header },
        });
        if header == func.entry_block {
            func.entry_block = preheader_id;
        }
        return preheader_id;
    }

    // Reuse if single non-loop predecessor with unconditional jump to header.
    if non_loop_preds.len() == 1 {
        let pred = non_loop_preds[0];
        let pred_idx = func.blocks.iter().position(|b| b.id == pred).unwrap();
        if matches!(func.blocks[pred_idx].terminator, IrTerminator::Jump { target } if target == header)
        {
            return pred;
        }
    }

    // Create a new preheader and redirect non-loop predecessors.
    let preheader_id = fresh_block_id(func);
    func.blocks.push(IrBlock {
        id: preheader_id,
        instructions: vec![],
        terminator: IrTerminator::Jump { target: header },
    });

    for block in &mut func.blocks {
        if non_loop_preds.contains(&block.id) {
            rewrite_terminator_target(&mut block.terminator, header, preheader_id);
        }
    }

    preheader_id
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{
        BinOp, IrBlock, IrFunction, IrInstr, IrTerminator, IrValue, TypeIdx, VarId, WasmType,
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

    // ── No loops → no changes ────────────────────────────────────────────

    #[test]
    fn no_loop_no_change() {
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(42),
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

        // No loops, so the const stays in block 0.
        assert_eq!(func.blocks[0].instructions.len(), 1);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const { dest: VarId(0), .. }
        ));
    }

    // ── Simple loop: const in header → hoisted to preheader ──────────────

    #[test]
    fn simple_loop_const_hoisted() {
        // B0 (entry): Jump(B1)
        // B1 (header): v0 = Const(42), BranchIf(v1, B2, B3)
        // B2 (body): Jump(B1)   ← back edge
        // B3 (exit): Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(42),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(1),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // B0 is the sole non-loop predecessor with Jump → reused as preheader.
        // v0 = Const(42) should be hoisted to B0.
        assert_eq!(func.blocks[0].instructions.len(), 1);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(42),
            }
        ));

        // B1 (header) should have no instructions.
        let header = func.blocks.iter().find(|b| b.id == BlockId(1)).unwrap();
        assert_eq!(header.instructions.len(), 0);
    }

    // ── BinOp with operands from outside loop → hoisted ──────────────────

    #[test]
    fn invariant_binop_hoisted() {
        // B0 (entry): v0 = Const(10), v1 = Const(20), Jump(B1)
        // B1 (header): v2 = BinOp::Add(v0, v1), BranchIf(v3, B2, B3)
        // B2 (body): Jump(B1)
        // B3 (exit): Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![
                    IrInstr::Const {
                        dest: VarId(0),
                        value: IrValue::I32(10),
                    },
                    IrInstr::Const {
                        dest: VarId(1),
                        value: IrValue::I32(20),
                    },
                ],
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
                terminator: IrTerminator::BranchIf {
                    condition: VarId(3),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // v2 = BinOp should be hoisted to B0 (preheader).
        assert_eq!(func.blocks[0].instructions.len(), 3);
        assert!(matches!(
            func.blocks[0].instructions[2],
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Add,
                ..
            }
        ));

        // Header should be empty.
        let header = func.blocks.iter().find(|b| b.id == BlockId(1)).unwrap();
        assert_eq!(header.instructions.len(), 0);
    }

    // ── Chained invariants: const → binop using that const ───────────────

    #[test]
    fn chained_invariants_hoisted() {
        // B0 (entry): v0 = Const(10), Jump(B1)
        // B1 (header): v1 = Const(65536), v2 = BinOp::Add(v0, v1), BranchIf(v3, B2, B3)
        // B2 (body): Jump(B1)
        // B3 (exit): Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(10),
                }],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![
                    IrInstr::Const {
                        dest: VarId(1),
                        value: IrValue::I32(65536),
                    },
                    IrInstr::BinOp {
                        dest: VarId(2),
                        op: BinOp::I32Add,
                        lhs: VarId(0),
                        rhs: VarId(1),
                    },
                ],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(3),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // Both v1 = Const and v2 = BinOp should be hoisted to B0.
        // B0 now has: v0 = Const(10), v1 = Const(65536), v2 = Add(v0, v1).
        assert_eq!(func.blocks[0].instructions.len(), 3);
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(65536),
            }
        ));
        assert!(matches!(
            func.blocks[0].instructions[2],
            IrInstr::BinOp {
                dest: VarId(2),
                op: BinOp::I32Add,
                ..
            }
        ));

        // Header should be empty.
        let header = func.blocks.iter().find(|b| b.id == BlockId(1)).unwrap();
        assert_eq!(header.instructions.len(), 0);
    }

    // ── Non-hoistable instructions stay in the header ────────────────────

    #[test]
    fn side_effectful_not_hoisted() {
        use crate::ir::MemoryAccessWidth;

        // B0: Jump(B1)
        // B1 (header): v0 = Const(0), v1 = Load(v0), BranchIf(v2, B2, B3)
        // B2: Jump(B1)
        // B3: Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![
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
                terminator: IrTerminator::BranchIf {
                    condition: VarId(2),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // v0 = Const is hoisted (invariant), but Load stays (not hoistable).
        assert_eq!(func.blocks[0].instructions.len(), 1);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const { dest: VarId(0), .. }
        ));

        let header = func.blocks.iter().find(|b| b.id == BlockId(1)).unwrap();
        assert_eq!(header.instructions.len(), 1);
        assert!(matches!(header.instructions[0], IrInstr::Load { .. }));
    }

    // ── BinOp with operand from loop body → NOT hoisted ──────────────────

    #[test]
    fn loop_dependent_not_hoisted() {
        // B0: v0 = Const(1), Jump(B1)
        // B1 (header): v2 = BinOp::Add(v0, v1), BranchIf(v3, B2, B3)
        //              v1 is defined in B2 (loop body) → v2 is NOT invariant
        // B2: v1 = Const(5), Jump(B1)
        // B3: Return
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
                instructions: vec![IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Add,
                    lhs: VarId(0),
                    rhs: VarId(1),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(3),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(5),
                }],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // v2 = BinOp should NOT be hoisted because v1 is defined in B2 (loop body).
        let header = func.blocks.iter().find(|b| b.id == BlockId(1)).unwrap();
        assert_eq!(header.instructions.len(), 1);
        assert!(matches!(header.instructions[0], IrInstr::BinOp { .. }));
    }

    // ── Preheader reuse: single non-loop predecessor with Jump ───────────

    #[test]
    fn preheader_reused_when_possible() {
        // B0 (entry): v0 = Const(99), Jump(B1)
        // B1 (header): v1 = Const(42), BranchIf(v2, B2, B3)
        // B2: Jump(B1)
        // B3: Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(99),
                }],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(42),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(2),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // B0 should be reused as preheader (sole non-loop pred with Jump).
        // No new blocks should be created.
        assert_eq!(func.blocks.len(), 4);
        assert_eq!(func.blocks[0].instructions.len(), 2);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(99),
            }
        ));
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(42),
            }
        ));
    }

    // ── Preheader creation: multiple non-loop predecessors ───────────────

    #[test]
    fn preheader_created_when_needed() {
        // B0 (entry): BranchIf(v0, B1, B2)
        // B1: Jump(B3)
        // B2: Jump(B3)
        // B3 (header): v1 = Const(42), BranchIf(v2, B4, B5)
        // B4 (body): Jump(B3)   ← back edge
        // B5 (exit): Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(0),
                    if_true: BlockId(1),
                    if_false: BlockId(2),
                },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(3) },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(3) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(42),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(2),
                    if_true: BlockId(4),
                    if_false: BlockId(5),
                },
            },
            IrBlock {
                id: BlockId(4),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(3) },
            },
            IrBlock {
                id: BlockId(5),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // A new preheader (B6) should be created.
        assert_eq!(func.blocks.len(), 7);

        let preheader = func.blocks.iter().find(|b| b.id == BlockId(6)).unwrap();
        assert_eq!(preheader.instructions.len(), 1);
        assert!(matches!(
            preheader.instructions[0],
            IrInstr::Const {
                dest: VarId(1),
                value: IrValue::I32(42),
            }
        ));
        assert!(matches!(
            preheader.terminator,
            IrTerminator::Jump { target: BlockId(3) }
        ));

        // B1 and B2 should now jump to the preheader (B6).
        let b1 = func.blocks.iter().find(|b| b.id == BlockId(1)).unwrap();
        assert!(matches!(
            b1.terminator,
            IrTerminator::Jump { target: BlockId(6) }
        ));
        let b2 = func.blocks.iter().find(|b| b.id == BlockId(2)).unwrap();
        assert!(matches!(
            b2.terminator,
            IrTerminator::Jump { target: BlockId(6) }
        ));

        // Header (B3) should be empty.
        let header = func.blocks.iter().find(|b| b.id == BlockId(3)).unwrap();
        assert_eq!(header.instructions.len(), 0);
    }

    // ── GlobalGet not hoisted (depends on mutable state) ─────────────────

    #[test]
    fn global_get_not_hoisted() {
        use crate::ir::GlobalIdx;

        // B0: Jump(B1)
        // B1 (header): v0 = GlobalGet(0), BranchIf(v1, B2, B3)
        // B2: Jump(B1)
        // B3: Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::GlobalGet {
                    dest: VarId(0),
                    index: GlobalIdx::new(0),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(1),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // GlobalGet should NOT be hoisted (mutable global may change each iteration).
        let header = func.blocks.iter().find(|b| b.id == BlockId(1)).unwrap();
        assert_eq!(header.instructions.len(), 1);
        assert!(matches!(header.instructions[0], IrInstr::GlobalGet { .. }));
    }

    // ── Self-loop: header is also the back-edge source ───────────────────

    #[test]
    fn self_loop_const_hoisted() {
        // B0: Jump(B1)
        // B1: v0 = Const(42), BranchIf(v1, B1, B2)  ← self-loop
        // B2: Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(42),
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(1),
                    if_true: BlockId(1),
                    if_false: BlockId(2),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // Const should be hoisted to B0 (preheader).
        assert_eq!(func.blocks[0].instructions.len(), 1);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(42),
            }
        ));

        let header = func.blocks.iter().find(|b| b.id == BlockId(1)).unwrap();
        assert_eq!(header.instructions.len(), 0);
    }

    // ── No invariant instructions → no changes ───────────────────────────

    #[test]
    fn no_invariants_no_change() {
        use crate::ir::MemoryAccessWidth;

        // B0: v0 = Const(0), Jump(B1)
        // B1 (header): v1 = Load(v0), BranchIf(v2, B2, B3)
        // B2: Jump(B1)
        // B3: Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(0),
                }],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::Load {
                    dest: VarId(1),
                    ty: WasmType::I32,
                    addr: VarId(0),
                    offset: 0,
                    width: MemoryAccessWidth::Full,
                    sign: None,
                }],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(2),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // No invariants to hoist — no new blocks, header unchanged.
        assert_eq!(func.blocks.len(), 4);
        let header = func.blocks.iter().find(|b| b.id == BlockId(1)).unwrap();
        assert_eq!(header.instructions.len(), 1);
        assert!(matches!(header.instructions[0], IrInstr::Load { .. }));
    }

    // ── Entry block as loop header ───────────────────────────────────────

    #[test]
    fn entry_block_loop_header() {
        // B0 (entry/header): v0 = Const(42), BranchIf(v1, B1, B2)
        // B1 (body): Jump(B0)   ← back edge
        // B2 (exit): Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(42),
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
                terminator: IrTerminator::Jump { target: BlockId(0) },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // A preheader should be created, and entry_block updated.
        assert_eq!(func.blocks.len(), 4);
        let preheader_id = func.entry_block;
        assert_ne!(preheader_id, BlockId(0));

        let preheader = func.blocks.iter().find(|b| b.id == preheader_id).unwrap();
        assert_eq!(preheader.instructions.len(), 1);
        assert!(matches!(
            preheader.instructions[0],
            IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(42),
            }
        ));
        assert!(matches!(
            preheader.terminator,
            IrTerminator::Jump { target: BlockId(0) }
        ));

        // Original header (B0) should be empty.
        let header = func.blocks.iter().find(|b| b.id == BlockId(0)).unwrap();
        assert_eq!(header.instructions.len(), 0);
    }

    // ── Mixed: some hoistable, some not ──────────────────────────────────

    #[test]
    fn mixed_hoistable_and_non_hoistable() {
        use crate::ir::MemoryAccessWidth;

        // B0: Jump(B1)
        // B1 (header): v0 = Const(100), v1 = Load(v0), v2 = Const(200)
        //              BranchIf(v3, B2, B3)
        // B2: Jump(B1)
        // B3: Return
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![
                    IrInstr::Const {
                        dest: VarId(0),
                        value: IrValue::I32(100),
                    },
                    IrInstr::Load {
                        dest: VarId(1),
                        ty: WasmType::I32,
                        addr: VarId(0),
                        offset: 0,
                        width: MemoryAccessWidth::Full,
                        sign: None,
                    },
                    IrInstr::Const {
                        dest: VarId(2),
                        value: IrValue::I32(200),
                    },
                ],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(3),
                    if_true: BlockId(2),
                    if_false: BlockId(3),
                },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        eliminate(&mut func);

        // v0 and v2 (Consts) should be hoisted; Load stays.
        assert_eq!(func.blocks[0].instructions.len(), 2);
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(100),
            }
        ));
        assert!(matches!(
            func.blocks[0].instructions[1],
            IrInstr::Const {
                dest: VarId(2),
                value: IrValue::I32(200),
            }
        ));

        let header = func.blocks.iter().find(|b| b.id == BlockId(1)).unwrap();
        assert_eq!(header.instructions.len(), 1);
        assert!(matches!(header.instructions[0], IrInstr::Load { .. }));
    }

    // ── Single-block function → no change ────────────────────────────────

    #[test]
    fn single_block_function_no_change() {
        let mut func = make_func(vec![IrBlock {
            id: BlockId(0),
            instructions: vec![IrInstr::Const {
                dest: VarId(0),
                value: IrValue::I32(42),
            }],
            terminator: IrTerminator::Return {
                value: Some(VarId(0)),
            },
        }]);

        eliminate(&mut func);

        assert_eq!(func.blocks.len(), 1);
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    // ── Dominator computation tests ──────────────────────────────────────

    #[test]
    fn dominators_linear_chain() {
        // B0 → B1 → B2
        let func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(2) },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        let preds = build_predecessors(&func);
        let dom = compute_dominators(&func, &preds);

        assert_eq!(dom[&BlockId(0)], HashSet::from([BlockId(0)]));
        assert_eq!(dom[&BlockId(1)], HashSet::from([BlockId(0), BlockId(1)]));
        assert_eq!(
            dom[&BlockId(2)],
            HashSet::from([BlockId(0), BlockId(1), BlockId(2)])
        );
    }

    #[test]
    fn dominators_diamond() {
        // B0 → B1, B0 → B2, B1 → B3, B2 → B3
        let func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::BranchIf {
                    condition: VarId(0),
                    if_true: BlockId(1),
                    if_false: BlockId(2),
                },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(3) },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(3) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Return { value: None },
            },
        ]);

        let preds = build_predecessors(&func);
        let dom = compute_dominators(&func, &preds);

        // B3 is dominated by B0 (only common dominator of B1 and B2).
        assert_eq!(dom[&BlockId(3)], HashSet::from([BlockId(0), BlockId(3)]));
    }

    #[test]
    fn back_edges_detected() {
        // B0 → B1 → B2 → B1 (back edge)
        let func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(2) },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
        ]);

        let preds = build_predecessors(&func);
        let dom = compute_dominators(&func, &preds);
        let back_edges = find_back_edges(&func, &dom);

        assert_eq!(back_edges.len(), 1);
        assert_eq!(back_edges[0], (BlockId(2), BlockId(1)));
    }

    #[test]
    fn natural_loop_blocks() {
        // B0 → B1 → B2 → B3 → B1 (back edge)
        // Loop = {B1, B2, B3}
        let func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(2) },
            },
            IrBlock {
                id: BlockId(2),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(3) },
            },
            IrBlock {
                id: BlockId(3),
                instructions: vec![],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
        ]);

        let preds = build_predecessors(&func);
        let dom = compute_dominators(&func, &preds);
        let back_edges = find_back_edges(&func, &dom);
        let loops = find_natural_loops(&back_edges, &preds);

        assert_eq!(loops.len(), 1);
        let (header, loop_blocks) = &loops[0];
        assert_eq!(*header, BlockId(1));
        assert_eq!(
            *loop_blocks,
            HashSet::from([BlockId(1), BlockId(2), BlockId(3)])
        );
    }
}
