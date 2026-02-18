//! Export implementation generation.
//!
//! Generates the `impl WasmModule { ... }` block with exported function methods
//! that forward calls to internal functions while managing shared state.

use crate::backend::Backend;
use crate::ir::*;

/// Generate the `impl WasmModule { ... }` block with export methods.
pub fn generate_export_impl<B: Backend>(_backend: &B, info: &ModuleInfo) -> String {
    let mut code = String::new();
    let has_mut_globals = info.has_mutable_globals();

    code.push_str("impl WasmModule {\n");

    for export in &info.func_exports {
        let func_idx = export.func_index;
        let sig = &info.func_signatures[func_idx];

        // Determine trait bounds for this export
        let trait_bounds_opt = if sig.needs_host {
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
        for (i, ty) in sig.params.iter().enumerate() {
            let rust_ty = crate::codegen::types::wasm_type_to_rust(ty);
            param_parts.push(format!("v{i}: {rust_ty}"));
        }

        // Add memory parameter if imported
        if info.has_memory_import {
            param_parts.push("memory: &mut IsolatedMemory<MP>".to_string());
        }

        // Add host parameter if function needs it
        if sig.needs_host {
            if let Some(trait_bounds) = &trait_bounds_opt {
                if has_multiple_bounds {
                    // Use generic parameter H
                    param_parts.push("host: &mut H".to_string());
                } else {
                    // Single trait bound - use impl directly
                    param_parts.push(format!("host: &mut impl {trait_bounds}"));
                }
            } else {
                // Fallback for backwards compatibility
                param_parts.push("host: &mut impl Host".to_string());
            }
        }

        let return_type = crate::codegen::types::format_return_type(sig.return_type.as_ref());

        // Generate method signature (with generics if needed)
        let generic_part = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };

        code.push_str(&format!(
            "    pub fn {}{generic_part}({}) -> {} {{\n",
            export.name,
            param_parts.join(", "),
            return_type
        ));

        // Forward call to internal function
        let mut call_args: Vec<String> = (0..sig.params.len()).map(|i| format!("v{i}")).collect();

        // Forward host parameter if needed
        if sig.needs_host {
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
            export.func_index,
            call_args.join(", ")
        ));
        code.push_str("    }\n");
    }

    code.push_str("}\n");
    code
}
