//! Safe backend — emits 100% safe Rust with bounds checking.
//!
//! This backend generates code that never uses `unsafe` and always performs
//! runtime bounds checks on memory accesses. All operations return `WasmResult<T>`.

use crate::backend::Backend;
use crate::ir::*;

const INDENT: &str = "                ";

/// Format a function call result assignment.
fn emit_call_result(dest: Option<VarId>, call_expr: &str) -> String {
    match dest {
        Some(d) => format!("{}{} = {};", INDENT, d, call_expr),
        None => format!("{}{};", INDENT, call_expr),
    }
}

/// Emit a f32 const, handling NaN and infinity special values.
fn emit_f32_const(dest: VarId, value: f32) -> String {
    if value.is_nan() {
        format!("{}{dest} = f32::NAN;", INDENT)
    } else if value.is_infinite() {
        if value.is_sign_positive() {
            format!("{}{dest} = f32::INFINITY;", INDENT)
        } else {
            format!("{}{dest} = f32::NEG_INFINITY;", INDENT)
        }
    } else {
        format!("{}{dest} = {value}f32;", INDENT)
    }
}

/// Emit a f64 const, handling NaN and infinity special values.
fn emit_f64_const(dest: VarId, value: f64) -> String {
    if value.is_nan() {
        format!("{}{dest} = f64::NAN;", INDENT)
    } else if value.is_infinite() {
        if value.is_sign_positive() {
            format!("{}{dest} = f64::INFINITY;", INDENT)
        } else {
            format!("{}{dest} = f64::NEG_INFINITY;", INDENT)
        }
    } else {
        format!("{}{dest} = {value}f64;", INDENT)
    }
}

/// Safe code generation backend.
pub struct SafeBackend;

impl SafeBackend {
    pub fn new() -> Self {
        SafeBackend
    }
}

impl Default for SafeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for SafeBackend {
    fn emit_const(&self, dest: VarId, value: &IrValue) -> String {
        match value {
            IrValue::I32(v) => format!("                {dest} = {v}i32;"),
            IrValue::I64(v) => format!("                {dest} = {v}i64;"),
            IrValue::F32(v) => emit_f32_const(dest, *v),
            IrValue::F64(v) => emit_f64_const(dest, *v),
        }
    }

    fn emit_binop(&self, dest: VarId, op: BinOp, lhs: VarId, rhs: VarId) -> String {
        let rust_op = match op {
            // i32 arithmetic - Wasm uses wrapping semantics
            BinOp::I32Add => return format!("                {dest} = {lhs}.wrapping_add({rhs});"),
            BinOp::I32Sub => return format!("                {dest} = {lhs}.wrapping_sub({rhs});"),
            BinOp::I32Mul => return format!("                {dest} = {lhs}.wrapping_mul({rhs});"),
            BinOp::I32DivS => {
                // Signed division can trap on overflow or division by zero
                return format!(
                    "                {dest} = {lhs}.checked_div({rhs}).ok_or(WasmTrap::DivisionByZero)?;"
                );
            }
            BinOp::I32DivU => {
                // Unsigned division: reinterpret as unsigned
                return format!(
                    "                {dest} = ({lhs} as u32).checked_div({rhs} as u32).ok_or(WasmTrap::DivisionByZero)? as i32;"
                );
            }
            BinOp::I32RemS => {
                return format!(
                    "                {dest} = {lhs}.checked_rem({rhs}).ok_or(WasmTrap::DivisionByZero)?;"
                );
            }
            BinOp::I32RemU => {
                return format!(
                    "                {dest} = ({lhs} as u32).checked_rem({rhs} as u32).ok_or(WasmTrap::DivisionByZero)? as i32;"
                );
            }
            BinOp::I32And => return format!("                {dest} = {lhs} & {rhs};"),
            BinOp::I32Or => return format!("                {dest} = {lhs} | {rhs};"),
            BinOp::I32Xor => return format!("                {dest} = {lhs} ^ {rhs};"),
            BinOp::I32Shl => return format!("                {dest} = {lhs}.wrapping_shl(({rhs} & 31) as u32);"),
            BinOp::I32ShrS => return format!("                {dest} = {lhs}.wrapping_shr(({rhs} & 31) as u32);"),
            BinOp::I32ShrU => {
                return format!("                {dest} = ({lhs} as u32).wrapping_shr(({rhs} & 31) as u32) as i32;")
            }
            BinOp::I32Rotl => return format!("                {dest} = {lhs}.rotate_left(({rhs} & 31) as u32);"),
            BinOp::I32Rotr => return format!("                {dest} = {lhs}.rotate_right(({rhs} & 31) as u32);"),

            // i32 comparisons
            BinOp::I32Eq => "==",
            BinOp::I32Ne => "!=",
            BinOp::I32LtS => "<",
            BinOp::I32LtU => {
                return format!("                {dest} = if ({lhs} as u32) < ({rhs} as u32) {{ 1 }} else {{ 0 }};")
            }
            BinOp::I32GtS => ">",
            BinOp::I32GtU => {
                return format!("                {dest} = if ({lhs} as u32) > ({rhs} as u32) {{ 1 }} else {{ 0 }};")
            }
            BinOp::I32LeS => "<=",
            BinOp::I32LeU => {
                return format!("                {dest} = if ({lhs} as u32) <= ({rhs} as u32) {{ 1 }} else {{ 0 }};")
            }
            BinOp::I32GeS => ">=",
            BinOp::I32GeU => {
                return format!("                {dest} = if ({lhs} as u32) >= ({rhs} as u32) {{ 1 }} else {{ 0 }};")
            }

            // i64 arithmetic (same pattern as i32)
            BinOp::I64Add => return format!("                {dest} = {lhs}.wrapping_add({rhs});"),
            BinOp::I64Sub => return format!("                {dest} = {lhs}.wrapping_sub({rhs});"),
            BinOp::I64Mul => return format!("                {dest} = {lhs}.wrapping_mul({rhs});"),
            BinOp::I64DivS => {
                return format!(
                    "                {dest} = {lhs}.checked_div({rhs}).ok_or(WasmTrap::DivisionByZero)?;"
                );
            }
            BinOp::I64DivU => {
                return format!(
                    "                {dest} = ({lhs} as u64).checked_div({rhs} as u64).ok_or(WasmTrap::DivisionByZero)? as i64;"
                );
            }
            BinOp::I64RemS => {
                return format!(
                    "                {dest} = {lhs}.checked_rem({rhs}).ok_or(WasmTrap::DivisionByZero)?;"
                );
            }
            BinOp::I64RemU => {
                return format!(
                    "                {dest} = ({lhs} as u64).checked_rem({rhs} as u64).ok_or(WasmTrap::DivisionByZero)? as i64;"
                );
            }
            BinOp::I64And => return format!("                {dest} = {lhs} & {rhs};"),
            BinOp::I64Or => return format!("                {dest} = {lhs} | {rhs};"),
            BinOp::I64Xor => return format!("                {dest} = {lhs} ^ {rhs};"),
            BinOp::I64Shl => return format!("                {dest} = {lhs}.wrapping_shl(({rhs} & 63) as u32);"),
            BinOp::I64ShrS => return format!("                {dest} = {lhs}.wrapping_shr(({rhs} & 63) as u32);"),
            BinOp::I64ShrU => {
                return format!("                {dest} = ({lhs} as u64).wrapping_shr(({rhs} & 63) as u32) as i64;")
            }
            BinOp::I64Rotl => return format!("                {dest} = {lhs}.rotate_left(({rhs} & 63) as u32);"),
            BinOp::I64Rotr => return format!("                {dest} = {lhs}.rotate_right(({rhs} & 63) as u32);"),

            // i64 comparisons
            BinOp::I64Eq => {
                return format!("                {dest} = if {lhs} == {rhs} {{ 1i32 }} else {{ 0i32 }};")
            }
            BinOp::I64Ne => {
                return format!("                {dest} = if {lhs} != {rhs} {{ 1i32 }} else {{ 0i32 }};")
            }
            BinOp::I64LtS => {
                return format!("                {dest} = if {lhs} < {rhs} {{ 1i32 }} else {{ 0i32 }};")
            }
            BinOp::I64LtU => {
                return format!("                {dest} = if ({lhs} as u64) < ({rhs} as u64) {{ 1i32 }} else {{ 0i32 }};")
            }
            BinOp::I64GtS => {
                return format!("                {dest} = if {lhs} > {rhs} {{ 1i32 }} else {{ 0i32 }};")
            }
            BinOp::I64GtU => {
                return format!("                {dest} = if ({lhs} as u64) > ({rhs} as u64) {{ 1i32 }} else {{ 0i32 }};")
            }
            BinOp::I64LeS => {
                return format!("                {dest} = if {lhs} <= {rhs} {{ 1i32 }} else {{ 0i32 }};")
            }
            BinOp::I64LeU => {
                return format!("                {dest} = if ({lhs} as u64) <= ({rhs} as u64) {{ 1i32 }} else {{ 0i32 }};")
            }
            BinOp::I64GeS => {
                return format!("                {dest} = if {lhs} >= {rhs} {{ 1i32 }} else {{ 0i32 }};")
            }
            BinOp::I64GeU => {
                return format!("                {dest} = if ({lhs} as u64) >= ({rhs} as u64) {{ 1i32 }} else {{ 0i32 }};")
            }

            // f32/f64 arithmetic (no wrapping needed)
            BinOp::F32Add => "+",
            BinOp::F32Sub => "-",
            BinOp::F32Mul => "*",
            BinOp::F32Div => "/",
            BinOp::F32Min => return format!("                {dest} = {lhs}.min({rhs});"),
            BinOp::F32Max => return format!("                {dest} = {lhs}.max({rhs});"),
            BinOp::F32Copysign => return format!("                {dest} = {lhs}.copysign({rhs});"),

            BinOp::F64Add => "+",
            BinOp::F64Sub => "-",
            BinOp::F64Mul => "*",
            BinOp::F64Div => "/",
            BinOp::F64Min => return format!("                {dest} = {lhs}.min({rhs});"),
            BinOp::F64Max => return format!("                {dest} = {lhs}.max({rhs});"),
            BinOp::F64Copysign => return format!("                {dest} = {lhs}.copysign({rhs});"),

            // Float comparisons
            BinOp::F32Eq => "==",
            BinOp::F32Ne => "!=",
            BinOp::F32Lt => "<",
            BinOp::F32Gt => ">",
            BinOp::F32Le => "<=",
            BinOp::F32Ge => ">=",

            BinOp::F64Eq => "==",
            BinOp::F64Ne => "!=",
            BinOp::F64Lt => "<",
            BinOp::F64Gt => ">",
            BinOp::F64Le => "<=",
            BinOp::F64Ge => ">=",
        };

        // Comparisons return i32 (bool → 0/1), arithmetic returns the type directly
        if matches!(
            op,
            BinOp::I32Eq
                | BinOp::I32Ne
                | BinOp::I32LtS
                | BinOp::I32GtS
                | BinOp::I32LeS
                | BinOp::I32GeS
                | BinOp::F32Eq
                | BinOp::F32Ne
                | BinOp::F32Lt
                | BinOp::F32Gt
                | BinOp::F32Le
                | BinOp::F32Ge
                | BinOp::F64Eq
                | BinOp::F64Ne
                | BinOp::F64Lt
                | BinOp::F64Gt
                | BinOp::F64Le
                | BinOp::F64Ge
        ) {
            format!("                {dest} = if {lhs} {rust_op} {rhs} {{ 1i32 }} else {{ 0i32 }};")
        } else {
            format!("                {dest} = {lhs} {rust_op} {rhs};")
        }
    }

    fn emit_unop(&self, dest: VarId, op: UnOp, operand: VarId) -> String {
        match op {
            UnOp::I32Clz => format!("                {dest} = {operand}.leading_zeros() as i32;"),
            UnOp::I32Ctz => format!("                {dest} = {operand}.trailing_zeros() as i32;"),
            UnOp::I32Popcnt => format!("                {dest} = {operand}.count_ones() as i32;"),
            UnOp::I32Eqz => {
                format!("                {dest} = if {operand} == 0 {{ 1 }} else {{ 0 }};")
            }

            UnOp::I64Eqz => {
                format!("                {dest} = if {operand} == 0 {{ 1i32 }} else {{ 0i32 }};")
            }
            UnOp::I64Clz => format!("                {dest} = {operand}.leading_zeros() as i64;"),
            UnOp::I64Ctz => format!("                {dest} = {operand}.trailing_zeros() as i64;"),
            UnOp::I64Popcnt => format!("                {dest} = {operand}.count_ones() as i64;"),

            UnOp::F32Abs => format!("                {dest} = {operand}.abs();"),
            UnOp::F32Neg => format!("                {dest} = -{operand};"),
            UnOp::F32Sqrt => format!("                {dest} = {operand}.sqrt();"),
            UnOp::F32Ceil => format!("                {dest} = {operand}.ceil();"),
            UnOp::F32Floor => format!("                {dest} = {operand}.floor();"),
            UnOp::F32Trunc => format!("                {dest} = {operand}.trunc();"),
            UnOp::F32Nearest => format!("                {dest} = {operand}.round_ties_even();"),

            UnOp::F64Abs => format!("                {dest} = {operand}.abs();"),
            UnOp::F64Neg => format!("                {dest} = -{operand};"),
            UnOp::F64Sqrt => format!("                {dest} = {operand}.sqrt();"),
            UnOp::F64Ceil => format!("                {dest} = {operand}.ceil();"),
            UnOp::F64Floor => format!("                {dest} = {operand}.floor();"),
            UnOp::F64Trunc => format!("                {dest} = {operand}.trunc();"),
            UnOp::F64Nearest => format!("                {dest} = {operand}.round_ties_even();"),

            // === Conversion operations ===

            // Integer truncation/extension
            UnOp::I32WrapI64 => format!("                {dest} = {operand} as i32;"),
            UnOp::I64ExtendI32S => format!("                {dest} = {operand} as i64;"),
            UnOp::I64ExtendI32U => format!("                {dest} = ({operand} as u32) as i64;"),

            // Float → i32 (trapping on NaN/overflow)
            UnOp::I32TruncF32S => {
                format!("                if {operand}.is_nan() || {operand} >= 2147483648.0f32 || {operand} < -2147483648.0f32 {{ return Err(WasmTrap::IntegerOverflow); }} {dest} = {operand} as i32;")
            }
            UnOp::I32TruncF32U => {
                format!("                if {operand}.is_nan() || {operand} >= 4294967296.0f32 || {operand} <= -1.0f32 {{ return Err(WasmTrap::IntegerOverflow); }} {dest} = {operand} as u32 as i32;")
            }
            UnOp::I32TruncF64S => {
                format!("                if {operand}.is_nan() || {operand} >= 2147483648.0f64 || {operand} < -2147483648.0f64 {{ return Err(WasmTrap::IntegerOverflow); }} {dest} = {operand} as i32;")
            }
            UnOp::I32TruncF64U => {
                format!("                if {operand}.is_nan() || {operand} >= 4294967296.0f64 || {operand} <= -1.0f64 {{ return Err(WasmTrap::IntegerOverflow); }} {dest} = {operand} as u32 as i32;")
            }

            // Float → i64 (trapping on NaN/overflow)
            UnOp::I64TruncF32S => {
                format!("                if {operand}.is_nan() || {operand} >= 9223372036854775808.0f32 || {operand} < -9223372036854775808.0f32 {{ return Err(WasmTrap::IntegerOverflow); }} {dest} = {operand} as i64;")
            }
            UnOp::I64TruncF32U => {
                format!("                if {operand}.is_nan() || {operand} >= 18446744073709551616.0f32 || {operand} <= -1.0f32 {{ return Err(WasmTrap::IntegerOverflow); }} {dest} = {operand} as u64 as i64;")
            }
            UnOp::I64TruncF64S => {
                format!("                if {operand}.is_nan() || {operand} >= 9223372036854775808.0f64 || {operand} < -9223372036854775808.0f64 {{ return Err(WasmTrap::IntegerOverflow); }} {dest} = {operand} as i64;")
            }
            UnOp::I64TruncF64U => {
                format!("                if {operand}.is_nan() || {operand} >= 18446744073709551616.0f64 || {operand} <= -1.0f64 {{ return Err(WasmTrap::IntegerOverflow); }} {dest} = {operand} as u64 as i64;")
            }

            // Integer → float
            UnOp::F32ConvertI32S => format!("                {dest} = {operand} as f32;"),
            UnOp::F32ConvertI32U => format!("                {dest} = ({operand} as u32) as f32;"),
            UnOp::F32ConvertI64S => format!("                {dest} = {operand} as f32;"),
            UnOp::F32ConvertI64U => format!("                {dest} = ({operand} as u64) as f32;"),
            UnOp::F64ConvertI32S => format!("                {dest} = {operand} as f64;"),
            UnOp::F64ConvertI32U => format!("                {dest} = ({operand} as u32) as f64;"),
            UnOp::F64ConvertI64S => format!("                {dest} = {operand} as f64;"),
            UnOp::F64ConvertI64U => format!("                {dest} = ({operand} as u64) as f64;"),

            // Float precision
            UnOp::F32DemoteF64 => format!("                {dest} = {operand} as f32;"),
            UnOp::F64PromoteF32 => format!("                {dest} = {operand} as f64;"),

            // Reinterpretations (bitcast)
            UnOp::I32ReinterpretF32 => {
                format!("                {dest} = {operand}.to_bits() as i32;")
            }
            UnOp::I64ReinterpretF64 => {
                format!("                {dest} = {operand}.to_bits() as i64;")
            }
            UnOp::F32ReinterpretI32 => {
                format!("                {dest} = f32::from_bits({operand} as u32);")
            }
            UnOp::F64ReinterpretI64 => {
                format!("                {dest} = f64::from_bits({operand} as u64);")
            }
        }
    }

    fn emit_load(
        &self,
        dest: VarId,
        ty: WasmType,
        addr: VarId,
        offset: u32,
        width: MemoryAccessWidth,
        sign: Option<SignExtension>,
    ) -> String {
        let addr_expr = if offset > 0 {
            format!("({addr} as usize).wrapping_add({offset} as usize)")
        } else {
            format!("{addr} as usize")
        };

        let load_expr = match (ty, width, sign) {
            // Full-width loads
            (WasmType::I32, MemoryAccessWidth::Full, _) => {
                format!("memory.load_i32({addr_expr})?")
            }
            (WasmType::I64, MemoryAccessWidth::Full, _) => {
                format!("memory.load_i64({addr_expr})?")
            }
            (WasmType::F32, MemoryAccessWidth::Full, _) => {
                format!("memory.load_f32({addr_expr})?")
            }
            (WasmType::F64, MemoryAccessWidth::Full, _) => {
                format!("memory.load_f64({addr_expr})?")
            }

            // i32.load8_s / i32.load8_u
            (WasmType::I32, MemoryAccessWidth::I8, Some(SignExtension::Signed)) => {
                format!("memory.load_u8({addr_expr})? as i8 as i32")
            }
            (WasmType::I32, MemoryAccessWidth::I8, Some(SignExtension::Unsigned)) => {
                format!("memory.load_u8({addr_expr})? as i32")
            }

            // i32.load16_s / i32.load16_u
            (WasmType::I32, MemoryAccessWidth::I16, Some(SignExtension::Signed)) => {
                format!("memory.load_u16({addr_expr})? as i16 as i32")
            }
            (WasmType::I32, MemoryAccessWidth::I16, Some(SignExtension::Unsigned)) => {
                format!("memory.load_u16({addr_expr})? as i32")
            }

            // i64.load8_s / i64.load8_u
            (WasmType::I64, MemoryAccessWidth::I8, Some(SignExtension::Signed)) => {
                format!("memory.load_u8({addr_expr})? as i8 as i64")
            }
            (WasmType::I64, MemoryAccessWidth::I8, Some(SignExtension::Unsigned)) => {
                format!("memory.load_u8({addr_expr})? as i64")
            }

            // i64.load16_s / i64.load16_u
            (WasmType::I64, MemoryAccessWidth::I16, Some(SignExtension::Signed)) => {
                format!("memory.load_u16({addr_expr})? as i16 as i64")
            }
            (WasmType::I64, MemoryAccessWidth::I16, Some(SignExtension::Unsigned)) => {
                format!("memory.load_u16({addr_expr})? as i64")
            }

            // i64.load32_s / i64.load32_u
            (WasmType::I64, MemoryAccessWidth::I32, Some(SignExtension::Signed)) => {
                format!("memory.load_i32({addr_expr})? as i64")
            }
            (WasmType::I64, MemoryAccessWidth::I32, Some(SignExtension::Unsigned)) => {
                format!("memory.load_i32({addr_expr})? as u32 as i64")
            }

            // Invalid combinations (shouldn't occur from valid Wasm)
            // TODO: return an error instead of silently ignoring invalid stores?
            _ => "0 /* unsupported load width */".to_string(),
        };

        format!("                {dest} = {load_expr};")
    }

    fn emit_store(
        &self,
        ty: WasmType,
        addr: VarId,
        value: VarId,
        offset: u32,
        width: MemoryAccessWidth,
    ) -> String {
        let addr_expr = if offset > 0 {
            format!("({addr} as usize).wrapping_add({offset} as usize)")
        } else {
            format!("{addr} as usize")
        };

        let store_call = match (ty, width) {
            // Full-width stores
            (WasmType::I32, MemoryAccessWidth::Full) => {
                format!("memory.store_i32({addr_expr}, {value})?")
            }
            (WasmType::I64, MemoryAccessWidth::Full) => {
                format!("memory.store_i64({addr_expr}, {value})?")
            }
            (WasmType::F32, MemoryAccessWidth::Full) => {
                format!("memory.store_f32({addr_expr}, {value})?")
            }
            (WasmType::F64, MemoryAccessWidth::Full) => {
                format!("memory.store_f64({addr_expr}, {value})?")
            }

            // i32.store8 / i64.store8
            (WasmType::I32 | WasmType::I64, MemoryAccessWidth::I8) => {
                format!("memory.store_u8({addr_expr}, {value} as u8)?")
            }

            // i32.store16 / i64.store16
            (WasmType::I32 | WasmType::I64, MemoryAccessWidth::I16) => {
                format!("memory.store_u16({addr_expr}, {value} as u16)?")
            }

            // i64.store32
            (WasmType::I64, MemoryAccessWidth::I32) => {
                format!("memory.store_i32({addr_expr}, {value} as i32)?")
            }

            // Invalid combinations
            // TODO: return an error instead of silently ignoring invalid stores?
            _ => "() /* unsupported store width */".to_string(),
        };

        format!("                {store_call};")
    }

    fn emit_call(
        &self,
        dest: Option<VarId>,
        func_idx: usize,
        args: &[VarId],
        has_globals: bool,
        has_memory: bool,
        has_table: bool,
    ) -> String {
        let mut call_args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        if has_globals {
            call_args.push("globals".to_string());
        }
        if has_memory {
            call_args.push("memory".to_string());
        }
        if has_table {
            call_args.push("table".to_string());
        }
        let call_expr = format!("func_{}({})?", func_idx, call_args.join(", "));
        emit_call_result(dest, &call_expr)
    }

    fn emit_call_import(
        &self,
        dest: Option<VarId>,
        _module_name: &str,
        func_name: &str,
        args: &[VarId],
    ) -> String {
        // Generate: host.func_name(args)?
        // Note: module_name is ignored for now (Milestone 3 will use it for trait names)
        let args_str: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        let call_expr = format!("host.{}({})?", func_name, args_str.join(", "));
        emit_call_result(dest, &call_expr)
    }

    fn emit_global_get(&self, dest: VarId, index: usize, is_mutable: bool) -> String {
        if is_mutable {
            format!("                {dest} = globals.g{index};")
        } else {
            format!("                {dest} = G{index};")
        }
    }

    fn emit_global_set(&self, index: usize, value: VarId) -> String {
        format!("                globals.g{index} = {value};")
    }

    fn emit_assign(&self, dest: VarId, src: VarId) -> String {
        format!("                {dest} = {src};")
    }

    fn emit_select(&self, dest: VarId, val1: VarId, val2: VarId, condition: VarId) -> String {
        format!("                {dest} = if {condition} != 0 {{ {val1} }} else {{ {val2} }};")
    }

    fn emit_return(&self, value: Option<VarId>) -> String {
        match value {
            Some(v) => format!("                return Ok({v});"),
            None => "                return Ok(());".to_string(),
        }
    }

    fn emit_memory_size(&self, dest: VarId) -> String {
        format!("                {dest} = memory.size();")
    }

    fn emit_memory_grow(&self, dest: VarId, delta: VarId) -> String {
        format!("                {dest} = memory.grow({delta} as u32);")
    }

    fn emit_unreachable(&self) -> String {
        "    return Err(WasmTrap::Unreachable);".to_string()
    }

    fn emit_jump_to_index(&self, target_idx: usize) -> String {
        format!(
            "                __current_block = Block::B{};\n                continue;",
            target_idx
        )
    }

    fn emit_branch_if_to_index(
        &self,
        condition: VarId,
        if_true_idx: usize,
        if_false_idx: usize,
    ) -> String {
        format!(
            "                if {condition} != 0 {{\n                    __current_block = Block::B{};\n                }} else {{\n                    __current_block = Block::B{};\n                }}\n                continue;",
            if_true_idx, if_false_idx
        )
    }

    fn emit_branch_table_to_index(
        &self,
        index: VarId,
        target_indices: &[usize],
        default_idx: usize,
    ) -> String {
        if target_indices.is_empty() {
            // No targets, always jump to default
            return format!(
                "                __current_block = Block::B{};\n                continue;",
                default_idx
            );
        }

        let mut code = format!("                __current_block = match {index} as usize {{\n");

        for (i, target_idx) in target_indices.iter().enumerate() {
            code.push_str(&format!(
                "                    {} => Block::B{},\n",
                i, target_idx
            ));
        }

        code.push_str(&format!(
            "                    _ => Block::B{},\n",
            default_idx
        ));
        code.push_str("                };\n");
        code.push_str("                continue;");

        code
    }
}
