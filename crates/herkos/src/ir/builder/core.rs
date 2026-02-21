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
//! The builder maintains **two stacks** that together implement Wasm semantics:
//!
//! ### Value Stack (`value_stack: Vec<VarId>`)
//!
//! Replaces Wasm's implicit evaluation stack. Instead of pushing raw values, we push
//! variable IDs. Example:
//!
//! ```text
//! i32.const 5 → emit: v0 = const 5; push(v0)
//! i32.const 3 → emit: v1 = const 3; push(v1)
//! i32.add     → pop(v1), pop(v0); emit: v2 = i32_add(v0, v1); push(v2)
//! ```
//!
//! ### Control Stack (`control_stack: Vec<ControlFrame>`)
//!
//! Tracks nested blocks/loops/if structures. Each frame records:
//! - `kind`: Block | Loop | If | Else
//! - `start_block`: jump target for loops (backward)
//! - `end_block`: jump target for block exit (forward/join)
//! - `else_block`: false branch for If
//! - `result_var`: PHI node if block has a result type
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
//!    and create new basic blocks as needed.
//!
//! 4. Return: `IrFunction` with all blocks, variables typed, and control flow explicit.
//!
//! ## Invariants
//!
//! - **Entry block**: First `new_block()` call returns `BlockId(0)` (function entry).
//! - **Value stack**: Operations pop N arguments, push ≤1 result.
//! - **Control stack**: Push/pop balanced; each frame has a unique start/end pair.
//! - **Local variables**: `local_vars[i]` = VarId for Wasm local index i (set once at start).
//!
//! ## Example 1
//!
//! Wasm: `(func (param $a i32) → i32 (local.get $a) (i32.const 1) (i32.add))`
//!
//! Generated IR:
//! ```text
//! block_0:
//!   v2 = i32_add(v0, v1)
//!   return v2
//! ```
//! where v0 = param $a, v1 = const 1 (allocated during init/translation).
//!
//! ## Example 2 (if-else)
//!
//! ```text
//! Wasm:
//!   if i32
//!     (const 1)
//!   else
//!     (const 2)
//!   end
//! ```
//!
//! Generated blocks:
//!
//! ```text
//! ┌──────────────────┐
//! │   block_0        │  Entry
//! │  (condition)     │
//! │  br_if block_2   │
//! │  br block_3      │
//! └────────┬─────────┘
//!          │
//!     ┌────┴────┐
//!     ▼         ▼
//! ┌─────────┐ ┌─────────┐
//! │ block_1 │ │ block_2 │  True and False branches
//! │ v1=1    │ │ v2=2    │
//! │ br block│ │ br block│
//! │   3     │ │   3     │
//! └────┬────┘ └────┬────┘
//!      │           │
//!      └────┬──────┘
//!           ▼
//!     ┌──────────────┐
//!     │  block_3     │  Join point
//!     │ v0=phi(v1,v2)│
//!     └──────────────┘
//! ```

use super::super::types::*;
use anyhow::{Context, Result};
use wasmparser::ValType;

/// Control flow frame for tracking nested blocks/loops/if.
///
/// Each control structure (block, loop, if, else) pushes a frame onto the control stack.
/// When translating branch instructions (br, br_if, br_table), we look up the frame
/// at the specified depth to determine the branch target. When an End operator is hit,
/// we pop the frame and finalize the structure.
///
/// # Frame Lifecycle
///
/// 1. **Push**: When Block, Loop, If, or Else operator is encountered
/// 2. **Use**: When br/br_if/br_table references it by depth
/// 3. **Pop**: When End operator is encountered for that structure
///
/// # Invariants
///
/// - Every frame has start_block ≠ end_block (except in edge cases)
/// - If result_var is Some, both branches must produce that variable's type
/// - else_block is only Some for If frames (deferred activation)
/// - All BlockIds in a frame must be distinct
#[derive(Debug, Clone)]
pub(super) struct ControlFrame {
    /// Kind of control structure (Block, Loop, If, Else)
    ///
    /// Determines how `br` instructions behave:
    /// - **Block**: `br` jumps forward to end_block (exit the block)
    /// - **Loop**: `br` jumps backward to start_block (re-enter the loop)
    /// - **If**: `br` jumps forward to end_block (exit the if/else)
    /// - **Else**: `br` jumps forward to end_block (exit the else branch)
    pub(super) kind: ControlKind,

    /// Start block (where to jump when looping, or where we are for blocks)
    ///
    /// **Meaning depends on kind**:
    /// - **Block**: The current block we're building in (no new block created)
    /// - **Loop**: The loop header block (where backward `br` jumps to re-enter)
    /// - **If**: The then-block (where true branch starts)
    /// - **Else**: The else-block where the else branch is executing. This is the
    ///   *same BlockId* as the parent If frame's `else_block` (retrieved when
    ///   `Operator::Else` was processed and activated). Unlike Loop's `start_block`,
    ///   it is NOT used by branch resolution — `br` inside an else body targets
    ///   `end_block`, not `start_block`. Stored here for documentation/context only.
    ///
    /// For backward branches (br in a loop), this is the jump target.
    pub(super) start_block: BlockId,

    /// End block (join point where all paths converge)
    ///
    /// **Meaning is consistent across all kinds**:
    /// - **Block**: Where `br` jumps to (exit the block)
    /// - **Loop**: Where `br` with depth > 0 jumps to (exit the loop)
    /// - **If**: Where both then and else branches merge
    /// - **Else**: Where the else branch merges back to the if's end
    ///
    /// This block is activated with `start_block(end_block)` when the structure's
    /// End operator is encountered. It's the join point where control flow resumes
    /// after the entire control structure (block/loop/if/else) completes.
    pub(super) end_block: BlockId,

    /// Else block (only for If constructs; None for Block/Loop/Else)
    ///
    /// For If frames:
    /// - Allocated upfront when If operator is processed
    /// - Activated (with `start_block()`) when Else operator is encountered
    /// - If no Else operator appears before End, remains empty (only contains jump to end_block)
    ///
    /// This field enables deferred activation: we know where the else branch will
    /// execute before we actually start generating its code. Both then and else
    /// branches must eventually jump to end_block.
    ///
    /// **Lifecycle**: When `Operator::Else` is processed, this BlockId is retrieved,
    /// activated, and becomes the `start_block` of the newly pushed Else frame.
    /// In other words: `If frame's else_block` == `Else frame's start_block`
    /// (same BlockId, different lifecycle phase: deferred vs. active).
    ///
    /// For Block/Loop/Else frames: always None.
    pub(super) else_block: Option<BlockId>,

    /// Result type of the control structure (None if no result)
    ///
    /// If the block/loop/if has a result type (e.g., "block i32 ... end"),
    /// both branches must produce a value of this type at their exit.
    ///
    /// Example:
    /// ```wasm
    /// block i32         ◄─── result_type = Some(I32)
    ///   i32.const 5
    /// end               ◄─── Must have an i32 value on stack
    /// ```
    ///
    /// Used to allocate result_var if needed.
    pub(super) result_type: Option<WasmType>,

    /// Result variable (PHI node placeholder for the control structure's result)
    ///
    /// If result_type is Some, result_var holds the VarId that will contain
    /// the final result value at the join point (end_block).
    ///
    /// **Flow**:
    /// 1. When the control structure is pushed, result_var is allocated (if result_type is Some)
    /// 2. As each branch executes, it computes a value (v0, v1, etc.)
    /// 3. Before jumping to end_block, each branch assigns: result_var = branch_value
    /// 4. At end_block, result_var contains the unified result
    ///
    /// **Example**:
    /// ```text
    /// block i32
    ///   result_var = v5
    ///   <then branch computes v1>
    ///   v5 = v1
    ///   br end_block
    /// end
    ///
    /// (at end_block)
    /// v5 contains the result, pushed onto value_stack
    /// ```
    ///
    /// If result_var is None, the structure produces no value.
    pub(super) result_var: Option<VarId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlKind {
    Block, // Forward branches only
    Loop,  // Backward branch to start
    If,    // Conditional with possible else
    Else,  // Else branch of if
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
    pub(super) value_stack: Vec<VarId>,

    /// Control flow stack for nested blocks/loops/if
    pub(super) control_stack: Vec<ControlFrame>,

    /// Mapping from Wasm local index → VarId.
    /// Populated at the start of each `translate_function` call.
    /// Indices 0..param_count-1 are parameters; param_count.. are declared locals.
    pub(super) local_vars: Vec<VarId>,

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
        }
    }

    /// Allocate a new SSA variable.
    pub(super) fn new_var(&mut self) -> VarId {
        let id = VarId(self.next_var_id);
        self.next_var_id += 1;
        id
    }

    /// Allocate a new basic block.
    pub(super) fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        id
    }

    /// Emit an instruction to the current block.
    pub(super) fn emit(&mut self, instr: IrInstr) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == self.current_block) {
            block.instructions.push(instr);
        } else {
            // Current block doesn't exist yet, create it
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
        self.func_signatures = module_ctx.func_signatures.clone();
        self.type_signatures = module_ctx.type_signatures.clone();
        self.num_imported_functions = module_ctx.num_imported_functions;
        self.func_imports = module_ctx.func_imports.clone();

        // Allocate VarIds for all locals (params first, then declared locals).
        // This ensures local_index maps directly to the correct VarId.
        let mut local_index_to_var: Vec<VarId> = Vec::new();

        // Allocate variables for parameters
        let param_vars: Vec<(VarId, WasmType)> = params
            .iter()
            .map(|(_, ty)| {
                let var = self.new_var();
                local_index_to_var.push(var);
                (var, *ty)
            })
            .collect();

        // Allocate variables for declared locals (zero-initialized by Wasm spec)
        let mut func_locals: Vec<(VarId, WasmType)> = Vec::new();
        for vt in locals {
            let ty = WasmType::from_wasmparser(*vt);
            let var = self.new_var();
            local_index_to_var.push(var);
            func_locals.push((var, ty));
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
        self.push_control(ControlKind::Block, entry, entry, None, return_type);

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
            needs_host: false,         // Set by enrich_ir_functions during assembly
        })
    }

    /// Push a control frame onto the control stack.
    pub(super) fn push_control(
        &mut self,
        kind: ControlKind,
        start_block: BlockId,
        end_block: BlockId,
        else_block: Option<BlockId>,
        result_type: Option<WasmType>,
    ) {
        // Allocate a result variable if block has result type
        let result_var = if result_type.is_some() {
            Some(self.new_var())
        } else {
            None
        };

        self.control_stack.push(ControlFrame {
            kind,
            start_block,
            end_block,
            else_block,
            result_type,
            result_var,
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
        let target = match frame.kind {
            ControlKind::Loop => frame.start_block,
            _ => frame.end_block,
        };

        Ok(target)
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
}

impl Default for IrBuilder {
    fn default() -> Self {
        Self::new()
    }
}
