# Modules

Wasm 1.0 module constructs (§2.5).

Source: [W3C WebAssembly Core Specification 1.0, §2.5](https://www.w3.org/TR/wasm-core-1/#modules%E2%91%A0)

```{wasm_spec} Type section
:id: WASM_MOD_TYPES
:wasm_section: §2.5.2
:tags: module, types

Wasm 1.0 §2.5.2: The type section defines a vector of function types used by
the module. Function signatures are referenced by type index throughout the
module.
```

```{wasm_spec} Functions
:id: WASM_MOD_FUNCTIONS
:wasm_section: §2.5.3
:tags: module, function

Wasm 1.0 §2.5.3: Functions are defined by their type, a vector of local
variable declarations, and a body (expression). Parameters are addressed
as the first locals.
```

```{wasm_spec} Tables
:id: WASM_MOD_TABLES
:wasm_section: §2.5.4
:tags: module, table

Wasm 1.0 §2.5.4: A table is a vector of opaque values of a given reference
type (`funcref` in 1.0). At most one table per module. Used for indirect calls.
```

```{wasm_spec} Memories
:id: WASM_MOD_MEMORIES
:wasm_section: §2.5.5
:tags: module, memory

Wasm 1.0 §2.5.5: A memory is a vector of raw bytes (linear memory). At most
one memory per module. Size is specified in units of page size (64 KiB).
```

```{wasm_spec} Globals
:id: WASM_MOD_GLOBALS
:wasm_section: §2.5.6
:tags: module, global

Wasm 1.0 §2.5.6: A global variable holds a single value of a given type. It
is either mutable or immutable and has a constant initializer expression.
```

```{wasm_spec} Element segments
:id: WASM_MOD_ELEM
:wasm_section: §2.5.7
:tags: module, table, element

Wasm 1.0 §2.5.7: Element segments initialize table ranges with function
references. An active segment copies elements into the table during
instantiation at a given offset.
```

```{wasm_spec} Data segments
:id: WASM_MOD_DATA
:wasm_section: §2.5.8
:tags: module, memory, data

Wasm 1.0 §2.5.8: Data segments initialize memory ranges with byte data. An
active segment copies bytes into linear memory during instantiation at a given
offset.
```

```{wasm_spec} Start function
:id: WASM_MOD_START
:wasm_section: §2.5.9
:tags: module, start

Wasm 1.0 §2.5.9: The start component declares a function index that is
automatically invoked after instantiation, before any exports become accessible.
```

```{wasm_spec} Exports
:id: WASM_MOD_EXPORTS
:wasm_section: §2.5.10
:tags: module, export

Wasm 1.0 §2.5.10: Exports make functions, tables, memories, or globals
accessible to the host environment under unique string names.
```

```{wasm_spec} Imports
:id: WASM_MOD_IMPORTS
:wasm_section: §2.5.11
:tags: module, import

Wasm 1.0 §2.5.11: Imports declare functions, tables, memories, or globals
provided by the host environment, identified by a two-level name (module, name).
```
