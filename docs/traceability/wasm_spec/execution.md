# Execution

Wasm 1.0 execution semantics (§4).

Source: [W3C WebAssembly Core Specification 1.0, §4](https://www.w3.org/TR/wasm-core-1/#execution%E2%91%A0)

## Numerics (§4.3)

```{wasm_spec} Integer operations semantics
:id: WASM_EXEC_INTEGER_OPS
:wasm_section: §4.3.2
:tags: execution, numeric, integer

Wasm 1.0 §4.3.2: Execution semantics for integer operations — wrapping
arithmetic, signed/unsigned division (trapping on zero or overflow), shifts,
rotations, comparisons, and test (eqz).
```

```{wasm_spec} Floating-point operations semantics
:id: WASM_EXEC_FLOAT_OPS
:wasm_section: §4.3.3
:tags: execution, numeric, float

Wasm 1.0 §4.3.3: Execution semantics for floating-point operations following
IEEE 754 — arithmetic, min/max, sqrt, rounding, comparisons. NaN propagation
per the Wasm spec's deterministic NaN rules.
```

```{wasm_spec} Conversion operations semantics
:id: WASM_EXEC_CONVERSIONS
:wasm_section: §4.3.4
:tags: execution, numeric, conversion

Wasm 1.0 §4.3.4: Execution semantics for type conversions — truncation
(trapping on NaN/overflow), extension, wrapping, promotion, demotion,
reinterpretation (bitcast).
```

## Instructions (§4.4)

```{wasm_spec} Parametric instruction execution
:id: WASM_EXEC_PARAMETRIC
:wasm_section: §4.4.2
:tags: execution, parametric

Wasm 1.0 §4.4.2: Execution of `drop` (discard top of stack) and `select`
(ternary selection based on i32 condition).
```

```{wasm_spec} Variable instruction execution
:id: WASM_EXEC_VARIABLE
:wasm_section: §4.4.3
:tags: execution, variable

Wasm 1.0 §4.4.3: Execution of `local.get`, `local.set`, `local.tee`,
`global.get`, `global.set` — reading and writing locals and globals.
```

```{wasm_spec} Memory instruction execution
:id: WASM_EXEC_MEMORY
:wasm_section: §4.4.4
:tags: execution, memory

Wasm 1.0 §4.4.4: Execution of memory instructions — load/store with
effective address = base + offset, trapping on out-of-bounds. Sub-width
access with sign/zero extension. `memory.size` and `memory.grow`.
```

```{wasm_spec} Control instruction execution
:id: WASM_EXEC_CONTROL
:wasm_section: §4.4.5
:tags: execution, control

Wasm 1.0 §4.4.5: Execution of control instructions — block entry/exit, branch
resolution, label unwinding, `unreachable` trap, `nop`.
```

```{wasm_spec} Function call execution
:id: WASM_EXEC_CALLS
:wasm_section: §4.4.7
:tags: execution, call

Wasm 1.0 §4.4.7: Execution of `call` (direct) and `call_indirect` (via table
with type check). Frame push/pop, argument passing, result collection. Traps
on type mismatch or undefined table element.
```

## Modules (§4.5)

```{wasm_spec} Module instantiation
:id: WASM_EXEC_INSTANTIATION
:wasm_section: §4.5.4
:tags: execution, module, instantiation

Wasm 1.0 §4.5.4: Module instantiation — allocating functions, tables, memories,
globals; initializing tables from element segments and memories from data
segments; invoking the start function.
```

```{wasm_spec} Function invocation
:id: WASM_EXEC_INVOCATION
:wasm_section: §4.5.5
:tags: execution, module, invocation

Wasm 1.0 §4.5.5: Invocation of an exported function from the host — argument
validation, frame creation, body execution, result extraction.
```
