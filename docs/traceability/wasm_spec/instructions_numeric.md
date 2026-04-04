# Numeric Instructions

Wasm 1.0 numeric instructions (§2.4.1): constants, unary, binary,
test, comparison, and conversion operations.

Source: [W3C WebAssembly Core Specification 1.0, §2.4.1](https://www.w3.org/TR/wasm-core-1/#numeric-instructions%E2%91%A0)

## Constants

```{wasm_spec} i32.const
:id: WASM_I32_CONST
:wasm_section: §2.4.1
:wasm_opcode: i32.const
:tags: numeric, i32, const

Wasm 1.0: `i32.const` instruction.
```

```{wasm_spec} i64.const
:id: WASM_I64_CONST
:wasm_section: §2.4.1
:wasm_opcode: i64.const
:tags: numeric, i64, const

Wasm 1.0: `i64.const` instruction.
```

```{wasm_spec} f32.const
:id: WASM_F32_CONST
:wasm_section: §2.4.1
:wasm_opcode: f32.const
:tags: numeric, f32, const

Wasm 1.0: `f32.const` instruction.
```

```{wasm_spec} f64.const
:id: WASM_F64_CONST
:wasm_section: §2.4.1
:wasm_opcode: f64.const
:tags: numeric, f64, const

Wasm 1.0: `f64.const` instruction.
```

## i32 Instructions

```{wasm_spec} i32.clz
:id: WASM_I32_CLZ
:wasm_section: §2.4.1
:wasm_opcode: i32.clz
:tags: numeric, i32, unary

Wasm 1.0: `i32.clz` instruction.
```

```{wasm_spec} i32.ctz
:id: WASM_I32_CTZ
:wasm_section: §2.4.1
:wasm_opcode: i32.ctz
:tags: numeric, i32, unary

Wasm 1.0: `i32.ctz` instruction.
```

```{wasm_spec} i32.popcnt
:id: WASM_I32_POPCNT
:wasm_section: §2.4.1
:wasm_opcode: i32.popcnt
:tags: numeric, i32, unary

Wasm 1.0: `i32.popcnt` instruction.
```

```{wasm_spec} i32.eqz
:id: WASM_I32_EQZ
:wasm_section: §2.4.1
:wasm_opcode: i32.eqz
:tags: numeric, i32, test

Wasm 1.0: `i32.eqz` instruction.
```

```{wasm_spec} i32.add
:id: WASM_I32_ADD
:wasm_section: §2.4.1
:wasm_opcode: i32.add
:tags: numeric, i32, binop

Wasm 1.0: `i32.add` instruction.
```

```{wasm_spec} i32.sub
:id: WASM_I32_SUB
:wasm_section: §2.4.1
:wasm_opcode: i32.sub
:tags: numeric, i32, binop

Wasm 1.0: `i32.sub` instruction.
```

```{wasm_spec} i32.mul
:id: WASM_I32_MUL
:wasm_section: §2.4.1
:wasm_opcode: i32.mul
:tags: numeric, i32, binop

Wasm 1.0: `i32.mul` instruction.
```

```{wasm_spec} i32.div_s
:id: WASM_I32_DIV_S
:wasm_section: §2.4.1
:wasm_opcode: i32.div_s
:tags: numeric, i32, binop

Wasm 1.0: `i32.div_s` instruction.
```

```{wasm_spec} i32.div_u
:id: WASM_I32_DIV_U
:wasm_section: §2.4.1
:wasm_opcode: i32.div_u
:tags: numeric, i32, binop

Wasm 1.0: `i32.div_u` instruction.
```

```{wasm_spec} i32.rem_s
:id: WASM_I32_REM_S
:wasm_section: §2.4.1
:wasm_opcode: i32.rem_s
:tags: numeric, i32, binop

Wasm 1.0: `i32.rem_s` instruction.
```

```{wasm_spec} i32.rem_u
:id: WASM_I32_REM_U
:wasm_section: §2.4.1
:wasm_opcode: i32.rem_u
:tags: numeric, i32, binop

Wasm 1.0: `i32.rem_u` instruction.
```

```{wasm_spec} i32.and
:id: WASM_I32_AND
:wasm_section: §2.4.1
:wasm_opcode: i32.and
:tags: numeric, i32, binop

Wasm 1.0: `i32.and` instruction.
```

```{wasm_spec} i32.or
:id: WASM_I32_OR
:wasm_section: §2.4.1
:wasm_opcode: i32.or
:tags: numeric, i32, binop

Wasm 1.0: `i32.or` instruction.
```

```{wasm_spec} i32.xor
:id: WASM_I32_XOR
:wasm_section: §2.4.1
:wasm_opcode: i32.xor
:tags: numeric, i32, binop

Wasm 1.0: `i32.xor` instruction.
```

```{wasm_spec} i32.shl
:id: WASM_I32_SHL
:wasm_section: §2.4.1
:wasm_opcode: i32.shl
:tags: numeric, i32, binop

Wasm 1.0: `i32.shl` instruction.
```

```{wasm_spec} i32.shr_s
:id: WASM_I32_SHR_S
:wasm_section: §2.4.1
:wasm_opcode: i32.shr_s
:tags: numeric, i32, binop

Wasm 1.0: `i32.shr_s` instruction.
```

```{wasm_spec} i32.shr_u
:id: WASM_I32_SHR_U
:wasm_section: §2.4.1
:wasm_opcode: i32.shr_u
:tags: numeric, i32, binop

Wasm 1.0: `i32.shr_u` instruction.
```

```{wasm_spec} i32.rotl
:id: WASM_I32_ROTL
:wasm_section: §2.4.1
:wasm_opcode: i32.rotl
:tags: numeric, i32, binop

Wasm 1.0: `i32.rotl` instruction.
```

```{wasm_spec} i32.rotr
:id: WASM_I32_ROTR
:wasm_section: §2.4.1
:wasm_opcode: i32.rotr
:tags: numeric, i32, binop

Wasm 1.0: `i32.rotr` instruction.
```

```{wasm_spec} i32.eq
:id: WASM_I32_EQ
:wasm_section: §2.4.1
:wasm_opcode: i32.eq
:tags: numeric, i32, comparison

Wasm 1.0: `i32.eq` instruction.
```

```{wasm_spec} i32.ne
:id: WASM_I32_NE
:wasm_section: §2.4.1
:wasm_opcode: i32.ne
:tags: numeric, i32, comparison

Wasm 1.0: `i32.ne` instruction.
```

```{wasm_spec} i32.lt_s
:id: WASM_I32_LT_S
:wasm_section: §2.4.1
:wasm_opcode: i32.lt_s
:tags: numeric, i32, comparison

Wasm 1.0: `i32.lt_s` instruction.
```

```{wasm_spec} i32.lt_u
:id: WASM_I32_LT_U
:wasm_section: §2.4.1
:wasm_opcode: i32.lt_u
:tags: numeric, i32, comparison

Wasm 1.0: `i32.lt_u` instruction.
```

```{wasm_spec} i32.gt_s
:id: WASM_I32_GT_S
:wasm_section: §2.4.1
:wasm_opcode: i32.gt_s
:tags: numeric, i32, comparison

Wasm 1.0: `i32.gt_s` instruction.
```

```{wasm_spec} i32.gt_u
:id: WASM_I32_GT_U
:wasm_section: §2.4.1
:wasm_opcode: i32.gt_u
:tags: numeric, i32, comparison

Wasm 1.0: `i32.gt_u` instruction.
```

```{wasm_spec} i32.le_s
:id: WASM_I32_LE_S
:wasm_section: §2.4.1
:wasm_opcode: i32.le_s
:tags: numeric, i32, comparison

Wasm 1.0: `i32.le_s` instruction.
```

```{wasm_spec} i32.le_u
:id: WASM_I32_LE_U
:wasm_section: §2.4.1
:wasm_opcode: i32.le_u
:tags: numeric, i32, comparison

Wasm 1.0: `i32.le_u` instruction.
```

```{wasm_spec} i32.ge_s
:id: WASM_I32_GE_S
:wasm_section: §2.4.1
:wasm_opcode: i32.ge_s
:tags: numeric, i32, comparison

Wasm 1.0: `i32.ge_s` instruction.
```

```{wasm_spec} i32.ge_u
:id: WASM_I32_GE_U
:wasm_section: §2.4.1
:wasm_opcode: i32.ge_u
:tags: numeric, i32, comparison

Wasm 1.0: `i32.ge_u` instruction.
```

## i64 Instructions

```{wasm_spec} i64.clz
:id: WASM_I64_CLZ
:wasm_section: §2.4.1
:wasm_opcode: i64.clz
:tags: numeric, i64, unary

Wasm 1.0: `i64.clz` instruction.
```

```{wasm_spec} i64.ctz
:id: WASM_I64_CTZ
:wasm_section: §2.4.1
:wasm_opcode: i64.ctz
:tags: numeric, i64, unary

Wasm 1.0: `i64.ctz` instruction.
```

```{wasm_spec} i64.popcnt
:id: WASM_I64_POPCNT
:wasm_section: §2.4.1
:wasm_opcode: i64.popcnt
:tags: numeric, i64, unary

Wasm 1.0: `i64.popcnt` instruction.
```

```{wasm_spec} i64.eqz
:id: WASM_I64_EQZ
:wasm_section: §2.4.1
:wasm_opcode: i64.eqz
:tags: numeric, i64, test

Wasm 1.0: `i64.eqz` instruction.
```

```{wasm_spec} i64.add
:id: WASM_I64_ADD
:wasm_section: §2.4.1
:wasm_opcode: i64.add
:tags: numeric, i64, binop

Wasm 1.0: `i64.add` instruction.
```

```{wasm_spec} i64.sub
:id: WASM_I64_SUB
:wasm_section: §2.4.1
:wasm_opcode: i64.sub
:tags: numeric, i64, binop

Wasm 1.0: `i64.sub` instruction.
```

```{wasm_spec} i64.mul
:id: WASM_I64_MUL
:wasm_section: §2.4.1
:wasm_opcode: i64.mul
:tags: numeric, i64, binop

Wasm 1.0: `i64.mul` instruction.
```

```{wasm_spec} i64.div_s
:id: WASM_I64_DIV_S
:wasm_section: §2.4.1
:wasm_opcode: i64.div_s
:tags: numeric, i64, binop

Wasm 1.0: `i64.div_s` instruction.
```

```{wasm_spec} i64.div_u
:id: WASM_I64_DIV_U
:wasm_section: §2.4.1
:wasm_opcode: i64.div_u
:tags: numeric, i64, binop

Wasm 1.0: `i64.div_u` instruction.
```

```{wasm_spec} i64.rem_s
:id: WASM_I64_REM_S
:wasm_section: §2.4.1
:wasm_opcode: i64.rem_s
:tags: numeric, i64, binop

Wasm 1.0: `i64.rem_s` instruction.
```

```{wasm_spec} i64.rem_u
:id: WASM_I64_REM_U
:wasm_section: §2.4.1
:wasm_opcode: i64.rem_u
:tags: numeric, i64, binop

Wasm 1.0: `i64.rem_u` instruction.
```

```{wasm_spec} i64.and
:id: WASM_I64_AND
:wasm_section: §2.4.1
:wasm_opcode: i64.and
:tags: numeric, i64, binop

Wasm 1.0: `i64.and` instruction.
```

```{wasm_spec} i64.or
:id: WASM_I64_OR
:wasm_section: §2.4.1
:wasm_opcode: i64.or
:tags: numeric, i64, binop

Wasm 1.0: `i64.or` instruction.
```

```{wasm_spec} i64.xor
:id: WASM_I64_XOR
:wasm_section: §2.4.1
:wasm_opcode: i64.xor
:tags: numeric, i64, binop

Wasm 1.0: `i64.xor` instruction.
```

```{wasm_spec} i64.shl
:id: WASM_I64_SHL
:wasm_section: §2.4.1
:wasm_opcode: i64.shl
:tags: numeric, i64, binop

Wasm 1.0: `i64.shl` instruction.
```

```{wasm_spec} i64.shr_s
:id: WASM_I64_SHR_S
:wasm_section: §2.4.1
:wasm_opcode: i64.shr_s
:tags: numeric, i64, binop

Wasm 1.0: `i64.shr_s` instruction.
```

```{wasm_spec} i64.shr_u
:id: WASM_I64_SHR_U
:wasm_section: §2.4.1
:wasm_opcode: i64.shr_u
:tags: numeric, i64, binop

Wasm 1.0: `i64.shr_u` instruction.
```

```{wasm_spec} i64.rotl
:id: WASM_I64_ROTL
:wasm_section: §2.4.1
:wasm_opcode: i64.rotl
:tags: numeric, i64, binop

Wasm 1.0: `i64.rotl` instruction.
```

```{wasm_spec} i64.rotr
:id: WASM_I64_ROTR
:wasm_section: §2.4.1
:wasm_opcode: i64.rotr
:tags: numeric, i64, binop

Wasm 1.0: `i64.rotr` instruction.
```

```{wasm_spec} i64.eq
:id: WASM_I64_EQ
:wasm_section: §2.4.1
:wasm_opcode: i64.eq
:tags: numeric, i64, comparison

Wasm 1.0: `i64.eq` instruction.
```

```{wasm_spec} i64.ne
:id: WASM_I64_NE
:wasm_section: §2.4.1
:wasm_opcode: i64.ne
:tags: numeric, i64, comparison

Wasm 1.0: `i64.ne` instruction.
```

```{wasm_spec} i64.lt_s
:id: WASM_I64_LT_S
:wasm_section: §2.4.1
:wasm_opcode: i64.lt_s
:tags: numeric, i64, comparison

Wasm 1.0: `i64.lt_s` instruction.
```

```{wasm_spec} i64.lt_u
:id: WASM_I64_LT_U
:wasm_section: §2.4.1
:wasm_opcode: i64.lt_u
:tags: numeric, i64, comparison

Wasm 1.0: `i64.lt_u` instruction.
```

```{wasm_spec} i64.gt_s
:id: WASM_I64_GT_S
:wasm_section: §2.4.1
:wasm_opcode: i64.gt_s
:tags: numeric, i64, comparison

Wasm 1.0: `i64.gt_s` instruction.
```

```{wasm_spec} i64.gt_u
:id: WASM_I64_GT_U
:wasm_section: §2.4.1
:wasm_opcode: i64.gt_u
:tags: numeric, i64, comparison

Wasm 1.0: `i64.gt_u` instruction.
```

```{wasm_spec} i64.le_s
:id: WASM_I64_LE_S
:wasm_section: §2.4.1
:wasm_opcode: i64.le_s
:tags: numeric, i64, comparison

Wasm 1.0: `i64.le_s` instruction.
```

```{wasm_spec} i64.le_u
:id: WASM_I64_LE_U
:wasm_section: §2.4.1
:wasm_opcode: i64.le_u
:tags: numeric, i64, comparison

Wasm 1.0: `i64.le_u` instruction.
```

```{wasm_spec} i64.ge_s
:id: WASM_I64_GE_S
:wasm_section: §2.4.1
:wasm_opcode: i64.ge_s
:tags: numeric, i64, comparison

Wasm 1.0: `i64.ge_s` instruction.
```

```{wasm_spec} i64.ge_u
:id: WASM_I64_GE_U
:wasm_section: §2.4.1
:wasm_opcode: i64.ge_u
:tags: numeric, i64, comparison

Wasm 1.0: `i64.ge_u` instruction.
```

## f32 Instructions

```{wasm_spec} f32.abs
:id: WASM_F32_ABS
:wasm_section: §2.4.1
:wasm_opcode: f32.abs
:tags: numeric, f32, unary

Wasm 1.0: `f32.abs` instruction.
```

```{wasm_spec} f32.neg
:id: WASM_F32_NEG
:wasm_section: §2.4.1
:wasm_opcode: f32.neg
:tags: numeric, f32, unary

Wasm 1.0: `f32.neg` instruction.
```

```{wasm_spec} f32.sqrt
:id: WASM_F32_SQRT
:wasm_section: §2.4.1
:wasm_opcode: f32.sqrt
:tags: numeric, f32, unary

Wasm 1.0: `f32.sqrt` instruction.
```

```{wasm_spec} f32.ceil
:id: WASM_F32_CEIL
:wasm_section: §2.4.1
:wasm_opcode: f32.ceil
:tags: numeric, f32, unary

Wasm 1.0: `f32.ceil` instruction.
```

```{wasm_spec} f32.floor
:id: WASM_F32_FLOOR
:wasm_section: §2.4.1
:wasm_opcode: f32.floor
:tags: numeric, f32, unary

Wasm 1.0: `f32.floor` instruction.
```

```{wasm_spec} f32.trunc
:id: WASM_F32_TRUNC
:wasm_section: §2.4.1
:wasm_opcode: f32.trunc
:tags: numeric, f32, unary

Wasm 1.0: `f32.trunc` instruction.
```

```{wasm_spec} f32.nearest
:id: WASM_F32_NEAREST
:wasm_section: §2.4.1
:wasm_opcode: f32.nearest
:tags: numeric, f32, unary

Wasm 1.0: `f32.nearest` instruction.
```

```{wasm_spec} f32.add
:id: WASM_F32_ADD
:wasm_section: §2.4.1
:wasm_opcode: f32.add
:tags: numeric, f32, binop

Wasm 1.0: `f32.add` instruction.
```

```{wasm_spec} f32.sub
:id: WASM_F32_SUB
:wasm_section: §2.4.1
:wasm_opcode: f32.sub
:tags: numeric, f32, binop

Wasm 1.0: `f32.sub` instruction.
```

```{wasm_spec} f32.mul
:id: WASM_F32_MUL
:wasm_section: §2.4.1
:wasm_opcode: f32.mul
:tags: numeric, f32, binop

Wasm 1.0: `f32.mul` instruction.
```

```{wasm_spec} f32.div
:id: WASM_F32_DIV
:wasm_section: §2.4.1
:wasm_opcode: f32.div
:tags: numeric, f32, binop

Wasm 1.0: `f32.div` instruction.
```

```{wasm_spec} f32.min
:id: WASM_F32_MIN
:wasm_section: §2.4.1
:wasm_opcode: f32.min
:tags: numeric, f32, binop

Wasm 1.0: `f32.min` instruction.
```

```{wasm_spec} f32.max
:id: WASM_F32_MAX
:wasm_section: §2.4.1
:wasm_opcode: f32.max
:tags: numeric, f32, binop

Wasm 1.0: `f32.max` instruction.
```

```{wasm_spec} f32.copysign
:id: WASM_F32_COPYSIGN
:wasm_section: §2.4.1
:wasm_opcode: f32.copysign
:tags: numeric, f32, binop

Wasm 1.0: `f32.copysign` instruction.
```

```{wasm_spec} f32.eq
:id: WASM_F32_EQ
:wasm_section: §2.4.1
:wasm_opcode: f32.eq
:tags: numeric, f32, comparison

Wasm 1.0: `f32.eq` instruction.
```

```{wasm_spec} f32.ne
:id: WASM_F32_NE
:wasm_section: §2.4.1
:wasm_opcode: f32.ne
:tags: numeric, f32, comparison

Wasm 1.0: `f32.ne` instruction.
```

```{wasm_spec} f32.lt
:id: WASM_F32_LT
:wasm_section: §2.4.1
:wasm_opcode: f32.lt
:tags: numeric, f32, comparison

Wasm 1.0: `f32.lt` instruction.
```

```{wasm_spec} f32.gt
:id: WASM_F32_GT
:wasm_section: §2.4.1
:wasm_opcode: f32.gt
:tags: numeric, f32, comparison

Wasm 1.0: `f32.gt` instruction.
```

```{wasm_spec} f32.le
:id: WASM_F32_LE
:wasm_section: §2.4.1
:wasm_opcode: f32.le
:tags: numeric, f32, comparison

Wasm 1.0: `f32.le` instruction.
```

```{wasm_spec} f32.ge
:id: WASM_F32_GE
:wasm_section: §2.4.1
:wasm_opcode: f32.ge
:tags: numeric, f32, comparison

Wasm 1.0: `f32.ge` instruction.
```

## f64 Instructions

```{wasm_spec} f64.abs
:id: WASM_F64_ABS
:wasm_section: §2.4.1
:wasm_opcode: f64.abs
:tags: numeric, f64, unary

Wasm 1.0: `f64.abs` instruction.
```

```{wasm_spec} f64.neg
:id: WASM_F64_NEG
:wasm_section: §2.4.1
:wasm_opcode: f64.neg
:tags: numeric, f64, unary

Wasm 1.0: `f64.neg` instruction.
```

```{wasm_spec} f64.sqrt
:id: WASM_F64_SQRT
:wasm_section: §2.4.1
:wasm_opcode: f64.sqrt
:tags: numeric, f64, unary

Wasm 1.0: `f64.sqrt` instruction.
```

```{wasm_spec} f64.ceil
:id: WASM_F64_CEIL
:wasm_section: §2.4.1
:wasm_opcode: f64.ceil
:tags: numeric, f64, unary

Wasm 1.0: `f64.ceil` instruction.
```

```{wasm_spec} f64.floor
:id: WASM_F64_FLOOR
:wasm_section: §2.4.1
:wasm_opcode: f64.floor
:tags: numeric, f64, unary

Wasm 1.0: `f64.floor` instruction.
```

```{wasm_spec} f64.trunc
:id: WASM_F64_TRUNC
:wasm_section: §2.4.1
:wasm_opcode: f64.trunc
:tags: numeric, f64, unary

Wasm 1.0: `f64.trunc` instruction.
```

```{wasm_spec} f64.nearest
:id: WASM_F64_NEAREST
:wasm_section: §2.4.1
:wasm_opcode: f64.nearest
:tags: numeric, f64, unary

Wasm 1.0: `f64.nearest` instruction.
```

```{wasm_spec} f64.add
:id: WASM_F64_ADD
:wasm_section: §2.4.1
:wasm_opcode: f64.add
:tags: numeric, f64, binop

Wasm 1.0: `f64.add` instruction.
```

```{wasm_spec} f64.sub
:id: WASM_F64_SUB
:wasm_section: §2.4.1
:wasm_opcode: f64.sub
:tags: numeric, f64, binop

Wasm 1.0: `f64.sub` instruction.
```

```{wasm_spec} f64.mul
:id: WASM_F64_MUL
:wasm_section: §2.4.1
:wasm_opcode: f64.mul
:tags: numeric, f64, binop

Wasm 1.0: `f64.mul` instruction.
```

```{wasm_spec} f64.div
:id: WASM_F64_DIV
:wasm_section: §2.4.1
:wasm_opcode: f64.div
:tags: numeric, f64, binop

Wasm 1.0: `f64.div` instruction.
```

```{wasm_spec} f64.min
:id: WASM_F64_MIN
:wasm_section: §2.4.1
:wasm_opcode: f64.min
:tags: numeric, f64, binop

Wasm 1.0: `f64.min` instruction.
```

```{wasm_spec} f64.max
:id: WASM_F64_MAX
:wasm_section: §2.4.1
:wasm_opcode: f64.max
:tags: numeric, f64, binop

Wasm 1.0: `f64.max` instruction.
```

```{wasm_spec} f64.copysign
:id: WASM_F64_COPYSIGN
:wasm_section: §2.4.1
:wasm_opcode: f64.copysign
:tags: numeric, f64, binop

Wasm 1.0: `f64.copysign` instruction.
```

```{wasm_spec} f64.eq
:id: WASM_F64_EQ
:wasm_section: §2.4.1
:wasm_opcode: f64.eq
:tags: numeric, f64, comparison

Wasm 1.0: `f64.eq` instruction.
```

```{wasm_spec} f64.ne
:id: WASM_F64_NE
:wasm_section: §2.4.1
:wasm_opcode: f64.ne
:tags: numeric, f64, comparison

Wasm 1.0: `f64.ne` instruction.
```

```{wasm_spec} f64.lt
:id: WASM_F64_LT
:wasm_section: §2.4.1
:wasm_opcode: f64.lt
:tags: numeric, f64, comparison

Wasm 1.0: `f64.lt` instruction.
```

```{wasm_spec} f64.gt
:id: WASM_F64_GT
:wasm_section: §2.4.1
:wasm_opcode: f64.gt
:tags: numeric, f64, comparison

Wasm 1.0: `f64.gt` instruction.
```

```{wasm_spec} f64.le
:id: WASM_F64_LE
:wasm_section: §2.4.1
:wasm_opcode: f64.le
:tags: numeric, f64, comparison

Wasm 1.0: `f64.le` instruction.
```

```{wasm_spec} f64.ge
:id: WASM_F64_GE
:wasm_section: §2.4.1
:wasm_opcode: f64.ge
:tags: numeric, f64, comparison

Wasm 1.0: `f64.ge` instruction.
```

## Conversions

```{wasm_spec} i32.wrap_i64
:id: WASM_I32_WRAP_I64
:wasm_section: §2.4.1
:wasm_opcode: i32.wrap_i64
:tags: numeric, conversion

Wasm 1.0: `i32.wrap_i64` — i64 to i32 (truncate to low 32 bits).
```

```{wasm_spec} i64.extend_i32_s
:id: WASM_I64_EXTEND_I32_S
:wasm_section: §2.4.1
:wasm_opcode: i64.extend_i32_s
:tags: numeric, conversion

Wasm 1.0: `i64.extend_i32_s` — i32 to i64 (sign-extend).
```

```{wasm_spec} i64.extend_i32_u
:id: WASM_I64_EXTEND_I32_U
:wasm_section: §2.4.1
:wasm_opcode: i64.extend_i32_u
:tags: numeric, conversion

Wasm 1.0: `i64.extend_i32_u` — i32 to i64 (zero-extend).
```

```{wasm_spec} i32.trunc_f32_s
:id: WASM_I32_TRUNC_F32_S
:wasm_section: §2.4.1
:wasm_opcode: i32.trunc_f32_s
:tags: numeric, conversion

Wasm 1.0: `i32.trunc_f32_s` — f32 to i32 (signed, trapping).
```

```{wasm_spec} i32.trunc_f32_u
:id: WASM_I32_TRUNC_F32_U
:wasm_section: §2.4.1
:wasm_opcode: i32.trunc_f32_u
:tags: numeric, conversion

Wasm 1.0: `i32.trunc_f32_u` — f32 to i32 (unsigned, trapping).
```

```{wasm_spec} i32.trunc_f64_s
:id: WASM_I32_TRUNC_F64_S
:wasm_section: §2.4.1
:wasm_opcode: i32.trunc_f64_s
:tags: numeric, conversion

Wasm 1.0: `i32.trunc_f64_s` — f64 to i32 (signed, trapping).
```

```{wasm_spec} i32.trunc_f64_u
:id: WASM_I32_TRUNC_F64_U
:wasm_section: §2.4.1
:wasm_opcode: i32.trunc_f64_u
:tags: numeric, conversion

Wasm 1.0: `i32.trunc_f64_u` — f64 to i32 (unsigned, trapping).
```

```{wasm_spec} i64.trunc_f32_s
:id: WASM_I64_TRUNC_F32_S
:wasm_section: §2.4.1
:wasm_opcode: i64.trunc_f32_s
:tags: numeric, conversion

Wasm 1.0: `i64.trunc_f32_s` — f32 to i64 (signed, trapping).
```

```{wasm_spec} i64.trunc_f32_u
:id: WASM_I64_TRUNC_F32_U
:wasm_section: §2.4.1
:wasm_opcode: i64.trunc_f32_u
:tags: numeric, conversion

Wasm 1.0: `i64.trunc_f32_u` — f32 to i64 (unsigned, trapping).
```

```{wasm_spec} i64.trunc_f64_s
:id: WASM_I64_TRUNC_F64_S
:wasm_section: §2.4.1
:wasm_opcode: i64.trunc_f64_s
:tags: numeric, conversion

Wasm 1.0: `i64.trunc_f64_s` — f64 to i64 (signed, trapping).
```

```{wasm_spec} i64.trunc_f64_u
:id: WASM_I64_TRUNC_F64_U
:wasm_section: §2.4.1
:wasm_opcode: i64.trunc_f64_u
:tags: numeric, conversion

Wasm 1.0: `i64.trunc_f64_u` — f64 to i64 (unsigned, trapping).
```

```{wasm_spec} f32.convert_i32_s
:id: WASM_F32_CONVERT_I32_S
:wasm_section: §2.4.1
:wasm_opcode: f32.convert_i32_s
:tags: numeric, conversion

Wasm 1.0: `f32.convert_i32_s` — i32 to f32 (signed).
```

```{wasm_spec} f32.convert_i32_u
:id: WASM_F32_CONVERT_I32_U
:wasm_section: §2.4.1
:wasm_opcode: f32.convert_i32_u
:tags: numeric, conversion

Wasm 1.0: `f32.convert_i32_u` — i32 to f32 (unsigned).
```

```{wasm_spec} f32.convert_i64_s
:id: WASM_F32_CONVERT_I64_S
:wasm_section: §2.4.1
:wasm_opcode: f32.convert_i64_s
:tags: numeric, conversion

Wasm 1.0: `f32.convert_i64_s` — i64 to f32 (signed).
```

```{wasm_spec} f32.convert_i64_u
:id: WASM_F32_CONVERT_I64_U
:wasm_section: §2.4.1
:wasm_opcode: f32.convert_i64_u
:tags: numeric, conversion

Wasm 1.0: `f32.convert_i64_u` — i64 to f32 (unsigned).
```

```{wasm_spec} f64.convert_i32_s
:id: WASM_F64_CONVERT_I32_S
:wasm_section: §2.4.1
:wasm_opcode: f64.convert_i32_s
:tags: numeric, conversion

Wasm 1.0: `f64.convert_i32_s` — i32 to f64 (signed).
```

```{wasm_spec} f64.convert_i32_u
:id: WASM_F64_CONVERT_I32_U
:wasm_section: §2.4.1
:wasm_opcode: f64.convert_i32_u
:tags: numeric, conversion

Wasm 1.0: `f64.convert_i32_u` — i32 to f64 (unsigned).
```

```{wasm_spec} f64.convert_i64_s
:id: WASM_F64_CONVERT_I64_S
:wasm_section: §2.4.1
:wasm_opcode: f64.convert_i64_s
:tags: numeric, conversion

Wasm 1.0: `f64.convert_i64_s` — i64 to f64 (signed).
```

```{wasm_spec} f64.convert_i64_u
:id: WASM_F64_CONVERT_I64_U
:wasm_section: §2.4.1
:wasm_opcode: f64.convert_i64_u
:tags: numeric, conversion

Wasm 1.0: `f64.convert_i64_u` — i64 to f64 (unsigned).
```

```{wasm_spec} f32.demote_f64
:id: WASM_F32_DEMOTE_F64
:wasm_section: §2.4.1
:wasm_opcode: f32.demote_f64
:tags: numeric, conversion

Wasm 1.0: `f32.demote_f64` — f64 to f32.
```

```{wasm_spec} f64.promote_f32
:id: WASM_F64_PROMOTE_F32
:wasm_section: §2.4.1
:wasm_opcode: f64.promote_f32
:tags: numeric, conversion

Wasm 1.0: `f64.promote_f32` — f32 to f64.
```

```{wasm_spec} i32.reinterpret_f32
:id: WASM_I32_REINTERPRET_F32
:wasm_section: §2.4.1
:wasm_opcode: i32.reinterpret_f32
:tags: numeric, conversion

Wasm 1.0: `i32.reinterpret_f32` — f32 to i32 (bitcast).
```

```{wasm_spec} i64.reinterpret_f64
:id: WASM_I64_REINTERPRET_F64
:wasm_section: §2.4.1
:wasm_opcode: i64.reinterpret_f64
:tags: numeric, conversion

Wasm 1.0: `i64.reinterpret_f64` — f64 to i64 (bitcast).
```

```{wasm_spec} f32.reinterpret_i32
:id: WASM_F32_REINTERPRET_I32
:wasm_section: §2.4.1
:wasm_opcode: f32.reinterpret_i32
:tags: numeric, conversion

Wasm 1.0: `f32.reinterpret_i32` — i32 to f32 (bitcast).
```

```{wasm_spec} f64.reinterpret_i64
:id: WASM_F64_REINTERPRET_I64
:wasm_section: §2.4.1
:wasm_opcode: f64.reinterpret_i64
:tags: numeric, conversion

Wasm 1.0: `f64.reinterpret_i64` — i64 to f64 (bitcast).
```
