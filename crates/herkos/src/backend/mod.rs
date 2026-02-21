//! Code generation backends.
//!
//! The Backend trait abstracts the difference between safe, verified, and hybrid
//! code generation. Each backend emits different Rust code for the same IR.

mod safe;
pub use safe::SafeBackend;

use crate::ir::*;
use anyhow::Result;

/// Code generation backend trait.
///
/// Different backends emit different Rust code from the same IR:
/// - SafeBackend: bounds-checked, returns Result
/// - VerifiedBackend: unsafe + proof comments (Milestone 6)
/// - HybridBackend: mix of safe and unsafe (Milestone 6)
pub trait Backend {
    /// Emit Rust code for a constant value.
    fn emit_const(&self, dest: VarId, value: &IrValue) -> String;

    /// Emit Rust code for a binary operation.
    fn emit_binop(&self, dest: VarId, op: BinOp, lhs: VarId, rhs: VarId) -> String;

    /// Emit Rust code for a unary operation.
    fn emit_unop(&self, dest: VarId, op: UnOp, operand: VarId) -> String;

    /// Emit Rust code for a memory load (full or sub-width).
    fn emit_load(
        &self,
        dest: VarId,
        ty: WasmType,
        addr: VarId,
        offset: u32,
        width: MemoryAccessWidth,
        sign: Option<SignExtension>,
    ) -> Result<String>;

    /// Emit Rust code for a memory store (full or sub-width).
    fn emit_store(
        &self,
        ty: WasmType,
        addr: VarId,
        value: VarId,
        offset: u32,
        width: MemoryAccessWidth,
    ) -> Result<String>;

    /// Emit Rust code for a function call (local function).
    fn emit_call(
        &self,
        dest: Option<VarId>,
        func_idx: usize,
        args: &[VarId],
        has_globals: bool,
        has_memory: bool,
        has_table: bool,
    ) -> String;

    /// Emit Rust code for an imported function call.
    /// Generates `host.func_name(args)?`
    fn emit_call_import(
        &self,
        dest: Option<VarId>,
        module_name: &str,
        func_name: &str,
        args: &[VarId],
    ) -> String;

    /// Emit Rust code for reading a global variable.
    /// Mutable globals: `globals.g{index}`, immutable: `G{index}` (const item).
    fn emit_global_get(&self, dest: VarId, index: usize, is_mutable: bool) -> String;

    /// Emit Rust code for writing a mutable global variable.
    fn emit_global_set(&self, index: usize, value: VarId) -> String;

    /// Emit Rust code for an assignment.
    fn emit_assign(&self, dest: VarId, src: VarId) -> String;

    /// Emit Rust code for select (conditional move).
    fn emit_select(&self, dest: VarId, val1: VarId, val2: VarId, condition: VarId) -> String;

    /// Emit Rust code for a return statement.
    fn emit_return(&self, value: Option<VarId>) -> String;

    /// Emit Rust code for memory.size (returns current page count as i32).
    fn emit_memory_size(&self, dest: VarId) -> String;

    /// Emit Rust code for memory.grow (grows by delta pages, returns old size or -1).
    fn emit_memory_grow(&self, dest: VarId, delta: VarId) -> String;

    /// Emit Rust code for unreachable.
    fn emit_unreachable(&self) -> String;

    /// Emit Rust code for an unconditional jump using block index.
    fn emit_jump_to_index(&self, target_idx: usize) -> String;

    /// Emit Rust code for a conditional branch using block indices.
    fn emit_branch_if_to_index(
        &self,
        condition: VarId,
        if_true_idx: usize,
        if_false_idx: usize,
    ) -> String;

    /// Emit Rust code for multi-way branch (br_table) using block indices.
    fn emit_branch_table_to_index(
        &self,
        index: VarId,
        target_indices: &[usize],
        default_idx: usize,
    ) -> String;
}
