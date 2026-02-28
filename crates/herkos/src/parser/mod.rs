//! WebAssembly module parser.
//!
//! This module wraps the `wasmparser` crate to extract structured information
//! from `.wasm` binary files.

use anyhow::{Context, Result};
use wasmparser::{ExternalKind, FuncType, Parser, Payload, TypeRef, ValType};

/// Memory information from the Wasm module.
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// Initial size in pages (64 KiB each)
    pub initial_pages: u32,

    /// Maximum size in pages (None = unlimited, up to implementation limit)
    pub maximum_pages: Option<u32>,
}

/// Information about a single Wasm global variable.
#[derive(Debug, Clone)]
pub struct GlobalInfo {
    /// The value type of the global (i32, i64, f32, f64).
    pub val_type: ValType,
    /// Whether the global is mutable.
    pub mutable: bool,
    /// The constant initializer value.
    pub init_value: InitValue,
}

/// Parsed constant initializer expression.
/// Wasm MVP globals are initialized with a single constant instruction.
#[derive(Debug, Clone, Copy)]
pub enum InitValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

/// Table declaration from the Wasm module.
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Initial number of table entries.
    pub initial_size: u32,
    /// Maximum number of table entries (None = no limit declared).
    pub max_size: Option<u32>,
}

/// An active element segment to initialize a table.
#[derive(Debug, Clone)]
pub struct ElementSegment {
    /// Starting offset in the table (from the i32.const in the offset expression).
    pub offset: u32,
    /// Function indices to place into the table starting at `offset`.
    pub func_indices: Vec<u32>,
}

/// An active data segment to initialize memory.
#[derive(Debug, Clone)]
pub struct DataSegment {
    /// Byte offset into memory 0 (from the i32.const in the offset expression).
    pub offset: u32,
    /// Raw data bytes to copy into memory at initialization.
    pub data: Vec<u8>,
}

/// An export from the Wasm module.
#[derive(Debug, Clone)]
pub struct ExportInfo {
    /// The exported name (used as the Rust method name).
    pub name: String,
    /// What kind of item is exported.
    pub kind: ExportKind,
    /// Index into the corresponding index space.
    pub index: u32,
}

/// Kind of export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Func,
    Table,
    Memory,
    Global,
}

/// An import from the Wasm module.
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// The import module name (e.g., "env").
    pub module_name: String,
    /// The import field name (e.g., "log").
    pub name: String,
    /// What kind of item is imported.
    pub kind: ImportKind,
}

/// Kind of import.
#[derive(Debug, Clone)]
pub enum ImportKind {
    /// Imported function (index into the type section).
    Function(u32),
    /// Imported global.
    Global { val_type: ValType, mutable: bool },
    /// Imported memory.
    Memory {
        initial_pages: u32,
        maximum_pages: Option<u32>,
    },
    /// Imported table.
    Table {
        initial_size: u32,
        max_size: Option<u32>,
    },
}

/// Parsed WebAssembly module.
#[derive(Debug, Clone)]
pub struct ParsedModule {
    /// Type section: function signatures
    pub types: Vec<FuncType>,

    /// Functions (index into types + bytecode)
    pub functions: Vec<ParsedFunction>,

    /// Memory (Milestone 2)
    /// Wasm MVP has at most one memory (index 0)
    pub memory: Option<MemoryInfo>,

    /// Table declaration (Wasm MVP has at most one table, index 0)
    pub table: Option<TableInfo>,

    /// Element segments for table initialization
    pub element_segments: Vec<ElementSegment>,

    /// Global variables (Milestone 4)
    pub globals: Vec<GlobalInfo>,

    /// Data segments for memory initialization (Milestone 4)
    pub data_segments: Vec<DataSegment>,

    /// Exports (Milestone 4)
    pub exports: Vec<ExportInfo>,

    /// Imports from the host environment
    pub imports: Vec<ImportInfo>,

    /// Number of imported functions (these occupy indices 0..N-1 in the
    /// function index space, before local functions).
    pub num_imported_functions: u32,

    /// Number of imported globals (these occupy indices 0..N-1 in the
    /// global index space, before local globals).
    pub num_imported_globals: u32,

    /// Wasm binary version from the module header.
    pub wasm_version: u16,
}

/// A single function in the module.
#[derive(Debug, Clone)]
pub struct ParsedFunction {
    /// Index into the types section
    pub type_idx: u32,

    /// Local variable types (parameters are in the function type)
    pub locals: Vec<ValType>,

    /// Function body (Wasm bytecode)
    pub body: Vec<u8>,
}

/// Evaluate a wasmparser ConstExpr into our InitValue.
/// Wasm MVP globals use a single i32.const/i64.const/f32.const/f64.const instruction.
fn eval_const_expr(const_expr: wasmparser::ConstExpr) -> Result<InitValue> {
    let mut reader = const_expr.get_operators_reader();
    let op = reader.read().context("reading const expr operator")?;
    match op {
        wasmparser::Operator::I32Const { value } => Ok(InitValue::I32(value)),
        wasmparser::Operator::I64Const { value } => Ok(InitValue::I64(value)),
        wasmparser::Operator::F32Const { value } => {
            Ok(InitValue::F32(f32::from_bits(value.bits())))
        }
        wasmparser::Operator::F64Const { value } => {
            Ok(InitValue::F64(f64::from_bits(value.bits())))
        }
        _ => anyhow::bail!("Unsupported const expression operator: {:?}", op),
    }
}

/// Parse an active element segment, or return None for passive/declared segments.
fn parse_element_segment(element: wasmparser::Element) -> Result<Option<ElementSegment>> {
    match element.kind {
        wasmparser::ElementKind::Active {
            table_index,
            offset_expr,
        } => {
            // table_index is Option<u32>; None means table 0 (MVP default)
            let tidx = table_index.unwrap_or(0);
            if tidx != 0 {
                anyhow::bail!(
                    "Multi-table element segments not supported (table_index={})",
                    tidx
                );
            }

            let offset = match eval_const_expr(offset_expr)? {
                InitValue::I32(v) => v as u32,
                _ => anyhow::bail!("Element segment offset must be i32"),
            };

            // Collect function indices from element items
            let mut func_indices = Vec::new();
            match element.items {
                wasmparser::ElementItems::Functions(funcs) => {
                    for func_idx in funcs {
                        let idx = func_idx.context("reading element func index")?;
                        func_indices.push(idx);
                    }
                }
                wasmparser::ElementItems::Expressions(..) => {
                    anyhow::bail!("Expression-based element segments not supported");
                }
            }

            Ok(Some(ElementSegment {
                offset,
                func_indices,
            }))
        }
        wasmparser::ElementKind::Passive | wasmparser::ElementKind::Declared => {
            // Passive segments are not copied into the table at start-up; they are only
            // activated via `table.init` / `elem.drop` (bulk-memory proposal). Declared
            // segments exist solely to mark functions referenced by `ref.func`. Neither
            // kind maps to anything in our static table model, so we skip them.
            Ok(None)
        }
    }
}

/// Parse an active data segment, or return None for passive segments.
fn parse_data_segment(data: wasmparser::Data) -> Result<Option<DataSegment>> {
    match data.kind {
        wasmparser::DataKind::Active {
            memory_index: 0,
            offset_expr,
        } => {
            let offset = match eval_const_expr(offset_expr)? {
                InitValue::I32(v) => v as u32,
                _ => anyhow::bail!("Data segment offset must be i32"),
            };
            Ok(Some(DataSegment {
                offset,
                data: data.data.to_vec(),
            }))
        }
        wasmparser::DataKind::Passive => {
            // Skip passive data segments (used with memory.init)
            Ok(None)
        }
        wasmparser::DataKind::Active { memory_index, .. } => {
            anyhow::bail!(
                "Multi-memory data segments not supported (memory_index={})",
                memory_index
            );
        }
    }
}

/// Parse a function code section entry, extracting locals and bytecode.
fn parse_code_entry(body: wasmparser::FunctionBody, type_idx: u32) -> Result<ParsedFunction> {
    // Extract locals
    let mut locals = Vec::new();
    let locals_reader = body.get_locals_reader().context("getting locals reader")?;
    for local in locals_reader {
        let (count, val_type) = local.context("reading local")?;
        for _ in 0..count {
            locals.push(val_type);
        }
    }

    // Extract operators as raw bytes (parsed later in the IR builder)
    let operators_reader = body
        .get_operators_reader()
        .context("getting operators reader")?;
    let mut binary_reader = operators_reader.get_binary_reader();
    let remaining = binary_reader.bytes_remaining();
    let body_bytes = binary_reader
        .read_bytes(remaining)
        .context("reading body bytes")?;

    Ok(ParsedFunction {
        type_idx,
        locals,
        body: body_bytes.to_vec(),
    })
}

/// Parse a WebAssembly binary into a structured module.
pub fn parse_wasm(wasm_bytes: &[u8]) -> Result<ParsedModule> {
    let parser = Parser::new(0);

    let mut types = Vec::new();
    let mut function_types: Vec<u32> = Vec::new(); // type index for each function
    let mut functions = Vec::new();
    let mut memory: Option<MemoryInfo> = None;
    let mut table: Option<TableInfo> = None;
    let mut element_segments = Vec::new();
    let mut globals = Vec::new();
    let mut data_segments = Vec::new();
    let mut exports = Vec::new();
    let mut imports = Vec::new();
    let mut num_imported_functions: u32 = 0;
    let mut num_imported_globals: u32 = 0;
    let mut wasm_version: u16 = 1;

    for payload in parser.parse_all(wasm_bytes) {
        let payload = payload.context("parsing wasm payload")?;

        match payload {
            Payload::Version { num, .. } => {
                wasm_version = num;
            }

            Payload::TypeSection(reader) => {
                for rec_group in reader {
                    let rec_group = rec_group.context("reading rec group")?;
                    // RecGroup contains SubTypes, each with a composite type
                    for sub_type in rec_group.types() {
                        // CompositeType has an 'inner' field with the actual type
                        match &sub_type.composite_type.inner {
                            wasmparser::CompositeInnerType::Func(func_ty) => {
                                types.push(func_ty.clone());
                            }
                            _ => {
                                // Skip non-function types (arrays, structs, conts from the GC proposal).
                                // herkos targets MVP + WASI Wasm, which only uses function types.
                                // GC proposal types have no role in the current memory model or
                                // codegen pipeline and are deferred to a later milestone.
                            }
                        }
                    }
                }
            }

            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import.context("reading import")?;
                    let kind = match import.ty {
                        TypeRef::Func(type_idx) => {
                            num_imported_functions += 1;
                            ImportKind::Function(type_idx)
                        }
                        TypeRef::Global(global_ty) => {
                            num_imported_globals += 1;
                            ImportKind::Global {
                                val_type: global_ty.content_type,
                                mutable: global_ty.mutable,
                            }
                        }
                        TypeRef::Memory(mem_ty) => ImportKind::Memory {
                            initial_pages: mem_ty.initial as u32,
                            maximum_pages: mem_ty.maximum.map(|m| m as u32),
                        },
                        TypeRef::Table(table_ty) => ImportKind::Table {
                            initial_size: table_ty.initial as u32,
                            max_size: table_ty.maximum.map(|m| m as u32),
                        },
                        _ => continue,
                    };
                    imports.push(ImportInfo {
                        module_name: import.module.to_string(),
                        name: import.name.to_string(),
                        kind,
                    });
                }
            }

            Payload::FunctionSection(reader) => {
                for func_type_idx in reader {
                    let func_type_idx = func_type_idx.context("reading function type index")?;
                    function_types.push(func_type_idx);
                }
            }

            Payload::CodeSectionEntry(body) => {
                let type_idx = function_types[functions.len()]; // Match with function section
                let parsed_func = parse_code_entry(body, type_idx)?;
                functions.push(parsed_func);
            }

            Payload::MemorySection(reader) => {
                // Wasm MVP: at most one memory (index 0)
                if let Some(mem) = reader.into_iter().next() {
                    let memory_type = mem.context("reading memory type")?;
                    memory = Some(MemoryInfo {
                        initial_pages: memory_type.initial as u32,
                        maximum_pages: memory_type.maximum.map(|m| m as u32),
                    });
                }
            }

            Payload::TableSection(reader) => {
                // Wasm MVP: at most one table (index 0), always funcref
                if let Some(tbl) = reader.into_iter().next() {
                    let tbl = tbl.context("reading table type")?;
                    table = Some(TableInfo {
                        initial_size: tbl.ty.initial as u32,
                        max_size: tbl.ty.maximum.map(|m| m as u32),
                    });
                }
            }

            Payload::ElementSection(reader) => {
                for element in reader {
                    let element = element.context("reading element segment")?;
                    if let Some(segment) = parse_element_segment(element)? {
                        element_segments.push(segment);
                    }
                }
            }

            Payload::GlobalSection(reader) => {
                for global in reader {
                    let global = global.context("reading global")?;
                    let init_value = eval_const_expr(global.init_expr)?;
                    globals.push(GlobalInfo {
                        val_type: global.ty.content_type,
                        mutable: global.ty.mutable,
                        init_value,
                    });
                }
            }

            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.context("reading export")?;
                    let kind = match export.kind {
                        ExternalKind::Func => ExportKind::Func,
                        ExternalKind::Table => ExportKind::Table,
                        ExternalKind::Memory => ExportKind::Memory,
                        ExternalKind::Global => ExportKind::Global,
                        ExternalKind::Tag => continue,
                    };
                    exports.push(ExportInfo {
                        name: export.name.to_string(),
                        kind,
                        index: export.index,
                    });
                }
            }

            Payload::DataSection(reader) => {
                for data in reader {
                    let data = data.context("reading data segment")?;
                    if let Some(segment) = parse_data_segment(data)? {
                        data_segments.push(segment);
                    }
                }
            }

            _ => {}
        }
    }

    Ok(ParsedModule {
        types,
        functions,
        memory,
        table,
        element_segments,
        globals,
        data_segments,
        exports,
        imports,
        num_imported_functions,
        num_imported_globals,
        wasm_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_module() {
        // Minimal Wasm module (empty)
        let wat = r#"
            (module)
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();
        assert_eq!(module.types.len(), 0);
        assert_eq!(module.functions.len(), 0);
    }

    #[test]
    fn parse_add_function() {
        let wat = r#"
            (module
                (func (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();
        assert_eq!(module.types.len(), 1);
        assert_eq!(module.functions.len(), 1);
        assert!(module.memory.is_none());
    }

    #[test]
    fn parse_memory_section() {
        let wat = r#"
            (module
                (memory 2 10)
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();
        let memory = module.memory.expect("module should have memory");
        assert_eq!(memory.initial_pages, 2);
        assert_eq!(memory.maximum_pages, Some(10));
    }

    #[test]
    fn parse_memory_no_max() {
        let wat = r#"
            (module
                (memory 1)
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();
        let memory = module.memory.expect("module should have memory");
        assert_eq!(memory.initial_pages, 1);
        assert_eq!(memory.maximum_pages, None);
    }

    #[test]
    fn parse_mutable_global() {
        let wat = r#"
            (module
                (global (mut i32) (i32.const 42))
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();
        assert_eq!(module.globals.len(), 1);
        assert!(module.globals[0].mutable);
        assert_eq!(module.globals[0].val_type, ValType::I32);
        match module.globals[0].init_value {
            InitValue::I32(v) => assert_eq!(v, 42),
            _ => panic!("expected I32 init value"),
        }
    }

    #[test]
    fn parse_immutable_global() {
        let wat = r#"
            (module
                (global i64 (i64.const 999))
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();
        assert_eq!(module.globals.len(), 1);
        assert!(!module.globals[0].mutable);
        assert_eq!(module.globals[0].val_type, ValType::I64);
        match module.globals[0].init_value {
            InitValue::I64(v) => assert_eq!(v, 999),
            _ => panic!("expected I64 init value"),
        }
    }

    #[test]
    fn parse_exports() {
        let wat = r#"
            (module
                (func (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.add)
                (export "add" (func 0))
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.exports[0].name, "add");
        assert_eq!(module.exports[0].kind, ExportKind::Func);
        assert_eq!(module.exports[0].index, 0);
    }

    #[test]
    fn parse_data_segment() {
        let wat = r#"
            (module
                (memory 1)
                (data (i32.const 16) "Hello")
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();
        assert_eq!(module.data_segments.len(), 1);
        assert_eq!(module.data_segments[0].offset, 16);
        assert_eq!(module.data_segments[0].data, b"Hello");
    }

    #[test]
    fn parse_multiple_exports() {
        let wat = r#"
            (module
                (func (result i32) i32.const 1)
                (func (result i32) i32.const 2)
                (export "first" (func 0))
                (export "second" (func 1))
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();
        assert_eq!(module.exports.len(), 2);
        assert_eq!(module.exports[0].name, "first");
        assert_eq!(module.exports[1].name, "second");
    }

    #[test]
    fn parse_function_import() {
        let wat = r#"
            (module
                (import "env" "log" (func (param i32)))
                (func (result i32)
                    i32.const 42
                )
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();

        // Check import was parsed correctly
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.imports[0].module_name, "env");
        assert_eq!(module.imports[0].name, "log");
        match &module.imports[0].kind {
            ImportKind::Function(type_idx) => {
                // Should reference type 0 (param i32)
                assert_eq!(*type_idx, 0);
            }
            _ => panic!("Expected function import"),
        }

        // Check import counter
        assert_eq!(module.num_imported_functions, 1);
        assert_eq!(module.num_imported_globals, 0);

        // Check that we still have 1 local function
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn parse_mixed_imports() {
        let wat = r#"
            (module
                (import "env" "print" (func (param i32)))
                (import "env" "read" (func (result i32)))
                (import "env" "counter" (global i32))
                (import "env" "flag" (global (mut i32)))
                (global (mut i32) (i32.const 100))
                (func (param i32) (result i32)
                    local.get 0
                    i32.const 1
                    i32.add
                )
                (func (result i32)
                    i32.const 99
                )
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = parse_wasm(&wasm).unwrap();

        // Check all imports were parsed
        assert_eq!(module.imports.len(), 4);

        // Check first two are function imports
        assert_eq!(module.imports[0].module_name, "env");
        assert_eq!(module.imports[0].name, "print");
        match &module.imports[0].kind {
            ImportKind::Function(_) => {}
            _ => panic!("Expected function import"),
        }

        assert_eq!(module.imports[1].module_name, "env");
        assert_eq!(module.imports[1].name, "read");
        match &module.imports[1].kind {
            ImportKind::Function(_) => {}
            _ => panic!("Expected function import"),
        }

        // Check next two are global imports
        assert_eq!(module.imports[2].module_name, "env");
        assert_eq!(module.imports[2].name, "counter");
        match &module.imports[2].kind {
            ImportKind::Global { val_type, mutable } => {
                assert_eq!(*val_type, ValType::I32);
                assert!(!mutable);
            }
            _ => panic!("Expected global import"),
        }

        assert_eq!(module.imports[3].module_name, "env");
        assert_eq!(module.imports[3].name, "flag");
        match &module.imports[3].kind {
            ImportKind::Global { val_type, mutable } => {
                assert_eq!(*val_type, ValType::I32);
                assert!(mutable);
            }
            _ => panic!("Expected global import"),
        }

        // Check counters
        assert_eq!(module.num_imported_functions, 2);
        assert_eq!(module.num_imported_globals, 2);

        // Check local definitions
        assert_eq!(module.functions.len(), 2);
        assert_eq!(module.globals.len(), 1); // Only local globals, not imports
    }
}
