# PR #12 Split Status — Detailed Plan & TODOs

## Overview

This document tracks the progress of splitting PR #12 ("feat: introduce new ir optimizations") into 5 smaller, reviewable PRs. Each PR adds a distinct layer of the optimization pipeline.

**Original PR #12**: 9,688 additions / 341 deletions across 31 files
**Split Target**: 5 focused PRs, each independently reviewable but stacked for merging

---

## PR Status Summary

| # | Branch | Title | Status | GitHub PR | Tests | Notes |
|---|--------|-------|--------|-----------|-------|-------|
| A | `pr-a/wasm-float-ops` | Runtime float ops | ✅ Complete | #28 | ✅ Pass | Ready for review |
| D | `pr-d/optimizer-infrastructure` | Optimizer infrastructure | ✅ Complete | #29 | ✅ Pass | Ready for review |
| E | `pr-e/value-optimizations` | Value optimization passes | ✅ Complete | #30 | ✅ Pass | Ready for review |
| F1 | `pr-f1/branch-fold` | Branch condition folding | ⚠️ WIP | #31 | ❌ Needs fix | Split from old PR F |
| F2 | `pr-f2/local-cse` | Local common subexpression elimination | ❌ Not started | — | — | Split from old PR F |
| F3 | `pr-f3/gvn` | Global value numbering | ❌ Not started | — | — | Split from old PR F |
| F4 | `pr-f4/licm` | Loop invariant code motion | ❌ Not started | — | — | Split from old PR F, last |
| G | `pr-g/backend-codegen` | Backend + codegen updates | ❌ Not started | — | — | Planned |

---

## Detailed PR Breakdown

### PR A #28: Runtime Float Operations ✅

**Status**: Complete, ready for review

**Files**:
- `crates/herkos-runtime/src/ops.rs` (+141 lines)
  - `wasm_min_f32`, `wasm_max_f32` — NaN propagation, ±0.0 handling
  - `wasm_min_f64`, `wasm_max_f64` — Wasm spec compliance
  - `wasm_nearest_f32`, `wasm_nearest_f64` — Banker's rounding without libm
- `crates/herkos-runtime/src/lib.rs` (+2): Re-exports
- `.gitignore` (+1): Add `**/*.wasm.rs`

**Tests**: ✅ All 121 tests pass, clippy clean

**Key Points**:
- No `std` required, no heap allocation
- Used by `const_prop` pass for compile-time float evaluation
- Zero runtime dependencies

**Review Checklist**:
- [ ] Verify Wasm spec compliance for min/max/nearest
- [ ] Check NaN and signed-zero handling
- [ ] Confirm no performance regression

---

### PR D #29: Optimizer Infrastructure ✅

**Status**: Complete, ready for review

**Files**:
- `crates/herkos-core/src/optimizer/utils.rs` (+644, new)
  - Shared utilities: `terminator_successors`, `build_predecessors`
  - Variable traversal: `for_each_use`, `for_each_use_terminator`, `count_uses_of`
  - Instruction classification: `instr_dest`, `set_instr_dest`, `is_side_effect_free`
  - Variable substitution: `replace_uses_of`, `rewrite_terminator_target`
- `crates/herkos-core/src/optimizer/dead_instrs.rs` (+363, new)
  - Dead instruction elimination (post-lowering pass)
- `crates/herkos-core/src/optimizer/empty_blocks.rs` (+333, new)
  - Passthrough block elimination
- `crates/herkos-core/src/optimizer/merge_blocks.rs` (+384, new)
  - Single-predecessor block merging
- `crates/herkos-core/src/optimizer/dead_blocks.rs` (refactored)
  - Now uses shared `terminator_successors` from utils
- `crates/herkos-core/src/optimizer/mod.rs` (updated)
  - Adds `optimize_lowered_ir` function
  - Registers post-lowering structural passes

**Tests**: ✅ All 117 tests pass, clippy clean

**Passes Registered** (post-lowering only):
- `empty_blocks` → `dead_blocks` → `merge_blocks` → `dead_blocks` → `dead_instrs` (2 iterations)

**Key Points**:
- Foundation for all subsequent optimizer PRs
- Handles new IR instruction types: `MemoryFill`, `MemoryInit`, `DataDrop`
- Runs structural cleanup in iterations until fixed point

**Review Checklist**:
- [ ] Verify utility functions are correct (especially `build_predecessors`)
- [ ] Check loop termination (2 iterations sufficient?)
- [ ] Confirm all new IR instructions handled

---

### PR E #30: Value Optimization Passes ✅

**Status**: Complete, ready for review

**Files**:
- `crates/herkos-core/src/optimizer/const_prop.rs` (+1,309, new)
  - Constant folding and propagation
  - Uses `herkos_runtime` functions for Wasm-spec-compliant evaluation
  - Tracks dataflow of constant values
- `crates/herkos-core/src/optimizer/algebraic.rs` (+782, new)
  - Algebraic simplifications (e.g., `x * 1 → x`, `x & x → x`)
  - Runs after `const_prop` to operate on known constants
- `crates/herkos-core/src/optimizer/copy_prop.rs` (+1,347, new)
  - Backward coalescing and forward substitution
  - Registered pre-lowering and post-lowering
- `crates/herkos-core/src/ir/types.rs` (updated)
  - Added `PartialEq` to `IrValue` (needed for test assertions)
- `crates/herkos-core/src/optimizer/mod.rs` (updated)
  - Registers pre-lowering passes in `optimize_ir`
  - Adds `copy_prop` to post-lowering pipeline
- `crates/herkos-core/Cargo.toml` (updated)
  - Added `herkos-runtime` dependency (needed for float op evaluation)

**Tests**: ✅ All 201 tests pass, clippy clean

**Passes Registered**:
- **Pre-lowering**: `dead_blocks` → `const_prop` → `algebraic` → `copy_prop`
- **Post-lowering**: `copy_prop` + structural passes in iterations

**Dependencies**:
- ✅ Requires PR A (float ops for const evaluation)
- ✅ Requires PR D (optimizer infrastructure)

**Key Points**:
- Const prop evaluates Wasm arithmetic using runtime functions
- Pre-lowering const prop simplifies phi nodes before SSA destruction
- Post-lowering copy_prop forwards lower_phis assignments
- IrValue::PartialEq enables test comparisons

**Review Checklist**:
- [ ] Verify const folding correctness (esp. float ops, overflow handling)
- [ ] Check algebraic simplification rules are sound
- [ ] Confirm copy propagation doesn't break SSA invariants
- [ ] Test with real Wasm modules (fibonacci, etc.)

---

### PR F1 #31: Branch Condition Folding ⚠️ WIP

**Status**: WIP — file exists on current branch, needs pattern match fixes

**Branch**: `pr-f1/branch-fold` (rename/split from `pr-f/redundancy-loop-passes`)

**Files**:
- `crates/herkos-core/src/optimizer/branch_fold.rs` (+366, new)
  - Simplifies branch conditions: constant branches, always-taken jumps
- `crates/herkos-core/src/optimizer/mod.rs` (updated)
  - Registers `branch_fold::eliminate` in `optimize_lowered_ir` pipeline

**Tests**: ❌ Compile errors — missing pattern match arms

**Known Issues**:
1. Missing pattern match arms for `MemoryFill`, `MemoryInit`, `DataDrop` in test helpers
   - Fix: Add exhaustive arms to all `match instr` blocks in branch_fold tests
2. Tests may reference removed `IrFunction.needs_host` field — verify and remove

**Passes Registered** (post-lowering, added to iteration loop):
- `branch_fold::eliminate` (after `dead_instrs`)

**Dependencies**:
- ✅ Requires PR D (optimizer infrastructure + utils)
- ✅ Requires PR E (value optimizations)

**TODO Before Merge**:
- [ ] Create branch `pr-f1/branch-fold` from PR E base
- [ ] Cherry-pick only `branch_fold.rs` + `mod.rs` changes
- [ ] Fix missing IR instruction pattern matches in branch_fold tests
- [ ] Run `cargo test -p herkos-core --lib`
- [ ] Run `cargo clippy` and `cargo fmt --check`

---

### PR F2: Local Common Subexpression Elimination ❌ Not Started

**Status**: Not started — code exists in `pr-f/redundancy-loop-passes`, needs isolation

**Branch**: `pr-f2/local-cse` (to be created from PR F1)

**Files**:
- `crates/herkos-core/src/optimizer/local_cse.rs` (+575, new)
  - Eliminates redundant computations within a single basic block
  - Keyed on instruction structure (opcode + operands), no cross-block analysis
- `crates/herkos-core/src/optimizer/mod.rs` (updated)
  - Registers `local_cse::eliminate` in `optimize_lowered_ir` pipeline

**Tests**: ❌ Compile errors (same pattern match issues as F1)

**Known Issues**:
- Same missing `MemoryFill`, `MemoryInit`, `DataDrop` pattern arms as F1
- Fix: Add exhaustive arms to all `match instr` blocks in local_cse tests

**Passes Registered** (post-lowering, in iteration loop):
- `local_cse::eliminate` (after structural passes, before GVN)

**Dependencies**:
- ✅ Requires PR D (optimizer infrastructure + utils)
- ✅ Requires PR E (value optimizations)
- ✅ Requires PR F1 (branch fold reduces branches before CSE runs)

**TODO Before Merge**:
- [ ] Create branch `pr-f2/local-cse` from PR F1 base
- [ ] Cherry-pick only `local_cse.rs` + `mod.rs` changes
- [ ] Fix missing IR instruction pattern matches in local_cse tests
- [ ] Run `cargo test -p herkos-core --lib`
- [ ] Run `cargo clippy` and `cargo fmt --check`

---

### PR F3: Global Value Numbering ❌ Not Started

**Status**: Not started — code exists in `pr-f/redundancy-loop-passes`, needs isolation

**Branch**: `pr-f3/gvn` (to be created from PR F2)

**Files**:
- `crates/herkos-core/src/optimizer/gvn.rs` (+619, new)
  - Value numbering across basic blocks using dominator tree traversal
  - Eliminates redundant computations that `local_cse` cannot catch across blocks
- `crates/herkos-core/src/optimizer/mod.rs` (updated)
  - Registers `gvn::eliminate` in `optimize_lowered_ir` pipeline

**Tests**: ❌ Compile errors (same pattern match issues as F1/F2)

**Known Issues**:
- Same missing `MemoryFill`, `MemoryInit`, `DataDrop` pattern arms
- Dominator tree construction correctness should be carefully verified

**Passes Registered** (post-lowering, in iteration loop):
- `gvn::eliminate` (after `local_cse`, before `dead_instrs`)

**Dependencies**:
- ✅ Requires PR D (optimizer infrastructure + utils, including `build_predecessors`)
- ✅ Requires PR E (value optimizations)
- ✅ Requires PR F2 (local CSE runs first, GVN handles cross-block residuals)

**TODO Before Merge**:
- [ ] Create branch `pr-f3/gvn` from PR F2 base
- [ ] Cherry-pick only `gvn.rs` + `mod.rs` changes
- [ ] Fix missing IR instruction pattern matches in gvn tests
- [ ] Verify dominator tree construction correctness
- [ ] Run `cargo test -p herkos-core --lib`
- [ ] Run `cargo clippy` and `cargo fmt --check`

---

### PR F4: Loop Invariant Code Motion ❌ Not Started

**Status**: Not started — code exists in `pr-f/redundancy-loop-passes`, needs isolation

**Branch**: `pr-f4/licm` (to be created from PR F3, **last of the F series**)

**Files**:
- `crates/herkos-core/src/optimizer/licm.rs` (+1,307, new)
  - Detects natural loops via back-edges and dominator tree
  - Hoists side-effect-free loop-invariant instructions to loop preheaders
- `crates/herkos-core/src/optimizer/mod.rs` (updated)
  - Uncomments and registers `licm::eliminate` in `optimize_lowered_ir` pipeline
  - Currently commented out: `// licm::eliminate(func);`

**Tests**: ❌ Compile errors (same pattern match issues + licm is currently commented out)

**Known Issues**:
- Same missing `MemoryFill`, `MemoryInit`, `DataDrop` pattern arms
- Loop detection may be conservative — verify back-edge identification
- LICM must not hoist instructions with side effects (memory writes, traps)

**Passes Registered** (post-lowering, end of iteration loop):
- `licm::eliminate` (after GVN and `dead_instrs`, final pass in loop body)

**Dependencies**:
- ✅ Requires PR D (optimizer infrastructure + utils, `is_side_effect_free`)
- ✅ Requires PR E (value optimizations)
- ✅ Requires PR F1 (branch fold first simplifies loop exit conditions)
- ✅ Requires PR F2 (local CSE)
- ✅ Requires PR F3 (GVN — hoisting is most effective after redundancies removed)

**TODO Before Merge**:
- [ ] Create branch `pr-f4/licm` from PR F3 base
- [ ] Cherry-pick only `licm.rs` + `mod.rs` changes (uncomment licm call)
- [ ] Fix missing IR instruction pattern matches in licm tests
- [ ] Verify loop detection and preheader insertion correctness
- [ ] Check `is_side_effect_free` covers all hoistable instructions
- [ ] Test with loop-heavy Wasm (e.g., array operations)
- [ ] Run `cargo test -p herkos-core --lib`
- [ ] Run `cargo clippy` and `cargo fmt --check`

---

### PR G: Backend + Codegen Updates ❌ NOT STARTED

**Planned Files**:
- `crates/herkos/Cargo.toml` (+1)
  - Add `herkos-runtime` dependency to transpiler crate
- `crates/herkos/src/backend/mod.rs` (+13)
  - Add trait methods for float operations
- `crates/herkos/src/backend/safe.rs` (+54)
  - Implement `wasm_min_f32`, `wasm_max_f32`, etc. in `SafeBackend`
- `crates/herkos/src/codegen/function.rs` (+60)
  - Update function code generation
- `crates/herkos/src/codegen/instruction.rs` (+25, -1)
  - Add float instruction code generation
- `crates/herkos/src/codegen/mod.rs` (+7, -4)
- `crates/herkos/src/codegen/module.rs` (+4, -1)
- `crates/herkos/tests/e2e.rs` (-3)
  - Test cleanup
- `Cargo.lock`: Auto-updated

**Purpose**:
- Wire runtime float ops into codegen path
- Ensure transpiled Rust code calls the correct runtime functions
- End-to-end integration: Wasm → IR → optimized IR → Rust code

**Dependencies**:
- Requires PR A (runtime float ops must exist)
- Requires PR D, E, F (optimizers must work)
- No direct code dependency, but all PRs should be merged first

**TODO**:
- [ ] Extract codegen changes from PR #12
- [ ] Map Wasm float instructions to backend method calls
- [ ] Update `SafeBackend` implementations
- [ ] Run E2E tests: `cargo test -p herkos-tests`
- [ ] Verify generated Rust code compiles
- [ ] Check performance with benchmarks: `cargo bench -p herkos-tests`

---

## Merge Order & Strategy

### Strict Linear Order (Recommended)
```
main
  ↓ merge PR A
  ├─ main + A
  ↓ merge PR D
  ├─ main + A + D
  ↓ merge PR E
  ├─ main + A + D + E
  ↓ merge PR F1 (branch fold)
  ├─ main + A + D + E + F1
  ↓ merge PR F2 (local CSE)
  ├─ main + A + D + E + F1 + F2
  ↓ merge PR F3 (GVN)
  ├─ main + A + D + E + F1 + F2 + F3
  ↓ merge PR F4 (LICM — last)
  ├─ main + A + D + E + F1 + F2 + F3 + F4
  ↓ merge PR G
  └─ main + A + D + E + F1–F4 + G (complete)
```

### Rationale
- A is independent (pure runtime addition)
- D is infrastructure foundation
- E depends on A (float ops) + D (utilities)
- F1 depends on E — branch fold simplifies control flow for later passes
- F2 depends on F1 — local CSE runs after branches are simplified
- F3 depends on F2 — GVN extends CSE globally across blocks
- F4 depends on F3 — LICM is most effective after redundancies are removed (last pass)
- G depends on A (must have runtime ops to codegen)

---

## Current Blockers

### PR F1–F4 Compilation Errors

**Error Pattern**:
```
error[E0004]: non-exhaustive patterns: `MemoryFill`, `MemoryInit`, `DataDrop`
```

**Root Cause**: PR #12 was created before `MemoryFill`, `MemoryInit`, `DataDrop` were added to `IrInstr` enum.

**Solution**: Add pattern match arms to all functions in optimizer files:

```rust
// Example fix pattern
IrInstr::MemoryFill { dst, val, len } => {
    f(*dst);
    f(*val);
    f(*len);
}
IrInstr::MemoryInit { dst, src_offset, len, .. } => {
    f(*dst);
    f(*src_offset);
    f(*len);
}
IrInstr::DataDrop { .. } => {}
```

**Affected files (fix needed in each isolated PR)**:
- `branch_fold.rs` test code → fix in PR F1
- `local_cse.rs` test code → fix in PR F2
- `gvn.rs` test code → fix in PR F3
- `licm.rs` test code → fix in PR F4

**Action Item**: Apply pattern match fixes per file when working on each isolated PR branch.

---

## Testing Checklist

### Unit Tests (Current)
- [x] PR A: `cargo test -p herkos-runtime` ✅ 121 pass
- [x] PR D: `cargo test -p herkos-core --lib` ✅ 117 pass
- [x] PR E: `cargo test -p herkos-core --lib` ✅ 201 pass
- [ ] PR F1: `cargo test -p herkos-core --lib` ❌ Needs fixes
- [ ] PR F2: `cargo test -p herkos-core --lib` ❌ Needs fixes
- [ ] PR F3: `cargo test -p herkos-core --lib` ❌ Needs fixes
- [ ] PR F4: `cargo test -p herkos-core --lib` ❌ Needs fixes
- [ ] PR G: `cargo test` (all crates) ⏳ Not started

### Integration Tests (After Merge)
- [ ] `cargo test -p herkos-tests` — E2E Wasm → Rust
- [ ] `cargo bench -p herkos-tests` — Fibonacci benchmarks

### Code Quality (All PRs)
- [x] PR A: `cargo clippy` ✅ clean
- [x] PR D: `cargo clippy` ✅ clean
- [x] PR E: `cargo clippy` ✅ clean
- [ ] PR F1: `cargo clippy` ⏳ Blocked on compile
- [ ] PR F2: `cargo clippy` ⏳ Not started
- [ ] PR F3: `cargo clippy` ⏳ Not started
- [ ] PR F4: `cargo clippy` ⏳ Not started
- [ ] PR G: `cargo clippy` ⏳ Not started

### Format Check (All PRs)
- [ ] `cargo fmt --check` on all PRs

---

## Known Limitations & Future Work

### Not Included (Future Enhancements)
- **Verified backend** — Currently only safe backend implemented
- **Hybrid backend** — Mix of safe and verified code
- **Temporal isolation** — Future feature
- **Contract-based verification** — Future feature
- **`--max-pages` CLI effect** — Not yet wired through transpiler
- **WASI traits** — Standard import traits not yet implemented

### Performance Notes
- Two-pass structural cleanup may iterate more than necessary
- GVN dominator tree construction could be optimized
- LICM may be conservative in loop detection

### Documentation
- Consider adding optimizer pass pipeline diagram to SPECIFICATION.md
- Document dataflow analysis assumptions (SSA form requirements)

---

## Quick Reference: File Locations

```
Optimizer Passes:
├── crates/herkos-core/src/optimizer/
│   ├── utils.rs                    [shared utilities, PR D]
│   ├── dead_blocks.rs              [refactored, PR D]
│   ├── dead_instrs.rs              [post-lowering, PR D]
│   ├── empty_blocks.rs             [post-lowering, PR D]
│   ├── merge_blocks.rs             [post-lowering, PR D]
│   ├── const_prop.rs               [pre-lowering, PR E]
│   ├── algebraic.rs                [pre-lowering, PR E]
│   ├── copy_prop.rs                [pre + post, PR E]
│   ├── branch_fold.rs              [post-lowering, PR F1]
│   ├── local_cse.rs                [post-lowering, PR F2]
│   ├── gvn.rs                      [post-lowering, PR F3]
│   ├── licm.rs                     [post-lowering, PR F4]
│   └── mod.rs                      [pipeline coordination]

Runtime:
├── crates/herkos-runtime/src/
│   ├── ops.rs                      [wasm_min/max/nearest, PR A]
│   └── lib.rs                      [re-exports, PR A]

Codegen (PR G):
├── crates/herkos/src/
│   ├── backend/mod.rs              [trait updates]
│   ├── backend/safe.rs             [float op implementations]
│   └── codegen/                    [instruction generation]
```

---

## Next Steps

### Immediate (Create PR F1 — branch fold)
1. [ ] Create branch `pr-f1/branch-fold` from PR E tip
2. [ ] Cherry-pick only `branch_fold.rs` + `mod.rs` (branch_fold registration) from old `pr-f/redundancy-loop-passes`
3. [ ] Fix missing `MemoryFill`/`MemoryInit`/`DataDrop` pattern arms in `branch_fold.rs` tests
4. [ ] Run `cargo test -p herkos-core --lib` — confirm all pass
5. [ ] Run `cargo clippy` and `cargo fmt --check`
6. [ ] Push and open PR against PR E branch (or main if E is merged)

### Next (PR F2 — local CSE)
1. [ ] Create branch `pr-f2/local-cse` from PR F1 tip
2. [ ] Cherry-pick only `local_cse.rs` + `mod.rs` changes
3. [ ] Fix pattern match arms in `local_cse.rs` tests
4. [ ] Run tests, clippy, fmt
5. [ ] Open PR

### Then (PR F3 — GVN)
1. [ ] Create branch `pr-f3/gvn` from PR F2 tip
2. [ ] Cherry-pick only `gvn.rs` + `mod.rs` changes
3. [ ] Fix pattern match arms in `gvn.rs` tests
4. [ ] Verify dominator tree construction
5. [ ] Run tests, clippy, fmt
6. [ ] Open PR

### Then (PR F4 — LICM, last)
1. [ ] Create branch `pr-f4/licm` from PR F3 tip
2. [ ] Cherry-pick only `licm.rs` + `mod.rs` changes (uncomment `licm::eliminate`)
3. [ ] Fix pattern match arms in `licm.rs` tests
4. [ ] Verify loop detection and preheader insertion
5. [ ] Run tests, clippy, fmt
6. [ ] Open PR

### Short Term (Prepare PR G)
1. [ ] Extract codegen changes from PR #12
2. [ ] Create `pr-g/backend-codegen` branch
3. [ ] Implement and test
4. [ ] Verify all E2E tests pass

### Post-Merge (Validation)
1. [ ] Run full integration suite: `cargo test`
2. [ ] Run benchmarks: `cargo bench`
3. [ ] Profile optimization impact (e.g., % code size reduction)
4. [ ] Document lessons learned in SPECIFICATION.md

---

## Contact & References

- **Original PR**: GitHub #12
- **Plan file**: `/home/vscode/.claude/plans/streamed-hatching-castle.md`
- **Branch tracking**: See PRs #28–#31 for current status
- **CLAUDE.md**: Project conventions and architecture

---

*Last updated: 2026-03-27*
*Status: 3 of 8 PRs complete (A, D, E); PR F split into F1–F4 (one pass each, LICM last); PR G planned*
