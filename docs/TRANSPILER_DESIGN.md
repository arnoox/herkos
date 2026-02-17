# Transpiler Architecture Design

## Overview

The `herkos` transpiler transforms WebAssembly modules into safe, isolated Rust code. This document describes the internal architecture and implementation strategy.

---

## Design Principles

1. **Correctness first**: The transpiler must preserve Wasm semantics exactly
2. **Readable output**: Generated Rust should be auditable by humans
3. **Modularity**: Clean separation between parsing, IR, and codegen
4. **Backend flexibility**: Same IR can target safe/verified/hybrid backends
5. **Incremental complexity**: Start simple (pure functions), add features progressively

---

## Architectural Layers

### Layer 1: Parser (`parser` module)

**Responsibility**: Convert `.wasm` binary to structured module representation

**Implementation**:
- Use `wasmparser` crate (industry standard, well-tested)
- Parse in one pass, validate sections
- Extract:
  - Type section → function signatures
  - Import section → trait requirements
  - Function section → function bodies (bytecode)
  - Table/Memory/Global sections → module state
  - Export section → public API
  - Data/Element sections → initialization

**Output**: `ParsedModule` struct containing all sections

```rust
struct ParsedModule {
    types: Vec<FuncType>,
    imports: Vec<Import>,
    functions: Vec<Function>,
    tables: Vec<Table>,
    memories: Vec<Memory>,
    globals: Vec<Global>,
    exports: Vec<Export>,
    start: Option<FuncIdx>,
    data: Vec<DataSegment>,
    elements: Vec<ElementSegment>,
}

struct Function {
    type_idx: u32,
    locals: Vec<ValType>,
    body: Vec<u8>,  // Wasm bytecode
}
```

**Key Challenge**: Wasm bytecode is stack-based, but we need to generate expression-based Rust. This is where the IR comes in.

---

### Layer 2: IR Builder (`ir` module)

**Responsibility**: Transform stack-based Wasm bytecode into a structured IR suitable for Rust codegen

**The Core Problem**: Wasm is stack-based:
```text
local.get 0
local.get 1
i32.add
local.set 2
```

Rust is expression-based:
```rust
let _2 = _0.wrapping_add(_1);
```

**Solution**: **Single-Static Assignment (SSA) IR** with structured control flow

#### IR Design

```rust
/// High-level IR for a function body
struct IrFunction {
    params: Vec<(VarId, WasmType)>,
    locals: Vec<(VarId, WasmType)>,
    blocks: Vec<IrBlock>,
    return_type: Option<WasmType>,
}

/// Basic block - sequence of instructions with no internal control flow
struct IrBlock {
    id: BlockId,
    instructions: Vec<IrInstr>,
    terminator: IrTerminator,
}

/// IR instruction (SSA form - each produces a new variable)
enum IrInstr {
    /// Define a new variable from a constant
    Const { dest: VarId, value: IrValue },

    /// Binary operation
    BinOp { dest: VarId, op: BinOp, lhs: VarId, rhs: VarId },

    /// Unary operation
    UnOp { dest: VarId, op: UnOp, operand: VarId },

    /// Memory load (checked or unchecked depending on backend)
    Load { dest: VarId, ty: WasmType, addr: VarId, offset: u32, proof: Option<ProofId> },

    /// Memory store
    Store { ty: WasmType, addr: VarId, value: VarId, offset: u32, proof: Option<ProofId> },

    /// Call direct function
    Call { dest: Option<VarId>, func_idx: u32, args: Vec<VarId> },

    /// Call indirect (via table)
    CallIndirect { dest: Option<VarId>, type_idx: u32, table_idx: VarId, args: Vec<VarId> },

    /// Assign (for Wasm local.set / local.tee)
    Assign { dest: VarId, src: VarId },
}

/// Block terminator - how control flow exits this block
enum IrTerminator {
    /// Return from function
    Return { value: Option<VarId> },

    /// Unconditional jump to block
    Jump { target: BlockId },

    /// Conditional branch
    BranchIf { condition: VarId, if_true: BlockId, if_false: BlockId },

    /// Multi-way branch (for br_table)
    BranchTable { index: VarId, targets: Vec<BlockId>, default: BlockId },

    /// Unreachable (trap)
    Unreachable,
}
```

#### Stack to SSA Translation

**Algorithm**: Modified stack machine interpreter that generates IR instead of executing

```rust
struct IrBuilder {
    current_block: BlockId,
    value_stack: Vec<VarId>,  // Wasm's evaluation stack, now SSA variables
    next_var_id: u32,
    blocks: Vec<IrBlock>,
}

impl IrBuilder {
    fn translate_bytecode(&mut self, bytecode: &[u8]) -> IrFunction {
        for instr in parse(bytecode) {
            match instr {
                // Push constant onto stack
                I32Const(n) => {
                    let var = self.new_var();
                    self.emit(IrInstr::Const { dest: var, value: IrValue::I32(n) });
                    self.value_stack.push(var);
                }

                // Pop two values, add, push result
                I32Add => {
                    let rhs = self.value_stack.pop().unwrap();
                    let lhs = self.value_stack.pop().unwrap();
                    let result = self.new_var();
                    self.emit(IrInstr::BinOp {
                        dest: result,
                        op: BinOp::I32Add,
                        lhs,
                        rhs,
                    });
                    self.value_stack.push(result);
                }

                // Load from memory
                I32Load { offset } => {
                    let addr = self.value_stack.pop().unwrap();
                    let result = self.new_var();
                    self.emit(IrInstr::Load {
                        dest: result,
                        ty: WasmType::I32,
                        addr,
                        offset,
                        proof: None,  // filled in by backend
                    });
                    self.value_stack.push(result);
                }

                // ... handle all Wasm instructions
            }
        }
    }
}
```

**Control Flow Handling**: Wasm's `block`/`loop`/`if` map to labeled blocks in the IR, with `br` becoming explicit jumps.

---

### Layer 3: IR Optimizer (`optimize` module)

**Responsibility**: Apply transformations to the IR to improve generated code

**Phase 2**: Skip this entirely - emit unoptimized IR
**Phase 6+**: Add optimizations:
- Dead code elimination
- Constant propagation
- Common subexpression elimination
- Stack slot coalescing (reduce number of locals)

**Why separate from codegen?**: Keep backend-specific logic out of general optimizations. All three backends benefit from the same IR-level opts.

---

### Layer 4: Backend Selection (`backend` module)

**Responsibility**: Choose how to translate each IR instruction based on mode and available proofs

**Trait-based design**:

```rust
trait Backend {
    /// Generate code for a memory load
    fn emit_load(
        &self,
        ctx: &CodegenContext,
        dest: VarId,
        ty: WasmType,
        addr: VarId,
        offset: u32,
        proof: Option<ProofId>,
    ) -> String;

    /// Generate code for a memory store
    fn emit_store(
        &self,
        ctx: &CodegenContext,
        ty: WasmType,
        addr: VarId,
        value: VarId,
        offset: u32,
        proof: Option<ProofId>,
    ) -> String;

    /// Generate code for arithmetic operations
    fn emit_binop(
        &self,
        ctx: &CodegenContext,
        dest: VarId,
        op: BinOp,
        lhs: VarId,
        rhs: VarId,
    ) -> String;

    // ... other instruction types
}
```

**Three implementations**:

#### SafeBackend
- All loads/stores → `memory.load_i32(addr)?` (bounds-checked)
- All arithmetic → `checked_add` where overflow is a trap
- Ignores `proof` field entirely
- Function returns `WasmResult<T>`

```rust
impl Backend for SafeBackend {
    fn emit_load(&self, ctx: &CodegenContext, dest: VarId, ty: WasmType, addr: VarId, offset: u32, _proof: Option<ProofId>) -> String {
        let addr_expr = if offset > 0 {
            format!("{}_{}.wrapping_add({})", ctx.var_prefix, addr, offset)
        } else {
            format!("{}_{}", ctx.var_prefix, addr)
        };

        match ty {
            WasmType::I32 => format!(
                "let {}_{} = memory.load_i32({} as usize)?;",
                ctx.var_prefix, dest, addr_expr
            ),
            // ... other types
        }
    }
}
```

#### VerifiedBackend
- Loads/stores **with proof** → `unsafe { memory.load_i32_unchecked(addr) }` + `// PROOF: bounds_0xABCD`
- Loads/stores **without proof** → **fail compilation** (error: "no proof for access at instruction offset 0x1234")
- Function returns `T` directly (no `Result`) if all paths proven safe

```rust
impl Backend for VerifiedBackend {
    fn emit_load(&self, ctx: &CodegenContext, dest: VarId, ty: WasmType, addr: VarId, offset: u32, proof: Option<ProofId>) -> String {
        let proof_id = proof.expect("verified backend requires proof for all memory accesses");
        let proof_meta = ctx.metadata.get_proof(proof_id);

        // Generate proof comment
        let comment = format!("// PROOF: {} — {}", proof_id, proof_meta.summary);

        // Generate unsafe access
        let access = format!(
            "unsafe {{ memory.load_i32_unchecked({} as usize) }}",
            format_addr(addr, offset)
        );

        format!("{}\nlet {}_{} = {};", comment, ctx.var_prefix, dest, access)
    }
}
```

#### HybridBackend
- Loads/stores **with proof** → `unsafe { ... }` + proof comment
- Loads/stores **without proof** → `memory.load_i32(...)?` + `// UNPROVEN: fallback to runtime check`
- Function returns `WasmResult<T>` (needs to handle both paths)

**Key insight**: The IR is backend-agnostic. The backend only affects *how* each IR instruction is rendered to Rust, not the structure.

---

### Layer 5: Codegen (`codegen` module)

**Responsibility**: Walk the IR and emit Rust source code using the selected backend

**Output Structure**:

```rust
// Generated module structure:
mod generated_module {
    use herkos_runtime::*;

    // Type aliases for clarity
    type Mem = IsolatedMemory<MAX_PAGES>;

    // Globals struct (one field per mutable global)
    struct Globals {
        g0: i32,
        g1: i64,
    }

    // Const globals (immutable)
    const G2: i32 = 42;

    // Function translation
    fn func_0(memory: &mut Mem, globals: &mut Globals, param0: i32) -> WasmResult<i32> {
        // IR → Rust statements
        let v0 = param0;
        let v1 = globals.g0;
        let v2 = v0.wrapping_add(v1);
        memory.store_i32(v2 as usize, 99)?;
        Ok(v2)
    }

    // Module wrapper
    pub struct MyModule<const MAX_PAGES: usize> {
        memory: IsolatedMemory<MAX_PAGES>,
        globals: Globals,
    }

    impl<const MAX_PAGES: usize> MyModule<MAX_PAGES> {
        pub fn new() -> Self {
            // Initialize memory, globals, data segments
        }

        // Exported functions
        pub fn exported_add(&mut self, a: i32, b: i32) -> WasmResult<i32> {
            func_0(&mut self.memory, &mut self.globals, a)
        }
    }
}
```

**Codegen Algorithm**:

```rust
struct RustCodegen {
    backend: Box<dyn Backend>,
    metadata: Option<VerificationMetadata>,
}

impl RustCodegen {
    fn generate_function(&self, ir: &IrFunction) -> String {
        let mut code = String::new();

        // Function signature
        code.push_str(&self.emit_signature(ir));
        code.push_str(" {\n");

        // Locals declaration
        for (var, ty) in &ir.locals {
            code.push_str(&format!("    let mut {}_{}: {};\n", "v", var, ty));
        }

        // Translate each block
        for block in &ir.blocks {
            code.push_str(&format!("    'block_{}:\n", block.id));

            for instr in &block.instructions {
                let stmt = self.backend.emit_instruction(instr, &context);
                code.push_str(&format!("    {}\n", stmt));
            }

            // Terminator
            let term = self.backend.emit_terminator(&block.terminator);
            code.push_str(&format!("    {}\n", term));
        }

        code.push_str("}\n");
        code
    }
}
```

**Post-processing**: Run generated code through `rustfmt` for consistent formatting.

#### State Machine Pattern for Control Flow

**Design Decision**: Control flow graphs (CFGs) with multiple basic blocks are implemented using a state machine pattern rather than Rust labeled blocks.

**The Problem**: WebAssembly's structured control flow (`block`, `loop`, `if`, `br`, `br_table`) creates CFGs where blocks can jump to non-enclosing blocks. For example:

```text
block $outer
  block $inner
    ;; code here
    br $outer    ;; Jump to outer block (not enclosing)
  end
  ;; more code
end
```

**Why Not Rust Labeled Blocks?**: The natural mapping would be Wasm blocks → Rust labeled blocks:

```rust
'outer: {
    'inner: {
        // code here
        break 'outer;  // Jump to outer
    }
    // more code
}
```

However, Rust labeled blocks have a critical limitation: **you can only `break` to an enclosing label, not to a sibling**. This fails:

```rust
'block_0: {
    if condition {
        break 'block_1;  // ERROR: 'block_1 is not an enclosing label
    }
}
'block_1: {
    // ...
}
```

**Solution: State Machine Pattern**

Generate a loop-based state machine where each basic block becomes a match arm:

```rust
pub fn example(mut v0: i32) -> WasmResult<i32> {
    let mut v1: i32 = 0;
    let mut v2: i32 = 0;

    let mut __current_block = 0u32;
    loop {
        match __current_block {
            0 => {
                // Block 0 instructions
                v1 = v0.wrapping_add(1);
                if v0 != 0 {
                    __current_block = 1;
                } else {
                    __current_block = 2;
                }
                continue;
            }
            1 => {
                // Block 1 instructions
                v2 = v1.wrapping_mul(2);
                __current_block = 3;
                continue;
            }
            2 => {
                // Block 2 instructions
                v2 = v1.wrapping_add(10);
                __current_block = 3;
                continue;
            }
            3 => {
                // Block 3 instructions (exit)
                return Ok(v2);
            }
            _ => unreachable!(),
        }
    }
}
```

**Key Implementation Details**:

1. **Variable Hoisting**: All SSA variables are declared at function start with `let mut`, enabling access across all blocks
2. **BlockId Mapping**: BlockIds (opaque identifiers in IR) map to sequential match arm indices via `HashMap<BlockId, usize>`
3. **Control Transfer**: Setting `__current_block` and `continue` simulates jumps
4. **PHI Nodes**: Result variables unify values from different control flow paths (e.g., if/else branches)
5. **Mutable Parameters**: WebAssembly allows mutation of parameters, so all params are `mut`

**Performance Considerations**:

- **Potential Overhead**: Indirect branching via match, loop iteration overhead
- **LLVM Optimizations**: Modern compilers (LLVM used by rustc) optimize state machines aggressively:
  - Jump table generation for dense match arms
  - Tail call optimization eliminating loop overhead
  - Inlining and specialization for small functions
  - Dead code elimination for unreachable states
- **Real-World Evidence**: Rust's async/await uses this exact pattern (poll-based state machines) with negligible overhead
- **Context**: For WebAssembly workloads, the overhead is typically insignificant compared to:
  - Memory access latency
  - Actual computation (SIMD, floating-point)
  - Function call overhead

**Alternative Approaches Considered**:

1. **Goto via `loop { match { ... break 'outer } }`** - Still hits Rust's label scope restrictions
2. **Recursive functions** - Stack overflow risk, poor performance, violates `no_std` constraints
3. **Function pointers/closures** - Excessive dynamic dispatch, allocation, not `no_std` compatible
4. **Macro-generated labeled blocks** - Can't dynamically compute jump targets (br_table)

**Verification**: Phase 9 includes runtime performance benchmarks comparing generated code against hand-written Rust and native Wasm runtimes to measure any overhead.

#### Function Call Strategy

The transpiler handles two Wasm calling instructions: `call` (direct) and `call_indirect` (indirect via table). Both are resolved at transpile time into static Rust function calls — no function pointers, trait objects, or dynamic dispatch at runtime.

##### Direct Calls (`call`)

Direct calls are straightforward. The `call` instruction specifies a function index; the transpiler emits a call to `func_{index}(...)`:

```rust
// IR: Call { dest: Some(v5), func_idx: 3, args: [v3, v4] }
// Codegen (module with globals + memory + table):
v5 = func_3(v3, v4, globals, memory, table)?;
```

The backend (`emit_call`) appends the shared state parameters based on the module's features:

| Module has... | Parameters appended |
|---------------|-------------------|
| Mutable globals | `globals` |
| Memory | `memory` |
| Table (indirect calls) | `table` |

This is determined statically during codegen from `ModuleInfo`. Functions that don't need globals/memory/table still receive them (to keep a uniform calling convention within a module), but the compiler eliminates unused parameters via dead code elimination.

##### Indirect Calls (`call_indirect`)

`call_indirect` is the Wasm mechanism for function pointers and vtable dispatch. The implementation uses a two-level indirection:

1. **Table lookup** (runtime): the table index from the stack selects a `FuncRef` entry
2. **Match dispatch** (static): the `func_index` field of the `FuncRef` selects the concrete function

###### IR Representation

```rust
IrInstr::CallIndirect {
    dest: Option<VarId>,    // where to store the result
    type_idx: u32,          // expected Wasm type index
    table_idx: VarId,       // table index (from the stack)
    args: Vec<VarId>,       // function arguments (from the stack)
}
```

The IR builder (`builder.rs`) handles `Operator::CallIndirect` by:
1. Popping the table index from the value stack
2. Looking up the type signature in `type_signatures` to determine param count and return type
3. Popping `param_count` arguments from the value stack
4. Emitting the `CallIndirect` IR instruction

The `type_signatures` field stores `(param_count, Option<WasmType>)` — just enough for stack manipulation during IR construction. Full structural type information is used later during codegen for the type check and dispatch filtering.

###### Codegen Strategy

The `generate_call_indirect` method in `codegen/mod.rs` emits three parts:

```rust
// 1. Table lookup — runtime, traps on out-of-bounds or uninitialized
let __entry = table.get(v2 as u32)?;

// 2. Type check — compares canonical type indices
if __entry.type_index != 0 { return Err(WasmTrap::IndirectCallTypeMismatch); }

// 3. Match dispatch — only arms for functions with matching canonical type
v4 = match __entry.func_index {
    0 => func_0(v0, v1, table)?,    // type_idx matches
    1 => func_1(v0, v1, table)?,    // type_idx matches
    2 => func_2(v0, v1, table)?,    // type_idx matches
    // func_3 omitted — different type
    _ => return Err(WasmTrap::UndefinedElement),
};
```

The match arms are filtered at transpile time: only functions whose `FuncSignature.type_idx` equals the canonical type index for the `call_indirect`'s expected type are included. This means:

- A table entry pointing to a function of the wrong type is caught by the type check (step 2)
- A table entry pointing to a function of the right type is dispatched by the match (step 3)
- A corrupted `func_index` that doesn't match any function of the right type hits the `_ =>` arm

###### Canonical Type Index Mapping

The Wasm spec requires structural type equivalence for `call_indirect`. The transpiler computes a canonical mapping in `lib.rs`:

```rust
// For each type index, find the smallest index with the same (params, results):
let canonical_type: Vec<u32> = {
    let mut mapping = Vec::with_capacity(parsed.types.len());
    for (i, ty) in parsed.types.iter().enumerate() {
        let canon = parsed.types[..i]
            .iter()
            .position(|earlier| earlier.params() == ty.params()
                              && earlier.results() == ty.results())
            .map(|pos| mapping[pos])
            .unwrap_or(i as u32);
        mapping.push(canon);
    }
    mapping
};
```

This mapping is used in two places:
1. **Element segment initialization**: `FuncRef.type_index` stores the canonical index
2. **`generate_call_indirect`**: the type check and dispatch filtering use canonical indices

This ensures that structurally identical types (e.g., `(i32, i32) → i32` declared twice) are treated as equivalent at runtime, as the spec requires.

###### Table Initialization in the Constructor

The `generate_constructor` method initializes the table from element segments:

```rust
// Module with memory:
let mut module = Module::try_new(initial_pages, globals, Table::try_new(table_initial));
module.table.set(0, Some(FuncRef { type_index: 0, func_index: 0 })).unwrap();
// ...

// Module without memory (LibraryModule):
let mut table = Table::try_new(table_initial);
table.set(0, Some(FuncRef { type_index: 0, func_index: 0 })).unwrap();
// ...
Ok(WasmModule(LibraryModule::new(globals, table)))
```

###### Function Signatures with Table Parameter

When a module uses indirect calls, all functions receive `table: &Table<TABLE_MAX>` as an additional parameter (immutable borrow — the table is not modified during execution in the MVP):

```rust
fn func_0(mut v0: i32, mut v1: i32, table: &Table<TABLE_MAX>) -> WasmResult<i32> {
    // ...
}
```

Functions that don't use `call_indirect` still receive the table parameter to maintain a uniform calling convention. The `#[allow(unused_variables)]` attribute suppresses warnings for these cases. The Rust compiler eliminates unused parameters in release builds.

###### Design Rationale

**Why match dispatch instead of function pointers?**

| Approach | `no_std` | `unsafe`-free | Auditability | Performance |
|----------|----------|---------------|-------------|-------------|
| Match dispatch | Yes | Yes | High — all targets visible in source | Excellent (jump table) |
| Function pointer array | Yes | Requires `unsafe` for calls | Low — opaque at call site | Excellent |
| `dyn Fn` trait objects | Requires `alloc` | Yes | Medium | Vtable overhead |
| Computed goto | N/A (not in Rust) | N/A | N/A | N/A |

The match-based approach is the only option that satisfies all constraints: `no_std`, zero `unsafe`, full auditability, and competitive performance. LLVM compiles dense match arms into jump tables, so the runtime cost is equivalent to a function pointer array.

---

## Data Structures Summary

### Core Types

```rust
// parser/mod.rs
pub struct ParsedModule { /* from wasmparser */ }

// ir/mod.rs
pub struct IrModule {
    functions: Vec<IrFunction>,
    globals: Vec<Global>,
    tables: Vec<Table>,
    memories: Vec<Memory>,
}

pub struct IrFunction {
    params: Vec<(VarId, WasmType)>,
    locals: Vec<(VarId, WasmType)>,
    blocks: Vec<IrBlock>,
    return_type: Option<WasmType>,
}

// backend/mod.rs
pub trait Backend {
    fn emit_load(...) -> String;
    fn emit_store(...) -> String;
    fn emit_binop(...) -> String;
    // ...
}

pub struct SafeBackend;
pub struct VerifiedBackend { metadata: VerificationMetadata }
pub struct HybridBackend { metadata: VerificationMetadata }

// codegen/mod.rs
pub struct RustCodegen {
    backend: Box<dyn Backend>,
}

impl RustCodegen {
    pub fn generate_module(&self, ir: &IrModule) -> String;
}
```

---

## Module Structure (Crate Organization)

```
crates/herkos/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library interface
│   ├── parser/
│   │   ├── mod.rs           # wasmparser wrapper
│   │   └── module.rs        # ParsedModule extraction
│   ├── ir/
│   │   ├── mod.rs           # IR types
│   │   ├── builder.rs       # Wasm bytecode → IR
│   │   ├── types.rs         # IrFunction, IrBlock, IrInstr
│   │   └── control_flow.rs  # Block/loop/if handling
│   ├── optimize/
│   │   ├── mod.rs           # Optimizer pass (Phase 6+)
│   │   └── passes/          # Individual optimization passes
│   ├── backend/
│   │   ├── mod.rs           # Backend trait
│   │   ├── safe.rs          # SafeBackend
│   │   ├── verified.rs      # VerifiedBackend
│   │   └── hybrid.rs        # HybridBackend
│   ├── codegen/
│   │   ├── mod.rs           # RustCodegen
│   │   ├── function.rs      # Function code generation
│   │   ├── module.rs        # Module wrapper generation
│   │   └── traits.rs        # Import/export trait generation
│   └── metadata/
│       ├── mod.rs           # VerificationMetadata parser
│       └── toml.rs          # TOML schema
└── tests/
    ├── simple.rs            # End-to-end tests
    └── fixtures/            # Test .wasm files
```

---

## Phase 2 Implementation Strategy

**Start small, validate incrementally**:

### Milestone 1: Hello World (Pure Math)
- Input: Wasm with one function, no imports, pure computation (add two numbers)
- Parser: Extract function type and body
- IR: Translate `local.get 0`, `local.get 1`, `i32.add`, `return` to IR
- Backend: SafeBackend only
- Codegen: Emit standalone Rust function
- Test: Compile generated Rust, call function, verify result

**Success criterion**: `fn add(a: i32, b: i32) -> i32` compiles and works

### Milestone 2: Memory Operations
- Input: Function that reads/writes memory
- IR: Add `Load` and `Store` instructions
- Backend: Emit `memory.load_i32(...)? `
- Test: Store value, load it back, verify roundtrip

### Milestone 3: Control Flow
- Input: Function with `if`, `block`, `br`
- IR: Build multi-block CFG with terminators
- Codegen: Labeled blocks + breaks
- Test: Conditional logic produces correct output

### Milestone 4: Module Wrapper
- Generate `Module<G, MAX_PAGES, TABLE_SIZE>` struct
- Initialize memory and globals
- Export functions as methods
- Test: Instantiate module, call methods

**By end of Phase 2**: Can transpile simple Wasm modules (no imports, no tables, no indirect calls) to safe Rust that compiles and runs correctly.

---

## Complexity Estimate

### Lines of Code Estimate
- Parser wrapper: ~300 LOC
- IR types & builder: ~800 LOC
- SafeBackend: ~500 LOC
- Codegen: ~600 LOC
- CLI & metadata: ~200 LOC
- **Phase 2 Total: ~2,400 LOC**

### Time Estimate (for experienced Rust developer)
- Milestone 1: 2-3 days
- Milestone 2: 1-2 days
- Milestone 3: 2-3 days
- Milestone 4: 1-2 days
- **Phase 2 Total: 6-10 days**

This is a **medium-large** feature, but the modular design makes it tractable. Each milestone is independently testable.

---

## Risks & Challenges

### 1. **Wasm Control Flow Complexity**
- Challenge: Wasm's `br`/`br_if`/`br_table` can jump to outer blocks
- Mitigation: Map Wasm blocks to Rust labeled blocks, `br N` → `break 'label_N`

### 2. **Stack Machine → SSA Translation**
- Challenge: Wasm stack needs to map to SSA variables correctly
- Mitigation: Well-understood algorithm (used by all Wasm engines), follow existing literature

### 3. **Type Inference**
- Challenge: Wasm is dynamically typed on the stack, Rust needs explicit types
- Mitigation: Wasm validation already infers types (via `wasmparser`), use those

### 4. **Error Handling**
- Challenge: Wasm traps vs Rust `Result`
- Mitigation: All Wasm traps map to `WasmTrap` enum, functions return `WasmResult<T>`

### 5. **Readability of Generated Code**
- Challenge: SSA form can be verbose (lots of temporaries)
- Mitigation:
  - Phase 2: Accept verbose output, focus on correctness
  - Phase 6+: Add optimizer to reduce temporaries

---

## Testing Strategy

### Overview: Four Testing Tiers

```
Tier 1: Unit tests         — fast, in-process, per-module
Tier 2: WAT integration    — inline WAT → transpile → compile → run (current build.rs approach)
Tier 3: C/Rust → Wasm E2E  — compile real source → .wasm → transpile → compile → run → compare
Tier 4: Differential fuzz  — random valid Wasm → transpile → compare against reference runtime
```

### Tier 1: Unit Tests

- IR builder: Wasm bytecode snippets → IR
- Backend: IR instructions → Rust code strings
- Codegen: IR functions → Rust functions

### Tier 2: WAT Integration Tests (current approach, keep as-is)

The `herkos-tests` crate uses `build.rs` to:
1. Define WAT snippets inline
2. Convert WAT → Wasm via `wat::parse_str()`
3. Transpile via `herkos::transpile()`
4. Generate `.rs` modules included at compile time
5. Test files call generated functions and assert results

This works well for small, targeted tests. **Keep it** — it's fast, self-contained, and needs no
external toolchain. But it can't test "real" programs compiled from C/Rust.

#### Property-based testing

Use `proptest` (optional dev-dependency) to verify Wasm semantics hold for arbitrary inputs:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_add_commutative(a: i32, b: i32) {
        let r1 = generated::add::func_0(a, b).unwrap();
        let r2 = generated::add::func_0(b, a).unwrap();
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn test_add_matches_rust(a: i32, b: i32) {
        let wasm_result = generated::add::func_0(a, b).unwrap();
        prop_assert_eq!(wasm_result, a.wrapping_add(b));
    }
}
```

This catches edge cases (overflow, zero, negative) that hand-written tests miss.

#### Golden file / snapshot testing (future)

Capture the generated Rust source as `.expected` files. On test runs, compare current output against
the snapshot. Any change requires explicit approval (`cargo test -- --bless` or similar). Useful for
catching unintended codegen regressions during refactoring.

### Tier 3: C/Rust → Wasm End-to-End Tests

**Goal**: Write a C or Rust program, compile it to `.wasm`, transpile with `herkos`, compile the
generated Rust, run it, and verify correctness — optionally by comparing output against a reference
Wasm runtime (`wasmtime`).

#### Why not Bazel?

Bazel is powerful for multi-language dependency graphs but is overkill here:
- Adds ~500MB+ tooling dependency and a separate build language (Starlark)
- The Rust ecosystem expects Cargo — Bazel's Rust rules are a maintenance burden
- Our actual cross-language step is narrow: just "compile C/Rust to .wasm" — not a full polyglot build
- Cargo's `build.rs` + a small task runner covers our needs

#### Recommended approach: `cargo xtask` + `wasi-sdk`

Use the **cargo xtask pattern** (a workspace member binary that acts as a custom task runner) to
orchestrate the C/Rust → Wasm compilation step, then feed results into the existing `build.rs`
test pipeline.

##### Toolchain requirements

| Source language | Compiler              | Target                      | Notes                                |
|-----------------|-----------------------|-----------------------------|--------------------------------------|
| C               | `wasi-sdk` (clang)    | `wasm32-wasi`               | Self-contained, no system deps       |
| Rust            | `rustc` + target      | `wasm32-unknown-unknown`    | `rustup target add` — already in CI  |
| WAT             | `wat` crate           | (in-process)                | Already used                         |

**`wasi-sdk`** is the simplest path for C. It bundles clang + sysroot for `wasm32-wasi` in a single
download (~100MB). No Emscripten needed. CI can cache it.

##### Directory layout

```
test_cases/
├── c/
│   ├── fibonacci.c          # C source files
│   ├── string_reverse.c
│   └── matrix_multiply.c
├── rust/
│   ├── collatz.rs            # Rust source files (compiled to wasm)
│   └── sorting.rs
├── wat/
│   ├── edge_cases.wat        # Hand-written WAT for edge cases
│   └── traps.wat
└── expected/
    ├── fibonacci.toml        # Expected outputs per test case
    ├── string_reverse.toml
    └── ...

crates/xtask/
├── Cargo.toml                # workspace member, [[bin]] name = "xtask"
└── src/
    └── main.rs               # orchestrates: compile sources → .wasm → copy to test fixtures
```

##### `cargo xtask` commands

```bash
# One-time setup: download wasi-sdk, add wasm32 target
cargo xtask setup

# Compile all test_cases/{c,rust,wat}/ → target/wasm-fixtures/*.wasm
cargo xtask build-test-wasm

# Run the full pipeline: build wasm → transpile → compile Rust → run → compare
cargo xtask test-e2e

# Add a new test case (scaffolds source + expected output)
cargo xtask new-test --lang c --name fibonacci
```

##### How `cargo xtask build-test-wasm` works

```
For each test_cases/c/*.c:
  1. wasi-sdk/bin/clang --target=wasm32-wasi -O2 -nostartfiles \
       -Wl,--no-entry -Wl,--export-all \
       -o target/wasm-fixtures/{name}.wasm {name}.c

For each test_cases/rust/*.rs:
  2. rustc --target wasm32-unknown-unknown --crate-type cdylib \
       -o target/wasm-fixtures/{name}.wasm {name}.rs

For each test_cases/wat/*.wat:
  3. wat2wasm (via `wat` crate or CLI) → target/wasm-fixtures/{name}.wasm
```

##### How `cargo xtask test-e2e` works (the full pipeline)

```
For each target/wasm-fixtures/*.wasm:
  1. Transpile:   cargo run -p herkos -- {name}.wasm -o target/e2e/{name}.rs
  2. Compile:     rustc target/e2e/{name}.rs --edition 2021 \
                    --extern herkos_runtime=... -o target/e2e/{name}
  3. Run:         target/e2e/{name} → capture output
  4. (Optional) Reference run:
                  wasmtime {name}.wasm → capture output
  5. Compare:     diff outputs, or compare against expected/ values
```

##### Integration with `build.rs`

The xtask-generated `.wasm` files can also be consumed by the existing `build.rs` in
`herkos-tests`. Add a fallback path:

```rust
// In build.rs: after inline WAT test cases, also pick up pre-compiled .wasm files
let fixture_dir = Path::new("../../target/wasm-fixtures");
if fixture_dir.exists() {
    for entry in fs::read_dir(fixture_dir)? {
        let path = entry?.path();
        if path.extension() == Some("wasm".as_ref()) {
            let name = path.file_stem().unwrap().to_str().unwrap();
            let wasm_bytes = fs::read(&path)?;
            let rust_code = transpile(&wasm_bytes, &options)?;
            // ... write to OUT_DIR, add to mod.rs
        }
    }
}
```

This means `cargo test` still works standalone (using inline WAT), but if you've run
`cargo xtask build-test-wasm` first, it also picks up the C/Rust-compiled fixtures.

##### Example: Adding a C test case

```c
// test_cases/c/fibonacci.c
__attribute__((export_name("fibonacci")))
int fibonacci(int n) {
    if (n <= 1) return n;
    int a = 0, b = 1;
    for (int i = 2; i <= n; i++) {
        int tmp = a + b;
        a = b;
        b = tmp;
    }
    return b;
}
```

```toml
# test_cases/expected/fibonacci.toml
[[cases]]
function = "fibonacci"
args = [0]
expected = 0

[[cases]]
function = "fibonacci"
args = [1]
expected = 1

[[cases]]
function = "fibonacci"
args = [10]
expected = 55

[[cases]]
function = "fibonacci"
args = [20]
expected = 6765
```

##### Example: Adding a Rust test case

```rust
// test_cases/rust/collatz.rs
#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn collatz_steps(mut n: i32) -> i32 {
    let mut steps = 0i32;
    while n != 1 {
        if n % 2 == 0 {
            n /= 2;
        } else {
            n = n.wrapping_mul(3).wrapping_add(1);
        }
        steps = steps.wrapping_add(1);
    }
    steps
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
```

#### Why this approach over alternatives

| Approach            | Pros                                      | Cons                                              |
|---------------------|-------------------------------------------|---------------------------------------------------|
| **Bazel**           | Hermetic, parallel, caches aggressively   | Huge dep, fights Cargo, Starlark learning curve   |
| **Makefile/just**   | Simple, universal                         | No Cargo integration, shell scripting gets messy  |
| **Pure build.rs**   | Zero extra tools                          | Can't invoke clang, slow for many files           |
| **cargo xtask** ✓   | Rust-native, workspace member, composable | Small amount of custom code (~200 LOC)            |
| **Nix**             | Perfectly reproducible                    | Steep learning curve, not standard in Rust        |

#### CI integration

```yaml
# .github/workflows/ci.yml additions
- name: Install wasi-sdk
  run: |
    curl -LO https://github.com/aspect-build/sysroot-wasm/releases/download/v${WASI_SDK_VERSION}/wasi-sdk-${WASI_SDK_VERSION}.0-x86_64-linux.tar.gz
    tar xf wasi-sdk-*.tar.gz
    echo "WASI_SDK_PATH=$(pwd)/wasi-sdk-${WASI_SDK_VERSION}.0" >> $GITHUB_ENV

- name: Add wasm32 target
  run: rustup target add wasm32-unknown-unknown

- name: Build test Wasm fixtures
  run: cargo xtask build-test-wasm

- name: Run E2E tests
  run: cargo xtask test-e2e
```

#### Optional: Reference runtime comparison with `wasmtime`

For high confidence, run each `.wasm` through `wasmtime` and compare outputs against the transpiled
Rust. This catches semantic divergences:

```bash
# In cargo xtask test-e2e:
wasmtime run --invoke fibonacci fibonacci.wasm 10
# Output: 55
# Compare against: target/e2e/fibonacci --invoke fibonacci 10
```

Add `wasmtime-cli` as an optional CI dependency. Not required for local dev — the `expected/` TOML
files serve as the ground truth when wasmtime isn't available.

### Tier 4: Differential Fuzzing (Phase 6+)

- Generate random valid Wasm modules (via `wasm-smith` crate)
- Transpile to Rust with `herkos`
- Execute both the original Wasm (via `wasmtime`) and the transpiled Rust
- Compare: return values, memory contents, trap behavior
- Any divergence is a bug

#### Wasm Test Suite

The official [WebAssembly spec test suite](https://github.com/aspect-build/spectest) contains
thousands of `.wast` test cases. Once the transpiler is mature enough (Phase 5+), run the full
spec test suite through the pipeline as a conformance check.

### Summary: What to use when

| I want to...                          | Use this                              |
|---------------------------------------|---------------------------------------|
| Test a small Wasm pattern             | Inline WAT in `build.rs` (Tier 2)    |
| Test a real C program end-to-end      | `cargo xtask test-e2e` (Tier 3)      |
| Test a real Rust program end-to-end   | `cargo xtask test-e2e` (Tier 3)      |
| Add a quick regression test           | WAT in `build.rs` (Tier 2)           |
| Validate spec conformance             | Spec test suite (Tier 4)             |
| Find edge-case bugs                   | Differential fuzzing (Tier 4)        |
| Run everything in CI                  | `cargo test` + `cargo xtask test-e2e`|

---

## Open Questions

1. **Variable naming**: Use `v0`, `v1` (SSA style) or try to recover names from DWARF debug info?
   - **Answer for Phase 2**: SSA style (`v0`, `v1`) - simple, predictable
   - **Future**: DWARF support for better names

2. **Indirect calls**: How to dispatch `call_indirect` without dynamic dispatch overhead?
   - **Answer**: Match statement over `func_index` — see "Function Call Strategy" section above and SPECIFICATION.md §8.5. The transpiler enumerates all functions with a matching canonical type index as match arms. LLVM optimizes dense arms into a jump table. Structural type equivalence is handled via a canonical type index mapping computed at transpile time.

3. **Initialization**: How to handle `start` function, data/element segments?
   - **Answer**: Generate `::new()` that runs initialization, populate memory/tables

4. **Multi-memory/multi-table** (Wasm 2.0)?
   - **Phase 2**: Single memory, single table only
   - **Phase 7+**: Extend to multi-memory

---

## References

- [wasmparser documentation](https://docs.rs/wasmparser/)
- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [SSA Book](https://pfalcon.github.io/ssabook/latest/book-full.pdf) (Chapter on translation from bytecode)
- [Cranelift IR](https://github.com/bytecodealliance/wasmtime/tree/main/cranelift) (reference for Wasm → IR design)

---

**Status**: Design proposal — ready for review and Phase 2 kickoff
