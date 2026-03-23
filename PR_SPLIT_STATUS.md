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
| F | `pr-f/redundancy-loop-passes` | Redundancy + loop passes | ⚠️ WIP | #31 | ❌ Needs fix | Pattern match errors in utils |
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

### PR F #31: Redundancy Elimination + Loop Passes ⚠️ WIP

**Status**: WIP — files copied, needs test fixes

**Files** (from PR #12, needs adjustment):
- `crates/herkos-core/src/optimizer/local_cse.rs` (+575, new)
  - Local common subexpression elimination within blocks
- `crates/herkos-core/src/optimizer/gvn.rs` (+619, new)
  - Global value numbering across blocks using dominator tree
- `crates/herkos-core/src/optimizer/licm.rs` (+1,307, new)
  - Loop invariant code motion
- `crates/herkos-core/src/optimizer/branch_fold.rs` (+366, new)
  - Branch condition simplification
- `crates/herkos-core/src/optimizer/mod.rs` (updated)
  - Registers all 4 passes in `optimize_lowered_ir` pipeline

**Tests**: ❌ Compile errors — pattern matching incomplete

**Known Issues**:
1. Missing pattern match arms for `MemoryFill`, `MemoryInit`, `DataDrop` in several functions
   - Affects: branch_fold, gvn, licm, local_cse tests
   - Fix: Add cases to all `match instr` blocks that pattern match IR instructions
2. Tests reference removed `IrFunction.needs_host` field
   - Already fixed for PRs D & E, needs same fix in PR F files

**Passes Registered** (post-lowering, in iteration loop):
- `local_cse` → `gvn` → `branch_fold` → `licm` (with dead_instrs between)

**Dependencies**:
- ✅ Requires PR D (optimizer infrastructure + utils)
- ✅ Requires PR E (value optimizations)

**TODO Before Merge**:
- [ ] Fix missing IR instruction pattern matches (MemoryFill, MemoryInit, DataDrop)
- [ ] Remove remaining `needs_host` references in test code
- [ ] Run full test suite: `cargo test -p herkos-core --lib`
- [ ] Verify dominator tree construction in GVN
- [ ] Check LICM loop detection correctness
- [ ] Test with loop-heavy Wasm (e.g., array operations)

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
  ↓ merge PR F (after fixes)
  ├─ main + A + D + E + F
  ↓ merge PR G
  └─ main + A + D + E + F + G (complete)
```

### Rationale
- A is independent (pure runtime addition)
- D is infrastructure foundation
- E depends on A (float ops) + D (utilities)
- F depends on E (builds on value passes)
- G depends on A (must have runtime ops to codegen)

---

## Current Blockers

### PR F Compilation Errors

**Error Pattern**:
```
error[E0004]: non-exhaustive patterns: `MemoryFill`, `MemoryInit`, `DataDrop`
```

**Root Cause**: PR #12 was created before `MemoryFill`, `MemoryInit`, `DataDrop` were added to `IrInstr` enum.

**Solution**: Add pattern match arms to all functions in optimizer files:

```rust
// Example fix for for_each_use in utils.rs
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

**Affected Files in PR F**:
- branch_fold.rs (test code)
- gvn.rs (test code)
- licm.rs (test code)
- local_cse.rs (test code)

**Action Item**: Apply pattern match fixes and re-run tests before final review.

---

## Testing Checklist

### Unit Tests (Current)
- [x] PR A: `cargo test -p herkos-runtime` ✅ 121 pass
- [x] PR D: `cargo test -p herkos-core --lib` ✅ 117 pass
- [x] PR E: `cargo test -p herkos-core --lib` ✅ 201 pass
- [ ] PR F: `cargo test -p herkos-core --lib` ❌ Needs fixes
- [ ] PR G: `cargo test` (all crates) ⏳ Not started

### Integration Tests (After Merge)
- [ ] `cargo test -p herkos-tests` — E2E Wasm → Rust
- [ ] `cargo bench -p herkos-tests` — Fibonacci benchmarks

### Code Quality (All PRs)
- [x] PR A: `cargo clippy` ✅ clean
- [x] PR D: `cargo clippy` ✅ clean
- [x] PR E: `cargo clippy` ✅ clean
- [ ] PR F: `cargo clippy` ⏳ Not checked (blocked on compile)
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
│   ├── local_cse.rs                [post-lowering, PR F]
│   ├── gvn.rs                      [post-lowering, PR F]
│   ├── licm.rs                     [post-lowering, PR F]
│   ├── branch_fold.rs              [post-lowering, PR F]
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

### Immediate (Complete PR F)
1. [ ] Fix pattern matches for MemoryFill/MemoryInit/DataDrop in PR F
2. [ ] Run `cargo test -p herkos-core --lib` and confirm all pass
3. [ ] Run `cargo clippy` and `cargo fmt --check`
4. [ ] Push fixes to `pr-f/redundancy-loop-passes`

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

*Last updated: 2026-03-23*
*Status: 4 of 5 PRs complete, PR F WIP, PR G planned*
