# Test Cases

Auto-generated from `crates/herkos-tests/tests/*.rs`.

## arithmetic

```{test} test_add_correctness
:id: TEST_ARITHMETIC_ADD_CORRECTNESS
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_add_correctness` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_add_wrapping
:id: TEST_ARITHMETIC_ADD_WRAPPING
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_add_wrapping` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_sub_correctness
:id: TEST_ARITHMETIC_SUB_CORRECTNESS
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_sub_correctness` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_sub_wrapping
:id: TEST_ARITHMETIC_SUB_WRAPPING
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_sub_wrapping` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_mul_correctness
:id: TEST_ARITHMETIC_MUL_CORRECTNESS
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_mul_correctness` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_mul_wrapping
:id: TEST_ARITHMETIC_MUL_WRAPPING
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_mul_wrapping` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_const_return
:id: TEST_ARITHMETIC_CONST_RETURN
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_const_return` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_nop
:id: TEST_ARITHMETIC_NOP
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_nop` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_add_commutative
:id: TEST_ARITHMETIC_ADD_COMMUTATIVE
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_add_commutative` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_i64_add_correctness
:id: TEST_ARITHMETIC_I64_ADD_CORRECTNESS
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_i64_add_correctness` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_i64_add_large_values
:id: TEST_ARITHMETIC_I64_ADD_LARGE_VALUES
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_i64_add_large_values` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_i64_add_wrapping
:id: TEST_ARITHMETIC_I64_ADD_WRAPPING
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_i64_add_wrapping` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_i64_const
:id: TEST_ARITHMETIC_I64_CONST
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_i64_const` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_fibonacci
:id: TEST_ARITHMETIC_FIBONACCI
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_fibonacci` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_gcd
:id: TEST_ARITHMETIC_GCD
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_gcd` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_gcd_commutative
:id: TEST_ARITHMETIC_GCD_COMMUTATIVE
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_gcd_commutative` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_factorial
:id: TEST_ARITHMETIC_FACTORIAL
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_factorial` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_factorial_wrapping
:id: TEST_ARITHMETIC_FACTORIAL_WRAPPING
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_factorial_wrapping` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_abs
:id: TEST_ARITHMETIC_ABS
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_abs` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_abs_min_wraps
:id: TEST_ARITHMETIC_ABS_MIN_WRAPS
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_abs_min_wraps` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_add_matches_rust_wrapping
:id: TEST_ARITHMETIC_ADD_MATCHES_RUST_WRAPPING
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_add_matches_rust_wrapping` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_i32_extend8_s
:id: TEST_ARITHMETIC_I32_EXTEND8_S
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_i32_extend8_s` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_i32_extend16_s
:id: TEST_ARITHMETIC_I32_EXTEND16_S
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_i32_extend16_s` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_i64_extend8_s
:id: TEST_ARITHMETIC_I64_EXTEND8_S
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_i64_extend8_s` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_i64_extend16_s
:id: TEST_ARITHMETIC_I64_EXTEND16_S
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_i64_extend16_s` in `crates/herkos-tests/tests/arithmetic.rs`.
```

```{test} test_i64_extend32_s
:id: TEST_ARITHMETIC_I64_EXTEND32_S
:source_file: crates/herkos-tests/tests/arithmetic.rs
:tags: arithmetic
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_CONST, WASM_I64_ADD, WASM_I64_CONST, WASM_NOP, WASM_CALL, WASM_EXEC_INTEGER_OPS

`test_i64_extend32_s` in `crates/herkos-tests/tests/arithmetic.rs`.
```

## bulk_memory

```{test} test_fill_writes_byte_pattern
:id: TEST_BULK_MEMORY_FILL_WRITES_BYTE_PATTERN
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_fill_writes_byte_pattern` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

```{test} test_fill_zero_len_is_noop
:id: TEST_BULK_MEMORY_FILL_ZERO_LEN_IS_NOOP
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_fill_zero_len_is_noop` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

```{test} test_fill_out_of_bounds_traps
:id: TEST_BULK_MEMORY_FILL_OUT_OF_BOUNDS_TRAPS
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_fill_out_of_bounds_traps` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

```{test} test_fill_byte_truncation
:id: TEST_BULK_MEMORY_FILL_BYTE_TRUNCATION
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_fill_byte_truncation` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

```{test} test_fill_entire_region
:id: TEST_BULK_MEMORY_FILL_ENTIRE_REGION
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_fill_entire_region` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

```{test} test_init_full_segment
:id: TEST_BULK_MEMORY_INIT_FULL_SEGMENT
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_init_full_segment` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

```{test} test_init_subrange
:id: TEST_BULK_MEMORY_INIT_SUBRANGE
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_init_subrange` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

```{test} test_init_zero_len_is_noop
:id: TEST_BULK_MEMORY_INIT_ZERO_LEN_IS_NOOP
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_init_zero_len_is_noop` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

```{test} test_init_src_out_of_bounds_traps
:id: TEST_BULK_MEMORY_INIT_SRC_OUT_OF_BOUNDS_TRAPS
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_init_src_out_of_bounds_traps` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

```{test} test_init_dst_out_of_bounds_traps
:id: TEST_BULK_MEMORY_INIT_DST_OUT_OF_BOUNDS_TRAPS
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_init_dst_out_of_bounds_traps` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

```{test} test_data_drop_is_noop
:id: TEST_BULK_MEMORY_DATA_DROP_IS_NOOP
:source_file: crates/herkos-tests/tests/bulk_memory.rs
:tags: bulk_memory
:verifies: WASM_EXEC_MEMORY

`test_data_drop_is_noop` in `crates/herkos-tests/tests/bulk_memory.rs`.
```

## c_e2e

```{test} test_add_i32
:id: TEST_C_E2E_ADD_I32
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_add_i32` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_add_i32_wrapping
:id: TEST_C_E2E_ADD_I32_WRAPPING
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_add_i32_wrapping` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_sub_i32
:id: TEST_C_E2E_SUB_I32
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_sub_i32` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_sub_i32_wrapping
:id: TEST_C_E2E_SUB_I32_WRAPPING
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_sub_i32_wrapping` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_mul_i32
:id: TEST_C_E2E_MUL_I32
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_mul_i32` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_mul_i32_wrapping
:id: TEST_C_E2E_MUL_I32_WRAPPING
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_mul_i32_wrapping` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_negate
:id: TEST_C_E2E_NEGATE
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_negate` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_const_42
:id: TEST_C_E2E_CONST_42
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_const_42` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_div_s
:id: TEST_C_E2E_DIV_S
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_div_s` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_div_s_traps_on_zero
:id: TEST_C_E2E_DIV_S_TRAPS_ON_ZERO
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_div_s_traps_on_zero` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_rem_s
:id: TEST_C_E2E_REM_S
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_rem_s` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_rem_s_traps_on_zero
:id: TEST_C_E2E_REM_S_TRAPS_ON_ZERO
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_rem_s_traps_on_zero` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_bitwise_and
:id: TEST_C_E2E_BITWISE_AND
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_bitwise_and` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_bitwise_or
:id: TEST_C_E2E_BITWISE_OR
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_bitwise_or` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_bitwise_xor
:id: TEST_C_E2E_BITWISE_XOR
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_bitwise_xor` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_shift_left
:id: TEST_C_E2E_SHIFT_LEFT
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_shift_left` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_shift_right_unsigned
:id: TEST_C_E2E_SHIFT_RIGHT_UNSIGNED
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_shift_right_unsigned` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_add_i64
:id: TEST_C_E2E_ADD_I64
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_add_i64` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_add_i64_large
:id: TEST_C_E2E_ADD_I64_LARGE
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_add_i64_large` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_factorial
:id: TEST_C_E2E_FACTORIAL
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_factorial` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_sum_1_to_n
:id: TEST_C_E2E_SUM_1_TO_N
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_sum_1_to_n` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_gcd
:id: TEST_C_E2E_GCD
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_gcd` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_diff_of_squares
:id: TEST_C_E2E_DIFF_OF_SQUARES
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_diff_of_squares` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_matches_native_c_semantics
:id: TEST_C_E2E_MATCHES_NATIVE_C_SEMANTICS
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_matches_native_c_semantics` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_commutative_ops
:id: TEST_C_E2E_COMMUTATIVE_OPS
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_commutative_ops` in `crates/herkos-tests/tests/c_e2e.rs`.
```

```{test} test_division_edge_cases
:id: TEST_C_E2E_DIVISION_EDGE_CASES
:source_file: crates/herkos-tests/tests/c_e2e.rs
:tags: c_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_DIV_S, WASM_I32_REM_S, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_division_edge_cases` in `crates/herkos-tests/tests/c_e2e.rs`.
```

## c_e2e_i64

```{test} test_mul_i64
:id: TEST_C_E2E_I64_MUL_I64
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_mul_i64` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

```{test} test_sub_i64
:id: TEST_C_E2E_I64_SUB_I64
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_sub_i64` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

```{test} test_div_i64_s
:id: TEST_C_E2E_I64_DIV_I64_S
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_div_i64_s` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

```{test} test_div_i64_traps_on_zero
:id: TEST_C_E2E_I64_DIV_I64_TRAPS_ON_ZERO
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_div_i64_traps_on_zero` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

```{test} test_rem_i64_s
:id: TEST_C_E2E_I64_REM_I64_S
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_rem_i64_s` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

```{test} test_bitwise_i64
:id: TEST_C_E2E_I64_BITWISE_I64
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_bitwise_i64` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

```{test} test_shift_i64
:id: TEST_C_E2E_I64_SHIFT_I64
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_shift_i64` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

```{test} test_negate_i64
:id: TEST_C_E2E_I64_NEGATE_I64
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_negate_i64` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

```{test} test_fib_i64
:id: TEST_C_E2E_I64_FIB_I64
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_fib_i64` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

```{test} test_factorial_i64
:id: TEST_C_E2E_I64_FACTORIAL_I64
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_factorial_i64` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

```{test} test_division_invariant_i64
:id: TEST_C_E2E_I64_DIVISION_INVARIANT_I64
:source_file: crates/herkos-tests/tests/c_e2e_i64.rs
:tags: c_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_DIV_S, WASM_I64_REM_S, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_division_invariant_i64` in `crates/herkos-tests/tests/c_e2e_i64.rs`.
```

## c_e2e_loops

```{test} test_power
:id: TEST_C_E2E_LOOPS_POWER
:source_file: crates/herkos-tests/tests/c_e2e_loops.rs
:tags: c_e2e_loops
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_power` in `crates/herkos-tests/tests/c_e2e_loops.rs`.
```

```{test} test_collatz_steps
:id: TEST_C_E2E_LOOPS_COLLATZ_STEPS
:source_file: crates/herkos-tests/tests/c_e2e_loops.rs
:tags: c_e2e_loops
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_collatz_steps` in `crates/herkos-tests/tests/c_e2e_loops.rs`.
```

```{test} test_is_prime
:id: TEST_C_E2E_LOOPS_IS_PRIME
:source_file: crates/herkos-tests/tests/c_e2e_loops.rs
:tags: c_e2e_loops
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_is_prime` in `crates/herkos-tests/tests/c_e2e_loops.rs`.
```

```{test} test_count_primes
:id: TEST_C_E2E_LOOPS_COUNT_PRIMES
:source_file: crates/herkos-tests/tests/c_e2e_loops.rs
:tags: c_e2e_loops
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_count_primes` in `crates/herkos-tests/tests/c_e2e_loops.rs`.
```

```{test} test_sum_of_divisors
:id: TEST_C_E2E_LOOPS_SUM_OF_DIVISORS
:source_file: crates/herkos-tests/tests/c_e2e_loops.rs
:tags: c_e2e_loops
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_sum_of_divisors` in `crates/herkos-tests/tests/c_e2e_loops.rs`.
```

```{test} test_is_perfect
:id: TEST_C_E2E_LOOPS_IS_PERFECT
:source_file: crates/herkos-tests/tests/c_e2e_loops.rs
:tags: c_e2e_loops
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_is_perfect` in `crates/herkos-tests/tests/c_e2e_loops.rs`.
```

```{test} test_digital_root
:id: TEST_C_E2E_LOOPS_DIGITAL_ROOT
:source_file: crates/herkos-tests/tests/c_e2e_loops.rs
:tags: c_e2e_loops
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_digital_root` in `crates/herkos-tests/tests/c_e2e_loops.rs`.
```

## c_e2e_memory

```{test} test_store_and_load
:id: TEST_C_E2E_MEMORY_STORE_AND_LOAD
:source_file: crates/herkos-tests/tests/c_e2e_memory.rs
:tags: c_e2e_memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_store_and_load` in `crates/herkos-tests/tests/c_e2e_memory.rs`.
```

```{test} test_array_sum
:id: TEST_C_E2E_MEMORY_ARRAY_SUM
:source_file: crates/herkos-tests/tests/c_e2e_memory.rs
:tags: c_e2e_memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_array_sum` in `crates/herkos-tests/tests/c_e2e_memory.rs`.
```

```{test} test_array_max
:id: TEST_C_E2E_MEMORY_ARRAY_MAX
:source_file: crates/herkos-tests/tests/c_e2e_memory.rs
:tags: c_e2e_memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_array_max` in `crates/herkos-tests/tests/c_e2e_memory.rs`.
```

```{test} test_array_max_negative
:id: TEST_C_E2E_MEMORY_ARRAY_MAX_NEGATIVE
:source_file: crates/herkos-tests/tests/c_e2e_memory.rs
:tags: c_e2e_memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_array_max_negative` in `crates/herkos-tests/tests/c_e2e_memory.rs`.
```

```{test} test_dot_product
:id: TEST_C_E2E_MEMORY_DOT_PRODUCT
:source_file: crates/herkos-tests/tests/c_e2e_memory.rs
:tags: c_e2e_memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_dot_product` in `crates/herkos-tests/tests/c_e2e_memory.rs`.
```

```{test} test_array_reverse
:id: TEST_C_E2E_MEMORY_ARRAY_REVERSE
:source_file: crates/herkos-tests/tests/c_e2e_memory.rs
:tags: c_e2e_memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_array_reverse` in `crates/herkos-tests/tests/c_e2e_memory.rs`.
```

```{test} test_array_reverse_even
:id: TEST_C_E2E_MEMORY_ARRAY_REVERSE_EVEN
:source_file: crates/herkos-tests/tests/c_e2e_memory.rs
:tags: c_e2e_memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_array_reverse_even` in `crates/herkos-tests/tests/c_e2e_memory.rs`.
```

```{test} test_bubble_sort
:id: TEST_C_E2E_MEMORY_BUBBLE_SORT
:source_file: crates/herkos-tests/tests/c_e2e_memory.rs
:tags: c_e2e_memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_bubble_sort` in `crates/herkos-tests/tests/c_e2e_memory.rs`.
```

```{test} test_bubble_sort_already_sorted
:id: TEST_C_E2E_MEMORY_BUBBLE_SORT_ALREADY_SORTED
:source_file: crates/herkos-tests/tests/c_e2e_memory.rs
:tags: c_e2e_memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_bubble_sort_already_sorted` in `crates/herkos-tests/tests/c_e2e_memory.rs`.
```

## call_import_transitive

```{test} test_direct_call_transitive_import
:id: TEST_CALL_IMPORT_TRANSITIVE_DIRECT_CALL_TRANSITIVE_IMPORT
:source_file: crates/herkos-tests/tests/call_import_transitive.rs
:tags: call_import_transitive
:verifies: WASM_CALL, WASM_MOD_IMPORTS

`test_direct_call_transitive_import` in `crates/herkos-tests/tests/call_import_transitive.rs`.
```

```{test} test_caller_multiple_invocations
:id: TEST_CALL_IMPORT_TRANSITIVE_CALLER_MULTIPLE_INVOCATIONS
:source_file: crates/herkos-tests/tests/call_import_transitive.rs
:tags: call_import_transitive
:verifies: WASM_CALL, WASM_MOD_IMPORTS

`test_caller_multiple_invocations` in `crates/herkos-tests/tests/call_import_transitive.rs`.
```

## control_flow

```{test} test_simple_if_true
:id: TEST_CONTROL_FLOW_SIMPLE_IF_TRUE
:source_file: crates/herkos-tests/tests/control_flow.rs
:tags: control_flow
:verifies: WASM_IF, WASM_BLOCK, WASM_LOOP, WASM_BR_IF, WASM_EXEC_CONTROL

`test_simple_if_true` in `crates/herkos-tests/tests/control_flow.rs`.
```

```{test} test_simple_if_false
:id: TEST_CONTROL_FLOW_SIMPLE_IF_FALSE
:source_file: crates/herkos-tests/tests/control_flow.rs
:tags: control_flow
:verifies: WASM_IF, WASM_BLOCK, WASM_LOOP, WASM_BR_IF, WASM_EXEC_CONTROL

`test_simple_if_false` in `crates/herkos-tests/tests/control_flow.rs`.
```

```{test} test_max_first_larger
:id: TEST_CONTROL_FLOW_MAX_FIRST_LARGER
:source_file: crates/herkos-tests/tests/control_flow.rs
:tags: control_flow
:verifies: WASM_IF, WASM_BLOCK, WASM_LOOP, WASM_BR_IF, WASM_EXEC_CONTROL

`test_max_first_larger` in `crates/herkos-tests/tests/control_flow.rs`.
```

```{test} test_max_second_larger
:id: TEST_CONTROL_FLOW_MAX_SECOND_LARGER
:source_file: crates/herkos-tests/tests/control_flow.rs
:tags: control_flow
:verifies: WASM_IF, WASM_BLOCK, WASM_LOOP, WASM_BR_IF, WASM_EXEC_CONTROL

`test_max_second_larger` in `crates/herkos-tests/tests/control_flow.rs`.
```

```{test} test_max_equal
:id: TEST_CONTROL_FLOW_MAX_EQUAL
:source_file: crates/herkos-tests/tests/control_flow.rs
:tags: control_flow
:verifies: WASM_IF, WASM_BLOCK, WASM_LOOP, WASM_BR_IF, WASM_EXEC_CONTROL

`test_max_equal` in `crates/herkos-tests/tests/control_flow.rs`.
```

```{test} test_countdown_loop
:id: TEST_CONTROL_FLOW_COUNTDOWN_LOOP
:source_file: crates/herkos-tests/tests/control_flow.rs
:tags: control_flow
:verifies: WASM_IF, WASM_BLOCK, WASM_LOOP, WASM_BR_IF, WASM_EXEC_CONTROL

`test_countdown_loop` in `crates/herkos-tests/tests/control_flow.rs`.
```

```{test} test_countdown_loop_zero
:id: TEST_CONTROL_FLOW_COUNTDOWN_LOOP_ZERO
:source_file: crates/herkos-tests/tests/control_flow.rs
:tags: control_flow
:verifies: WASM_IF, WASM_BLOCK, WASM_LOOP, WASM_BR_IF, WASM_EXEC_CONTROL

`test_countdown_loop_zero` in `crates/herkos-tests/tests/control_flow.rs`.
```

## early_return

```{test} test_early_return_negative
:id: TEST_EARLY_RETURN_EARLY_RETURN_NEGATIVE
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_early_return_negative` in `crates/herkos-tests/tests/early_return.rs`.
```

```{test} test_early_return_positive
:id: TEST_EARLY_RETURN_EARLY_RETURN_POSITIVE
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_early_return_positive` in `crates/herkos-tests/tests/early_return.rs`.
```

```{test} test_first_positive_already_positive
:id: TEST_EARLY_RETURN_FIRST_POSITIVE_ALREADY_POSITIVE
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_first_positive_already_positive` in `crates/herkos-tests/tests/early_return.rs`.
```

```{test} test_first_positive_from_zero
:id: TEST_EARLY_RETURN_FIRST_POSITIVE_FROM_ZERO
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_first_positive_from_zero` in `crates/herkos-tests/tests/early_return.rs`.
```

```{test} test_first_positive_from_negative
:id: TEST_EARLY_RETURN_FIRST_POSITIVE_FROM_NEGATIVE
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_first_positive_from_negative` in `crates/herkos-tests/tests/early_return.rs`.
```

```{test} test_sum_early_exit_invalid
:id: TEST_EARLY_RETURN_SUM_EARLY_EXIT_INVALID
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_sum_early_exit_invalid` in `crates/herkos-tests/tests/early_return.rs`.
```

```{test} test_sum_normal
:id: TEST_EARLY_RETURN_SUM_NORMAL
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_sum_normal` in `crates/herkos-tests/tests/early_return.rs`.
```

```{test} test_nested_return_first_positive
:id: TEST_EARLY_RETURN_NESTED_RETURN_FIRST_POSITIVE
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_nested_return_first_positive` in `crates/herkos-tests/tests/early_return.rs`.
```

```{test} test_nested_return_second_positive
:id: TEST_EARLY_RETURN_NESTED_RETURN_SECOND_POSITIVE
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_nested_return_second_positive` in `crates/herkos-tests/tests/early_return.rs`.
```

```{test} test_nested_return_both_positive
:id: TEST_EARLY_RETURN_NESTED_RETURN_BOTH_POSITIVE
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_nested_return_both_positive` in `crates/herkos-tests/tests/early_return.rs`.
```

```{test} test_nested_return_neither_positive
:id: TEST_EARLY_RETURN_NESTED_RETURN_NEITHER_POSITIVE
:source_file: crates/herkos-tests/tests/early_return.rs
:tags: early_return
:verifies: WASM_RETURN, WASM_IF, WASM_EXEC_CONTROL

`test_nested_return_neither_positive` in `crates/herkos-tests/tests/early_return.rs`.
```

## function_calls

```{test} test_call_helper_basic
:id: TEST_FUNCTION_CALLS_CALL_HELPER_BASIC
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_helper_basic` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_call_helper_wrapping
:id: TEST_FUNCTION_CALLS_CALL_HELPER_WRAPPING
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_helper_wrapping` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_call_helper_direct_vs_indirect
:id: TEST_FUNCTION_CALLS_CALL_HELPER_DIRECT_VS_INDIRECT
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_helper_direct_vs_indirect` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_recursive_fib
:id: TEST_FUNCTION_CALLS_RECURSIVE_FIB
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_recursive_fib` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_recursive_fib_matches_iterative
:id: TEST_FUNCTION_CALLS_RECURSIVE_FIB_MATCHES_ITERATIVE
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_recursive_fib_matches_iterative` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_call_with_memory_store_via_call
:id: TEST_FUNCTION_CALLS_CALL_WITH_MEMORY_STORE_VIA_CALL
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_with_memory_store_via_call` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_call_with_memory_multiple_stores
:id: TEST_FUNCTION_CALLS_CALL_WITH_MEMORY_MULTIPLE_STORES
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_with_memory_multiple_stores` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_call_i64_basic
:id: TEST_FUNCTION_CALLS_CALL_I64_BASIC
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_i64_basic` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_call_i64_large_values
:id: TEST_FUNCTION_CALLS_CALL_I64_LARGE_VALUES
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_i64_large_values` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_call_i64_wrapping
:id: TEST_FUNCTION_CALLS_CALL_I64_WRAPPING
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_i64_wrapping` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_call_with_globals_set_and_get
:id: TEST_FUNCTION_CALLS_CALL_WITH_GLOBALS_SET_AND_GET
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_with_globals_set_and_get` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_call_with_globals_multiple
:id: TEST_FUNCTION_CALLS_CALL_WITH_GLOBALS_MULTIPLE
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_with_globals_multiple` in `crates/herkos-tests/tests/function_calls.rs`.
```

```{test} test_call_with_globals_isolation
:id: TEST_FUNCTION_CALLS_CALL_WITH_GLOBALS_ISOLATION
:source_file: crates/herkos-tests/tests/function_calls.rs
:tags: function_calls
:verifies: WASM_CALL, WASM_CALL_INDIRECT, WASM_GLOBAL_GET, WASM_GLOBAL_SET, WASM_I32_LOAD, WASM_I32_STORE, WASM_I64_ADD, WASM_EXEC_CALLS

`test_call_with_globals_isolation` in `crates/herkos-tests/tests/function_calls.rs`.
```

## import_memory

```{test} test_library_module_memory_lending
:id: TEST_IMPORT_MEMORY_LIBRARY_MODULE_MEMORY_LENDING
:source_file: crates/herkos-tests/tests/import_memory.rs
:tags: import_memory
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_I32_LOAD, WASM_I32_STORE

`test_library_module_memory_lending` in `crates/herkos-tests/tests/import_memory.rs`.
```

```{test} test_library_module_write_to_borrowed_memory
:id: TEST_IMPORT_MEMORY_LIBRARY_MODULE_WRITE_TO_BORROWED_MEMORY
:source_file: crates/herkos-tests/tests/import_memory.rs
:tags: import_memory
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_I32_LOAD, WASM_I32_STORE

`test_library_module_write_to_borrowed_memory` in `crates/herkos-tests/tests/import_memory.rs`.
```

```{test} test_library_module_roundtrip
:id: TEST_IMPORT_MEMORY_LIBRARY_MODULE_ROUNDTRIP
:source_file: crates/herkos-tests/tests/import_memory.rs
:tags: import_memory
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_I32_LOAD, WASM_I32_STORE

`test_library_module_roundtrip` in `crates/herkos-tests/tests/import_memory.rs`.
```

```{test} test_library_module_multiple_offsets
:id: TEST_IMPORT_MEMORY_LIBRARY_MODULE_MULTIPLE_OFFSETS
:source_file: crates/herkos-tests/tests/import_memory.rs
:tags: import_memory
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_I32_LOAD, WASM_I32_STORE

`test_library_module_multiple_offsets` in `crates/herkos-tests/tests/import_memory.rs`.
```

```{test} test_library_module_imports_and_memory
:id: TEST_IMPORT_MEMORY_LIBRARY_MODULE_IMPORTS_AND_MEMORY
:source_file: crates/herkos-tests/tests/import_memory.rs
:tags: import_memory
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_I32_LOAD, WASM_I32_STORE

`test_library_module_imports_and_memory` in `crates/herkos-tests/tests/import_memory.rs`.
```

```{test} test_memory_size_with_import
:id: TEST_IMPORT_MEMORY_MEMORY_SIZE_WITH_IMPORT
:source_file: crates/herkos-tests/tests/import_memory.rs
:tags: import_memory
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_I32_LOAD, WASM_I32_STORE

`test_memory_size_with_import` in `crates/herkos-tests/tests/import_memory.rs`.
```

```{test} test_memory_grow_borrowed
:id: TEST_IMPORT_MEMORY_MEMORY_GROW_BORROWED
:source_file: crates/herkos-tests/tests/import_memory.rs
:tags: import_memory
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_I32_LOAD, WASM_I32_STORE

`test_memory_grow_borrowed` in `crates/herkos-tests/tests/import_memory.rs`.
```

```{test} test_memory_isolation_different_modules
:id: TEST_IMPORT_MEMORY_MEMORY_ISOLATION_DIFFERENT_MODULES
:source_file: crates/herkos-tests/tests/import_memory.rs
:tags: import_memory
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_I32_LOAD, WASM_I32_STORE

`test_memory_isolation_different_modules` in `crates/herkos-tests/tests/import_memory.rs`.
```

## import_multi

```{test} test_mixed_local_and_import_calls
:id: TEST_IMPORT_MULTI_MIXED_LOCAL_AND_IMPORT_CALLS
:source_file: crates/herkos-tests/tests/import_multi.rs
:tags: import_multi
:verifies: WASM_MOD_IMPORTS, WASM_CALL, WASM_EXEC_CALLS

`test_mixed_local_and_import_calls` in `crates/herkos-tests/tests/import_multi.rs`.
```

```{test} test_multiple_imports_called
:id: TEST_IMPORT_MULTI_MULTIPLE_IMPORTS_CALLED
:source_file: crates/herkos-tests/tests/import_multi.rs
:tags: import_multi
:verifies: WASM_MOD_IMPORTS, WASM_CALL, WASM_EXEC_CALLS

`test_multiple_imports_called` in `crates/herkos-tests/tests/import_multi.rs`.
```

```{test} test_wasi_import
:id: TEST_IMPORT_MULTI_WASI_IMPORT
:source_file: crates/herkos-tests/tests/import_multi.rs
:tags: import_multi
:verifies: WASM_MOD_IMPORTS, WASM_CALL, WASM_EXEC_CALLS

`test_wasi_import` in `crates/herkos-tests/tests/import_multi.rs`.
```

```{test} test_call_all_imports
:id: TEST_IMPORT_MULTI_CALL_ALL_IMPORTS
:source_file: crates/herkos-tests/tests/import_multi.rs
:tags: import_multi
:verifies: WASM_MOD_IMPORTS, WASM_CALL, WASM_EXEC_CALLS

`test_call_all_imports` in `crates/herkos-tests/tests/import_multi.rs`.
```

```{test} test_call_local_functions_only
:id: TEST_IMPORT_MULTI_CALL_LOCAL_FUNCTIONS_ONLY
:source_file: crates/herkos-tests/tests/import_multi.rs
:tags: import_multi
:verifies: WASM_MOD_IMPORTS, WASM_CALL, WASM_EXEC_CALLS

`test_call_local_functions_only` in `crates/herkos-tests/tests/import_multi.rs`.
```

```{test} test_local_then_import_sequence
:id: TEST_IMPORT_MULTI_LOCAL_THEN_IMPORT_SEQUENCE
:source_file: crates/herkos-tests/tests/import_multi.rs
:tags: import_multi
:verifies: WASM_MOD_IMPORTS, WASM_CALL, WASM_EXEC_CALLS

`test_local_then_import_sequence` in `crates/herkos-tests/tests/import_multi.rs`.
```

```{test} test_counter_management
:id: TEST_IMPORT_MULTI_COUNTER_MANAGEMENT
:source_file: crates/herkos-tests/tests/import_multi.rs
:tags: import_multi
:verifies: WASM_MOD_IMPORTS, WASM_CALL, WASM_EXEC_CALLS

`test_counter_management` in `crates/herkos-tests/tests/import_multi.rs`.
```

```{test} test_mixed_import_sources
:id: TEST_IMPORT_MULTI_MIXED_IMPORT_SOURCES
:source_file: crates/herkos-tests/tests/import_multi.rs
:tags: import_multi
:verifies: WASM_MOD_IMPORTS, WASM_CALL, WASM_EXEC_CALLS

`test_mixed_import_sources` in `crates/herkos-tests/tests/import_multi.rs`.
```

```{test} test_call_sequence_complexity
:id: TEST_IMPORT_MULTI_CALL_SEQUENCE_COMPLEXITY
:source_file: crates/herkos-tests/tests/import_multi.rs
:tags: import_multi
:verifies: WASM_MOD_IMPORTS, WASM_CALL, WASM_EXEC_CALLS

`test_call_sequence_complexity` in `crates/herkos-tests/tests/import_multi.rs`.
```

```{test} test_multiple_hosts_with_different_implementations
:id: TEST_IMPORT_MULTI_MULTIPLE_HOSTS_WITH_DIFFERENT_IMPLEMENTATIONS
:source_file: crates/herkos-tests/tests/import_multi.rs
:tags: import_multi
:verifies: WASM_MOD_IMPORTS, WASM_CALL, WASM_EXEC_CALLS

`test_multiple_hosts_with_different_implementations` in `crates/herkos-tests/tests/import_multi.rs`.
```

## import_traits

```{test} test_trait_generation
:id: TEST_IMPORT_TRAITS_TRAIT_GENERATION
:source_file: crates/herkos-tests/tests/import_traits.rs
:tags: import_traits
:verifies: WASM_MOD_IMPORTS, WASM_MOD_EXPORTS, WASM_EXTERNAL_TYPE

`test_trait_generation` in `crates/herkos-tests/tests/import_traits.rs`.
```

```{test} test_wasi_import
:id: TEST_IMPORT_TRAITS_WASI_IMPORT
:source_file: crates/herkos-tests/tests/import_traits.rs
:tags: import_traits
:verifies: WASM_MOD_IMPORTS, WASM_MOD_EXPORTS, WASM_EXTERNAL_TYPE

`test_wasi_import` in `crates/herkos-tests/tests/import_traits.rs`.
```

```{test} test_multiple_trait_bounds
:id: TEST_IMPORT_TRAITS_MULTIPLE_TRAIT_BOUNDS
:source_file: crates/herkos-tests/tests/import_traits.rs
:tags: import_traits
:verifies: WASM_MOD_IMPORTS, WASM_MOD_EXPORTS, WASM_EXTERNAL_TYPE

`test_multiple_trait_bounds` in `crates/herkos-tests/tests/import_traits.rs`.
```

## indirect_call_import

```{test} test_call_indirect_with_import
:id: TEST_INDIRECT_CALL_IMPORT_CALL_INDIRECT_WITH_IMPORT
:source_file: crates/herkos-tests/tests/indirect_call_import.rs
:tags: indirect_call_import
:verifies: WASM_CALL_INDIRECT, WASM_MOD_IMPORTS, WASM_MOD_TABLES, WASM_MOD_ELEM

`test_call_indirect_with_import` in `crates/herkos-tests/tests/indirect_call_import.rs`.
```

```{test} test_call_indirect_multiple_dispatches
:id: TEST_INDIRECT_CALL_IMPORT_CALL_INDIRECT_MULTIPLE_DISPATCHES
:source_file: crates/herkos-tests/tests/indirect_call_import.rs
:tags: indirect_call_import
:verifies: WASM_CALL_INDIRECT, WASM_MOD_IMPORTS, WASM_MOD_TABLES, WASM_MOD_ELEM

`test_call_indirect_multiple_dispatches` in `crates/herkos-tests/tests/indirect_call_import.rs`.
```

## indirect_calls

```{test} test_binop_dispatch_add
:id: TEST_INDIRECT_CALLS_BINOP_DISPATCH_ADD
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_binop_dispatch_add` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_binop_dispatch_sub
:id: TEST_INDIRECT_CALLS_BINOP_DISPATCH_SUB
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_binop_dispatch_sub` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_binop_dispatch_mul
:id: TEST_INDIRECT_CALLS_BINOP_DISPATCH_MUL
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_binop_dispatch_mul` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_binop_dispatch_all_ops
:id: TEST_INDIRECT_CALLS_BINOP_DISPATCH_ALL_OPS
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_binop_dispatch_all_ops` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_binop_direct_vs_indirect
:id: TEST_INDIRECT_CALLS_BINOP_DIRECT_VS_INDIRECT
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_binop_direct_vs_indirect` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_unop_dispatch_negate
:id: TEST_INDIRECT_CALLS_UNOP_DISPATCH_NEGATE
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_unop_dispatch_negate` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_unop_direct_vs_indirect
:id: TEST_INDIRECT_CALLS_UNOP_DIRECT_VS_INDIRECT
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_unop_direct_vs_indirect` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_binop_dispatch_hits_unop_entry
:id: TEST_INDIRECT_CALLS_BINOP_DISPATCH_HITS_UNOP_ENTRY
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_binop_dispatch_hits_unop_entry` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_unop_dispatch_hits_binop_entry
:id: TEST_INDIRECT_CALLS_UNOP_DISPATCH_HITS_BINOP_ENTRY
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_unop_dispatch_hits_binop_entry` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_undefined_element
:id: TEST_INDIRECT_CALLS_UNDEFINED_ELEMENT
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_undefined_element` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_table_out_of_bounds
:id: TEST_INDIRECT_CALLS_TABLE_OUT_OF_BOUNDS
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_table_out_of_bounds` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

```{test} test_negative_index_out_of_bounds
:id: TEST_INDIRECT_CALLS_NEGATIVE_INDEX_OUT_OF_BOUNDS
:source_file: crates/herkos-tests/tests/indirect_calls.rs
:tags: indirect_calls
:verifies: WASM_CALL_INDIRECT, WASM_MOD_TABLES, WASM_MOD_ELEM, WASM_EXEC_CALLS

`test_negative_index_out_of_bounds` in `crates/herkos-tests/tests/indirect_calls.rs`.
```

## inter_module_lending

```{test} test_host_writes_library_reads
:id: TEST_INTER_MODULE_LENDING_HOST_WRITES_LIBRARY_READS
:source_file: crates/herkos-tests/tests/inter_module_lending.rs
:tags: inter_module_lending
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_GROW

`test_host_writes_library_reads` in `crates/herkos-tests/tests/inter_module_lending.rs`.
```

```{test} test_library_writes_host_reads
:id: TEST_INTER_MODULE_LENDING_LIBRARY_WRITES_HOST_READS
:source_file: crates/herkos-tests/tests/inter_module_lending.rs
:tags: inter_module_lending
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_GROW

`test_library_writes_host_reads` in `crates/herkos-tests/tests/inter_module_lending.rs`.
```

```{test} test_roundtrip_through_library
:id: TEST_INTER_MODULE_LENDING_ROUNDTRIP_THROUGH_LIBRARY
:source_file: crates/herkos-tests/tests/inter_module_lending.rs
:tags: inter_module_lending
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_GROW

`test_roundtrip_through_library` in `crates/herkos-tests/tests/inter_module_lending.rs`.
```

```{test} test_library_with_imports_and_memory
:id: TEST_INTER_MODULE_LENDING_LIBRARY_WITH_IMPORTS_AND_MEMORY
:source_file: crates/herkos-tests/tests/inter_module_lending.rs
:tags: inter_module_lending
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_GROW

`test_library_with_imports_and_memory` in `crates/herkos-tests/tests/inter_module_lending.rs`.
```

```{test} test_two_libraries_same_memory
:id: TEST_INTER_MODULE_LENDING_TWO_LIBRARIES_SAME_MEMORY
:source_file: crates/herkos-tests/tests/inter_module_lending.rs
:tags: inter_module_lending
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_GROW

`test_two_libraries_same_memory` in `crates/herkos-tests/tests/inter_module_lending.rs`.
```

```{test} test_multiple_module_types_shared_host
:id: TEST_INTER_MODULE_LENDING_MULTIPLE_MODULE_TYPES_SHARED_HOST
:source_file: crates/herkos-tests/tests/inter_module_lending.rs
:tags: inter_module_lending
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_GROW

`test_multiple_module_types_shared_host` in `crates/herkos-tests/tests/inter_module_lending.rs`.
```

```{test} test_memory_grow_visible_across_modules
:id: TEST_INTER_MODULE_LENDING_MEMORY_GROW_VISIBLE_ACROSS_MODULES
:source_file: crates/herkos-tests/tests/inter_module_lending.rs
:tags: inter_module_lending
:verifies: WASM_MOD_IMPORTS, WASM_MOD_MEMORIES, WASM_MEMORY_GROW

`test_memory_grow_visible_across_modules` in `crates/herkos-tests/tests/inter_module_lending.rs`.
```

## locals

```{test} test_func_0_basic
:id: TEST_LOCALS_FUNC_0_BASIC
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_0_basic` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_0_zeros
:id: TEST_LOCALS_FUNC_0_ZEROS
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_0_zeros` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_0_negative
:id: TEST_LOCALS_FUNC_0_NEGATIVE
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_0_negative` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_0_large_values
:id: TEST_LOCALS_FUNC_0_LARGE_VALUES
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_0_large_values` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_1_i32_local
:id: TEST_LOCALS_FUNC_1_I32_LOCAL
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_1_i32_local` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_1_zero
:id: TEST_LOCALS_FUNC_1_ZERO
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_1_zero` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_1_negative
:id: TEST_LOCALS_FUNC_1_NEGATIVE
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_1_negative` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_2_tee_basic
:id: TEST_LOCALS_FUNC_2_TEE_BASIC
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_2_tee_basic` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_2_tee_zero
:id: TEST_LOCALS_FUNC_2_TEE_ZERO
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_2_tee_zero` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_2_tee_negative
:id: TEST_LOCALS_FUNC_2_TEE_NEGATIVE
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_2_tee_negative` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_2_tee_large
:id: TEST_LOCALS_FUNC_2_TEE_LARGE
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_2_tee_large` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_3_zero_initialization
:id: TEST_LOCALS_FUNC_3_ZERO_INITIALIZATION
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_3_zero_initialization` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_3_zero_init_negative
:id: TEST_LOCALS_FUNC_3_ZERO_INIT_NEGATIVE
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_3_zero_init_negative` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_3_zero_init_zero
:id: TEST_LOCALS_FUNC_3_ZERO_INIT_ZERO
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_3_zero_init_zero` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_4_running_sum_basic
:id: TEST_LOCALS_FUNC_4_RUNNING_SUM_BASIC
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_4_running_sum_basic` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_4_running_sum_zeros
:id: TEST_LOCALS_FUNC_4_RUNNING_SUM_ZEROS
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_4_running_sum_zeros` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_4_running_sum_mixed
:id: TEST_LOCALS_FUNC_4_RUNNING_SUM_MIXED
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_4_running_sum_mixed` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_4_running_sum_negative
:id: TEST_LOCALS_FUNC_4_RUNNING_SUM_NEGATIVE
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_4_running_sum_negative` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_func_4_running_sum_large
:id: TEST_LOCALS_FUNC_4_RUNNING_SUM_LARGE
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_func_4_running_sum_large` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_local_bounds
:id: TEST_LOCALS_LOCAL_BOUNDS
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_local_bounds` in `crates/herkos-tests/tests/locals.rs`.
```

```{test} test_all_locals_functions_accessible
:id: TEST_LOCALS_ALL_LOCALS_FUNCTIONS_ACCESSIBLE
:source_file: crates/herkos-tests/tests/locals.rs
:tags: locals
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE, WASM_EXEC_VARIABLE

`test_all_locals_functions_accessible` in `crates/herkos-tests/tests/locals.rs`.
```

## locals_aliasing

```{test} test_mod10_tee_n10_regression
:id: TEST_LOCALS_ALIASING_MOD10_TEE_N10_REGRESSION
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_mod10_tee_n10_regression` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_mod10_tee_exact_zero_remainders
:id: TEST_LOCALS_ALIASING_MOD10_TEE_EXACT_ZERO_REMAINDERS
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_mod10_tee_exact_zero_remainders` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_mod10_tee_nonzero_remainders
:id: TEST_LOCALS_ALIASING_MOD10_TEE_NONZERO_REMAINDERS
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_mod10_tee_nonzero_remainders` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_preserve_across_set_basic
:id: TEST_LOCALS_ALIASING_PRESERVE_ACROSS_SET_BASIC
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_preserve_across_set_basic` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_preserve_across_set_one
:id: TEST_LOCALS_ALIASING_PRESERVE_ACROSS_SET_ONE
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_preserve_across_set_one` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_preserve_across_set_zero
:id: TEST_LOCALS_ALIASING_PRESERVE_ACROSS_SET_ZERO
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_preserve_across_set_zero` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_preserve_across_set_negative
:id: TEST_LOCALS_ALIASING_PRESERVE_ACROSS_SET_NEGATIVE
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_preserve_across_set_negative` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_preserve_across_set_varied
:id: TEST_LOCALS_ALIASING_PRESERVE_ACROSS_SET_VARIED
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_preserve_across_set_varied` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_get_snap_vs_tee_basic
:id: TEST_LOCALS_ALIASING_GET_SNAP_VS_TEE_BASIC
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_get_snap_vs_tee_basic` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_get_snap_vs_tee_zero_b
:id: TEST_LOCALS_ALIASING_GET_SNAP_VS_TEE_ZERO_B
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_get_snap_vs_tee_zero_b` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_get_snap_vs_tee_zero_a
:id: TEST_LOCALS_ALIASING_GET_SNAP_VS_TEE_ZERO_A
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_get_snap_vs_tee_zero_a` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_get_snap_vs_tee_negative_b
:id: TEST_LOCALS_ALIASING_GET_SNAP_VS_TEE_NEGATIVE_B
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_get_snap_vs_tee_negative_b` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_get_snap_vs_tee_equal
:id: TEST_LOCALS_ALIASING_GET_SNAP_VS_TEE_EQUAL
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_get_snap_vs_tee_equal` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_get_tee_then_set_basic
:id: TEST_LOCALS_ALIASING_GET_TEE_THEN_SET_BASIC
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_get_tee_then_set_basic` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_get_tee_then_set_zero_b
:id: TEST_LOCALS_ALIASING_GET_TEE_THEN_SET_ZERO_B
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_get_tee_then_set_zero_b` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_get_tee_then_set_zero_a
:id: TEST_LOCALS_ALIASING_GET_TEE_THEN_SET_ZERO_A
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_get_tee_then_set_zero_a` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_get_tee_then_set_negative
:id: TEST_LOCALS_ALIASING_GET_TEE_THEN_SET_NEGATIVE
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_get_tee_then_set_negative` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

```{test} test_get_tee_then_set_varied
:id: TEST_LOCALS_ALIASING_GET_TEE_THEN_SET_VARIED
:source_file: crates/herkos-tests/tests/locals_aliasing.rs
:tags: locals_aliasing
:verifies: WASM_LOCAL_GET, WASM_LOCAL_SET, WASM_LOCAL_TEE

`test_get_tee_then_set_varied` in `crates/herkos-tests/tests/locals_aliasing.rs`.
```

## memory

```{test} test_memory_store
:id: TEST_MEMORY_MEMORY_STORE
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_memory_store` in `crates/herkos-tests/tests/memory.rs`.
```

```{test} test_memory_load
:id: TEST_MEMORY_MEMORY_LOAD
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_memory_load` in `crates/herkos-tests/tests/memory.rs`.
```

```{test} test_memory_roundtrip
:id: TEST_MEMORY_MEMORY_ROUNDTRIP
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_memory_roundtrip` in `crates/herkos-tests/tests/memory.rs`.
```

```{test} test_memory_roundtrip_different_values
:id: TEST_MEMORY_MEMORY_ROUNDTRIP_DIFFERENT_VALUES
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_memory_roundtrip_different_values` in `crates/herkos-tests/tests/memory.rs`.
```

```{test} test_out_of_bounds
:id: TEST_MEMORY_OUT_OF_BOUNDS
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_out_of_bounds` in `crates/herkos-tests/tests/memory.rs`.
```

```{test} test_memory_store_at_boundary
:id: TEST_MEMORY_MEMORY_STORE_AT_BOUNDARY
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_memory_store_at_boundary` in `crates/herkos-tests/tests/memory.rs`.
```

```{test} test_memory_sum_basic
:id: TEST_MEMORY_MEMORY_SUM_BASIC
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_memory_sum_basic` in `crates/herkos-tests/tests/memory.rs`.
```

```{test} test_memory_sum_empty
:id: TEST_MEMORY_MEMORY_SUM_EMPTY
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_memory_sum_empty` in `crates/herkos-tests/tests/memory.rs`.
```

```{test} test_memory_sum_single
:id: TEST_MEMORY_MEMORY_SUM_SINGLE
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_memory_sum_single` in `crates/herkos-tests/tests/memory.rs`.
```

```{test} test_memory_sum_negative_values
:id: TEST_MEMORY_MEMORY_SUM_NEGATIVE_VALUES
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_memory_sum_negative_values` in `crates/herkos-tests/tests/memory.rs`.
```

```{test} test_memory_sum_as_static
:id: TEST_MEMORY_MEMORY_SUM_AS_STATIC
:source_file: crates/herkos-tests/tests/memory.rs
:tags: memory
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_memory_sum_as_static` in `crates/herkos-tests/tests/memory.rs`.
```

## memory_grow

```{test} test_initial_size
:id: TEST_MEMORY_GROW_INITIAL_SIZE
:source_file: crates/herkos-tests/tests/memory_grow.rs
:tags: memory_grow
:verifies: WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_EXEC_MEMORY

`test_initial_size` in `crates/herkos-tests/tests/memory_grow.rs`.
```

```{test} test_grow_success
:id: TEST_MEMORY_GROW_GROW_SUCCESS
:source_file: crates/herkos-tests/tests/memory_grow.rs
:tags: memory_grow
:verifies: WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_EXEC_MEMORY

`test_grow_success` in `crates/herkos-tests/tests/memory_grow.rs`.
```

```{test} test_grow_multiple
:id: TEST_MEMORY_GROW_GROW_MULTIPLE
:source_file: crates/herkos-tests/tests/memory_grow.rs
:tags: memory_grow
:verifies: WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_EXEC_MEMORY

`test_grow_multiple` in `crates/herkos-tests/tests/memory_grow.rs`.
```

```{test} test_grow_failure_returns_neg1
:id: TEST_MEMORY_GROW_GROW_FAILURE_RETURNS_NEG1
:source_file: crates/herkos-tests/tests/memory_grow.rs
:tags: memory_grow
:verifies: WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_EXEC_MEMORY

`test_grow_failure_returns_neg1` in `crates/herkos-tests/tests/memory_grow.rs`.
```

```{test} test_grow_then_use_new_memory
:id: TEST_MEMORY_GROW_GROW_THEN_USE_NEW_MEMORY
:source_file: crates/herkos-tests/tests/memory_grow.rs
:tags: memory_grow
:verifies: WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_EXEC_MEMORY

`test_grow_then_use_new_memory` in `crates/herkos-tests/tests/memory_grow.rs`.
```

```{test} test_grow_zero
:id: TEST_MEMORY_GROW_GROW_ZERO
:source_file: crates/herkos-tests/tests/memory_grow.rs
:tags: memory_grow
:verifies: WASM_MEMORY_SIZE, WASM_MEMORY_GROW, WASM_EXEC_MEMORY

`test_grow_zero` in `crates/herkos-tests/tests/memory_grow.rs`.
```

## module_wrapper

```{test} test_counter_initial_value
:id: TEST_MODULE_WRAPPER_COUNTER_INITIAL_VALUE
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_counter_initial_value` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_counter_increment
:id: TEST_MODULE_WRAPPER_COUNTER_INCREMENT
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_counter_increment` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_counter_get_count_after_increment
:id: TEST_MODULE_WRAPPER_COUNTER_GET_COUNT_AFTER_INCREMENT
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_counter_get_count_after_increment` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_counter_instances_are_isolated
:id: TEST_MODULE_WRAPPER_COUNTER_INSTANCES_ARE_ISOLATED
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_counter_instances_are_isolated` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_hello_data_init
:id: TEST_MODULE_WRAPPER_HELLO_DATA_INIT
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_hello_data_init` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_hello_data_second_byte
:id: TEST_MODULE_WRAPPER_HELLO_DATA_SECOND_BYTE
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_hello_data_second_byte` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_data_segments_active_init
:id: TEST_MODULE_WRAPPER_DATA_SEGMENTS_ACTIVE_INIT
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_data_segments_active_init` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_data_segments_second_active_init
:id: TEST_MODULE_WRAPPER_DATA_SEGMENTS_SECOND_ACTIVE_INIT
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_data_segments_second_active_init` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_data_segments_byte_access
:id: TEST_MODULE_WRAPPER_DATA_SEGMENTS_BYTE_ACCESS
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_data_segments_byte_access` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_data_segments_passive_does_not_crash
:id: TEST_MODULE_WRAPPER_DATA_SEGMENTS_PASSIVE_DOES_NOT_CRASH
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_data_segments_passive_does_not_crash` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_const_global
:id: TEST_MODULE_WRAPPER_CONST_GLOBAL
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_const_global` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

```{test} test_const_global_value
:id: TEST_MODULE_WRAPPER_CONST_GLOBAL_VALUE
:source_file: crates/herkos-tests/tests/module_wrapper.rs
:tags: module_wrapper
:verifies: WASM_MOD_GLOBALS, WASM_MOD_DATA, WASM_GLOBAL_GET, WASM_EXEC_INSTANTIATION

`test_const_global_value` in `crates/herkos-tests/tests/module_wrapper.rs`.
```

## numeric_ops

```{test} test_i64_div_s
:id: TEST_NUMERIC_OPS_I64_DIV_S
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_div_s` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_div_s_trap_zero
:id: TEST_NUMERIC_OPS_I64_DIV_S_TRAP_ZERO
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_div_s_trap_zero` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_bitand
:id: TEST_NUMERIC_OPS_I64_BITAND
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_bitand` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_shl
:id: TEST_NUMERIC_OPS_I64_SHL
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_shl` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_lt_s
:id: TEST_NUMERIC_OPS_I64_LT_S
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_lt_s` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_clz
:id: TEST_NUMERIC_OPS_I64_CLZ
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_clz` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_rotl
:id: TEST_NUMERIC_OPS_I64_ROTL
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_rotl` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_rem_u
:id: TEST_NUMERIC_OPS_I64_REM_U
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_rem_u` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_rem_u_trap_zero
:id: TEST_NUMERIC_OPS_I64_REM_U_TRAP_ZERO
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_rem_u_trap_zero` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f64_div
:id: TEST_NUMERIC_OPS_F64_DIV
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f64_div` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f64_min
:id: TEST_NUMERIC_OPS_F64_MIN
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f64_min` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f64_lt
:id: TEST_NUMERIC_OPS_F64_LT
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f64_lt` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f64_sqrt
:id: TEST_NUMERIC_OPS_F64_SQRT
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f64_sqrt` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f64_floor
:id: TEST_NUMERIC_OPS_F64_FLOOR
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f64_floor` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f64_ceil
:id: TEST_NUMERIC_OPS_F64_CEIL
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f64_ceil` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f64_neg
:id: TEST_NUMERIC_OPS_F64_NEG
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f64_neg` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i32_wrap_i64
:id: TEST_NUMERIC_OPS_I32_WRAP_I64
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i32_wrap_i64` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_extend_i32_s
:id: TEST_NUMERIC_OPS_I64_EXTEND_I32_S
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_extend_i32_s` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_extend_i32_u
:id: TEST_NUMERIC_OPS_I64_EXTEND_I32_U
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_extend_i32_u` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i32_extend8_s
:id: TEST_NUMERIC_OPS_I32_EXTEND8_S
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i32_extend8_s` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i32_extend16_s
:id: TEST_NUMERIC_OPS_I32_EXTEND16_S
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i32_extend16_s` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_extend8_s
:id: TEST_NUMERIC_OPS_I64_EXTEND8_S
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_extend8_s` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_extend16_s
:id: TEST_NUMERIC_OPS_I64_EXTEND16_S
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_extend16_s` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i64_extend32_s
:id: TEST_NUMERIC_OPS_I64_EXTEND32_S
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i64_extend32_s` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f64_convert_i32_s
:id: TEST_NUMERIC_OPS_F64_CONVERT_I32_S
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f64_convert_i32_s` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i32_trunc_f64_s
:id: TEST_NUMERIC_OPS_I32_TRUNC_F64_S
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i32_trunc_f64_s` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i32_trunc_f64_s_trap_nan
:id: TEST_NUMERIC_OPS_I32_TRUNC_F64_S_TRAP_NAN
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i32_trunc_f64_s_trap_nan` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i32_trunc_f64_s_trap_overflow
:id: TEST_NUMERIC_OPS_I32_TRUNC_F64_S_TRAP_OVERFLOW
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i32_trunc_f64_s_trap_overflow` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f32_demote_f64
:id: TEST_NUMERIC_OPS_F32_DEMOTE_F64
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f32_demote_f64` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f64_promote_f32
:id: TEST_NUMERIC_OPS_F64_PROMOTE_F32
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f64_promote_f32` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_i32_reinterpret_f32
:id: TEST_NUMERIC_OPS_I32_REINTERPRET_F32
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_i32_reinterpret_f32` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

```{test} test_f32_reinterpret_i32
:id: TEST_NUMERIC_OPS_F32_REINTERPRET_I32
:source_file: crates/herkos-tests/tests/numeric_ops.rs
:tags: numeric_ops
:verifies: WASM_I64_DIV_S, WASM_I64_AND, WASM_I64_SHL, WASM_I64_LT_S, WASM_I64_CLZ, WASM_I64_ROTL, WASM_I64_REM_U, WASM_F64_DIV, WASM_F64_MIN, WASM_F64_LT, WASM_F64_SQRT, WASM_F64_FLOOR, WASM_F64_CEIL, WASM_F64_NEG, WASM_I32_WRAP_I64, WASM_I64_EXTEND_I32_S, WASM_I64_EXTEND_I32_U, WASM_EXEC_INTEGER_OPS, WASM_EXEC_FLOAT_OPS, WASM_EXEC_CONVERSIONS

`test_f32_reinterpret_i32` in `crates/herkos-tests/tests/numeric_ops.rs`.
```

## rust_e2e

```{test} test_add_i32
:id: TEST_RUST_E2E_ADD_I32
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_add_i32` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_add_i32_wrapping
:id: TEST_RUST_E2E_ADD_I32_WRAPPING
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_add_i32_wrapping` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_sub_i32
:id: TEST_RUST_E2E_SUB_I32
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_sub_i32` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_sub_i32_wrapping
:id: TEST_RUST_E2E_SUB_I32_WRAPPING
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_sub_i32_wrapping` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_mul_i32
:id: TEST_RUST_E2E_MUL_I32
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_mul_i32` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_mul_i32_wrapping
:id: TEST_RUST_E2E_MUL_I32_WRAPPING
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_mul_i32_wrapping` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_add_i64
:id: TEST_RUST_E2E_ADD_I64
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_add_i64` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_add_i64_large
:id: TEST_RUST_E2E_ADD_I64_LARGE
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_add_i64_large` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_add_i64_wrapping
:id: TEST_RUST_E2E_ADD_I64_WRAPPING
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_add_i64_wrapping` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_bitwise_and
:id: TEST_RUST_E2E_BITWISE_AND
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_bitwise_and` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_bitwise_or
:id: TEST_RUST_E2E_BITWISE_OR
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_bitwise_or` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_bitwise_xor
:id: TEST_RUST_E2E_BITWISE_XOR
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_bitwise_xor` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_shift_left
:id: TEST_RUST_E2E_SHIFT_LEFT
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_shift_left` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_shift_right_unsigned
:id: TEST_RUST_E2E_SHIFT_RIGHT_UNSIGNED
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_shift_right_unsigned` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_negate
:id: TEST_RUST_E2E_NEGATE
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_negate` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_negate_wrapping
:id: TEST_RUST_E2E_NEGATE_WRAPPING
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_negate_wrapping` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_const_42
:id: TEST_RUST_E2E_CONST_42
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_const_42` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_diff_of_squares
:id: TEST_RUST_E2E_DIFF_OF_SQUARES
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_diff_of_squares` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_matches_native_rust
:id: TEST_RUST_E2E_MATCHES_NATIVE_RUST
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_matches_native_rust` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

```{test} test_commutative_ops
:id: TEST_RUST_E2E_COMMUTATIVE_OPS
:source_file: crates/herkos-tests/tests/rust_e2e.rs
:tags: rust_e2e
:verifies: WASM_I32_ADD, WASM_I32_SUB, WASM_I32_MUL, WASM_I32_AND, WASM_I32_OR, WASM_I32_XOR, WASM_I32_SHL, WASM_I32_SHR_U, WASM_I64_ADD, WASM_MOD_EXPORTS, WASM_MOD_FUNCTIONS

`test_commutative_ops` in `crates/herkos-tests/tests/rust_e2e.rs`.
```

## rust_e2e_control

```{test} test_power
:id: TEST_RUST_E2E_CONTROL_POWER
:source_file: crates/herkos-tests/tests/rust_e2e_control.rs
:tags: rust_e2e_control
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_power` in `crates/herkos-tests/tests/rust_e2e_control.rs`.
```

```{test} test_collatz_steps
:id: TEST_RUST_E2E_CONTROL_COLLATZ_STEPS
:source_file: crates/herkos-tests/tests/rust_e2e_control.rs
:tags: rust_e2e_control
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_collatz_steps` in `crates/herkos-tests/tests/rust_e2e_control.rs`.
```

```{test} test_digital_root
:id: TEST_RUST_E2E_CONTROL_DIGITAL_ROOT
:source_file: crates/herkos-tests/tests/rust_e2e_control.rs
:tags: rust_e2e_control
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_digital_root` in `crates/herkos-tests/tests/rust_e2e_control.rs`.
```

```{test} test_gcd
:id: TEST_RUST_E2E_CONTROL_GCD
:source_file: crates/herkos-tests/tests/rust_e2e_control.rs
:tags: rust_e2e_control
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_gcd` in `crates/herkos-tests/tests/rust_e2e_control.rs`.
```

```{test} test_lcm
:id: TEST_RUST_E2E_CONTROL_LCM
:source_file: crates/herkos-tests/tests/rust_e2e_control.rs
:tags: rust_e2e_control
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_lcm` in `crates/herkos-tests/tests/rust_e2e_control.rs`.
```

```{test} test_popcount
:id: TEST_RUST_E2E_CONTROL_POPCOUNT
:source_file: crates/herkos-tests/tests/rust_e2e_control.rs
:tags: rust_e2e_control
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_popcount` in `crates/herkos-tests/tests/rust_e2e_control.rs`.
```

```{test} test_is_power_of_two
:id: TEST_RUST_E2E_CONTROL_IS_POWER_OF_TWO
:source_file: crates/herkos-tests/tests/rust_e2e_control.rs
:tags: rust_e2e_control
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_is_power_of_two` in `crates/herkos-tests/tests/rust_e2e_control.rs`.
```

```{test} test_isqrt
:id: TEST_RUST_E2E_CONTROL_ISQRT
:source_file: crates/herkos-tests/tests/rust_e2e_control.rs
:tags: rust_e2e_control
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_isqrt` in `crates/herkos-tests/tests/rust_e2e_control.rs`.
```

```{test} test_matches_c_collatz
:id: TEST_RUST_E2E_CONTROL_MATCHES_C_COLLATZ
:source_file: crates/herkos-tests/tests/rust_e2e_control.rs
:tags: rust_e2e_control
:verifies: WASM_LOOP, WASM_BR_IF, WASM_IF, WASM_BLOCK, WASM_EXEC_CONTROL

`test_matches_c_collatz` in `crates/herkos-tests/tests/rust_e2e_control.rs`.
```

## rust_e2e_heavy_fibo

```{test} test_fibo_base_cases
:id: TEST_RUST_E2E_HEAVY_FIBO_FIBO_BASE_CASES
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_fibo_base_cases` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

```{test} test_fibo_small_values
:id: TEST_RUST_E2E_HEAVY_FIBO_FIBO_SMALL_VALUES
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_fibo_small_values` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

```{test} test_fibo_matches_reference
:id: TEST_RUST_E2E_HEAVY_FIBO_FIBO_MATCHES_REFERENCE
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_fibo_matches_reference` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

```{test} test_fibo_negative_returns_zero
:id: TEST_RUST_E2E_HEAVY_FIBO_FIBO_NEGATIVE_RETURNS_ZERO
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_fibo_negative_returns_zero` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

```{test} test_cache_seeded_with_two_entries
:id: TEST_RUST_E2E_HEAVY_FIBO_CACHE_SEEDED_WITH_TWO_ENTRIES
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_cache_seeded_with_two_entries` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

```{test} test_cache_grows_to_cover_requested_index
:id: TEST_RUST_E2E_HEAVY_FIBO_CACHE_GROWS_TO_COVER_REQUESTED_INDEX
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_cache_grows_to_cover_requested_index` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

```{test} test_cache_extends_incrementally
:id: TEST_RUST_E2E_HEAVY_FIBO_CACHE_EXTENDS_INCREMENTALLY
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_cache_extends_incrementally` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

```{test} test_cache_does_not_shrink_on_smaller_query
:id: TEST_RUST_E2E_HEAVY_FIBO_CACHE_DOES_NOT_SHRINK_ON_SMALLER_QUERY
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_cache_does_not_shrink_on_smaller_query` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

```{test} test_cache_len_is_monotone
:id: TEST_RUST_E2E_HEAVY_FIBO_CACHE_LEN_IS_MONOTONE
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_cache_len_is_monotone` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

```{test} test_fibo_wrapping_overflow
:id: TEST_RUST_E2E_HEAVY_FIBO_FIBO_WRAPPING_OVERFLOW
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_fibo_wrapping_overflow` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

```{test} test_fibo_idempotent
:id: TEST_RUST_E2E_HEAVY_FIBO_FIBO_IDEMPOTENT
:source_file: crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs
:tags: rust_e2e_heavy_fibo
:verifies: WASM_CALL, WASM_I32_ADD, WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_CALLS

`test_fibo_idempotent` in `crates/herkos-tests/tests/rust_e2e_heavy_fibo.rs`.
```

## rust_e2e_i64

```{test} test_mul_i64
:id: TEST_RUST_E2E_I64_MUL_I64
:source_file: crates/herkos-tests/tests/rust_e2e_i64.rs
:tags: rust_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_mul_i64` in `crates/herkos-tests/tests/rust_e2e_i64.rs`.
```

```{test} test_sub_i64
:id: TEST_RUST_E2E_I64_SUB_I64
:source_file: crates/herkos-tests/tests/rust_e2e_i64.rs
:tags: rust_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_sub_i64` in `crates/herkos-tests/tests/rust_e2e_i64.rs`.
```

```{test} test_bitwise_i64
:id: TEST_RUST_E2E_I64_BITWISE_I64
:source_file: crates/herkos-tests/tests/rust_e2e_i64.rs
:tags: rust_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_bitwise_i64` in `crates/herkos-tests/tests/rust_e2e_i64.rs`.
```

```{test} test_shift_i64
:id: TEST_RUST_E2E_I64_SHIFT_I64
:source_file: crates/herkos-tests/tests/rust_e2e_i64.rs
:tags: rust_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_shift_i64` in `crates/herkos-tests/tests/rust_e2e_i64.rs`.
```

```{test} test_negate_i64
:id: TEST_RUST_E2E_I64_NEGATE_I64
:source_file: crates/herkos-tests/tests/rust_e2e_i64.rs
:tags: rust_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_negate_i64` in `crates/herkos-tests/tests/rust_e2e_i64.rs`.
```

```{test} test_fib_i64
:id: TEST_RUST_E2E_I64_FIB_I64
:source_file: crates/herkos-tests/tests/rust_e2e_i64.rs
:tags: rust_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_fib_i64` in `crates/herkos-tests/tests/rust_e2e_i64.rs`.
```

```{test} test_factorial_i64
:id: TEST_RUST_E2E_I64_FACTORIAL_I64
:source_file: crates/herkos-tests/tests/rust_e2e_i64.rs
:tags: rust_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_factorial_i64` in `crates/herkos-tests/tests/rust_e2e_i64.rs`.
```

```{test} test_fib_matches_c
:id: TEST_RUST_E2E_I64_FIB_MATCHES_C
:source_file: crates/herkos-tests/tests/rust_e2e_i64.rs
:tags: rust_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_fib_matches_c` in `crates/herkos-tests/tests/rust_e2e_i64.rs`.
```

```{test} test_factorial_matches_c
:id: TEST_RUST_E2E_I64_FACTORIAL_MATCHES_C
:source_file: crates/herkos-tests/tests/rust_e2e_i64.rs
:tags: rust_e2e_i64
:verifies: WASM_I64_MUL, WASM_I64_SUB, WASM_I64_AND, WASM_I64_OR, WASM_I64_XOR, WASM_I64_SHL, WASM_I64_SHR_S

`test_factorial_matches_c` in `crates/herkos-tests/tests/rust_e2e_i64.rs`.
```

## rust_e2e_memory_bench

```{test} test_zero_elements_returns_zero
:id: TEST_RUST_E2E_MEMORY_BENCH_ZERO_ELEMENTS_RETURNS_ZERO
:source_file: crates/herkos-tests/tests/rust_e2e_memory_bench.rs
:tags: rust_e2e_memory_bench
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_zero_elements_returns_zero` in `crates/herkos-tests/tests/rust_e2e_memory_bench.rs`.
```

```{test} test_negative_n_returns_zero
:id: TEST_RUST_E2E_MEMORY_BENCH_NEGATIVE_N_RETURNS_ZERO
:source_file: crates/herkos-tests/tests/rust_e2e_memory_bench.rs
:tags: rust_e2e_memory_bench
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_negative_n_returns_zero` in `crates/herkos-tests/tests/rust_e2e_memory_bench.rs`.
```

```{test} test_single_element
:id: TEST_RUST_E2E_MEMORY_BENCH_SINGLE_ELEMENT
:source_file: crates/herkos-tests/tests/rust_e2e_memory_bench.rs
:tags: rust_e2e_memory_bench
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_single_element` in `crates/herkos-tests/tests/rust_e2e_memory_bench.rs`.
```

```{test} test_matches_reference_small_n
:id: TEST_RUST_E2E_MEMORY_BENCH_MATCHES_REFERENCE_SMALL_N
:source_file: crates/herkos-tests/tests/rust_e2e_memory_bench.rs
:tags: rust_e2e_memory_bench
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_matches_reference_small_n` in `crates/herkos-tests/tests/rust_e2e_memory_bench.rs`.
```

```{test} test_matches_reference_various_seeds
:id: TEST_RUST_E2E_MEMORY_BENCH_MATCHES_REFERENCE_VARIOUS_SEEDS
:source_file: crates/herkos-tests/tests/rust_e2e_memory_bench.rs
:tags: rust_e2e_memory_bench
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_matches_reference_various_seeds` in `crates/herkos-tests/tests/rust_e2e_memory_bench.rs`.
```

```{test} test_matches_reference_full_buffer
:id: TEST_RUST_E2E_MEMORY_BENCH_MATCHES_REFERENCE_FULL_BUFFER
:source_file: crates/herkos-tests/tests/rust_e2e_memory_bench.rs
:tags: rust_e2e_memory_bench
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_matches_reference_full_buffer` in `crates/herkos-tests/tests/rust_e2e_memory_bench.rs`.
```

```{test} test_n_capped_at_1024
:id: TEST_RUST_E2E_MEMORY_BENCH_N_CAPPED_AT_1024
:source_file: crates/herkos-tests/tests/rust_e2e_memory_bench.rs
:tags: rust_e2e_memory_bench
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_n_capped_at_1024` in `crates/herkos-tests/tests/rust_e2e_memory_bench.rs`.
```

```{test} test_buffer_is_sorted_after_call
:id: TEST_RUST_E2E_MEMORY_BENCH_BUFFER_IS_SORTED_AFTER_CALL
:source_file: crates/herkos-tests/tests/rust_e2e_memory_bench.rs
:tags: rust_e2e_memory_bench
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_buffer_is_sorted_after_call` in `crates/herkos-tests/tests/rust_e2e_memory_bench.rs`.
```

```{test} test_deterministic_across_calls
:id: TEST_RUST_E2E_MEMORY_BENCH_DETERMINISTIC_ACROSS_CALLS
:source_file: crates/herkos-tests/tests/rust_e2e_memory_bench.rs
:tags: rust_e2e_memory_bench
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_deterministic_across_calls` in `crates/herkos-tests/tests/rust_e2e_memory_bench.rs`.
```

```{test} test_sequential_calls_independent
:id: TEST_RUST_E2E_MEMORY_BENCH_SEQUENTIAL_CALLS_INDEPENDENT
:source_file: crates/herkos-tests/tests/rust_e2e_memory_bench.rs
:tags: rust_e2e_memory_bench
:verifies: WASM_I32_LOAD, WASM_I32_STORE, WASM_EXEC_MEMORY

`test_sequential_calls_independent` in `crates/herkos-tests/tests/rust_e2e_memory_bench.rs`.
```

## select

```{test} test_max_first_larger
:id: TEST_SELECT_MAX_FIRST_LARGER
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_max_first_larger` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_max_second_larger
:id: TEST_SELECT_MAX_SECOND_LARGER
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_max_second_larger` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_max_equal
:id: TEST_SELECT_MAX_EQUAL
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_max_equal` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_max_negative
:id: TEST_SELECT_MAX_NEGATIVE
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_max_negative` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_min_first_smaller
:id: TEST_SELECT_MIN_FIRST_SMALLER
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_min_first_smaller` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_min_second_smaller
:id: TEST_SELECT_MIN_SECOND_SMALLER
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_min_second_smaller` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_min_equal
:id: TEST_SELECT_MIN_EQUAL
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_min_equal` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_min_negative
:id: TEST_SELECT_MIN_NEGATIVE
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_min_negative` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_abs_positive
:id: TEST_SELECT_ABS_POSITIVE
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_abs_positive` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_abs_negative
:id: TEST_SELECT_ABS_NEGATIVE
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_abs_negative` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_abs_zero
:id: TEST_SELECT_ABS_ZERO
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_abs_zero` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_clamp_positive
:id: TEST_SELECT_CLAMP_POSITIVE
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_clamp_positive` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_clamp_negative
:id: TEST_SELECT_CLAMP_NEGATIVE
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_clamp_negative` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_clamp_zero
:id: TEST_SELECT_CLAMP_ZERO
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_clamp_zero` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_cond_inc_true
:id: TEST_SELECT_COND_INC_TRUE
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_cond_inc_true` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_cond_inc_false
:id: TEST_SELECT_COND_INC_FALSE
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_cond_inc_false` in `crates/herkos-tests/tests/select.rs`.
```

```{test} test_cond_inc_nonzero_flag
:id: TEST_SELECT_COND_INC_NONZERO_FLAG
:source_file: crates/herkos-tests/tests/select.rs
:tags: select
:verifies: WASM_SELECT, WASM_EXEC_PARAMETRIC

`test_cond_inc_nonzero_flag` in `crates/herkos-tests/tests/select.rs`.
```

## subwidth_mem

```{test} test_store_load_byte_unsigned
:id: TEST_SUBWIDTH_MEM_STORE_LOAD_BYTE_UNSIGNED
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_store_load_byte_unsigned` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_store_load_byte_signed
:id: TEST_SUBWIDTH_MEM_STORE_LOAD_BYTE_SIGNED
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_store_load_byte_signed` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_store_load_byte_positive
:id: TEST_SUBWIDTH_MEM_STORE_LOAD_BYTE_POSITIVE
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_store_load_byte_positive` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_store_byte_truncates
:id: TEST_SUBWIDTH_MEM_STORE_BYTE_TRUNCATES
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_store_byte_truncates` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_store_load_i16_unsigned
:id: TEST_SUBWIDTH_MEM_STORE_LOAD_I16_UNSIGNED
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_store_load_i16_unsigned` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_store_load_i16_signed
:id: TEST_SUBWIDTH_MEM_STORE_LOAD_I16_SIGNED
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_store_load_i16_signed` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_store_i16_truncates
:id: TEST_SUBWIDTH_MEM_STORE_I16_TRUNCATES
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_store_i16_truncates` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_i64_store_load_byte_unsigned
:id: TEST_SUBWIDTH_MEM_I64_STORE_LOAD_BYTE_UNSIGNED
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_i64_store_load_byte_unsigned` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_i64_store_load_byte_signed
:id: TEST_SUBWIDTH_MEM_I64_STORE_LOAD_BYTE_SIGNED
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_i64_store_load_byte_signed` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_i64_store32_load32_unsigned
:id: TEST_SUBWIDTH_MEM_I64_STORE32_LOAD32_UNSIGNED
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_i64_store32_load32_unsigned` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_i64_store32_load32_signed
:id: TEST_SUBWIDTH_MEM_I64_STORE32_LOAD32_SIGNED
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_i64_store32_load32_signed` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

```{test} test_i64_store32_positive
:id: TEST_SUBWIDTH_MEM_I64_STORE32_POSITIVE
:source_file: crates/herkos-tests/tests/subwidth_mem.rs
:tags: subwidth_mem
:verifies: WASM_I32_LOAD8_S, WASM_I32_LOAD8_U, WASM_I32_LOAD16_S, WASM_I32_LOAD16_U, WASM_I32_STORE8, WASM_I32_STORE16, WASM_I64_LOAD8_S, WASM_I64_LOAD8_U, WASM_I64_LOAD32_S, WASM_I64_LOAD32_U, WASM_I64_STORE32

`test_i64_store32_positive` in `crates/herkos-tests/tests/subwidth_mem.rs`.
```

## unreachable

```{test} test_always_traps
:id: TEST_UNREACHABLE_ALWAYS_TRAPS
:source_file: crates/herkos-tests/tests/unreachable.rs
:tags: unreachable
:verifies: WASM_UNREACHABLE, WASM_EXEC_CONTROL

`test_always_traps` in `crates/herkos-tests/tests/unreachable.rs`.
```

```{test} test_trap_on_zero
:id: TEST_UNREACHABLE_TRAP_ON_ZERO
:source_file: crates/herkos-tests/tests/unreachable.rs
:tags: unreachable
:verifies: WASM_UNREACHABLE, WASM_EXEC_CONTROL

`test_trap_on_zero` in `crates/herkos-tests/tests/unreachable.rs`.
```

```{test} test_no_trap_on_nonzero
:id: TEST_UNREACHABLE_NO_TRAP_ON_NONZERO
:source_file: crates/herkos-tests/tests/unreachable.rs
:tags: unreachable
:verifies: WASM_UNREACHABLE, WASM_EXEC_CONTROL

`test_no_trap_on_nonzero` in `crates/herkos-tests/tests/unreachable.rs`.
```

```{test} test_safe_div_normal
:id: TEST_UNREACHABLE_SAFE_DIV_NORMAL
:source_file: crates/herkos-tests/tests/unreachable.rs
:tags: unreachable
:verifies: WASM_UNREACHABLE, WASM_EXEC_CONTROL

`test_safe_div_normal` in `crates/herkos-tests/tests/unreachable.rs`.
```

```{test} test_safe_div_traps_on_zero
:id: TEST_UNREACHABLE_SAFE_DIV_TRAPS_ON_ZERO
:source_file: crates/herkos-tests/tests/unreachable.rs
:tags: unreachable
:verifies: WASM_UNREACHABLE, WASM_EXEC_CONTROL

`test_safe_div_traps_on_zero` in `crates/herkos-tests/tests/unreachable.rs`.
```

```{test} test_dead_unreachable
:id: TEST_UNREACHABLE_DEAD_UNREACHABLE
:source_file: crates/herkos-tests/tests/unreachable.rs
:tags: unreachable
:verifies: WASM_UNREACHABLE, WASM_EXEC_CONTROL

`test_dead_unreachable` in `crates/herkos-tests/tests/unreachable.rs`.
```
