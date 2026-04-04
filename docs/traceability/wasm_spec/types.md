# Types

Wasm 1.0 type constructs (§2.3).

Source: [W3C WebAssembly Core Specification 1.0, §2.3](https://www.w3.org/TR/wasm-core-1/#types%E2%91%A0)

## Value Types

```{wasm_spec} Value type i32
:id: WASM_VALTYPE_I32
:wasm_section: §2.3.1
:tags: type, valtype, i32

Wasm 1.0 §2.3.1: 32-bit integer value type.
```

```{wasm_spec} Value type i64
:id: WASM_VALTYPE_I64
:wasm_section: §2.3.1
:tags: type, valtype, i64

Wasm 1.0 §2.3.1: 64-bit integer value type.
```

```{wasm_spec} Value type f32
:id: WASM_VALTYPE_F32
:wasm_section: §2.3.1
:tags: type, valtype, f32

Wasm 1.0 §2.3.1: 32-bit IEEE 754 floating-point value type.
```

```{wasm_spec} Value type f64
:id: WASM_VALTYPE_F64
:wasm_section: §2.3.1
:tags: type, valtype, f64

Wasm 1.0 §2.3.1: 64-bit IEEE 754 floating-point value type.
```

## Composite Types

```{wasm_spec} Result types
:id: WASM_RESULT_TYPE
:wasm_section: §2.3.2
:tags: type, result

Wasm 1.0 §2.3.2: Result types classify the result of executing instructions
or blocks, as a sequence of values. In Wasm 1.0, at most one result value is
permitted.
```

```{wasm_spec} Function types
:id: WASM_FUNC_TYPE
:wasm_section: §2.3.3
:tags: type, function

Wasm 1.0 §2.3.3: Function types classify the signature of functions, mapping a
vector of parameter types to a vector of result types (at most one in 1.0).
```

```{wasm_spec} Limits
:id: WASM_LIMITS
:wasm_section: §2.3.4
:tags: type, limits

Wasm 1.0 §2.3.4: Limits classify the size range of resizeable storage
(memories and tables), with a required minimum and optional maximum.
```

```{wasm_spec} Memory types
:id: WASM_MEMORY_TYPE
:wasm_section: §2.3.5
:tags: type, memory

Wasm 1.0 §2.3.5: Memory types classify linear memories and their size range,
specified in units of page size (64 KiB).
```

```{wasm_spec} Table types
:id: WASM_TABLE_TYPE
:wasm_section: §2.3.6
:tags: type, table

Wasm 1.0 §2.3.6: Table types classify tables over elements of reference type
within a size range. In Wasm 1.0, the only element type is `funcref`.
```

```{wasm_spec} Global types
:id: WASM_GLOBAL_TYPE
:wasm_section: §2.3.7
:tags: type, global

Wasm 1.0 §2.3.7: Global types classify global variables, which hold a value
of the given value type and can either be mutable or immutable.
```

```{wasm_spec} External types
:id: WASM_EXTERNAL_TYPE
:wasm_section: §2.3.8
:tags: type, external

Wasm 1.0 §2.3.8: External types classify imports and external values with
their respective types (function, table, memory, global).
```
