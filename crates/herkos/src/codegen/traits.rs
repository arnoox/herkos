//! Host trait generation from WebAssembly imports.
//!
//! Generates Rust trait definitions for imported functions and globals,
//! organized by module name. Also provides helper functions for building
//! trait bounds and grouping imports by module.

use crate::backend::Backend;
use crate::ir::*;
use std::collections::HashMap;

/// Convert a module name to a Rust trait name.
///
/// Examples:
/// - "env" → "EnvImports"
/// - "wasi_snapshot_preview1" → "WasiSnapshotPreview1Imports"
pub fn module_name_to_trait_name(module_name: &str) -> String {
    use heck::ToUpperCamelCase;
    format!("{}Imports", module_name.to_upper_camel_case())
}

/// Group imports by module name.
pub fn group_by_module<T, F>(imports: &[T], get_module: F) -> HashMap<String, Vec<&T>>
where
    F: Fn(&T) -> &str,
{
    let mut grouped: HashMap<String, Vec<&T>> = HashMap::new();
    for imp in imports {
        grouped
            .entry(get_module(imp).to_string())
            .or_default()
            .push(imp);
    }
    grouped
}

/// Collect all module names from both function and global imports.
pub fn all_import_module_names(info: &ModuleInfo) -> Vec<String> {
    let mut modules: Vec<String> = Vec::new();

    for imp in &info.func_imports {
        if !modules.contains(&imp.module_name) {
            modules.push(imp.module_name.clone());
        }
    }

    for glob in &info.imported_globals {
        if !modules.contains(&glob.module_name) {
            modules.push(glob.module_name.clone());
        }
    }

    modules
}

/// Generate host trait definitions from imports.
///
/// Includes both function imports and global import accessors.
/// Returns an empty string if there are no imports.
pub fn generate_host_traits<B: Backend>(_backend: &B, info: &ModuleInfo) -> String {
    if info.func_imports.is_empty() && info.imported_globals.is_empty() {
        return String::new();
    }

    let mut code = String::new();

    // Group function imports by module name
    let func_grouped = group_by_module(&info.func_imports, |i| &i.module_name);

    // Group global imports by module name
    let global_grouped = group_by_module(&info.imported_globals, |g| &g.module_name);

    // Collect all module names
    let all_modules = all_import_module_names(info);

    // Generate one trait per module
    for module_name in &all_modules {
        let trait_name = module_name_to_trait_name(module_name);
        code.push_str(&format!("pub trait {trait_name} {{\n"));

        // Function imports for this module
        if let Some(imports) = func_grouped.get(module_name) {
            for imp in imports {
                // Generate method signature
                let mut params: Vec<String> = Vec::new();
                params.push("&mut self".to_string());
                for (i, ty) in imp.params.iter().enumerate() {
                    let rust_ty = crate::codegen::types::wasm_type_to_rust(ty);
                    params.push(format!("arg{i}: {rust_ty}"));
                }

                let return_ty = crate::codegen::types::format_return_type(imp.return_type.as_ref());

                code.push_str(&format!(
                    "    fn {}({}) -> {};\n",
                    imp.func_name,
                    params.join(", "),
                    return_ty
                ));
            }
        }

        // Global import accessors for this module
        if let Some(globals) = global_grouped.get(module_name) {
            for g in globals {
                let rust_ty = crate::codegen::types::wasm_type_to_rust(&g.wasm_type);

                // Getter (always)
                code.push_str(&format!("    fn get_{}(&self) -> {rust_ty};\n", g.name));

                // Setter (only if mutable)
                if g.mutable {
                    code.push_str(&format!(
                        "    fn set_{}(&mut self, val: {rust_ty});\n",
                        g.name
                    ));
                }
            }
        }

        code.push_str("}\n\n");
    }

    code
}

/// Build trait bounds string from imports (e.g., "EnvImports + WasiImports").
pub fn build_trait_bounds(info: &ModuleInfo) -> Option<String> {
    if info.func_imports.is_empty() && info.imported_globals.is_empty() {
        return None;
    }

    let module_names = all_import_module_names(info);
    let trait_names: Vec<String> = module_names
        .iter()
        .map(|module_name| module_name_to_trait_name(module_name))
        .collect();

    Some(trait_names.join(" + "))
}
