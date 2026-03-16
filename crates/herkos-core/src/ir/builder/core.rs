//! Core IR builder state and control flow management.
//!
//! ## Overview
//!
//! `IrBuilder` translates WebAssembly bytecode to SSA-form (Static Single Assignment)
//! intermediate representation. Each Wasm value becomes a unique variable (v0, v1, ...),
//! and explicit control flow is built via a state machine over nested blocks.
//!
//! ## Architecture
//!
//! The builder maintains **three** key data structures that together implement Wasm semantics:
//!
//! ### Value Stack (`value_stack: Vec<UseVar>`)
//!
//! Replaces Wasm's implicit evaluation stack. Instead of pushing raw values, we push
//! [`UseVar`] tokens for already-defined SSA variables. Example:
//!
//! ```text
//! i32.const 5 → emit: v0 = const 5; push(v0)
//! i32.const 3 → emit: v1 = const 3; push(v1)
//! i32.add     → pop(v1), pop(v0); emit: v2 = i32_add(v0, v1); push(v2)
//! ```
//!
//! ### Control Stack (`control_stack: Vec<ControlFrame>`)
//!
//! Tracks nested blocks/loops/if structures. `ControlFrame` is an enum with one
//! variant per construct (`Block`, `Loop`, `If`, `Else`). Each variant holds only
//! the fields relevant to it, making illegal states unrepresentable. Key fields:
//! - `end_block`: join point for forward branches / block exit
//! - `Loop::start_block`: backward-branch target (re-enter the loop)
//! - `result_var`: phi convergence slot if the block has a result type
//! - `locals_at_entry`: snapshot of `local_vars` at frame push
//! - `branch_incoming`: predecessor snapshots from forward `br`/`br_if`/`br_table`
//! - `Loop::loop_phi_vars` / `phi_patches`: phi source collection for loop back-edges
//!
//! ### Local Variable Map (`local_vars: Vec<UseVar>`)
//!
//! Maps each Wasm local index to the **current** SSA variable holding its value.
//! This map is the key to SSA construction for mutable locals:
//!
//! - At function entry, each local (param or declared) gets a fresh variable via
//!   `new_pre_alloc_var()`.  The map starts as `[v0, v1, ..., vN]`.
//! - `local.get i`  → emit `vX = Assign(local_vars[i])` and push vX.  A fresh copy
//!   is emitted (rather than re-using `local_vars[i]`) so that a later `local.set`
//!   cannot retroactively change what was already pushed onto the value stack.
//! - `local.set i`  → allocate `vY`, emit `vY = Assign(popped_value)`, set
//!   `local_vars[i] = vY`.  Subsequent `local.get i` will see vY.
//! - `local.tee i`  → same as `local.set`, but the value is also left on the stack.
//!
//! When control flow merges (at `End` of block/loop/if), different predecessors may
//! hold different variables for the same local.  The builder compares predecessor
//! snapshots and inserts `IrInstr::Phi` nodes for diverging locals, then updates
//! `local_vars` to the phi dest variables.  See `translate.rs` for the full protocol.
//!
//! ## Flow
//!
//! 1. `translate_function()` initializes state: allocates VarIds for all locals (params + declared),
//!    creates entry block (BlockId(0)), pushes function-level control frame.
//!
//! 2. For each Wasm operator: `translate_operator()` (in `translate.rs`) pops arguments from
//!    `value_stack`, allocates new variables, emits IR instructions, pushes results.
//!
//! 3. Control flow instructions (`block`, `loop`, `if`, `br`, etc.) manipulate the control stack
//!    and create new basic blocks as needed.  Branch instructions (`br`, `br_if`, `br_table`)
//!    additionally snapshot `local_vars` and record it as a phi predecessor via
//!    `record_forward_branch()` or `record_loop_back_branch()`.
//!
//! 4. At each `End`: predecessor snapshots are collected, `IrInstr::Phi` nodes are inserted for
//!    locals with differing values, and `local_vars` is updated to the phi dest variables.
//!
//! 5. Return: `IrFunction` with all blocks, variables typed, control flow explicit, and phi
//!    nodes at every join point.  The `lower_phis` pass then destructs these phis into
//!    predecessor-block assignments before codegen.
//!
//! ## Invariants
//!
//! - **Entry block**: First `new_block()` call returns `BlockId(0)` (function entry).
//! - **Value stack**: Operations pop N arguments, push ≤1 result.
//! - **Control stack**: Push/pop balanced; each frame has a unique start/end pair.
//! - **Local variables**: `local_vars[i]` always holds the *current* SSA variable for Wasm
//!   local `i`.  It is updated on every `local.set`/`local.tee` and at every join point
//!   (where it may be replaced by a phi dest variable).
//!
//! ## Example 1 — simple arithmetic with a local
//!
//! ```wasm
//! (func (param $a i32) (result i32)
//!   local.get $a
//!   i32.const 1
//!   i32.add)
//! ```
//!
//! Generated IR:
//! ```text
//! block_0:
//!   v2 = Assign(v0)      ;; local.get $a  → fresh SSA copy of param
//!   v3 = Const(1)        ;; i32.const 1
//!   v4 = I32Add(v2, v3)  ;; i32.add
//!   return v4
//!
//! ;; v0 = param $a (implicitly defined at function entry)
//! ;; v1 = result convergence slot allocated by push_control (internal)
//! ```
//!
//! Note: `local.get` emits a fresh `Assign` copy (v2) rather than pushing v0 directly.
//! This ensures a later `local.set $a` cannot retroactively change values already
//! pushed on the value stack.
//!
//! ## Example 2 — local.set creates fresh SSA variables
//!
//! ```wasm
//! (func (param $n i32) (result i32)
//!   local.get $n    ;; read $n
//!   i32.const 1
//!   i32.add
//!   local.set $n    ;; overwrite $n with $n+1
//!   local.get $n)   ;; read updated $n
//! ```
//!
//! Generated IR (local_vars evolution shown inline):
//! ```text
//! block_0:
//!   ;; local_vars = [v0]   (v0 = param $n)
//!   v2 = Assign(v0)        ;; local.get $n  → copy; local_vars = [v0]  (unchanged)
//!   v3 = Const(1)
//!   v4 = I32Add(v2, v3)
//!   v5 = Assign(v4)        ;; local.set $n  → fresh var; local_vars = [v5]
//!   v6 = Assign(v5)        ;; local.get $n  → copy of v5; local_vars = [v5] (unchanged)
//!   return v6
//! ```
//!
//! ## Example 3 — if/else with phi at join
//!
//! ```wasm
//! ;; (func (param $cond i32) (param $n i32) (result i32)
//! ;;   local.get $cond
//! ;;   if
//! ;;     i32.const 10
//! ;;     local.set $n    ;; then-branch: $n ← 10
//! ;;   else
//! ;;     i32.const 20
//! ;;     local.set $n    ;; else-branch: $n ← 20
//! ;;   end
//! ;;   local.get $n)     ;; which value?
//! ```
//!
//! SSA IR after translation (simplified var names):
//! ```text
//! block_0 (entry):
//!   ;; local_vars = [v_cond, v_n]
//!   v_cv = Assign(v_cond)               ;; local.get $cond
//!   BranchIf(v_cv, then=block_1, else=block_2)
//!
//! block_1 (then):
//!   ;; At If push: locals_at_entry = [v_cond, v_n]
//!   ;; local_vars = [v_cond, v_n]   ← snapshot preserved
//!   v_ten = Const(10)
//!   v_nt  = Assign(v_ten)               ;; local.set $n → local_vars = [v_cond, v_nt]
//!   jump block_3
//!   ;; At Else: then_pred_info = (block_1, [v_cond, v_nt])
//!
//! block_2 (else):
//!   ;; local_vars restored to [v_cond, v_n]  ← locals_at_entry
//!   v_twenty = Const(20)
//!   v_ne     = Assign(v_twenty)          ;; local.set $n → local_vars = [v_cond, v_ne]
//!   jump block_3
//!
//! block_3 (join):
//!   ;; Predecessors: (block_1, [v_cond, v_nt]) and (block_2, [v_cond, v_ne])
//!   ;; $n differs → insert phi; $cond same → no phi
//!   v_phi_n = Phi [(block_1, v_nt), (block_2, v_ne)]
//!   ;; local_vars = [v_cond, v_phi_n]
//!   v_r = Assign(v_phi_n)               ;; local.get $n
//!   return v_r
//! ```
//!
//! ## Example 4 — loop with phi vars for the back-edge
//!
//! ```wasm
//! ;; (func (param $n i32) (result i32)
//! ;;   (local $i i32)     ;; zero-initialized
//! ;;   loop $L
//! ;;     local.get $i
//! ;;     i32.const 1
//! ;;     i32.add
//! ;;     local.set $i     ;; $i++
//! ;;     local.get $i
//! ;;     local.get $n
//! ;;     i32.lt_s
//! ;;     br_if $L)        ;; keep looping while $i < $n
//! ;;   local.get $i)      ;; return final $i
//! ```
//!
//! SSA IR after translation (simplified):
//! ```text
//! block_0 (pre-loop):
//!   ;; local_vars = [v_n, v_i0]  (v_n=param, v_i0=0-init local)
//!   ;; push_control(Loop) snapshots locals_at_entry=[v_n, v_i0]
//!   ;; and substitutes local_vars = [p_n, p_i]  ← phi vars
//!   jump block_1
//!
//! block_1 (loop header):     ← br_if backward target
//!   ;; Phi instructions inserted here by emit_loop_phis at End:
//!   p_n = Phi [(block_0, v_n),  (block_2, v_n2)]   ;; $n unchanged → trivial
//!   p_i = Phi [(block_0, v_i0), (block_2, v_inew)]  ;; $i modified  → real phi
//!   v_ic = Assign(p_i)              ;; local.get $i
//!   v_ip = I32Add(v_ic, Const(1))
//!   v_inew = Assign(v_ip)          ;; local.set $i → local_vars = [p_n, v_inew]
//!   v_ic2 = Assign(v_inew)         ;; local.get $i
//!   v_nc  = Assign(p_n)            ;; local.get $n
//!   v_cmp = I32LtS(v_ic2, v_nc)
//!   ;; br_if 0: record_loop_back_branch → phi_patches += (p_i, block_1, v_inew)
//!   BranchIf(v_cmp, if_true=block_1, if_false=block_2)
//!
//! block_2 (loop exit):
//!   ;; local_vars = [p_n, v_inew]  (fall-through from br_if false path)
//!   v_ret = Assign(v_inew)         ;; local.get $i
//!   return v_ret
//! ```
//!
//! After `lower_phis`, the trivial `p_n = Phi[..., v_n]` becomes `p_n = Assign(v_n)`
//! and the real `p_i` phi is rewritten as predecessor-block assignments:
//! `block_0` gets `p_i = v_i0`, `block_1` gets `p_i = v_inew` (before its terminator).

use super::super::types::*;
use anyhow::{Context, Result};
use wasmparser::ValType;

/// Control flow frame for tracking nested blocks/loops/if.
///
/// Each variant holds only the fields relevant to that control construct,
/// making illegal states unrepresentable.
///
/// # Frame Lifecycle
///
/// 1. **Push**: When Block, Loop, If, or Else operator is encountered
/// 2. **Use**: When br/br_if/br_table references it by depth
/// 3. **Pop**: When End operator is encountered for that structure
#[derive(Debug, Clone)]
pub(super) enum ControlFrame {
    /// A `block ... end` construct.
    /// `br N` targeting this frame jumps forward to `end_block`.
    Block {
        /// Join point where all paths (fall-through + forward branches) converge.
        end_block: BlockId,
        /// Phi convergence slot for the block's result value; `None` if no result.
        result_var: Option<UseVar>,
        /// Forward branches (`br`/`br_if`/`br_table`) that target this frame's `end_block`.
        branch_incoming: Vec<(BlockId, Vec<UseVar>)>,
    },

    /// A `loop ... end` construct.
    /// `br N` targeting this frame jumps *backward* to `start_block` (re-enters the loop).
    Loop {
        /// Loop header block — target of backward branches.
        start_block: BlockId,
        /// Block immediately before the loop header; entry predecessor for loop phis.
        pre_loop_block: BlockId,
        /// Exit join point (target of forward `br` that exits the loop).
        end_block: BlockId,
        /// Phi convergence slot for the loop's result value; `None` if no result.
        result_var: Option<UseVar>,
        /// Pre-loop snapshot of `local_vars`; used as the entry predecessor for loop phis.
        locals_at_entry: Vec<UseVar>,
        /// Forward branches that exit the loop (depth > 0 past the loop frame).
        branch_incoming: Vec<(BlockId, Vec<UseVar>)>,
        /// Pre-allocated phi vars (one per Wasm local); substituted into `local_vars` at push.
        loop_phi_vars: Vec<UseVar>,
    },

    /// The then-branch of an `if ... end` or `if ... else ... end` construct.
    /// `br N` targeting this frame jumps forward to `end_block`.
    If {
        /// Pre-allocated else block (activated at `Operator::Else` or as empty-else at `End`).
        else_block: BlockId,
        /// Join point where then and else branches converge.
        end_block: BlockId,
        /// Phi convergence slot for the if's result value; `None` if no result.
        result_var: Option<UseVar>,
        /// Pre-if snapshot of `local_vars`; restored at `Operator::Else`.
        locals_at_entry: Vec<UseVar>,
        /// Forward branches from the then-body targeting `end_block`.
        branch_incoming: Vec<(BlockId, Vec<UseVar>)>,
    },

    /// The else-branch of an `if ... else ... end` construct.
    /// `br N` targeting this frame jumps forward to `end_block`.
    Else {
        /// Join point inherited from the If frame.
        end_block: BlockId,
        /// Phi convergence slot inherited from the If frame.
        result_var: Option<UseVar>,
        /// Forward branches from *both* then-body and else-body targeting `end_block`.
        branch_incoming: Vec<(BlockId, Vec<UseVar>)>,
        /// Then-branch fall-through info; `None` if the then-branch ended in dead code.
        then_pred_info: Option<(BlockId, Vec<UseVar>)>,
    },
}

impl ControlFrame {
    /// End block (join point where all paths converge).
    pub(super) fn end_block(&self) -> BlockId {
        match self {
            ControlFrame::Block { end_block, .. }
            | ControlFrame::Loop { end_block, .. }
            | ControlFrame::If { end_block, .. }
            | ControlFrame::Else { end_block, .. } => *end_block,
        }
    }

    /// Mutable reference to the forward-branch predecessor list.
    pub(super) fn branch_incoming_mut(&mut self) -> &mut Vec<(BlockId, Vec<UseVar>)> {
        match self {
            ControlFrame::Block {
                branch_incoming, ..
            }
            | ControlFrame::Loop {
                branch_incoming, ..
            }
            | ControlFrame::If {
                branch_incoming, ..
            }
            | ControlFrame::Else {
                branch_incoming, ..
            } => branch_incoming,
        }
    }

    /// Loop phi vars (one per Wasm local); empty slice for non-Loop frames.
    pub(super) fn loop_phi_vars(&self) -> &[UseVar] {
        match self {
            ControlFrame::Loop { loop_phi_vars, .. } => loop_phi_vars,
            _ => &[],
        }
    }

    /// Result var (the phi convergence slot), if any.
    pub(super) fn result_var(&self) -> Option<UseVar> {
        match self {
            ControlFrame::Block { result_var, .. }
            | ControlFrame::Loop { result_var, .. }
            | ControlFrame::If { result_var, .. }
            | ControlFrame::Else { result_var, .. } => *result_var,
        }
    }
}

/// Module-level context for function translation.
///
/// Contains information about the module's functions, types, and imports that
/// is needed during translation of individual functions.
#[derive(Debug, Clone)]
pub struct ModuleContext {
    /// Callee function signatures: (param_count, return_type) per function index.
    pub func_signatures: Vec<(usize, Option<WasmType>)>,

    /// Type section signatures: (param_count, return_type) per type index.
    /// Used for call_indirect to resolve the expected type signature.
    pub type_signatures: Vec<(usize, Option<WasmType>)>,

    /// Number of imported functions (these occupy indices 0..N-1 in the
    /// function index space, before local functions).
    pub num_imported_functions: usize,

    /// Function import details: (module_name, func_name) for each imported function.
    /// Indexed by import_idx (0..num_imported_functions-1).
    pub func_imports: Vec<(String, String)>,
}

/// IR builder state.
pub struct IrBuilder {
    /// All blocks created so far
    pub(super) blocks: Vec<IrBlock>,

    /// Current block being built
    pub(super) current_block: BlockId,

    /// Next variable ID to allocate
    pub(super) next_var_id: u32,

    /// Next block ID to allocate
    pub(super) next_block_id: u32,

    /// Wasm value stack (now SSA variables instead of actual values)
    pub(super) value_stack: Vec<UseVar>,

    /// Control flow stack for nested blocks/loops/if
    pub(super) control_stack: Vec<ControlFrame>,

    /// Mapping from Wasm local index → UseVar.
    /// Populated at the start of each `translate_function` call.
    /// Indices 0..param_count-1 are parameters; param_count.. are declared locals.
    pub(super) local_vars: Vec<UseVar>,

    /// Callee function signatures: (param_count, return_type) per function index.
    /// Set at the start of each `translate_function` call.
    pub(super) func_signatures: Vec<(usize, Option<WasmType>)>,

    /// Type section signatures: (param_count, return_type) per type index.
    /// Used for call_indirect to resolve the expected type signature.
    pub(super) type_signatures: Vec<(usize, Option<WasmType>)>,

    /// Number of imported functions (these occupy indices 0..N-1 in the
    /// function index space, before local functions).
    pub(super) num_imported_functions: usize,

    /// Function import details: (module_name, func_name) for each imported function.
    /// Indexed by import_idx (0..num_imported_functions-1).
    pub(super) func_imports: Vec<(String, String)>,

    /// True when the current insertion point is unreachable code.
    ///
    /// Set to `true` by `Br`, `BrTable`, `Return`, and `Unreachable` instructions.
    /// Cleared by `start_real_block()`.
    ///
    /// When `dead_code` is true, emitted instructions are discarded and branches
    /// are NOT recorded as phi predecessors.
    pub(super) dead_code: bool,

    /// Deferred loop phi sources from backward branches.
    ///
    /// Each entry is `(phi_dest, pred_block, src_var)`. When a `Br` targets a loop
    /// header (backward edge), we record the current values of all loop phi vars here
    /// instead of directly mutating the phi instructions. Consumed at `End` of each
    /// Loop frame by `emit_loop_phis()`.
    pub(super) phi_patches: Vec<(UseVar, BlockId, UseVar)>,
}

impl IrBuilder {
    /// Create a new IR builder.
    ///
    /// INVARIANT: The first block created (via `new_block()`) will always be `BlockId(0)`,
    /// which serves as the entry block for the function. This matches WebAssembly semantics
    /// where execution always starts at the first instruction (byte offset 0).
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            current_block: BlockId(0), // Entry block (will be created first)
            next_var_id: 0,
            next_block_id: 0, // First call to new_block() returns BlockId(0)
            value_stack: Vec::new(),
            control_stack: Vec::new(),
            local_vars: Vec::new(),
            func_signatures: Vec::new(),
            type_signatures: Vec::new(),
            num_imported_functions: 0,
            func_imports: Vec::new(),
            dead_code: false,
            phi_patches: Vec::new(),
        }
    }

    /// Allocate a new SSA variable definition token.
    ///
    /// Returns a [`DefVar`] that must be consumed by exactly one call to
    /// [`emit_def`] or [`emit_phi_def`]. The compiler will reject any attempt
    /// to emit the same variable twice.
    pub(super) fn new_var(&mut self) -> DefVar {
        let id = VarId(self.next_var_id);
        self.next_var_id += 1;
        DefVar(id)
    }

    /// Emit an instruction that produces a value.
    ///
    /// Consumes `dest` (enforcing single-definition) and returns a [`UseVar`]
    /// that can be read any number of times. The closure receives the raw
    /// [`VarId`] to embed in the [`IrInstr`].
    pub(super) fn emit_def(&mut self, dest: DefVar, f: impl FnOnce(VarId) -> IrInstr) -> UseVar {
        let id = dest.into_var_id();
        self.emit_void(f(id));
        UseVar(id)
    }

    /// Allocate a variable for phi pre-allocation or function parameters.
    ///
    /// Returns both the raw [`VarId`] (for use in [`IrInstr`] dest fields that are
    /// assembled and prepended to non-current blocks) and a [`UseVar`] for later
    /// reading. Unlike [`new_var`]+[`emit_def`], this does **not** enforce
    /// single-definition at compile time — use it only for:
    /// - Function entry parameters/locals (implicitly defined by the call)
    /// - `push_control` result_var (phi convergence slot assigned by each branch)
    /// - Loop phi pre-allocation (emitted later by `emit_loop_phis`)
    /// - `insert_phis_at_join` (phi dests inserted into non-current blocks)
    pub(super) fn new_pre_alloc_var(&mut self) -> (VarId, UseVar) {
        let id = VarId(self.next_var_id);
        self.next_var_id += 1;
        (id, UseVar(id))
    }

    /// Allocate a new basic block.
    pub(super) fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        id
    }

    /// Emit an instruction (with no result, or whose result is already embedded) to the current block.
    pub(super) fn emit_void(&mut self, instr: IrInstr) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == self.current_block) {
            block.instructions.push(instr);
        } else {
            // Current block doesn't exist yet, create it as a fallback.
            // This handles cases where instructions are emitted before explicit block creation,
            // which is valid in the IR builder's lazy block creation model.
            self.blocks.push(IrBlock {
                id: self.current_block,
                instructions: vec![instr],
                terminator: IrTerminator::Unreachable, // Will be set later
            });
        }
    }

    /// Set the terminator for the current block.
    pub(super) fn terminate(&mut self, term: IrTerminator) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == self.current_block) {
            block.terminator = term;
        }
    }

    /// Set a terminator only if code is reachable (not dead).
    ///
    /// This is the safe way to terminate blocks in branches that may be unreachable
    /// (e.g., the fall-through of an if-then without else).
    pub(super) fn terminate_if_live(&mut self, term: IrTerminator) {
        if !self.dead_code {
            self.terminate(term);
        }
    }

    /// Record the current block and local state as a phi predecessor, only if code is reachable.
    ///
    /// This is used at join points (End of Block/Loop/If) to collect predecessor information
    /// needed to build phi nodes. The predecessor is silently dropped if the current block
    /// is unreachable (dead code after an unconditional branch or return).
    pub(super) fn push_predecessor_if_live(&self, preds: &mut Vec<(BlockId, Vec<UseVar>)>) {
        if !self.dead_code {
            preds.push((self.current_block, self.local_vars.clone()));
        }
    }

    /// Translate a function from Wasm bytecode to IR.
    pub fn translate_function(
        &mut self,
        params: &[(ValType, WasmType)],
        locals: &[ValType],
        return_type: Option<WasmType>,
        operators: &[wasmparser::Operator],
        module_ctx: &ModuleContext,
    ) -> Result<IrFunction> {
        // Reset per-function state so each function starts fresh
        self.blocks.clear();
        self.value_stack.clear();
        self.control_stack.clear();
        self.next_var_id = 0;
        self.next_block_id = 0;
        self.current_block = BlockId(0);
        self.local_vars.clear();
        self.dead_code = false;
        self.phi_patches.clear();
        self.func_signatures = module_ctx.func_signatures.clone();
        self.type_signatures = module_ctx.type_signatures.clone();
        self.num_imported_functions = module_ctx.num_imported_functions;
        self.func_imports = module_ctx.func_imports.clone();

        // Allocate VarIds for all locals (params first, then declared locals).
        // This ensures local_index maps directly to the correct UseVar.
        // Parameters and zero-initialized locals are implicitly defined at function entry
        // (not via emit_def), so we use new_pre_alloc_var to get both VarId and UseVar.
        let mut local_index_to_var: Vec<UseVar> = Vec::new();

        // Allocate variables for parameters
        let param_vars: Vec<(VarId, WasmType)> = params
            .iter()
            .map(|(_, ty)| {
                let (var_id, use_var) = self.new_pre_alloc_var();
                local_index_to_var.push(use_var);
                (var_id, *ty)
            })
            .collect();

        // Allocate variables for declared locals (zero-initialized by Wasm spec)
        let mut func_locals: Vec<(VarId, WasmType)> = Vec::new();
        for vt in locals {
            let ty = WasmType::from_wasmparser(*vt);
            let (var_id, use_var) = self.new_pre_alloc_var();
            local_index_to_var.push(use_var);
            func_locals.push((var_id, ty));
        }

        self.local_vars = local_index_to_var;

        // Create entry block
        // INVARIANT: This is the first call to new_block(), so entry == BlockId(0).
        // All WebAssembly functions start execution at the first instruction,
        // so we always begin with block 0 as the entry point.
        let entry = self.new_block(); // Returns BlockId(0)
        self.current_block = entry;
        self.blocks.push(IrBlock {
            id: entry,
            instructions: Vec::new(),
            terminator: IrTerminator::Unreachable,
        });

        // Push function-level control frame
        self.push_block(entry, return_type);

        // Translate each Wasm operator to IR
        for op in operators {
            self.translate_operator(op)
                .with_context(|| format!("translating operator {:?}", op))?;
        }

        // Build final function
        Ok(IrFunction {
            params: param_vars,
            locals: func_locals,
            blocks: self.blocks.clone(),
            entry_block: entry,
            return_type,
            type_idx: TypeIdx::new(0), // Set by enrich_ir_functions during assembly
        })
    }

    /// Allocate a result variable if the block has a result type.
    fn alloc_result_var(&mut self, result_type: Option<WasmType>) -> Option<UseVar> {
        if result_type.is_some() {
            let (_, use_var) = self.new_pre_alloc_var();
            Some(use_var)
        } else {
            None
        }
    }

    /// Push a Block control frame onto the control stack.
    pub(super) fn push_block(&mut self, end_block: BlockId, result_type: Option<WasmType>) {
        let result_var = self.alloc_result_var(result_type);
        self.control_stack.push(ControlFrame::Block {
            end_block,
            result_var,
            branch_incoming: Vec::new(),
        });
    }

    /// Push a Loop control frame onto the control stack.
    ///
    /// Pre-allocates phi VarIds for all locals and immediately updates `self.local_vars`
    /// to point to them. This ensures all code inside the loop body reads/writes through
    /// the phi vars, making backward-branch phi sources correct.
    ///
    /// Must be called while `self.current_block` still points to the pre-loop block
    /// (before switching to the loop header).
    pub(super) fn push_loop(
        &mut self,
        start_block: BlockId,
        end_block: BlockId,
        result_type: Option<WasmType>,
    ) {
        let result_var = self.alloc_result_var(result_type);
        let locals_at_entry = self.local_vars.clone();
        let pre_loop_block = self.current_block;
        let loop_phi_vars: Vec<UseVar> = (0..self.local_vars.len())
            .map(|_| self.new_pre_alloc_var().1)
            .collect();
        self.local_vars.clone_from(&loop_phi_vars);
        self.control_stack.push(ControlFrame::Loop {
            start_block,
            pre_loop_block,
            end_block,
            result_var,
            locals_at_entry,
            branch_incoming: Vec::new(),
            loop_phi_vars,
        });
    }

    /// Push an If control frame onto the control stack.
    pub(super) fn push_if(
        &mut self,
        else_block: BlockId,
        end_block: BlockId,
        result_type: Option<WasmType>,
    ) {
        let result_var = self.alloc_result_var(result_type);
        let locals_at_entry = self.local_vars.clone();
        self.control_stack.push(ControlFrame::If {
            else_block,
            end_block,
            result_var,
            locals_at_entry,
            branch_incoming: Vec::new(),
        });
    }

    /// Pop a control frame from the control stack.
    pub(super) fn pop_control(&mut self) -> Result<ControlFrame> {
        self.control_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Control stack underflow"))
    }

    /// Get branch target for relative depth N.
    ///
    /// Depth 0 = innermost frame, depth 1 = next outer, etc.
    /// For loops: branch to start_block
    /// For blocks/if: branch to end_block
    pub(super) fn get_branch_target(&self, depth: u32) -> Result<BlockId> {
        let frame_idx = self
            .control_stack
            .len()
            .checked_sub(depth as usize + 1)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Branch depth {} exceeds control stack depth {}",
                    depth,
                    self.control_stack.len()
                )
            })?;

        let frame = &self.control_stack[frame_idx];

        // Loops branch back to start, others branch forward to end
        let target = match frame {
            ControlFrame::Loop { start_block, .. } => *start_block,
            _ => frame.end_block(),
        };

        Ok(target)
    }

    /// Resolve branch target and metadata for relative depth N.
    ///
    /// Returns `(target_block, is_loop, frame_idx)` needed for recording phi predecessors
    /// and determining terminator type.
    pub(super) fn resolve_branch_info(&self, depth: u32) -> Result<(BlockId, bool, usize)> {
        let frame_idx = self
            .control_stack
            .len()
            .checked_sub(depth as usize + 1)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Branch depth {} exceeds control stack depth {}",
                    depth,
                    self.control_stack.len()
                )
            })?;

        let frame = &self.control_stack[frame_idx];
        let (target, is_loop) = match frame {
            ControlFrame::Loop { start_block, .. } => (*start_block, true),
            _ => (frame.end_block(), false),
        };

        Ok((target, is_loop, frame_idx))
    }

    /// Start a new block (create and switch to it).
    pub(super) fn start_block(&mut self, block_id: BlockId) {
        self.current_block = block_id;
        self.blocks.push(IrBlock {
            id: block_id,
            instructions: Vec::new(),
            terminator: IrTerminator::Unreachable,
        });
    }

    /// Start a new reachable block: creates the block and clears `dead_code`.
    ///
    /// Use this instead of `start_block` whenever the new block is a real join point
    /// reachable from live code (e.g., after If/Else/End, or after BrIf fallthrough).
    pub(super) fn start_real_block(&mut self, block_id: BlockId) {
        self.dead_code = false;
        self.start_block(block_id);
    }

    /// Record a forward branch to a non-loop frame.
    ///
    /// Saves `(current_block, local_vars_snapshot)` in the target frame's `branch_incoming`.
    /// No-op if `dead_code` is set (unreachable branches are not phi predecessors).
    ///
    /// `frame_idx` is the index into `self.control_stack`.
    pub(super) fn record_forward_branch(&mut self, frame_idx: usize) {
        if self.dead_code {
            return;
        }
        let pred_block = self.current_block;
        let locals_snap = self.local_vars.clone();
        self.control_stack[frame_idx]
            .branch_incoming_mut()
            .push((pred_block, locals_snap));
    }

    /// Record a backward branch to a loop frame (adds to `phi_patches`).
    ///
    /// For each loop phi var, records `(phi_var, current_block, current_local_value)`.
    /// No-op if `dead_code` is set.
    ///
    /// `frame_idx` is the index into `self.control_stack` for the Loop frame.
    pub(super) fn record_loop_back_branch(&mut self, frame_idx: usize) {
        if self.dead_code {
            return;
        }
        let pred_block = self.current_block;
        // Clone to avoid borrow conflict (local_vars is also in self)
        let phi_vars = self.control_stack[frame_idx].loop_phi_vars().to_vec();
        for (local_idx, &phi_var) in phi_vars.iter().enumerate() {
            let src_var = self.local_vars[local_idx];
            self.phi_patches.push((phi_var, pred_block, src_var));
        }
    }

    /// Insert SSA phi nodes at a join block for locals with differing predecessor values.
    ///
    /// For each local index, if any predecessor provides a different VarId for that local,
    /// a `IrInstr::Phi` node is inserted at the beginning of `join_block` and
    /// `self.local_vars[i]` is updated to the phi dest.
    ///
    /// Phis are inserted in local-index order at the very start of the block's instruction
    /// list, before any instructions already in the block.
    pub(super) fn insert_phis_at_join(
        &mut self,
        join_block: BlockId,
        predecessors: &[(BlockId, Vec<UseVar>)],
    ) -> Result<()> {
        let num_locals = self.local_vars.len();
        if predecessors.is_empty() || num_locals == 0 {
            return Ok(());
        }

        // Collect phis to insert: allocate dest vars before touching self.blocks.
        let mut phi_instrs: Vec<IrInstr> = Vec::new();
        let mut new_locals = self.local_vars.clone();

        for local_idx in 0..num_locals {
            let first_var = predecessors[0].1[local_idx];
            let all_same = predecessors
                .iter()
                .all(|(_, locals)| locals[local_idx] == first_var);
            if !all_same {
                // Use new_pre_alloc_var because the phi dest is inserted into a
                // non-current block — we can't go through emit_def here.
                let (dest_id, dest_use) = self.new_pre_alloc_var();
                let srcs: Vec<(BlockId, VarId)> = predecessors
                    .iter()
                    .map(|(bid, locals)| (*bid, locals[local_idx].var_id()))
                    .collect();
                new_locals[local_idx] = dest_use;
                phi_instrs.push(IrInstr::Phi {
                    dest: dest_id,
                    srcs,
                });
            } else {
                // All predecessors agree on this local — no phi needed, but we still
                // update local_vars to the canonical value. This ensures correctness
                // when arriving from dead code (where local_vars may be stale).
                new_locals[local_idx] = first_var;
            }
        }

        self.local_vars = new_locals;

        if !phi_instrs.is_empty() {
            let block = self
                .blocks
                .iter_mut()
                .find(|b| b.id == join_block)
                .ok_or_else(|| {
                    anyhow::anyhow!("join block {:?} not found in blocks", join_block)
                })?;
            let old = std::mem::take(&mut block.instructions);
            block.instructions = phi_instrs;
            block.instructions.extend(old);
        }
        Ok(())
    }

    /// Emit phi instructions for a loop frame into its header block.
    ///
    /// Called at `End` of a Loop frame (after `pop_control`). Inserts `IrInstr::Phi`
    /// at the start of the loop header (`start_block`) for each local. Sources come from:
    /// 1. The pre-loop predecessor (`pre_loop_block`, `locals_at_entry`).
    /// 2. All backward branches recorded in `self.phi_patches` for this loop's phi vars.
    ///
    /// Consumes the relevant entries from `self.phi_patches`.
    /// Trivial phis (all sources are the same var, or the only non-self source) are left
    /// for the `lower_phis` pass to eliminate.
    pub(super) fn emit_loop_phis(&mut self, frame: &ControlFrame) {
        let (start_block, pre_loop_block, loop_phi_vars, locals_at_entry) = match frame {
            ControlFrame::Loop {
                start_block,
                pre_loop_block,
                loop_phi_vars,
                locals_at_entry,
                ..
            } => (
                *start_block,
                *pre_loop_block,
                loop_phi_vars,
                locals_at_entry,
            ),
            _ => return,
        };

        let num_locals = loop_phi_vars.len();
        if num_locals == 0 {
            return;
        }

        let mut phi_srcs: Vec<Vec<(BlockId, VarId)>> = vec![Vec::new(); num_locals];

        // Entry from before the loop (pre_loop_block is always present for Loop frames)
        for (local_idx, phi_src) in phi_srcs.iter_mut().enumerate() {
            phi_src.push((pre_loop_block, locals_at_entry[local_idx].var_id()));
        }

        // Backward branch sources from phi_patches
        for &(phi_dest, pred_block, src_var) in &self.phi_patches {
            if let Some(local_idx) = loop_phi_vars.iter().position(|v| *v == phi_dest) {
                phi_srcs[local_idx].push((pred_block, src_var.var_id()));
            }
        }

        // Consume the processed patches
        self.phi_patches
            .retain(|&(phi_dest, _, _)| !loop_phi_vars.contains(&phi_dest));

        // Build phi instructions and prepend to loop header
        let mut phi_instrs: Vec<IrInstr> = Vec::new();
        for (local_idx, &phi_var) in loop_phi_vars.iter().enumerate() {
            let srcs = std::mem::take(&mut phi_srcs[local_idx]);
            phi_instrs.push(IrInstr::Phi {
                dest: phi_var.var_id(),
                srcs,
            });
        }

        let block = self
            .blocks
            .iter_mut()
            .find(|b| b.id == start_block)
            .unwrap_or_else(|| {
                panic!(
                    "Loop header block {:?} not found when emitting loop phis. \
                     This indicates a bug in the IR builder's block or control frame management.",
                    start_block
                );
            });
        let old = std::mem::take(&mut block.instructions);
        block.instructions = phi_instrs;
        block.instructions.extend(old);
    }
}

impl Default for IrBuilder {
    fn default() -> Self {
        Self::new()
    }
}
