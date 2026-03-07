//! Operator translation — converts WebAssembly bytecode to SSA IR instructions.
//!
//! This module contains [`IrBuilder::translate_operator`] and the emit helpers
//! that form the core of the Wasm-to-IR conversion logic.
//!
//! ## SSA and Phi-Node Tracking
//!
//! WebAssembly locals are mutable, but the IR is in SSA form: every variable is
//! defined exactly once.  To reconcile this, every `local.set`/`local.tee`
//! allocates a **fresh variable** and updates `self.local_vars[i]` to point to
//! it.  Reads via `local.get` emit a copy of the current `local_vars[i]` so that
//! the copy's definition is stable even if the local is overwritten later.
//!
//! When control flow merges (at the exit of a `block`, `loop`, or `if`/`else`),
//! different predecessors may have written different variables to the same local.
//! The IR resolves this with **phi nodes** (`IrInstr::Phi`), inserted at the join
//! block, that select the right value depending on which predecessor ran.
//!
//! ### How phi predecessors are tracked
//!
//! Three mechanisms accumulate the predecessor information needed to build phi
//! nodes:
//!
//! 1. **`ControlFrame::locals_at_entry`** — a snapshot of `self.local_vars` taken
//!    when the frame is pushed.  Used as the "implicit else" predecessor for
//!    `if`-without-`else`, and as the loop-entry predecessor for loop phis.
//!
//! 2. **`ControlFrame::branch_incoming`** — filled by `record_forward_branch()`
//!    whenever a `br`/`br_if`/`br_table` jumps *forward* to the frame's
//!    `end_block`.  Each entry is `(predecessor_block_id, local_vars_snapshot)`.
//!    Loop frames are exempt: their backward branches go into `phi_patches` instead.
//!
//! 3. **`IrBuilder::phi_patches`** — filled by `record_loop_back_branch()` for
//!    backward edges (`br` inside a loop).  Consumed by `emit_loop_phis()` when
//!    the loop's `End` is processed.
//!
//! ### Protocol for branch instructions (`Br`, `BrIf`, `BrTable`)
//!
//! For every branch:
//! 1. Determine whether the target frame is a `Loop` (backward) or not (forward).
//! 2. Call the appropriate record method *before* terminating the block, so the
//!    snapshot captures the local state at the branch point.
//! 3. Terminate the block with the appropriate `IrTerminator`.
//! 4. Mark subsequent code as dead (`dead_code = true`) for unconditional branches.
//!
//! ### Protocol for join points (`End` of Block / If / Else / Loop)
//!
//! When an `End` is processed:
//! 1. Collect all predecessor snapshots: fall-through (current block, if reachable)
//!    + `frame.branch_incoming` for forward targets, or `phi_patches` for loops.
//! 2. Terminate the current block (jump to `end_block`).
//! 3. Activate `end_block` with `start_real_block` (clears `dead_code`).
//! 4. Call `insert_phis_at_join` to insert `IrInstr::Phi` for every local whose
//!    value differs across predecessors, and update `self.local_vars` to the phi
//!    dest vars.
//!
//! After all operators are translated, `lower_phis::lower` converts these phi
//! nodes into ordinary predecessor-block assignments before the optimizer runs.
//!
//! ### End-to-end tracing example
//!
//! Consider an `if`/`else` that writes to a local:
//!
//! ```wasm
//! ;; (func (param $cond i32) (param $x i32) (result i32)
//! ;;   local.get $cond
//! ;;   if
//! ;;     i32.const 10  local.set $x   ;; then: $x ← 10
//! ;;   else
//! ;;     i32.const 20  local.set $x   ;; else: $x ← 20
//! ;;   end
//! ;;   local.get $x)                  ;; result = chosen value
//! ```
//!
//! Step-by-step state of `local_vars` and the tracking structures:
//!
//! ```text
//! ── entry ──────────────────────────────────────────────────────────
//! local_vars = [v_cond, v_x]
//!
//! ── Operator::If ───────────────────────────────────────────────────
//! push_control(If):  locals_at_entry = [v_cond, v_x]  ← snapshot
//! BranchIf(v_cv, if_true=block_then, if_false=block_else)
//! start_real_block(block_then)
//!
//! ── then-branch ─────────────────────────────────────────────────────
//! local_vars = [v_cond, v_x]     (same as entry; then branch starts from snapshot)
//! local.set $x → v_xt = Assign(10); local_vars = [v_cond, v_xt]
//!
//! ── Operator::Else ──────────────────────────────────────────────────
//! then_pred_info = (block_then, [v_cond, v_xt])   ← saved for phi use at End
//! terminate block_then: Jump → block_join
//! local_vars restored to [v_cond, v_x]             ← locals_at_entry
//! start_real_block(block_else)
//!
//! ── else-branch ─────────────────────────────────────────────────────
//! local_vars = [v_cond, v_x]     (restored; else branch is independent of then)
//! local.set $x → v_xe = Assign(20); local_vars = [v_cond, v_xe]
//!
//! ── Operator::End (Else frame) ──────────────────────────────────────
//! preds = [
//!   (block_then, [v_cond, v_xt]),   ← from then_pred_info
//!   (block_else, [v_cond, v_xe]),   ← else fall-through (current block)
//! ]
//! terminate block_else: Jump → block_join
//! start_real_block(block_join)
//! insert_phis_at_join(block_join, preds):
//!   $cond: v_cond == v_cond → no phi needed
//!   $x:    v_xt   != v_xe   → v_phi = Phi[(block_then, v_xt), (block_else, v_xe)]
//! local_vars = [v_cond, v_phi]
//!
//! ── block_join ──────────────────────────────────────────────────────
//! local.get $x → Assign(v_phi)   ← reads the phi result
//! return
//! ```

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
                let v = IrValue::I32(*value);
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::Const { dest: d, value: v });
                self.value_stack.push(use_v);
            }

            Operator::I64Const { value } => {
                let v = IrValue::I64(*value);
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::Const { dest: d, value: v });
                self.value_stack.push(use_v);
            }

            Operator::F32Const { value } => {
                let v = IrValue::F32(f32::from_bits(value.bits()));
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::Const { dest: d, value: v });
                self.value_stack.push(use_v);
            }

            Operator::F64Const { value } => {
                let v = IrValue::F64(f64::from_bits(value.bits()));
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::Const { dest: d, value: v });
                self.value_stack.push(use_v);
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
                // Emit a copy rather than pushing the local's UseVar directly.
                // If we push the local's UseVar, a later local.tee/local.set that
                // overwrites the same local will corrupt any already-pushed reference
                // to it, because the backend emits sequential mutable assignments.
                // A fresh variable captures the value at this point in time.
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::Assign {
                    dest: d,
                    src: src.var_id(),
                });
                self.value_stack.push(use_v);
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
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::Assign {
                    dest: d,
                    src: value.var_id(),
                });
                // Update the local mapping so subsequent reads see the new value.
                self.local_vars[idx] = use_v;
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
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::Assign {
                    dest: d,
                    src: value.var_id(),
                });
                // Update the local mapping so subsequent reads see the new value.
                self.local_vars[idx] = use_v;
                // Value stays on stack (already there via .last())
            }

            // Global variable access
            Operator::GlobalGet { global_index } => {
                let idx = GlobalIdx::new(*global_index as usize);
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::GlobalGet {
                    dest: d,
                    index: idx,
                });
                self.value_stack.push(use_v);
            }

            Operator::GlobalSet { global_index } => {
                let value = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for global.set"))?;
                self.emit_void(IrInstr::GlobalSet {
                    index: GlobalIdx::new(*global_index as usize),
                    value: value.var_id(),
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
                            let mut preds: Vec<(BlockId, Vec<UseVar>)> = Vec::new();
                            if !self.dead_code {
                                preds.push((self.current_block, self.local_vars.clone()));
                            }
                            preds.extend(frame.branch_incoming.iter().cloned());

                            // Assign result if needed (then-branch fall-through)
                            if let Some(result_var) = frame.result_var {
                                if let Some(stack_value) = self.value_stack.pop() {
                                    self.emit_void(IrInstr::Assign {
                                        dest: result_var.var_id(),
                                        src: stack_value.var_id(),
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
                            let mut preds: Vec<(BlockId, Vec<UseVar>)> = Vec::new();
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
                                    self.emit_void(IrInstr::Assign {
                                        dest: result_var.var_id(),
                                        src: stack_value.var_id(),
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
                            let mut preds: Vec<(BlockId, Vec<UseVar>)> =
                                frame.branch_incoming.clone();
                            if !self.dead_code {
                                preds.push((self.current_block, self.local_vars.clone()));
                            }

                            // Assign result if needed (loop fall-through)
                            if let Some(result_var) = frame.result_var {
                                if let Some(stack_value) = self.value_stack.pop() {
                                    self.emit_void(IrInstr::Assign {
                                        dest: result_var.var_id(),
                                        src: stack_value.var_id(),
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
                            let mut preds: Vec<(BlockId, Vec<UseVar>)> =
                                frame.branch_incoming.clone();
                            if !self.dead_code {
                                preds.push((self.current_block, self.local_vars.clone()));
                            }

                            // Assign result if needed (block fall-through)
                            if let Some(result_var) = frame.result_var {
                                if let Some(stack_value) = self.value_stack.pop() {
                                    // Normal case: block fell through with a result value
                                    self.emit_void(IrInstr::Assign {
                                        dest: result_var.var_id(),
                                        src: stack_value.var_id(),
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
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::MemorySize { dest: d });
                self.value_stack.push(use_v);
            }

            Operator::MemoryGrow { mem: 0, .. } => {
                let delta = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for memory.grow"))?;
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::MemoryGrow {
                    dest: d,
                    delta: delta.var_id(),
                });
                self.value_stack.push(use_v);
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
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for if condition"))?
                    .var_id();

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
                // === Transition from then-branch to else-branch ===
                //
                // At this point we are at the end of the then-branch body.  We need to:
                //   1. Finalize the then-branch (assign result, emit jump to end_block).
                //   2. Save the then-branch's fall-through predecessor info so it can
                //      be included in the phi computation at end_block later.
                //   3. Restore local_vars to the pre-if snapshot so the else-branch
                //      starts with the same local state that the then-branch started with.
                //   4. Activate the pre-allocated else_block and push an Else frame.
                //
                // Why restore local_vars?
                //   The then-branch may have modified locals (local.set).  The else-branch
                //   must start with the *pre-if* locals, not the then-branch's modified ones.
                //   This is captured in `if_frame.locals_at_entry`.
                //
                // Why save then_pred_info instead of calling record_forward_branch?
                //   At Else time, end_block hasn't been activated yet, and the Else frame
                //   doesn't exist yet.  We stash the then-fall-through as `then_pred_info`
                //   in the new Else frame; the End handler reads it from there.
                //   (Any `br 0` inside the then-body went through record_forward_branch
                //   normally and is already in `if_frame.branch_incoming`.)

                let if_frame = self.pop_control().context("else without matching if")?;

                if if_frame.kind != super::core::ControlKind::If {
                    bail!("else without matching if");
                }

                // Step 1a: Save then-branch fall-through as a phi predecessor for end_block.
                // Only recorded if the then-branch body is reachable (not dead code after
                // an unconditional `br` or `return` inside the then-branch).
                let then_pred_info = if !self.dead_code {
                    Some((self.current_block, self.local_vars.clone()))
                } else {
                    None
                };

                // Step 1b: Assign the result variable from the then-branch value (if typed).
                let result_var = if_frame.result_var;
                if let Some(result_var) = result_var {
                    if let Some(stack_value) = self.value_stack.pop() {
                        self.emit_void(IrInstr::Assign {
                            dest: result_var.var_id(),
                            src: stack_value.var_id(),
                        });
                    }
                }

                // Step 1c: Terminate the then-branch with a jump to end_block.
                if !self.dead_code {
                    self.terminate(IrTerminator::Jump {
                        target: if_frame.end_block,
                    });
                }

                // Step 3: Restore local_vars to the pre-if snapshot.
                // The else-branch must see the same locals that entered the if.
                self.local_vars = if_frame.locals_at_entry.clone();

                // Step 4: Activate the else block (always reachable: false path of BranchIf).
                let else_block = if_frame
                    .else_block
                    .expect("If frame should have else_block");
                self.start_real_block(else_block);

                // Push Else frame.  Transfer `branch_incoming` from the If frame so that
                // any `br 0` emitted inside the then-branch is still tracked as a predecessor
                // of end_block when End is processed.
                self.control_stack.push(super::core::ControlFrame {
                    kind: super::core::ControlKind::Else,
                    start_block: else_block,
                    end_block: if_frame.end_block,
                    else_block: None,
                    result_type: if_frame.result_type,
                    result_var,
                    locals_at_entry: if_frame.locals_at_entry,
                    branch_incoming: if_frame.branch_incoming, // then-body forward branches
                    then_pred_info,                            // then fall-through predecessor
                    loop_phi_vars: Vec::new(),
                    pre_loop_block: None,
                });
            }

            Operator::Br { relative_depth } => {
                // === Unconditional branch ===
                //
                // "br N" jumps to the Nth enclosing control structure:
                //   - Loop frame  → backward branch to start_block (re-enter the loop)
                //   - Other frame → forward branch to end_block (exit the block)
                //
                // Before terminating the block we snapshot `local_vars` and record it
                // as a phi predecessor for the target frame.  This snapshot is used by
                // `insert_phis_at_join` (or `emit_loop_phis`) to build the phi sources.
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

                // Record snapshot *before* terminating so local_vars is still valid.
                if is_loop {
                    // Backward branch: store (phi_var, current_block, src_var) in phi_patches.
                    // Consumed by emit_loop_phis when the loop's End is processed.
                    self.record_loop_back_branch(frame_idx);
                } else {
                    // Forward branch: push (current_block, local_vars) into branch_incoming.
                    // Consumed by insert_phis_at_join when the target frame's End is processed.
                    self.record_forward_branch(frame_idx);
                }

                self.terminate(IrTerminator::Jump { target });

                // Everything after an unconditional branch is unreachable.
                // We still need a block to absorb any subsequent instructions
                // (e.g. the End of the enclosing block) without corrupting the
                // already-terminated block.
                let dead_block = self.new_block();
                self.start_block(dead_block);
                self.dead_code = true;
            }

            Operator::BrIf { relative_depth } => {
                // === Conditional branch ===
                //
                // "br_if N" pops an i32 condition.  If nonzero, branches to the Nth
                // enclosing frame's target; otherwise falls through to the next instruction.
                //
                // Phi tracking: we record the *taken* path as a predecessor of the target
                // frame (same as unconditional Br).  The fall-through path continues in a
                // new block and does NOT need recording here — the current `local_vars`
                // state carries forward naturally into the continuation block.
                let condition = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for br_if"))?
                    .var_id();

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

                // Record the taken branch as a phi predecessor (snapshot before termination).
                if is_loop {
                    self.record_loop_back_branch(frame_idx);
                } else {
                    self.record_forward_branch(frame_idx);
                }

                // The fall-through block is always reachable (the false path of BranchIf).
                let continue_block = self.new_block();

                self.terminate(IrTerminator::BranchIf {
                    condition,
                    if_true: target,
                    if_false: continue_block,
                });

                // start_real_block clears dead_code — the fallthrough is always live.
                self.start_real_block(continue_block);
            }

            Operator::BrTable { targets } => {
                // === Jump table ===
                //
                // "br_table [d0 d1 ... dN] default" pops an index and branches to depth
                // d_index if index ≤ N, otherwise to `default`.
                //
                // Phi tracking: each distinct target frame must receive a predecessor
                // snapshot.  Multiple table entries may resolve to the *same* frame
                // (same depth → same frame_idx), so we deduplicate by frame_idx using
                // `recorded` to avoid recording the same block twice for the same phi.
                let index = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Stack underflow for br_table"))?
                    .var_id();

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

                // Everything after br_table is unreachable (same as Br).
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

                // For optional-result calls we use new_pre_alloc_var: the dest is
                // defined by the call instruction itself, not via emit_def.
                let (dest_id, dest_use) = if callee_return_type.is_some() {
                    let (id, u) = self.new_pre_alloc_var();
                    (Some(id), Some(u))
                } else {
                    (None, None)
                };

                // Check if this is a call to an imported function or a local function
                if func_idx < self.num_imported_functions {
                    // Call to imported function
                    let import_idx = func_idx;
                    let (module_name, func_name) =
                        self.func_imports.get(import_idx).cloned().ok_or_else(|| {
                            anyhow::anyhow!("Call: import index {} out of range", import_idx)
                        })?;

                    self.emit_void(IrInstr::CallImport {
                        dest: dest_id,
                        import_idx: ImportIdx::new(import_idx),
                        module_name,
                        func_name,
                        args,
                    });
                } else {
                    // Call to local function - convert to local index
                    let local_func_idx = func_idx - self.num_imported_functions;
                    self.emit_void(IrInstr::Call {
                        dest: dest_id,
                        func_idx: LocalFuncIdx::new(local_func_idx),
                        args,
                    });
                }

                if let Some(u) = dest_use {
                    self.value_stack.push(u);
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
                let table_idx_var = self
                    .value_stack
                    .pop()
                    .ok_or_else(|| {
                        anyhow::anyhow!("Stack underflow for call_indirect table index")
                    })?
                    .var_id();

                // Pop arguments
                let args = self.pop_call_args(
                    param_count,
                    &format!("call_indirect type {}", type_idx_usize),
                )?;

                let (dest_id, dest_use) = if callee_return_type.is_some() {
                    let (id, u) = self.new_pre_alloc_var();
                    (Some(id), Some(u))
                } else {
                    (None, None)
                };
                self.emit_void(IrInstr::CallIndirect {
                    dest: dest_id,
                    type_idx: TypeIdx::new(*type_index as usize),
                    table_idx: table_idx_var,
                    args,
                });

                if let Some(u) = dest_use {
                    self.value_stack.push(u);
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
                let def = self.new_var();
                let use_v = self.emit_def(def, |d| IrInstr::Select {
                    dest: d,
                    val1: val1.var_id(),
                    val2: val2.var_id(),
                    condition: condition.var_id(),
                });
                self.value_stack.push(use_v);
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
                self.emit_void(IrInstr::MemoryCopy {
                    dst: dst.var_id(),
                    src: src.var_id(),
                    len: len.var_id(),
                });
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
                    .ok_or_else(|| anyhow::anyhow!("stack underflow in return"))?
                    .var_id(),
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
        let use_v = self.emit_def(dest, |v| IrInstr::BinOp {
            dest: v,
            op,
            lhs: lhs.var_id(),
            rhs: rhs.var_id(),
        });
        self.value_stack.push(use_v);

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
        let use_v = self.emit_def(dest, |v| IrInstr::UnOp {
            dest: v,
            op,
            operand: operand.var_id(),
        });
        self.value_stack.push(use_v);

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
        let use_v = self.emit_def(dest, |v| IrInstr::Load {
            dest: v,
            ty,
            addr: addr.var_id(),
            offset: offset as u32,
            width,
            sign,
        });
        self.value_stack.push(use_v);
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
            args.push(
                self.value_stack
                    .pop()
                    .ok_or_else(|| {
                        anyhow::anyhow!("stack underflow collecting {} arguments", context)
                    })?
                    .var_id(),
            );
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

        self.emit_void(IrInstr::Store {
            ty,
            addr: addr.var_id(),
            value: value.var_id(),
            offset: offset as u32,
            width,
        });

        // Store has no result
        Ok(())
    }
}
