//! End-to-end integration tests for herkos.
//!
//! These tests verify the complete pipeline: Wasm → IR → Rust source.

use anyhow::{Context, Result};
use herkos::{transpile, TranspileOptions};

/// Helper to transpile WAT source to Rust code.
fn transpile_wat(wat_source: &str) -> Result<String> {
    let wasm_bytes = wat::parse_str(wat_source).context("failed to parse WAT")?;
    let options = TranspileOptions::default();
    transpile(&wasm_bytes, &options)
}

#[test]
fn test_simple_add() -> Result<()> {
    let wat = r#"
        (module
            (func (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // Verify the generated code contains expected elements
    assert!(rust_code.contains("pub fn func_0"));
    assert!(rust_code.contains("v0: i32"));
    assert!(rust_code.contains("v1: i32"));
    assert!(rust_code.contains("-> WasmResult<i32>"));
    assert!(rust_code.contains("wrapping_add"));
    assert!(rust_code.contains("return Ok("));

    Ok(())
}

#[test]
fn test_simple_sub() -> Result<()> {
    let wat = r#"
        (module
            (func (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.sub
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("pub fn func_0"));
    assert!(rust_code.contains("wrapping_sub"));
    assert!(rust_code.contains("return Ok("));

    Ok(())
}

#[test]
fn test_simple_mul() -> Result<()> {
    let wat = r#"
        (module
            (func (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.mul
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("pub fn func_0"));
    assert!(rust_code.contains("wrapping_mul"));
    assert!(rust_code.contains("return Ok("));

    Ok(())
}

#[test]
fn test_constant_arithmetic() -> Result<()> {
    let wat = r#"
        (module
            (func (result i32)
                i32.const 10
                i32.const 20
                i32.add
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("pub fn func_0"));
    assert!(rust_code.contains("10i32"));
    assert!(rust_code.contains("20i32"));
    assert!(rust_code.contains("wrapping_add"));
    assert!(rust_code.contains("return Ok("));

    Ok(())
}

#[test]
fn test_chained_operations() -> Result<()> {
    // (a + b) * c
    let wat = r#"
        (module
            (func (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
                local.get 2
                i32.mul
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("pub fn func_0"));
    assert!(rust_code.contains("v0: i32"));
    assert!(rust_code.contains("v1: i32"));
    assert!(rust_code.contains("v2: i32"));
    assert!(rust_code.contains("wrapping_add"));
    assert!(rust_code.contains("wrapping_mul"));

    Ok(())
}

#[test]
fn test_void_return() -> Result<()> {
    let wat = r#"
        (module
            (func
                nop
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("pub fn func_0"));
    assert!(rust_code.contains("-> WasmResult<()>"));
    assert!(rust_code.contains("return Ok(())"));

    Ok(())
}

// ==================== Milestone 2: Memory Operations ====================

#[test]
fn test_memory_store_load_i32() -> Result<()> {
    let wat = r#"
        (module
            (memory 1)
            (func (param i32 i32)
                local.get 0
                local.get 1
                i32.store
            )
            (func (param i32) (result i32)
                local.get 0
                i32.load
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("const MAX_PAGES"));
    assert!(rust_code.contains("memory: &mut IsolatedMemory<MAX_PAGES>"));
    assert!(rust_code.contains("memory.store_i32"));
    assert!(rust_code.contains("memory.load_i32"));

    Ok(())
}

#[test]
fn test_memory_with_offset() -> Result<()> {
    let wat = r#"
        (module
            (memory 1)
            (func (param i32 i32)
                local.get 0
                local.get 1
                i32.store offset=4
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("wrapping_add(4"));

    Ok(())
}

#[test]
fn test_all_memory_types() -> Result<()> {
    let wat = r#"
        (module
            (memory 1)
            (func (param i32 i32) local.get 0 local.get 1 i32.store)
            (func (param i32 i64) local.get 0 local.get 1 i64.store)
            (func (param i32 f32) local.get 0 local.get 1 f32.store)
            (func (param i32 f64) local.get 0 local.get 1 f64.store)
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("memory.store_i32"));
    assert!(rust_code.contains("memory.store_i64"));
    assert!(rust_code.contains("memory.store_f32"));
    assert!(rust_code.contains("memory.store_f64"));

    Ok(())
}

#[test]
fn test_memory_max_pages() -> Result<()> {
    let wat = r#"
        (module
            (memory 2 10)
            (func (param i32) (result i32)
                local.get 0
                i32.load
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // Should use the declared maximum
    assert!(rust_code.contains("const MAX_PAGES: usize = 10"));

    Ok(())
}

#[test]
fn test_module_without_memory() -> Result<()> {
    let wat = r#"
        (module
            (func (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // Should NOT have memory parameter
    assert!(!rust_code.contains("memory:"));
    assert!(!rust_code.contains("MAX_PAGES"));
    assert!(!rust_code.contains("IsolatedMemory"));

    Ok(())
}

// ==================== Milestone 3: Control Flow ====================

#[test]
fn test_simple_if() -> Result<()> {
    let wat = r#"
        (module
            (func (param i32) (result i32)
                local.get 0
                if (result i32)
                    i32.const 42
                else
                    i32.const 99
                end
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("42i32"));
    assert!(rust_code.contains("99i32"));
    assert!(rust_code.contains("__current_block") || rust_code.contains("continue"));

    Ok(())
}

#[test]
fn test_simple_loop() -> Result<()> {
    // Loop that counts down from 10
    let wat = r#"
        (module
            (func (param i32) (result i32)
                local.get 0
                loop (result i32)
                    local.get 0
                    i32.const 1
                    i32.sub
                    local.tee 0
                    i32.const 0
                    i32.gt_s
                    br_if 0
                    local.get 0
                end
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // Should have loop-related structures
    assert!(rust_code.contains("__current_block"));
    assert!(rust_code.contains("__current_block") || rust_code.contains("continue"));

    Ok(())
}

#[test]
fn test_br_if() -> Result<()> {
    let wat = r#"
        (module
            (func (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.gt_s
                if (result i32)
                    local.get 0
                else
                    local.get 1
                end
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("__current_block") || rust_code.contains("continue"));

    Ok(())
}

#[test]
fn test_nested_blocks() -> Result<()> {
    let wat = r#"
        (module
            (func (param i32) (result i32)
                block (result i32)
                    block (result i32)
                        local.get 0
                        i32.const 0
                        i32.eq
                        br_if 0
                        i32.const 1
                        br 1
                    end
                    drop
                    i32.const 2
                end
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // Should have multiple labeled blocks
    assert!(rust_code.contains("__current_block"));
    assert!(rust_code.contains("__current_block") || rust_code.contains("continue"));

    Ok(())
}

#[test]
fn test_br_table() -> Result<()> {
    let wat = r#"
        (module
            (func (param i32) (result i32)
                block (result i32)
                    block (result i32)
                        block (result i32)
                            local.get 0
                            br_table 0 1 2 2
                        end
                        i32.const 10
                        br 1
                    end
                    i32.const 20
                    br 0
                end
                i32.const 30
                i32.add
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("match"));
    assert!(rust_code.contains("__current_block") || rust_code.contains("continue"));

    Ok(())
}

#[test]
fn test_if_without_else() -> Result<()> {
    let wat = r#"
        (module
            (func (param i32 i32) (result i32)
                local.get 0
                i32.const 0
                i32.gt_s
                if
                    local.get 1
                    drop
                end
                local.get 0
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("__current_block") || rust_code.contains("continue"));

    Ok(())
}

#[test]
fn test_i64_add() -> Result<()> {
    let wat = r#"
        (module
            (func (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.add
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // Parameters should be i64
    assert!(rust_code.contains("v0: i64"));
    assert!(rust_code.contains("v1: i64"));
    // Return type should be i64
    assert!(rust_code.contains("-> WasmResult<i64>"));
    // Result variable should be declared as i64, not i32
    assert!(rust_code.contains(": i64 = 0i64;"));
    assert!(!rust_code.contains("v2: i32"));

    Ok(())
}

#[test]
fn test_i64_const() -> Result<()> {
    let wat = r#"
        (module
            (func (result i64)
                i64.const 9999999999
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("-> WasmResult<i64>"));
    // Constant variable should be i64
    assert!(rust_code.contains(": i64 = 0i64;"));

    Ok(())
}

// ==================== Milestone 4: Module Wrapper ====================

#[test]
fn test_module_with_mutable_global() -> Result<()> {
    let wat = r#"
        (module
            (global (mut i32) (i32.const 0))
            (func (result i32)
                global.get 0
                i32.const 1
                i32.add
                global.set 0
                global.get 0)
            (func (result i32)
                global.get 0)
            (export "increment" (func 0))
            (export "get_count" (func 1)))
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // Should generate module wrapper
    assert!(rust_code.contains("pub struct Globals"));
    assert!(rust_code.contains("pub g0: i32"));
    assert!(rust_code.contains("pub struct WasmModule(pub LibraryModule<Globals, 0>)"));
    assert!(rust_code.contains("pub fn new() -> WasmResult<WasmModule>"));
    assert!(rust_code.contains("Globals { g0: 0i32 }"));
    // Internal functions should be private
    assert!(rust_code.contains("fn func_0("));
    assert!(!rust_code.contains("pub fn func_0("));
    // Export methods
    assert!(rust_code.contains("impl WasmModule"));
    assert!(rust_code.contains("pub fn increment(&mut self) -> WasmResult<i32>"));
    assert!(rust_code.contains("pub fn get_count(&mut self) -> WasmResult<i32>"));
    // Global access
    assert!(rust_code.contains("globals.g0"));

    Ok(())
}

#[test]
fn test_module_with_data_segment() -> Result<()> {
    let wat = r#"
        (module
            (memory 1)
            (data (i32.const 0) "Hello")
            (func (param i32) (result i32)
                local.get 0
                i32.load)
            (export "load_word" (func 0)))
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // Should generate module wrapper (data segment triggers it)
    assert!(rust_code.contains("pub struct WasmModule(pub Module<(), MAX_PAGES, 0>)"));
    assert!(rust_code.contains("pub fn new() -> WasmResult<WasmModule>"));
    assert!(rust_code.contains(
        "Module::try_new(1, (), Table::try_new(0)?).map_err(|_| WasmTrap::OutOfBounds)?"
    ));
    // Data segment initialization
    assert!(rust_code.contains("module.memory.store_u8(0, 72)?")); // 'H'
    assert!(rust_code.contains("module.memory.store_u8(1, 101)?")); // 'e'
    assert!(rust_code.contains("module.memory.store_u8(4, 111)?")); // 'o'
                                                                    // Export
    assert!(rust_code.contains("pub fn load_word(&mut self, v0: i32) -> WasmResult<i32>"));
    assert!(rust_code.contains("&mut self.0.memory"));

    Ok(())
}

#[test]
fn test_module_with_immutable_global() -> Result<()> {
    let wat = r#"
        (module
            (global i32 (i32.const 42))
            (func (result i32)
                global.get 0)
            (export "get_answer" (func 0)))
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // Immutable global only → standalone (no wrapper)
    assert!(rust_code.contains("pub const G0: i32 = 42i32;"));
    assert!(rust_code.contains("pub fn func_0"));
    // GlobalGet for immutable should reference const
    assert!(rust_code.contains("G0"));
    // No wrapper
    assert!(!rust_code.contains("pub struct Globals"));
    assert!(!rust_code.contains("WasmModule"));

    Ok(())
}

#[test]
fn test_backwards_compat_no_wrapper() -> Result<()> {
    // Existing modules with exports but no globals/data should still work
    let wat = r#"
        (module
            (func (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add)
            (export "add" (func 0)))
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // No wrapper needed
    assert!(!rust_code.contains("pub struct Globals"));
    assert!(!rust_code.contains("WasmModule"));
    assert!(!rust_code.contains("impl"));
    // Standalone function
    assert!(rust_code.contains("pub fn func_0("));

    Ok(())
}

// ==================== Milestone 5: Complete Numeric Ops ====================

#[test]
fn test_i64_division() -> Result<()> {
    let wat = r#"
        (module
            (func (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.div_s
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("checked_div"));
    assert!(rust_code.contains("WasmTrap::DivisionByZero"));

    Ok(())
}

#[test]
fn test_i64_bitwise_and_shifts() -> Result<()> {
    let wat = r#"
        (module
            (func (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.and
            )
            (func (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.shl
            )
            (func (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.rotl
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("& v1"));
    assert!(rust_code.contains("wrapping_shl"));
    assert!(rust_code.contains("rotate_left"));

    Ok(())
}

#[test]
fn test_i64_comparisons() -> Result<()> {
    let wat = r#"
        (module
            (func (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.eq
            )
            (func (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.lt_u
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("1i32"));
    assert!(rust_code.contains("0i32"));
    assert!(rust_code.contains("as u64"));

    Ok(())
}

#[test]
fn test_f64_operations() -> Result<()> {
    let wat = r#"
        (module
            (func (param f64 f64) (result f64)
                local.get 0
                local.get 1
                f64.div
            )
            (func (param f64) (result f64)
                local.get 0
                f64.floor
            )
            (func (param f64) (result f64)
                local.get 0
                f64.ceil
            )
            (func (param f64) (result f64)
                local.get 0
                f64.sqrt
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    assert!(rust_code.contains("v0 / v1"));
    assert!(rust_code.contains(".floor()"));
    assert!(rust_code.contains(".ceil()"));
    assert!(rust_code.contains(".sqrt()"));

    Ok(())
}

#[test]
fn test_conversion_ops() -> Result<()> {
    let wat = r#"
        (module
            (func (param i64) (result i32)
                local.get 0
                i32.wrap_i64
            )
            (func (param i32) (result i64)
                local.get 0
                i64.extend_i32_s
            )
            (func (param i32) (result i64)
                local.get 0
                i64.extend_i32_u
            )
            (func (param f64) (result i32)
                local.get 0
                i32.trunc_f64_s
            )
            (func (param i32) (result f64)
                local.get 0
                f64.convert_i32_s
            )
            (func (param f32) (result i32)
                local.get 0
                i32.reinterpret_f32
            )
            (func (param i32) (result f32)
                local.get 0
                f32.reinterpret_i32
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    // wrap
    assert!(rust_code.contains("v0 as i32"));
    // extend signed
    assert!(rust_code.contains("v0 as i64"));
    // extend unsigned
    assert!(rust_code.contains("(v0 as u32) as i64"));
    // trunc trapping
    assert!(rust_code.contains("is_nan()"));
    assert!(rust_code.contains("WasmTrap::IntegerOverflow"));
    // convert
    assert!(rust_code.contains("v0 as f64"));
    // reinterpret
    assert!(rust_code.contains("to_bits()"));
    assert!(rust_code.contains("from_bits"));

    Ok(())
}

#[test]
fn test_subwidth_memory_ops() -> Result<()> {
    let wat = r#"
        (module
            (memory 1)
            (func (param i32 i32)
                local.get 0
                local.get 1
                i32.store8
            )
            (func (param i32) (result i32)
                local.get 0
                i32.load8_u
            )
            (func (param i32) (result i32)
                local.get 0
                i32.load8_s
            )
            (func (param i32) (result i32)
                local.get 0
                i32.load16_u
            )
            (func (param i32) (result i64)
                local.get 0
                i64.load32_u
            )
            (func (param i32) (result i64)
                local.get 0
                i64.load32_s
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    // i32.store8 → store_u8 with truncation
    assert!(rust_code.contains("store_u8"));
    assert!(rust_code.contains("as u8"));
    // i32.load8_u → load_u8 zero-extended
    assert!(rust_code.contains("load_u8"));
    assert!(rust_code.contains("as i32"));
    // i32.load8_s → load_u8 then sign-extend via i8
    assert!(rust_code.contains("as i8 as i32"));
    // i32.load16_u → load_u16 zero-extended
    assert!(rust_code.contains("load_u16"));
    // i64.load32_u → load_i32 then zero-extend via u32
    assert!(rust_code.contains("as u32 as i64"));
    // i64.load32_s → load_i32 then sign-extend
    assert!(rust_code.contains("load_i32"));

    Ok(())
}

#[test]
fn test_memory_size_and_grow() -> Result<()> {
    let wat = r#"
        (module
            (memory 1 4)
            (func (result i32)
                memory.size
            )
            (func (param i32) (result i32)
                local.get 0
                memory.grow
            )
        )
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    // memory.size → memory.size()
    assert!(rust_code.contains("memory.size()"));
    // memory.grow → memory.grow(delta as u32)
    assert!(rust_code.contains("memory.grow("));

    Ok(())
}

// ==================== Direct Function Calls ====================

#[test]
fn test_direct_call_simple() -> Result<()> {
    let wat = r#"
        (module
            (func $helper (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add)
            (func $main (param i32 i32) (result i32)
                local.get 0
                local.get 1
                call 0)
            (export "main" (func $main)))
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    // Should contain a call to func_0
    assert!(rust_code.contains("func_0("));
    // The helper should have wrapping_add
    assert!(rust_code.contains("wrapping_add"));

    Ok(())
}

#[test]
fn test_recursive_fibonacci() -> Result<()> {
    let wat = r#"
        (module
            (func $fib (param i32) (result i32)
                (if (result i32) (i32.lt_s (local.get 0) (i32.const 2))
                    (then (local.get 0))
                    (else
                        (i32.add
                            (call 0 (i32.sub (local.get 0) (i32.const 1)))
                            (call 0 (i32.sub (local.get 0) (i32.const 2)))))))
            (export "fib" (func $fib)))
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    // Should contain recursive calls to func_0
    assert!(rust_code.contains("func_0("));
    assert!(rust_code.contains("wrapping_add"));
    assert!(rust_code.contains("wrapping_sub"));

    Ok(())
}

#[test]
fn test_void_function_call() -> Result<()> {
    let wat = r#"
        (module
            (memory 1)
            (func $set_value (param i32 i32)
                local.get 0
                local.get 1
                i32.store)
            (func $main (param i32 i32)
                local.get 0
                local.get 1
                call 0)
            (export "main" (func $main)))
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    // Should contain a call without dest assignment
    assert!(rust_code.contains("func_0("));
    assert!(rust_code.contains("memory.store_i32"));

    Ok(())
}

#[test]
fn test_call_i64_return_type() -> Result<()> {
    let wat = r#"
        (module
            (func $add64 (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.add)
            (func $main (param i64 i64) (result i64)
                local.get 0
                local.get 1
                call 0)
            (export "main" (func $main)))
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    // Should contain a call to func_0
    assert!(rust_code.contains("func_0("));
    // The result variable in func_1 should be i64, not i32
    // func_1 is the main function which calls func_0 returning i64
    assert!(rust_code.contains("v0: i64"));

    Ok(())
}

#[test]
fn test_module_with_globals_and_memory() -> Result<()> {
    let wat = r#"
        (module
            (memory 1)
            (global (mut i32) (i32.const 100))
            (data (i32.const 0) "\01\02\03\04")
            (func (result i32)
                global.get 0)
            (export "get_offset" (func 0)))
    "#;

    let rust_code = transpile_wat(wat)?;

    println!("Generated Rust code:\n{}", rust_code);

    // Module wrapper with both globals and memory
    assert!(rust_code.contains("pub struct Globals"));
    assert!(rust_code.contains("pub g0: i32"));
    assert!(rust_code.contains("pub struct WasmModule(pub Module<Globals, MAX_PAGES, 0>)"));
    // Constructor initializes both
    assert!(rust_code.contains("Globals { g0: 100i32 }"));
    assert!(rust_code.contains("module.memory.store_u8("));
    // Function gets both globals and memory params
    assert!(rust_code.contains("globals: &mut Globals"));
    assert!(rust_code.contains("memory: &mut IsolatedMemory<MAX_PAGES>"));
    // Export forwards both
    assert!(rust_code.contains("&mut self.0.globals"));
    assert!(rust_code.contains("&mut self.0.memory"));

    Ok(())
}

// ==================== CLI Options ====================

#[test]
fn test_max_pages_override() -> Result<()> {
    let wat = r#"
        (module
            (memory 1)
            (func (param i32) (result i32)
                local.get 0
                i32.load
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).context("failed to parse WAT")?;

    // Default max_pages (256) when module has no maximum declared
    let default_opts = TranspileOptions::default();
    let code_default = transpile(&wasm_bytes, &default_opts)?;
    assert!(code_default.contains("const MAX_PAGES: usize = 256;"));

    // Custom max_pages override
    let custom_opts = TranspileOptions {
        max_pages: 16,
        ..TranspileOptions::default()
    };
    let code_custom = transpile(&wasm_bytes, &custom_opts)?;
    assert!(code_custom.contains("const MAX_PAGES: usize = 16;"));

    Ok(())
}

#[test]
fn test_max_pages_respects_wasm_declared_max() -> Result<()> {
    let wat = r#"
        (module
            (memory 1 4)
            (func (param i32) (result i32)
                local.get 0
                i32.load
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).context("failed to parse WAT")?;

    // Module declares max=4, so --max-pages should be ignored
    let opts = TranspileOptions {
        max_pages: 256,
        ..TranspileOptions::default()
    };
    let code = transpile(&wasm_bytes, &opts)?;
    assert!(
        code.contains("const MAX_PAGES: usize = 4;"),
        "Should use Wasm-declared max (4), not override (256)"
    );

    Ok(())
}

#[test]
fn test_mode_safe_produces_bounds_checks() -> Result<()> {
    let wat = r#"
        (module
            (memory 1)
            (func (param i32) (result i32)
                local.get 0
                i32.load
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).context("failed to parse WAT")?;
    let opts = TranspileOptions {
        mode: "safe".to_string(),
        ..TranspileOptions::default()
    };
    let code = transpile(&wasm_bytes, &opts)?;

    // Safe mode uses bounds-checked memory access (via load_i32 which returns Result)
    assert!(code.contains("memory.load_i32("));
    // No unsafe blocks in safe mode
    assert!(!code.contains("unsafe"));

    Ok(())
}

// ==================== Indirect Function Calls ====================

#[test]
fn test_call_indirect_basic() -> Result<()> {
    let wat = r#"
        (module
            (type $sig (func (param i32 i32) (result i32)))
            (table 2 funcref)
            (elem (i32.const 0) $add $sub)
            (func $add (type $sig)
                local.get 0
                local.get 1
                i32.add)
            (func $sub (type $sig)
                local.get 0
                local.get 1
                i32.sub)
            (func $dispatch (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                call_indirect (type $sig))
            (export "dispatch" (func $dispatch)))
    "#;

    let rust_code = transpile_wat(wat)?;
    println!("Generated Rust code:\n{}", rust_code);

    // Should contain table-related code
    assert!(rust_code.contains("Table"), "should import Table type");
    assert!(
        rust_code.contains("TABLE_MAX"),
        "should have TABLE_MAX const"
    );
    assert!(
        rust_code.contains("FuncRef"),
        "should import FuncRef for element init"
    );
    // Should contain indirect dispatch code
    assert!(
        rust_code.contains("table.get("),
        "should look up table entry"
    );
    assert!(rust_code.contains("type_index"), "should check type index");
    assert!(
        rust_code.contains("IndirectCallTypeMismatch"),
        "should check for type mismatch"
    );
    assert!(
        rust_code.contains("func_index"),
        "should dispatch on func_index"
    );
    // Element segment initialization
    assert!(
        rust_code.contains("table.set("),
        "should init table with element segments"
    );

    Ok(())
}
