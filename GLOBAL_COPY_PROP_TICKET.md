# Ticket: Re-Introduce Global Copy Propagation with Safe Data-Flow Analysis

**Status:** TODO
**Priority:** Medium (performance optimization)
**Component:** Optimizer (copy_prop.rs)
**Related:** Fixed by commit 5f40d55 (removed buggy implementation)

## Problem Statement

Global copy propagation was removed due to a correctness bug that manifested in loop-based Wasm code. The optimization is valuable for reducing variable allocations and improving code quality, but the original implementation lacked proper data-flow safety analysis.

### Historical Context

- Original implementation was disabled for multi-block functions to prevent incorrect variable substitution
- Bug: Variables captured via `Assign` (e.g., `v20 = Assign(v10)`) followed by redefinition of the source (e.g., `v10 = result`) would cause later uses of the captured variable to reference the NEW value instead of the captured OLD value
- Test case: `test_fib_i64` returned 512 instead of 55 due to this bug
- Decision: Remove the pass entirely rather than attempt to patch the complex logic

## Objective

Re-introduce the global copy propagation optimization with a **correct and efficient implementation** that properly handles:
1. Single-block functions (safe case - no loops)
2. Multi-block functions with loops (requires data-flow analysis)
3. Variable dependency chains across assignments
4. Proper use-def chains in SSA form

## Detailed Requirements

### R1: Single-Block Functions (Phase 1 - LOW RISK)

Restore global copy propagation for single-block functions where there are no control-flow edges between the Assign and its uses.

**Algorithm:**
1. Collect all `Assign { dest, src }` pairs where `dest` has exactly one definition
2. Verify `src` is not redefined anywhere in the block
3. Chase transitive chains to find the root variable
4. Replace all uses of `dest` with the root
5. Remove now-dead Assign instructions

**Test Coverage Required:**
- `const_assign_coalesced`
- `binop_assign_coalesced`
- `chain_coalesced`
- `fibo_b0_pattern`
- `dead_local_pruned_after_coalesce`
- `multi_use_src_global_prop_removes_both`

### R2: Multi-Block Functions (Phase 2 - REQUIRES DATA-FLOW ANALYSIS)

Implement proper data-flow analysis for multi-block functions (loops) to safely perform global copy propagation.

**Safety Constraint:**
For each `Assign { dest, src }` at position P in block B, only substitute `dest` with its root if:
- Every variable in the dependency chain (dest → ... → root) is NOT redefined in B after position P
- If the variable is redefined in B after P, there exists no use of `dest` after that redefinition within the same block

**Recommended Approach:**
1. Build a **use-def chain** for each variable
2. For each Assign candidate, compute the set of variables in its dependency chain
3. Check each variable in the chain:
   - Find all definitions of that variable across all blocks
   - If any definition occurs after the Assign in the same block, mark as unsafe
   - If the variable is a loop-carried variable (appears in phi nodes), require additional analysis
4. Only perform substitution if all safety checks pass

**Test Coverage Required:**
- `global_copy_prop_chain_across_blocks`
- `global_copy_prop_multiple_uses_across_blocks`
- `forward_cross_block_use_propagated`
- `cross_block_all_copies_resolved`

### R3: Integration

- Update `copy_prop::eliminate()` to conditionally call global_copy_prop only after safety analysis
- Ensure backward coalescing and forward propagation still work independently
- Run optimization multiple iterations (current 2-iteration pattern in optimize_ir)

## Acceptance Criteria

- [ ] Phase 1: All single-block tests pass with global copy prop enabled
- [ ] Phase 2: All multi-block tests pass with proper data-flow analysis
- [ ] `HERKOS_OPTIMIZE=1 cargo test` passes all tests (1000+ tests)
- [ ] `test_fib_i64` returns correct value (55 for fib(10)) with optimization enabled
- [ ] No regressions in other optimization passes
- [ ] Code review: At least 2 approvals for data-flow analysis implementation
- [ ] Performance: Measure improvement in generated code size/quality
- [ ] Documentation: Add comments explaining the safety analysis

## Technical Notes

### Use-Def Analysis

For safe multi-block global copy propagation, implement a use-def chain that tracks:
- Definition points: `Assign { dest, src }` instructions
- Use points: reads of each variable
- Cross-block dependencies: phi nodes representing loop-carried variables
- Redefinition sites: where a variable is reassigned

### SSA Form Assumption

The implementation can assume strict SSA form:
- Each variable is defined exactly once
- Phi nodes represent values from different control-flow paths
- After phi lowering, phi nodes become Assign instructions in predecessor blocks

### Data-Flow Lattice

Consider using a simple lattice:
```
variable ∈ {Unsafe, MaybeUnsafe, Safe}
```

- `Unsafe`: Redefined after Assign in same block → don't substitute globally
- `MaybeUnsafe`: Loop-carried variable → requires additional analysis
- `Safe`: No redefinition after Assign → safe to substitute

## Implementation Strategy

**Phase 1 (Restore single-block):**
1. Un-ignore single-block tests
2. Re-enable global_copy_prop for single-block functions
3. Verify all single-block tests pass

**Phase 2 (Add multi-block support):**
1. Implement use-def chain analysis in a new helper function
2. Create a predicate function: `is_safe_to_substitute(dest, src, func) -> bool`
3. Update global_copy_prop to check safety before substitution
4. Un-ignore multi-block tests
5. Iterate on safety analysis until all tests pass

**Phase 3 (Verification):**
1. Run full test suite
2. Benchmark code generation quality
3. Document implementation approach
4. Code review

## Related Code

- `/workspaces/herkos/crates/herkos-core/src/optimizer/copy_prop.rs` - Main implementation
- `/workspaces/herkos/crates/herkos-core/src/optimizer/utils.rs` - Helper utilities (instr_dest, for_each_use, etc.)
- Commit 5f40d55: "refactor: remove global copy propagation optimization"
- Commit 7791567: "fix: disable global copy propagation for multi-block functions"

## References

- Muchnick, S.S. (1997). *Advanced Compiler Design and Implementation.* Chapter 8.4: Copy Propagation
- SSA form and use-def chains: https://en.wikipedia.org/wiki/Static_single_assignment_form
- Wasm spec on locals and SSA: https://webassembly.github.io/spec/

## Risk Assessment

**Low Risk (Phase 1):** Single-block functions are straightforward and well-tested
**Medium Risk (Phase 2):** Multi-block analysis requires careful verification to avoid data-flow errors

## Estimated Effort

- Phase 1: 2-4 hours (testing and validation)
- Phase 2: 4-8 hours (data-flow analysis design and testing)
- Phase 3: 2-4 hours (verification and benchmarking)
- **Total: 8-16 hours**
