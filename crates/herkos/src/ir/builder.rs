//! IR builder — translates WebAssembly bytecode to SSA-form IR.
//!
//! This module implements a modified stack machine interpreter that generates
//! IR instructions instead of executing them. Each value on the Wasm evaluation
//! stack becomes an SSA variable.

use super::types::*;
use crate::parser::{ExportKind, ImportKind, ParsedModule};
use crate::TranspileOptions;
use anyhow::{bail, Context, Result};
use wasmparser::{Operator, ValType};

/// Control flow frame for tracking nested blocks/loops/if.
#[derive(Debug, Clone)]
struct ControlFrame {
    /// Kind of control structure
    kind: ControlKind,

    /// Start block (where loop branches return to)
    start_block: BlockId,

    /// End block (where forward branches go to)
    end_block: BlockId,

    /// Else block (for If constructs - where the false branch goes)
    else_block: Option<BlockId>,

    /// Result type (None for void)
    result_type: Option<WasmType>,

    /// Result variable (for blocks with result type - PHI node)
    result_var: Option<VarId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlKind {
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
    blocks: Vec<IrBlock>,

    /// Current block being built
    current_block: BlockId,

    /// Next variable ID to allocate
    next_var_id: u32,

    /// Next block ID to allocate
    next_block_id: u32,

    /// Wasm value stack (now SSA variables instead of actual values)
    value_stack: Vec<VarId>,

    /// Control flow stack for nested blocks/loops/if
    control_stack: Vec<ControlFrame>,

    /// Mapping from Wasm local index → VarId.
    /// Populated at the start of each `translate_function` call.
    /// Indices 0..param_count-1 are parameters; param_count.. are declared locals.
    local_vars: Vec<VarId>,

    /// Callee function signatures: (param_count, return_type) per function index.
    /// Set at the start of each `translate_function` call.
    func_signatures: Vec<(usize, Option<WasmType>)>,

    /// Type section signatures: (param_count, return_type) per type index.
    /// Used for call_indirect to resolve the expected type signature.
    type_signatures: Vec<(usize, Option<WasmType>)>,

    /// Number of imported functions (these occupy indices 0..N-1 in the
    /// function index space, before local functions).
    num_imported_functions: usize,

    /// Function import details: (module_name, func_name) for each imported function.
    /// Indexed by import_idx (0..num_imported_functions-1).
    func_imports: Vec<(String, String)>,
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
    fn new_var(&mut self) -> VarId {
        let id = VarId(self.next_var_id);
        self.next_var_id += 1;
        id
    }

    /// Allocate a new basic block.
    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        id
    }

    /// Emit an instruction to the current block.
    fn emit(&mut self, instr: IrInstr) {
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
    fn terminate(&mut self, term: IrTerminator) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == self.current_block) {
            block.terminator = term;
        }
    }

    /// Translate a function from Wasm bytecode to IR.
    fn translate_function(
        &mut self,
        params: &[(ValType, WasmType)],
        locals: &[ValType],
        return_type: Option<WasmType>,
        operators: &[Operator],
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
        })
    }

    /// Translate a single Wasm operator to IR instructions.
    fn translate_operator(&mut self, op: &Operator) -> Result<()> {
        match op {
            // Constants
            Operator::I32Const { value } => {
                let dest = self.new_var();
                self.emit(IrInstr::Const {
                    dest,
                    value: IrValue::I32(*value),
                });
                self.value_stack.push(dest);
            }

            Operator::I64Const { value } => {
                let dest = self.new_var();
                self.emit(IrInstr::Const {
                    dest,
                    value: IrValue::I64(*value),
                });
                self.value_stack.push(dest);
            }

            Operator::F32Const { value } => {
                let dest = self.new_var();
                self.emit(IrInstr::Const {
                    dest,
                    value: IrValue::F32(f32::from_bits(value.bits())),
                });
                self.value_stack.push(dest);
            }

            Operator::F64Const { value } => {
                let dest = self.new_var();
                self.emit(IrInstr::Const {
                    dest,
                    value: IrValue::F64(f64::from_bits(value.bits())),
                });
                self.value_stack.push(dest);
            }

            // Local variable access
            Operator::LocalGet { local_index } => {
                let var = self
                    .local_vars
                    .get(*local_index as usize)
                    .copied()
                    .ok_or_else(|| {
                        anyhow::anyhow!("local.get: local index {} out of range", local_index)
                    })?;
                self.value_stack.push(var);
            }

            Operator::LocalSet { local_index } => {
                // Pop value and assign to local
                let value = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for local.set"))?;

                let dest = self
                    .local_vars
                    .get(*local_index as usize)
                    .copied()
                    .ok_or_else(|| {
                        anyhow::anyhow!("local.set: local index {} out of range", local_index)
                    })?;
                self.emit(IrInstr::Assign { dest, src: value });
            }

            Operator::LocalTee { local_index } => {
                // Like LocalSet but keeps value on stack
                let value = self
                    .value_stack
                    .last()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for local.tee"))?;

                let dest = self
                    .local_vars
                    .get(*local_index as usize)
                    .copied()
                    .ok_or_else(|| {
                        anyhow::anyhow!("local.tee: local index {} out of range", local_index)
                    })?;
                self.emit(IrInstr::Assign { dest, src: *value });
                // Value stays on stack (we already have it via .last())
            }

            // Global variable access
            Operator::GlobalGet { global_index } => {
                let dest = self.new_var();
                self.emit(IrInstr::GlobalGet {
                    dest,
                    index: *global_index as usize,
                });
                self.value_stack.push(dest);
            }

            Operator::GlobalSet { global_index } => {
                let value = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for global.set"))?;
                self.emit(IrInstr::GlobalSet {
                    index: *global_index as usize,
                    value,
                });
            }

            // === i32 binary operations ===
            Operator::I32Add => self.emit_binop(BinOp::I32Add)?,
            Operator::I32Sub => self.emit_binop(BinOp::I32Sub)?,
            Operator::I32Mul => self.emit_binop(BinOp::I32Mul)?,
            Operator::I32DivS => self.emit_binop(BinOp::I32DivS)?,
            Operator::I32DivU => self.emit_binop(BinOp::I32DivU)?,
            Operator::I32RemS => self.emit_binop(BinOp::I32RemS)?,
            Operator::I32RemU => self.emit_binop(BinOp::I32RemU)?,
            Operator::I32And => self.emit_binop(BinOp::I32And)?,
            Operator::I32Or => self.emit_binop(BinOp::I32Or)?,
            Operator::I32Xor => self.emit_binop(BinOp::I32Xor)?,
            Operator::I32Shl => self.emit_binop(BinOp::I32Shl)?,
            Operator::I32ShrS => self.emit_binop(BinOp::I32ShrS)?,
            Operator::I32ShrU => self.emit_binop(BinOp::I32ShrU)?,
            Operator::I32Rotl => self.emit_binop(BinOp::I32Rotl)?,
            Operator::I32Rotr => self.emit_binop(BinOp::I32Rotr)?,

            // i32 comparisons
            Operator::I32Eq => self.emit_binop(BinOp::I32Eq)?,
            Operator::I32Ne => self.emit_binop(BinOp::I32Ne)?,
            Operator::I32LtS => self.emit_binop(BinOp::I32LtS)?,
            Operator::I32LtU => self.emit_binop(BinOp::I32LtU)?,
            Operator::I32GtS => self.emit_binop(BinOp::I32GtS)?,
            Operator::I32GtU => self.emit_binop(BinOp::I32GtU)?,
            Operator::I32LeS => self.emit_binop(BinOp::I32LeS)?,
            Operator::I32LeU => self.emit_binop(BinOp::I32LeU)?,
            Operator::I32GeS => self.emit_binop(BinOp::I32GeS)?,
            Operator::I32GeU => self.emit_binop(BinOp::I32GeU)?,

            // i32 unary
            Operator::I32Eqz => self.emit_unop(UnOp::I32Eqz)?,
            Operator::I32Clz => self.emit_unop(UnOp::I32Clz)?,
            Operator::I32Ctz => self.emit_unop(UnOp::I32Ctz)?,
            Operator::I32Popcnt => self.emit_unop(UnOp::I32Popcnt)?,

            // === i64 binary operations ===
            Operator::I64Add => self.emit_binop(BinOp::I64Add)?,
            Operator::I64Sub => self.emit_binop(BinOp::I64Sub)?,
            Operator::I64Mul => self.emit_binop(BinOp::I64Mul)?,
            Operator::I64DivS => self.emit_binop(BinOp::I64DivS)?,
            Operator::I64DivU => self.emit_binop(BinOp::I64DivU)?,
            Operator::I64RemS => self.emit_binop(BinOp::I64RemS)?,
            Operator::I64RemU => self.emit_binop(BinOp::I64RemU)?,
            Operator::I64And => self.emit_binop(BinOp::I64And)?,
            Operator::I64Or => self.emit_binop(BinOp::I64Or)?,
            Operator::I64Xor => self.emit_binop(BinOp::I64Xor)?,
            Operator::I64Shl => self.emit_binop(BinOp::I64Shl)?,
            Operator::I64ShrS => self.emit_binop(BinOp::I64ShrS)?,
            Operator::I64ShrU => self.emit_binop(BinOp::I64ShrU)?,
            Operator::I64Rotl => self.emit_binop(BinOp::I64Rotl)?,
            Operator::I64Rotr => self.emit_binop(BinOp::I64Rotr)?,

            // i64 comparisons
            Operator::I64Eq => self.emit_binop(BinOp::I64Eq)?,
            Operator::I64Ne => self.emit_binop(BinOp::I64Ne)?,
            Operator::I64LtS => self.emit_binop(BinOp::I64LtS)?,
            Operator::I64LtU => self.emit_binop(BinOp::I64LtU)?,
            Operator::I64GtS => self.emit_binop(BinOp::I64GtS)?,
            Operator::I64GtU => self.emit_binop(BinOp::I64GtU)?,
            Operator::I64LeS => self.emit_binop(BinOp::I64LeS)?,
            Operator::I64LeU => self.emit_binop(BinOp::I64LeU)?,
            Operator::I64GeS => self.emit_binop(BinOp::I64GeS)?,
            Operator::I64GeU => self.emit_binop(BinOp::I64GeU)?,

            // i64 unary
            Operator::I64Eqz => self.emit_unop(UnOp::I64Eqz)?,
            Operator::I64Clz => self.emit_unop(UnOp::I64Clz)?,
            Operator::I64Ctz => self.emit_unop(UnOp::I64Ctz)?,
            Operator::I64Popcnt => self.emit_unop(UnOp::I64Popcnt)?,

            // === f32 binary operations ===
            Operator::F32Add => self.emit_binop(BinOp::F32Add)?,
            Operator::F32Sub => self.emit_binop(BinOp::F32Sub)?,
            Operator::F32Mul => self.emit_binop(BinOp::F32Mul)?,
            Operator::F32Div => self.emit_binop(BinOp::F32Div)?,
            Operator::F32Min => self.emit_binop(BinOp::F32Min)?,
            Operator::F32Max => self.emit_binop(BinOp::F32Max)?,
            Operator::F32Copysign => self.emit_binop(BinOp::F32Copysign)?,

            // f32 comparisons
            Operator::F32Eq => self.emit_binop(BinOp::F32Eq)?,
            Operator::F32Ne => self.emit_binop(BinOp::F32Ne)?,
            Operator::F32Lt => self.emit_binop(BinOp::F32Lt)?,
            Operator::F32Gt => self.emit_binop(BinOp::F32Gt)?,
            Operator::F32Le => self.emit_binop(BinOp::F32Le)?,
            Operator::F32Ge => self.emit_binop(BinOp::F32Ge)?,

            // f32 unary
            Operator::F32Abs => self.emit_unop(UnOp::F32Abs)?,
            Operator::F32Neg => self.emit_unop(UnOp::F32Neg)?,
            Operator::F32Ceil => self.emit_unop(UnOp::F32Ceil)?,
            Operator::F32Floor => self.emit_unop(UnOp::F32Floor)?,
            Operator::F32Trunc => self.emit_unop(UnOp::F32Trunc)?,
            Operator::F32Nearest => self.emit_unop(UnOp::F32Nearest)?,
            Operator::F32Sqrt => self.emit_unop(UnOp::F32Sqrt)?,

            // === f64 binary operations ===
            Operator::F64Add => self.emit_binop(BinOp::F64Add)?,
            Operator::F64Sub => self.emit_binop(BinOp::F64Sub)?,
            Operator::F64Mul => self.emit_binop(BinOp::F64Mul)?,
            Operator::F64Div => self.emit_binop(BinOp::F64Div)?,
            Operator::F64Min => self.emit_binop(BinOp::F64Min)?,
            Operator::F64Max => self.emit_binop(BinOp::F64Max)?,
            Operator::F64Copysign => self.emit_binop(BinOp::F64Copysign)?,

            // f64 comparisons
            Operator::F64Eq => self.emit_binop(BinOp::F64Eq)?,
            Operator::F64Ne => self.emit_binop(BinOp::F64Ne)?,
            Operator::F64Lt => self.emit_binop(BinOp::F64Lt)?,
            Operator::F64Gt => self.emit_binop(BinOp::F64Gt)?,
            Operator::F64Le => self.emit_binop(BinOp::F64Le)?,
            Operator::F64Ge => self.emit_binop(BinOp::F64Ge)?,

            // f64 unary
            Operator::F64Abs => self.emit_unop(UnOp::F64Abs)?,
            Operator::F64Neg => self.emit_unop(UnOp::F64Neg)?,
            Operator::F64Ceil => self.emit_unop(UnOp::F64Ceil)?,
            Operator::F64Floor => self.emit_unop(UnOp::F64Floor)?,
            Operator::F64Trunc => self.emit_unop(UnOp::F64Trunc)?,
            Operator::F64Nearest => self.emit_unop(UnOp::F64Nearest)?,
            Operator::F64Sqrt => self.emit_unop(UnOp::F64Sqrt)?,

            // === Conversion operations ===
            Operator::I32WrapI64 => self.emit_unop(UnOp::I32WrapI64)?,
            Operator::I64ExtendI32S => self.emit_unop(UnOp::I64ExtendI32S)?,
            Operator::I64ExtendI32U => self.emit_unop(UnOp::I64ExtendI32U)?,
            Operator::I32TruncF32S => self.emit_unop(UnOp::I32TruncF32S)?,
            Operator::I32TruncF32U => self.emit_unop(UnOp::I32TruncF32U)?,
            Operator::I32TruncF64S => self.emit_unop(UnOp::I32TruncF64S)?,
            Operator::I32TruncF64U => self.emit_unop(UnOp::I32TruncF64U)?,
            Operator::I64TruncF32S => self.emit_unop(UnOp::I64TruncF32S)?,
            Operator::I64TruncF32U => self.emit_unop(UnOp::I64TruncF32U)?,
            Operator::I64TruncF64S => self.emit_unop(UnOp::I64TruncF64S)?,
            Operator::I64TruncF64U => self.emit_unop(UnOp::I64TruncF64U)?,
            Operator::F32ConvertI32S => self.emit_unop(UnOp::F32ConvertI32S)?,
            Operator::F32ConvertI32U => self.emit_unop(UnOp::F32ConvertI32U)?,
            Operator::F32ConvertI64S => self.emit_unop(UnOp::F32ConvertI64S)?,
            Operator::F32ConvertI64U => self.emit_unop(UnOp::F32ConvertI64U)?,
            Operator::F64ConvertI32S => self.emit_unop(UnOp::F64ConvertI32S)?,
            Operator::F64ConvertI32U => self.emit_unop(UnOp::F64ConvertI32U)?,
            Operator::F64ConvertI64S => self.emit_unop(UnOp::F64ConvertI64S)?,
            Operator::F64ConvertI64U => self.emit_unop(UnOp::F64ConvertI64U)?,
            Operator::F32DemoteF64 => self.emit_unop(UnOp::F32DemoteF64)?,
            Operator::F64PromoteF32 => self.emit_unop(UnOp::F64PromoteF32)?,
            Operator::I32ReinterpretF32 => self.emit_unop(UnOp::I32ReinterpretF32)?,
            Operator::I64ReinterpretF64 => self.emit_unop(UnOp::I64ReinterpretF64)?,
            Operator::F32ReinterpretI32 => self.emit_unop(UnOp::F32ReinterpretI32)?,
            Operator::F64ReinterpretI64 => self.emit_unop(UnOp::F64ReinterpretI64)?,

            // Return
            Operator::Return => {
                let value = if self.value_stack.is_empty() {
                    None
                } else {
                    Some(
                        self.value_stack
                            .pop()
                            .ok_or_else(|| anyhow::anyhow!("stack underflow in Return"))?,
                    )
                };
                self.terminate(IrTerminator::Return { value });
                // Start a dead-code continuation block so subsequent Wasm
                // instructions (e.g. End for enclosing blocks) don't corrupt
                // the already-terminated block's control flow.
                let dead_block = self.new_block();
                self.start_block(dead_block);
            }

            // End (end of function or block)
            Operator::End => {
                if self.control_stack.len() <= 1 {
                    // End of function - treat as implicit return
                    let value = if self.value_stack.is_empty() {
                        None
                    } else {
                        Some(self.value_stack.pop().ok_or_else(|| {
                            anyhow::anyhow!("stack underflow in End (function return)")
                        })?)
                    };
                    self.terminate(IrTerminator::Return { value });
                } else {
                    // End of block/loop/if/else
                    let frame = self.pop_control()?;

                    // If this is an If without Else, we need to create the else block
                    if frame.kind == ControlKind::If {
                        if let Some(else_block) = frame.else_block {
                            // Then branch: assign result if needed (only if value is on stack)
                            if let Some(result_var) = frame.result_var {
                                if let Some(stack_value) = self.value_stack.pop() {
                                    self.emit(IrInstr::Assign {
                                        dest: result_var,
                                        src: stack_value,
                                    });
                                }
                                // If stack is empty, then branch ended with br/return (unreachable after)
                            }

                            // Current block (then branch) jumps to end
                            self.terminate(IrTerminator::Jump {
                                target: frame.end_block,
                            });

                            // Create the unused else block
                            self.start_block(else_block);

                            // Else block is empty, just jump to end
                            self.terminate(IrTerminator::Jump {
                                target: frame.end_block,
                            });

                            // Continue building in end block
                            self.start_block(frame.end_block);
                        } else {
                            // Should not happen - If always has else_block
                            bail!("If frame missing else_block");
                        }
                    } else {
                        // Block/Loop/Else: assign result if needed (only if value is on stack)
                        if let Some(result_var) = frame.result_var {
                            if let Some(stack_value) = self.value_stack.pop() {
                                self.emit(IrInstr::Assign {
                                    dest: result_var,
                                    src: stack_value,
                                });
                            }
                            // If stack is empty, block ended with br/return (unreachable after)
                        }

                        // Current block jumps to end block
                        self.terminate(IrTerminator::Jump {
                            target: frame.end_block,
                        });

                        // Continue building in end block
                        self.start_block(frame.end_block);
                    }

                    // If block has result type, push result var onto stack
                    if let Some(result_var) = frame.result_var {
                        self.value_stack.push(result_var);
                    }
                }
            }

            // Nop does nothing
            Operator::Nop => {
                // No-op: do nothing
            }

            // Drop removes top value from stack
            Operator::Drop => {
                if self.value_stack.is_empty() {
                    bail!("Stack underflow for drop");
                }
                self.value_stack.pop();
            }

            // === Memory loads ===
            // Full-width loads
            Operator::I32Load { memarg } => {
                self.emit_load(WasmType::I32, memarg.offset)?;
            }
            Operator::I64Load { memarg } => {
                self.emit_load(WasmType::I64, memarg.offset)?;
            }
            Operator::F32Load { memarg } => {
                self.emit_load(WasmType::F32, memarg.offset)?;
            }
            Operator::F64Load { memarg } => {
                self.emit_load(WasmType::F64, memarg.offset)?;
            }

            // Sub-width i32 loads
            Operator::I32Load8S { memarg } => {
                self.emit_load_ext(
                    WasmType::I32,
                    memarg.offset,
                    MemoryAccessWidth::I8,
                    Some(SignExtension::Signed),
                )?;
            }
            Operator::I32Load8U { memarg } => {
                self.emit_load_ext(
                    WasmType::I32,
                    memarg.offset,
                    MemoryAccessWidth::I8,
                    Some(SignExtension::Unsigned),
                )?;
            }
            Operator::I32Load16S { memarg } => {
                self.emit_load_ext(
                    WasmType::I32,
                    memarg.offset,
                    MemoryAccessWidth::I16,
                    Some(SignExtension::Signed),
                )?;
            }
            Operator::I32Load16U { memarg } => {
                self.emit_load_ext(
                    WasmType::I32,
                    memarg.offset,
                    MemoryAccessWidth::I16,
                    Some(SignExtension::Unsigned),
                )?;
            }

            // Sub-width i64 loads
            Operator::I64Load8S { memarg } => {
                self.emit_load_ext(
                    WasmType::I64,
                    memarg.offset,
                    MemoryAccessWidth::I8,
                    Some(SignExtension::Signed),
                )?;
            }
            Operator::I64Load8U { memarg } => {
                self.emit_load_ext(
                    WasmType::I64,
                    memarg.offset,
                    MemoryAccessWidth::I8,
                    Some(SignExtension::Unsigned),
                )?;
            }
            Operator::I64Load16S { memarg } => {
                self.emit_load_ext(
                    WasmType::I64,
                    memarg.offset,
                    MemoryAccessWidth::I16,
                    Some(SignExtension::Signed),
                )?;
            }
            Operator::I64Load16U { memarg } => {
                self.emit_load_ext(
                    WasmType::I64,
                    memarg.offset,
                    MemoryAccessWidth::I16,
                    Some(SignExtension::Unsigned),
                )?;
            }
            Operator::I64Load32S { memarg } => {
                self.emit_load_ext(
                    WasmType::I64,
                    memarg.offset,
                    MemoryAccessWidth::I32,
                    Some(SignExtension::Signed),
                )?;
            }
            Operator::I64Load32U { memarg } => {
                self.emit_load_ext(
                    WasmType::I64,
                    memarg.offset,
                    MemoryAccessWidth::I32,
                    Some(SignExtension::Unsigned),
                )?;
            }

            // === Memory stores ===
            // Full-width stores
            Operator::I32Store { memarg } => {
                self.emit_store(WasmType::I32, memarg.offset)?;
            }
            Operator::I64Store { memarg } => {
                self.emit_store(WasmType::I64, memarg.offset)?;
            }
            Operator::F32Store { memarg } => {
                self.emit_store(WasmType::F32, memarg.offset)?;
            }
            Operator::F64Store { memarg } => {
                self.emit_store(WasmType::F64, memarg.offset)?;
            }

            // Sub-width stores
            Operator::I32Store8 { memarg } => {
                self.emit_store_narrow(WasmType::I32, memarg.offset, MemoryAccessWidth::I8)?;
            }
            Operator::I32Store16 { memarg } => {
                self.emit_store_narrow(WasmType::I32, memarg.offset, MemoryAccessWidth::I16)?;
            }
            Operator::I64Store8 { memarg } => {
                self.emit_store_narrow(WasmType::I64, memarg.offset, MemoryAccessWidth::I8)?;
            }
            Operator::I64Store16 { memarg } => {
                self.emit_store_narrow(WasmType::I64, memarg.offset, MemoryAccessWidth::I16)?;
            }
            Operator::I64Store32 { memarg } => {
                self.emit_store_narrow(WasmType::I64, memarg.offset, MemoryAccessWidth::I32)?;
            }

            // === Memory size and grow ===
            Operator::MemorySize { mem: 0, .. } => {
                let dest = self.new_var();
                self.emit(IrInstr::MemorySize { dest });
                self.value_stack.push(dest);
            }

            Operator::MemoryGrow { mem: 0, .. } => {
                let delta = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for memory.grow"))?;
                let dest = self.new_var();
                self.emit(IrInstr::MemoryGrow { dest, delta });
                self.value_stack.push(dest);
            }

            // Control flow (Milestone 3)
            Operator::Block { blockty } => {
                let result_type = match blockty {
                    wasmparser::BlockType::Empty => None,
                    wasmparser::BlockType::Type(vt) => Some(WasmType::from_wasmparser(*vt)),
                    wasmparser::BlockType::FuncType(_) => bail!("Multi-value blocks not supported"),
                };

                let end_block = self.new_block();
                let start_block = self.current_block;

                self.push_control(
                    ControlKind::Block,
                    start_block,
                    end_block,
                    None,
                    result_type,
                );
            }

            Operator::Loop { blockty } => {
                let result_type = match blockty {
                    wasmparser::BlockType::Empty => None,
                    wasmparser::BlockType::Type(vt) => Some(WasmType::from_wasmparser(*vt)),
                    wasmparser::BlockType::FuncType(_) => bail!("Multi-value blocks not supported"),
                };

                // Loop: start block is the loop header (for backward branches)
                let loop_header = self.new_block();
                let end_block = self.new_block();

                // Jump to loop header
                self.terminate(IrTerminator::Jump {
                    target: loop_header,
                });

                // Start building loop body
                self.start_block(loop_header);
                self.push_control(ControlKind::Loop, loop_header, end_block, None, result_type);
            }

            Operator::If { blockty } => {
                let result_type = match blockty {
                    wasmparser::BlockType::Empty => None,
                    wasmparser::BlockType::Type(vt) => Some(WasmType::from_wasmparser(*vt)),
                    wasmparser::BlockType::FuncType(_) => bail!("Multi-value blocks not supported"),
                };

                // Pop condition
                let condition = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for if condition"))?;

                let then_block = self.new_block();
                let else_block = self.new_block(); // Pre-allocate for else branch
                let end_block = self.new_block();

                // Branch based on condition
                self.terminate(IrTerminator::BranchIf {
                    condition,
                    if_true: then_block,
                    if_false: else_block,
                });

                // Start then block
                self.start_block(then_block);
                self.push_control(
                    ControlKind::If,
                    then_block,
                    end_block,
                    Some(else_block),
                    result_type,
                );

                // Note: else_block will be activated by Operator::Else or End
            }

            Operator::Else => {
                // Pop if frame
                let if_frame = self.pop_control().context("else without matching if")?;

                if if_frame.kind != ControlKind::If {
                    bail!("else without matching if");
                }

                // Then branch: assign result if needed
                let result_var = if_frame.result_var;
                if let Some(result_var) = result_var {
                    let stack_value = self.value_stack.pop().ok_or_else(|| {
                        anyhow::anyhow!("Stack underflow for then result in else")
                    })?;
                    self.emit(IrInstr::Assign {
                        dest: result_var,
                        src: stack_value,
                    });
                }

                // Current block (then branch) jumps to end
                self.terminate(IrTerminator::Jump {
                    target: if_frame.end_block,
                });

                // Use the pre-allocated else block
                let else_block = if_frame
                    .else_block
                    .expect("If frame should have else_block");
                self.start_block(else_block);

                // Push else frame (same end block, same result_var, no else_block needed)
                // We manually create the frame to preserve result_var
                self.control_stack.push(ControlFrame {
                    kind: ControlKind::Else,
                    start_block: else_block,
                    end_block: if_frame.end_block,
                    else_block: None,
                    result_type: if_frame.result_type,
                    result_var,
                });
            }

            Operator::Br { relative_depth } => {
                let target = self.get_branch_target(*relative_depth)?;
                self.terminate(IrTerminator::Jump { target });

                // Create unreachable continuation block
                let unreachable_block = self.new_block();
                self.start_block(unreachable_block);
            }

            Operator::BrIf { relative_depth } => {
                let condition = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for br_if"))?;

                let target = self.get_branch_target(*relative_depth)?;

                // Create continuation block (fallthrough)
                let continue_block = self.new_block();

                self.terminate(IrTerminator::BranchIf {
                    condition,
                    if_true: target,
                    if_false: continue_block,
                });

                // Continue building in continuation block
                self.start_block(continue_block);
            }

            Operator::BrTable { targets } => {
                let index = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for br_table"))?;

                // Convert targets to BlockIds
                let target_depths: Vec<u32> = targets.targets().collect::<Result<Vec<_>, _>>()?;
                let target_blocks: Vec<BlockId> = target_depths
                    .iter()
                    .map(|depth| self.get_branch_target(*depth))
                    .collect::<Result<Vec<_>>>()?;

                let default = self.get_branch_target(targets.default())?;

                self.terminate(IrTerminator::BranchTable {
                    index,
                    targets: target_blocks,
                    default,
                });

                // Create unreachable continuation block
                let unreachable_block = self.new_block();
                self.start_block(unreachable_block);
            }

            Operator::Call { function_index } => {
                let func_idx = *function_index as usize;
                let (param_count, callee_return_type) = *self
                    .func_signatures
                    .get(func_idx)
                    .ok_or_else(|| anyhow::anyhow!("Call to unknown function {}", func_idx))?;

                if self.value_stack.len() < param_count {
                    bail!("Stack underflow for call to func_{}", func_idx);
                }
                let mut args = Vec::new();
                for _ in 0..param_count {
                    args.push(self.value_stack.pop().ok_or_else(|| {
                        anyhow::anyhow!("stack underflow collecting call arguments")
                    })?);
                }
                args.reverse();

                let dest = callee_return_type.map(|_| self.new_var());

                // Check if this is a call to an imported function or a local function
                if func_idx < self.num_imported_functions {
                    // Call to imported function
                    let import_idx = func_idx;
                    let (module_name, func_name) = self
                        .func_imports
                        .get(import_idx)
                        .cloned()
                        .unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string())); // TODO: really unknown? Isnt that an error?

                    self.emit(IrInstr::CallImport {
                        dest,
                        import_idx,
                        module_name,
                        func_name,
                        args,
                    });
                } else {
                    // Call to local function - convert to local index
                    let local_func_idx = func_idx - self.num_imported_functions;
                    self.emit(IrInstr::Call {
                        dest,
                        func_idx: local_func_idx,
                        args,
                    });
                }

                if let Some(d) = dest {
                    self.value_stack.push(d);
                }
            }

            Operator::CallIndirect {
                type_index,
                table_index,
            } => {
                if *table_index != 0 {
                    bail!("Multi-table not supported (table_index={})", table_index);
                }
                let type_idx = *type_index as usize;
                let (param_count, callee_return_type) =
                    *self.type_signatures.get(type_idx).ok_or_else(|| {
                        anyhow::anyhow!("CallIndirect: unknown type index {}", type_idx)
                    })?;

                // Pop table element index (on top of stack)
                let table_idx_var = self.value_stack.pop().ok_or_else(|| {
                    anyhow::anyhow!("Stack underflow for call_indirect table index")
                })?;

                // Pop arguments
                if self.value_stack.len() < param_count {
                    bail!("Stack underflow for call_indirect type {}", type_idx);
                }
                let mut args = Vec::new();
                for _ in 0..param_count {
                    args.push(self.value_stack.pop().ok_or_else(|| {
                        anyhow::anyhow!("stack underflow collecting call_indirect arguments")
                    })?);
                }
                args.reverse();

                let dest = callee_return_type.map(|_| self.new_var());
                self.emit(IrInstr::CallIndirect {
                    dest,
                    type_idx,
                    table_idx: table_idx_var,
                    args,
                });

                if let Some(d) = dest {
                    self.value_stack.push(d);
                }
            }

            Operator::Unreachable => {
                self.terminate(IrTerminator::Unreachable);
                // Create unreachable continuation block (dead code follows)
                let unreachable_block = self.new_block();
                self.start_block(unreachable_block);
            }

            Operator::Select => {
                if self.value_stack.len() < 3 {
                    bail!("Stack underflow for select (need 3 values)");
                }
                let condition = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("stack underflow in Select (condition)"))?;
                let val2 = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("stack underflow in Select (val2)"))?;
                let val1 = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("stack underflow in Select (val1)"))?;
                let dest = self.new_var();
                self.emit(IrInstr::Select {
                    dest,
                    val1,
                    val2,
                    condition,
                });
                self.value_stack.push(dest);
            }

            _ => bail!("Unsupported operator: {:?}", op),
        }

        Ok(())
    }

    /// Emit a binary operation.
    fn emit_binop(&mut self, op: BinOp) -> Result<()> {
        if self.value_stack.len() < 2 {
            bail!("Stack underflow for binary operation {:?}", op);
        }

        let rhs = self
            .value_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack underflow in binop (rhs)"))?;
        let lhs = self
            .value_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack underflow in binop (lhs)"))?;
        let dest = self.new_var();

        self.emit(IrInstr::BinOp { dest, op, lhs, rhs });
        self.value_stack.push(dest);

        Ok(())
    }

    /// Emit a unary operation.
    fn emit_unop(&mut self, op: UnOp) -> Result<()> {
        if self.value_stack.is_empty() {
            bail!("Stack underflow for unary operation {:?}", op);
        }

        let operand = self
            .value_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack underflow in unop (operand)"))?;
        let dest = self.new_var();

        self.emit(IrInstr::UnOp { dest, op, operand });
        self.value_stack.push(dest);

        Ok(())
    }

    /// Emit a full-width memory load instruction.
    /// Stack: [addr] -> [value]
    fn emit_load(&mut self, ty: WasmType, offset: u64) -> Result<()> {
        self.emit_load_ext(ty, offset, MemoryAccessWidth::Full, None)
    }

    /// Emit a sub-width memory load instruction with extension.
    /// Stack: [addr] -> [value]
    fn emit_load_ext(
        &mut self,
        ty: WasmType,
        offset: u64,
        width: MemoryAccessWidth,
        sign: Option<SignExtension>,
    ) -> Result<()> {
        if self.value_stack.is_empty() {
            bail!("Stack underflow for load operation");
        }

        let addr = self
            .value_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack underflow in load (addr)"))?;
        let dest = self.new_var();

        self.emit(IrInstr::Load {
            dest,
            ty,
            addr,
            offset: offset as u32,
            width,
            sign,
        });

        self.value_stack.push(dest);
        Ok(())
    }

    /// Emit a full-width memory store instruction.
    /// Stack: [addr, value] -> []
    fn emit_store(&mut self, ty: WasmType, offset: u64) -> Result<()> {
        self.emit_store_narrow(ty, offset, MemoryAccessWidth::Full)
    }

    /// Emit a sub-width memory store instruction.
    /// Stack: [addr, value] -> []
    fn emit_store_narrow(
        &mut self,
        ty: WasmType,
        offset: u64,
        width: MemoryAccessWidth,
    ) -> Result<()> {
        if self.value_stack.len() < 2 {
            bail!("Stack underflow for store operation");
        }

        let value = self
            .value_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack underflow in store (value)"))?;
        let addr = self
            .value_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack underflow in store (addr)"))?;

        self.emit(IrInstr::Store {
            ty,
            addr,
            value,
            offset: offset as u32,
            width,
        });

        // Store has no result
        Ok(())
    }

    /// Push a control frame onto the control stack.
    fn push_control(
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
    fn pop_control(&mut self) -> Result<ControlFrame> {
        self.control_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Control stack underflow"))
    }

    /// Get branch target for relative depth N.
    ///
    /// Depth 0 = innermost frame, depth 1 = next outer, etc.
    /// For loops: branch to start_block
    /// For blocks/if: branch to end_block
    fn get_branch_target(&self, depth: u32) -> Result<BlockId> {
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
    fn start_block(&mut self, block_id: BlockId) {
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

/// Build complete module metadata from a parsed WebAssembly module.
///
/// This is the main entry point for IR construction, coordinating all
/// the intermediate steps needed to produce a fully-formed ModuleInfo.
pub fn build_module_info(parsed: &ParsedModule, options: &TranspileOptions) -> Result<ModuleInfo> {
    // Analyze module structure (memory, table, types)
    let mem_info = extract_memory_info(parsed, options)?;
    let table_info = extract_table_info(parsed);
    let (canonical_type, type_sigs) = build_type_mappings(parsed);

    // Analyze imports
    let imported_globals = build_imported_globals(parsed);
    let num_imported_functions = parsed.num_imported_functions;

    // Translate WebAssembly to intermediate representation
    let ir_functions = build_ir_functions(parsed, &type_sigs, num_imported_functions)?;

    // Assemble module metadata for code generation
    assemble_module_metadata(
        parsed,
        &mem_info,
        &table_info,
        &canonical_type,
        ir_functions,
        num_imported_functions as usize,
        &imported_globals,
    )
}

/// Memory information extracted from the module.
struct MemoryInfo {
    has_memory: bool,
    has_memory_import: bool,
    max_pages: usize,
    initial_pages: usize,
}

/// Table information extracted from the module.
struct TableInfo {
    initial: usize,
    max: usize,
}

/// Extracts memory information from a parsed WASM module.
fn extract_memory_info(parsed: &ParsedModule, options: &TranspileOptions) -> Result<MemoryInfo> {
    let has_memory = parsed.memory.is_some();
    let has_memory_import = parsed
        .imports
        .iter()
        .any(|imp| matches!(imp.kind, ImportKind::Memory { .. }));
    let max_pages = if let Some(ref mem) = parsed.memory {
        mem.maximum_pages
            .map(|p| p as usize)
            .unwrap_or(options.max_pages)
    } else {
        options.max_pages
    };
    let initial_pages = parsed
        .memory
        .as_ref()
        .map(|m| m.initial_pages as usize)
        .unwrap_or(0);

    Ok(MemoryInfo {
        has_memory,
        has_memory_import,
        max_pages,
        initial_pages,
    })
}

/// Extracts table information from a parsed WASM module.
fn extract_table_info(parsed: &ParsedModule) -> TableInfo {
    if let Some(ref tbl) = parsed.table {
        TableInfo {
            initial: tbl.initial_size as usize,
            max: (tbl.max_size.unwrap_or(tbl.initial_size) as usize),
        }
    } else {
        TableInfo { initial: 0, max: 0 }
    }
}

/// Builds canonical type index mapping and type signatures.
///
/// Canonical mapping ensures that call_indirect type checks follow the Wasm spec:
/// two different type indices with identical (params, results) must match.
/// We map each type_idx to the smallest index with the same structural signature.
fn build_type_mappings(parsed: &ParsedModule) -> (Vec<usize>, Vec<(usize, Option<WasmType>)>) {
    let canonical_type: Vec<usize> = {
        let mut mapping = Vec::with_capacity(parsed.types.len());
        for (i, ty) in parsed.types.iter().enumerate() {
            let canon = parsed.types[..i]
                .iter()
                .position(|earlier| {
                    earlier.params() == ty.params() && earlier.results() == ty.results()
                })
                .map(|pos| mapping[pos])
                .unwrap_or(i);
            mapping.push(canon);
        }
        mapping
    };

    let type_sigs: Vec<(usize, Option<WasmType>)> = parsed
        .types
        .iter()
        .map(|ty| {
            let param_count = ty.params().len();
            let ret = ty
                .results()
                .first()
                .map(|vt| WasmType::from_wasmparser(*vt));
            (param_count, ret)
        })
        .collect();

    (canonical_type, type_sigs)
}

/// Extracts imported globals from a parsed WASM module.
fn build_imported_globals(parsed: &ParsedModule) -> Vec<ImportedGlobalDef> {
    parsed
        .imports
        .iter()
        .filter_map(|imp| {
            if let ImportKind::Global { val_type, mutable } = &imp.kind {
                Some(ImportedGlobalDef {
                    module_name: imp.module_name.clone(),
                    name: imp.name.clone(),
                    wasm_type: WasmType::from_wasmparser(*val_type),
                    mutable: *mutable,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Translates all functions in the module to intermediate representation.
fn build_ir_functions(
    parsed: &ParsedModule,
    type_sigs: &[(usize, Option<WasmType>)],
    num_imported_functions: u32,
) -> Result<Vec<IrFunction>> {
    let mut ir_builder = IrBuilder::new();
    let mut ir_functions = Vec::new();

    // Build function signature list (imported + local)
    let func_sigs = build_function_signatures(parsed);

    // Build function import list for IR builder
    let func_imports: Vec<(String, String)> = parsed
        .imports
        .iter()
        .filter_map(|imp| match &imp.kind {
            ImportKind::Function(_) => Some((imp.module_name.clone(), imp.name.clone())),
            _ => None,
        })
        .collect();

    let module_ctx = ModuleContext {
        func_signatures: func_sigs,
        type_signatures: type_sigs.to_vec(),
        num_imported_functions: num_imported_functions as usize,
        func_imports,
    };

    for (func_idx, func) in parsed.functions.iter().enumerate() {
        let func_type = &parsed.types[func.type_idx as usize];

        let params: Vec<_> = func_type
            .params()
            .iter()
            .map(|vt| (*vt, WasmType::from_wasmparser(*vt)))
            .collect();

        let return_type = func_type
            .results()
            .first()
            .map(|vt| WasmType::from_wasmparser(*vt));

        let operators = parse_function_operators(&func.body)?;

        let ir_func = ir_builder
            .translate_function(&params, &func.locals, return_type, &operators, &module_ctx)
            .with_context(|| format!("failed to build IR for function {}", func_idx))?;

        ir_functions.push(ir_func);
    }

    Ok(ir_functions)
}

/// Builds the function signature list (imported functions followed by local functions).
fn build_function_signatures(parsed: &ParsedModule) -> Vec<(usize, Option<WasmType>)> {
    let mut func_sigs: Vec<(usize, Option<WasmType>)> = Vec::new();

    // Imported function signatures
    for import in &parsed.imports {
        if let ImportKind::Function(type_idx) = &import.kind {
            let func_type = &parsed.types[*type_idx as usize];
            let param_count = func_type.params().len();
            let ret = func_type
                .results()
                .first()
                .map(|vt| WasmType::from_wasmparser(*vt));
            func_sigs.push((param_count, ret));
        }
    }

    // Local function signatures
    for func in &parsed.functions {
        let func_type = &parsed.types[func.type_idx as usize];
        let param_count = func_type.params().len();
        let ret = func_type
            .results()
            .first()
            .map(|vt| WasmType::from_wasmparser(*vt));
        func_sigs.push((param_count, ret));
    }

    func_sigs
}

/// Parses Wasm operators from a function body.
fn parse_function_operators(body: &[u8]) -> Result<Vec<Operator<'_>>> {
    let mut operators = Vec::new();
    let mut binary_reader = wasmparser::BinaryReader::new(body, 0);

    while !binary_reader.eof() {
        let op = binary_reader
            .read_operator()
            .context("failed to read operator")?;
        operators.push(op);
    }

    Ok(operators)
}

/// Assembles module metadata for code generation.
#[allow(clippy::too_many_arguments)]
fn assemble_module_metadata(
    parsed: &ParsedModule,
    mem_info: &MemoryInfo,
    table_info: &TableInfo,
    canonical_type: &[usize],
    ir_functions: Vec<IrFunction>,
    num_imported_functions: usize,
    imported_globals: &[ImportedGlobalDef],
) -> Result<ModuleInfo> {
    let globals = build_globals(parsed);
    let data_segments = build_data_segments(parsed);
    let element_segments = build_element_segments(parsed);
    let func_exports = build_function_exports(parsed, num_imported_functions);
    let func_signatures =
        build_function_type_signatures(parsed, canonical_type, &ir_functions, imported_globals);
    let type_signatures = build_call_indirect_signatures(parsed);
    let func_imports = build_function_imports(parsed);

    Ok(ModuleInfo {
        has_memory: mem_info.has_memory,
        has_memory_import: mem_info.has_memory_import,
        max_pages: mem_info.max_pages,
        initial_pages: mem_info.initial_pages,
        table_initial: table_info.initial,
        table_max: table_info.max,
        element_segments,
        globals,
        data_segments,
        func_exports,
        func_signatures,
        type_signatures,
        canonical_type: canonical_type.to_vec(),
        func_imports,
        imported_globals: imported_globals.to_vec(),
        ir_functions,
    })
}

/// Builds global variable definitions.
fn build_globals(parsed: &ParsedModule) -> Vec<GlobalDef> {
    parsed
        .globals
        .iter()
        .map(|g| {
            let wasm_type = WasmType::from_wasmparser(g.val_type);
            let init_value = match g.init_value {
                crate::parser::InitValue::I32(v) => GlobalInit::I32(v),
                crate::parser::InitValue::I64(v) => GlobalInit::I64(v),
                crate::parser::InitValue::F32(v) => GlobalInit::F32(v),
                crate::parser::InitValue::F64(v) => GlobalInit::F64(v),
            };
            GlobalDef {
                wasm_type,
                mutable: g.mutable,
                init_value,
            }
        })
        .collect()
}

/// Builds data segment definitions.
fn build_data_segments(parsed: &ParsedModule) -> Vec<DataSegmentDef> {
    parsed
        .data_segments
        .iter()
        .map(|ds| DataSegmentDef {
            offset: ds.offset,
            data: ds.data.clone(),
        })
        .collect()
}

/// Builds element segment (table initialization) definitions.
fn build_element_segments(parsed: &ParsedModule) -> Vec<ElementSegmentDef> {
    parsed
        .element_segments
        .iter()
        .map(|es| ElementSegmentDef {
            offset: es.offset as usize,
            func_indices: es.func_indices.iter().map(|idx| *idx as usize).collect(),
        })
        .collect()
}

/// Builds exported function definitions.
///
/// Export indices use global numbering (imports + locals). We filter to local
/// functions and offset to local function index space for codegen (func_0, func_1, ...).
fn build_function_exports(parsed: &ParsedModule, num_imported_functions: usize) -> Vec<FuncExport> {
    parsed
        .exports
        .iter()
        .filter(|e| e.kind == ExportKind::Func && (e.index as usize) >= num_imported_functions)
        .map(|e| FuncExport {
            name: e.name.clone(),
            func_index: (e.index as usize) - num_imported_functions,
        })
        .collect()
}

/// Builds function type signatures for exported functions.
///
/// This is indexed by local function index (0-based, excluding imports) and includes
/// the `needs_host` flag to indicate if a function requires a host parameter.
fn build_function_type_signatures(
    parsed: &ParsedModule,
    canonical_type: &[usize],
    ir_functions: &[IrFunction],
    imported_globals: &[ImportedGlobalDef],
) -> Vec<FuncSignature> {
    let num_imported_globals = imported_globals.len();
    parsed
        .functions
        .iter()
        .enumerate()
        .map(|(func_idx, func)| {
            let func_type = &parsed.types[func.type_idx as usize];
            let params = func_type
                .params()
                .iter()
                .map(|vt| WasmType::from_wasmparser(*vt))
                .collect();
            let return_type = func_type
                .results()
                .first()
                .map(|vt| WasmType::from_wasmparser(*vt));

            let needs_host = ir_functions
                .get(func_idx)
                .map(|ir_func| function_calls_imports(ir_func, num_imported_globals))
                .unwrap_or(false);

            FuncSignature {
                params,
                return_type,
                type_idx: canonical_type[func.type_idx as usize],
                needs_host,
            }
        })
        .collect()
}

/// Determines if a function calls imports or accesses imported globals.
fn function_calls_imports(ir_func: &IrFunction, num_imported_globals: usize) -> bool {
    ir_func.blocks.iter().any(|block| {
        block.instructions.iter().any(|instr| {
            matches!(instr, IrInstr::CallImport { .. })
                || (num_imported_globals > 0
                    && matches!(
                        instr,
                        IrInstr::GlobalGet { index, .. }
                            | IrInstr::GlobalSet { index, .. }
                            if *index < num_imported_globals
                    ))
        })
    })
}

/// Builds type signatures for call_indirect type checking.
fn build_call_indirect_signatures(parsed: &ParsedModule) -> Vec<FuncSignature> {
    parsed
        .types
        .iter()
        .map(|ty| {
            let params = ty
                .params()
                .iter()
                .map(|vt| WasmType::from_wasmparser(*vt))
                .collect();
            let return_type = ty
                .results()
                .first()
                .map(|vt| WasmType::from_wasmparser(*vt));
            FuncSignature {
                params,
                return_type,
                type_idx: 0,
                needs_host: false,
            }
        })
        .collect()
}

/// Builds function import trait definitions.
fn build_function_imports(parsed: &ParsedModule) -> Vec<FuncImport> {
    parsed
        .imports
        .iter()
        .filter_map(|imp| match &imp.kind {
            ImportKind::Function(type_idx) => {
                let func_type = &parsed.types[*type_idx as usize];
                let params = func_type
                    .params()
                    .iter()
                    .map(|vt| WasmType::from_wasmparser(*vt))
                    .collect();
                let return_type = func_type
                    .results()
                    .first()
                    .map(|vt| WasmType::from_wasmparser(*vt));
                Some(FuncImport {
                    module_name: imp.module_name.clone(),
                    func_name: imp.name.clone(),
                    params,
                    return_type,
                })
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the invariant: entry_block is always BlockId(0)
    #[test]
    fn entry_block_is_always_block_zero() {
        let mut builder = IrBuilder::new();

        // Simple function: fn add(a: i32, b: i32) -> i32 { a + b }
        let params = vec![
            (wasmparser::ValType::I32, WasmType::I32),
            (wasmparser::ValType::I32, WasmType::I32),
        ];
        let operators = vec![
            wasmparser::Operator::LocalGet { local_index: 0 },
            wasmparser::Operator::LocalGet { local_index: 1 },
            wasmparser::Operator::I32Add,
            wasmparser::Operator::End,
        ];

        let module_ctx = ModuleContext {
            func_signatures: vec![],
            type_signatures: vec![],
            num_imported_functions: 0,
            func_imports: vec![],
        };

        let ir_func = builder
            .translate_function(&params, &[], Some(WasmType::I32), &operators, &module_ctx)
            .expect("translation should succeed");

        // INVARIANT CHECK: entry_block must be BlockId(0)
        assert_eq!(
            ir_func.entry_block,
            BlockId(0),
            "entry_block must always be BlockId(0)"
        );

        // Additional sanity checks
        assert!(
            !ir_func.blocks.is_empty(),
            "function must have at least one block"
        );
        assert_eq!(
            ir_func.blocks[0].id,
            BlockId(0),
            "first block in the blocks vector must be BlockId(0)"
        );
    }

    /// Test that entry_block == BlockId(0) even for void functions
    #[test]
    fn entry_block_is_zero_for_void_function() {
        let mut builder = IrBuilder::new();

        // Void function: fn noop() { }
        let operators = vec![wasmparser::Operator::Nop, wasmparser::Operator::End];

        let module_ctx = ModuleContext {
            func_signatures: vec![],
            type_signatures: vec![],
            num_imported_functions: 0,
            func_imports: vec![],
        };

        let ir_func = builder
            .translate_function(&[], &[], None, &operators, &module_ctx)
            .expect("translation should succeed");

        assert_eq!(
            ir_func.entry_block,
            BlockId(0),
            "entry_block must be BlockId(0) even for void functions"
        );
    }

    /// Test that entry_block == BlockId(0) for functions with locals
    #[test]
    fn entry_block_is_zero_with_locals() {
        let mut builder = IrBuilder::new();

        let params = vec![(wasmparser::ValType::I32, WasmType::I32)];
        let locals = vec![wasmparser::ValType::I32, wasmparser::ValType::I32];
        let operators = vec![
            wasmparser::Operator::I32Const { value: 42 },
            wasmparser::Operator::End,
        ];

        let module_ctx = ModuleContext {
            func_signatures: vec![],
            type_signatures: vec![],
            num_imported_functions: 0,
            func_imports: vec![],
        };

        let ir_func = builder
            .translate_function(
                &params,
                &locals,
                Some(WasmType::I32),
                &operators,
                &module_ctx,
            )
            .expect("translation should succeed");

        assert_eq!(
            ir_func.entry_block,
            BlockId(0),
            "entry_block must be BlockId(0) regardless of locals"
        );
    }

    /// Test that local variables are properly tracked and distinguished from parameters
    #[test]
    fn local_variables_separate_from_params() {
        let mut builder = IrBuilder::new();

        // Function: (param i32) (result i32)
        //   (local i32)
        //   local.get 0      ;; param
        //   local.set 1      ;; store to local (index 1)
        //   local.get 1      ;; get local back
        let params = vec![(wasmparser::ValType::I32, WasmType::I32)];
        let locals = vec![wasmparser::ValType::I32];
        let operators = vec![
            wasmparser::Operator::LocalGet { local_index: 0 },
            wasmparser::Operator::LocalSet { local_index: 1 },
            wasmparser::Operator::LocalGet { local_index: 1 },
            wasmparser::Operator::End,
        ];

        let module_ctx = ModuleContext {
            func_signatures: vec![],
            type_signatures: vec![],
            num_imported_functions: 0,
            func_imports: vec![],
        };

        let ir_func = builder
            .translate_function(
                &params,
                &locals,
                Some(WasmType::I32),
                &operators,
                &module_ctx,
            )
            .expect("translation should succeed");

        // Parameter 0 and local 1 must be different VarIds
        let param_var = ir_func.params[0].0;
        let local_var = ir_func.locals[0].0;
        assert_ne!(
            param_var, local_var,
            "param and local must have distinct VarIds"
        );

        // Verify local tracking
        assert_eq!(
            ir_func.locals.len(),
            1,
            "should have exactly 1 declared local"
        );
        assert_eq!(
            ir_func.locals[0].1,
            WasmType::I32,
            "local should have type i32"
        );

        // Verify params are still tracked
        assert_eq!(ir_func.params.len(), 1, "should have exactly 1 param");
        assert_eq!(
            ir_func.params[0].1,
            WasmType::I32,
            "param should have type i32"
        );
    }

    /// Test with multiple locals of different types
    #[test]
    fn multiple_locals_different_types() {
        let mut builder = IrBuilder::new();

        // Function: (param i32) (result i32)
        //   (local i32 i64 f32)
        //   local.get 0
        let params = vec![(wasmparser::ValType::I32, WasmType::I32)];
        let locals = vec![
            wasmparser::ValType::I32,
            wasmparser::ValType::I64,
            wasmparser::ValType::F32,
        ];
        let operators = vec![
            wasmparser::Operator::LocalGet { local_index: 0 },
            wasmparser::Operator::End,
        ];

        let module_ctx = ModuleContext {
            func_signatures: vec![],
            type_signatures: vec![],
            num_imported_functions: 0,
            func_imports: vec![],
        };

        let ir_func = builder
            .translate_function(
                &params,
                &locals,
                Some(WasmType::I32),
                &operators,
                &module_ctx,
            )
            .expect("translation should succeed");

        // Verify all locals are tracked with correct types
        assert_eq!(ir_func.locals.len(), 3);
        assert_eq!(ir_func.locals[0].1, WasmType::I32);
        assert_eq!(ir_func.locals[1].1, WasmType::I64);
        assert_eq!(ir_func.locals[2].1, WasmType::F32);

        // All locals should have different VarIds
        let var_ids: Vec<VarId> = ir_func.locals.iter().map(|(v, _)| *v).collect();
        assert_eq!(var_ids[0], var_ids[0]);
        assert_ne!(var_ids[0], var_ids[1]);
        assert_ne!(var_ids[1], var_ids[2]);
    }
}
