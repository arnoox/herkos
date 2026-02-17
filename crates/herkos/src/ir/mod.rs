//! Intermediate Representation (IR) for Wasm → Rust transpilation.
//!
//! This module defines an SSA-form IR that sits between WebAssembly bytecode
//! and generated Rust source code. The IR is backend-agnostic: the same IR can
//! be used to generate safe, verified, or hybrid Rust code.

mod types;
pub use types::*;

mod builder;
pub use builder::{IrBuilder, ModuleContext};
