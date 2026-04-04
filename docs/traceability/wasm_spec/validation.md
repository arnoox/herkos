# Validation

Wasm 1.0 validation rules (§3).

Source: [W3C WebAssembly Core Specification 1.0, §3](https://www.w3.org/TR/wasm-core-1/#validation%E2%91%A0)

## Instruction Validation (§3.3)

```{wasm_spec} Numeric instruction validation
:id: WASM_VALID_NUMERIC
:wasm_section: §3.3.1
:tags: validation, numeric

Wasm 1.0 §3.3.1: Typing rules for numeric instructions — each instruction
consumes and produces values of specific types on the operand stack.
```

```{wasm_spec} Parametric instruction validation
:id: WASM_VALID_PARAMETRIC
:wasm_section: §3.3.2
:tags: validation, parametric

Wasm 1.0 §3.3.2: Typing rules for `drop` and `select`. `select` is
value-polymorphic (operand type unconstrained).
```

```{wasm_spec} Variable instruction validation
:id: WASM_VALID_VARIABLE
:wasm_section: §3.3.3
:tags: validation, variable

Wasm 1.0 §3.3.3: Typing rules for local and global variable instructions.
Validates type consistency with the context's local/global declarations.
```

```{wasm_spec} Memory instruction validation
:id: WASM_VALID_MEMORY
:wasm_section: §3.3.4
:tags: validation, memory

Wasm 1.0 §3.3.4: Typing rules for memory instructions. Validates that a memory
exists (index 0), and that alignment is within bounds for the access width.
```

```{wasm_spec} Control instruction validation
:id: WASM_VALID_CONTROL
:wasm_section: §3.3.5
:tags: validation, control

Wasm 1.0 §3.3.5: Typing rules for control instructions. Validates block/loop/if
types, branch target label depths, and call type signatures.
```

## Module Validation (§3.4)

```{wasm_spec} Function validation
:id: WASM_VALID_FUNCTIONS
:wasm_section: §3.4.1
:tags: validation, function

Wasm 1.0 §3.4.1: A function is valid when its body expression is valid under
a context with the function's locals and the expected result type.
```

```{wasm_spec} Table validation
:id: WASM_VALID_TABLES
:wasm_section: §3.4.2
:tags: validation, table

Wasm 1.0 §3.4.2: A table definition is valid when its type is valid
(limits within range, element type is funcref).
```

```{wasm_spec} Memory validation
:id: WASM_VALID_MEMORIES
:wasm_section: §3.4.3
:tags: validation, memory

Wasm 1.0 §3.4.3: A memory definition is valid when its type is valid
(limits within the maximum page count of 65536).
```

```{wasm_spec} Global validation
:id: WASM_VALID_GLOBALS
:wasm_section: §3.4.4
:tags: validation, global

Wasm 1.0 §3.4.4: A global definition is valid when its initializer expression
is a constant expression of the declared type.
```

```{wasm_spec} Module validation
:id: WASM_VALID_MODULE
:wasm_section: §3.4.10
:tags: validation, module

Wasm 1.0 §3.4.10: A module is valid when all its components are valid and
cross-component constraints are met (at most one memory, at most one table,
export names are unique, start function type is [] -> []).
```
