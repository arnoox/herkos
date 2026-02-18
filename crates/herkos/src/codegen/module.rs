//! Module-level code generation.
//!
//! Handles the two main generation modes:
//! - **Standalone**: generates `pub fn func_N(...)` functions (backwards compatible)
//! - **Module wrapper**: generates a `Module<Globals, MAX_PAGES, 0>` struct with
//!   constructor, internal functions, and exported methods

use crate::backend::Backend;
use crate::ir::*;
use anyhow::{Context, Result};

/// Generate a complete Rust module from IR functions with full module info.
///
/// This is the main entry point for Milestone 4+. It decides between
/// standalone functions and a module wrapper based on `info.needs_wrapper()`.
pub fn generate_module_with_info<B: Backend>(backend: &B, info: &ModuleInfo) -> Result<String> {
    if info.needs_wrapper() {
        generate_wrapper_module(backend, info)
    } else {
        generate_standalone_module(backend, info)
    }
}

/// Generate standalone functions (no module wrapper).
fn generate_standalone_module<B: Backend>(backend: &B, info: &ModuleInfo) -> Result<String> {
    let mut rust_code = crate::codegen::constructor::rust_code_preamble();

    if info.has_memory {
        rust_code.push_str(&format!("const MAX_PAGES: usize = {};\n\n", info.max_pages));
    }

    // Host trait definitions
    rust_code.push_str(&crate::codegen::traits::generate_host_traits(backend, info));

    // Const items for immutable globals
    rust_code.push_str(&crate::codegen::constructor::emit_const_globals(
        backend, info,
    ));

    // Generate each function
    for (idx, ir_func) in info.ir_functions.iter().enumerate() {
        let func_name = format!("func_{}", idx);
        let code = crate::codegen::function::generate_function_with_info(
            backend, ir_func, &func_name, info, true,
        )
        .with_context(|| format!("failed to generate code for function {}", idx))?;
        rust_code.push_str(&code);
        rust_code.push('\n');
    }

    Ok(rust_code)
}

/// Generate a module wrapper with Globals struct, constructor, and export methods.
fn generate_wrapper_module<B: Backend>(backend: &B, info: &ModuleInfo) -> Result<String> {
    let mut rust_code = crate::codegen::constructor::rust_code_preamble();
    let has_mut_globals = info.has_mutable_globals();

    if info.has_memory {
        rust_code.push_str(&format!("const MAX_PAGES: usize = {};\n\n", info.max_pages));
    }

    if info.has_table() {
        rust_code.push_str(&format!("const TABLE_MAX: usize = {};\n", info.table_max));
    }
    rust_code.push('\n');

    // Host trait definitions
    rust_code.push_str(&crate::codegen::traits::generate_host_traits(backend, info));

    // Globals struct (mutable globals only)
    if has_mut_globals {
        rust_code.push_str("pub struct Globals {\n");
        for (idx, g) in info.globals.iter().enumerate() {
            if g.mutable {
                let rust_ty = crate::codegen::types::wasm_type_to_rust(&g.wasm_type);
                rust_code.push_str(&format!("    pub g{idx}: {rust_ty},\n"));
            }
        }
        rust_code.push_str("}\n\n");
    }

    // Const items for immutable globals
    rust_code.push_str(&crate::codegen::constructor::emit_const_globals(
        backend, info,
    ));

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
    rust_code.push_str(&crate::codegen::constructor::generate_constructor(
        backend,
        info,
        has_mut_globals,
    ));
    rust_code.push('\n');

    // Internal functions (private)
    for (idx, ir_func) in info.ir_functions.iter().enumerate() {
        let func_name = format!("func_{}", idx);
        let code = crate::codegen::function::generate_function_with_info(
            backend, ir_func, &func_name, info, false,
        )
        .with_context(|| format!("failed to generate code for function {}", idx))?;
        rust_code.push_str(&code);
        rust_code.push('\n');
    }

    // Export impl block
    if !info.func_exports.is_empty() {
        rust_code.push_str(&crate::codegen::export::generate_export_impl(backend, info));
        rust_code.push('\n');
    }

    Ok(rust_code)
}
