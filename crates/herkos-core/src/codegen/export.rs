//! Export implementation generation.
//!
//! Generates the `impl WasmModule { ... }` block with methods for all functions.
//! Exported functions are thin wrappers that construct an Env<H> and forward to internal functions.

use crate::backend::Backend;
use crate::ir::*;

/// Generate the `impl WasmModule { ... }` block with accessor methods for all functions.
pub fn generate_export_impl<B: Backend>(_backend: &B, info: &ModuleInfo) -> String {
    let mut code = String::new();
    let has_imports = !info.func_imports.is_empty() || !info.imported_globals.is_empty();

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

        // Build generics
        let mut generics: Vec<String> = Vec::new();
        if info.has_memory_import {
            generics.push("const MP: usize".to_string());
        }
        if has_imports {
            generics.push("H: ModuleHostTrait".to_string());
        }

        // Method signature
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

        // Add host parameter if module has imports
        if has_imports {
            param_parts.push("host: &mut H".to_string());
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

        // Construct Env and forward call to internal function
        if has_imports {
            code.push_str("        let mut env = Env { host, globals: &mut self.0.globals };\n");
        } else {
            code.push_str("        let mut __host = herkos_runtime::NoHost;\n");
            code.push_str("        let mut env = Env { host: &mut __host, globals: &mut self.0.globals };\n");
        }

        // Build call arguments: wasm params + env + memory (if owned) + table
        let mut call_args: Vec<String> =
            (0..ir_func.params.len()).map(|i| format!("v{i}")).collect();
        call_args.push("&mut env".to_string());

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
