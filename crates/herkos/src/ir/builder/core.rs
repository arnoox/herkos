//! Core IR builder state and control flow management.
//!
//! This module contains the `IrBuilder` state machine that translates Wasm bytecode
//! to SSA-form IR by simulating the Wasm evaluation stack.

use super::super::types::*;
use anyhow::{Context, Result};
use wasmparser::ValType;

/// Control flow frame for tracking nested blocks/loops/if.
#[derive(Debug, Clone)]
pub(super) struct ControlFrame {
    /// Kind of control structure
    pub(super) kind: ControlKind,

    /// Start block (where loop branches return to)
    pub(super) start_block: BlockId,

    /// End block (where forward branches go to)
    pub(super) end_block: BlockId,

    /// Else block (for If constructs - where the false branch goes)
    pub(super) else_block: Option<BlockId>,

    /// Result type (None for void)
    pub(super) result_type: Option<WasmType>,

    /// Result variable (for blocks with result type - PHI node)
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
                label: format!("block_{}", self.current_block.0),
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
            label: format!("block_{}", entry.0), // "block_0"
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
            type_idx: 0,       // Set by enrich_ir_functions during assembly
            needs_host: false, // Set by enrich_ir_functions during assembly
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
            label: format!("block_{}", block_id.0),
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
