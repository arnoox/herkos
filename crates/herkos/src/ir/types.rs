//! IR type definitions.
//!
//! These types represent a structured, SSA-form intermediate representation
//! of WebAssembly functions. Each Wasm instruction is translated to one or
//! more IR instructions, with explicit variable names (v0, v1, ...) instead
//! of an implicit stack.

use std::fmt;

/// Unique identifier for a variable in SSA form.
/// Variables are numbered sequentially: v0, v1, v2, ...
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub u32);

/// Generic index type with a phantom tag to distinguish different index spaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Idx<TAG> {
    idx: usize,
    _marker: std::marker::PhantomData<TAG>,
}

impl<TAG> Idx<TAG> {
    pub fn new(idx: usize) -> Self {
        Self {
            idx,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn as_usize(&self) -> usize {
        self.idx
    }
}

impl<TAG> From<Idx<TAG>> for usize {
    fn from(idx: Idx<TAG>) -> Self {
        idx.idx
    }
}

/// Type index — indexes into `ModuleInfo::type_signatures` and `ModuleInfo::canonical_type`.
pub type TypeIdx = Idx<FuncSignature>;

/// Marker type for local function indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalFuncIdxTag;

/// Local function index — indexes into `ModuleInfo::ir_functions`.
/// Import count has already been subtracted (imports occupy 0..num_imported_functions-1 in the
/// global Wasm function index space).
pub type LocalFuncIdx = Idx<LocalFuncIdxTag>;

/// Imported function index — indexes into `ModuleInfo::func_imports`.
/// Numerically equivalent to the position in the global Wasm function index space
/// (imports occupy indices 0..num_imports-1).
pub type ImportIdx = Idx<FuncImport>;

/// Global index — unified index into the global space (imported globals first, then local globals).
/// Resolved via `ModuleInfo::resolve_global()` to distinguish imported from local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalIdxdxTag;
pub type GlobalIdx = Idx<GlobalIdxdxTag>;

/// Imported global index — indexes into `ModuleInfo::imported_globals` (raw position).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImportedGlobalIdxTag;
pub type ImportedGlobalIdx = Idx<ImportedGlobalIdxTag>;

/// Local global index — indexes into `ModuleInfo::globals` (import count already subtracted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalGlobalIdxTag;
pub type LocalGlobalIdx = Idx<LocalGlobalIdxTag>;

impl fmt::Display for VarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// Unique identifier for a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "block_{}", self.0)
    }
}

/// WebAssembly value types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
}

// `i32`, `i64`, `f32`, `f64` are the canonical names from the Wasm spec, which happen
// to coincide with Rust's primitive names. This Display implementation reflects the
// spec-level name, not a Rust-specific choice, so it belongs here in the IR layer.
// Any backend that uses different type names (e.g., `int32_t` for C) should do its
// own formatting in codegen rather than calling `to_string()` on a `WasmType`.
impl fmt::Display for WasmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmType::I32 => write!(f, "i32"),
            WasmType::I64 => write!(f, "i64"),
            WasmType::F32 => write!(f, "f32"),
            WasmType::F64 => write!(f, "f64"),
        }
    }
}

impl WasmType {
    /// Convert wasmparser::ValType to our WasmType.
    pub fn from_wasmparser(vt: wasmparser::ValType) -> Self {
        use wasmparser::ValType;
        match vt {
            ValType::I32 => WasmType::I32,
            ValType::I64 => WasmType::I64,
            ValType::F32 => WasmType::F32,
            ValType::F64 => WasmType::F64,
            _ => panic!("Unsupported value type: {:?}", vt),
        }
    }

    /// Get the default/zero value literal string for this type (used in code generation).
    ///
    /// Examples:
    /// - `I32` → `"0i32"`
    /// - `F64` → `"0.0f64"`
    pub fn default_value_literal(&self) -> &'static str {
        match self {
            WasmType::I32 => "0i32",
            WasmType::I64 => "0i64",
            WasmType::F32 => "0.0f32",
            WasmType::F64 => "0.0f64",
        }
    }
}

/// IR representation of a complete function.
#[derive(Debug, Clone)]
pub struct IrFunction {
    /// Function parameters (variable ID + type)
    pub params: Vec<(VarId, WasmType)>,

    /// Local variables (variable ID + type)
    /// Note: params are also locals in Wasm, but we separate them for clarity
    pub locals: Vec<(VarId, WasmType)>,

    /// All basic blocks in the function
    pub blocks: Vec<IrBlock>,

    /// Entry block (where execution starts)
    ///
    /// INVARIANT: This is always `BlockId(0)`. WebAssembly functions always start
    /// execution at the first instruction, so we guarantee that the entry block
    /// is the first block created during IR translation.
    pub entry_block: BlockId,

    /// Return type (None for void functions)
    pub return_type: Option<WasmType>,

    /// Index into the Wasm type section (needed for call_indirect dispatch).
    pub type_idx: TypeIdx,

    /// Whether this function calls imported functions or accesses imported globals (needs host parameter).
    pub needs_host: bool,
}

/// A basic block — sequence of instructions with a single entry and exit.
#[derive(Debug, Clone)]
pub struct IrBlock {
    /// Unique identifier for this block
    pub id: BlockId,

    /// Instructions in this block (no control flow within)
    pub instructions: Vec<IrInstr>,

    /// How control exits this block
    pub terminator: IrTerminator,
}

/// Width of a memory access.
///
/// Wasm supports sub-width loads/stores (e.g., `i32.load8_s` loads 1 byte
/// and sign-extends to i32).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryAccessWidth {
    /// Full type width (i32=4 bytes, i64=8 bytes, f32=4, f64=8)
    Full,
    /// 8-bit access
    I8,
    /// 16-bit access
    I16,
    /// 32-bit access (only valid for i64 loads/stores)
    I32,
}

/// Sign extension for sub-width loads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignExtension {
    Signed,
    Unsigned,
}

/// Result of resolving a `GlobalIdx` to distinguish imported from local globals.
#[derive(Debug, Clone)]
pub enum ResolvedGlobal<'a> {
    /// Imported global with its index and definition.
    Imported(ImportedGlobalIdx, &'a ImportedGlobalDef),
    /// Local global with its index and definition.
    Local(LocalGlobalIdx, &'a GlobalDef),
}

/// A single IR instruction (SSA form — each produces a new variable).
#[derive(Debug, Clone)]
pub enum IrInstr {
    /// Define a variable from a constant value
    Const { dest: VarId, value: IrValue },

    /// Binary operation (dest = lhs op rhs)
    BinOp {
        dest: VarId,
        op: BinOp,
        lhs: VarId,
        rhs: VarId,
    },

    /// Unary operation (dest = op operand)
    UnOp {
        dest: VarId,
        op: UnOp,
        operand: VarId,
    },

    /// Memory load (dest = memory[addr + offset])
    ///
    /// For sub-width loads, `width` specifies the access width and `sign`
    /// specifies sign/zero extension. For full-width loads, `width` is `Full`
    /// and `sign` is `None`.
    Load {
        dest: VarId,
        ty: WasmType,
        addr: VarId,
        offset: u32,
        width: MemoryAccessWidth,
        sign: Option<SignExtension>,
    },

    /// Memory store (memory[addr + offset] = value)
    ///
    /// For sub-width stores, `width` specifies the access width (the value is
    /// truncated to that width). For full-width stores, `width` is `Full`.
    Store {
        ty: WasmType,
        addr: VarId,
        value: VarId,
        offset: u32,
        width: MemoryAccessWidth,
    },

    /// Call direct function (local, not imported)
    Call {
        dest: Option<VarId>,    // None for void functions
        func_idx: LocalFuncIdx, // Local function index (imports are handled separately)
        args: Vec<VarId>,
    },

    /// Call imported function from host
    CallImport {
        dest: Option<VarId>,   // None for void functions
        import_idx: ImportIdx, // Index into the imports list
        module_name: String,   // Import module name (e.g., "env")
        func_name: String,     // Import field name (e.g., "log")
        args: Vec<VarId>,
    },

    /// Call indirect (via table)
    CallIndirect {
        dest: Option<VarId>,
        type_idx: TypeIdx,
        table_idx: VarId,
        args: Vec<VarId>,
    },

    /// Assign variable (for Wasm local.set / local.tee)
    /// Note: This is not pure SSA, but matches Wasm's local semantics
    Assign { dest: VarId, src: VarId },

    /// Read a global variable (dest = globals.g{index} or const G{index})
    GlobalGet { dest: VarId, index: GlobalIdx },

    /// Write a mutable global variable (globals.g{index} = value)
    GlobalSet { index: GlobalIdx, value: VarId },

    /// Query current memory size in pages (dest = memory.size())
    MemorySize { dest: VarId },

    /// Grow memory by delta pages (dest = memory.grow(delta))
    /// Returns previous page count on success, or -1 on failure.
    MemoryGrow { dest: VarId, delta: VarId },

    /// Conditional select (dest = if condition != 0 { val1 } else { val2 })
    Select {
        dest: VarId,
        val1: VarId,
        val2: VarId,
        condition: VarId,
    },
}

/// Block terminator — how control flow exits a basic block.
#[derive(Debug, Clone)]
pub enum IrTerminator {
    /// Return from function
    Return { value: Option<VarId> },

    /// Unconditional jump to target block
    Jump { target: BlockId },

    /// Conditional branch
    BranchIf {
        condition: VarId,
        if_true: BlockId,
        if_false: BlockId,
    },

    /// Multi-way branch (for br_table)
    BranchTable {
        index: VarId,
        targets: Vec<BlockId>,
        default: BlockId,
    },

    /// Unreachable (trap)
    Unreachable,
}

/// Constant value in the IR.
#[derive(Debug, Clone, Copy)]
pub enum IrValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl IrValue {
    /// Returns the WasmType of this constant value.
    pub fn wasm_type(&self) -> WasmType {
        match self {
            IrValue::I32(_) => WasmType::I32,
            IrValue::I64(_) => WasmType::I64,
            IrValue::F32(_) => WasmType::F32,
            IrValue::F64(_) => WasmType::F64,
        }
    }
}

// This Display emits Rust literal syntax (`42i32`, `1.5f32`) and is intentionally
// backend-specific. It is used by the IR debug output and tests to verify that
// constants will be emitted correctly. Since herkos currently has one codegen target
// (Rust), keeping it here avoids premature abstraction. If a second backend is ever
// added, replace this with an explicit `to_rust_literal()` method in the codegen
// layer and repurpose Display for a backend-neutral debug form.
impl fmt::Display for IrValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrValue::I32(v) => write!(f, "{}i32", v),
            IrValue::I64(v) => write!(f, "{}i64", v),
            IrValue::F32(v) => write!(f, "{}f32", v),
            IrValue::F64(v) => write!(f, "{}f64", v),
        }
    }
}

/// Binary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // i32 operations
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS, // Signed division
    I32DivU, // Unsigned division
    I32RemS, // Signed remainder
    I32RemU, // Unsigned remainder
    I32And,
    I32Or,
    I32Xor,
    I32Shl,  // Shift left
    I32ShrS, // Shift right (signed)
    I32ShrU, // Shift right (unsigned)
    I32Rotl, // Rotate left
    I32Rotr, // Rotate right

    // i32 comparisons
    I32Eq,
    I32Ne,
    I32LtS, // Less than (signed)
    I32LtU, // Less than (unsigned)
    I32GtS, // Greater than (signed)
    I32GtU, // Greater than (unsigned)
    I32LeS, // Less or equal (signed)
    I32LeU, // Less or equal (unsigned)
    I32GeS, // Greater or equal (signed)
    I32GeU, // Greater or equal (unsigned)

    // i64 operations (same pattern as i32)
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,

    // i64 comparisons
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,

    // f32 operations
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,

    // f32 comparisons
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,

    // f64 operations
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,

    // f64 comparisons
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
}

/// Unary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    // i32 unary
    I32Clz,    // Count leading zeros
    I32Ctz,    // Count trailing zeros
    I32Popcnt, // Population count (count 1 bits)
    I32Eqz,    // Equal to zero (i32 → i32, 0 or 1)

    // i64 unary
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Eqz,

    // f32 unary
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,

    // f64 unary
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    // Conversions: integer truncation/extension
    I32WrapI64,    // i64 → i32 (truncate to low 32 bits)
    I64ExtendI32S, // i32 → i64 (sign-extend)
    I64ExtendI32U, // i32 → i64 (zero-extend)

    // Conversions: float → integer (trapping on NaN/overflow)
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,

    // Conversions: integer → float
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,

    // Conversions: float precision
    F32DemoteF64,  // f64 → f32
    F64PromoteF32, // f32 → f64

    // Reinterpretations (bitcast)
    I32ReinterpretF32, // f32 → i32
    I64ReinterpretF64, // f64 → i64
    F32ReinterpretI32, // i32 → f32
    F64ReinterpretI64, // i64 → f64
}

impl BinOp {
    /// Returns the WasmType of the result produced by this operation.
    ///
    /// Note: all comparison operations return i32 (0 or 1), even for i64/f32/f64 operands.
    pub fn result_type(&self) -> WasmType {
        match self {
            // i32 arithmetic → i32
            BinOp::I32Add
            | BinOp::I32Sub
            | BinOp::I32Mul
            | BinOp::I32DivS
            | BinOp::I32DivU
            | BinOp::I32RemS
            | BinOp::I32RemU
            | BinOp::I32And
            | BinOp::I32Or
            | BinOp::I32Xor
            | BinOp::I32Shl
            | BinOp::I32ShrS
            | BinOp::I32ShrU
            | BinOp::I32Rotl
            | BinOp::I32Rotr => WasmType::I32,

            // i32 comparisons → i32
            BinOp::I32Eq
            | BinOp::I32Ne
            | BinOp::I32LtS
            | BinOp::I32LtU
            | BinOp::I32GtS
            | BinOp::I32GtU
            | BinOp::I32LeS
            | BinOp::I32LeU
            | BinOp::I32GeS
            | BinOp::I32GeU => WasmType::I32,

            // i64 arithmetic → i64
            BinOp::I64Add
            | BinOp::I64Sub
            | BinOp::I64Mul
            | BinOp::I64DivS
            | BinOp::I64DivU
            | BinOp::I64RemS
            | BinOp::I64RemU
            | BinOp::I64And
            | BinOp::I64Or
            | BinOp::I64Xor
            | BinOp::I64Shl
            | BinOp::I64ShrS
            | BinOp::I64ShrU
            | BinOp::I64Rotl
            | BinOp::I64Rotr => WasmType::I64,

            // i64 comparisons → i32
            BinOp::I64Eq
            | BinOp::I64Ne
            | BinOp::I64LtS
            | BinOp::I64LtU
            | BinOp::I64GtS
            | BinOp::I64GtU
            | BinOp::I64LeS
            | BinOp::I64LeU
            | BinOp::I64GeS
            | BinOp::I64GeU => WasmType::I32,

            // f32 arithmetic → f32
            BinOp::F32Add
            | BinOp::F32Sub
            | BinOp::F32Mul
            | BinOp::F32Div
            | BinOp::F32Min
            | BinOp::F32Max
            | BinOp::F32Copysign => WasmType::F32,

            // f32 comparisons → i32
            BinOp::F32Eq
            | BinOp::F32Ne
            | BinOp::F32Lt
            | BinOp::F32Gt
            | BinOp::F32Le
            | BinOp::F32Ge => WasmType::I32,

            // f64 arithmetic → f64
            BinOp::F64Add
            | BinOp::F64Sub
            | BinOp::F64Mul
            | BinOp::F64Div
            | BinOp::F64Min
            | BinOp::F64Max
            | BinOp::F64Copysign => WasmType::F64,

            // f64 comparisons → i32
            BinOp::F64Eq
            | BinOp::F64Ne
            | BinOp::F64Lt
            | BinOp::F64Gt
            | BinOp::F64Le
            | BinOp::F64Ge => WasmType::I32,
        }
    }
}

impl UnOp {
    /// Returns the WasmType of the result produced by this operation.
    ///
    /// Note: `I64Eqz` returns i32 (0 or 1), not i64.
    pub fn result_type(&self) -> WasmType {
        match self {
            UnOp::I32Clz | UnOp::I32Ctz | UnOp::I32Popcnt | UnOp::I32Eqz => WasmType::I32,
            UnOp::I64Clz | UnOp::I64Ctz | UnOp::I64Popcnt => WasmType::I64,
            UnOp::I64Eqz => WasmType::I32,
            UnOp::F32Abs
            | UnOp::F32Neg
            | UnOp::F32Ceil
            | UnOp::F32Floor
            | UnOp::F32Trunc
            | UnOp::F32Nearest
            | UnOp::F32Sqrt => WasmType::F32,
            UnOp::F64Abs
            | UnOp::F64Neg
            | UnOp::F64Ceil
            | UnOp::F64Floor
            | UnOp::F64Trunc
            | UnOp::F64Nearest
            | UnOp::F64Sqrt => WasmType::F64,

            // Conversions → i32
            UnOp::I32WrapI64
            | UnOp::I32TruncF32S
            | UnOp::I32TruncF32U
            | UnOp::I32TruncF64S
            | UnOp::I32TruncF64U
            | UnOp::I32ReinterpretF32 => WasmType::I32,

            // Conversions → i64
            UnOp::I64ExtendI32S
            | UnOp::I64ExtendI32U
            | UnOp::I64TruncF32S
            | UnOp::I64TruncF32U
            | UnOp::I64TruncF64S
            | UnOp::I64TruncF64U
            | UnOp::I64ReinterpretF64 => WasmType::I64,

            // Conversions → f32
            UnOp::F32ConvertI32S
            | UnOp::F32ConvertI32U
            | UnOp::F32ConvertI64S
            | UnOp::F32ConvertI64U
            | UnOp::F32DemoteF64
            | UnOp::F32ReinterpretI32 => WasmType::F32,

            // Conversions → f64
            UnOp::F64ConvertI32S
            | UnOp::F64ConvertI32U
            | UnOp::F64ConvertI64S
            | UnOp::F64ConvertI64U
            | UnOp::F64PromoteF32
            | UnOp::F64ReinterpretI64 => WasmType::F64,
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::I32Add => "i32.add",
            BinOp::I32Sub => "i32.sub",
            BinOp::I32Mul => "i32.mul",
            BinOp::I64Add => "i64.add",
            BinOp::I64Sub => "i64.sub",
            BinOp::I64Mul => "i64.mul",
            BinOp::F32Add => "f32.add",
            BinOp::F32Sub => "f32.sub",
            BinOp::F32Mul => "f32.mul",
            BinOp::F64Add => "f64.add",
            BinOp::F64Sub => "f64.sub",
            BinOp::F64Mul => "f64.mul",
            _ => return fmt::Debug::fmt(self, f), // Use debug format for others
        };
        write!(f, "{}", s)
    }
}

// ─── Module-level IR metadata ───────────────────────────────────────────────

/// Definition of a Wasm global variable.
#[derive(Debug, Clone)]
pub struct GlobalDef {
    /// Whether the global is mutable.
    pub mutable: bool,
    /// The constant initializer value (also encodes the type).
    pub init_value: GlobalInit,
}

/// Constant initializer value for a global.
#[derive(Debug, Clone, Copy)]
pub enum GlobalInit {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl GlobalInit {
    /// Get the WasmType of this global init value.
    pub fn ty(&self) -> WasmType {
        match self {
            GlobalInit::I32(_) => WasmType::I32,
            GlobalInit::I64(_) => WasmType::I64,
            GlobalInit::F32(_) => WasmType::F32,
            GlobalInit::F64(_) => WasmType::F64,
        }
    }
}

/// A data segment to initialize memory.
#[derive(Debug, Clone)]
pub struct DataSegmentDef {
    /// Byte offset into memory.
    pub offset: u32,
    /// Raw bytes to write.
    pub data: Vec<u8>,
}

/// An exported function mapping.
#[derive(Debug, Clone)]
pub struct FuncExport {
    /// The exported name (becomes a Rust method name).
    pub name: String,
    /// Index into the local function index space (imports excluded).
    pub func_index: LocalFuncIdx,
}

/// Signature of a function.
#[derive(Debug, Clone)]
pub struct FuncSignature {
    /// Parameter types.
    pub params: Vec<WasmType>,
    /// Return type (None for void).
    pub return_type: Option<WasmType>,
    /// Index into the Wasm type section (needed for call_indirect dispatch).
    /// Note: This field is currently always set to 0 and not used in codegen.
    pub type_idx: TypeIdx,
    /// Whether this function calls imported functions (needs host parameter).
    pub needs_host: bool,
}

/// An element segment to initialize a table.
#[derive(Debug, Clone)]
pub struct ElementSegmentDef {
    /// Starting offset in the table.
    pub offset: usize,
    /// Function indices to place into the table starting at `offset`.
    /// These are in the local function index space (imports already subtracted).
    pub func_indices: Vec<LocalFuncIdx>,
}

/// An imported function for trait generation.
#[derive(Debug, Clone)]
pub struct FuncImport {
    /// Import module name (e.g., "env").
    pub module_name: String,
    /// Import function name (e.g., "log").
    pub func_name: String,
    /// Parameter types.
    pub params: Vec<WasmType>,
    /// Return type (None for void).
    pub return_type: Option<WasmType>,
}

/// An imported global variable.
#[derive(Debug, Clone)]
pub struct ImportedGlobalDef {
    /// Import module name.
    pub module_name: String,
    /// Import field name (used as method name in host trait).
    pub name: String,
    /// The Wasm value type.
    pub wasm_type: WasmType,
    /// Whether the global is mutable.
    pub mutable: bool,
}

/// Module-level information describing a WebAssembly module.
/// Memory ownership model for a WebAssembly module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMode {
    /// Module owns its memory (declares the memory section).
    Owned,
    /// Module imports memory from the host.
    Imported,
    /// Module does not use memory.
    None,
}

/// This is the IR representation of a module's structure and metadata,
/// independent of any specific code generation backend. It includes memory
/// layout, table configuration, globals, imports, exports, and code segments.
#[derive(Debug, Clone, Default)]
pub struct ModuleInfo {
    /// Whether the module declares linear memory.
    pub has_memory: bool,
    /// Maximum memory pages (from Wasm memory section or default).
    pub max_pages: usize,
    /// Initial memory pages (from Wasm memory section).
    pub initial_pages: usize,
    /// Initial table size (number of entries).
    pub table_initial: usize,
    /// Maximum table size (for const generic TABLE_MAX).
    pub table_max: usize,
    /// Element segments for table initialization.
    pub element_segments: Vec<ElementSegmentDef>,
    /// Global variable definitions (mutable + immutable).
    pub globals: Vec<GlobalDef>,
    /// Data segments for memory initialization.
    pub data_segments: Vec<DataSegmentDef>,
    /// Exported functions.
    pub func_exports: Vec<FuncExport>,
    /// Type section signatures (for call_indirect dispatch).
    pub type_signatures: Vec<FuncSignature>,
    /// Canonical type index mapping: maps each Wasm type index to the
    /// smallest index with the same structural signature.
    /// Used for spec-compliant structural type equivalence in call_indirect.
    pub canonical_type: Vec<usize>,
    /// Imported functions for trait generation.
    /// The number of imported functions is `func_imports.len()`.
    pub func_imports: Vec<FuncImport>,
    /// Whether memory is imported rather than locally declared.
    pub has_memory_import: bool,
    /// Imported global definitions, in import declaration order.
    pub imported_globals: Vec<ImportedGlobalDef>,
    /// All IR functions in the module.
    pub ir_functions: Vec<IrFunction>,
}

impl ModuleInfo {
    /// Get the number of imported functions.
    ///
    /// This is derived from `func_imports.len()` rather than storing it separately.
    pub fn num_imported_functions(&self) -> usize {
        self.func_imports.len()
    }

    // ─── Typed accessors ───────────────────────────────────────────────────

    /// Get an IR function by local function index.
    pub fn ir_function(&self, idx: LocalFuncIdx) -> Option<&IrFunction> {
        self.ir_functions.get(idx.as_usize())
    }

    /// Get a type signature by type index.
    pub fn type_signature(&self, idx: TypeIdx) -> Option<&FuncSignature> {
        self.type_signatures.get(idx.as_usize())
    }

    /// Get a function import by import index.
    pub fn func_import(&self, idx: ImportIdx) -> Option<&FuncImport> {
        self.func_imports.get(idx.as_usize())
    }

    /// Get a local global by local global index.
    pub fn local_global(&self, idx: LocalGlobalIdx) -> Option<&GlobalDef> {
        self.globals.get(idx.as_usize())
    }

    /// Get an imported global by imported global index.
    pub fn imported_global(&self, idx: ImportedGlobalIdx) -> Option<&ImportedGlobalDef> {
        self.imported_globals.get(idx.as_usize())
    }

    /// Resolve a global index to distinguish imported from local globals.
    pub fn resolve_global(&self, idx: GlobalIdx) -> ResolvedGlobal<'_> {
        let i = idx.as_usize();
        if i < self.imported_globals.len() {
            ResolvedGlobal::Imported(ImportedGlobalIdx::new(i), &self.imported_globals[i])
        } else {
            let local_i = i - self.imported_globals.len();
            ResolvedGlobal::Local(LocalGlobalIdx::new(local_i), &self.globals[local_i])
        }
    }

    // ─── Builder methods ───────────────────────────────────────────────────

    /// Push an IR function and return its local function index.
    pub fn push_ir_function(&mut self, f: IrFunction) -> LocalFuncIdx {
        let idx = LocalFuncIdx::new(self.ir_functions.len());
        self.ir_functions.push(f);
        idx
    }

    /// Push a type signature and return its type index.
    pub fn push_type_signature(&mut self, s: FuncSignature) -> TypeIdx {
        let idx = TypeIdx::new(self.type_signatures.len());
        self.type_signatures.push(s);
        idx
    }

    /// Push a function import and return its import index.
    pub fn push_func_import(&mut self, i: FuncImport) -> ImportIdx {
        let idx = ImportIdx::new(self.func_imports.len());
        self.func_imports.push(i);
        idx
    }

    /// Push a local global and return its local global index.
    pub fn push_global(&mut self, g: GlobalDef) -> LocalGlobalIdx {
        let idx = LocalGlobalIdx::new(self.globals.len());
        self.globals.push(g);
        idx
    }

    /// Push an imported global and return its imported global index.
    pub fn push_imported_global(&mut self, g: ImportedGlobalDef) -> ImportedGlobalIdx {
        let idx = ImportedGlobalIdx::new(self.imported_globals.len());
        self.imported_globals.push(g);
        idx
    }

    // ─── Query methods ─────────────────────────────────────────────────────

    /// Whether the module has any mutable globals.
    pub fn has_mutable_globals(&self) -> bool {
        self.globals.iter().any(|g| g.mutable)
    }

    /// Whether the module has a non-trivial table (for indirect calls).
    pub fn has_table(&self) -> bool {
        self.table_max > 0
    }

    /// Determine the memory ownership model.
    pub fn memory_mode(&self) -> MemoryMode {
        match (self.has_memory, self.has_memory_import) {
            (true, false) => MemoryMode::Owned,
            (false, true) => MemoryMode::Imported,
            _ => MemoryMode::None,
        }
    }
}

/// Group items by module name using a key function.
pub fn group_by_module<'a, T, F>(
    items: &'a [T],
    key: F,
) -> std::collections::BTreeMap<String, Vec<&'a T>>
where
    F: Fn(&T) -> &str,
{
    let mut grouped: std::collections::BTreeMap<String, Vec<&'a T>> =
        std::collections::BTreeMap::new();
    for item in items {
        grouped.entry(key(item).to_string()).or_default().push(item);
    }
    grouped
}

/// Collect all unique module names from function and global imports.
pub fn all_import_module_names(info: &ModuleInfo) -> std::collections::BTreeSet<String> {
    info.func_imports
        .iter()
        .map(|i| i.module_name.clone())
        .chain(info.imported_globals.iter().map(|g| g.module_name.clone()))
        .collect()
}

/// Check if an IR function contains any CallImport instructions.
pub fn has_import_calls(ir_func: &IrFunction) -> bool {
    ir_func.blocks.iter().any(|block| {
        block
            .instructions
            .iter()
            .any(|instr| matches!(instr, IrInstr::CallImport { .. }))
    })
}

/// Check if an IR function accesses any imported globals.
pub fn has_global_import_access(ir_func: &IrFunction, num_imported_globals: usize) -> bool {
    if num_imported_globals == 0 {
        return false;
    }
    ir_func.blocks.iter().any(|block| {
        block.instructions.iter().any(|instr| {
            matches!(
                instr,
                IrInstr::GlobalGet { index, .. } | IrInstr::GlobalSet { index, .. }
                if index.as_usize() < num_imported_globals
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_id_display() {
        assert_eq!(VarId(0).to_string(), "v0");
        assert_eq!(VarId(42).to_string(), "v42");
        assert_eq!(VarId(1000).to_string(), "v1000");
    }

    #[test]
    fn test_var_id_equality() {
        assert_eq!(VarId(5), VarId(5));
        assert_ne!(VarId(5), VarId(6));
    }

    #[test]
    fn test_block_id_display() {
        assert_eq!(BlockId(0).to_string(), "block_0");
        assert_eq!(BlockId(42).to_string(), "block_42");
        assert_eq!(BlockId(1000).to_string(), "block_1000");
    }

    #[test]
    fn test_block_id_equality() {
        assert_eq!(BlockId(5), BlockId(5));
        assert_ne!(BlockId(5), BlockId(6));
    }

    #[test]
    fn test_wasm_type_display() {
        assert_eq!(WasmType::I32.to_string(), "i32");
        assert_eq!(WasmType::I64.to_string(), "i64");
        assert_eq!(WasmType::F32.to_string(), "f32");
        assert_eq!(WasmType::F64.to_string(), "f64");
    }

    #[test]
    fn test_wasm_type_default_value_literal() {
        assert_eq!(WasmType::I32.default_value_literal(), "0i32");
        assert_eq!(WasmType::I64.default_value_literal(), "0i64");
        assert_eq!(WasmType::F32.default_value_literal(), "0.0f32");
        assert_eq!(WasmType::F64.default_value_literal(), "0.0f64");
    }

    #[test]
    fn test_wasm_type_equality() {
        assert_eq!(WasmType::I32, WasmType::I32);
        assert_ne!(WasmType::I32, WasmType::I64);
        assert_ne!(WasmType::F32, WasmType::F64);
    }

    #[test]
    fn test_ir_value_wasm_type() {
        assert_eq!(IrValue::I32(42).wasm_type(), WasmType::I32);
        assert_eq!(IrValue::I64(100).wasm_type(), WasmType::I64);
        assert_eq!(IrValue::F32(1.5).wasm_type(), WasmType::F32);
        assert_eq!(IrValue::F64(2.7).wasm_type(), WasmType::F64);
    }

    #[test]
    fn test_ir_value_display() {
        assert_eq!(IrValue::I32(42).to_string(), "42i32");
        assert_eq!(IrValue::I64(-100).to_string(), "-100i64");
        assert_eq!(IrValue::F32(1.5).to_string(), "1.5f32");
        assert_eq!(IrValue::F64(2.7).to_string(), "2.7f64");
    }

    #[test]
    fn test_binop_result_type_i32_arithmetic() {
        assert_eq!(BinOp::I32Add.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32Sub.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32Mul.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32DivS.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32DivU.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32And.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32Or.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32Xor.result_type(), WasmType::I32);
    }

    #[test]
    fn test_binop_result_type_i32_comparisons() {
        assert_eq!(BinOp::I32Eq.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32Ne.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32LtS.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32LtU.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32GtS.result_type(), WasmType::I32);
        assert_eq!(BinOp::I32GeU.result_type(), WasmType::I32);
    }

    #[test]
    fn test_binop_result_type_i64_arithmetic() {
        assert_eq!(BinOp::I64Add.result_type(), WasmType::I64);
        assert_eq!(BinOp::I64Sub.result_type(), WasmType::I64);
        assert_eq!(BinOp::I64Mul.result_type(), WasmType::I64);
        assert_eq!(BinOp::I64DivS.result_type(), WasmType::I64);
    }

    #[test]
    fn test_binop_result_type_i64_comparisons() {
        // i64 comparisons return i32
        assert_eq!(BinOp::I64Eq.result_type(), WasmType::I32);
        assert_eq!(BinOp::I64Ne.result_type(), WasmType::I32);
        assert_eq!(BinOp::I64LtS.result_type(), WasmType::I32);
        assert_eq!(BinOp::I64LtU.result_type(), WasmType::I32);
    }

    #[test]
    fn test_binop_result_type_f32_arithmetic() {
        assert_eq!(BinOp::F32Add.result_type(), WasmType::F32);
        assert_eq!(BinOp::F32Sub.result_type(), WasmType::F32);
        assert_eq!(BinOp::F32Mul.result_type(), WasmType::F32);
        assert_eq!(BinOp::F32Div.result_type(), WasmType::F32);
        assert_eq!(BinOp::F32Min.result_type(), WasmType::F32);
        assert_eq!(BinOp::F32Max.result_type(), WasmType::F32);
    }

    #[test]
    fn test_binop_result_type_f32_comparisons() {
        // f32 comparisons return i32
        assert_eq!(BinOp::F32Eq.result_type(), WasmType::I32);
        assert_eq!(BinOp::F32Ne.result_type(), WasmType::I32);
        assert_eq!(BinOp::F32Lt.result_type(), WasmType::I32);
        assert_eq!(BinOp::F32Gt.result_type(), WasmType::I32);
    }

    #[test]
    fn test_binop_result_type_f64_arithmetic() {
        assert_eq!(BinOp::F64Add.result_type(), WasmType::F64);
        assert_eq!(BinOp::F64Sub.result_type(), WasmType::F64);
        assert_eq!(BinOp::F64Mul.result_type(), WasmType::F64);
        assert_eq!(BinOp::F64Div.result_type(), WasmType::F64);
    }

    #[test]
    fn test_binop_result_type_f64_comparisons() {
        // f64 comparisons return i32
        assert_eq!(BinOp::F64Eq.result_type(), WasmType::I32);
        assert_eq!(BinOp::F64Ne.result_type(), WasmType::I32);
        assert_eq!(BinOp::F64Lt.result_type(), WasmType::I32);
        assert_eq!(BinOp::F64Le.result_type(), WasmType::I32);
    }

    #[test]
    fn test_unop_result_type_i32() {
        assert_eq!(UnOp::I32Clz.result_type(), WasmType::I32);
        assert_eq!(UnOp::I32Ctz.result_type(), WasmType::I32);
        assert_eq!(UnOp::I32Popcnt.result_type(), WasmType::I32);
        assert_eq!(UnOp::I32Eqz.result_type(), WasmType::I32);
    }

    #[test]
    fn test_unop_result_type_i64() {
        assert_eq!(UnOp::I64Clz.result_type(), WasmType::I64);
        assert_eq!(UnOp::I64Ctz.result_type(), WasmType::I64);
        assert_eq!(UnOp::I64Popcnt.result_type(), WasmType::I64);
        // i64.eqz returns i32
        assert_eq!(UnOp::I64Eqz.result_type(), WasmType::I32);
    }

    #[test]
    fn test_unop_result_type_f32() {
        assert_eq!(UnOp::F32Abs.result_type(), WasmType::F32);
        assert_eq!(UnOp::F32Neg.result_type(), WasmType::F32);
        assert_eq!(UnOp::F32Ceil.result_type(), WasmType::F32);
        assert_eq!(UnOp::F32Floor.result_type(), WasmType::F32);
        assert_eq!(UnOp::F32Sqrt.result_type(), WasmType::F32);
    }

    #[test]
    fn test_unop_result_type_f64() {
        assert_eq!(UnOp::F64Abs.result_type(), WasmType::F64);
        assert_eq!(UnOp::F64Neg.result_type(), WasmType::F64);
        assert_eq!(UnOp::F64Ceil.result_type(), WasmType::F64);
        assert_eq!(UnOp::F64Floor.result_type(), WasmType::F64);
        assert_eq!(UnOp::F64Sqrt.result_type(), WasmType::F64);
    }

    #[test]
    fn test_unop_result_type_conversions_to_i32() {
        assert_eq!(UnOp::I32WrapI64.result_type(), WasmType::I32);
        assert_eq!(UnOp::I32TruncF32S.result_type(), WasmType::I32);
        assert_eq!(UnOp::I32TruncF32U.result_type(), WasmType::I32);
        assert_eq!(UnOp::I32TruncF64S.result_type(), WasmType::I32);
        assert_eq!(UnOp::I32TruncF64U.result_type(), WasmType::I32);
        assert_eq!(UnOp::I32ReinterpretF32.result_type(), WasmType::I32);
    }

    #[test]
    fn test_unop_result_type_conversions_to_i64() {
        assert_eq!(UnOp::I64ExtendI32S.result_type(), WasmType::I64);
        assert_eq!(UnOp::I64ExtendI32U.result_type(), WasmType::I64);
        assert_eq!(UnOp::I64TruncF32S.result_type(), WasmType::I64);
        assert_eq!(UnOp::I64TruncF64S.result_type(), WasmType::I64);
        assert_eq!(UnOp::I64ReinterpretF64.result_type(), WasmType::I64);
    }

    #[test]
    fn test_unop_result_type_conversions_to_f32() {
        assert_eq!(UnOp::F32ConvertI32S.result_type(), WasmType::F32);
        assert_eq!(UnOp::F32ConvertI32U.result_type(), WasmType::F32);
        assert_eq!(UnOp::F32ConvertI64S.result_type(), WasmType::F32);
        assert_eq!(UnOp::F32DemoteF64.result_type(), WasmType::F32);
        assert_eq!(UnOp::F32ReinterpretI32.result_type(), WasmType::F32);
    }

    #[test]
    fn test_unop_result_type_conversions_to_f64() {
        assert_eq!(UnOp::F64ConvertI32S.result_type(), WasmType::F64);
        assert_eq!(UnOp::F64ConvertI32U.result_type(), WasmType::F64);
        assert_eq!(UnOp::F64ConvertI64S.result_type(), WasmType::F64);
        assert_eq!(UnOp::F64PromoteF32.result_type(), WasmType::F64);
        assert_eq!(UnOp::F64ReinterpretI64.result_type(), WasmType::F64);
    }

    #[test]
    fn test_memory_mode_owned() {
        let info = ModuleInfo {
            has_memory: true,
            has_memory_import: false,
            ..Default::default()
        };
        assert_eq!(info.memory_mode(), MemoryMode::Owned);
    }

    #[test]
    fn test_memory_mode_imported() {
        let info = ModuleInfo {
            has_memory: false,
            has_memory_import: true,
            ..Default::default()
        };
        assert_eq!(info.memory_mode(), MemoryMode::Imported);
    }

    #[test]
    fn test_memory_mode_none() {
        let info = ModuleInfo::default();
        assert_eq!(info.memory_mode(), MemoryMode::None);
    }

    #[test]
    fn test_module_info_num_imported_functions() {
        let info = ModuleInfo {
            func_imports: vec![
                FuncImport {
                    module_name: "env".to_string(),
                    func_name: "log".to_string(),
                    params: vec![WasmType::I32],
                    return_type: None,
                },
                FuncImport {
                    module_name: "env".to_string(),
                    func_name: "read".to_string(),
                    params: vec![],
                    return_type: Some(WasmType::I32),
                },
            ],
            ..Default::default()
        };
        assert_eq!(info.num_imported_functions(), 2);
    }

    #[test]
    fn test_module_info_has_mutable_globals() {
        let mut info = ModuleInfo::default();
        assert!(!info.has_mutable_globals());

        info.globals.push(GlobalDef {
            mutable: false,
            init_value: GlobalInit::I32(0),
        });
        assert!(!info.has_mutable_globals());

        info.globals.push(GlobalDef {
            mutable: true,
            init_value: GlobalInit::I32(0),
        });
        assert!(info.has_mutable_globals());
    }

    #[test]
    fn test_module_info_has_table() {
        let mut info = ModuleInfo::default();
        assert!(!info.has_table());

        info.table_max = 10;
        assert!(info.has_table());
    }

    #[test]
    fn test_group_by_module() {
        let imports = vec![
            FuncImport {
                module_name: "env".to_string(),
                func_name: "log".to_string(),
                params: vec![],
                return_type: None,
            },
            FuncImport {
                module_name: "wasi".to_string(),
                func_name: "read".to_string(),
                params: vec![],
                return_type: Some(WasmType::I32),
            },
            FuncImport {
                module_name: "env".to_string(),
                func_name: "debug".to_string(),
                params: vec![],
                return_type: None,
            },
        ];

        let grouped = group_by_module(&imports, |i| &i.module_name);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped["env"].len(), 2);
        assert_eq!(grouped["wasi"].len(), 1);
        assert_eq!(grouped["env"][0].func_name, "log");
        assert_eq!(grouped["env"][1].func_name, "debug");
    }

    #[test]
    fn test_all_import_module_names() {
        let info = ModuleInfo {
            func_imports: vec![
                FuncImport {
                    module_name: "env".to_string(),
                    func_name: "log".to_string(),
                    params: vec![],
                    return_type: None,
                },
                FuncImport {
                    module_name: "wasi".to_string(),
                    func_name: "read".to_string(),
                    params: vec![],
                    return_type: Some(WasmType::I32),
                },
            ],
            imported_globals: vec![
                ImportedGlobalDef {
                    module_name: "env".to_string(),
                    name: "mem_ptr".to_string(),
                    wasm_type: WasmType::I32,
                    mutable: false,
                },
                ImportedGlobalDef {
                    module_name: "sys".to_string(),
                    name: "errno".to_string(),
                    wasm_type: WasmType::I32,
                    mutable: true,
                },
            ],
            ..Default::default()
        };

        let module_names = all_import_module_names(&info);
        assert_eq!(module_names.len(), 3);
        assert!(module_names.contains("env"));
        assert!(module_names.contains("wasi"));
        assert!(module_names.contains("sys"));
    }

    #[test]
    fn test_has_import_calls() {
        // Test without import calls
        let ir_func_no_imports = IrFunction {
            params: vec![],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(42),
                }],
                terminator: IrTerminator::Return { value: None },
            }],
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
            needs_host: false,
        };
        assert!(!has_import_calls(&ir_func_no_imports));

        // Test with import calls
        let ir_func_with_imports = IrFunction {
            params: vec![],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                instructions: vec![
                    IrInstr::Const {
                        dest: VarId(0),
                        value: IrValue::I32(42),
                    },
                    IrInstr::CallImport {
                        dest: None,
                        import_idx: ImportIdx::new(0),
                        module_name: "env".to_string(),
                        func_name: "log".to_string(),
                        args: vec![VarId(0)],
                    },
                ],
                terminator: IrTerminator::Return { value: None },
            }],
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
            needs_host: true,
        };
        assert!(has_import_calls(&ir_func_with_imports));
    }

    #[test]
    fn test_has_global_import_access() {
        let ir_func = IrFunction {
            params: vec![],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::Const {
                    dest: VarId(0),
                    value: IrValue::I32(42),
                }],
                terminator: IrTerminator::Return { value: None },
            }],
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
            needs_host: false,
        };

        // No imported globals
        assert!(!has_global_import_access(&ir_func, 0));

        // Has imported globals but function doesn't access them
        assert!(!has_global_import_access(&ir_func, 2));

        // Test with GlobalGet accessing imported global
        let ir_func_with_global_get = IrFunction {
            params: vec![],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                instructions: vec![
                    IrInstr::Const {
                        dest: VarId(0),
                        value: IrValue::I32(42),
                    },
                    IrInstr::GlobalGet {
                        dest: VarId(1),
                        index: GlobalIdx::new(0), // First imported global
                    },
                ],
                terminator: IrTerminator::Return { value: None },
            }],
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
            needs_host: true,
        };
        assert!(has_global_import_access(&ir_func_with_global_get, 2));
    }

    #[test]
    fn test_has_global_import_access_set() {
        let ir_func = IrFunction {
            params: vec![],
            locals: vec![],
            blocks: vec![IrBlock {
                id: BlockId(0),
                instructions: vec![IrInstr::GlobalSet {
                    index: GlobalIdx::new(1), // Second imported global
                    value: VarId(0),
                }],
                terminator: IrTerminator::Return { value: None },
            }],
            entry_block: BlockId(0),
            return_type: None,
            type_idx: TypeIdx::new(0),
            needs_host: true,
        };

        assert!(has_global_import_access(&ir_func, 2));
        assert!(!has_global_import_access(&ir_func, 1)); // Only 1 imported global, index 1 is out of range
    }

    #[test]
    fn test_memory_access_width_equality() {
        assert_eq!(MemoryAccessWidth::Full, MemoryAccessWidth::Full);
        assert_eq!(MemoryAccessWidth::I8, MemoryAccessWidth::I8);
        assert_ne!(MemoryAccessWidth::I8, MemoryAccessWidth::I16);
    }

    #[test]
    fn test_sign_extension_equality() {
        assert_eq!(SignExtension::Signed, SignExtension::Signed);
        assert_eq!(SignExtension::Unsigned, SignExtension::Unsigned);
        assert_ne!(SignExtension::Signed, SignExtension::Unsigned);
    }
}
