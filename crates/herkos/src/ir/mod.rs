//! Intermediate Representation (IR) for Wasm → Rust transpilation.
//!
//! This module defines an SSA-form IR that sits between WebAssembly bytecode
//! and generated Rust source code. The IR is backend-agnostic: the same IR can
//! be used to generate safe, verified, or hybrid Rust code.
//!
//! It includes:
//! - **Per-function IR** ([`IrFunction`], [`IrBlock`], [`IrInstr`]): SSA-form IR for function bodies
//! - **Module-level IR** ([`ModuleInfo`] and related types): Module structure and metadata

mod types;
pub use types::*;

pub mod builder;
pub use builder::{build_module_info, IrBuilder, ModuleContext};
