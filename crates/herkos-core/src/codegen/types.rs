//! Type conversion utilities for generating Rust type strings from WebAssembly types.

use crate::ir::*;

/// Convert WasmType to Rust type string.
pub fn wasm_type_to_rust(ty: &WasmType) -> &'static str {
    match ty {
        WasmType::I32 => "i32",
        WasmType::I64 => "i64",
        WasmType::F32 => "f32",
        WasmType::F64 => "f64",
    }
}

/// Format a Wasm return type as a Rust WasmResult type.
///
/// Examples:
/// - `Some(I32)` → `"WasmResult<i32>"`
/// - `None` → `"WasmResult<()>"`
pub fn format_return_type(ty: Option<&WasmType>) -> String {
    match ty {
        Some(t) => format!("WasmResult<{}>", wasm_type_to_rust(t)),
        None => "WasmResult<()>".to_string(),
    }
}

/// Convert a GlobalInit to (Rust type string, value literal string).
pub fn global_init_to_rust(init: &GlobalInit) -> (&'static str, String) {
    let ty = init.ty();
    let rust_ty = wasm_type_to_rust(&ty);
    let value = match init {
        GlobalInit::I32(v) => format!("{v}i32"),
        GlobalInit::I64(v) => format!("{v}i64"),
        GlobalInit::F32(v) => format!("{v}f32"),
        GlobalInit::F64(v) => format!("{v}f64"),
    };
    (rust_ty, value)
}
