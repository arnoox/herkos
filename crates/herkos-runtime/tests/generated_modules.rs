//! Integration tests — examples of what transpiler-generated code looks like.
//!
//! Each example below corresponds to a realistic Wasm module. The struct
//! definitions, `new()` constructors, and exported function bodies are
//! exactly what `herkos` would emit. These tests validate that the
//! runtime API composes correctly.

use herkos_runtime::*;

// ═══════════════════════════════════════════════════════════════════════
// Example 1: Pure computation — no memory, no table, no globals
// ═══════════════════════════════════════════════════════════════════════
//
// Source Wasm:
//   (module
//     (func (export "add") (param i32 i32) (result i32)
//       local.get 0  local.get 1  i32.add))

mod pure_add {
    use super::*;

    // Transpiler output: no memory → LibraryModule, no globals → (), no table → 0
    type PureAdd = LibraryModule<(), 0>;

    fn new() -> PureAdd {
        LibraryModule::new((), Table::try_new(0).unwrap())
    }

    // Exported function — pure computation, no memory parameter needed.
    fn export_add(_module: &PureAdd, a: i32, b: i32) -> WasmResult<i32> {
        // Wasm: local.get 0, local.get 1, i32.add
        Ok(a.wrapping_add(b))
    }

    #[test]
    fn test_add() {
        let m = new();
        assert_eq!(export_add(&m, 2, 3), Ok(5));
        assert_eq!(export_add(&m, i32::MAX, 1), Ok(i32::MIN)); // wrapping
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Example 2: Counter — mutable global, no memory
// ═══════════════════════════════════════════════════════════════════════
//
// Source Wasm:
//   (module
//     (global $count (mut i32) (i32.const 0))
//     (func (export "increment") (result i32)
//       global.get $count  i32.const 1  i32.add
//       global.set $count  global.get $count)
//     (func (export "get") (result i32)
//       global.get $count))

mod counter {
    use super::*;

    struct Globals {
        count: i32, // (global $count (mut i32) (i32.const 0))
    }

    type Counter = LibraryModule<Globals, 0>;

    fn new() -> Counter {
        LibraryModule::new(Globals { count: 0 }, Table::try_new(0).unwrap())
    }

    fn export_increment(module: &mut Counter) -> WasmResult<i32> {
        // global.get $count, i32.const 1, i32.add, global.set $count
        module.globals.count = module.globals.count.wrapping_add(1);
        // global.get $count
        Ok(module.globals.count)
    }

    fn export_get(module: &Counter) -> WasmResult<i32> {
        Ok(module.globals.count)
    }

    #[test]
    fn test_counter() {
        let mut m = new();
        assert_eq!(export_get(&m), Ok(0));
        assert_eq!(export_increment(&mut m), Ok(1));
        assert_eq!(export_increment(&mut m), Ok(2));
        assert_eq!(export_get(&m), Ok(2));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Example 3: Memory buffer — sum an array stored in linear memory
// ═══════════════════════════════════════════════════════════════════════
//
// Source Wasm:
//   (module
//     (memory (export "memory") 1)
//     (func (export "sum_array") (param $ptr i32) (param $len i32) (result i32)
//       (local $i i32) (local $sum i32)
//       ... loop: load i32 at ptr+i*4, add to sum ...))

mod memory_sum {
    use super::*;

    // No globals needed for this module → G = ()
    type MemModule = Module<(), 1, 0>;

    fn new() -> MemModule {
        Module::try_new(1, (), Table::try_new(0).unwrap()).unwrap()
    }

    // Internal function (not exported) — the transpiler emits these as
    // free functions taking &memory and &globals.
    fn func_sum_array(memory: &IsolatedMemory<1>, ptr: i32, len: i32) -> WasmResult<i32> {
        let mut sum: i32 = 0;
        let mut i: i32 = 0;
        // Wasm loop → Rust loop
        loop {
            if i >= len {
                break;
            }
            // i32.load at ptr + i*4
            let offset = (ptr as u32).wrapping_add((i as u32).wrapping_mul(4)) as usize;
            let val = memory.load_i32(offset)?;
            sum = sum.wrapping_add(val);
            i = i.wrapping_add(1);
        }
        Ok(sum)
    }

    // Exported function — dispatches to internal function with module state.
    fn export_sum_array(module: &MemModule, ptr: i32, len: i32) -> WasmResult<i32> {
        func_sum_array(&module.memory, ptr, len)
    }

    #[test]
    fn test_sum_array() {
        let mut m = new();

        // Write [10, 20, 30, 40] to memory at offset 0.
        m.memory.store_i32(0, 10).unwrap();
        m.memory.store_i32(4, 20).unwrap();
        m.memory.store_i32(8, 30).unwrap();
        m.memory.store_i32(12, 40).unwrap();

        assert_eq!(export_sum_array(&m, 0, 4), Ok(100));
        assert_eq!(export_sum_array(&m, 4, 2), Ok(50)); // subset
        assert_eq!(export_sum_array(&m, 0, 0), Ok(0)); // empty
    }

    #[test]
    fn test_sum_array_out_of_bounds() {
        let m = new();
        // ptr near end of memory → load will go out of bounds
        let result = export_sum_array(&m, (PAGE_SIZE - 2) as i32, 1);
        assert_eq!(result, Err(WasmTrap::OutOfBounds));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Example 4: Indirect calls — function table dispatch
// ═══════════════════════════════════════════════════════════════════════
//
// Source Wasm:
//   (module
//     (type $binop (func (param i32 i32) (result i32)))
//     (table 2 funcref)
//     (elem (i32.const 0) $add $mul)
//     (func $add (param i32 i32) (result i32) local.get 0 local.get 1 i32.add)
//     (func $mul (param i32 i32) (result i32) local.get 0 local.get 1 i32.mul)
//     (func (export "apply") (param $op i32) (param $a i32) (param $b i32) (result i32)
//       local.get 1  local.get 2  local.get 0  call_indirect (type $binop)))

mod indirect_call {
    use super::*;

    // Type section: type 0 = (i32, i32) -> i32
    const TYPE_BINOP: u32 = 0;

    type IndirectModule = LibraryModule<(), 4>;

    fn new() -> IndirectModule {
        let mut table = Table::<4>::try_new(2).unwrap();
        // Element segment: table[0] = func 0 (add), table[1] = func 1 (mul)
        table
            .set(
                0,
                Some(FuncRef {
                    type_index: 0,
                    func_index: 0,
                }),
            )
            .unwrap();
        table
            .set(
                1,
                Some(FuncRef {
                    type_index: 0,
                    func_index: 1,
                }),
            )
            .unwrap();
        LibraryModule::new((), table)
    }

    // Internal functions (func_index 0 and 1)
    fn func_add(_globals: &(), a: i32, b: i32) -> WasmResult<i32> {
        Ok(a.wrapping_add(b))
    }

    fn func_mul(_globals: &(), a: i32, b: i32) -> WasmResult<i32> {
        Ok(a.wrapping_mul(b))
    }

    // Transpiler-generated dispatch for call_indirect with type $binop.
    // This is the match-based dispatch over func_index.
    fn call_indirect_type0(
        module: &IndirectModule,
        table_index: u32,
        a: i32,
        b: i32,
    ) -> WasmResult<i32> {
        let entry = module.table.get(table_index)?;
        if entry.type_index != TYPE_BINOP {
            return Err(WasmTrap::IndirectCallTypeMismatch);
        }
        match entry.func_index {
            0 => func_add(&module.globals, a, b),
            1 => func_mul(&module.globals, a, b),
            _ => Err(WasmTrap::UndefinedElement),
        }
    }

    fn export_apply(module: &IndirectModule, op: i32, a: i32, b: i32) -> WasmResult<i32> {
        call_indirect_type0(module, op as u32, a, b)
    }

    #[test]
    fn test_indirect_add() {
        let m = new();
        assert_eq!(export_apply(&m, 0, 7, 3), Ok(10)); // table[0] = add
    }

    #[test]
    fn test_indirect_mul() {
        let m = new();
        assert_eq!(export_apply(&m, 1, 7, 3), Ok(21)); // table[1] = mul
    }

    #[test]
    fn test_indirect_out_of_bounds() {
        let m = new();
        assert_eq!(export_apply(&m, 5, 1, 1), Err(WasmTrap::TableOutOfBounds));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Example 5: Library module — operates on borrowed memory
// ═══════════════════════════════════════════════════════════════════════
//
// Source Wasm (imports memory):
//   (module
//     (import "env" "memory" (memory 1))
//     (func (export "memset") (param $ptr i32) (param $val i32) (param $len i32)
//       ... loop: store val as byte at ptr+i ...))

mod library_memset {
    use super::*;

    type MemsetLib = LibraryModule<(), 0>;

    fn new() -> MemsetLib {
        LibraryModule::new((), Table::try_new(0).unwrap())
    }

    // Library functions take &mut IsolatedMemory as a parameter —
    // they borrow the caller's memory for the duration of the call.
    fn export_memset<const MAX_PAGES: usize>(
        _module: &MemsetLib,
        memory: &mut IsolatedMemory<MAX_PAGES>,
        ptr: i32,
        val: i32,
        len: i32,
    ) -> WasmResult<()> {
        let byte = val as u8;
        let mut i: i32 = 0;
        loop {
            if i >= len {
                break;
            }
            let offset = (ptr as u32).wrapping_add(i as u32) as usize;
            memory.store_u8(offset, byte)?;
            i = i.wrapping_add(1);
        }
        Ok(())
    }

    #[test]
    fn test_memset() {
        let lib = new();
        // Caller owns the memory.
        let mut memory = IsolatedMemory::<1>::try_new(1).unwrap();

        // Caller lends memory to library for the call.
        export_memset(&lib, &mut memory, 100, 0xAB, 4).unwrap();

        assert_eq!(memory.load_u8(100), Ok(0xAB));
        assert_eq!(memory.load_u8(101), Ok(0xAB));
        assert_eq!(memory.load_u8(102), Ok(0xAB));
        assert_eq!(memory.load_u8(103), Ok(0xAB));
        assert_eq!(memory.load_u8(104), Ok(0)); // untouched
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Example 6: Inter-module call — caller lends memory to library
// ═══════════════════════════════════════════════════════════════════════
//
// Scenario: an "app" module owns memory and calls a "codec" library
// to encode data in-place. The codec borrows the app's memory.

mod inter_module {
    use super::*;

    // ── Codec library (imports memory) ──

    struct CodecGlobals {
        xor_key: i32,
    }

    type Codec = LibraryModule<CodecGlobals, 0>;

    fn codec_new() -> Codec {
        LibraryModule::new(CodecGlobals { xor_key: 0x55 }, Table::try_new(0).unwrap())
    }

    /// XOR-encode `len` bytes starting at `ptr`.
    fn codec_export_encode<const MP: usize>(
        codec: &Codec,
        memory: &mut IsolatedMemory<MP>,
        ptr: i32,
        len: i32,
    ) -> WasmResult<()> {
        let key = codec.globals.xor_key as u8;
        let mut i: i32 = 0;
        loop {
            if i >= len {
                break;
            }
            let offset = (ptr as u32).wrapping_add(i as u32) as usize;
            let byte = memory.load_u8(offset)?;
            memory.store_u8(offset, byte ^ key)?;
            i = i.wrapping_add(1);
        }
        Ok(())
    }

    // ── App module (owns memory, calls codec) ──

    type App = Module<(), 1, 0>;

    fn app_new() -> App {
        Module::try_new(1, (), Table::try_new(0).unwrap()).unwrap()
    }

    #[test]
    fn test_inter_module_call() {
        let mut app = app_new();
        let codec = codec_new();

        // App writes data to its own memory.
        app.memory.store_u8(0, 0x48).unwrap(); // 'H'
        app.memory.store_u8(1, 0x65).unwrap(); // 'e'
        app.memory.store_u8(2, 0x6C).unwrap(); // 'l'
        app.memory.store_u8(3, 0x6C).unwrap(); // 'l'

        // App lends its memory to the codec — Rust borrow checker guarantees:
        // 1. Codec cannot store the reference beyond this call
        // 2. App cannot use its memory while codec holds it
        // 3. Codec cannot access any other module's memory
        codec_export_encode(&codec, &mut app.memory, 0, 4).unwrap();

        // After the call, app has full ownership again.
        // Data is now XOR'd with 0x55.
        assert_eq!(app.memory.load_u8(0), Ok(0x48 ^ 0x55));
        assert_eq!(app.memory.load_u8(1), Ok(0x65 ^ 0x55));

        // Encode again → back to original (XOR is its own inverse).
        codec_export_encode(&codec, &mut app.memory, 0, 4).unwrap();
        assert_eq!(app.memory.load_u8(0), Ok(0x48));
        assert_eq!(app.memory.load_u8(1), Ok(0x65));
    }

    #[test]
    fn test_modules_are_isolated() {
        let mut app1 = app_new();
        let mut app2 = app_new();
        let codec = codec_new();

        app1.memory.store_i32(0, 42).unwrap();
        app2.memory.store_i32(0, 99).unwrap();

        // Encode only app1's memory — app2 is completely unaffected.
        codec_export_encode(&codec, &mut app1.memory, 0, 4).unwrap();

        assert_eq!(app2.memory.load_i32(0), Ok(99)); // untouched
    }
}
