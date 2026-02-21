//! Module-level code generation.
//!
//! Generates a `Module<Globals, MAX_PAGES, 0>` or `LibraryModule<Globals, 0>` struct
//! with constructor, internal functions, and exported methods.

use crate::backend::Backend;
use crate::codegen::constructor::{emit_const_globals, generate_constructor, rust_code_preamble};
use crate::codegen::export::generate_export_impl;
use crate::codegen::function::generate_function_with_info;
use crate::codegen::traits::generate_host_traits;
use crate::codegen::types::wasm_type_to_rust;
use crate::ir::*;
use anyhow::{Context, Result};

/// Generate a complete Rust module from IR functions with full module info.
///
/// This is the main entry point. It generates a module wrapper structure.
pub fn generate_module_with_info<B: Backend>(backend: &B, info: &ModuleInfo) -> Result<String> {
    generate_wrapper_module(backend, info)
}

/// Generate a module wrapper with Globals struct, constructor, and export methods.
fn generate_wrapper_module<B: Backend>(backend: &B, info: &ModuleInfo) -> Result<String> {
    let mut rust_code = rust_code_preamble();
    let has_mut_globals = info.has_mutable_globals();

    if info.has_memory {
        rust_code.push_str(&format!("const MAX_PAGES: usize = {};\n", info.max_pages));
    }

    if info.has_table() {
        rust_code.push_str(&format!("const TABLE_MAX: usize = {};\n", info.table_max));
    }
    rust_code.push('\n');

    // Host trait definitions
    rust_code.push_str(&generate_host_traits(backend, info));

    // Globals struct (mutable globals only)
    if has_mut_globals {
        rust_code.push_str("pub struct Globals {\n");
        for (idx, g) in info.globals.iter().enumerate() {
            if g.mutable {
                let rust_ty = wasm_type_to_rust(&g.init_value.ty());
                rust_code.push_str(&format!("    pub g{idx}: {rust_ty},\n"));
            }
        }
        rust_code.push_str("}\n\n");
    }

    // Const items for immutable globals
    rust_code.push_str(&emit_const_globals(backend, info));

    // Newtype wrapper struct (required to allow `impl WasmModule` on a foreign type)
    let globals_type = if has_mut_globals { "Globals" } else { "()" };
    let table_size_str = if info.has_table() { "TABLE_MAX" } else { "0" };
    if info.has_memory {
        rust_code.push_str(&format!(
            "pub struct WasmModule(pub Module<{globals_type}, MAX_PAGES, {table_size_str}>);\n\n"
        ));
    } else {
        rust_code.push_str(&format!(
            "pub struct WasmModule(pub LibraryModule<{globals_type}, {table_size_str}>);\n\n"
        ));
    }

    // Constructor (standalone for backwards compatibility)
    rust_code.push_str(&generate_constructor(backend, info, has_mut_globals)?);
    rust_code.push('\n');

    // Internal functions (private)
    for (idx, ir_func) in info.ir_functions.iter().enumerate() {
        let func_name = format!("func_{}", idx);
        let code = generate_function_with_info(backend, ir_func, &func_name, info, false)
            .with_context(|| format!("failed to generate code for function {}", idx))?;
        rust_code.push_str(&code);
        rust_code.push('\n');
    }

    // Impl block with accessor methods for all functions
    if !info.ir_functions.is_empty() {
        rust_code.push_str(&generate_export_impl(backend, info));
        rust_code.push('\n');
    }

    Ok(rust_code)
}
