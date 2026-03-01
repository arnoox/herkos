//! IR optimization passes.
//!
//! This module implements optimizations on the intermediate representation (IR)
//! to improve code generation quality and runtime performance.
//!
//! Each optimization is a self-contained sub-module. The top-level
//! [`optimize_ir`] function runs all passes in order.

use crate::ir::ModuleInfo;
use anyhow::Result;

// ── Shared utilities ─────────────────────────────────────────────────────────
pub(crate) mod utils;

// ── Passes ───────────────────────────────────────────────────────────────────
mod const_prop;
mod copy_prop;
mod dead_blocks;
mod dead_instrs;
mod empty_blocks;
mod local_cse;
mod merge_blocks;

/// Optimizes the IR representation by running all passes in order.
pub fn optimize_ir(module_info: ModuleInfo) -> Result<ModuleInfo> {
    let mut module_info = module_info;
    for func in &mut module_info.ir_functions {
        empty_blocks::eliminate(func);
        merge_blocks::eliminate(func);
        dead_blocks::eliminate(func)?;
        const_prop::eliminate(func);
        copy_prop::eliminate(func);
        local_cse::eliminate(func);
        dead_instrs::eliminate(func);
    }
    Ok(module_info)
}

// ── optimize_ir integration tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // TODO: Add tests that verify the correctness of the optimized IR and the generated code.
}
