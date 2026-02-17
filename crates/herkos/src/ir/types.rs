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

// TODO: interesting that this is done here, because it is very Rust specific. Maybe we want to move this to the codegen phase instead? For now it is convenient to have it here for debugging and testing.
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
}

/// A basic block — sequence of instructions with a single entry and exit.
#[derive(Debug, Clone)]
pub struct IrBlock {
    /// Unique identifier for this block
    pub id: BlockId,

    /// Label for Rust codegen ('block_0, 'block_1, etc.)
    pub label: String,

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
        dest: Option<VarId>, // None for void functions
        func_idx: u32,       // Local function index (imports are handled separately)
        args: Vec<VarId>,
    },

    /// Call imported function from host
    CallImport {
        dest: Option<VarId>, // None for void functions
        import_idx: u32,     // Index into the imports list
        module_name: String, // Import module name (e.g., "env")
        func_name: String,   // Import field name (e.g., "log")
        args: Vec<VarId>,
    },

    /// Call indirect (via table)
    CallIndirect {
        dest: Option<VarId>,
        type_idx: u32,
        table_idx: VarId,
        args: Vec<VarId>,
    },

    /// Assign variable (for Wasm local.set / local.tee)
    /// Note: This is not pure SSA, but matches Wasm's local semantics
    Assign { dest: VarId, src: VarId },

    /// Read a global variable (dest = globals.g{index} or const G{index})
    GlobalGet { dest: VarId, index: u32 },

    /// Write a mutable global variable (globals.g{index} = value)
    GlobalSet { index: u32, value: VarId },

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

// TODO: interesting that this is done here, because it is very Rust specific. Maybe we want to move this to the codegen phase instead? For now it is convenient to have it here for debugging and testing.
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
