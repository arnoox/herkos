//! Operator translation - converts WebAssembly bytecode to SSA IR instructions.
//!
//! This module contains the `translate_operator` method and related emit helpers
//! that form the core of the Wasm-to-IR conversion logic.

use super::super::types::*;
use super::core::IrBuilder;
use anyhow::{bail, Context, Result};
use wasmparser::Operator;

impl IrBuilder {
    /// Translate a single Wasm operator to IR instructions.
    pub(super) fn translate_operator(&mut self, op: &Operator) -> Result<()> {
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
                let src = self
                    .local_vars
                    .get(*local_index as usize)
                    .copied()
                    .ok_or_else(|| {
                        anyhow::anyhow!("local.get: local index {} out of range", local_index)
                    })?;
                // Emit a copy rather than pushing the local's VarId directly.
                // If we push the local's VarId, a later local.tee/local.set that
                // overwrites the same local will corrupt any already-pushed reference
                // to it, because the backend emits sequential mutable assignments.
                // A fresh variable captures the value at this point in time.
                let dest = self.new_var();
                self.emit(IrInstr::Assign { dest, src });
                self.value_stack.push(dest);
            }

            Operator::LocalSet { local_index } => {
                let idx = *local_index as usize;
                // Pop value and assign to local
                let value = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for local.set"))?;

                if idx >= self.local_vars.len() {
                    bail!("local.set: local index {} out of range", local_index);
                }

                // Allocate a fresh dest to satisfy SSA single-definition rule.
                let dest = self.new_var();
                self.emit(IrInstr::Assign { dest, src: value });
                // Update the local mapping so subsequent reads see the new value.
                self.local_vars[idx] = dest;
            }

            Operator::LocalTee { local_index } => {
                let idx = *local_index as usize;
                // Like LocalSet but keeps value on stack
                let value = self
                    .value_stack
                    .last()
                    .copied()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for local.tee"))?;

                if idx >= self.local_vars.len() {
                    bail!("local.tee: local index {} out of range", local_index);
                }

                // Allocate a fresh dest to satisfy SSA single-definition rule.
                let dest = self.new_var();
                self.emit(IrInstr::Assign { dest, src: value });
                // Update the local mapping so subsequent reads see the new value.
                self.local_vars[idx] = dest;
                // Value stays on stack (already there via .last())
            }

            // Global variable access
            Operator::GlobalGet { global_index } => {
                let dest = self.new_var();
                self.emit(IrInstr::GlobalGet {
                    dest,
                    index: GlobalIdx::new(*global_index as usize),
                });
                self.value_stack.push(dest);
            }

            Operator::GlobalSet { global_index } => {
                let value = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for global.set"))?;
                self.emit(IrInstr::GlobalSet {
                    index: GlobalIdx::new(*global_index as usize),
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
                self.emit_return()?;
                // Start a dead-code continuation block so subsequent Wasm
                // instructions (e.g. End for enclosing blocks) don't corrupt
                // the already-terminated block's control flow.
                let dead_block = self.new_block();
                self.start_block(dead_block);
                self.dead_code = true;
            }

            // End (end of function or block)
            Operator::End => {
                if self.control_stack.len() <= 1 {
                    // End of function - treat as implicit return
                    self.emit_return()?;
                } else {
                    let frame = self.pop_control()?;

                    match frame.kind {
                        super::core::ControlKind::If => {
                            // === IF without ELSE ===
                            // The then-branch just finished. We create an implicit empty else
                            // block and insert phi nodes at the join point.

                            let else_block =
                                frame.else_block.expect("If frame must have else_block");

                            // Collect predecessors of end_block for phi computation.
                            // 1. Fall-through from then-body (if reachable)
                            // 2. Any `br` inside then-body that targeted end_block
                            // 3. Implicit else block (with pre-if locals = locals_at_entry)
                            let mut preds: Vec<(BlockId, Vec<VarId>)> = Vec::new();
                            if !self.dead_code {
                                preds.push((self.current_block, self.local_vars.clone()));
                            }
                            preds.extend(frame.branch_incoming.iter().cloned());

                            // Assign result if needed (then-branch fall-through)
                            if let Some(result_var) = frame.result_var {
                                if let Some(stack_value) = self.value_stack.pop() {
                                    self.emit(IrInstr::Assign {
                                        dest: result_var,
                                        src: stack_value,
                                    });
                                }
                            }

                            // Terminate then-branch (if reachable)
                            if !self.dead_code {
                                self.terminate(IrTerminator::Jump {
                                    target: frame.end_block,
                                });
                            }

                            // Create implicit else block (empty; just jumps to end).
                            // The else_block is always reachable (false-branch of BranchIf),
                            // but we don't need to clear dead_code for it — it has no user
                            // instructions; we just terminate it directly.
                            self.start_block(else_block);
                            self.terminate(IrTerminator::Jump {
                                target: frame.end_block,
                            });
                            // The implicit else carries the pre-if local state.
                            preds.push((else_block, frame.locals_at_entry.clone()));

                            // Restore local_vars to pre-if state before computing phis
                            // (preds already captured the necessary snapshots above).
                            self.local_vars = frame.locals_at_entry.clone();

                            // Start the join block (always reachable — else_block always jumps here)
                            self.start_real_block(frame.end_block);

                            // Insert phi nodes for locals with differing predecessor values.
                            // If no live predecessors, mark as dead code.
                            if preds.is_empty() {
                                self.dead_code = true;
                            } else {
                                self.insert_phis_at_join(frame.end_block, &preds);
                            }
                        }

                        super::core::ControlKind::Else => {
                            // === IF-ELSE END ===
                            // The else-branch just finished. Insert phis at the join point
                            // using then-pred and else-pred info saved during Operator::Else.

                            // Collect predecessors of end_block:
                            // 1. then-branch fall-through (saved as then_pred_info in Else frame)
                            // 2. else-branch fall-through (current block, if reachable)
                            // 3. Any `br` from either branch targeting end_block (branch_incoming)
                            let mut preds: Vec<(BlockId, Vec<VarId>)> = Vec::new();
                            if let Some((then_block, then_locals)) = frame.then_pred_info.clone() {
                                preds.push((then_block, then_locals));
                            }
                            if !self.dead_code {
                                preds.push((self.current_block, self.local_vars.clone()));
                            }
                            preds.extend(frame.branch_incoming.iter().cloned());

                            // Assign result if needed (else-branch fall-through)
                            if let Some(result_var) = frame.result_var {
                                if let Some(stack_value) = self.value_stack.pop() {
                                    self.emit(IrInstr::Assign {
                                        dest: result_var,
                                        src: stack_value,
                                    });
                                }
                            }

                            // Terminate else-branch (if reachable)
                            if !self.dead_code {
                                self.terminate(IrTerminator::Jump {
                                    target: frame.end_block,
                                });
                            }

                            // Start join block
                            self.start_real_block(frame.end_block);

                            if preds.is_empty() {
                                self.dead_code = true;
                            } else {
                                self.insert_phis_at_join(frame.end_block, &preds);
                            }
                        }

                        super::core::ControlKind::Loop => {
                            // === LOOP END ===
                            // Emit the phi instructions into the loop header (start_block).
                            // Then fall through to end_block.

                            // Collect fall-through predecessor BEFORE switching blocks.
                            let mut preds: Vec<(BlockId, Vec<VarId>)> =
                                frame.branch_incoming.clone();
                            if !self.dead_code {
                                preds.push((self.current_block, self.local_vars.clone()));
                            }

                            // Assign result if needed (loop fall-through)
                            if let Some(result_var) = frame.result_var {
                                if let Some(stack_value) = self.value_stack.pop() {
                                    self.emit(IrInstr::Assign {
                                        dest: result_var,
                                        src: stack_value,
                                    });
                                }
                            }

                            // Terminate loop body fall-through (if reachable)
                            if !self.dead_code {
                                self.terminate(IrTerminator::Jump {
                                    target: frame.end_block,
                                });
                            }

                            // Emit Phi instructions into the loop header block.
                            // This consumes the relevant phi_patches.
                            self.emit_loop_phis(&frame);

                            // Start the loop's exit block
                            self.start_real_block(frame.end_block);

                            // Insert phis at end_block for locals with differing exit values
                            if preds.is_empty() {
                                self.dead_code = true;
                            } else {
                                self.insert_phis_at_join(frame.end_block, &preds);
                            }
                        }

                        super::core::ControlKind::Block => {
                            // === BLOCK END ===
                            // Collect fall-through predecessor BEFORE switching blocks.
                            let mut preds: Vec<(BlockId, Vec<VarId>)> =
                                frame.branch_incoming.clone();
                            if !self.dead_code {
                                preds.push((self.current_block, self.local_vars.clone()));
                            }

                            // Assign result if needed (block fall-through)
                            if let Some(result_var) = frame.result_var {
                                if let Some(stack_value) = self.value_stack.pop() {
                                    // Normal case: block fell through with a result value
                                    self.emit(IrInstr::Assign {
                                        dest: result_var,
                                        src: stack_value,
                                    });
                                }
                                // Empty stack: block ended with br/return (dead code after)
                            }

                            // Terminate block fall-through (if reachable)
                            if !self.dead_code {
                                self.terminate(IrTerminator::Jump {
                                    target: frame.end_block,
                                });
                            }

                            // Start join block
                            self.start_real_block(frame.end_block);

                            if preds.is_empty() {
                                self.dead_code = true;
                            } else {
                                self.insert_phis_at_join(frame.end_block, &preds);
                            }
                        }
                    }

                    // If the control structure produced a result, push it onto the value stack.
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

            // Control flow
            Operator::Block { blockty } => {
                // === Parse the block's result type ===
                // A block can optionally produce a value (e.g., "block i32 ... end").
                // If no result type, the block just groups instructions without producing a value.
                let result_type = match blockty {
                    wasmparser::BlockType::Empty => None,
                    wasmparser::BlockType::Type(vt) => Some(WasmType::from_wasmparser(*vt)),
                    wasmparser::BlockType::FuncType(_) => bail!("Multi-value blocks not supported"),
                };

                // === Create the exit block ===
                // When a "br" (branch) instruction inside this block executes,
                // it jumps to end_block (forward jump, not backward).
                // This is where we merge back together after the block.
                let end_block = self.new_block();

                // === Reuse the current block as the loop start target ===
                // This is KEY DIFFERENCE from Loop:
                //   Block: start_block = current block (we stay in it, no jump needed)
                //   Loop:  start_block = newly created block (we jump to it)
                //
                // Why? Because blocks execute sequentially with no backward jumps.
                // We don't need to set up a loop header; we just keep building
                // instructions in the current block.
                let start_block = self.current_block;

                // === STEP 1: Push control frame ===
                // Record this block's control structure:
                //   - kind=Block: marks this as a block (for br/br_if dispatch)
                //   - start_block=current_block: where we are now (no backward jumps)
                //   - end_block=end_block: where br jumps (exit the block)
                //   - else_block=None: blocks don't have else branches (that's just If)
                //   - result_type: the block's result type (if any)
                self.push_control(
                    super::core::ControlKind::Block,
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

                // === KEY DIFFERENCE: Loop vs Block ===
                // Block:
                //   - "br" branches FORWARD to end_block (exit)
                //   - start_block = current block, end_block = exit target
                //
                // Loop:
                //   - "br" branches BACKWARD to loop_header (continue loop)
                //   - start_block = loop_header (for backward branches), end_block = exit
                //
                // The loop_header is the target of backward branches (continue).
                // The end_block is the target of forward branches (break out of loop).

                let loop_header = self.new_block();
                let end_block = self.new_block();

                // === STEP 1: Jump from current block to loop header ===
                // The loop doesn't start executing immediately; we first jump to the
                // loop header block. This is where control jumps back to during iteration.
                self.terminate(IrTerminator::Jump {
                    target: loop_header,
                });

                // === STEP 2: Push control frame BEFORE switching blocks ===
                // push_control captures self.current_block as pre_loop_block (the block
                // that jumps into the loop). It must be called while current_block still
                // points to the pre-loop block, before start_block changes it.
                //
                // push_control also pre-allocates phi vars for all locals and updates
                // self.local_vars to point to them, so all code inside the loop body
                // reads/writes through the phi vars from the start.
                self.push_control(
                    super::core::ControlKind::Loop,
                    loop_header,
                    end_block,
                    None,
                    result_type,
                );

                // === STEP 3: Begin codegen in the loop header block ===
                // This block is the entry point to the loop and the target of backward
                // branches (via "br" inside the loop body).
                self.start_block(loop_header);
            }

            Operator::If { blockty } => {
                // === Parse the if's result type ===
                // An if can optionally produce a value (e.g., "if i32 ... else ... end").
                // Both then and else branches must produce the same type.
                let result_type = match blockty {
                    wasmparser::BlockType::Empty => None,
                    wasmparser::BlockType::Type(vt) => Some(WasmType::from_wasmparser(*vt)),
                    wasmparser::BlockType::FuncType(_) => bail!("Multi-value blocks not supported"),
                };

                // === STEP 1: Pop the condition from the value stack ===
                // The condition (i32, treated as bool: 0 = false, nonzero = true)
                // is on top of the stack. Pop it and use it to branch.
                let condition = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for if condition"))?;

                // === STEP 2: Pre-allocate all three blocks ===
                // We create all blocks upfront so we can reference them in the BranchIf.
                // The then_block and else_block are allocated here, but activation
                // (start_block) happens later when we actually begin generating code for them.
                let then_block = self.new_block();
                let else_block = self.new_block(); // Pre-allocate for else branch
                let end_block = self.new_block(); // Where both branches converge

                // === STEP 3: Emit the conditional branch instruction ===
                // Terminate the current block with a BranchIf that splits control flow:
                //   - If condition is true (nonzero) → jump to then_block
                //   - If condition is false (zero) → jump to else_block
                //
                // This is the branch instruction that will appear in the IR.
                self.terminate(IrTerminator::BranchIf {
                    condition,
                    if_true: then_block,
                    if_false: else_block,
                });

                // === STEP 4: Start building the THEN branch ===
                // Activate the then_block. This is always reachable (the BranchIf true path),
                // so use start_real_block to clear dead_code.
                self.start_real_block(then_block);

                // === STEP 5: Push If control frame ===
                // Record this if's control structure for later resolution:
                //   - kind=If: marks this as an if (different dispatch for br/br_if)
                //   - start_block=then_block: where br 0 jumps (inside if context)
                //   - end_block=end_block: where br 1 jumps (out of if/else)
                //   - else_block=Some(else_block): deferred; we'll activate it when we see Else or End
                //   - result_type: the if's result type (if any)
                self.push_control(
                    super::core::ControlKind::If,
                    then_block,
                    end_block,
                    Some(else_block),
                    result_type,
                );

                // === NOTE: Deferred activation ===
                // The else_block is NOT activated yet. It's stored in the control frame.
                // It will be activated when:
                //   1. Operator::Else arrives → finalize then-branch, switch to else-branch
                //   2. Operator::End arrives → if there was no Else, create empty else-branch
            }

            Operator::Else => {
                // Pop if frame
                let if_frame = self.pop_control().context("else without matching if")?;

                if if_frame.kind != super::core::ControlKind::If {
                    bail!("else without matching if");
                }

                // Save then-branch fall-through info (predecessor for end_block phis).
                // Only recorded if the then-branch body is reachable (not dead code).
                let then_pred_info = if !self.dead_code {
                    Some((self.current_block, self.local_vars.clone()))
                } else {
                    None
                };

                // Then branch: assign result if needed
                let result_var = if_frame.result_var;
                if let Some(result_var) = result_var {
                    if let Some(stack_value) = self.value_stack.pop() {
                        self.emit(IrInstr::Assign {
                            dest: result_var,
                            src: stack_value,
                        });
                    }
                }

                // Current block (then branch) jumps to end (if reachable)
                if !self.dead_code {
                    self.terminate(IrTerminator::Jump {
                        target: if_frame.end_block,
                    });
                }

                // Restore local_vars to pre-if state so the else branch starts
                // with the same local variable bindings as the then branch.
                self.local_vars = if_frame.locals_at_entry.clone();

                // Use the pre-allocated else block (always reachable: false branch of BranchIf)
                let else_block = if_frame
                    .else_block
                    .expect("If frame should have else_block");
                self.start_real_block(else_block);

                // Push Else frame, preserving result_var and transferring branch_incoming
                // (any `br 0` inside the then-branch already recorded in if_frame).
                self.control_stack.push(super::core::ControlFrame {
                    kind: super::core::ControlKind::Else,
                    start_block: else_block,
                    end_block: if_frame.end_block,
                    else_block: None,
                    result_type: if_frame.result_type,
                    result_var,
                    locals_at_entry: if_frame.locals_at_entry,
                    branch_incoming: if_frame.branch_incoming, // transfer then-body br's
                    then_pred_info,
                    loop_phi_vars: Vec::new(),
                    pre_loop_block: None,
                });
            }

            Operator::Br { relative_depth } => {
                let depth = *relative_depth as usize;
                let frame_idx =
                    self.control_stack
                        .len()
                        .checked_sub(depth + 1)
                        .ok_or_else(|| {
                            anyhow::anyhow!("br: depth {} exceeds control stack", relative_depth)
                        })?;

                let (target, is_loop) = {
                    let frame = &self.control_stack[frame_idx];
                    match frame.kind {
                        super::core::ControlKind::Loop => (frame.start_block, true),
                        _ => (frame.end_block, false),
                    }
                };

                // Record this branch as a phi predecessor (before terminate)
                if is_loop {
                    self.record_loop_back_branch(frame_idx);
                } else {
                    self.record_forward_branch(frame_idx);
                }

                self.terminate(IrTerminator::Jump { target });

                // Everything after an unconditional branch is unreachable
                let dead_block = self.new_block();
                self.start_block(dead_block);
                self.dead_code = true;
            }

            Operator::BrIf { relative_depth } => {
                let condition = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for br_if"))?;

                let depth = *relative_depth as usize;
                let frame_idx =
                    self.control_stack
                        .len()
                        .checked_sub(depth + 1)
                        .ok_or_else(|| {
                            anyhow::anyhow!("br_if: depth {} exceeds control stack", relative_depth)
                        })?;

                let (target, is_loop) = {
                    let frame = &self.control_stack[frame_idx];
                    match frame.kind {
                        super::core::ControlKind::Loop => (frame.start_block, true),
                        _ => (frame.end_block, false),
                    }
                };

                // Record the taken branch as a phi predecessor
                if is_loop {
                    self.record_loop_back_branch(frame_idx);
                } else {
                    self.record_forward_branch(frame_idx);
                }

                // Create continuation block (fallthrough — always reachable)
                let continue_block = self.new_block();

                self.terminate(IrTerminator::BranchIf {
                    condition,
                    if_true: target,
                    if_false: continue_block,
                });

                // Fallthrough is always reachable; clear dead_code
                self.start_real_block(continue_block);
            }

            Operator::BrTable { targets } => {
                let index = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for br_table"))?;

                // Collect all depths (table entries + default), deduplicate by frame_idx
                // to avoid recording the same predecessor block twice for the same target.
                let target_depths: Vec<u32> = targets.targets().collect::<Result<Vec<_>, _>>()?;
                let default_depth = targets.default();

                let stack_len = self.control_stack.len();
                let mut recorded: std::collections::HashSet<usize> =
                    std::collections::HashSet::new();
                for depth in target_depths
                    .iter()
                    .copied()
                    .chain(std::iter::once(default_depth))
                {
                    let depth = depth as usize;
                    let frame_idx = stack_len.saturating_sub(depth + 1);
                    if recorded.insert(frame_idx) {
                        let is_loop =
                            self.control_stack[frame_idx].kind == super::core::ControlKind::Loop;
                        if is_loop {
                            self.record_loop_back_branch(frame_idx);
                        } else {
                            self.record_forward_branch(frame_idx);
                        }
                    }
                }

                let target_blocks: Vec<BlockId> = target_depths
                    .iter()
                    .map(|depth| self.get_branch_target(*depth))
                    .collect::<Result<Vec<_>>>()?;

                let default = self.get_branch_target(default_depth)?;

                self.terminate(IrTerminator::BranchTable {
                    index,
                    targets: target_blocks,
                    default,
                });

                // Everything after br_table is unreachable
                let dead_block = self.new_block();
                self.start_block(dead_block);
                self.dead_code = true;
            }

            Operator::Call { function_index } => {
                let func_idx = *function_index as usize;
                let (param_count, callee_return_type) = *self
                    .func_signatures
                    .get(func_idx)
                    .ok_or_else(|| anyhow::anyhow!("Call to unknown function {}", func_idx))?;

                let args =
                    self.pop_call_args(param_count, &format!("call to func_{}", func_idx))?;

                let dest = callee_return_type.map(|_| self.new_var());

                // Check if this is a call to an imported function or a local function
                if func_idx < self.num_imported_functions {
                    // Call to imported function
                    let import_idx = func_idx;
                    let (module_name, func_name) =
                        self.func_imports.get(import_idx).cloned().ok_or_else(|| {
                            anyhow::anyhow!("Call: import index {} out of range", import_idx)
                        })?;

                    self.emit(IrInstr::CallImport {
                        dest,
                        import_idx: ImportIdx::new(import_idx),
                        module_name,
                        func_name,
                        args,
                    });
                } else {
                    // Call to local function - convert to local index
                    let local_func_idx = func_idx - self.num_imported_functions;
                    self.emit(IrInstr::Call {
                        dest,
                        func_idx: LocalFuncIdx::new(local_func_idx),
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
                let type_idx_usize = *type_index as usize;
                let (param_count, callee_return_type) =
                    *self.type_signatures.get(type_idx_usize).ok_or_else(|| {
                        anyhow::anyhow!("CallIndirect: unknown type index {}", type_idx_usize)
                    })?;

                // Pop table element index (on top of stack)
                let table_idx_var = self.value_stack.pop().ok_or_else(|| {
                    anyhow::anyhow!("Stack underflow for call_indirect table index")
                })?;

                // Pop arguments
                let args = self.pop_call_args(
                    param_count,
                    &format!("call_indirect type {}", type_idx_usize),
                )?;

                let dest = callee_return_type.map(|_| self.new_var());
                self.emit(IrInstr::CallIndirect {
                    dest,
                    type_idx: TypeIdx::new(*type_index as usize),
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
                self.dead_code = true;
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

            // === Bulk memory operations ===
            Operator::MemoryCopy {
                dst_mem: 0,
                src_mem: 0,
            } => {
                // Stack: [dst, src, len] (len on top)
                let len = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for memory.copy (len)"))?;
                let src = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for memory.copy (src)"))?;
                let dst = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for memory.copy (dst)"))?;
                self.emit(IrInstr::MemoryCopy { dst, src, len });
            }

            _ => bail!("Unsupported operator: {:?}", op),
        }

        Ok(())
    }

    /// Pop the top-of-stack value (if any) and terminate the current block with a Return.
    fn emit_return(&mut self) -> Result<()> {
        let value = if self.value_stack.is_empty() {
            None
        } else {
            Some(
                self.value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("stack underflow in return"))?,
            )
        };
        self.terminate(IrTerminator::Return { value });
        Ok(())
    }

    /// Emit a binary operation.
    pub(super) fn emit_binop(&mut self, op: BinOp) -> Result<()> {
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
    pub(super) fn emit_unop(&mut self, op: UnOp) -> Result<()> {
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
    pub(super) fn emit_load(&mut self, ty: WasmType, offset: u64) -> Result<()> {
        self.emit_load_ext(ty, offset, MemoryAccessWidth::Full, None)
    }

    /// Emit a sub-width memory load instruction with extension.
    /// Stack: [addr] -> [value]
    pub(super) fn emit_load_ext(
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
    pub(super) fn emit_store(&mut self, ty: WasmType, offset: u64) -> Result<()> {
        self.emit_store_narrow(ty, offset, MemoryAccessWidth::Full)
    }

    /// Pop `param_count` arguments from the value stack and return them in call order
    /// (first argument first). Returns an error if the stack underflows.
    fn pop_call_args(&mut self, param_count: usize, context: &str) -> Result<Vec<VarId>> {
        if self.value_stack.len() < param_count {
            bail!("Stack underflow for {}", context);
        }
        let mut args = Vec::with_capacity(param_count);
        for _ in 0..param_count {
            args.push(self.value_stack.pop().ok_or_else(|| {
                anyhow::anyhow!("stack underflow collecting {} arguments", context)
            })?);
        }
        args.reverse();
        Ok(args)
    }

    /// Emit a sub-width memory store instruction.
    /// Stack: [addr, value] -> []
    pub(super) fn emit_store_narrow(
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
}
