//! IR optimization passes.
//!
//! This module implements optimizations on the intermediate representation (IR)
//! to improve code generation quality and runtime performance.
//!
//! Each optimization is a self-contained sub-module. The top-level
//! [`optimize_ir`] function runs all passes in order.

use crate::ir::LoweredModuleInfo;
use anyhow::Result;

// ── Shared utilities ─────────────────────────────────────────────────────────
pub(crate) mod utils;

// ── Passes ───────────────────────────────────────────────────────────────────
mod algebraic;
mod branch_fold;
mod const_prop;
mod copy_prop;
mod dead_blocks;
mod dead_instrs;
mod empty_blocks;
mod licm;
mod local_cse;
mod merge_blocks;

/// Optimizes the IR representation by running all passes in order.
///
/// Expects a [`LoweredModuleInfo`] — i.e. phi nodes have already been lowered
/// by [`crate::ir::lower_phis::lower`] before calling this function.
pub fn optimize_ir(module_info: LoweredModuleInfo) -> Result<LoweredModuleInfo> {
    let mut module_info = module_info;
    for func in &mut module_info.ir_functions {
        // Two structural passes: dead_instrs may have emptied blocks that
        // lower_phis had populated with now-dead assignments (e.g. loop-exit
        // locals that were never used after the join block). Re-run structural
        // cleanup to remove those passthrough blocks.
        for _ in 0..2 {
            // Empty block optimizations
            empty_blocks::eliminate(func);
            dead_blocks::eliminate(func)?;

            // Control flow optimizations
            merge_blocks::eliminate(func);
            dead_blocks::eliminate(func)?;

            // Value optimizations
            const_prop::eliminate(func);
            algebraic::eliminate(func);
            copy_prop::eliminate(func);

            // Redundancy elimination
            local_cse::eliminate(func);
            copy_prop::eliminate(func);
            dead_instrs::eliminate(func);

            // Branch simplification
            branch_fold::eliminate(func);
            dead_instrs::eliminate(func);

            // Loop optimization
            licm::eliminate(func);
            dead_instrs::eliminate(func);
            copy_prop::eliminate(func);
        }
    }
    Ok(module_info)
}

// ── optimize_ir integration tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // TODO: Add tests that verify the correctness of the optimized IR and the generated code.
}
