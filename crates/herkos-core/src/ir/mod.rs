//! Intermediate Representation (IR) for Wasm → Rust transpilation.
//!
//! This module defines an SSA-form IR that sits between WebAssembly bytecode
//! and generated Rust source code. The IR is backend-agnostic: the same IR can
//! be used to generate safe, verified, or hybrid Rust code.
//!
//! It includes:
//! - **Per-function IR** ([`IrFunction`], [`IrBlock`], [`IrInstr`]): SSA-form IR for function bodies
//! - **Module-level IR** ([`ModuleInfo`] and related types): Module structure and metadata
//! - **[`LoweredModuleInfo`]**: Post-SSA-destruction wrapper; no `IrInstr::Phi` nodes remain

mod types;
pub use types::*;

pub mod builder;
pub use builder::{build_module_info, ModuleContext};

pub mod lower_phis;

/// [`ModuleInfo`] with all `IrInstr::Phi` nodes lowered to `IrInstr::Assign`.
///
/// Constructed exclusively by [`lower_phis::lower`]. Signals the phase
/// boundary between SSA IR (with phi nodes) and post-SSA IR (without).
/// After this point, no optimizer pass or codegen module will encounter
/// `IrInstr::Phi` in any function body.
pub struct LoweredModuleInfo(ModuleInfo);

impl std::ops::Deref for LoweredModuleInfo {
    type Target = ModuleInfo;
    fn deref(&self) -> &ModuleInfo {
        &self.0
    }
}

impl std::ops::DerefMut for LoweredModuleInfo {
    fn deref_mut(&mut self) -> &mut ModuleInfo {
        &mut self.0
    }
}
