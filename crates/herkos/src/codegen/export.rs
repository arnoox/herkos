//! Export implementation generation.
//!
//! Generates the `impl WasmModule { ... }` block with methods for all functions.
//! Exported functions use their export names, internal functions use func_N names.

use crate::backend::Backend;
use crate::ir::*;

/// Generate the `impl WasmModule { ... }` block with accessor methods for all functions.
pub fn generate_export_impl<B: Backend>(_backend: &B, info: &ModuleInfo) -> String {
    let mut code = String::new();
    let has_mut_globals = info.has_mutable_globals();

    code.push_str("impl WasmModule {\n");

    // Build a map of function index -> export name for quick lookup
    let export_names: std::collections::HashMap<usize, &str> = info
        .func_exports
        .iter()
        .map(|e| (e.func_index.as_usize(), e.name.as_str()))
        .collect();

    // Generate accessor methods for all functions
    for func_idx in 0..info.ir_functions.len() {
        let ir_func = &info.ir_functions[func_idx];

        // Use export name if available, otherwise use func_N
        let method_name = if let Some(export_name) = export_names.get(&func_idx) {
            (*export_name).to_string()
        } else {
            format!("func_{}", func_idx)
        };

        // Determine trait bounds for this export
        let trait_bounds_opt = if ir_func.needs_host {
            crate::codegen::traits::build_trait_bounds(info)
        } else {
            None
        };
        let has_multiple_bounds = trait_bounds_opt.as_ref().is_some_and(|b| b.contains(" + "));

        // Build generics: handle both H (host) and MP (imported memory size)
        let mut generics: Vec<String> = Vec::new();
        if info.has_memory_import {
            generics.push("const MP: usize".to_string());
        }
        if has_multiple_bounds {
            generics.push(format!("H: {}", trait_bounds_opt.as_ref().unwrap()));
        }

        // Method signature with optional generic parameter
        let mut param_parts: Vec<String> = Vec::new();
        param_parts.push("&mut self".to_string());
        for (i, (_, ty)) in ir_func.params.iter().enumerate() {
            let rust_ty = crate::codegen::types::wasm_type_to_rust(ty);
            param_parts.push(format!("v{i}: {rust_ty}"));
        }

        // Add memory parameter if imported
        if info.has_memory_import {
            param_parts.push("memory: &mut IsolatedMemory<MP>".to_string());
        }

        // Add host parameter if function needs it
        if let Some(trait_bounds) = &trait_bounds_opt {
            if has_multiple_bounds {
                // Use generic parameter H
                param_parts.push("host: &mut H".to_string());
            } else {
                // Single trait bound - use impl directly
                param_parts.push(format!("host: &mut impl {trait_bounds}"));
            }
        }

        let return_type = crate::codegen::types::format_return_type(ir_func.return_type.as_ref());

        // Generate method signature (with generics if needed)
        let generic_part = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };

        code.push_str(&format!(
            "    pub fn {}{generic_part}({}) -> {} {{\n",
            method_name,
            param_parts.join(", "),
            return_type
        ));

        // Forward call to internal function
        let mut call_args: Vec<String> =
            (0..ir_func.params.len()).map(|i| format!("v{i}")).collect();

        // Forward host parameter if needed
        if ir_func.needs_host {
            call_args.push("host".to_string());
        }

        if has_mut_globals {
            call_args.push("&mut self.0.globals".to_string());
        }
        if info.has_memory {
            call_args.push("&mut self.0.memory".to_string());
        } else if info.has_memory_import {
            call_args.push("memory".to_string());
        }
        if info.has_table() {
            call_args.push("&self.0.table".to_string());
        }

        code.push_str(&format!(
            "        func_{}({})\n",
            func_idx,
            call_args.join(", ")
        ));
        code.push_str("    }\n");
    }

    code.push_str("}\n");
    code
}
