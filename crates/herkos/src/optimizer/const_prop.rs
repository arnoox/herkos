//! Constant folding and propagation.
//!
//! ## What it does
//!
//! Tracks which `VarId`s hold known constant values, then:
//!
//! - **Propagates** constants through `Assign` chains.
//! - **Folds** `BinOp(Const, Const)` and `UnOp(Const)` into `Const` when the
//!   result is statically computable.
//!
//! ## Algorithm (block-local)
//!
//! 1. Maintain `HashMap<VarId, IrValue>` of known constants per block.
//! 2. Walk instructions in order:
//!    - `Const { dest, value }` → record `dest → value`
//!    - `Assign { dest, src }` where src is known → replace with `Const`, record
//!    - `BinOp { dest, op, lhs, rhs }` where both known → fold via `try_eval_binop`
//!    - `UnOp { dest, op, operand }` where operand known → fold via `try_eval_unop`
//! 3. Run to fixpoint (across all blocks).
//!
//! ## Safety
//!
//! Operations that may trap at runtime are **not** folded:
//! - `DivS`/`DivU`/`RemS`/`RemU` with divisor 0
//! - `I32DivS(I32_MIN, -1)` / `I64DivS(I64_MIN, -1)` — signed overflow
//! - `TruncF*` with NaN or out-of-range float

use super::utils::instr_dest;
use crate::ir::{BinOp, IrFunction, IrInstr, IrValue, UnOp, VarId};
use std::collections::HashMap;

// ── Public entry point ────────────────────────────────────────────────────────

/// Run constant propagation and folding to fixpoint.
pub fn eliminate(func: &mut IrFunction) {
    loop {
        let mut changed = false;
        for block in &mut func.blocks {
            let mut known: HashMap<VarId, IrValue> = HashMap::new();

            for instr in &mut block.instructions {
                // Track whether we recorded a known constant for this instruction's dest.
                let mut folded = false;

                match instr {
                    IrInstr::Const { dest, value } => {
                        known.insert(*dest, *value);
                        folded = true;
                    }
                    IrInstr::Assign { dest, src } => {
                        let d = *dest;
                        if let Some(val) = known.get(src).copied() {
                            *instr = IrInstr::Const {
                                dest: d,
                                value: val,
                            };
                            known.insert(d, val);
                            changed = true;
                            folded = true;
                        }
                    }
                    IrInstr::BinOp {
                        dest, op, lhs, rhs, ..
                    } => {
                        let (d, o) = (*dest, *op);
                        if let (Some(lv), Some(rv)) =
                            (known.get(lhs).copied(), known.get(rhs).copied())
                        {
                            if let Some(result) = try_eval_binop(o, lv, rv) {
                                *instr = IrInstr::Const {
                                    dest: d,
                                    value: result,
                                };
                                known.insert(d, result);
                                changed = true;
                                folded = true;
                            }
                        }
                    }
                    IrInstr::UnOp {
                        dest, op, operand, ..
                    } => {
                        let (d, o) = (*dest, *op);
                        if let Some(val) = known.get(operand).copied() {
                            if let Some(result) = try_eval_unop(o, val) {
                                *instr = IrInstr::Const {
                                    dest: d,
                                    value: result,
                                };
                                known.insert(d, result);
                                changed = true;
                                folded = true;
                            }
                        }
                    }
                    _ => {}
                }

                // If this instruction defines a variable but we didn't fold it to
                // a constant, invalidate any stale entry. This is critical for
                // non-SSA variables that are redefined (e.g., loop accumulators).
                if !folded {
                    if let Some(dest) = instr_dest(instr) {
                        known.remove(&dest);
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
}

// ── Binary operation folding ──────────────────────────────────────────────────

/// Attempt to evaluate a binary operation on two constant values.
///
/// Returns `None` when:
/// - The value types don't match the expected operand types for the op.
/// - The operation would trap at runtime (div/rem by zero, signed overflow,
///   etc.) — we must preserve the runtime trap rather than folding it away.
fn try_eval_binop(op: BinOp, lhs: IrValue, rhs: IrValue) -> Option<IrValue> {
    match (op, lhs, rhs) {
        // ── i32 arithmetic ──────────────────────────────────────────────
        (BinOp::I32Add, IrValue::I32(a), IrValue::I32(b)) => Some(IrValue::I32(a.wrapping_add(b))),
        (BinOp::I32Sub, IrValue::I32(a), IrValue::I32(b)) => Some(IrValue::I32(a.wrapping_sub(b))),
        (BinOp::I32Mul, IrValue::I32(a), IrValue::I32(b)) => Some(IrValue::I32(a.wrapping_mul(b))),

        // Division/remainder: do NOT fold if it would trap.
        (BinOp::I32DivS, IrValue::I32(a), IrValue::I32(b)) => a.checked_div(b).map(IrValue::I32),
        (BinOp::I32DivU, IrValue::I32(a), IrValue::I32(b)) => (a as u32)
            .checked_div(b as u32)
            .map(|v| IrValue::I32(v as i32)),
        (BinOp::I32RemS, IrValue::I32(a), IrValue::I32(b)) => {
            if b == 0 {
                None
            } else if a == i32::MIN && b == -1 {
                Some(IrValue::I32(0))
            } else {
                Some(IrValue::I32(a % b))
            }
        }
        (BinOp::I32RemU, IrValue::I32(a), IrValue::I32(b)) => (a as u32)
            .checked_rem(b as u32)
            .map(|v| IrValue::I32(v as i32)),

        // Bitwise
        (BinOp::I32And, IrValue::I32(a), IrValue::I32(b)) => Some(IrValue::I32(a & b)),
        (BinOp::I32Or, IrValue::I32(a), IrValue::I32(b)) => Some(IrValue::I32(a | b)),
        (BinOp::I32Xor, IrValue::I32(a), IrValue::I32(b)) => Some(IrValue::I32(a ^ b)),

        // Shifts/rotates (Wasm masks shift amount by type width)
        (BinOp::I32Shl, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(a.wrapping_shl(b as u32 & 31)))
        }
        (BinOp::I32ShrS, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(a.wrapping_shr(b as u32 & 31)))
        }
        (BinOp::I32ShrU, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32((a as u32).wrapping_shr(b as u32 & 31) as i32))
        }
        (BinOp::I32Rotl, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32((a as u32).rotate_left(b as u32 & 31) as i32))
        }
        (BinOp::I32Rotr, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32((a as u32).rotate_right(b as u32 & 31) as i32))
        }

        // i32 comparisons
        (BinOp::I32Eq, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(if a == b { 1 } else { 0 }))
        }
        (BinOp::I32Ne, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(if a != b { 1 } else { 0 }))
        }
        (BinOp::I32LtS, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(if a < b { 1 } else { 0 }))
        }
        (BinOp::I32LtU, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(if (a as u32) < (b as u32) { 1 } else { 0 }))
        }
        (BinOp::I32GtS, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(if a > b { 1 } else { 0 }))
        }
        (BinOp::I32GtU, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(if (a as u32) > (b as u32) { 1 } else { 0 }))
        }
        (BinOp::I32LeS, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(if a <= b { 1 } else { 0 }))
        }
        (BinOp::I32LeU, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(if (a as u32) <= (b as u32) { 1 } else { 0 }))
        }
        (BinOp::I32GeS, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(if a >= b { 1 } else { 0 }))
        }
        (BinOp::I32GeU, IrValue::I32(a), IrValue::I32(b)) => {
            Some(IrValue::I32(if (a as u32) >= (b as u32) { 1 } else { 0 }))
        }

        // ── i64 arithmetic ──────────────────────────────────────────────
        (BinOp::I64Add, IrValue::I64(a), IrValue::I64(b)) => Some(IrValue::I64(a.wrapping_add(b))),
        (BinOp::I64Sub, IrValue::I64(a), IrValue::I64(b)) => Some(IrValue::I64(a.wrapping_sub(b))),
        (BinOp::I64Mul, IrValue::I64(a), IrValue::I64(b)) => Some(IrValue::I64(a.wrapping_mul(b))),

        (BinOp::I64DivS, IrValue::I64(a), IrValue::I64(b)) => a.checked_div(b).map(IrValue::I64),
        (BinOp::I64DivU, IrValue::I64(a), IrValue::I64(b)) => (a as u64)
            .checked_div(b as u64)
            .map(|v| IrValue::I64(v as i64)),
        (BinOp::I64RemS, IrValue::I64(a), IrValue::I64(b)) => {
            if b == 0 {
                None
            } else if a == i64::MIN && b == -1 {
                Some(IrValue::I64(0))
            } else {
                Some(IrValue::I64(a % b))
            }
        }
        (BinOp::I64RemU, IrValue::I64(a), IrValue::I64(b)) => (a as u64)
            .checked_rem(b as u64)
            .map(|v| IrValue::I64(v as i64)),

        // Bitwise
        (BinOp::I64And, IrValue::I64(a), IrValue::I64(b)) => Some(IrValue::I64(a & b)),
        (BinOp::I64Or, IrValue::I64(a), IrValue::I64(b)) => Some(IrValue::I64(a | b)),
        (BinOp::I64Xor, IrValue::I64(a), IrValue::I64(b)) => Some(IrValue::I64(a ^ b)),

        // Shifts/rotates (Wasm masks by 63 for i64)
        (BinOp::I64Shl, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I64(a.wrapping_shl(b as u32 & 63)))
        }
        (BinOp::I64ShrS, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I64(a.wrapping_shr(b as u32 & 63)))
        }
        (BinOp::I64ShrU, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I64((a as u64).wrapping_shr(b as u32 & 63) as i64))
        }
        (BinOp::I64Rotl, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I64((a as u64).rotate_left(b as u32 & 63) as i64))
        }
        (BinOp::I64Rotr, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I64((a as u64).rotate_right(b as u32 & 63) as i64))
        }

        // i64 comparisons (result is i32)
        (BinOp::I64Eq, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I32(if a == b { 1 } else { 0 }))
        }
        (BinOp::I64Ne, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I32(if a != b { 1 } else { 0 }))
        }
        (BinOp::I64LtS, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I32(if a < b { 1 } else { 0 }))
        }
        (BinOp::I64LtU, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I32(if (a as u64) < (b as u64) { 1 } else { 0 }))
        }
        (BinOp::I64GtS, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I32(if a > b { 1 } else { 0 }))
        }
        (BinOp::I64GtU, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I32(if (a as u64) > (b as u64) { 1 } else { 0 }))
        }
        (BinOp::I64LeS, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I32(if a <= b { 1 } else { 0 }))
        }
        (BinOp::I64LeU, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I32(if (a as u64) <= (b as u64) { 1 } else { 0 }))
        }
        (BinOp::I64GeS, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I32(if a >= b { 1 } else { 0 }))
        }
        (BinOp::I64GeU, IrValue::I64(a), IrValue::I64(b)) => {
            Some(IrValue::I32(if (a as u64) >= (b as u64) { 1 } else { 0 }))
        }

        // ── f32 arithmetic ──────────────────────────────────────────────
        (BinOp::F32Add, IrValue::F32(a), IrValue::F32(b)) => Some(IrValue::F32(a + b)),
        (BinOp::F32Sub, IrValue::F32(a), IrValue::F32(b)) => Some(IrValue::F32(a - b)),
        (BinOp::F32Mul, IrValue::F32(a), IrValue::F32(b)) => Some(IrValue::F32(a * b)),
        (BinOp::F32Div, IrValue::F32(a), IrValue::F32(b)) => Some(IrValue::F32(a / b)),
        (BinOp::F32Min, IrValue::F32(a), IrValue::F32(b)) => Some(IrValue::F32(a.min(b))),
        (BinOp::F32Max, IrValue::F32(a), IrValue::F32(b)) => Some(IrValue::F32(a.max(b))),
        (BinOp::F32Copysign, IrValue::F32(a), IrValue::F32(b)) => Some(IrValue::F32(a.copysign(b))),

        // f32 comparisons (result is i32)
        (BinOp::F32Eq, IrValue::F32(a), IrValue::F32(b)) => {
            Some(IrValue::I32(if a == b { 1 } else { 0 }))
        }
        (BinOp::F32Ne, IrValue::F32(a), IrValue::F32(b)) => {
            Some(IrValue::I32(if a != b { 1 } else { 0 }))
        }
        (BinOp::F32Lt, IrValue::F32(a), IrValue::F32(b)) => {
            Some(IrValue::I32(if a < b { 1 } else { 0 }))
        }
        (BinOp::F32Gt, IrValue::F32(a), IrValue::F32(b)) => {
            Some(IrValue::I32(if a > b { 1 } else { 0 }))
        }
        (BinOp::F32Le, IrValue::F32(a), IrValue::F32(b)) => {
            Some(IrValue::I32(if a <= b { 1 } else { 0 }))
        }
        (BinOp::F32Ge, IrValue::F32(a), IrValue::F32(b)) => {
            Some(IrValue::I32(if a >= b { 1 } else { 0 }))
        }

        // ── f64 arithmetic ──────────────────────────────────────────────
        (BinOp::F64Add, IrValue::F64(a), IrValue::F64(b)) => Some(IrValue::F64(a + b)),
        (BinOp::F64Sub, IrValue::F64(a), IrValue::F64(b)) => Some(IrValue::F64(a - b)),
        (BinOp::F64Mul, IrValue::F64(a), IrValue::F64(b)) => Some(IrValue::F64(a * b)),
        (BinOp::F64Div, IrValue::F64(a), IrValue::F64(b)) => Some(IrValue::F64(a / b)),
        (BinOp::F64Min, IrValue::F64(a), IrValue::F64(b)) => Some(IrValue::F64(a.min(b))),
        (BinOp::F64Max, IrValue::F64(a), IrValue::F64(b)) => Some(IrValue::F64(a.max(b))),
        (BinOp::F64Copysign, IrValue::F64(a), IrValue::F64(b)) => Some(IrValue::F64(a.copysign(b))),

        // f64 comparisons (result is i32)
        (BinOp::F64Eq, IrValue::F64(a), IrValue::F64(b)) => {
            Some(IrValue::I32(if a == b { 1 } else { 0 }))
        }
        (BinOp::F64Ne, IrValue::F64(a), IrValue::F64(b)) => {
            Some(IrValue::I32(if a != b { 1 } else { 0 }))
        }
        (BinOp::F64Lt, IrValue::F64(a), IrValue::F64(b)) => {
            Some(IrValue::I32(if a < b { 1 } else { 0 }))
        }
        (BinOp::F64Gt, IrValue::F64(a), IrValue::F64(b)) => {
            Some(IrValue::I32(if a > b { 1 } else { 0 }))
        }
        (BinOp::F64Le, IrValue::F64(a), IrValue::F64(b)) => {
            Some(IrValue::I32(if a <= b { 1 } else { 0 }))
        }
        (BinOp::F64Ge, IrValue::F64(a), IrValue::F64(b)) => {
            Some(IrValue::I32(if a >= b { 1 } else { 0 }))
        }

        // Type mismatch — don't fold.
        _ => None,
    }
}

// ── Unary operation folding ───────────────────────────────────────────────────

/// Attempt to evaluate a unary operation on a constant value.
///
/// Returns `None` for trapping conversions (`TruncF*` with NaN/out-of-range).
fn try_eval_unop(op: UnOp, val: IrValue) -> Option<IrValue> {
    match (op, val) {
        // ── i32 unary ───────────────────────────────────────────────────
        (UnOp::I32Clz, IrValue::I32(v)) => Some(IrValue::I32((v as u32).leading_zeros() as i32)),
        (UnOp::I32Ctz, IrValue::I32(v)) => Some(IrValue::I32((v as u32).trailing_zeros() as i32)),
        (UnOp::I32Popcnt, IrValue::I32(v)) => Some(IrValue::I32((v as u32).count_ones() as i32)),
        (UnOp::I32Eqz, IrValue::I32(v)) => Some(IrValue::I32(if v == 0 { 1 } else { 0 })),

        // ── i64 unary ───────────────────────────────────────────────────
        (UnOp::I64Clz, IrValue::I64(v)) => Some(IrValue::I64((v as u64).leading_zeros() as i64)),
        (UnOp::I64Ctz, IrValue::I64(v)) => Some(IrValue::I64((v as u64).trailing_zeros() as i64)),
        (UnOp::I64Popcnt, IrValue::I64(v)) => Some(IrValue::I64((v as u64).count_ones() as i64)),
        (UnOp::I64Eqz, IrValue::I64(v)) => Some(IrValue::I32(if v == 0 { 1 } else { 0 })),

        // ── f32 unary ───────────────────────────────────────────────────
        (UnOp::F32Abs, IrValue::F32(v)) => Some(IrValue::F32(v.abs())),
        (UnOp::F32Neg, IrValue::F32(v)) => Some(IrValue::F32(-v)),
        (UnOp::F32Ceil, IrValue::F32(v)) => Some(IrValue::F32(v.ceil())),
        (UnOp::F32Floor, IrValue::F32(v)) => Some(IrValue::F32(v.floor())),
        (UnOp::F32Trunc, IrValue::F32(v)) => Some(IrValue::F32(v.trunc())),
        (UnOp::F32Nearest, IrValue::F32(v)) => Some(IrValue::F32(wasm_nearest_f32(v))),
        (UnOp::F32Sqrt, IrValue::F32(v)) => Some(IrValue::F32(v.sqrt())),

        // ── f64 unary ───────────────────────────────────────────────────
        (UnOp::F64Abs, IrValue::F64(v)) => Some(IrValue::F64(v.abs())),
        (UnOp::F64Neg, IrValue::F64(v)) => Some(IrValue::F64(-v)),
        (UnOp::F64Ceil, IrValue::F64(v)) => Some(IrValue::F64(v.ceil())),
        (UnOp::F64Floor, IrValue::F64(v)) => Some(IrValue::F64(v.floor())),
        (UnOp::F64Trunc, IrValue::F64(v)) => Some(IrValue::F64(v.trunc())),
        (UnOp::F64Nearest, IrValue::F64(v)) => Some(IrValue::F64(wasm_nearest_f64(v))),
        (UnOp::F64Sqrt, IrValue::F64(v)) => Some(IrValue::F64(v.sqrt())),

        // ── Integer conversions ─────────────────────────────────────────
        (UnOp::I32WrapI64, IrValue::I64(v)) => Some(IrValue::I32(v as i32)),
        (UnOp::I64ExtendI32S, IrValue::I32(v)) => Some(IrValue::I64(v as i64)),
        (UnOp::I64ExtendI32U, IrValue::I32(v)) => Some(IrValue::I64((v as u32) as i64)),

        // ── Float → integer (trapping) — do NOT fold on NaN/overflow ──
        (UnOp::I32TruncF32S, IrValue::F32(v)) => {
            if v.is_nan() || !(-2147483648.0f32..2147483648.0f32).contains(&v) {
                None
            } else {
                Some(IrValue::I32(v as i32))
            }
        }
        (UnOp::I32TruncF32U, IrValue::F32(v)) => {
            if v.is_nan() || v >= 4294967296.0f32 || v <= -1.0f32 {
                None
            } else {
                Some(IrValue::I32(v as u32 as i32))
            }
        }
        (UnOp::I32TruncF64S, IrValue::F64(v)) => {
            if v.is_nan() || !(-2147483648.0f64..2147483648.0f64).contains(&v) {
                None
            } else {
                Some(IrValue::I32(v as i32))
            }
        }
        (UnOp::I32TruncF64U, IrValue::F64(v)) => {
            if v.is_nan() || v >= 4294967296.0f64 || v <= -1.0f64 {
                None
            } else {
                Some(IrValue::I32(v as u32 as i32))
            }
        }
        (UnOp::I64TruncF32S, IrValue::F32(v)) => {
            if v.is_nan() || !(-9223372036854775808.0f32..9223372036854775808.0f32).contains(&v) {
                None
            } else {
                Some(IrValue::I64(v as i64))
            }
        }
        (UnOp::I64TruncF32U, IrValue::F32(v)) => {
            if v.is_nan() || v >= 18446744073709551616.0f32 || v <= -1.0f32 {
                None
            } else {
                Some(IrValue::I64(v as u64 as i64))
            }
        }
        (UnOp::I64TruncF64S, IrValue::F64(v)) => {
            if v.is_nan() || !(-9223372036854775808.0f64..9223372036854775808.0f64).contains(&v) {
                None
            } else {
                Some(IrValue::I64(v as i64))
            }
        }
        (UnOp::I64TruncF64U, IrValue::F64(v)) => {
            if v.is_nan() || v >= 18446744073709551616.0f64 || v <= -1.0f64 {
                None
            } else {
                Some(IrValue::I64(v as u64 as i64))
            }
        }

        // ── Integer → float conversions ─────────────────────────────────
        (UnOp::F32ConvertI32S, IrValue::I32(v)) => Some(IrValue::F32(v as f32)),
        (UnOp::F32ConvertI32U, IrValue::I32(v)) => Some(IrValue::F32((v as u32) as f32)),
        (UnOp::F32ConvertI64S, IrValue::I64(v)) => Some(IrValue::F32(v as f32)),
        (UnOp::F32ConvertI64U, IrValue::I64(v)) => Some(IrValue::F32((v as u64) as f32)),
        (UnOp::F64ConvertI32S, IrValue::I32(v)) => Some(IrValue::F64(v as f64)),
        (UnOp::F64ConvertI32U, IrValue::I32(v)) => Some(IrValue::F64((v as u32) as f64)),
        (UnOp::F64ConvertI64S, IrValue::I64(v)) => Some(IrValue::F64(v as f64)),
        (UnOp::F64ConvertI64U, IrValue::I64(v)) => Some(IrValue::F64((v as u64) as f64)),

        // ── Float precision conversions ─────────────────────────────────
        (UnOp::F32DemoteF64, IrValue::F64(v)) => Some(IrValue::F32(v as f32)),
        (UnOp::F64PromoteF32, IrValue::F32(v)) => Some(IrValue::F64(v as f64)),

        // ── Reinterpretations (bitcast) ─────────────────────────────────
        (UnOp::I32ReinterpretF32, IrValue::F32(v)) => Some(IrValue::I32(v.to_bits() as i32)),
        (UnOp::I64ReinterpretF64, IrValue::F64(v)) => Some(IrValue::I64(v.to_bits() as i64)),
        (UnOp::F32ReinterpretI32, IrValue::I32(v)) => Some(IrValue::F32(f32::from_bits(v as u32))),
        (UnOp::F64ReinterpretI64, IrValue::I64(v)) => Some(IrValue::F64(f64::from_bits(v as u64))),

        // Type mismatch — don't fold.
        _ => None,
    }
}

// ── Wasm "nearest" semantics ──────────────────────────────────────────────────

/// Wasm `f32.nearest` — round to nearest even (banker's rounding).
fn wasm_nearest_f32(v: f32) -> f32 {
    if v.is_nan() || v.is_infinite() || v == 0.0 {
        return v;
    }
    let rounded = v.round();
    // When exactly between two integers, round to even.
    if (v - rounded).abs() == 0.5 {
        let truncated = v.trunc();
        if truncated % 2.0 == 0.0 {
            truncated
        } else {
            rounded
        }
    } else {
        rounded
    }
}

/// Wasm `f64.nearest` — round to nearest even (banker's rounding).
fn wasm_nearest_f64(v: f64) -> f64 {
    if v.is_nan() || v.is_infinite() || v == 0.0 {
        return v;
    }
    let rounded = v.round();
    if (v - rounded).abs() == 0.5 {
        let truncated = v.trunc();
        if truncated % 2.0 == 0.0 {
            truncated
        } else {
            rounded
        }
    } else {
        rounded
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{BlockId, IrBlock, IrFunction, IrTerminator, TypeIdx, WasmType};

    fn make_func(blocks: Vec<IrBlock>) -> IrFunction {
        IrFunction {
            params: vec![],
            locals: vec![],
            blocks,
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
            needs_host: false,
        }
    }

    fn single_block(instrs: Vec<IrInstr>, term: IrTerminator) -> Vec<IrBlock> {
        vec![IrBlock {
            id: BlockId(0),
            instructions: instrs,
            terminator: term,
        }]
    }

    fn ret_none() -> IrTerminator {
        IrTerminator::Return { value: None }
    }

    // ── Basic constant propagation through Assign ────────────────────────

    #[test]
    fn assign_propagation() {
        // v0 = Const(42); v1 = Assign(v0) → v1 = Const(42)
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(42),
                },
                IrInstr::Assign {
                    dest: VarId(1),
                    src: VarId(0),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[1] {
            IrInstr::Const {
                dest,
                value: IrValue::I32(42),
            } => assert_eq!(*dest, VarId(1)),
            other => panic!("expected Const(v1, 42), got {other:?}"),
        }
    }

    // ── BinOp folding ───────────────────────────────────────────────────

    #[test]
    fn fold_i32_add() {
        // v0 = 10; v1 = 20; v2 = v0 + v1 → v2 = 30
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(10),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(20),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Add,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[2] {
            IrInstr::Const {
                dest,
                value: IrValue::I32(30),
            } => assert_eq!(*dest, VarId(2)),
            other => panic!("expected Const(v2, 30), got {other:?}"),
        }
    }

    #[test]
    fn fold_i32_comparison() {
        // v0 = 5; v1 = 10; v2 = v0 < v1 → v2 = 1
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(5),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(10),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32LtS,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[2] {
            IrInstr::Const {
                value: IrValue::I32(1),
                ..
            } => {}
            other => panic!("expected Const(1), got {other:?}"),
        }
    }

    // ── Div by zero: must NOT fold ──────────────────────────────────────

    #[test]
    fn div_by_zero_not_folded() {
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(10),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(0),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32DivS,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        assert!(
            matches!(&func.blocks[0].instructions[2], IrInstr::BinOp { .. }),
            "div-by-zero must not be folded"
        );
    }

    #[test]
    fn i32_div_s_overflow_not_folded() {
        // i32::MIN / -1 → trap, do NOT fold
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(i32::MIN),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(-1),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32DivS,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        assert!(matches!(
            &func.blocks[0].instructions[2],
            IrInstr::BinOp { .. }
        ));
    }

    #[test]
    fn i32_rem_s_min_neg_one_folds_to_zero() {
        // i32::MIN % -1 = 0 (does NOT trap in Wasm)
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(i32::MIN),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(-1),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32RemS,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[2] {
            IrInstr::Const {
                value: IrValue::I32(0),
                ..
            } => {}
            other => panic!("expected Const(0), got {other:?}"),
        }
    }

    // ── UnOp folding ────────────────────────────────────────────────────

    #[test]
    fn fold_i32_eqz() {
        // v0 = 0; v1 = eqz(v0) → v1 = 1
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(0),
                },
                IrInstr::UnOp {
                    dest: VarId(1),
                    op: UnOp::I32Eqz,
                    operand: VarId(0),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[1] {
            IrInstr::Const {
                value: IrValue::I32(1),
                ..
            } => {}
            other => panic!("expected Const(1), got {other:?}"),
        }
    }

    #[test]
    fn fold_i32_clz() {
        // v0 = 1; v1 = clz(v0) → v1 = 31
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                },
                IrInstr::UnOp {
                    dest: VarId(1),
                    op: UnOp::I32Clz,
                    operand: VarId(0),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[1] {
            IrInstr::Const {
                value: IrValue::I32(31),
                ..
            } => {}
            other => panic!("expected Const(31), got {other:?}"),
        }
    }

    // ── Trapping TruncF must NOT fold ───────────────────────────────────

    #[test]
    fn trunc_f32_nan_not_folded() {
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::F32(f32::NAN),
                },
                IrInstr::UnOp {
                    dest: VarId(1),
                    op: UnOp::I32TruncF32S,
                    operand: VarId(0),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        assert!(
            matches!(&func.blocks[0].instructions[1], IrInstr::UnOp { .. }),
            "trunc(NaN) must not be folded"
        );
    }

    #[test]
    fn trunc_f32_valid_folds() {
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::F32(3.7),
                },
                IrInstr::UnOp {
                    dest: VarId(1),
                    op: UnOp::I32TruncF32S,
                    operand: VarId(0),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[1] {
            IrInstr::Const {
                value: IrValue::I32(3),
                ..
            } => {}
            other => panic!("expected Const(3), got {other:?}"),
        }
    }

    // ── Chain folding (fixpoint) ────────────────────────────────────────

    #[test]
    fn chain_through_assign_then_binop() {
        // v0 = 5; v1 = Assign(v0); v2 = 3; v3 = v1 + v2 → v3 = 8
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(5),
                },
                IrInstr::Assign {
                    dest: VarId(1),
                    src: VarId(0),
                },
                IrInstr::Const {
                    dest: VarId(2),
                    value: IrValue::I32(3),
                },
                IrInstr::BinOp {
                    dest: VarId(3),
                    op: BinOp::I32Add,
                    lhs: VarId(1),
                    rhs: VarId(2),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[3] {
            IrInstr::Const {
                value: IrValue::I32(8),
                ..
            } => {}
            other => panic!("expected Const(8), got {other:?}"),
        }
    }

    // ── Non-constant operand: must NOT fold ─────────────────────────────

    #[test]
    fn non_constant_operand_not_folded() {
        // v0 = param (not const); v1 = 5; v2 = v0 + v1 — v0 unknown
        let mut func = IrFunction {
            params: vec![(VarId(0), WasmType::I32)],
            locals: vec![],
            blocks: single_block(
                vec![
                    IrInstr::Const {
                        dest: VarId(1),
                        value: IrValue::I32(5),
                    },
                    IrInstr::BinOp {
                        dest: VarId(2),
                        op: BinOp::I32Add,
                        lhs: VarId(0),
                        rhs: VarId(1),
                    },
                ],
                ret_none(),
            ),
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
            needs_host: false,
        };
        eliminate(&mut func);
        assert!(
            matches!(&func.blocks[0].instructions[1], IrInstr::BinOp { .. }),
            "BinOp with non-const operand must not be folded"
        );
    }

    // ── Conversion folding ──────────────────────────────────────────────

    #[test]
    fn fold_i32_wrap_i64() {
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I64(0x1_0000_0005),
                },
                IrInstr::UnOp {
                    dest: VarId(1),
                    op: UnOp::I32WrapI64,
                    operand: VarId(0),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[1] {
            IrInstr::Const {
                value: IrValue::I32(5),
                ..
            } => {}
            other => panic!("expected Const(5), got {other:?}"),
        }
    }

    #[test]
    fn fold_reinterpret_roundtrip() {
        // i32 → f32 → i32 via reinterpret should preserve bits
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(0x4048_0000), // f32 bits for 3.125
                },
                IrInstr::UnOp {
                    dest: VarId(1),
                    op: UnOp::F32ReinterpretI32,
                    operand: VarId(0),
                },
                IrInstr::UnOp {
                    dest: VarId(2),
                    op: UnOp::I32ReinterpretF32,
                    operand: VarId(1),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[2] {
            IrInstr::Const {
                value: IrValue::I32(v),
                ..
            } => assert_eq!(*v, 0x4048_0000),
            other => panic!("expected Const(0x40480000), got {other:?}"),
        }
    }

    // ── i32 shift masking ───────────────────────────────────────────────

    #[test]
    fn fold_i32_shl_masks_shift_amount() {
        // Wasm: shift amount masked by 31. shl(1, 33) == shl(1, 1) == 2
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(1),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(33),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Shl,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[2] {
            IrInstr::Const {
                value: IrValue::I32(2),
                ..
            } => {}
            other => panic!("expected Const(2), got {other:?}"),
        }
    }

    // ── Wrapping arithmetic ─────────────────────────────────────────────

    #[test]
    fn fold_i32_wrapping_add() {
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(i32::MAX),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::I32(1),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::I32Add,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[2] {
            IrInstr::Const {
                value: IrValue::I32(v),
                ..
            } => assert_eq!(*v, i32::MIN),
            other => panic!("expected Const(i32::MIN), got {other:?}"),
        }
    }

    // ── f64 folding ─────────────────────────────────────────────────────

    #[test]
    fn fold_f64_mul() {
        let mut func = make_func(single_block(
            vec![
                IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::F64(2.5),
                },
                IrInstr::Const {
                    dest: VarId(1),
                    value: IrValue::F64(4.0),
                },
                IrInstr::BinOp {
                    dest: VarId(2),
                    op: BinOp::F64Mul,
                    lhs: VarId(0),
                    rhs: VarId(1),
                },
            ],
            ret_none(),
        ));
        eliminate(&mut func);
        match &func.blocks[0].instructions[2] {
            IrInstr::Const {
                value: IrValue::F64(v),
                ..
            } => assert!((*v - 10.0).abs() < f64::EPSILON),
            other => panic!("expected Const(10.0), got {other:?}"),
        }
    }

    // ── Multi-block: constants are block-local ──────────────────────────

    #[test]
    fn constant_does_not_leak_across_blocks() {
        // B0: v0 = 5; Jump(B1)
        // B1: v1 = Assign(v0) — v0 is NOT known in B1's local map
        let mut func = make_func(vec![
            IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(5),
                }],
                terminator: IrTerminator::Jump { target: BlockId(1) },
            },
            IrBlock {
                id: BlockId(1),
                instructions: vec![IrInstr::Assign {
                    dest: VarId(1),
                    src: VarId(0),
                }],
                terminator: IrTerminator::Return { value: None },
            },
        ]);
        eliminate(&mut func);
        // The Assign in B1 should NOT be folded because v0 is not known in B1.
        assert!(
            matches!(&func.blocks[1].instructions[0], IrInstr::Assign { .. }),
            "Assign must remain — constant is block-local"
        );
    }

    // ── try_eval_binop unit tests ───────────────────────────────────────

    #[test]
    fn binop_i32_unsigned_comparison() {
        // -1 as u32 > 0 as u32
        assert_eq!(
            try_eval_binop(BinOp::I32GtU, IrValue::I32(-1), IrValue::I32(0)),
            Some(IrValue::I32(1))
        );
    }

    #[test]
    fn binop_i32_rotl() {
        assert_eq!(
            try_eval_binop(
                BinOp::I32Rotl,
                IrValue::I32(0x8000_0001_u32 as i32),
                IrValue::I32(1)
            ),
            Some(IrValue::I32(3))
        );
    }

    #[test]
    fn binop_i64_div_zero() {
        assert_eq!(
            try_eval_binop(BinOp::I64DivS, IrValue::I64(10), IrValue::I64(0)),
            None
        );
    }

    #[test]
    fn binop_i64_div_signed_overflow() {
        assert_eq!(
            try_eval_binop(BinOp::I64DivS, IrValue::I64(i64::MIN), IrValue::I64(-1)),
            None
        );
    }

    #[test]
    fn binop_i64_rem_s_min_neg_one() {
        assert_eq!(
            try_eval_binop(BinOp::I64RemS, IrValue::I64(i64::MIN), IrValue::I64(-1)),
            Some(IrValue::I64(0))
        );
    }

    #[test]
    fn binop_type_mismatch_returns_none() {
        assert_eq!(
            try_eval_binop(BinOp::I32Add, IrValue::I32(1), IrValue::I64(2)),
            None
        );
    }

    // ── try_eval_unop unit tests ────────────────────────────────────────

    #[test]
    fn unop_i64_extend_s() {
        assert_eq!(
            try_eval_unop(UnOp::I64ExtendI32S, IrValue::I32(-1)),
            Some(IrValue::I64(-1))
        );
    }

    #[test]
    fn unop_i64_extend_u() {
        assert_eq!(
            try_eval_unop(UnOp::I64ExtendI32U, IrValue::I32(-1)),
            Some(IrValue::I64(0xFFFF_FFFF))
        );
    }

    #[test]
    fn unop_f32_neg() {
        match try_eval_unop(UnOp::F32Neg, IrValue::F32(1.5)) {
            Some(IrValue::F32(v)) => assert!((v - (-1.5)).abs() < f32::EPSILON),
            other => panic!("expected F32(-1.5), got {other:?}"),
        }
    }

    #[test]
    fn unop_type_mismatch() {
        assert_eq!(try_eval_unop(UnOp::I32Clz, IrValue::I64(1)), None);
    }

    // ── Wasm nearest ────────────────────────────────────────────────────

    #[test]
    fn nearest_f32_bankers_rounding() {
        assert_eq!(wasm_nearest_f32(0.5), 0.0); // round to even (0)
        assert_eq!(wasm_nearest_f32(1.5), 2.0); // round to even (2)
        assert_eq!(wasm_nearest_f32(2.5), 2.0); // round to even (2)
        assert_eq!(wasm_nearest_f32(3.5), 4.0); // round to even (4)
    }

    #[test]
    fn nearest_f64_bankers_rounding() {
        assert_eq!(wasm_nearest_f64(0.5), 0.0);
        assert_eq!(wasm_nearest_f64(1.5), 2.0);
        assert_eq!(wasm_nearest_f64(2.5), 2.0);
    }

    // ── Valid i32 div/rem that SHOULD fold ───────────────────────────────

    #[test]
    fn i32_div_s_valid_folds() {
        assert_eq!(
            try_eval_binop(BinOp::I32DivS, IrValue::I32(10), IrValue::I32(3)),
            Some(IrValue::I32(3))
        );
    }

    #[test]
    fn i32_div_u_valid_folds() {
        // -1 as u32 = u32::MAX; u32::MAX / 2 = 2147483647
        assert_eq!(
            try_eval_binop(BinOp::I32DivU, IrValue::I32(-1), IrValue::I32(2)),
            Some(IrValue::I32(2147483647))
        );
    }

    #[test]
    fn i32_rem_u_valid_folds() {
        assert_eq!(
            try_eval_binop(BinOp::I32RemU, IrValue::I32(10), IrValue::I32(3)),
            Some(IrValue::I32(1))
        );
    }
}
