//! Environment struct generation.
//!
//! Generates the uniform `Env<H>` context struct that bundles host + globals,
//! along with the `ModuleHostTrait` and `Globals` struct that it contains.

use crate::ir::*;

/// Generate the environment block: ModuleHostTrait, Globals struct, and Env<H> struct.
///
/// This always generates:
/// - `pub trait ModuleHostTrait { ... }` (empty if no imports, with methods if imports)
/// - `impl ModuleHostTrait for herkos_runtime::NoHost {}` (only for modules with NO imports)
/// - `pub struct Globals { ... }` (empty struct if no mutable globals, fields otherwise)
/// - `struct Env<H: ModuleHostTrait> { pub host: H, pub globals: Globals }`
pub fn generate_env_block(info: &ModuleInfo) -> String {
    let mut code = String::new();

    // Generate ModuleHostTrait (unified, with all imports merged)
    code.push_str(&generate_module_host_trait(info));
    code.push('\n');

    // Generate NoHost impl only for modules with NO imports
    let has_imports = !info.func_imports.is_empty() || !info.imported_globals.is_empty();
    if !has_imports {
        code.push_str("impl ModuleHostTrait for herkos_runtime::NoHost {}\n\n");
    }

    // Generate Globals struct
    code.push_str(&generate_globals_struct(info));
    code.push('\n');

    // Generate Env<H> struct
    code.push_str("#[allow(dead_code)]\n");
    code.push_str("struct Env<'a, H: ModuleHostTrait + ?Sized> {\n");
    code.push_str("    pub host: &'a mut H,\n");
    code.push_str("    pub globals: &'a mut Globals,\n");
    code.push_str("}\n\n");

    code
}

/// Generate the unified ModuleHostTrait from both function and global imports.
fn generate_module_host_trait(info: &ModuleInfo) -> String {
    let mut code = String::from("pub trait ModuleHostTrait {\n");

    // Add all function import methods
    for imp in &info.func_imports {
        let mut params = vec!["&mut self".to_string()];
        for (i, ty) in imp.params.iter().enumerate() {
            let rust_ty = crate::codegen::types::wasm_type_to_rust(ty);
            params.push(format!("arg{}: {}", i, rust_ty));
        }

        let return_ty = crate::codegen::types::format_return_type(imp.return_type.as_ref());
        code.push_str(&format!(
            "    fn {}({}) -> {};\n",
            imp.func_name,
            params.join(", "),
            return_ty
        ));
    }

    // Add all global import accessors
    for g in &info.imported_globals {
        let rust_ty = crate::codegen::types::wasm_type_to_rust(&g.wasm_type);

        // Getter (always)
        code.push_str(&format!("    fn get_{}(&self) -> {};\n", g.name, rust_ty));

        // Setter (only if mutable)
        if g.mutable {
            code.push_str(&format!(
                "    fn set_{}(&mut self, val: {});\n",
                g.name, rust_ty
            ));
        }
    }

    code.push_str("}\n");
    code
}

/// Generate the Globals struct containing all mutable globals.
fn generate_globals_struct(info: &ModuleInfo) -> String {
    let mut code = String::from("pub struct Globals {\n");

    for (idx, g) in info.globals.iter().enumerate() {
        if g.mutable {
            let rust_ty = crate::codegen::types::wasm_type_to_rust(&g.init_value.ty());
            code.push_str(&format!("    pub g{}: {},\n", idx, rust_ty));
        }
    }

    code.push_str("}\n");
    code
}
