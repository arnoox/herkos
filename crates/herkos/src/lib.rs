//! herkos — WebAssembly to Rust transpiler.
//!
//! This crate provides the core transpilation pipeline that converts WebAssembly
//! modules into memory-safe Rust source code.

pub mod backend;
pub mod codegen;
pub mod ir;
pub mod optimizer;
pub mod parser;

// Re-export key types for convenience
pub use anyhow::{Context, Result};
use backend::SafeBackend;
use codegen::CodeGenerator;
use ir::builder::build_module_info;
use ir::{lower_phis, LoweredModuleInfo};
use optimizer::{optimize_ir, optimize_lowered_ir};
use parser::parse_wasm;

/// Configuration options for transpilation
#[derive(Debug, Clone)]
pub struct TranspileOptions {
    /// Code generation backend mode ("safe", "verified", "hybrid")
    pub mode: String,
    /// Maximum memory pages (used when Wasm module declares no maximum)
    pub max_pages: usize,
    /// Enable optimizations (default: true)
    pub optimize: bool,
}

impl Default for TranspileOptions {
    fn default() -> Self {
        Self {
            mode: "safe".to_string(),
            max_pages: 256,
            optimize: true,
        }
    }
}

/// Transpile a WebAssembly module to Rust source code.
///
/// This is the main entry point for the transpilation pipeline.
/// It takes raw WASM bytes and returns generated Rust code as a String.
///
/// # Arguments
/// * `wasm_bytes` - Raw WebAssembly binary data
/// * `options` - Transpilation configuration options
///
/// # Returns
/// Generated Rust source code ready to be compiled
///
/// # Example
/// ```no_run
/// use herkos::{transpile, TranspileOptions};
///
/// let wasm_bytes = std::fs::read("input.wasm").unwrap();
/// let options = TranspileOptions::default();
/// let rust_code = transpile(&wasm_bytes, &options).unwrap();
/// std::fs::write("output.rs", rust_code).unwrap();
/// ```
pub fn transpile(wasm_bytes: &[u8], options: &TranspileOptions) -> Result<String> {
    // Parse the WebAssembly binary
    let parsed = parse_wasm(wasm_bytes).context("failed to parse WebAssembly module")?;

    // Build complete module metadata from parsed module
    let module_info =
        build_module_info(&parsed, options).context("failed to build module metadata")?;

    // Optimize the pure SSA IR.
    let module_info = optimize_ir(module_info, options.optimize)?;

    // SSA destruction: lower phi nodes to predecessor assignments.
    let lowered_module_info = lower_phis::lower(module_info);

    // Optimize the lowered IR
    let lowered_module_info = optimize_lowered_ir(lowered_module_info, options.optimize)?;

    // Generate Rust source code
    let rust_code = generate_rust_code(&lowered_module_info)?;

    Ok(rust_code)
}

/// Generates Rust source code from IR and module metadata.
fn generate_rust_code(module_info: &LoweredModuleInfo) -> Result<String> {
    let backend = SafeBackend::new();
    let codegen = CodeGenerator::new(&backend);

    codegen
        .generate_module_with_info(module_info)
        .context("failed to generate Rust code")
}
