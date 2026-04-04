# Non-Numeric Instructions

Wasm 1.0 parametric, variable, memory, and control instructions.

## Parametric Instructions (§2.4.2)

Source: [W3C WebAssembly Core Specification 1.0, §2.4.2](https://www.w3.org/TR/wasm-core-1/#parametric-instructions%E2%91%A0)

```{wasm_spec} drop
:id: WASM_DROP
:wasm_section: §2.4.2
:wasm_opcode: drop
:tags: parametric

Wasm 1.0: `drop` — discard a single operand from the stack.
```

```{wasm_spec} select
:id: WASM_SELECT
:wasm_section: §2.4.2
:wasm_opcode: select
:tags: parametric

Wasm 1.0: `select` — choose between two operands based on a condition.
```

## Variable Instructions (§2.4.3)

Source: [W3C WebAssembly Core Specification 1.0, §2.4.3](https://www.w3.org/TR/wasm-core-1/#variable-instructions%E2%91%A0)

```{wasm_spec} local.get
:id: WASM_LOCAL_GET
:wasm_section: §2.4.3
:wasm_opcode: local.get
:tags: variable, local

Wasm 1.0: `local.get` — read a local variable.
```

```{wasm_spec} local.set
:id: WASM_LOCAL_SET
:wasm_section: §2.4.3
:wasm_opcode: local.set
:tags: variable, local

Wasm 1.0: `local.set` — write a local variable.
```

```{wasm_spec} local.tee
:id: WASM_LOCAL_TEE
:wasm_section: §2.4.3
:wasm_opcode: local.tee
:tags: variable, local

Wasm 1.0: `local.tee` — write a local variable and return the value.
```

```{wasm_spec} global.get
:id: WASM_GLOBAL_GET
:wasm_section: §2.4.3
:wasm_opcode: global.get
:tags: variable, global

Wasm 1.0: `global.get` — read a global variable.
```

```{wasm_spec} global.set
:id: WASM_GLOBAL_SET
:wasm_section: §2.4.3
:wasm_opcode: global.set
:tags: variable, global

Wasm 1.0: `global.set` — write a mutable global variable.
```

## Memory Instructions (§2.4.4)

Source: [W3C WebAssembly Core Specification 1.0, §2.4.4](https://www.w3.org/TR/wasm-core-1/#memory-instructions%E2%91%A0)

### Loads

```{wasm_spec} i32.load
:id: WASM_I32_LOAD
:wasm_section: §2.4.4
:wasm_opcode: i32.load
:tags: memory, load, i32

Wasm 1.0: `i32.load` — load 32-bit integer from linear memory.
```

```{wasm_spec} i64.load
:id: WASM_I64_LOAD
:wasm_section: §2.4.4
:wasm_opcode: i64.load
:tags: memory, load, i64

Wasm 1.0: `i64.load` — load 64-bit integer from linear memory.
```

```{wasm_spec} f32.load
:id: WASM_F32_LOAD
:wasm_section: §2.4.4
:wasm_opcode: f32.load
:tags: memory, load, f32

Wasm 1.0: `f32.load` — load 32-bit float from linear memory.
```

```{wasm_spec} f64.load
:id: WASM_F64_LOAD
:wasm_section: §2.4.4
:wasm_opcode: f64.load
:tags: memory, load, f64

Wasm 1.0: `f64.load` — load 64-bit float from linear memory.
```

### Sub-width Loads

```{wasm_spec} i32.load8_s
:id: WASM_I32_LOAD8_S
:wasm_section: §2.4.4
:wasm_opcode: i32.load8_s
:tags: memory, load, i32, subwidth

Wasm 1.0: `i32.load8_s` — load 8-bit value, sign-extend to i32.
```

```{wasm_spec} i32.load8_u
:id: WASM_I32_LOAD8_U
:wasm_section: §2.4.4
:wasm_opcode: i32.load8_u
:tags: memory, load, i32, subwidth

Wasm 1.0: `i32.load8_u` — load 8-bit value, zero-extend to i32.
```

```{wasm_spec} i32.load16_s
:id: WASM_I32_LOAD16_S
:wasm_section: §2.4.4
:wasm_opcode: i32.load16_s
:tags: memory, load, i32, subwidth

Wasm 1.0: `i32.load16_s` — load 16-bit value, sign-extend to i32.
```

```{wasm_spec} i32.load16_u
:id: WASM_I32_LOAD16_U
:wasm_section: §2.4.4
:wasm_opcode: i32.load16_u
:tags: memory, load, i32, subwidth

Wasm 1.0: `i32.load16_u` — load 16-bit value, zero-extend to i32.
```

```{wasm_spec} i64.load8_s
:id: WASM_I64_LOAD8_S
:wasm_section: §2.4.4
:wasm_opcode: i64.load8_s
:tags: memory, load, i64, subwidth

Wasm 1.0: `i64.load8_s` — load 8-bit value, sign-extend to i64.
```

```{wasm_spec} i64.load8_u
:id: WASM_I64_LOAD8_U
:wasm_section: §2.4.4
:wasm_opcode: i64.load8_u
:tags: memory, load, i64, subwidth

Wasm 1.0: `i64.load8_u` — load 8-bit value, zero-extend to i64.
```

```{wasm_spec} i64.load16_s
:id: WASM_I64_LOAD16_S
:wasm_section: §2.4.4
:wasm_opcode: i64.load16_s
:tags: memory, load, i64, subwidth

Wasm 1.0: `i64.load16_s` — load 16-bit value, sign-extend to i64.
```

```{wasm_spec} i64.load16_u
:id: WASM_I64_LOAD16_U
:wasm_section: §2.4.4
:wasm_opcode: i64.load16_u
:tags: memory, load, i64, subwidth

Wasm 1.0: `i64.load16_u` — load 16-bit value, zero-extend to i64.
```

```{wasm_spec} i64.load32_s
:id: WASM_I64_LOAD32_S
:wasm_section: §2.4.4
:wasm_opcode: i64.load32_s
:tags: memory, load, i64, subwidth

Wasm 1.0: `i64.load32_s` — load 32-bit value, sign-extend to i64.
```

```{wasm_spec} i64.load32_u
:id: WASM_I64_LOAD32_U
:wasm_section: §2.4.4
:wasm_opcode: i64.load32_u
:tags: memory, load, i64, subwidth

Wasm 1.0: `i64.load32_u` — load 32-bit value, zero-extend to i64.
```

### Stores

```{wasm_spec} i32.store
:id: WASM_I32_STORE
:wasm_section: §2.4.4
:wasm_opcode: i32.store
:tags: memory, store, i32

Wasm 1.0: `i32.store` — store 32-bit integer to linear memory.
```

```{wasm_spec} i64.store
:id: WASM_I64_STORE
:wasm_section: §2.4.4
:wasm_opcode: i64.store
:tags: memory, store, i64

Wasm 1.0: `i64.store` — store 64-bit integer to linear memory.
```

```{wasm_spec} f32.store
:id: WASM_F32_STORE
:wasm_section: §2.4.4
:wasm_opcode: f32.store
:tags: memory, store, f32

Wasm 1.0: `f32.store` — store 32-bit float to linear memory.
```

```{wasm_spec} f64.store
:id: WASM_F64_STORE
:wasm_section: §2.4.4
:wasm_opcode: f64.store
:tags: memory, store, f64

Wasm 1.0: `f64.store` — store 64-bit float to linear memory.
```

### Sub-width Stores

```{wasm_spec} i32.store8
:id: WASM_I32_STORE8
:wasm_section: §2.4.4
:wasm_opcode: i32.store8
:tags: memory, store, i32, subwidth

Wasm 1.0: `i32.store8` — store low 8 bits of i32 to linear memory.
```

```{wasm_spec} i32.store16
:id: WASM_I32_STORE16
:wasm_section: §2.4.4
:wasm_opcode: i32.store16
:tags: memory, store, i32, subwidth

Wasm 1.0: `i32.store16` — store low 16 bits of i32 to linear memory.
```

```{wasm_spec} i64.store8
:id: WASM_I64_STORE8
:wasm_section: §2.4.4
:wasm_opcode: i64.store8
:tags: memory, store, i64, subwidth

Wasm 1.0: `i64.store8` — store low 8 bits of i64 to linear memory.
```

```{wasm_spec} i64.store16
:id: WASM_I64_STORE16
:wasm_section: §2.4.4
:wasm_opcode: i64.store16
:tags: memory, store, i64, subwidth

Wasm 1.0: `i64.store16` — store low 16 bits of i64 to linear memory.
```

```{wasm_spec} i64.store32
:id: WASM_I64_STORE32
:wasm_section: §2.4.4
:wasm_opcode: i64.store32
:tags: memory, store, i64, subwidth

Wasm 1.0: `i64.store32` — store low 32 bits of i64 to linear memory.
```

### Memory Management

```{wasm_spec} memory.size
:id: WASM_MEMORY_SIZE
:wasm_section: §2.4.4
:wasm_opcode: memory.size
:tags: memory, management

Wasm 1.0: `memory.size` — return current memory size in pages.
```

```{wasm_spec} memory.grow
:id: WASM_MEMORY_GROW
:wasm_section: §2.4.4
:wasm_opcode: memory.grow
:tags: memory, management

Wasm 1.0: `memory.grow` — grow memory by delta pages, return previous size or -1.
```

## Control Instructions (§2.4.5)

Source: [W3C WebAssembly Core Specification 1.0, §2.4.5](https://www.w3.org/TR/wasm-core-1/#control-instructions%E2%91%A0)

```{wasm_spec} nop
:id: WASM_NOP
:wasm_section: §2.4.5
:wasm_opcode: nop
:tags: control

Wasm 1.0: `nop` — no operation.
```

```{wasm_spec} unreachable
:id: WASM_UNREACHABLE
:wasm_section: §2.4.5
:wasm_opcode: unreachable
:tags: control

Wasm 1.0: `unreachable` — cause an unconditional trap.
```

```{wasm_spec} block
:id: WASM_BLOCK
:wasm_section: §2.4.5
:wasm_opcode: block
:tags: control, structured

Wasm 1.0: `block` — begin a structured block of instructions with a label.
```

```{wasm_spec} loop
:id: WASM_LOOP
:wasm_section: §2.4.5
:wasm_opcode: loop
:tags: control, structured

Wasm 1.0: `loop` — begin a structured loop with a label (branch target is the
loop header).
```

```{wasm_spec} if
:id: WASM_IF
:wasm_section: §2.4.5
:wasm_opcode: if
:tags: control, structured

Wasm 1.0: `if` — conditional block: execute then-branch or else-branch based
on a condition.
```

```{wasm_spec} br
:id: WASM_BR
:wasm_section: §2.4.5
:wasm_opcode: br
:tags: control, branch

Wasm 1.0: `br` — unconditional branch to the label at the given depth.
```

```{wasm_spec} br_if
:id: WASM_BR_IF
:wasm_section: §2.4.5
:wasm_opcode: br_if
:tags: control, branch

Wasm 1.0: `br_if` — conditional branch: branch if the condition is non-zero.
```

```{wasm_spec} br_table
:id: WASM_BR_TABLE
:wasm_section: §2.4.5
:wasm_opcode: br_table
:tags: control, branch

Wasm 1.0: `br_table` — indirect branch via operand indexing into a table of
labels, with a default target.
```

```{wasm_spec} call
:id: WASM_CALL
:wasm_section: §2.4.5
:wasm_opcode: call
:tags: control, call

Wasm 1.0: `call` — direct function call by function index.
```

```{wasm_spec} call_indirect
:id: WASM_CALL_INDIRECT
:wasm_section: §2.4.5
:wasm_opcode: call_indirect
:tags: control, call, indirect

Wasm 1.0: `call_indirect` — indirect function call via table, with type check.
```

```{wasm_spec} return
:id: WASM_RETURN
:wasm_section: §2.4.5
:wasm_opcode: return
:tags: control

Wasm 1.0: `return` — return from the current function.
```
