# Binary Format

Wasm 1.0 binary format (§5).

Source: [W3C WebAssembly Core Specification 1.0, §5](https://www.w3.org/TR/wasm-core-1/#binary-format%E2%91%A0)

```{wasm_spec} Binary type encoding
:id: WASM_BIN_TYPES
:wasm_section: §5.3
:tags: binary, types

Wasm 1.0 §5.3: Binary encoding of types — value types (0x7F=i32, 0x7E=i64,
0x7D=f32, 0x7C=f64), function types (0x60 prefix), limits, memory types,
table types, global types.
```

```{wasm_spec} Binary instruction encoding
:id: WASM_BIN_INSTRUCTIONS
:wasm_section: §5.4
:tags: binary, instructions

Wasm 1.0 §5.4: Binary encoding of instructions — single-byte opcodes (0x00
through 0xBF), memarg encoding (alignment + offset), block type encoding,
br_table encoding.
```

```{wasm_spec} Binary module encoding
:id: WASM_BIN_MODULES
:wasm_section: §5.5
:tags: binary, module

Wasm 1.0 §5.5: Binary encoding of the module structure — magic number
(0x00 0x61 0x73 0x6D), version (0x01), section-based layout.
```

```{wasm_spec} Binary section encoding
:id: WASM_BIN_SECTIONS
:wasm_section: §5.5.2
:tags: binary, module, sections

Wasm 1.0 §5.5.2: Encoding of the 12 known section types: custom (0), type (1),
import (2), function (3), table (4), memory (5), global (6), export (7),
start (8), element (9), code (10), data (11). Sections must appear in order.
```
