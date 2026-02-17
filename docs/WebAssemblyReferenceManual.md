WebAssembly Reference Manual
================================================================================

0. [Introduction](#introduction)
0. [Basics](#basics)
0. [Module](#module)
0. [Instruction Descriptions](#instruction-descriptions)
0. [Instructions](#instructions)
0. [Instantiation](#instantiation)
0. [Execution](#execution)
0. [Text Format](#text-format)

![WebAssembly Logo][logo]

[logo]: https://raw.githubusercontent.com/WebAssembly/web-assembly-logo/main/dist/logo/web-assembly-logo.png "The WebAssembly Logo"

(introduction)=
Introduction
--------------------------------------------------------------------------------

WebAssembly, or "wasm", is a general-purpose virtual [ISA] designed to be a
compilation target for a wide variety of programming languages. Much of its
distinct personality derives from its security, code compression, and decoding
optimization features.

```{req} Title
:id: REQ_MODULE
:status: open
:tags: wasm
The unit of WebAssembly code is the [*module*](#module). Modules consist of a
header followed by a sequence of sections. There are sections describing a
WebAssembly's interactions with other modules ([imports] and [exports]),
sections declaring [data](#data-section) and other implements used by the
module, and sections defining [*functions*].
```

```{req} Title
:id: REQ_MODULE_FORMS
:status: open
:tags: wasm
WebAssembly modules are encoded in binary form for size and decoding efficiency.
They may be losslessly translated to [text form] for readability.
```

```{req} Title
:id: REQ_MODULE_VALIDATION
:status: open
:tags: wasm
WebAssembly code must be validated before it can be instantiated and executed.
WebAssembly is designed to allow decoding and validation to be performed in a
single linear pass through a WebAssembly module, and to enable many parts of
decoding and validation to be performed concurrently.
```

For example, [loops are explicitly identified](#loop), and decoders can be sure that program
state is consistent at all control flow merge points within a function without
having to see the entire function body first.

```{req} Title
:id: REQ_MODULE_INSTANTIATION
:status: open
:tags: wasm
A WebAssembly module can be [*instantiated*] to produce a WebAssembly instance,
which contains all the data structures required by the module's code for
execution.
```

```{req} Title
:id: REQ_MODULE_MEMORY
:status: open
:tags: wasm
Instances can include [linear memories], which can serve the purpose
of an address space for program data. For security and determinism, linear
memory is *sandboxed*, and the other data structures in an instance, including
the call stack, are allocated outside of linear memory so that they cannot be
corrupted by errant linear-memory accesses.
```

```{req} Title
:id: REQ_MODULE_TABLE
:status: open
:tags: wasm
Instances can also include [tables],
which can serve the purpose of an address space for indirect function calls,
among other things.
```

```{req} Title
:id: REQ_MODULE_EXECUTION
:status: open
:tags: wasm
An instance can then be executed, either by execution of its
[start function](#start-section) or by calls to its exported functions, and its
exported linear memories and global variables can be accessed.
```

```{req} Title
:id: REQ_MODULE_INSTRUCTIONS
:status: open
:tags: wasm
Along with the other contents, each function contains a sequence of
[*instructions*], which are [described](#instruction-descriptions) here with some
simple conventions. There are instructions for performing integer and
floating-point arithmetic, directing control flow, loading and storing to linear
memory (as a [load-store architecture]), calling functions, and more. During
[*execution*], instructions conceptually communicate with each other primarily
via pushing and popping values on a virtual stack, which allows them to have a
very compact encoding.
```

Implementations of WebAssembly need not perform all the steps literally as
described here; they need only behave ["as if"] they did so in all observable
respects.

> Except where specified otherwise, WebAssembly instructions are not required to
> execute in [constant time].

[ISA]: https://en.wikipedia.org/wiki/Instruction_set
[imports]: #import-section
[exports]: #export-section
[*execution*]: #execution
[*functions*]: https://en.wikipedia.org/wiki/Subroutine
[*instructions*]: #instructions
[*instantiated*]: #instantiation
[load-store architecture]: https://en.wikipedia.org/wiki/Load/store_architecture
["as if"]: https://en.wikipedia.org/wiki/As-if_rule
[constant time]: https://www.bearssl.org/constanttime.html

(basics)=
Basics
--------------------------------------------------------------------------------

0. [Bytes](#bytes)
0. [Pages](#pages)
0. [Nondeterminism](#nondeterminism)
0. [Linear Memories](#linear-memories)
0. [Tables](#tables)
0. [Encoding Types](#encoding-types)
0. [Language Types](#language-types)
0. [Embedding Environment](#embedding-environment)

(bytes)=
### Bytes

```{req} Title
:id: REQ_BYTES
:status: open
:tags: wasm
[*Bytes*] in WebAssembly are 8-[bit], and are the addressing unit of
[linear-memory] accesses.
```

[*Bytes*]: https://en.wikipedia.org/wiki/Byte

(pages)=
### Pages

```{req} Title
:id: REQ_PAGES
:status: open
:tags: wasm
[*Pages*] in WebAssembly are 64 [KiB], and are the units used in [linear-memory]
size declarations and size operations.
```

[*Pages*]: https://en.wikipedia.org/wiki/Page_(computer_memory)

(nondeterminism)=
### Nondeterminism

When semantics are specified as *nondeterministic*, a WebAssembly implementation
may perform any one of the discrete set of specified alternatives.

There is no requirement that a given implementation make the same choice every
time, even for successive executions of the same instruction within the same
instance of a module.

```{req} Title
:id: REQ_UNDEFINED_BEHAVIOR
:status: open
:tags: wasm
There is no ["undefined behavior"] in WebAssembly where the semantics become
completely unspecified. Thus, WebAssembly has only
*Limited Local Nondeterminism*.
````

> All instances of nondeterminism in WebAssembly are explicitly described as
such with a link to here.

["undefined behavior"]: https://en.wikipedia.org/wiki/Undefined_behavior

(linear-memories)=
### Linear Memories

```{req} Title
:id: REQ_LINEAR_MEMORY_DEFINITION
:status: open
:tags: wasm
A *linear memory* is a contiguous, [byte]-addressable, readable and writable
range of memory spanning from offset `0` and extending up to a *linear-memory
size*, allocated as part of a WebAssembly instance.
````

```{req} Title
:id: REQ_LINEAR_MEMORY_GROWTH
:status: open
:tags: wasm
The size of a linear memory
is always a multiple of the [page] size and may be increased dynamically (with
the [`memory.grow`](#grow-linear-memory-size) instruction) up to an optional
declared *maximum length*.
```

```{req} Title
:id: REQ_LINEAR_MEMORY_SANDBOXING
:status: open
:tags: wasm
Linear memories are sandboxed, so they don't overlap
with each other or with other parts of a WebAssembly instance, including the
call stack, globals, and tables, and their bounds are enforced.
```

```{req} Title
:id: REQ_LINEAR_MEMORY_SOURCE
:status: open
:tags: wasm
Linear memories can either be [defined by a module](#linear-memory-section)
or [imported](#import-section).
```

(tables)=
### Tables

A *table* is similar to a [linear memory] whose elements, instead of being
bytes, are opaque values. Each table has a [table element type] specifying what
kind of data they hold. A table of `funcref` is used as the index space for
[indirect calls](#indirect-call).

Tables can be [defined by a module](#table-section) or
[imported](#import-section).

> In the future, tables are expected to be generalized to hold a wide variety of
opaque values and serve a wide variety of purposes.

(encoding-types)=
### Encoding Types

0. [Primitive Encoding Types](#primitive-encoding-types)
0. [varuPTR Immediate Type](#varuptr-immediate-type)
0. [memflags Immediate Type](#memflags-immediate-type)
0. [Array](#array)
0. [Byte Array](#byte-array)
0. [Identifier](#identifier)
0. [Type Encoding Type](#language-types)

(primitive-encoding-types)=
#### Primitive Encoding Types

*Primitive encoding types* are the basic types used to represent fields within a
Module.

| Name               | Size (in bytes)  | Description                                  |
| ------------------ | ---------------- | -------------------------------------------- |
| `uint32`           | 4                | unsigned; value limited to 32 bits           |
|                    |                  |                                              |
| `varuint1`         | 1                | unsigned [LEB128]; value limited to 1 bit    |
| `varuint7`         | 1                | unsigned [LEB128]; value limited to 7 bits   |
| `varuint32`        | 1-5              | unsigned [LEB128]; value limited to 32 bits  |
| `varuint64`        | 1-10             | unsigned [LEB128]; value limited to 64 bits  |
|                    |                  |                                              |
| `varsint7`         | 1                | [signed LEB128]; value limited to 7 bits     |
| `varsint32`        | 1-5              | [signed LEB128]; value limited to 32 bits    |
| `varsint64`        | 1-10             | [signed LEB128]; value limited to 64 bits    |
|                    |                  |                                              |
| `float32`          | 4                | IEEE 754-2008 [binary32]                     |
| `float64`          | 8                | IEEE 754-2008 [binary64]                     |

[LEB128] encodings may contain padding `0x80` bytes, and [signed LEB128]
encodings may contain padding `0x80` and `0xff` bytes.

Except when specified otherwise, all values are encoded in
[little-endian byte order].

**Validation:**
 - For the types that have value limits, the encoded value is required to be
   within the limit.
 - The size of the encoded value is required to be within the type's size range.

> These types aren't used to describe values at execution time.

[LEB128]: https://en.wikipedia.org/wiki/LEB128
[signed LEB128]: https://en.wikipedia.org/wiki/LEB128#Signed_LEB128

(varuptr-immediate-type)=
#### varuPTR Immediate Type

A *varuPTR immediate* is either [varuint32] or [varuint64] depending on whether
the linear memory associated with the instruction using it is 32-bit or 64-bit.

(memflags-immediate-type)=
#### memflags Immediate Type

A *memflags immediate* is a [varuint32] containing the following bit fields:

| Name     | Description                                            |
| -------- | ------------------------------------------------------ |
| `$align` | alignment in bytes, encoded as the [log2] of the value |

> As implied by the log2 encoding, `$align` can only be a [power of 2].

> `$flags` may hold additional fields in the future.

[power of 2]: https://en.wikipedia.org/wiki/Power_of_two
[log2]: https://en.wikipedia.org/wiki/Binary_logarithm

(array)=
#### Array

An *array* of a given type is a [varuint32] indicating a number of elements,
followed by a sequence of that many elements of that type.

> Array elements needn't all be the same size in some representations.

(byte-array)=
#### Byte Array

A *byte array* is an [array] of [bytes].

> Byte arrays may contain arbitrary bytes and aren't required to be
[valid UTF-8] or any other format.

(identifier)=
#### Identifier

An *identifier* is a [byte array] which is valid [UTF-8].

**Validation:**
 - Decoding the bytes according to the [UTF-8 decode without BOM or fail]
   algorithm is required to succeed.

[UTF-8 decode without BOM or fail]: https://encoding.spec.whatwg.org/#utf-8-decode-without-bom-or-fail

> Identifiers may contain [NUL] characters, aren't required to be
[NUL-terminated], aren't required to be [normalized][unicode normalization], and
aren't required to be marked with a [BOM] (though they aren't prohibited from
containing a BOM).

[NUL]: https://en.wikipedia.org/wiki/Null_character
[NUL-terminated]: https://en.wikipedia.org/wiki/Null-terminated_string
[BOM]: https://en.wikipedia.org/wiki/Byte_order_mark
[unicode normalization]: http://www.unicode.org/reports/tr15/

> Normalization is not performed when considering whether two identifiers are
the same.

(type-encoding-type)=
#### Type Encoding Type

A *type encoding* is a value indicating a particular [language type].

| Name      | Binary Encoding |
| --------- | --------------- |
| `i32`     | `-0x01`         |
| `i64`     | `-0x02`         |
| `f32`     | `-0x03`         |
| `f64`     | `-0x04`         |
| `funcref` | `-0x10`         |
| `func`    | `-0x20`         |
| `void`    | `-0x40`         |

Type encodings are encoded as their Binary Encoding value in a [varsint7].

**Validation:**
 - A type encoding is required to be one of the values defined here.

(language-types)=
### Language Types

*Language types* describe runtime values and language constructs. Each language
type has a [type encoding](#type-encoding-type).

0. [Value Types](#value-types)
0. [Table Element Types](#table-element-types)
0. [Signature Types](#signature-types)
0. [Block Signature Types](#block-signature-types)

(value-types)=
#### Value Types

*Value types* are the types of individual input and output values of
instructions at execution time.

**Validation:**
 - A value type is required to be one of the integer or floating-point value
   types.

0. [Integer Value Types](#integer-value-types)
0. [Booleans](#booleans)
0. [Floating-Point Value Types](#floating-point-value-types)

(integer-value-types)=
##### Integer Value Types

*Integer value types* describe fixed-width integer values.

| Name  | Bits | Description                                                      |
| ----- | ---- | ---------------------------------------------------------------- |
| `i32` | 32   | [32-bit integer](https://en.wikipedia.org/wiki/32-bit)           |
| `i64` | 64   | [64-bit integer](https://en.wikipedia.org/wiki/64-bit_computing) |

Integer value types in WebAssembly aren't inherently signed or unsigned. They
may be interpreted as signed or unsigned by individual operations. When
interpreted as signed, a [two's complement] interpretation is used.

**Validation:**
 - An integer value type is required to be one of the values defined here.

> The [minimum signed integer value] is supported; consequently, two's
complement signed integers aren't symmetric around zero.

> When used as linear-memory indices or function table indices, integer types
may play the role of "pointers".

> Integer value types are sometimes described as fixed-point types with no
fractional digits in other languages.

(booleans)=
##### Booleans

[Boolean][actual boolean] values in WebAssembly are represented as values of
type `i32`. In a boolean context, such as a `br_if` condition, any non-zero
value is interpreted as true and `0` is interpreted as false.

Any operation that produces a boolean value, such as a comparison, produces the
values `0` and `1` for false and true.

[actual boolean]: https://en.wikipedia.org/wiki/Boolean_data_type

> WebAssembly often uses alternate encodings for integers and boolean values,
rather than using the literal encodings described here.

(floating-point-value-types)=
##### Floating-Point Value Types

*Floating-point value types* describe IEEE 754-2008 floating-point values.

| Name  | Bits | Description                                                    |
| ----- | ---- | -------------------------------------------------------------- |
| `f32` | 32   | IEEE 754-2008 [binary32], commonly known as "single precision" |
| `f64` | 64   | IEEE 754-2008 [binary64], commonly known as "double precision" |

**Validation:**
 - A floating-point value type is required to be one of the values defined here.

> Unlike with [Numbers in ECMAScript], [NaN] values in WebAssembly have sign
bits and significand fields which may be observed and manipulated (though they
are usually unimportant).

[Numbers in ECMAScript]: https://tc39.github.io/ecma262/#sec-ecmascript-language-types-number-type
[NaN]: https://en.wikipedia.org/wiki/NaN

(table-element-types)=
#### Table Element Types

*Table element types* are the types that may be used in a [table].

| Name       | Description                                  |
| ---------- | -------------------------------------------- |
| `funcref`  | a reference to a function with any signature |

**Validation:**
 - Table element types are required to be one of the values defined here.

(signature-types)=
#### Signature Types

*Signature types* are the types that may be defined in the [Type Section].

| Name       | Description                                  |
| ---------- | -------------------------------------------- |
| `func`     | a function signature                         |

**Validation:**
 - Signature types are required to be one of the values defined here.

(block-signature-types)=
#### Block Signature Types

*Block signature types* are the types that may be used as a `block` or other
control-flow construct signature.

Block signature types include the [value types], which indicate single-element
type sequences containing the type, and the following:

| Name       | Description                                  |
| ---------- | -------------------------------------------- |
| `void`     | an empty type sequence                       |

**Validation:**
 - Block signature types are required to be either a [value type] or one of the
   values defined here.

(embedding-environment)=
### Embedding Environment

A WebAssembly runtime environment will typically provide APIs for
interacting with the outside world, as well as mechanisms for loading and
linking wasm modules. This is the *embedding environment*.

```{req} Embedding Environment Must Provide Module Loading Mechanism
:id: REQ_EMBEDDING_MODULE_LOADING
:status: open
:tags: embedding, runtime
A WebAssembly embedding environment must provide a mechanism to load WebAssembly module binaries and convert them into instantiable module representations.
```

```{req} Embedding Environment Must Provide Module Linking Mechanism
:id: REQ_EMBEDDING_MODULE_LINKING
:status: open
:tags: embedding, runtime
A WebAssembly embedding environment must provide a mechanism for linking WebAssembly modules and resolving import dependencies between them.
```

```{req} Embedding Environment Must Provide External API Access
:id: REQ_EMBEDDING_EXTERNAL_API
:status: open
:tags: embedding, runtime
A WebAssembly embedding environment must provide APIs for WebAssembly modules to interact with external resources (host functions, memory, I/O, etc.) in a sandboxed manner.
```


(module)=
Module
--------------------------------------------------------------------------------

0. [Module Contents](#module-contents)
0. [Known Sections](#known-sections)
0. [Custom Sections](#custom-sections)
0. [Module Index Spaces](#module-index-spaces)
0. [Module Types](#module-types)

(module-contents)=
### Module Contents

A module starts with a header:

| Field Name     | Type     | Description                                                             |
| -------------- | -------- | ----------------------------------------------------------------------- |
| `magic_cookie` | [uint32] | magic cookie identifying the contents of a file as a WebAssembly module |
| `version`      | [uint32] | WebAssembly version number.                                             |

The header is then followed by a sequence of sections. Each section consists of
a [varuint7] *opcode* followed by a [byte array] *payload*. The opcode is
required to either indicate a *known section*, or be `0x00`, indicating a
[*custom section*](#custom-sections).

```{req} WebAssembly Module Magic Cookie Must Equal 0x6d736100
:id: REQ_MODULE_MAGIC_COOKIE_VALUE
:status: open
:tags: validation, module-structure
The `magic_cookie` field of a WebAssembly module header must have the value `0x6d736100` (representing the null-terminated string "asm" in UTF-8), enabling identification of WebAssembly binaries by generic file inspection tools.
```

```{req} WebAssembly Module Version Must Equal 0x1
:id: REQ_MODULE_VERSION_VALUE
:status: open
:tags: validation, module-structure
The `version` field of a WebAssembly module header must have the value `0x1` to indicate compatibility with the current WebAssembly specification.
```

```{req} Known Sections Must Appear At Most Once
:id: REQ_MODULE_SECTION_UNIQUENESS
:status: open
:tags: validation, module-structure
Each known section type must appear at most one time within a WebAssembly module. Duplicate section types must be rejected as invalid.
```

```{req} Known Sections Must Follow Canonical Ordering
:id: REQ_MODULE_SECTION_ORDERING
:status: open
:tags: validation, module-structure
Known sections that are present in a WebAssembly module must appear in canonical order as enumerated in the Known Sections specification (Type → Import → Function → Table → Memory → Global → Export → Start → Element → Code → Data).
```

```{req} Custom Sections Must Begin With Identifier Name
:id: REQ_MODULE_CUSTOM_SECTION_NAME
:status: open
:tags: validation, module-structure
Each custom section's payload must be prefixed with an identifier value representing the custom section's name, enabling custom extension mechanisms.
```

```{req} Module Encoding Must Consist Exclusively Of Header And Sections
:id: REQ_MODULE_ENCODING_COMPLETENESS
:status: open
:tags: validation, module-structure
A WebAssembly module's binary encoding must consist exclusively of the module header followed by zero or more sections, with no additional or trailing bytes beyond this structure.
```

```{req} Module Components Must Conform To Encoding Type Requirements
:id: REQ_MODULE_ENCODING_TYPES_COMPLIANCE
:status: open
:tags: validation, module-structure
All component encoding types used within a WebAssembly module must satisfy their respective encoding type validation requirements (size limits, value constraints, etc.).
```

```{req} Known Section Validation Requirements Must Be Applied
:id: REQ_MODULE_KNOWN_SECTION_VALIDATION
:status: open
:tags: validation, module-structure
For each known section present in a WebAssembly module, the section-specific validation requirements must be applied if defined for that section kind.
```

```{req} Index Space Validation Requirements Must Be Applied
:id: REQ_MODULE_INDEX_SPACE_VALIDATION
:status: open
:tags: validation, module-structure
All validation requirements defined for module index spaces must be applied to verify the consistency of function, global, memory, and table index spaces within a module.
```

**Validation:**

> The `magic_cookie` bytes begin with a [NUL] character, indicating to generic
tools that the ensuing contents are not generally "text", followed by the
[UTF-8] encoding of the string "asm".

> The version is expected to change infrequently if ever; forward-compatible
extension is intended to be achieved by adding sections, types, instructions and
others without bumping the version.

> The section byte array length field is usually redundant, as most section
encodings end up specifying their size through other means as well, however it
is still useful to allow streaming decoders to quickly skip over whole sections.

(known-sections)=
### Known Sections

There are several *known sections*:

1. [Type Section]
0. [Import Section]
0. [Function Section]
0. [Table Section]
0. [Linear-Memory Section]
0. [Global Section]
0. [Export Section]
0. [Start Section]
0. [Element Section]
0. [Code Section]
0. [Data Section]

(type-section)=
#### Type Section

**Opcode:** `0x01`.

The Type Section consists of an [array] of function signatures.

Each *function signature* consists of:

| Field Name      | Type               | Description                             |
| --------------- | ------------------ | --------------------------------------- |
| `form`          | [signature type]   | the type of signature                   |

If `form` is `func`, the following fields are appended.

| Field Name      | Type                    | Description                             |
| --------------- | ----------------------- | --------------------------------------- |
| `params`        | [array] of [value type] | the parameters to the function          |
| `returns`       | [array] of [value type] | the return types of the function        |

```{req} Function Signature Form Must Be func
:id: REQ_TYPE_SECTION_FORM_FUNC
:status: open
:tags: validation, type-section
The `form` field of each function signature in the Type Section must be `func`, disallowing other signature forms in the current specification version.
```

```{req} Function Return Types Limited To Single Element
:id: REQ_TYPE_SECTION_SINGLE_RETURN
:status: open
:tags: validation, type-section
Each function signature's `returns` array in the Type Section must contain at most one element, restricting functions to return zero or one value.
```

> In the future, this section may contain other forms of type entries as well,
and support for function signatures with multiple return types.

(import-section)=
#### Import Section

**Opcode:** `0x02`.

The Import Section consists of an [array] of imports.

An *import* consists of:

| Field Name      | Type                 | Description                              |
| --------------- | -------------------- | ---------------------------------------- |
| `module_name`   | [identifier]         | the name of the module to import from    |
| `export_name`   | [identifier]         | the name of the export in that module    |
| `kind`          | [external kind]      | the kind of import                       |

If `kind` is `Function`, the following fields are appended.

| Field Name      | Type                 | Description                              |
| --------------- | -------------------- | ---------------------------------------- |
| `sig_index`     | [varuint32]          | signature index into the [Type Section]  |

If `kind` is `Table`, the following fields are appended.

| Field Name      | Type                 | Description                              |
| --------------- | -------------------- | ---------------------------------------- |
| `desc`          | [table description]  | a description of the table               |

If `kind` is `Memory`, the following fields are appended.

| Field Name      | Type                        | Description                        |
| --------------- | --------------------------- | ---------------------------------- |
| `desc`          | [linear-memory description] | a description of the linear memory |

If `kind` is `Global`, the following fields are appended.

| Field Name      | Type                 | Description                              |
| --------------- | -------------------- | ---------------------------------------- |
| `desc`          | [global description] | a description of the global variable     |

The meaning of an import's `module_name` and `export_name` are determined by
the [embedding environment].

Imports provide access to constructs, defined and allocated by external entities
outside the scope of this reference manual (though they may be exports provided
by other WebAssembly modules), but which have behavior consistent with their
corresponding concepts defined in this reference manual. They can be accessed
through their respective [module index spaces](#module-index-spaces).

```{req} Global Imports Must Be Immutable
:id: REQ_IMPORT_GLOBALS_IMMUTABLE
:status: open
:tags: validation, import-section
All global variable imports must be immutable, preventing imported global state from being modified by the importing module.
```

```{req} Each Import Must Be Resolvable By Embedding Environment
:id: REQ_IMPORT_RESOLUTION
:status: open
:tags: validation, import-section
Each import declaration must be successfully resolved by the embedding environment to a corresponding external entity at module instantiation time.
```

```{req} Linear Memory Import Minimum Size Constraint
:id: REQ_IMPORT_MEMORY_MIN_SIZE
:status: open
:tags: validation, import-section
A linear-memory import's `minimum` page count must not exceed the `minimum` page count of the imported linear memory instance.
```

```{req} Linear Memory Import Maximum Size Requirement
:id: REQ_IMPORT_MEMORY_MAX_REQUIRED
:status: open
:tags: validation, import-section
If the imported linear memory has a declared `maximum` page count, the importing module's linear-memory import must also declare a `maximum` page count.
```

```{req} Linear Memory Import Maximum Size Constraint
:id: REQ_IMPORT_MEMORY_MAX_SIZE
:status: open
:tags: validation, import-section
If a linear-memory import declares a `maximum` page count, that value must be greater than or equal to the `maximum` page count of the imported linear memory.
```

```{req} Table Import Minimum Size Constraint
:id: REQ_IMPORT_TABLE_MIN_SIZE
:status: open
:tags: validation, import-section
A table import's `minimum` element count must not exceed the `minimum` element count of the imported table instance.
```

```{req} Table Import Maximum Size Requirement
:id: REQ_IMPORT_TABLE_MAX_REQUIRED
:status: open
:tags: validation, import-section
If the imported table has a declared `maximum` element count, the importing module's table import must also declare a `maximum` element count.
```

```{req} Table Import Maximum Size Constraint
:id: REQ_IMPORT_TABLE_MAX_SIZE
:status: open
:tags: validation, import-section
If a table import declares a `maximum` element count, that value must be greater than or equal to the `maximum` element count of the imported table.
```

**Validation:**

> Global imports may be permitted to be mutable in the future.

> `module_name` will often identify a module to import from, and `export_name`
an export in that module to import, but [embedding environments] may provide
other mechanisms for resolving imports as well.

> For example, even though WebAssembly itself does not support overloading of
functions or other entities based on their signature, embedding environments may
provide imports that do so. Or they may ignore any or all of the signature,
`kind`, and `export_name`, and any other fields, and resolve imports
arbitrarily. Two identical imports need not even be resolved to the same entity
within an instantiation.

(function-section)=
#### Function Section

**Opcode:** `0x03`.

The Function Section consists of an [array] of function declarations. Its
elements directly correspond to elements in the [Code Section] array.

A *function declaration* consists of:
 - an index in the [Type Section] of the signature of the function.

**Validation:**
 - The array is required to be the same length as the [Code Section] array.

(table-section)=
#### Table Section

**Opcode:** `0x04`.

The Table Section consists of an [array] of [table descriptions].

(linear-memory-section)=
#### Linear-Memory Section

**Opcode:** `0x05`.

The Memory Section consists of an [array] of [linear-memory descriptions].

```{req} Linear Memory Descriptions Must Be Valid
:id: REQ_MEMORY_SECTION_VALID_DESCRIPTIONS
:status: open
:tags: validation, memory-section
All linear-memory description items in the Linear-Memory Section must be valid according to their respective constraints (size representability, bounds, etc.).
```

> Implementations are encouraged to attempt to reserve enough resources for
allocating up to the `maximum` length up front, if a `maximum` length is
present. Otherwise, implementations are encouraged to allocate only enough for
the `minimum` length up front.

(global-section)=
#### Global Section

**Opcode:** `0x06`.

The Global Section consists of an [array] of global declarations.

A *global declaration* consists of:

| Field Name      | Type                             | Description                              |
| --------------- | -------------------------------- | ---------------------------------------- |
| `desc`          | [global description]             | a description of the global variable     |
| `init`          | [instantiation-time initializer] | the initial value of the global variable |

```{req} Global Initializer Type Must Match Global Description Type
:id: REQ_GLOBAL_SECTION_INIT_TYPE
:status: open
:tags: validation, global-section
The type of the value returned by a global variable's `init` expression must be the same as the type specified in the global's `desc` field.
```

> Exporting of mutable global variables may be permitted in the future.

(export-section)=
#### Export Section

**Opcode:** `0x07`.

The Export Section consists of an [array] of exports.

An *export* consists of:

| Field Name      | Type               | Description                             |
| --------------- | ------------------ | --------------------------------------- |
| `name`          | [identifier]       | field name                              |
| `kind`          | [external kind]    | the kind of export                      |
| `index`         | [varuint32]        | an index into an [index space]          |

If `kind` is `Function`, `index` identifies an element in the
[function index space].

If `kind` is `Table`, `index` identifies an element in the [table index space].

If `kind` is `Memory`, `index` identifies an element in the
[linear-memory index space].

If `kind` is `Global`, `index` identifies an element in the
[global index space].

The meaning of `name` is determined by the [embedding environment].

Exports provide access to an instance's constructs to external entities outside
the scope of this reference manual (though they may be other WebAssembly
modules), but which have behavior consistent with their corresponding concepts
defined in this reference manual.

```{req} Export Names Must Be Unique
:id: REQ_EXPORT_SECTION_NAME_UNIQUENESS
:status: open
:tags: validation, export-section
Each export's name must be unique among all the exports' names within a module, preventing duplicate export identities.
```

```{req} Export Index Must Be Within Associated Index Space Bounds
:id: REQ_EXPORT_SECTION_INDEX_BOUNDS
:status: open
:tags: validation, export-section
Each export's index must refer to a valid element within its associated index space (function, table, memory, or global space).
```

```{req} Global Exports Must Be Immutable
:id: REQ_EXPORT_GLOBALS_IMMUTABLE
:status: open
:tags: validation, export-section
All global variable exports must be immutable, preventing external modification of exported global state.
```

> Because exports reference index spaces which include imports, modules can
re-export their imports.
> The immutability restriction might be lifted in a future version, as part of the [threads proprosal](https://github.com/WebAssembly/threads/blob/master/proposals/threads/Globals.md).

(start-section)=
#### Start Section

**Opcode:** `0x08`.

The Start Section consists of a [varuint32] index into the
[function index space]. This is used by
[Instance Execution](#instance-execution).

```{req} Start Function Index Must Be Within Code Section Bounds
:id: REQ_START_SECTION_INDEX_BOUNDS
:status: open
:tags: validation, start-section
The function index in the Start Section must refer to a valid function within the bounds of the Code Section array.
```

```{req} Start Function Must Have Empty Parameters And Returns
:id: REQ_START_SECTION_SIGNATURE
:status: open
:tags: validation, start-section
The function referenced by the Start Section must have a type signature with an empty parameter list and empty return list.
```

(element-section)=
#### Element Section

**Opcode:** `0x09`.

The Element Section consists of an [array] of table initializers.

A *table initializer* consists of:

| Field Name      | Type                             | Description                                       |
| --------------- | -------------------------------- | ------------------------------------------------- |
| `index`         | [varuint32]                      | identifies a table in the [table index space]     |
| `offset`        | [instantiation-time initializer] | the index of the element in the table to start at |

If the [table]'s `element_type` is `funcref`, the following fields are appended.

| Field Name      | Type                             | Description                                       |
| --------------- | -------------------------------- | ------------------------------------------------- |
| `elems`         | [array] of [varuint32]           | indices into the [function index space]           |

```{req} Element Section Offset Expression Must Return i32
:id: REQ_ELEMENT_SECTION_OFFSET_TYPE
:status: open
:tags: validation, element-section
The `offset` field of each table initializer must be an instantiation-time initializer expression that evaluates to type `i32`.
```

```{req} Element Section Table Index Must Be Within Table Index Space
:id: REQ_ELEMENT_SECTION_TABLE_INDEX_BOUNDS
:status: open
:tags: validation, element-section
Each table initializer's `index` must refer to a valid table within the bounds of the table index space.
```

```{req} Element Section Initialization Must Not Exceed Table Bounds
:id: REQ_ELEMENT_SECTION_NO_OVERFLOW
:status: open
:tags: validation, element-section
For each table initializer, the sum of its `offset` value and the number of elements in its `elems` array must not exceed the `minimum` length declared for the target table.
```

```{req} Element Section Function Indices Must Be Within Function Index Space
:id: REQ_ELEMENT_SECTION_FUNC_INDEX_BOUNDS
:status: open
:tags: validation, element-section
Each element in a table initializer's `elems` array must be a valid index within the bounds of the function index space.
```

> The Element Sections is to the [Table Section] as the [Data Section] is to the
[Linear-Memory Section].

> Table initializers are sometimes called "segments".

(code-section)=
#### Code Section

**Opcode:** `0x0a`.

The Code Section consists of an [array] of function bodies.

A *function body* consists of:

| Field Name      | Type                       | Description                                       |
| --------------- | -------------------------- | ------------------------------------------------- |
| `body_size`     | [varuint32]                | the size of `locals` and `instructions`, in bytes |
| `locals`        | [array] of local entry     | local variable declarations                       |
| `instructions`  | sequence of [instructions] | the instructions                                  |

A *local entry* consists of:

| Field Name      | Type                       | Description                                     |
| --------------- | -------------------------- | ----------------------------------------------- |
| `count`         | [varuint32]                | number of local variables of the following type |
| `type`          | [value type]               | type of the variables                           |

```{req} Control Flow Constructs Must Form Properly Nested Regions
:id: REQ_CODE_SECTION_NESTED_REGIONS
:status: open
:tags: validation, code-section
Control-flow constructs (loop, block, if/else) must form properly nested regions with matching termination instructions (end, else) such that each region has exactly one entry and one or two exits.
```

```{req} Function Body Must Terminate With End Instruction
:id: REQ_CODE_SECTION_FUNCTION_END
:status: open
:tags: validation, code-section
The last instruction in every function body must be an `end` instruction, ensuring proper function termination.
```

```{req} All Instruction-Specific Validation Requirements Must Be Satisfied
:id: REQ_CODE_SECTION_INSTRUCTION_VALIDATION
:status: open
:tags: validation, code-section
For each instruction in a function body, the validation requirements specified in the instruction's description must be satisfied.
```

```{req} Value Stack Must Contain Sufficient Operands At Instruction Execution
:id: REQ_CODE_SECTION_STACK_DEPTH
:status: open
:tags: validation, code-section
For each reachable instruction, the value stack must contain at least as many elements as required by the instruction's signature on every control-flow path leading to that instruction.
```

```{req} Instruction Operand Types Must Conform To Signature
:id: REQ_CODE_SECTION_OPERAND_TYPE_CONFORMANCE
:status: open
:tags: validation, code-section
The types of values passed as operands to each reachable instruction must conform to the instruction's signature on every control-flow path.
```

```{req} Value Stack Types Must Be Consistent Across Control Flow Paths
:id: REQ_CODE_SECTION_TYPE_CONSISTENCY
:status: open
:tags: validation, code-section
The types of values on the value stack must be the same for all control-flow paths that reach any given instruction.
```

```{req} Values Must Be Popped From Same Region Or Nested Region
:id: REQ_CODE_SECTION_REGION_ISOLATION
:status: open
:tags: validation, code-section
All values popped from the value stack at an instruction must have been pushed within the same region or within a region nested inside it, preventing cross-region value leakage.
```

```{req} Unreachable Instructions Must Satisfy Stack Polymorphism Constraints
:id: REQ_CODE_SECTION_UNREACHABLE_POLYMORPHISM
:status: open
:tags: validation, code-section
For unreachable instructions, if fallthrough paths were added to barrier instructions, the instruction must satisfy reachability requirements in at least one such configuration.
```

> These validation requirements are sufficient to ensure that WebAssembly has
*reducible control flow*, which essentially means that all loops have exactly
one entry point.

> There are no implicit type conversions, subtyping, or function overloading in
WebAssembly.

> The constraint on unreachable instructions is sometimes called
"polymorphic type checking", or "stack-polymorphism", however it does not
require any kind of dynamic typing behavior.

(positions-within-a-function-body)=
##### Positions Within A Function Body

A *position* within a function refers to an element of the instruction sequence.

(data-section)=
#### Data Section

**Opcode:** `0x0b`.

The Data Section consists of an [array] of data initializers.

A *data initializer* consists of:

| Field Name      | Type                             | Description                                          |
| --------------- | -------------------------------- | ---------------------------------------------------- |
| `index`         | [varuint32]                      | a [linear-memory index](#linear-memory-index-space)  |
| `offset`        | [instantiation-time initializer] | the index of the byte in memory to start at          |
| `data`          | [byte array]                     | data to initialize the contents of the linear memory |

It describes data to be loaded into the linear memory identified by the index in
the [linear-memory index space] during
[linear-memory instantiation](#linear-memory-instantiation).

```{req} Data Section Offset Expression Must Return i32
:id: REQ_DATA_SECTION_OFFSET_TYPE
:status: open
:tags: validation, data-section
The `offset` field of each data initializer must be an instantiation-time initializer expression that evaluates to type `i32`.
```

```{req} Data Section Memory Index Must Be Within Linear Memory Index Space
:id: REQ_DATA_SECTION_MEMORY_INDEX_BOUNDS
:status: open
:tags: validation, data-section
Each data initializer's linear-memory index must refer to a valid linear memory within the bounds of the linear-memory index space.
```

```{req} Data Section Initialization Must Not Exceed Memory Bounds
:id: REQ_DATA_SECTION_NO_OVERFLOW
:status: open
:tags: validation, data-section
For each data initializer, the sum of its `offset` value and the length of its `data` array must not exceed the `minimum` length (in bytes) declared for the target linear memory.
```

> Data initializers are sometimes called "segments".

(custom-sections)=
### Custom Sections

Custom sections may be used for debugging information or non-standard language
extensions. The contents of the payload of a custom section after its name are
not subject to validation. Some custom sections are described here to promote
interoperability, though they aren't required to be used.

```{req} Custom Sections Support Debugging Information
:id: REQ_CUSTOM_SECTIONS_DEBUG_INFO
:status: open
:tags: feature, custom-sections
Custom sections must be supported for encoding debugging information such as function and variable names to improve module auditability and analysis.
```

```{req} Custom Sections Support Non-Standard Extensions
:id: REQ_CUSTOM_SECTIONS_EXTENSIONS
:status: open
:tags: feature, custom-sections
Custom sections must be supported for encoding non-standard language extensions and tool-specific metadata without modifying the core module semantics.
```

```{req} Custom Section Payload Is Not Subject To Validation
:id: REQ_CUSTOM_SECTIONS_NO_VALIDATION
:status: open
:tags: validation, custom-sections
The contents of a custom section's payload after its name identifier are not subject to specification validation, allowing arbitrary extension-specific encoding.
```

0. [Name Section]

(name-section)=
#### Name Section

**Name:** `name`

TODO: This section currently describes a now-obsolete format. This needs to
be updated to the [new extensible name section format].

[new extensible name section format]: https://github.com/WebAssembly/design/blob/master/BinaryEncoding.md#name-section

The Name Section consists of an [array] of function name descriptors, which
each describe names for the function with the corresponding index in the
[function index space] and which consist of:
 - the function name, an [identifier].
 - the names of the locals in the function, an [array] of [identifiers].

```{req} Name Section Must Be Positioned After Known Sections
:id: REQ_NAME_SECTION_ORDERING
:status: open
:tags: structure, name-section
The Name Section, if present, should be sequenced after any known sections to maintain consistent module structure.
```

```{req} Name Section Malformed Constructs Must Be Ignored
:id: REQ_NAME_SECTION_FAULT_TOLERANCE
:status: open
:tags: validation, name-section
Malformed constructs in the Name Section (such as out-of-bounds indices or incorrect positioning) must not cause validation failures; instead, the section must be ignored to maintain robustness.
```

```{req} Name Section Must Not Change Execution Semantics
:id: REQ_NAME_SECTION_SEMANTICS_NEUTRAL
:status: open
:tags: semantics, name-section
The Name Section must not modify the execution semantics of a WebAssembly module; it provides only debugging information and does not affect runtime behavior.
```

> Name data is represented as an explicit section in WebAssembly, however in
[text form] it may be represented as an integrated part of the syntax for
functions rather than as a discrete section.

> The expectation is that, when a binary WebAssembly module is presented in a
human-readable format in a browser or other development environment, the names
in this section are to be used as the names of functions and locals in
[text form].

(module-index-spaces)=
### Module Index Spaces

*Module Index Spaces* are abstract mappings from indices, starting from zero, to
various types of elements.

```{req} Module Index Spaces Must Map Indices To Typed Elements
:id: REQ_MODULE_INDEX_SPACES_CONCEPT
:status: open
:tags: architecture, module-structure
Module Index Spaces are abstract zero-indexed mappings that organize functions, globals, linear memories, and tables in a unified address space, enabling consistent indexing across imports and definitions.
```

0. [Function Index Space]
0. [Global Index Space]
0. [Linear-Memory Index Space]
0. [Table Index Space]

(function-index-space)=
#### Function Index Space

The *function index space* begins with an index for each imported function, in
the order the imports appear in the [Import Section], if present, followed by an
index for each function in the [Function Section], if present, in the order of
that section.

```{req} Function Index Space Must Contain Imported Functions First
:id: REQ_FUNCTION_INDEX_SPACE_IMPORTS_FIRST
:status: open
:tags: architecture, function-index-space
The function index space must assign indices to imported functions first, in the order they appear in the Import Section.
```

```{req} Function Index Space Must Contain Defined Functions After Imports
:id: REQ_FUNCTION_INDEX_SPACE_DEFINITIONS
:status: open
:tags: architecture, function-index-space
After imported functions, the function index space must assign indices to functions defined in the Function Section, in the order they appear in that section.
```

```{req} Function Index Space Type Index Must Be Valid
:id: REQ_FUNCTION_INDEX_SPACE_TYPE_BOUNDS
:status: open
:tags: validation, function-index-space
For each function in the function index space, its type index must refer to a valid entry within the bounds of the Type Section array.
```

> The function index space is used by [`call`](#call) instructions to identify
the callee of a direct call.

(global-index-space)=
#### Global Index Space

The *global index space* begins with an index for each imported global, in the
order the imports appear in the [Import Section], if present, followed by an
index for each global in the [Global Section], if present, in the order of that
section.

```{req} Global Index Space Must Contain Imported Globals First
:id: REQ_GLOBAL_INDEX_SPACE_IMPORTS_FIRST
:status: open
:tags: architecture, global-index-space
The global index space must assign indices to imported globals first, in the order they appear in the Import Section.
```

```{req} Global Index Space Must Contain Defined Globals After Imports
:id: REQ_GLOBAL_INDEX_SPACE_DEFINITIONS
:status: open
:tags: architecture, global-index-space
After imported globals, the global index space must assign indices to globals defined in the Global Section, in the order they appear in that section.
```

> The global index space is used by:
 - the [`global.get`](#get-global) and [`global.set`](#set-global) instructions.
 - the [Data Section], to define the offset of a data initializer (in a linear
   memory) as the value of a global variable.

(linear-memory-index-space)=
#### Linear-Memory Index Space

The *linear-memory index space* begins with an index for each imported linear
memory, in the order the imports appear in the [Import Section], if present,
followed by an index for each linear memory in the [Linear-Memory Section], if
present, in the order of that section.

```{req} Linear Memory Index Space Must Contain At Most One Memory
:id: REQ_LINEAR_MEMORY_INDEX_SPACE_UNIQUENESS
:status: open
:tags: validation, linear-memory-index-space
The linear-memory index space must contain at most one linear memory, enforcing single-memory modules and preventing memory fragmentation.
```

```{req} Linear Memory Maximum Must Be At Least Minimum
:id: REQ_LINEAR_MEMORY_MAX_MIN_ORDERING
:status: open
:tags: validation, linear-memory-index-space
If a linear memory declares a `maximum` page count, it must be greater than or equal to the declared `minimum` page count.
```

```{req} Linear Memory Byte Indices Must Be representable In varuPTR
:id: REQ_LINEAR_MEMORY_BYTE_ADDRESSABILITY
:status: open
:tags: validation, linear-memory-index-space
If a linear memory declares a `maximum` page count, every byte index in the addressable range (0 to max_bytes-1) must be representable as a valid [varuPTR] value.
```

> The validation rules here specifically avoid requiring the size in bytes of
any linear memory to be representable as a [varuPTR]. For example a 32-bit
linear-memory address space could theoretically be resized to 4 GiB if the
implementation has sufficient resources; the index of every byte would be
addressable, even though the total number of bytes would not be.

> Multiple linear memories may be permitted in the future.

> 64-bit linear memories may be permitted in the future.

(default-linear-memory)=
##### Default Linear Memory

The linear memory with index `0`, if there is one, is called the
*default linear memory*, which is used by several instructions.

```{req} Default Linear Memory At Index Zero Used By Instructions
:id: REQ_DEFAULT_LINEAR_MEMORY_DEFINITION
:status: open
:tags: architecture, linear-memory-index-space
The linear memory with index `0` in the linear-memory index space, if present, is designated as the default linear memory and must be used by memory access instructions.
```

(table-index-space)=
#### Table Index Space

The *table index space* begins with an index for each imported table, in the
order the imports appear in the [Import Section], if present, followed by an
index for each table in the [Table Section], if present, in the order of that
section.

```{req} Table Index Space Must Contain At Most One Table
:id: REQ_TABLE_INDEX_SPACE_UNIQUENESS
:status: open
:tags: validation, table-index-space
The table index space must contain at most one table, enforcing single-table modules and preventing table fragmentation.
```

```{req} Table Maximum Must Be At Least Minimum
:id: REQ_TABLE_MAX_MIN_ORDERING
:status: open
:tags: validation, table-index-space
If a table declares a `maximum` element count, it must be greater than or equal to the declared `minimum` element count.
```

```{req} Table Element Indices Must Be Representable In varuPTR
:id: REQ_TABLE_ELEMENT_ADDRESSABILITY
:status: open
:tags: validation, table-index-space
If a table declares a `maximum` element count, every element index in the addressable range (0 to max_elements-1) must be representable as a valid [varuPTR] value.
```

> The table index space is currently only used by the [Element Section].

(module-types)=
### Module Types

These types describe various data structures present in WebAssembly modules:

0. [Resizable Limits](#resizable-limits)
0. [Linear-Memory Description](#linear-memory-description)
0. [Table Description](#table-description)
0. [Global Description](#global-description)
0. [External Kinds](#external-kinds)
0. [Instantiation-Time Initializers](#instantiation-time-initializers)

> These types aren't used to describe values at execution time.

(resizable-limits)=
#### Resizable Limits

| Field Name | Type         | Description                                            |
| ---------- | ------------ | ------------------------------------------------------ |
| `flags`    | [varuint32]  | bit-packed flags; see below.                           |
| `minimum`  | [varuint32]  | minimum length (in units of table elements or [pages]) |

If bit `0x1` is set in `flags`, the following fields are appended.

| Field Name | Type         | Description                                            |
| ---------- | ------------ | ------------------------------------------------------ |
| `maximum`  | [varuint32]  | maximum length (in same units as `minimum`)            |

```{req} Resizable Limits Maximum Must Not Be Smaller Than Minimum
:id: REQ_RESIZABLE_LIMITS_MAX_MIN
:status: open
:tags: validation, module-types
When a `maximum` value is specified in resizable limits, it must not be smaller than the `minimum` value.
```

(linear-memory-description)=
#### Linear-Memory Description

| Field Name | Type               | Description                                       |
| ---------- | ------------------ | ------------------------------------------------- |
| `limits`   | [resizable limits] | linear-memory flags and sizes in units of [pages] |

```{req} Linear Memory Limits Must Be Valid
:id: REQ_LINEAR_MEMORY_LIMITS_VALID
:status: open
:tags: validation, module-types
The resizable limits item of a linear-memory description must be valid, satisfying all resizable limits constraints.
```

```{req} Linear Memory Maximum Size Is 4 GiB
:id: REQ_LINEAR_MEMORY_MAX_SIZE_LIMIT
:status: open
:tags: validation, module-types
The maximum size of a linear memory must not exceed 65536 pages (4 GiB), ensuring compatibility with 32-bit address spaces.
```

(table-description)=
#### Table Description

| Field Name      | Type                 | Description                                |
| --------------- | -------------------- | ------------------------------------------ |
| `element_type`  | [table element type] | the element type of the [table]            |
| `resizable`     | [resizable limits]   | table flags and sizes in units of elements |

```{req} Table Element Type Must Be funcref
:id: REQ_TABLE_ELEMENT_TYPE_FUNCREF
:status: open
:tags: validation, table-description
The `element_type` field of a table description must be `funcref`, restricting tables to store function references.
```

> The words "size" and "length" are used interchangeably when describing linear
memory, since the elements are byte-sized.

> In the future, other `element_type` values may be permitted.

(global-description)=
#### Global Description

| Field Name      | Type                 | Description                               |
| --------------- | -------------------- | ----------------------------------------- |
| `type`          | [value type]         | the type of the global variable           |
| `mutability`    | [varuint1]           | `0` if immutable, `1` if mutable          |

```{req} Global Variable Type Must Be Specified
:id: REQ_GLOBAL_DESCRIPTION_TYPE
:status: open
:tags: structure, global-description
Each global variable description must specify a value type indicating the type of values the global can hold.
```

```{req} Global Variable Mutability Must Be Specified
:id: REQ_GLOBAL_DESCRIPTION_MUTABILITY
:status: open
:tags: structure, global-description
Each global variable description must explicitly specify mutability as either immutable (0) or mutable (1).
```

(external-kinds)=
#### External Kinds

Externals are entities which can either be defined within a module and
[exported], or [imported] from another module. They are encoded as a [varuint7]
and can be any one of the following values:

| Name       | Binary Encoding |
| ---------- | --------------- |
| `Function` | `0x00`          |
| `Table`    | `0x01`          |
| `Memory`   | `0x02`          |
| `Global`   | `0x03`          |

```{req} External Kind Must Be Valid Value
:id: REQ_EXTERNAL_KIND_VALID
:status: open
:tags: validation, external-kinds
The external kind value must be one of: Function (0x00), Table (0x01), Memory (0x02), or Global (0x03).
```

```{req} External Kind Determines Import/Export Semantics
:id: REQ_EXTERNAL_KIND_SEMANTICS
:status: open
:tags: architecture, external-kinds
The external kind value determines the type of entity being imported or exported and must be consistently interpreted by validation logic.
```

(instantiation-time-initializers)=
#### Instantiation-Time Initializers

An *instantiation-time initializer* is a single [instruction], which is one of
the following:
 - [`const`](#constant) (of any type).
 - [`global.get`](#get-global).

The value produced by a module initializer is the value that such an instruction
would produce if it were executed within a function body.

```{req} Instantiation-Time Initializers Must Be Const Or Global.get Instructions
:id: REQ_INIT_INSTRUCTION_TYPES
:status: open
:tags: validation, instantiation-time-initializers
An instantiation-time initializer must consist of either a `const` instruction (of any type) or a `global.get` instruction, and no other instructions.
```

```{req} Initializer Instruction Validation Must Be Satisfied
:id: REQ_INIT_INSTRUCTION_VALIDATION
:status: open
:tags: validation, instantiation-time-initializers
The specific validation requirements of the instruction used in an instantiation-time initializer must be satisfied.
```

```{req} Global.get In Initializer Must Reference Immutable Import
:id: REQ_INIT_GLOBAL_GET_IMMUTABLE_IMPORT
:status: open
:tags: validation, instantiation-time-initializers
When a `global.get` instruction is used in an instantiation-time initializer, the indexed global must be an immutable import, preventing circular initialization dependencies.
```

> In the future, more instructions may be permitted as instantiation-time
initializers.

> Instantiation-time initializers are sometimes called "constant expressions".

(instruction-descriptions)=
Instruction Descriptions
--------------------------------------------------------------------------------

Instructions in the [Instructions](#instructions) section are introduced with
tables giving a concise description of several of their attributes, followed by
additional content.

[Instructions](#instructions) are encoded as their Opcode value followed by
their immediate operand values.

0. [Instruction Mnemonic Field](#instruction-mnemonic-field)
0. [Instruction Opcode Field](#instruction-opcode-field)
0. [Instruction Immediates Field](#instruction-immediates-field)
0. [Instruction Signature Field](#instruction-signature-field)
0. [Instruction Families Field](#instruction-families-field)
0. [Instruction Description](#instruction-description)

(instruction-mnemonic-field)=
### Instruction Mnemonic Field

Instruction [mnemonics] are short names identifying specific instructions.

Many instructions have type-specific behavior, in which case there is a unique
mnemonic for each type, formed by prepending a *type prefix*, such as `i32.` or
`f64.`, to the base instruction mnemonic.

Conversion instructions have additional type-specific behavior; their mnemonic
additionally has a *type suffix* appended, such as `/i32` or `/f64`, indicating
the input type.

The base mnemonics for [signed][S] and [unsigned][U] instructions have the
convention of ending in "_s" and "_u" respectively.

[mnemonics]: https://en.wikipedia.org/wiki/Assembly_language#Opcode_mnemonics_and_extended_mnemonics

(instruction-opcode-field)=
### Instruction Opcode Field

These values are used in WebAssembly the to encode instruction [opcodes].

The opcodes for [signed][S] and [unsigned][U] instructions have the convention
that the unsigned opcode is always one greater than the signed opcode.

[opcodes]: https://en.wikipedia.org/wiki/Opcode

(instruction-immediates-field)=
### Instruction Immediates Field

Immediates, if present, is a list of value names with associated
[encoding types], representing values provided by the module itself as input to
an instruction.

(instruction-signature-field)=
### Instruction Signature Field

Instruction signatures describe the explicit inputs and outputs to an
instruction. They are described in the following form:

`(` *operands* `)` `:` `(` *returns* `)`

*Operands* describes a list of [types] for values provided by program execution
as input to an instruction. *Returns* describes a list of [types] for values
computed by the instruction that are provided back to the program execution.

Within the signature for a [linear-memory access instruction][M], `iPTR` refers
an [integer  type](#integer-value-types) with the index bitwidth of the accessed
linear memory.

Besides literal [types], descriptions of [types] can be from the following
mechanisms:
 - A [typed] value name of the form

   *name*`:` *type*

   where *name* just provides an identifier for use in
   [instruction descriptions](#instruction-description). It is replaced by
   *type*.
 - A type parameter list of the form

   *name*`[` *length* `]`

   where *name* identifies the list, and *length* gives the length of the list.
   The length may be a literal value, an immediate operand value, or one of the
   named values defined below. Each type parameter in the list may be *bound* to
   a type as described in the instruction's description, or it may be inferred
   from the type of a corresponding operand value. The parameter list is
   replaced by the types bound to its parameters. If the list appears multiple
   times in a signature, it is replaced by the same types at each appearance.

(named-values)=
#### Named Values

The following named values are defined:
 - `$args` is defined in [call instructions][L] and indicates the length of the
   callee signature parameter list.
 - `$returns` is also defined in [call instructions][L] and indicates the length
   of callee signature return list.
 - `$any` indicates the number of values on the value stack pushed within the
   enclosing region.
 - `$block_arity` is defined in [branch instructions][B] and indicates the
   number of values types in the target control-flow stack entry's signature.

(instruction-families-field)=
### Instruction Families Field

WebAssembly instructions may belong to several families, indicated in the tables
by their family letter:

0. [B: Branch Instruction Family][B]
0. [Q: Control-Flow Barrier Instruction Family][Q]
0. [L: Call Instruction Family][L]
0. [G: Generic Integer Instruction Family][G]
0. [S: Signed Integer Instruction Family][S]
0. [U: Unsigned Integer Instruction Family][U]
0. [T: Shift Instruction Family][T]
0. [R: Remainder Instruction Family][R]
0. [F: Floating-Point Instruction Family][F]
0. [E: Floating-Point Bitwise Instruction Family][E]
0. [C: Comparison Instruction Family][C]
0. [M: Linear-Memory Access Instruction Family][M]
0. [Z: Linear-Memory Size Instruction Family][Z]

(b-branch-instruction-family)=
#### B: Branch Instruction Family

(branching)=
##### Branching

In a branch according to a given control-flow stack entry, first the value stack
is resized down to the entry's limit value.

Then, if the entry's [label] is bound, the current position is set to the bound
position. Otherwise, the position to bind the label to is found by scanning
forward through the instructions, as if executing just [`block`](#block),
[`loop`](#loop), and [`end`](#end) instructions, until the label is bound. Then
the current position is set to that position.

Then, control-flow stack entries are popped until the given control-flow stack
entry is popped.

> In practice, implementations may precompute the destinations of branches so
that they don't literally need to scan in this manner.

> Branching is sometimes called "jumping" in other languages.

> Branch instructions can only target [labels] within the same function.

(branch-index-validation)=
##### Branch Index Validation

A depth index is a valid branch index if it is less than the length of the
control-flow stack at the branch instruction.

```{req} Branch Depth Index Must Be Less Than Control Flow Stack Length
:id: REQ_BRANCH_DEPTH_VALIDATION
:status: open
:tags: validation, branch-instructions
A depth index used in a branch instruction must be less than the length of the control-flow stack at the point of the branch instruction.
```

(q-control-flow-barrier-instruction-family)=
#### Q: Control-Flow Barrier Instruction Family

These instructions either trap or reassign the current position, such that
execution doesn't proceed (or "fall through") to the instruction that lexically
follows them.

```{req} Barrier Instructions Must Not Fall Through
:id: REQ_BARRIER_NO_FALLTHROUGH
:status: open
:tags: semantics, barrier-instructions
Control-flow barrier instructions (e.g., `return`, `unreachable`, `call` in some contexts) must either trap or reassign the current position to prevent fall-through execution to the lexically following instruction.
```

(l-call-instruction-family)=
#### L: Call Instruction Family

(calling)=
##### Calling

If the called function&mdash;the *callee*&mdash;is a function in the module, it
is [executed](#function-execution). Otherwise the callee is an imported function
which is executed according to its own semantics. The [`$args`] operands,
excluding `$callee` when present, are passed as the incoming arguments. The
return value of the call is defined by the execution.

At least one unit of [call-stack resources] is consumed during the execution of
the callee, and released when it completes.

```{req} Call Instruction Must Consume Call Stack Resources
:id: REQ_CALL_STACK_RESOURCE_CONSUMPTION
:status: open
:tags: semantics, call-instructions
Each call instruction must consume at least one unit of call-stack resources during the execution of the callee, and release it when the callee completes.
```

```{req} Call Instruction Must Trap If Stack Exhausted
:id: REQ_CALL_STACK_EXHAUSTION_TRAP
:status: open
:tags: validation, call-instructions
A call instruction must trap with a "Call Stack Exhausted" trap if the instance has insufficient call-stack resources to execute the callee.
```

```{req} Call Instruction Must Propagate Callee Traps
:id: REQ_CALL_TRAP_PROPAGATION
:status: open
:tags: semantics, call-instructions
If a trap occurs during the execution of the callee, the call instruction must propagate that trap (Callee Trap) to the caller.
```

```{req} Call Instructions Must Not Perform Implicit Tail Call Optimization
:id: REQ_CALL_NO_IMPLICIT_TCO
:status: open
:tags: semantics, call-instructions
Implementations are not permitted to perform implicit opportunistic tail-call optimization (TCO) on call instructions; call stacks must be preserved for each call.
```

**Trap:** Call Stack Exhausted, if the instance has insufficient
[call-stack resources].

**Trap:** Callee Trap, if a trap occurred during the execution of the callee.

> The execution state of the function currently being executed remains live
during the call, and the execution of the called function is performed
independently. In this way, calls form a stack-like data structure called the
*call stack*.

> Data associated with the call stack is stored outside any linear-memory
address space and isn't directly accessible to applications.

(call-validation)=
##### Call Validation

 - The members of `$T[`[`$args`]`]` are bound to the operand types of the callee
   signature, and the members of `$T[`[`$returns`]`]` are bound to the return
   types of the callee signature.

(g-generic-integer-instruction-family)=
#### G: Generic Integer Instruction Family

Except where otherwise specified, these instructions don't specifically
interpret their operands as explicitly signed or unsigned, and therefore don't
have an inherent concept of overflow.

```{req} Generic Integer Instructions Must Not Interpret As Signed Or Unsigned
:id: REQ_GENERIC_INTEGER_NO_INTERPRETATION
:status: open
:tags: semantics, integer-instructions
Generic integer instructions must not interpret operands or results as signed or unsigned, and must not have an inherent overflow concept unless explicitly specified.
```

(s-signed-integer-instruction-family)=
#### S: Signed Integer Instruction Family

Except where otherwise specified, these instructions interpret their operand
values as signed, return result values interpreted as signed, and [trap] when
the result value can't be represented as such.

```{req} Signed Integer Instructions Must Interpret Operands As Signed
:id: REQ_SIGNED_INTEGER_INTERPRETATION
:status: open
:tags: semantics, integer-instructions
Signed integer instructions must interpret operand values as two's complement signed integers and return result values in the same representation.
```

```{req} Signed Integer Overflow Must Trap
:id: REQ_SIGNED_INTEGER_OVERFLOW_TRAP
:status: open
:tags: validation, integer-instructions
A signed integer instruction must trap if the result value cannot be represented as a signed integer, preventing overflow.
```

(u-unsigned-integer-instruction-family)=
#### U: Unsigned Integer Instruction Family

Except where otherwise specified, these instructions interpret their operand
values as unsigned, return result values interpreted as unsigned, and [trap]
when the result value can't be represented as such.

```{req} Unsigned Integer Instructions Must Interpret Operands As Unsigned
:id: REQ_UNSIGNED_INTEGER_INTERPRETATION
:status: open
:tags: semantics, integer-instructions
Unsigned integer instructions must interpret operand values as unsigned integers and return result values in the same representation.
```

```{req} Unsigned Integer Overflow Must Trap
:id: REQ_UNSIGNED_INTEGER_OVERFLOW_TRAP
:status: open
:tags: validation, integer-instructions
An unsigned integer instruction must trap if the result value cannot be represented as an unsigned integer, preventing overflow.
```

(t-shift-instruction-family)=
#### T: Shift Instruction Family

In the shift and rotate instructions, *left* means in the direction of greater
significance, and *right* means in the direction of lesser significance.

(shift-count)=
##### Shift Count

The second operand in shift and rotate instructions specifies a *shift count*,
which is interpreted as an unsigned quantity modulo the number of bits in the
first operand.

```{req} Shift Count Must Be Interpreted As Unsigned Modulo
:id: REQ_SHIFT_COUNT_MODULO
:status: open
:tags: semantics, shift-instructions
The shift count (second operand) in shift and rotate instructions must be interpreted as an unsigned quantity modulo the bit-width of the first operand (5 bits for i32, 6 bits for i64).
```

```{req} Shift Count Is Unsigned Even In Signed Operations
:id: REQ_SHIFT_COUNT_UNSIGNED
:status: open
:tags: semantics, shift-instructions
The shift count must be interpreted as unsigned even in signed shift instructions like `shr_s`, ensuring consistent behavior across instruction families.
```

> As a result of the modulo, in `i32.` instructions, only the least-significant
5 bits of the second operand affect the result, and in `i64.` instructions only
the least-significant 6 bits of the second operand affect the result.

(r-remainder-instruction-family)=
#### R: Remainder Instruction Family

> The remainder instructions (`%`) are related to their corresponding division
instructions (`/`) by the identity `x == (x/y)*y + (x%y)`.

(f-floating-point-instruction-family)=
#### F: Floating-Point Instruction Family

Instructions in this family follow the [IEEE 754-2008] standard, except that:

 - They support only "non-stop" mode, and floating-point exceptions aren't
   otherwise observable. In particular, neither alternate floating-point
   exception handling attributes nor the non-computational operations on status
   flags are supported.

 - They use the IEEE 754-2008 `roundTiesToEven` rounding attribute, except where
   otherwise specified. Non-default directed rounding attributes aren't
   supported.

 - Extended and extendable precision formats aren't supported. All computations
   must be strictly and correctly rounded after each instruction.

```{req} Floating Point Instructions Must Follow IEEE 754-2008 Standard
:id: REQ_FLOAT_IEEE754_COMPLIANCE
:status: open
:tags: semantics, floating-point-instructions
Floating-point instructions must conform to the IEEE 754-2008 standard with specified deviations for rounding, exception handling, and precision.
```

```{req} Floating Point Operations Must Support Non-Stop Mode Only
:id: REQ_FLOAT_NONSTOP_MODE
:status: open
:tags: semantics, floating-point-instructions
Floating-point instructions must support only IEEE 754-2008 "non-stop" mode, where floating-point exceptions are not observable and alternate exception handling is not supported.
```

```{req} Floating Point Rounding Must Use RoundTiesToEven By Default
:id: REQ_FLOAT_ROUNDTIESTOVEN
:status: open
:tags: semantics, floating-point-instructions
Floating-point instructions must use the IEEE 754-2008 `roundTiesToEven` rounding attribute by default, with no support for directed rounding modes.
```

```{req} Floating Point Computations Must Be Strictly Rounded After Each Instruction
:id: REQ_FLOAT_STRICT_ROUNDING
:status: open
:tags: semantics, floating-point-instructions
All floating-point computations must be strictly and correctly rounded after each instruction, preventing intermediate rounding adjustments or operation fusion that would elide rounding steps.
```

```{req} Floating Point Instructions Must Produce Deterministic Numeric Results
:id: REQ_FLOAT_DETERMINISTIC_RESULTS
:status: open
:tags: semantics, floating-point-instructions
All numeric floating-point results must be deterministic, including the rules for how NaNs are handled as operands and when NaNs are generated as results, with only NaN bit-patterns being nondeterministic.
```

> The exception and rounding behavior specified here are the default behavior on
most contemporary software environments.

> All computations are correctly rounded, subnormal values are fully supported,
and negative zero, NaNs, and infinities are all produced as result values to
indicate overflow, invalid, and divide-by-zero exceptional conditions, and
interpreted appropriately when they appear as operands. Compiler optimizations
that introduce changes to the effective precision, rounding, or range of any
computation are not permitted. Implementations are not permitted to contract or
fuse operations to elide intermediate rounding steps. All numeric results are
deterministic, as are the rules for how NaNs are handled as operands and for
when NaNs are to be generated as results. The only floating-point nondeterminism
is in the specific bit-patterns of NaN result values.

> In IEEE 754-1985, ["subnormal numbers"] are called "denormal numbers";
WebAssembly follows IEEE 754-2008, which calls them "subnormal numbers".

When the result of any instruction in this family (which excludes `neg`, `abs`,
`copysign`, `load`, `store`, and `const`) is a NaN, the sign bit and the
significand field (which doesn't include the implicit leading digit of the
significand) of the NaN are computed as follows:

 - If the instruction has any NaN non-immediate operand values with significand
   fields that have any bits set to `1` other than the most significant bit of
   the significand field, the result is a NaN with a nondeterministic sign bit,
   `1` in the most significant digit of the significand, and nondeterministic
   values in the remaining bits of the significand field.

 - Otherwise, the result is a NaN with a nondeterministic sign bit, `1` in the
   most significant digit of the significand, and `0` in the remaining bits of
   the significand field.

Implementations are permitted to further implement the IEEE 754-2008 section
"Operations with NaNs" recommendation that operations propagate NaN bits from
their operands, however it isn't required.

> The NaN propagation rules are intended to support NaN-boxing. If all inputs
to an arithmetic operator are "canonical", the result is also "canonical", so
NaN-boxing implementations don't need to worry about non-"canonical" NaNs
being generated as a result of arithmetic.

> At present, there is no observable difference between quiet and signaling NaN
other than the difference in the bit pattern.

> IEEE 754-2008 is the current revision of IEEE 754; a new revision is expected
to be released some time in 2018, and it's expected to be a minor and
backwards-compatible revision, so WebAssembly is expected to update to it.

[IEEE 754-2008]: https://en.wikipedia.org/wiki/IEEE_floating_point
["subnormal numbers"]: https://en.wikipedia.org/wiki/Subnormal_number

(e-floating-point-bitwise-instruction-family)=
#### E: Floating-Point Bitwise Instruction Family

These instructions operate on floating-point values, but do so in purely bitwise
ways, including in how they operate on NaN and zero values.

They correspond to the "Sign bit operations" in IEEE 754-2008.

(c-comparison-instruction-family)=
#### C: Comparison Instruction Family

WebAssembly comparison instructions compare two values and return a [boolean]
result value.

> In accordance with IEEE 754-2008, for the comparison instructions, negative
zero is considered equal to zero, and NaN values aren't less than, greater than,
or equal to any other values, including themselves.

(m-linear-memory-access-instruction-family)=
#### M: Linear-Memory Access Instruction Family

These instructions load from and store to a linear memory.

```{req} Memory Access Instructions Must Access Linear Memory
:id: REQ_MEMORY_ACCESS_LINEAR_MEMORY
:status: open
:tags: semantics, memory-access-instructions
Memory access instructions (load, store) must operate on the linear memory associated with the module, respecting memory bounds and access semantics.
```

(effective-address)=
##### Effective Address

The *effective address* of a linear-memory access is computed by adding `$base`
and `$offset`, both interpreted as unsigned, at infinite range and precision, so
that there is no overflow.

```{req} Effective Address Must Be Computed At Infinite Precision
:id: REQ_EFFECTIVE_ADDRESS_INFINITE_PRECISION
:status: open
:tags: semantics, memory-access-instructions
The effective address of a memory access must be computed by adding the base address and offset operands as unsigned values at infinite range and precision, preventing overflow during address calculation.
```

(alignment)=
##### Alignment

**Slow:** If the effective address isn't a multiple of `$align`, the access is
*misaligned*, and the instruction may execute very slowly.

```{req} Misaligned Memory Access May Execute Slowly
:id: REQ_MEMORY_MISALIGNMENT_SLOWNESS
:status: open
:tags: semantics, memory-access-instructions
If the effective address is not a multiple of the declared alignment, the memory access is misaligned and may execute very slowly on some implementations.
```

```{req} Alignment Does Not Affect Memory Access Semantics
:id: REQ_ALIGNMENT_NO_SEMANTIC_EFFECT
:status: open
:tags: semantics, memory-access-instructions
The alignment attribute has no semantic effect on memory access behavior; both naturally aligned and misaligned accesses must behave correctly and produce identical results.
```

> When `$align` is at least the size of the access, the access is
*naturally aligned*. When it's less, the access is *unaligned*. Naturally
aligned accesses may be faster than unaligned accesses, though both may be much
faster than misaligned accesses.

(accessed-bytes)=
##### Accessed Bytes

The *accessed bytes* consist of a contiguous sequence of [bytes] starting at the
[effective address], with a size implied by the accessing instruction.

```{req} Accessed Bytes Must Be Contiguous Sequence
:id: REQ_ACCESSED_BYTES_CONTIGUOUS
:status: open
:tags: semantics, memory-access-instructions
The accessed bytes for a memory operation must form a contiguous sequence of bytes starting at the effective address.
```

**Trap:** Out Of Bounds, if any of the accessed bytes are beyond the end of the
accessed linear memory. This trap is triggered before any of the bytes are
actually accessed.

```{req} Out Of Bounds Access Must Trap Before Access
:id: REQ_OUT_OF_BOUNDS_TRAP
:status: open
:tags: validation, memory-access-instructions
If any accessed byte is beyond the end of linear memory, an "Out Of Bounds" trap must be triggered before any bytes are actually accessed, preventing partial memory access.
```

> Linear-memory accesses trap on an out-of-bound access, which differs from
[TypedArrays in ECMAScript] where storing out of bounds silently does nothing
and loading out of bounds silently returns `undefined`.

[TypedArrays in ECMAScript]: https://tc39.github.io/ecma262/#sec-typedarray-objects

(loading)=
##### Loading

For a load access, a value is read from the [accessed bytes], in
[little-endian byte order], and returned.

```{req} Load Must Use Little-Endian Byte Order
:id: REQ_LOAD_LITTLE_ENDIAN
:status: open
:tags: semantics, memory-access-instructions
Load operations must read values from the accessed bytes in little-endian byte order.
```

(storing)=
##### Storing

For a store access, the value to store is written to the [accessed bytes], in
[little-endian byte order].

```{req} Store Must Use Little-Endian Byte Order
:id: REQ_STORE_LITTLE_ENDIAN
:status: open
:tags: semantics, memory-access-instructions
Store operations must write values to the accessed bytes in little-endian byte order.
```

```{req} Store Out Of Bounds Must Trap Before Writing
:id: REQ_STORE_OOB_TRAP_FIRST
:status: open
:tags: validation, memory-access-instructions
If any store access bytes are out of bounds, the Out Of Bounds trap must be triggered before any bytes are written to memory.
```

> If any of the bytes are out of bounds, the Out Of Bounds trap is triggered
before any of the bytes are written to.

(linear-memory-access-validation)=
##### Linear-Memory Access Validation

```{req} Alignment Must Not Exceed Access Size
:id: REQ_MEMORY_ALIGNMENT_NATURAL
:status: open
:tags: validation, memory-access-instructions
The alignment value must not exceed the number of accessed bytes, enforcing natural alignment constraints.
```

```{req} Default Linear Memory Required For Access Instructions
:id: REQ_MEMORY_DEFAULT_REQUIRED
:status: open
:tags: validation, memory-access-instructions
A module must contain a default linear memory (index 0) for memory access instructions to be valid.
```

 - `$align` is required to be at most the number of [accessed bytes] (the
   [*natural alignment*](#alignment)).
 - The module is required to contain a default linear memory.

(z-linear-memory-size-instruction-family)=
#### Z: Linear-Memory Size Instruction Family

(linear-memory-size-validation)=
##### Linear-Memory Size Validation

```{req} Default Linear Memory Required For Size Instructions
:id: REQ_MEMORY_SIZE_DEFAULT_REQUIRED
:status: open
:tags: validation, memory-size-instructions
Memory size instructions (memory.size) require that the module contains a default linear memory (index 0).
```

 - The module is required to contain a default linear memory.

(instruction-description)=
### Instruction Description

Instruction semantics are described for use in the context of
[function-body execution](#function-body-execution). Some instructions also have
a special validation clause, introduced by "**Validation:**", which defines
instruction-specific validation requirements.

(instructions)=
Instructions
--------------------------------------------------------------------------------

0. [Control Flow Instructions](#control-flow-instructions)
0. [Basic Instructions](#basic-instructions)
0. [Integer Arithmetic Instructions](#integer-arithmetic-instructions)
0. [Floating-Point Arithmetic Instructions](#floating-point-arithmetic-instructions)
0. [Integer Comparison Instructions](#integer-comparison-instructions)
0. [Floating-Point Comparison Instructions](#floating-point-comparison-instructions)
0. [Conversion Instructions](#conversion-instructions)
0. [Load And Store Instructions](#load-and-store-instructions)
0. [Additional Memory-Related Instructions](#additional-memory-related-instructions)

(control-flow-instructions)=
### Control Flow Instructions

0. [Block](#block)
0. [Loop](#loop)
0. [Unconditional Branch](#unconditional-branch)
0. [Conditional Branch](#conditional-branch)
0. [Table Branch](#table-branch)
0. [If](#if)
0. [Else](#else)
0. [End](#end)
0. [Return](#return)
0. [Unreachable](#unreachable)

(block)=
#### Block

| Mnemonic    | Opcode | Immediates                           | Signature | Families |
| ----------- | ------ | ------------------------------------ | --------- | -------- |
| `block`     | 0x02   | `$signature`: [block signature type] | `() : ()` |          |

The `block` instruction pushes an entry onto the control-flow stack. The entry
contains an unbound [label], the current length of the value stack, and
`$signature`.

```{req} Block Instruction Must Push Control Flow Stack Entry
:id: REQ_BLOCK_PUSH_ENTRY
:status: open
:tags: semantics, control-flow-instructions
The `block` instruction must push an entry onto the control-flow stack containing an unbound label, the current value stack length, and the block's signature type.
```

```{req} Block Must Have Matching End Instruction
:id: REQ_BLOCK_REQUIRES_END
:status: open
:tags: validation, control-flow-instructions
Each `block` instruction must be properly terminated with a corresponding [`end`](#end) instruction that binds the label and pops the control-flow stack entry.
```

> Each `block` needs a corresponding [`end`](#end) to bind its label and pop
its control-flow stack entry.

(loop)=
#### Loop

| Mnemonic    | Opcode | Immediates                           | Signature | Families |
| ----------- | ------ | ------------------------------------ | --------- | -------- |
| `loop`      | 0x03   | `$signature`: [block signature type] | `() : ()` |          |

The `loop` instruction binds a [label] to the current position, and pushes an
entry onto the control-flow stack. The entry contains that label, the current
length of the value stack, and `$signature`.

```{req} Loop Instruction Must Bind Label To Current Position
:id: REQ_LOOP_BIND_LABEL
:status: open
:tags: semantics, control-flow-instructions
The `loop` instruction must bind a label to the current instruction position, enabling branches to re-enter the loop.
```

```{req} Loop Instruction Must Push Control Flow Stack Entry
:id: REQ_LOOP_PUSH_ENTRY
:status: open
:tags: semantics, control-flow-instructions
The `loop` instruction must push an entry onto the control-flow stack containing the bound label, the current value stack length, and the loop's signature type.
```

```{req} Loop Must Have Matching End Instruction
:id: REQ_LOOP_REQUIRES_END
:status: open
:tags: validation, control-flow-instructions
Each `loop` instruction must be properly terminated with a corresponding [`end`](#end) instruction that pops the control-flow stack entry.
```

> The `loop` instruction doesn't perform a loop by itself. It merely introduces
a label that may be used by a branch to form an actual loop.

> Since `loop`'s control-flow stack entry starts with an empty type sequence,
branches to the top of the loop must not have any result values.

> Each `loop` needs a corresponding [`end`](#end) to pop its control-flow stack
entry.

> There is no requirement that loops eventually terminate or contain observable
side effects.

(unconditional-branch)=
#### Unconditional Branch

| Mnemonic    | Opcode | Immediates            | Signature                                             | Families |
| ----------- | ------ | --------------------- | ----------------------------------------------------- | -------- |
| `br`        | 0x0c   | `$depth`: [varuint32] | `($T[`[`$block_arity`]`]) : ($T[`[`$block_arity`]`])` | [B] [Q]  |

The `br` instruction [branches](#branching) according to the control-flow stack
entry `$depth` from the top. It returns the values of its operands.

```{req} Unconditional Branch Must Execute Branch Operation
:id: REQ_BR_EXECUTES_BRANCH
:status: open
:tags: semantics, branch-instructions
The `br` instruction must execute a branch operation according to the control-flow stack entry at depth `$depth`, unconditionally transferring control flow.
```

```{req} Unconditional Branch Must Return Operand Values
:id: REQ_BR_RETURNS_OPERANDS
:status: open
:tags: semantics, branch-instructions
The `br` instruction must return the values of its operands to the target block's value stack after branching.
```

**Validation:**
 - $depth is required to be a [valid branch index](#branch-index-validation).

```{req} Unconditional Branch Depth Must Be Valid Index
:id: REQ_BR_VALID_DEPTH
:status: open
:tags: validation, branch-instructions
The `$depth` operand of a `br` instruction must be a valid branch index within the control-flow stack depth at the branch point.
```

TODO: Explicitly describe the binding of $T.

(conditional-branch)=
#### Conditional Branch

| Mnemonic    | Opcode | Immediates            | Signature                                                              | Families |
| ----------- | ------ | --------------------- | ---------------------------------------------------------------------- | -------- |
| `br_if`     | 0x0d   | `$depth`: [varuint32] | `($T[`[`$block_arity`]`], $condition: i32) : ($T[`[`$block_arity`]`])` | [B]      |

If `$condition` is [true], the `br_if` instruction [branches](#branching)
according to the control-flow stack entry `$depth` from the top. Otherwise, it
does nothing (and control "falls through"). It returns the values of its
operands, except `$condition`.

```{req} Conditional Branch Must Test Condition Value
:id: REQ_BR_IF_TEST_CONDITION
:status: open
:tags: semantics, branch-instructions
The `br_if` instruction must evaluate the `$condition` operand (as i32), interpreting non-zero as true.
```

```{req} Conditional Branch Must Execute Conditionally
:id: REQ_BR_IF_CONDITIONAL_EXECUTION
:status: open
:tags: semantics, branch-instructions
If `$condition` is true, `br_if` must execute a branch to the target block; otherwise, it must fall through to the next instruction without branching.
```

```{req} Conditional Branch Must Return Non-Condition Operands
:id: REQ_BR_IF_RETURNS_OPERANDS
:status: open
:tags: semantics, branch-instructions
The `br_if` instruction must return the values of its operands to the target block, excluding the `$condition` operand.
```

**Validation:**
 - $depth is required to be a [valid branch index](#branch-index-validation).

```{req} Conditional Branch Depth Must Be Valid Index
:id: REQ_BR_IF_VALID_DEPTH
:status: open
:tags: validation, branch-instructions
The `$depth` operand of a `br_if` instruction must be a valid branch index within the control-flow stack depth at the branch point.
```

TODO: Explicitly describe the binding of $T.

(table-branch)=
#### Table Branch

| Mnemonic    | Opcode | Immediates                                                | Signature                                                          | Families |
| ----------- | ------ | --------------------------------------------------------- | ------------------------------------------------------------------ | -------- |
| `br_table`  | 0x0e   | `$table`: [array] of [varuint32], `$default`: [varuint32] | `($T[`[`$block_arity`]`], $index: i32) : ($T[`[`$block_arity`]`])` | [B] [Q]  |

First, the `br_table` instruction selects a depth to use. If `$index` is within
the bounds of `$table`, the depth is the value of the indexed `$table` element.
Otherwise, the depth is `$default`.

Then, it [branches](#branching) according to the control-flow stack entry that
depth from the top. It returns the values of its operands, except `$index`.

```{req} Table Branch Must Select Depth Based On Index
:id: REQ_BR_TABLE_SELECT_DEPTH
:status: open
:tags: semantics, branch-instructions
The `br_table` instruction must select a branch depth by checking if `$index` is within the table bounds; if in bounds, use the table element; otherwise use `$default`.
```

```{req} Table Branch Must Execute Selected Branch
:id: REQ_BR_TABLE_EXECUTES_BRANCH
:status: open
:tags: semantics, branch-instructions
After selecting the appropriate depth, `br_table` must execute a branch operation according to the control-flow stack entry at that depth.
```

```{req} Table Branch Must Return Non-Index Operands
:id: REQ_BR_TABLE_RETURNS_OPERANDS
:status: open
:tags: semantics, branch-instructions
The `br_table` instruction must return the values of its operands to the target block, excluding the `$index` operand.
```

**Validation:**
 - All entries of the table, and `$default`, are required to be
   [valid branch indices](#branch-index-validation).

```{req} Table Branch All Indices Must Be Valid
:id: REQ_BR_TABLE_VALID_INDICES
:status: open
:tags: validation, branch-instructions
All entries in the `$table` array and the `$default` depth must be valid branch indices within the control-flow stack depth.
```

> This instruction serves the role of what is sometimes called a ["jump table"]
in other languages. "Branch" is used here instead to emphasize the commonality
with the other branch instructions.

> The `$default` label isn't considered to be part of the branch table.

["jump table"]: https://en.wikipedia.org/w/index.php?title=Jump_table

TODO: Explicitly describe the binding of $T.

(if)=
#### If

| Mnemonic    | Opcode | Immediates                           | Signature                   | Families |
| ----------- | ------ | ------------------------------------ | --------------------------- | -------- |
| `if`        | 0x04   | `$signature`: [block signature type] | `($condition: i32) : ()`    | [B]      |

The `if` instruction pushes an entry onto the control-flow stack. The entry
contains an unbound [label], the current length of the value stack, and
`$signature`. If `$condition` is [false], it then [branches](#branching)
according to this entry.

```{req} If Instruction Must Push Control Flow Stack Entry
:id: REQ_IF_PUSH_ENTRY
:status: open
:tags: semantics, control-flow-instructions
The `if` instruction must push an entry onto the control-flow stack containing an unbound label, the current value stack length, and the if's signature type.
```

```{req} If Instruction Must Test Condition Value
:id: REQ_IF_TEST_CONDITION
:status: open
:tags: semantics, control-flow-instructions
The `if` instruction must evaluate the `$condition` operand (as i32), interpreting zero as false and non-zero as true.
```

```{req} If Instruction Must Branch On False Condition
:id: REQ_IF_BRANCH_ON_FALSE
:status: open
:tags: semantics, control-flow-instructions
If `$condition` is false, the `if` instruction must execute a branch according to the pushed control-flow stack entry; otherwise it continues to the next instruction.
```

```{req} If Must Have Matching Else Or End
:id: REQ_IF_REQUIRES_ELSE_OR_END
:status: open
:tags: validation, control-flow-instructions
Each `if` instruction must be properly terminated with either a corresponding [`else`](#else) or [`end`](#end) instruction that binds the label and pops the control-flow stack entry.
```

> Each `if` needs either a corresponding [`else`](#else) or [`end`](#end) to
bind its label and pop its control-flow stack entry.

(else)=
#### Else

| Mnemonic    | Opcode | Signature                             | Families |
| ----------- | ------ | ------------------------------------- | -------- |
| `else`      | 0x05   | `($T[`[`$any`]`]) : ($T[`[`$any`]`])` | [B]      |

The `else` instruction binds the control-flow stack top's [label] to the current
position, pops an entry from the control-flow stack, pushes a new entry onto the
control-flow stack containing an unbound [label], the length of the current
value stack, and the signature of the control-flow stack entry that was just
popped, and then [branches](#branching) according to this entry. It returns the
values of its operands.

```{req} Else Must Bind If Label To Current Position
:id: REQ_ELSE_BIND_IF_LABEL
:status: open
:tags: semantics, control-flow-instructions
The `else` instruction must bind the if block's label to the current position, enabling backwards branches to skip to the else branch.
```

```{req} Else Must Pop And Push Control Flow Entry
:id: REQ_ELSE_POP_PUSH_ENTRY
:status: open
:tags: semantics, control-flow-instructions
The `else` instruction must pop a control-flow stack entry from the if block and push a new entry with an unbound label and the same signature for the else block.
```

```{req} Else Must Branch According To Popped Entry
:id: REQ_ELSE_BRANCH
:status: open
:tags: semantics, control-flow-instructions
After popping and pushing, the `else` instruction must execute a branch according to the pushed control-flow stack entry.
```

```{req} Else Must Return Operand Values
:id: REQ_ELSE_RETURNS_OPERANDS
:status: open
:tags: semantics, control-flow-instructions
The `else` instruction must return the values of its operands to the else block's value stack.
```

**Validation:**
 - `$T[`[`$any`]`]` is required to be the type sequence described by the
   signature of the popped control-flow stack entry.

```{req} Else Operand Types Must Match Popped Signature
:id: REQ_ELSE_OPERAND_TYPE_MATCH
:status: open
:tags: validation, control-flow-instructions
The operand types of the `else` instruction must match the type sequence described by the signature of the if block's control-flow stack entry.
```

> Each `else` needs a corresponding [`end`](#end) to bind its label and pop its
control-flow stack entry.

> Unlike in the branch instructions, `else` and `end` do not ignore surplus
values on the stack, as [`$any`] is bound to the number of values pushed within
the current block.

TODO: Explicitly describe the binding of $T.

(end)=
#### End

| Mnemonic    | Opcode | Signature                             | Families |
| ----------- | ------ | ------------------------------------- | -------- |
| `end`       | 0x0b   | `($T[`[`$any`]`]) : ($T[`[`$any`]`])` |          |

The `end` instruction pops an entry from the control-flow stack. If the entry's
[label] is unbound, the label is bound to the current position. It returns the
values of its operands.

```{req} End Must Pop Control Flow Stack Entry
:id: REQ_END_POP_ENTRY
:status: open
:tags: semantics, control-flow-instructions
The `end` instruction must pop an entry from the control-flow stack.
```

```{req} End Must Bind Unbound Label
:id: REQ_END_BIND_LABEL
:status: open
:tags: semantics, control-flow-instructions
If the popped control-flow stack entry's label is unbound, the `end` instruction must bind the label to the current position.
```

```{req} End Must Return Operand Values
:id: REQ_END_RETURNS_OPERANDS
:status: open
:tags: semantics, control-flow-instructions
The `end` instruction must return the values of its operands as the block's result values.
```

**Validation:**
 - `$T[`[`$any`]`]` is required to be the type sequence described by the
   signature of the popped control-flow stack entry.
 - If the control-flow stack entry was pushed by an `if` (and there was no
   `else`), the signature is required to be `void`.

```{req} End Operand Types Must Match Popped Signature
:id: REQ_END_OPERAND_TYPE_MATCH
:status: open
:tags: validation, control-flow-instructions
The operand types of the `end` instruction must match the type sequence described by the signature of the popped control-flow stack entry.
```

```{req} If Without Else Must Have Void Signature
:id: REQ_IF_NO_ELSE_VOID_SIGNATURE
:status: open
:tags: validation, control-flow-instructions
If a control-flow stack entry was pushed by an `if` instruction without a corresponding `else`, the entry's signature must be `void` (no return values).
```

> Each `end` ends a region begun by a corresponding `block`, `loop`, `if`,
`else`, or the function entry.

> Unlike in the branch instructions, `else` and `end` do not ignore surplus
values on the stack, as [`$any`] is bound to the number of values pushed within
the current block.

TODO: Explicitly describe the binding of $T.

(return)=
#### Return

| Mnemonic    | Opcode | Signature                                             | Families |
| ----------- | ------ | ----------------------------------------------------- | -------- |
| `return`    | 0x0f   | `($T[`[`$block_arity`]`]) : ($T[`[`$block_arity`]`])` | [B] [Q]  |

The `return` instruction [branches](#branching) according to the control-flow
stack bottom. It returns the values of its operands.

```{req} Return Must Branch To Function Bottom
:id: REQ_RETURN_BRANCH_TO_BOTTOM
:status: open
:tags: semantics, control-flow-instructions
The `return` instruction must execute a branch according to the control-flow stack bottom (outermost region), effectively returning from the function.
```

```{req} Return Must Return Operand Values
:id: REQ_RETURN_RETURNS_OPERANDS
:status: open
:tags: semantics, control-flow-instructions
The `return` instruction must return the values of its operands as the function's return values.
```

> `return` is semantically equivalent to a `br` to the outermost control region.

> Implementations needn't literally perform a branch before performing the
actual function return.

TODO: Explicitly describe the binding of $T.

(unreachable)=
#### Unreachable

| Mnemonic      | Opcode | Signature                 | Families |
| ------------- | ------ | ------------------------- | -------- |
| `unreachable` | 0x00   | `() : ()`                 | [Q]      |

**Trap:** Unreachable reached, always.

```{req} Unreachable Must Always Trap
:id: REQ_UNREACHABLE_ALWAYS_TRAPS
:status: open
:tags: semantics, control-flow-instructions
The `unreachable` instruction must always trap with an "Unreachable reached" trap, unconditionally terminating execution.
```

> The `unreachable` instruction is meant to represent code that isn't meant to
be executed except in the case of a bug in the application.

(basic-instructions)=
### Basic Instructions

0. [No-Op](#no-op)
0. [Drop](#drop)
0. [Constant](#constant)
0. [Get Local](#get-local)
0. [Set Local](#set-local)
0. [Tee Local](#tee-local)
0. [Get Global](#get-global)
0. [Set Global](#set-global)
0. [Select](#select)
0. [Call](#call)
0. [Indirect Call](#indirect-call)

(no-op)=
#### No-Op

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `nop`       | 0x01   | `() : ()`                   |          |

The `nop` instruction does nothing.

```{req} NOP Instruction Must Do Nothing
:id: REQ_NOP_NO_OPERATION
:status: open
:tags: semantics, basic-instructions
The `nop` instruction must have no semantic effect; it does not modify state or push/pop values.
```

(drop)=
#### Drop

| Mnemonic    | Opcode | Signature                    | Families |
| ----------- | ------ | ---------------------------- | -------- |
| `drop`      | 0x1a   | `($T[1]) : ()`               |          |

The `drop` instruction does nothing.

```{req} Drop Instruction Must Discard Value
:id: REQ_DROP_DISCARD_VALUE
:status: open
:tags: semantics, basic-instructions
The `drop` instruction must remove the value from the top of the value stack without using or returning it.
```

> This differs from `nop` in that it has an operand, so it can be used to
discard unneeded values from the value stack.

> This instruction is sometimes called "value-polymorphic" because it can
accept values of any type.

TODO: Explicitly describe the binding of $T.

(constant)=
#### Constant

| Mnemonic    | Opcode | Immediates            | Signature    | Families |
| ----------- | ------ | --------------------- | ------------ | -------- |
| `i32.const` | 0x41   | `$value`: [varsint32] | `() : (i32)` |          |
| `i64.const` | 0x42   | `$value`: [varsint64] | `() : (i64)` |          |
| `f32.const` | 0x43   | `$value`: [float32]   | `() : (f32)` |          |
| `f64.const` | 0x44   | `$value`: [float64]   | `() : (f64)` |          |

The `const` instruction returns the value of `$value`.

```{req} Const Instruction Must Push Constant Value
:id: REQ_CONST_PUSH_VALUE
:status: open
:tags: semantics, basic-instructions
The `const` instruction must push the immediate constant `$value` onto the value stack.
```

```{req} Const Instruction Type Must Match Value Type
:id: REQ_CONST_TYPE_MATCH
:status: open
:tags: semantics, basic-instructions
The type of the constant instruction (i32.const, i64.const, f32.const, f64.const) must match the type of the immediate value being pushed.
```

> Floating-point constants can be created with arbitrary bit-patterns.

(get-local)=
#### Get Local

| Mnemonic    | Opcode | Immediates         | Signature      | Families |
| ----------- | ------ | ------------------ | -------------- | -------- |
| `local.get` | 0x20   | `$id`: [varuint32] | `() : ($T[1])` |          |

The `local.get` instruction returns the value of the local at index `$id` in the
locals vector of the current [function execution]. The type parameter is bound
to the type of the local.

```{req} Get Local Must Push Local Value
:id: REQ_LOCAL_GET_PUSH_VALUE
:status: open
:tags: semantics, basic-instructions
The `local.get` instruction must push the current value of the local variable at index `$id` onto the value stack.
```

```{req} Get Local Value Type Must Match Declaration
:id: REQ_LOCAL_GET_TYPE_MATCH
:status: open
:tags: validation, basic-instructions
The type of the value returned by `local.get` must match the declared type of the local variable.
```

**Validation:**
 - `$id` is required to be within the bounds of the locals vector.

```{req} Get Local Index Must Be Within Bounds
:id: REQ_LOCAL_GET_INDEX_BOUNDS
:status: open
:tags: validation, basic-instructions
The local variable index `$id` must refer to a valid local variable within the function's locals vector.
```

(set-local)=
#### Set Local

| Mnemonic    | Opcode | Immediates         | Signature      | Families |
| ----------- | ------ | ------------------ | -------------- | -------- |
| `local.set` | 0x21   | `$id`: [varuint32] | `($T[1]) : ()` |          |

The `local.set` instruction sets the value of the local at index `$id` in the
locals vector of the current [function execution] to the value given in the
operand. The type parameter is bound to the type of the local.

```{req} Set Local Must Update Local Value
:id: REQ_LOCAL_SET_UPDATE_VALUE
:status: open
:tags: semantics, basic-instructions
The `local.set` instruction must update the value of the local variable at index `$id` with the operand value.
```

```{req} Set Local Operand Type Must Match Declaration
:id: REQ_LOCAL_SET_TYPE_MATCH
:status: open
:tags: validation, basic-instructions
The type of the operand passed to `local.set` must match the declared type of the local variable.
```

```{req} Set Local Does Not Return Value
:id: REQ_LOCAL_SET_NO_RETURN
:status: open
:tags: semantics, basic-instructions
The `local.set` instruction must not push a value onto the value stack; it only updates the local variable.
```

**Validation:**
 - `$id` is required to be within the bounds of the locals vector.

```{req} Set Local Index Must Be Within Bounds
:id: REQ_LOCAL_SET_INDEX_BOUNDS
:status: open
:tags: validation, basic-instructions
The local variable index `$id` must refer to a valid local variable within the function's locals vector.
```

> `local.set` is semantically equivalent to a similar `local.tee` followed by a
`drop`.

(tee-local)=
#### Tee Local

| Mnemonic    | Opcode | Immediates         | Signature           | Families |
| ----------- | ------ | ------------------ | ------------------- | -------- |
| `local.tee` | 0x22   | `$id`: [varuint32] | `($T[1]) : ($T[1])` |          |

The `local.tee` instruction sets the value of the locals at index `$id` in the
locals vector of the current [function execution] to the value given in the
operand. Its return value is the value of its operand. The type parameter is
bound to the type of the local.

```{req} Tee Local Must Update And Return Value
:id: REQ_LOCAL_TEE_UPDATE_AND_RETURN
:status: open
:tags: semantics, basic-instructions
The `local.tee` instruction must update the value of the local variable at index `$id` with the operand value and push the same value onto the value stack.
```

```{req} Tee Local Operand Type Must Match Declaration
:id: REQ_LOCAL_TEE_TYPE_MATCH
:status: open
:tags: validation, basic-instructions
The type of the operand passed to `local.tee` must match the declared type of the local variable.
```

```{req} Tee Local Must Return Operand Value
:id: REQ_LOCAL_TEE_RETURNS_OPERAND
:status: open
:tags: semantics, basic-instructions
The `local.tee` instruction must push the operand value onto the value stack after updating the local.
```

**Validation:**
 - `$id` is required to be within the bounds of the locals vector.

```{req} Tee Local Index Must Be Within Bounds
:id: REQ_LOCAL_TEE_INDEX_BOUNDS
:status: open
:tags: validation, basic-instructions
The local variable index `$id` must refer to a valid local variable within the function's locals vector.
```

> This instruction's name is inspired by the ["tee" command] in other languages,
since it forwards the value of its operand to two places, the local and the
return value.

["tee" command]: https://en.wikipedia.org/wiki/Tee_(command)

(get-global)=
#### Get Global

| Mnemonic     | Opcode | Immediates         | Signature      | Families |
| ------------ | ------ | ------------------ | -------------- | -------- |
| `global.get` | 0x23   | `$id`: [varuint32] | `() : ($T[1])` |          |

The `global.get` instruction returns the value of the global identified by index
`$id` in the [global index space]. The type parameter is bound to the type of
the global.

```{req} Get Global Must Push Global Value
:id: REQ_GLOBAL_GET_PUSH_VALUE
:status: open
:tags: semantics, basic-instructions
The `global.get` instruction must push the current value of the global variable at index `$id` onto the value stack.
```

```{req} Get Global Value Type Must Match Declaration
:id: REQ_GLOBAL_GET_TYPE_MATCH
:status: open
:tags: validation, basic-instructions
The type of the value returned by `global.get` must match the declared type of the global variable.
```

**Validation:**
 - `$id` is required to be within the bounds of the global index space.

```{req} Get Global Index Must Be Within Bounds
:id: REQ_GLOBAL_GET_INDEX_BOUNDS
:status: open
:tags: validation, basic-instructions
The global variable index `$id` must refer to a valid global variable within the global index space.
```

(set-global)=
#### Set Global

| Mnemonic     | Opcode | Immediates         | Signature      | Families |
| ------------ | ------ | ------------------ | -------------- | -------- |
| `global.set` | 0x24   | `$id`: [varuint32] | `($T[1]) : ()` |          |

The `global.set` instruction sets the value of the global identified by index
`$id` in the [global index space] to the value given in the operand. The type
parameter is bound to the type of the global.

```{req} Set Global Must Update Global Value
:id: REQ_GLOBAL_SET_UPDATE_VALUE
:status: open
:tags: semantics, basic-instructions
The `global.set` instruction must update the value of the global variable at index `$id` with the operand value.
```

```{req} Set Global Operand Type Must Match Declaration
:id: REQ_GLOBAL_SET_TYPE_MATCH
:status: open
:tags: validation, basic-instructions
The type of the operand passed to `global.set` must match the declared type of the global variable.
```

```{req} Set Global Does Not Return Value
:id: REQ_GLOBAL_SET_NO_RETURN
:status: open
:tags: semantics, basic-instructions
The `global.set` instruction must not push a value onto the value stack; it only updates the global variable.
```

**Validation:**
 - `$id` is required to be within the bounds of the global index space.
 - The indexed global is required to be declared not immutable.

```{req} Set Global Index Must Be Within Bounds
:id: REQ_GLOBAL_SET_INDEX_BOUNDS
:status: open
:tags: validation, basic-instructions
The global variable index `$id` must refer to a valid global variable within the global index space.
```

```{req} Set Global Requires Mutable Global
:id: REQ_GLOBAL_SET_REQUIRES_MUTABLE
:status: open
:tags: validation, basic-instructions
The indexed global must be declared as mutable; attempting to set an immutable global is invalid.
```

(select)=
#### Select

| Mnemonic    | Opcode | Signature                                   | Families |
| ----------- | ------ | ------------------------------------------- | -------- |
| `select`    | 0x1b   | `($T[1], $T[1], $condition: i32) : ($T[1])` |          |

The `select` instruction returns its first operand if `$condition` is [true], or
its second operand otherwise.

> This instruction differs from the conditional or ternary operator, eg.
`x?y:z`, in some languages, in that it's not short-circuiting.

> This instruction is similar to a "conditional move" in other languages and is
meant to have similar performance properties.

> This instruction is sometimes called "value-polymorphic" because it can
operate on values of any type.

TODO: Explicitly describe the binding of $T.

```{req} Select Returns First Operand If True
:id: REQ_SELECT_FIRST_IF_TRUE
:status: open
:tags: control-flow, validation, semantics
The `select` instruction must return the first operand if the condition value is non-zero (true).
```

```{req} Select Returns Second Operand If False
:id: REQ_SELECT_SECOND_IF_FALSE
:status: open
:tags: control-flow, validation, semantics
The `select` instruction must return the second operand if the condition value is zero (false).
```

```{req} Select Non-Short-Circuit Evaluation
:id: REQ_SELECT_NOT_SHORT_CIRCUIT
:status: open
:tags: control-flow, validation, semantics
The `select` instruction must evaluate both operands before selecting, not short-circuiting the evaluation.
```

(call)=
#### Call

| Mnemonic    | Opcode | Immediates             | Signature                                  | Families |
| ----------- | ------ | ---------------------- | ------------------------------------------ | -------- |
| `call`      | 0x10   | `$callee`: [varuint32] | `($T[`[`$args`]`]) : ($T[`[`$returns`]`])` | [L]      |

The `call` instruction performs a [call](#calling) to the function with index
`$callee` in the [function index space].

**Validation:**
 - `$callee` is required to be within the bounds of the function index space.
 - [Call validation](#call-validation) is required; the callee signature is the
   signature of the indexed function.

```{req} Call Function Index Bounds
:id: REQ_CALL_CALLEE_INDEX_BOUNDS
:status: open
:tags: control-flow, validation, semantics
The `call` instruction's callee index must be within the bounds of the function index space.
```

```{req} Call Function Signature Validation
:id: REQ_CALL_SIGNATURE_VALIDATION
:status: open
:tags: control-flow, validation, semantics
The `call` instruction must validate that the callee signature matches the caller's signature expectations.
```

```{req} Call Direct Function Invocation
:id: REQ_CALL_DIRECT_INVOCATION
:status: open
:tags: control-flow, validation, semantics
The `call` instruction must invoke the function indexed in the function index space directly.
```

(indirect-call)=
#### Indirect Call

| Mnemonic        | Opcode | Immediates                                         | Signature                                                | Families |
| --------------- | ------ | -------------------------------------------------- | -------------------------------------------------------- | -------- |
| `call_indirect` | 0x11   | `$signature`: [varuint32], `$reserved`: [varuint1] | `($T[`[`$args`]`], $callee: i32) : ($T[`[`$returns`]`])` | [L]      |

The `call_indirect` instruction performs a [call](#calling) to the function in
the default table with index `$callee`.

**Trap:** Indirect Callee Absent, if the indexed table element is the special
"null" value.

**Trap:** Indirect Call Type Mismatch, if the signature of the function with
index `$callee` differs from the signature in the [Type Section] with index
`$signature`.

**Validation:**
 - [Call validation](#call-validation) is required; the callee signature is the
   signature with index `$signature` in the [Type Section].

> The dynamic caller/callee signature match is structural rather than nominal.

> Indices in the default table can provide applications with the functionality
of function pointers.

> In future versions of WebAssembly, the reserved immediate may be used to index additional tables.

```{req} Indirect Call Table Lookup
:id: REQ_CALL_INDIRECT_TABLE_LOOKUP
:status: open
:tags: control-flow, validation, semantics
The `call_indirect` instruction must look up the callee function in the default table using the provided index.
```

```{req} Indirect Call Callee Present Check
:id: REQ_CALL_INDIRECT_CALLEE_PRESENT
:status: open
:tags: control-flow, validation, semantics
The `call_indirect` instruction must trap with "Indirect Callee Absent" if the table entry is the null value.
```

```{req} Indirect Call Signature Validation
:id: REQ_CALL_INDIRECT_SIGNATURE_MATCH
:status: open
:tags: control-flow, validation, semantics
The `call_indirect` instruction must validate that the callee's signature from the Type Section matches the indexed type.
```

```{req} Indirect Call Type Mismatch Trap
:id: REQ_CALL_INDIRECT_TYPE_MISMATCH_TRAP
:status: open
:tags: control-flow, validation, semantics
The `call_indirect` instruction must trap with "Indirect Call Type Mismatch" if the callee signature does not match the expected signature.
```

(integer-arithmetic-instructions)=
### Integer Arithmetic Instructions

0. [Integer Add](#integer-add)
0. [Integer Subtract](#integer-subtract)
0. [Integer Multiply](#integer-multiply)
0. [Integer Divide, Signed](#integer-divide-signed)
0. [Integer Divide, Unsigned](#integer-divide-unsigned)
0. [Integer Remainder, Signed](#integer-remainder-signed)
0. [Integer Remainder, Unsigned](#integer-remainder-unsigned)
0. [Integer Bitwise And](#integer-bitwise-and)
0. [Integer Bitwise Or](#integer-bitwise-or)
0. [Integer Bitwise Exclusive-Or](#integer-bitwise-exclusive-or)
0. [Integer Shift Left](#integer-shift-left)
0. [Integer Shift Right, Signed](#integer-shift-right-signed)
0. [Integer Shift Right, Unsigned](#integer-shift-right-unsigned)
0. [Integer Rotate Left](#integer-rotate-left)
0. [Integer Rotate Right](#integer-rotate-right)
0. [Integer Count Leading Zeros](#integer-count-leading-zeros)
0. [Integer Count Trailing Zeros](#integer-count-trailing-zeros)
0. [Integer Population Count](#integer-population-count)
0. [Integer Equal To Zero](#integer-equal-to-zero)

(integer-add)=
#### Integer Add

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.add`   | 0x6a   | `(i32, i32) : (i32)`        | [G]      |
| `i64.add`   | 0x7c   | `(i64, i64) : (i64)`        | [G]      |

The integer `add` instruction returns the [two's complement sum] of its
operands. The carry bit is silently discarded.

> Due to WebAssembly's use of [two's complement] to represent signed values,
this instruction can be used to add either signed or unsigned values.

```{req} Integer Addition
:id: REQ_INT_ADD_SUM
:status: open
:tags: arithmetic, validation, semantics
The `add` instruction must return the two's complement sum of its operands with carry bit discarded.
```

(integer-subtract)=
#### Integer Subtract

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.sub`   | 0x6b   | `(i32, i32) : (i32)`        | [G]      |
| `i64.sub`   | 0x7d   | `(i64, i64) : (i64)`        | [G]      |

The integer `sub` instruction returns the [two's complement difference] of its
operands. The borrow bit is silently discarded.

> Due to WebAssembly's use of [two's complement] to represent signed values,
this instruction can be used to subtract either signed or unsigned values.

> An integer negate operation can be performed by a `sub` instruction with zero
as the first operand.

```{req} Integer Subtraction
:id: REQ_INT_SUB_DIFF
:status: open
:tags: arithmetic, validation, semantics
The `sub` instruction must return the two's complement difference of its operands with borrow bit discarded.
```

(integer-multiply)=
#### Integer Multiply

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.mul`   | 0x6c   | `(i32, i32) : (i32)`        | [G]      |
| `i64.mul`   | 0x7e   | `(i64, i64) : (i64)`        | [G]      |

The integer `mul` instruction returns the low half of the
[two's complement product] its operands.

> Due to WebAssembly's use of [two's complement] to represent signed values,
this instruction can be used to multiply either signed or unsigned values.

```{req} Integer Multiplication
:id: REQ_INT_MUL_PRODUCT
:status: open
:tags: arithmetic, validation, semantics
The `mul` instruction must return the low half of the two's complement product of its operands.
```

(integer-divide-signed)=
#### Integer Divide, Signed

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.div_s` | 0x6d   | `(i32, i32) : (i32)`        | [S]      |
| `i64.div_s` | 0x7f   | `(i64, i64) : (i64)`        | [S]      |

The `div_s` instruction returns the signed quotient of its operands, interpreted
as signed. The quotient is silently rounded to the nearest integer toward zero.

**Trap:** Signed Integer Overflow, when the [minimum signed integer value] is
divided by `-1`.

**Trap:** Integer Division By Zero, when the second operand (the divisor) is
zero.

```{req} Integer Signed Division
:id: REQ_INT_DIV_S_QUOTIENT
:status: open
:tags: arithmetic, validation, semantics
The `div_s` instruction must return the signed quotient of its operands, rounded toward zero.
```

```{req} Integer Signed Division By Zero Trap
:id: REQ_INT_DIV_S_ZERO_TRAP
:status: open
:tags: arithmetic, validation, semantics
The `div_s` instruction must trap with "Integer Division By Zero" if the divisor is zero.
```

```{req} Integer Signed Division Overflow Trap
:id: REQ_INT_DIV_S_OVERFLOW_TRAP
:status: open
:tags: arithmetic, validation, semantics
The `div_s` instruction must trap with "Signed Integer Overflow" if dividing the minimum signed value by -1.
```

(integer-divide-unsigned)=
#### Integer Divide, Unsigned

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.div_u` | 0x6e   | `(i32, i32) : (i32)`        | [U]      |
| `i64.div_u` | 0x80   | `(i64, i64) : (i64)`        | [U]      |

The `div_u` instruction returns the unsigned quotient of its operands,
interpreted as unsigned. The quotient is silently rounded to the nearest integer
toward zero.

**Trap:** Integer Division By Zero, when the second operand (the divisor) is
zero.

```{req} Integer Unsigned Division
:id: REQ_INT_DIV_U_QUOTIENT
:status: open
:tags: arithmetic, validation, semantics
The `div_u` instruction must return the unsigned quotient of its operands, rounded toward zero.
```

```{req} Integer Unsigned Division By Zero Trap
:id: REQ_INT_DIV_U_ZERO_TRAP
:status: open
:tags: arithmetic, validation, semantics
The `div_u` instruction must trap with "Integer Division By Zero" if the divisor is zero.
```

(integer-remainder-signed)=
#### Integer Remainder, Signed

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.rem_s` | 0x6f   | `(i32, i32) : (i32)`        | [S] [R]  |
| `i64.rem_s` | 0x81   | `(i64, i64) : (i64)`        | [S] [R]  |

The `rem_s` instruction returns the signed remainder from a division of its
operand values interpreted as signed, with the result having the same sign as
the first operand (the dividend).

**Trap:** Integer Division By Zero, when the second operand (the divisor) is
zero.

> This instruction doesn't trap when the [minimum signed integer value] is
divided by `-1`; it returns `0` which is the correct remainder (even though the
same operands to `div_s` do cause a trap).

> This instruction differs from what is often called a ["modulo" operation] in
its handling of negative numbers.

> This instruction has some [common pitfalls] to avoid.

["modulo" operation]: https://en.wikipedia.org/wiki/Modulo_operation
[common pitfalls]: https://en.wikipedia.org/wiki/Modulo_operation#Common_pitfalls

```{req} Integer Signed Remainder
:id: REQ_INT_REM_S_REMAINDER
:status: open
:tags: arithmetic, validation, semantics
The `rem_s` instruction must return the signed remainder with the same sign as the dividend.
```

```{req} Integer Signed Remainder By Zero Trap
:id: REQ_INT_REM_S_ZERO_TRAP
:status: open
:tags: arithmetic, validation, semantics
The `rem_s` instruction must trap with "Integer Division By Zero" if the divisor is zero.
```

```{req} Integer Signed Remainder No Overflow
:id: REQ_INT_REM_S_NO_OVERFLOW_TRAP
:status: open
:tags: arithmetic, validation, semantics
The `rem_s` instruction must not trap for minimum signed value divided by -1; it returns 0.
```

(integer-remainder-unsigned)=
#### Integer Remainder, Unsigned

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.rem_u` | 0x70   | `(i32, i32) : (i32)`        | [U] [R]  |
| `i64.rem_u` | 0x82   | `(i64, i64) : (i64)`        | [U] [R]  |

The `rem_u` instruction returns the unsigned remainder from a division of its
operand values interpreted as unsigned.

**Trap:** Integer Division By Zero, when the second operand (the divisor) is
zero.

> This instruction corresponds to what is sometimes called "modulo" in other
languages.

```{req} Integer Unsigned Remainder
:id: REQ_INT_REM_U_REMAINDER
:status: open
:tags: arithmetic, validation, semantics
The `rem_u` instruction must return the unsigned remainder of its operands.
```

```{req} Integer Unsigned Remainder By Zero Trap
:id: REQ_INT_REM_U_ZERO_TRAP
:status: open
:tags: arithmetic, validation, semantics
The `rem_u` instruction must trap with "Integer Division By Zero" if the divisor is zero.
```

(integer-bitwise-and)=
#### Integer Bitwise And

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.and`   | 0x71   | `(i32, i32) : (i32)`        | [G]      |
| `i64.and`   | 0x83   | `(i64, i64) : (i64)`        | [G]      |

The `and` instruction returns the [bitwise and] of its operands.

[bitwise and]: https://en.wikipedia.org/wiki/Bitwise_operation#AND

```{req} Integer Bitwise And
:id: REQ_INT_AND_BITWISE
:status: open
:tags: arithmetic, validation, semantics
The `and` instruction must return the bitwise AND of its operands.
```

(integer-bitwise-or)=
#### Integer Bitwise Or

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.or`    | 0x72   | `(i32, i32) : (i32)`        | [G]      |
| `i64.or`    | 0x84   | `(i64, i64) : (i64)`        | [G]      |

The `or` instruction returns the [bitwise inclusive-or] of its operands.

[bitwise inclusive-or]: https://en.wikipedia.org/wiki/Bitwise_operation#OR

```{req} Integer Bitwise Or
:id: REQ_INT_OR_BITWISE
:status: open
:tags: arithmetic, validation, semantics
The `or` instruction must return the bitwise OR of its operands.
```

(integer-bitwise-exclusive-or)=
#### Integer Bitwise Exclusive-Or

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.xor`   | 0x73   | `(i32, i32) : (i32)`        | [G]      |
| `i64.xor`   | 0x85   | `(i64, i64) : (i64)`        | [G]      |

The `xor` instruction returns the [bitwise exclusive-or] of its operands.

> A [bitwise negate] operation can be performed by a `xor` instruction with
negative one as the first operand, an operation sometimes called
"one's complement" in other languages.

[bitwise exclusive-or]: https://en.wikipedia.org/wiki/Bitwise_operation#XOR
[bitwise negate]: https://en.wikipedia.org/wiki/Bitwise_operation#NOT

```{req} Integer Bitwise Exclusive-Or
:id: REQ_INT_XOR_BITWISE
:status: open
:tags: arithmetic, validation, semantics
The `xor` instruction must return the bitwise XOR of its operands.
```

(integer-shift-left)=
#### Integer Shift Left

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.shl`   | 0x74   | `(i32, i32) : (i32)`        | [T], [G] |
| `i64.shl`   | 0x86   | `(i64, i64) : (i64)`        | [T], [G] |

The `shl` instruction returns the value of the first operand [shifted] to the
left by the [shift count].

> This instruction effectively performs a multiplication by two to the power of
the shift count.

```{req} Integer Shift Left
:id: REQ_INT_SHL_SHIFT
:status: open
:tags: arithmetic, validation, semantics
The `shl` instruction must shift the first operand left by the shift count, with vacated bits filled with zeros.
```

(integer-shift-right-signed)=
#### Integer Shift Right, Signed

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.shr_s` | 0x75   | `(i32, i32) : (i32)`        | [T], [S] |
| `i64.shr_s` | 0x87   | `(i64, i64) : (i64)`        | [T], [S] |

The `shr_s` instruction returns the value of the first operand
[shifted](https://en.wikipedia.org/wiki/Arithmetic_shift) to the right by the
[shift count].

> This instruction corresponds to what is sometimes called
"arithmetic right shift" in other languages.

> `shr_s` is similar to `div_s` when the divisor is a power of two, however the
rounding of negative values is different. `shr_s` effectively rounds down, while
`div_s` rounds toward zero.

```{req} Integer Arithmetic Shift Right
:id: REQ_INT_SHR_S_SHIFT
:status: open
:tags: arithmetic, validation, semantics
The `shr_s` instruction must arithmetically shift right with sign extension filling vacated high-order bits.
```

(integer-shift-right-unsigned)=
#### Integer Shift Right, Unsigned

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.shr_u` | 0x76   | `(i32, i32) : (i32)`        | [T], [U] |
| `i64.shr_u` | 0x88   | `(i64, i64) : (i64)`        | [T], [U] |

The `shr_u` instruction returns the value of the first operand [shifted] to the
right by the [shift count].

> This instruction corresponds to what is sometimes called
"logical right shift" in other languages.

> This instruction effectively performs an unsigned division by two to the power
of the shift count.

```{req} Integer Logical Shift Right
:id: REQ_INT_SHR_U_SHIFT
:status: open
:tags: arithmetic, validation, semantics
The `shr_u` instruction must logically shift right with zeros filling vacated high-order bits.
```

(integer-rotate-left)=
#### Integer Rotate Left

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.rotl`  | 0x77   | `(i32, i32) : (i32)`        | [T], [G] |
| `i64.rotl`  | 0x89   | `(i64, i64) : (i64)`        | [T], [G] |

The `rotl` instruction returns the value of the first operand [rotated] to the
left by the [shift count].

> Rotating left is similar to shifting left, however vacated bits are set to the
values of the bits which would otherwise be discarded by the shift, so the bits
conceptually "rotate back around".

```{req} Integer Rotate Left
:id: REQ_INT_ROTL_ROTATE
:status: open
:tags: arithmetic, validation, semantics
The `rotl` instruction must rotate the bits left with bits rotated out at the high end moving to the low end.
```

(integer-rotate-right)=
#### Integer Rotate Right

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.rotr`  | 0x78   | `(i32, i32) : (i32)`        | [T], [G] |
| `i64.rotr`  | 0x8a   | `(i64, i64) : (i64)`        | [T], [G] |

The `rotr` instruction returns the value of the first operand [rotated] to the
right by the [shift count].

> Rotating right is similar to shifting right, however vacated bits are set to
the values of the bits which would otherwise be discarded by the shift, so the
bits conceptually "rotate back around".

```{req} Integer Rotate Right
:id: REQ_INT_ROTR_ROTATE
:status: open
:tags: arithmetic, validation, semantics
The `rotr` instruction must rotate the bits right with bits rotated out at the low end moving to the high end.
```

(integer-count-leading-zeros)=
#### Integer Count Leading Zeros

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.clz`   | 0x67   | `(i32) : (i32)`             | [G]      |
| `i64.clz`   | 0x79   | `(i64) : (i64)`             | [G]      |

The `clz` instruction returns the number of leading zeros in its operand. The
*leading zeros* are the longest contiguous sequence of zero-bits starting at the
most significant bit and extending downward.

> This instruction is fully defined when all bits are zero; it returns the
number of bits in the operand type.

```{req} Integer Count Leading Zeros
:id: REQ_INT_CLZ_COUNT
:status: open
:tags: arithmetic, validation, semantics
The `clz` instruction must count the longest contiguous zero-bits from the most significant bit downward.
```

(integer-count-trailing-zeros)=
#### Integer Count Trailing Zeros

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.ctz`   | 0x68   | `(i32) : (i32)`             | [G]      |
| `i64.ctz`   | 0x7a   | `(i64) : (i64)`             | [G]      |

The `ctz` instruction returns the number of trailing zeros in its operand. The
*trailing zeros* are the longest contiguous sequence of zero-bits starting at
the least significant bit and extending upward.

> This instruction is fully defined when all bits are zero; it returns the
number of bits in the operand type.

```{req} Integer Count Trailing Zeros
:id: REQ_INT_CTZ_COUNT
:status: open
:tags: arithmetic, validation, semantics
The `ctz` instruction must count the longest contiguous zero-bits from the least significant bit upward.
```

(integer-population-count)=
#### Integer Population Count

| Mnemonic     | Opcode | Signature                  | Families |
| ------------ | ------ | -------------------------- | -------- |
| `i32.popcnt` | 0x69   | `(i32) : (i32)`            | [G]      |
| `i64.popcnt` | 0x7b   | `(i64) : (i64)`            | [G]      |

The `popcnt` instruction returns the number of 1-bits in its operand.

> This instruction is fully defined when all bits are zero; it returns `0`.

> This instruction corresponds to what is sometimes called a ["hamming weight"]
in other languages.

["hamming weight"]: https://en.wikipedia.org/wiki/Hamming_weight

```{req} Integer Population Count
:id: REQ_INT_POPCNT_COUNT
:status: open
:tags: arithmetic, validation, semantics
The `popcnt` instruction must count the number of 1-bits in the operand.
```

(integer-equal-to-zero)=
#### Integer Equal To Zero

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.eqz`   | 0x45   | `(i32) : (i32)`             | [G]      |
| `i64.eqz`   | 0x50   | `(i64) : (i32)`             | [G]      |

The `eqz` instruction returns [true] if the operand is equal to zero, or [false]
otherwise.

> This serves as a form of "logical not" operation which can be used to invert
[boolean] values.

```{req} Integer Equal To Zero
:id: REQ_INT_EQZ_TEST
:status: open
:tags: comparison, validation, semantics
The `eqz` instruction must return 1 (i32) if the operand is zero, 0 otherwise.
```

(floating-point-arithmetic-instructions)=
### Floating-Point Arithmetic Instructions

0. [Floating-Point Add](#floating-point-add)
0. [Floating-Point Subtract](#floating-point-subtract)
0. [Floating-Point Multiply](#floating-point-multiply)
0. [Floating-Point Divide](#floating-point-divide)
0. [Floating-Point Square Root](#floating-point-square-root)
0. [Floating-Point Minimum](#floating-point-minimum)
0. [Floating-Point Maximum](#floating-point-maximum)
0. [Floating-Point Ceiling](#floating-point-ceiling)
0. [Floating-Point Floor](#floating-point-floor)
0. [Floating-Point Truncate](#floating-point-truncate)
0. [Floating-Point Round To Nearest Integer](#floating-point-round-to-nearest-integer)
0. [Floating-Point Absolute Value](#floating-point-absolute-value)
0. [Floating-Point Negate](#floating-point-negate)
0. [Floating-Point CopySign](#floating-point-copysign)

(floating-point-add)=
#### Floating-Point Add

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.add`   | 0x92   | `(f32, f32) : (f32)`        | [F]      |
| `f64.add`   | 0xa0   | `(f64, f64) : (f64)`        | [F]      |

The floating-point `add` instruction performs the IEEE 754-2008 `addition`
operation according to the [general floating-point rules][F].

(floating-point-subtract)=
#### Floating-Point Subtract

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.sub`   | 0x93   | `(f32, f32) : (f32)`        | [F]      |
| `f64.sub`   | 0xa1   | `(f64, f64) : (f64)`        | [F]      |

The floating-point `sub` instruction performs the IEEE 754-2008 `subtraction`
operation according to the [general floating-point rules][F].

(floating-point-multiply)=
#### Floating-Point Multiply

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.mul`   | 0x94   | `(f32, f32) : (f32)`        | [F]      |
| `f64.mul`   | 0xa2   | `(f64, f64) : (f64)`        | [F]      |

The floating-point `mul` instruction performs the IEEE 754-2008 `multiplication`
operation according to the [general floating-point rules][F].

(floating-point-divide)=
#### Floating-Point Divide

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.div`   | 0x95   | `(f32, f32) : (f32)`        | [F]      |
| `f64.div`   | 0xa3   | `(f64, f64) : (f64)`        | [F]      |

The `div` instruction performs the IEEE 754-2008 `division` operation according
to the [general floating-point rules][F].

(floating-point-square-root)=
#### Floating-Point Square Root

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.sqrt`  | 0x91   | `(f32) : (f32)`             | [F]      |
| `f64.sqrt`  | 0x9f   | `(f64) : (f64)`             | [F]      |

The `sqrt` instruction performs the IEEE 754-2008 `squareRoot` operation
according to the [general floating-point rules][F].

(floating-point-minimum)=
#### Floating-Point Minimum

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.min`   | 0x96   | `(f32, f32) : (f32)`        | [F]      |
| `f64.min`   | 0xa4   | `(f64, f64) : (f64)`        | [F]      |

The `min` instruction returns the minimum value among its operands. For this
instruction, negative zero is considered less than zero. If either operand is a
NaN, the result is a NaN determined by the [general floating-point rules][F].

> This instruction corresponds to what is sometimes called "minNaN" in other
languages.

> This differs from the IEEE 754-2008 `minNum` operation in that it returns a
NaN if either operand is a NaN, and in that the behavior when the operands are
zeros of differing signs is fully specified.

> This differs from the common `x<y?x:y` expansion in its handling of
negative zero and NaN values.

(floating-point-maximum)=
#### Floating-Point Maximum

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.max`   | 0x97   | `(f32, f32) : (f32)`        | [F]      |
| `f64.max`   | 0xa5   | `(f64, f64) : (f64)`        | [F]      |

The `max` instruction returns the maximum value among its operands. For this
instruction, negative zero is considered less than zero. If either operand is a
NaN, the result is a NaN determined by the [general floating-point rules][F].

> This instruction corresponds to what is sometimes called "maxNaN" in other
languages.

> This differs from the IEEE 754-2008 `maxNum` operation in that it returns a
NaN if either operand is a NaN, and in that the behavior when the operands are
zeros of differing signs is fully specified.

> This differs from the common `x>y?x:y` expansion in its handling of negative
zero and NaN values.

(floating-point-ceiling)=
#### Floating-Point Ceiling

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.ceil`  | 0x8d   | `(f32) : (f32)`             | [F]      |
| `f64.ceil`  | 0x9b   | `(f64) : (f64)`             | [F]      |

The `ceil` instruction performs the IEEE 754-2008
`roundToIntegralTowardPositive` operation according to the
[general floating-point rules][F].

> ["Ceiling"][Floor and Ceiling Functions] describes the rounding method used
here; the value is rounded up to the nearest integer.

(floating-point-floor)=
#### Floating-Point Floor

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.floor` | 0x8e   | `(f32) : (f32)`             | [F]      |
| `f64.floor` | 0x9c   | `(f64) : (f64)`             | [F]      |

The `floor` instruction performs the IEEE 754-2008
`roundToIntegralTowardNegative` operation according to the
[general floating-point rules][F].

> ["Floor"][Floor and Ceiling Functions] describes the rounding method used
here; the value is rounded down to the nearest integer.

(floating-point-truncate)=
#### Floating-Point Truncate

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.trunc` | 0x8f   | `(f32) : (f32)`             | [F]      |
| `f64.trunc` | 0x9d   | `(f64) : (f64)`             | [F]      |

The `trunc` instruction performs the IEEE 754-2008
`roundToIntegralTowardZero` operation according to the
[general floating-point rules][F].

> ["Truncate"] describes the rounding method used here; the fractional part of
the value is discarded, effectively rounding to the nearest integer toward zero.

> This form of rounding is called a `chop` in other languages.

["Truncate"]: https://en.wikipedia.org/wiki/Truncation

(floating-point-round-to-nearest-integer)=
#### Floating-Point Round To Nearest Integer

| Mnemonic      | Opcode | Signature                 | Families |
| ------------- | ------ | ------------------------- | -------- |
| `f32.nearest` | 0x90   | `(f32) : (f32)`           | [F]      |
| `f64.nearest` | 0x9e   | `(f64) : (f64)`           | [F]      |

The `nearest` instruction performs the IEEE 754-2008
`roundToIntegralTiesToEven` operation according to the
[general floating-point rules][F].

> "Nearest" describes the rounding method used here; the value is
[rounded to the nearest integer], with
[ties rounded toward the value with an even least-significant digit].

> This instruction differs from [`Math.round` in ECMAScript] which rounds ties
up, and it differs from [`round` in C] which rounds ties away from zero.

> This instruction corresponds to what is called `roundeven` in other languages.

[rounded to the nearest integer]: https://en.wikipedia.org/wiki/Nearest_integer_function
[ties rounded toward the value with an even least-significant digit]: https://en.wikipedia.org/wiki/Rounding#Round_half_to_even
[`Math.round` in ECMAScript]: https://tc39.github.io/ecma262/#sec-math.round
[`round` in C]: http://en.cppreference.com/w/c/numeric/math/round

(floating-point-absolute-value)=
#### Floating-Point Absolute Value

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.abs`   | 0x8b   | `(f32) : (f32)`             | [E]      |
| `f64.abs`   | 0x99   | `(f64) : (f64)`             | [E]      |

The `abs` instruction performs the IEEE 754-2008 `abs` operation.

> This is a bitwise instruction; it sets the sign bit to zero and preserves all
other bits, even when the operand is a NaN or a zero.

> This differs from comparing whether the operand value is less than zero and
negating it, because comparisons treat negative zero as equal to zero, and NaN
values as not less than zero.

(floating-point-negate)=
#### Floating-Point Negate

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.neg`   | 0x8c   | `(f32) : (f32)`             | [E]      |
| `f64.neg`   | 0x9a   | `(f64) : (f64)`             | [E]      |

The `neg` instruction performs the IEEE 754-2008 `negate` operation.

> This is a bitwise instruction; it inverts the sign bit and preserves all other
bits, even when the operand is a NaN or a zero.

> This differs from subtracting the operand value from negative zero or
multiplying it by negative one, because subtraction and multiplication follow
the [general floating-point rules][F] and may not preserve the bits of NaN
values.

(floating-point-copysign)=
#### Floating-Point CopySign

| Mnemonic       | Opcode | Signature                | Families |
| -------------- | ------ | ------------------------ | -------- |
| `f32.copysign` | 0x98   | `(f32, f32) : (f32)`     | [E]      |
| `f64.copysign` | 0xa6   | `(f64, f64) : (f64)`     | [E]      |

The `copysign` instruction performs the IEEE 754-2008 `copySign` operation.

> This is a bitwise instruction; it combines the sign bit from the second
operand with all bits other than the sign bit from the first operand, even if
either operand is a NaN or a zero.

(integer-comparison-instructions)=
### Integer Comparison Instructions

0. [Integer Equality](#integer-equality)
0. [Integer Inequality](#integer-inequality)
0. [Integer Less Than, Signed](#integer-less-than-signed)
0. [Integer Less Than, Unsigned](#integer-less-than-unsigned)
0. [Integer Less Than Or Equal To, Signed](#integer-less-than-or-equal-to-signed)
0. [Integer Less Than Or Equal To, Unsigned](#integer-less-than-or-equal-to-unsigned)
0. [Integer Greater Than, Signed](#integer-greater-than-signed)
0. [Integer Greater Than, Unsigned](#integer-greater-than-unsigned)
0. [Integer Greater Than Or Equal To, Signed](#integer-greater-than-or-equal-to-signed)
0. [Integer Greater Than Or Equal To, Unsigned](#integer-greater-than-or-equal-to-unsigned)

(integer-equality)=
#### Integer Equality

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.eq`    | 0x46   | `(i32, i32) : (i32)`        | [C], [G] |
| `i64.eq`    | 0x51   | `(i64, i64) : (i32)`        | [C], [G] |

The integer `eq` instruction tests whether the operands are equal.

```{req} Integer Equality Comparison
:id: REQ_INT_EQ_COMPARE
:status: open
:tags: comparison, validation, semantics
The `eq` instruction must compare the two operands and return 1 (i32) if they are equal, 0 otherwise.
```

(integer-inequality)=
#### Integer Inequality

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.ne`    | 0x47   | `(i32, i32) : (i32)`        | [C], [G] |
| `i64.ne`    | 0x52   | `(i64, i64) : (i32)`        | [C], [G] |

The integer `ne` instruction tests whether the operands are not equal.

> This instruction corresponds to what is sometimes called "differs" in other
languages.

```{req} Integer Inequality Comparison
:id: REQ_INT_NE_COMPARE
:status: open
:tags: comparison, validation, semantics
The `ne` instruction must compare the two operands and return 1 (i32) if they are not equal, 0 otherwise.
```

(integer-less-than-signed)=
#### Integer Less Than, Signed

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.lt_s`  | 0x48   | `(i32, i32) : (i32)`        | [C], [S] |
| `i64.lt_s`  | 0x53   | `(i64, i64) : (i32)`        | [C], [S] |

The `lt_s` instruction tests whether the first operand is less than the second
operand, interpreting the operands as signed.

```{req} Integer Less Than Comparison (Signed)
:id: REQ_INT_LT_S_COMPARE
:status: open
:tags: comparison, validation, semantics
The `lt_s` instruction must interpret operands as signed integers and return 1 (i32) if the first operand is less than the second, 0 otherwise.
```

(integer-less-than-unsigned)=
#### Integer Less Than, Unsigned

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.lt_u`  | 0x49   | `(i32, i32) : (i32)`        | [C], [U] |
| `i64.lt_u`  | 0x54   | `(i64, i64) : (i32)`        | [C], [U] |

The `lt_u` instruction tests whether the first operand is less than the second
operand, interpreting the operands as unsigned.

```{req} Integer Less Than Comparison (Unsigned)
:id: REQ_INT_LT_U_COMPARE
:status: open
:tags: comparison, validation, semantics
The `lt_u` instruction must interpret operands as unsigned integers and return 1 (i32) if the first operand is less than the second, 0 otherwise.
```

(integer-less-than-or-equal-to-signed)=
#### Integer Less Than Or Equal To, Signed

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.le_s`  | 0x4c   | `(i32, i32) : (i32)`        | [C], [S] |
| `i64.le_s`  | 0x57   | `(i64, i64) : (i32)`        | [C], [S] |

The `le_s` instruction tests whether the first operand is less than or equal to
the second operand, interpreting the operands as signed.

> This instruction corresponds to what is sometimes called "at most" in other
languages.

```{req} Integer Less Than Or Equal Comparison (Signed)
:id: REQ_INT_LE_S_COMPARE
:status: open
:tags: comparison, validation, semantics
The `le_s` instruction must interpret operands as signed integers and return 1 (i32) if the first operand is less than or equal to the second, 0 otherwise.
```

(integer-less-than-or-equal-to-unsigned)=
#### Integer Less Than Or Equal To, Unsigned

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.le_u`  | 0x4d   | `(i32, i32) : (i32)`        | [C], [U] |
| `i64.le_u`  | 0x58   | `(i64, i64) : (i32)`        | [C], [U] |

The `le_u` instruction tests whether the first operand is less than or equal to
the second operand, interpreting the operands as unsigned.

> This instruction corresponds to what is sometimes called "at most" in other
languages.

```{req} Integer Less Than Or Equal Comparison (Unsigned)
:id: REQ_INT_LE_U_COMPARE
:status: open
:tags: comparison, validation, semantics
The `le_u` instruction must interpret operands as unsigned integers and return 1 (i32) if the first operand is less than or equal to the second, 0 otherwise.
```

(integer-greater-than-signed)=
#### Integer Greater Than, Signed

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.gt_s`  | 0x4a   | `(i32, i32) : (i32)`        | [C], [S] |
| `i64.gt_s`  | 0x55   | `(i64, i64) : (i32)`        | [C], [S] |

The `gt_s` instruction tests whether the first operand is greater than the
second operand, interpreting the operands as signed.

```{req} Integer Greater Than Comparison (Signed)
:id: REQ_INT_GT_S_COMPARE
:status: open
:tags: comparison, validation, semantics
The `gt_s` instruction must interpret operands as signed integers and return 1 (i32) if the first operand is greater than the second, 0 otherwise.
```

(integer-greater-than-unsigned)=
#### Integer Greater Than, Unsigned

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.gt_u`  | 0x4b   | `(i32, i32) : (i32)`        | [C], [U] |
| `i64.gt_u`  | 0x56   | `(i64, i64) : (i32)`        | [C], [U] |

The `gt_u` instruction tests whether the first operand is greater than the
second operand, interpreting the operands as unsigned.

```{req} Integer Greater Than Comparison (Unsigned)
:id: REQ_INT_GT_U_COMPARE
:status: open
:tags: comparison, validation, semantics
The `gt_u` instruction must interpret operands as unsigned integers and return 1 (i32) if the first operand is greater than the second, 0 otherwise.
```

(integer-greater-than-or-equal-to-signed)=
#### Integer Greater Than Or Equal To, Signed

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.ge_s`  | 0x4e   | `(i32, i32) : (i32)`        | [C], [S] |
| `i64.ge_s`  | 0x59   | `(i64, i64) : (i32)`        | [C], [S] |

The `ge_s` instruction tests whether the first operand is greater than or equal
to the second operand, interpreting the operands as signed.

> This instruction corresponds to what is sometimes called "at least" in other
languages.

```{req} Integer Greater Than Or Equal Comparison (Signed)
:id: REQ_INT_GE_S_COMPARE
:status: open
:tags: comparison, validation, semantics
The `ge_s` instruction must interpret operands as signed integers and return 1 (i32) if the first operand is greater than or equal to the second, 0 otherwise.
```

(integer-greater-than-or-equal-to-unsigned)=
#### Integer Greater Than Or Equal To, Unsigned

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `i32.ge_u`  | 0x4f   | `(i32, i32) : (i32)`        | [C], [U] |
| `i64.ge_u`  | 0x5a   | `(i64, i64) : (i32)`        | [C], [U] |

The `ge_u` instruction tests whether the first operand is greater than or equal
to the second operand, interpreting the operands as unsigned.

> This instruction corresponds to what is sometimes called "at least" in other
languages.

```{req} Integer Greater Than Or Equal Comparison (Unsigned)
:id: REQ_INT_GE_U_COMPARE
:status: open
:tags: comparison, validation, semantics
The `ge_u` instruction must interpret operands as unsigned integers and return 1 (i32) if the first operand is greater than or equal to the second, 0 otherwise.
```

(floating-point-comparison-instructions)=
### Floating-Point Comparison Instructions

0. [Floating-Point Equality](#floating-point-equality)
0. [Floating-Point Inequality](#floating-point-inequality)
0. [Floating-Point Less Than](#floating-point-less-than)
0. [Floating-Point Less Than Or Equal To](#floating-point-less-than-or-equal-to)
0. [Floating-Point Greater Than](#floating-point-greater-than)
0. [Floating-Point Greater Than Or Equal To](#floating-point-greater-than-or-equal-to)

(floating-point-equality)=
#### Floating-Point Equality

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.eq`    | 0x5b   | `(f32, f32) : (i32)`        | [C], [F] |
| `f64.eq`    | 0x61   | `(f64, f64) : (i32)`        | [C], [F] |

The floating-point `eq` instruction performs the IEEE 754-2008
`compareQuietEqual` operation according to the
[general floating-point rules][F].

> This instruction corresponds to what is sometimes called "ordered and equal",
or "oeq", in other languages.

```{req} Floating-Point Equality Comparison
:id: REQ_FP_EQ_COMPARE
:status: open
:tags: comparison, validation, semantics, floating-point
The `eq` instruction must perform IEEE 754-2008 compareQuietEqual and return 1 (i32) if operands are equal and neither is NaN, 0 otherwise.
```

(floating-point-inequality)=
#### Floating-Point Inequality

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.ne`    | 0x5c   | `(f32, f32) : (i32)`        | [C], [F] |
| `f64.ne`    | 0x62   | `(f64, f64) : (i32)`        | [C], [F] |

The floating-point `ne` instruction performs the IEEE 754-2008
`compareQuietNotEqual` operation according to the
[general floating-point rules][F].

> Unlike the other floating-point comparison instructions, this instruction
returns [true] if either operand is a NaN. It is the logical inverse of the `eq`
instruction.

> This instruction corresponds to what is sometimes called
"unordered or not equal", or "une", in other languages.

```{req} Floating-Point Inequality Comparison
:id: REQ_FP_NE_COMPARE
:status: open
:tags: comparison, validation, semantics, floating-point
The `ne` instruction must perform IEEE 754-2008 compareQuietNotEqual and return 1 (i32) if operands are not equal or either is NaN, 0 otherwise.
```

(floating-point-less-than)=
#### Floating-Point Less Than

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.lt`    | 0x5d   | `(f32, f32) : (i32)`        | [C], [F] |
| `f64.lt`    | 0x63   | `(f64, f64) : (i32)`        | [C], [F] |

The `lt` instruction performs the IEEE 754-2008 `compareQuietLess` operation
according to the [general floating-point rules][F].

> This instruction corresponds to what is sometimes called "ordered and less
than", or "olt", in other languages.

```{req} Floating-Point Less Than Comparison
:id: REQ_FP_LT_COMPARE
:status: open
:tags: comparison, validation, semantics, floating-point
The `lt` instruction must perform IEEE 754-2008 compareQuietLess and return 1 (i32) if first operand is less than second and neither is NaN, 0 otherwise.
```

(floating-point-less-than-or-equal-to)=
#### Floating-Point Less Than Or Equal To

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.le`    | 0x5f   | `(f32, f32) : (i32)`        | [C], [F] |
| `f64.le`    | 0x65   | `(f64, f64) : (i32)`        | [C], [F] |

The `le` instruction performs the IEEE 754-2008 `compareQuietLessEqual`
operation according to the [general floating-point rules][F].

> This instruction corresponds to what is sometimes called "ordered and less
than or equal", or "ole", in other languages.

```{req} Floating-Point Less Than Or Equal Comparison
:id: REQ_FP_LE_COMPARE
:status: open
:tags: comparison, validation, semantics, floating-point
The `le` instruction must perform IEEE 754-2008 compareQuietLessEqual and return 1 (i32) if first operand is less than or equal to second and neither is NaN, 0 otherwise.
```

(floating-point-greater-than)=
#### Floating-Point Greater Than

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.gt`    | 0x5e   | `(f32, f32) : (i32)`        | [C], [F] |
| `f64.gt`    | 0x64   | `(f64, f64) : (i32)`        | [C], [F] |

The `gt` instruction performs the IEEE 754-2008 `compareQuietGreater` operation
according to the [general floating-point rules][F].

> This instruction corresponds to what is sometimes called "ordered and greater
than", or "ogt", in other languages.

```{req} Floating-Point Greater Than Comparison
:id: REQ_FP_GT_COMPARE
:status: open
:tags: comparison, validation, semantics, floating-point
The `gt` instruction must perform IEEE 754-2008 compareQuietGreater and return 1 (i32) if first operand is greater than second and neither is NaN, 0 otherwise.
```

(floating-point-greater-than-or-equal-to)=
#### Floating-Point Greater Than Or Equal To

| Mnemonic    | Opcode | Signature                   | Families |
| ----------- | ------ | --------------------------- | -------- |
| `f32.ge`    | 0x60   | `(f32, f32) : (i32)`        | [C], [F] |
| `f64.ge`    | 0x66   | `(f64, f64) : (i32)`        | [C], [F] |

The `ge` instruction performs the IEEE 754-2008 `compareQuietGreaterEqual`
operation according to the [general floating-point rules][F].

> This instruction corresponds to what is sometimes called "ordered and greater
than or equal", or "oge", in other languages.

```{req} Floating-Point Greater Than Or Equal Comparison
:id: REQ_FP_GE_COMPARE
:status: open
:tags: comparison, validation, semantics, floating-point
The `ge` instruction must perform IEEE 754-2008 compareQuietGreaterEqual and return 1 (i32) if first operand is greater than or equal to second and neither is NaN, 0 otherwise.
```

(conversion-instructions)=
### Conversion Instructions

0. [Integer Wrap](#integer-wrap)
0. [Integer Extend, Signed](#integer-extend-signed)
0. [Integer Extend, Unsigned](#integer-extend-unsigned)
0. [Truncate Floating-Point to Integer, Signed](#truncate-floating-point-to-integer-signed)
0. [Truncate Floating-Point to Integer, Unsigned](#truncate-floating-point-to-integer-unsigned)
0. [Floating-Point Demote](#floating-point-demote)
0. [Floating-Point Promote](#floating-point-promote)
0. [Convert Integer To Floating-Point, Signed](#convert-integer-to-floating-point-signed)
0. [Convert Integer To Floating-Point, Unsigned](#convert-integer-to-floating-point-unsigned)
0. [Reinterpret](#reinterpret)
0. [Narrow-Width Integer Sign Extension](#narrow-width-integer-sign-extension)

(integer-wrap)=
#### Integer Wrap

| Mnemonic       | Opcode | Signature                | Families |
| -------------- | ------ | ------------------------ | -------- |
| `i32.wrap_i64` | 0xa7   | `(i64) : (i32)`          | [G]      |

The `wrap` instruction returns the value of its operand silently wrapped to its
result type. Wrapping means reducing the value modulo the number of unique
values in the result type.

> This instruction corresponds to what is sometimes called an integer "truncate"
in other languages, however WebAssembly uses the word "truncate" to mean
effectively discarding the least significant digits, and the word "wrap" to mean
effectively discarding the most significant digits.

```{req} Integer Wrap Conversion
:id: REQ_INT_WRAP_REDUCE
:status: open
:tags: conversion, validation, semantics, arithmetic
The `wrap` instruction must reduce the operand value modulo the number of unique values in the result type (2^result_width).
```

(integer-extend-signed)=
#### Integer Extend, Signed

| Mnemonic           | Opcode | Signature            | Families |
| ------------------ | ------ | -------------------- | -------- |
| `i64.extend_i32_s` | 0xac   | `(i32) : (i64)`      | [S]      |

The `extend_i32_s` instruction returns the value of its operand [sign-extended] to
its result type.

```{req} Integer Sign Extension
:id: REQ_INT_EXTEND_S_SIGN_EXTEND
:status: open
:tags: conversion, validation, semantics, arithmetic
The `extend_i32_s` instruction must sign-extend the operand to the result type, preserving the sign bit and all significant bits.
```

(integer-extend-unsigned)=
#### Integer Extend, Unsigned

| Mnemonic           | Opcode | Signature            | Families |
| ------------------ | ------ | -------------------- | -------- |
| `i64.extend_i32_u` | 0xad   | `(i32) : (i64)`      | [U]      |

The `extend_i32_u` instruction returns the value of its operand zero-extended to its
result type.

```{req} Integer Zero Extension
:id: REQ_INT_EXTEND_U_ZERO_EXTEND
:status: open
:tags: conversion, validation, semantics, arithmetic
The `extend_i32_u` instruction must zero-extend the operand to the result type, filling high-order bits with zeros.
```

(truncate-floating-point-to-integer-signed)=
#### Truncate Floating-Point to Integer, Signed

| Mnemonic          | Opcode | Signature             | Families |
| ----------------- | ------ | --------------------- | -------- |
| `i32.trunc_f32_s` | 0xa8   | `(f32) : (i32)`       | [F], [S] |
| `i32.trunc_f64_s` | 0xaa   | `(f64) : (i32)`       | [F], [S] |
| `i64.trunc_f32_s` | 0xae   | `(f32) : (i64)`       | [F], [S] |
| `i64.trunc_f64_s` | 0xb0   | `(f64) : (i64)`       | [F], [S] |

The `trunc_s` instruction performs the IEEE 754-2008
`convertToIntegerTowardZero` operation, with the result value interpreted as
signed, according to the [general floating-point rules][F].

**Trap:** Invalid Conversion To Integer, when a floating-point Invalid condition
occurs, due to the operand being outside the range that can be converted
(including NaN values and infinities).

> This form of rounding is called a `chop` in other languages.

```{req} Truncate Floating-Point to Signed Integer
:id: REQ_TRUNC_FP_S_CONVERT
:status: open
:tags: conversion, validation, semantics, floating-point
The `trunc_s` instruction must perform IEEE 754-2008 convertToIntegerTowardZero and trap if operand is outside representable range or is NaN.
```

```{req} Truncate Floating-Point to Signed Integer Result
:id: REQ_TRUNC_FP_S_SIGNED
:status: open
:tags: conversion, validation, semantics, floating-point
The `trunc_s` instruction must interpret the result value as signed, rounding toward zero.
```

(truncate-floating-point-to-integer-unsigned)=
#### Truncate Floating-Point to Integer, Unsigned

| Mnemonic          | Opcode | Signature             | Families |
| ----------------- | ------ | --------------------- | -------- |
| `i32.trunc_f32_u` | 0xa9   | `(f32) : (i32)`       | [F], [U] |
| `i32.trunc_f64_u` | 0xab   | `(f64) : (i32)`       | [F], [U] |
| `i64.trunc_f32_u` | 0xaf   | `(f32) : (i64)`       | [F], [U] |
| `i64.trunc_f64_u` | 0xb1   | `(f64) : (i64)`       | [F], [U] |

The `trunc_u` instruction performs the IEEE 754-2008
`convertToIntegerTowardZero` operation, with the result value interpreted as
unsigned, according to the [general floating-point rules][F].

**Trap:** Invalid Conversion To Integer, when an Invalid condition occurs, due
to the operand being outside the range that can be converted (including NaN
values and infinities).

> This instruction's result is unsigned, so it almost always rounds down,
however it does round up in one place: negative values greater than negative one
truncate up to zero.

```{req} Truncate Floating-Point to Unsigned Integer
:id: REQ_TRUNC_FP_U_CONVERT
:status: open
:tags: conversion, validation, semantics, floating-point
The `trunc_u` instruction must perform IEEE 754-2008 convertToIntegerTowardZero and trap if operand is outside representable range or is NaN.
```

```{req} Truncate Floating-Point to Unsigned Integer Result
:id: REQ_TRUNC_FP_U_UNSIGNED
:status: open
:tags: conversion, validation, semantics, floating-point
The `trunc_u` instruction must interpret the result value as unsigned, rounding toward zero.
```

(floating-point-demote)=
#### Floating-Point Demote

| Mnemonic         | Opcode | Signature              | Families |
| ---------------- | ------ | ---------------------- | -------- |
| `f32.demote_f64` | 0xb6   | `(f64) : (f32)`        | [F]      |

The `demote` instruction performs the IEEE 754-2008 `convertFormat` operation,
converting from its operand type to its result type, according to the
[general floating-point rules][F].

> This is a narrowing conversion which may round or overflow to infinity.

```{req} Floating-Point Demote Conversion
:id: REQ_FP_DEMOTE_CONVERT
:status: open
:tags: conversion, validation, semantics, floating-point
The `demote` instruction must perform IEEE 754-2008 convertFormat from f64 to f32, which may round or overflow to infinity.
```

(floating-point-promote)=
#### Floating-Point Promote

| Mnemonic          | Opcode | Signature             | Families |
| ----------------- | ------ | --------------------- | -------- |
| `f64.promote_f32` | 0xbb   | `(f32) : (f64)`       | [F]      |

The `promote` instruction performs the IEEE 754-2008 `convertFormat` operation,
converting from its operand type to its result type, according to the
[general floating-point rules][F].

> This is a widening conversion and is always exact.

```{req} Floating-Point Promote Conversion
:id: REQ_FP_PROMOTE_CONVERT
:status: open
:tags: conversion, validation, semantics, floating-point
The `promote` instruction must perform IEEE 754-2008 convertFormat from f32 to f64, which is always exact.
```

(convert-integer-to-floating-point-signed)=
#### Convert Integer To Floating-Point, Signed

| Mnemonic            | Opcode | Signature           | Families |
| ------------------- | ------ | ------------------- | -------- |
| `f32.convert_i32_s` | 0xb2   | `(i32) : (f32)`     | [F], [S] |
| `f32.convert_i64_s` | 0xb4   | `(i64) : (f32)`     | [F], [S] |
| `f64.convert_i32_s` | 0xb7   | `(i32) : (f64)`     | [F], [S] |
| `f64.convert_i64_s` | 0xb9   | `(i64) : (f64)`     | [F], [S] |

The `convert_s` instruction performs the IEEE 754-2008 `convertFromInt`
operation, with its operand value interpreted as signed, according to the
[general floating-point rules][F].

> `f64.convert_i32_s` is always exact; the other instructions here may round.

```{req} Convert Signed Integer to Floating-Point
:id: REQ_CONVERT_INT_S_TO_FP
:status: open
:tags: conversion, validation, semantics, floating-point
The `convert_s` instruction must perform IEEE 754-2008 convertFromInt with the operand interpreted as signed, may round except f64.convert_i32_s which is exact.
```

(convert-integer-to-floating-point-unsigned)=
#### Convert Integer To Floating-Point, Unsigned

| Mnemonic            | Opcode | Signature           | Families |
| ------------------- | ------ | ------------------- | -------- |
| `f32.convert_i32_u` | 0xb3   | `(i32) : (f32)`     | [F], [U] |
| `f32.convert_i64_u` | 0xb5   | `(i64) : (f32)`     | [F], [U] |
| `f64.convert_i32_u` | 0xb8   | `(i32) : (f64)`     | [F], [U] |
| `f64.convert_i64_u` | 0xba   | `(i64) : (f64)`     | [F], [U] |

The `convert_u` instruction performs the IEEE 754-2008 `convertFromInt`
operation, with its operand value interpreted as unsigned, according to the
[general floating-point rules][F].

> `f64.convert_i32_u` is always exact; the other instructions here may round.

```{req} Convert Unsigned Integer to Floating-Point
:id: REQ_CONVERT_INT_U_TO_FP
:status: open
:tags: conversion, validation, semantics, floating-point
The `convert_u` instruction must perform IEEE 754-2008 convertFromInt with the operand interpreted as unsigned, may round except f64.convert_i32_u which is exact.
```

(reinterpret)=
#### Reinterpret

| Mnemonic              | Opcode | Signature         | Families |
| --------------------- | ------ | ----------------- | -------- |
| `i32.reinterpret_f32` | 0xbc   | `(f32) : (i32)`   |          |
| `i64.reinterpret_f64` | 0xbd   | `(f64) : (i64)`   |          |
| `f32.reinterpret_i32` | 0xbe   | `(i32) : (f32)`   |          |
| `f64.reinterpret_i64` | 0xbf   | `(i64) : (f64)`   |          |

The `reinterpret` instruction returns a value which has the same bit-pattern as
its operand value, in its result type.

> The operand type is always the same width as the result type, so this
instruction is always exact.

```{req} Reinterpret Bit Pattern
:id: REQ_REINTERPRET_BITPATTERN
:status: open
:tags: conversion, validation, semantics
The `reinterpret` instruction must return a value with the same bit-pattern as the operand in the result type without interpretation.
```

```{req} Reinterpret Type Equivalence
:id: REQ_REINTERPRET_WIDTH_SAME
:status: open
:tags: conversion, validation, semantics
The `reinterpret` instruction operand and result types must be the same width, making the conversion always exact.
```

(narrow-width-integer-sign-extension)=
#### Narrow-Width Integer Sign Extension

| Mnemonic           | Opcode | Signature            | Families |
| ------------------ | ------ | -------------------- | -------- |
| `i32.extend8_s`    | 0xc0   | `(i32) : (i32)`      | [S]      |
| `i32.extend16_s`   | 0xc1   | `(i32) : (i32)`      | [S]      |
|                    |        |                      |          |
| `i64.extend8_s`    | 0xc2   | `(i64) : (i64)`      | [S]      |
| `i64.extend16_s`   | 0xc3   | `(i64) : (i64)`      | [S]      |
| `i64.extend32_s`   | 0xc4   | `(i64) : (i64)`      | [S]      |

The `extend_s` instruction interprets its operand as the corresponding signed
narrower-width type and returns its value [sign-extended] to its full width.
 - `extend8_s` extends the 8 least significant bits to the full width of its
type.
 - `extend16_s` extends the 16 least significant bits to the full width of its
 type.
 - `extend32_s` extends the 32 least significant bits to the full width of its
 type.

```{req} Narrow-Width Integer Sign Extension
:id: REQ_EXTEND_NARROW_SIGN_EXTEND
:status: open
:tags: conversion, validation, semantics, arithmetic
The `extend_s` instruction must interpret the operand as a narrower-width signed type and sign-extend it to the full width.
```

```{req} Extend8 Sign Extension
:id: REQ_EXTEND8_S_SIGEXT
:status: open
:tags: conversion, validation, semantics, arithmetic
The `extend8_s` instruction must sign-extend the 8 least significant bits to the full width of the type.
```

```{req} Extend16 Sign Extension
:id: REQ_EXTEND16_S_SIGEXT
:status: open
:tags: conversion, validation, semantics, arithmetic
The `extend16_s` instruction must sign-extend the 16 least significant bits to the full width of the type.
```

```{req} Extend32 Sign Extension
:id: REQ_EXTEND32_S_SIGEXT
:status: open
:tags: conversion, validation, semantics, arithmetic
The `extend32_s` instruction must sign-extend the 32 least significant bits to the full width of the i64 type.
```

(load-and-store-instructions)=
### Load And Store Instructions

0. [Load](#load)
0. [Store](#store)
0. [Extending Load, Signed](#extending-load-signed)
0. [Extending Load, Unsigned](#extending-load-unsigned)
0. [Wrapping Store](#wrapping-store)

(load)=
#### Load

| Mnemonic    | Opcode | Immediates                                 | Signature               | Families |
| ----------- | ------ | ------------------------------------------ | ----------------------- | -------- |
| `i32.load`  | 0x28   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i32)` | [M], [G] |
| `i64.load`  | 0x29   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i64)` | [M], [G] |
| `f32.load`  | 0x2a   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (f32)` | [M], [E] |
| `f64.load`  | 0x2b   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (f64)` | [M], [E] |

The `load` instruction performs a [load](#loading) of the same size as its type
from the [default linear memory].

Floating-point loads preserve all the bits of the value, performing an
IEEE 754-2008 `copy` operation.

```{req} Load Instruction Must Read From Memory
:id: REQ_LOAD_READ_FROM_MEMORY
:status: open
:tags: semantics, load-instructions
The `load` instruction must read a value from the default linear memory at the effective address (base + offset) and push the loaded value onto the value stack.
```

```{req} Load Size Must Match Instruction Type
:id: REQ_LOAD_SIZE_MATCH
:status: open
:tags: semantics, load-instructions
The load must read bytes equal to the size of the instruction type (i32.load reads 4 bytes, i64.load reads 8 bytes, etc.).
```

```{req} Load Must Check Memory Access Bounds
:id: REQ_LOAD_BOUNDS_CHECK
:status: open
:tags: validation, load-instructions
The `load` instruction must validate that all accessed bytes are within the bounds of the default linear memory, trapping if any access is out of bounds.
```

**Validation:**
 - [Linear-memory access validation] is required.

```{req} Load Instruction Must Apply Linear Memory Access Validation
:id: REQ_LOAD_MEMORY_VALIDATION
:status: open
:tags: validation, load-instructions
All linear-memory access validation requirements must be applied to load instructions, including alignment and default memory checks.
```

(store)=
#### Store

| Mnemonic    | Opcode | Immediates                                 | Signature                         | Families |
| ----------- | ------ | ------------------------------------------ | --------------------------------- | -------- |
| `i32.store` | 0x36   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR, $value: i32) : ()` | [M], [G] |
| `i64.store` | 0x37   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR, $value: i64) : ()` | [M], [G] |
| `f32.store` | 0x38   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR, $value: f32) : ()` | [M], [F] |
| `f64.store` | 0x39   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR, $value: f64) : ()` | [M], [F] |

The `store` instruction performs a [store](#storing) of `$value` of the same
size as its type to the [default linear memory].

Floating-point stores preserve all the bits of the value, performing an
IEEE 754-2008 `copy` operation.

```{req} Store Instruction Must Write To Memory
:id: REQ_STORE_WRITE_TO_MEMORY
:status: open
:tags: semantics, store-instructions
The `store` instruction must write the operand value to the default linear memory at the effective address (base + offset).
```

```{req} Store Size Must Match Instruction Type
:id: REQ_STORE_SIZE_MATCH
:status: open
:tags: semantics, store-instructions
The store must write bytes equal to the size of the instruction type (i32.store writes 4 bytes, i64.store writes 8 bytes, etc.).
```

```{req} Store Must Check Memory Access Bounds
:id: REQ_STORE_BOUNDS_CHECK
:status: open
:tags: validation, store-instructions
The `store` instruction must validate that all accessed bytes are within the bounds of the default linear memory, trapping before writing if any access is out of bounds.
```

```{req} Store Must Not Return Value
:id: REQ_STORE_NO_RETURN
:status: open
:tags: semantics, store-instructions
The `store` instruction must not push a value onto the value stack; it only modifies memory.
```

**Validation:**
 - [Linear-memory access validation] is required.

```{req} Store Instruction Must Apply Linear Memory Access Validation
:id: REQ_STORE_MEMORY_VALIDATION
:status: open
:tags: validation, store-instructions
All linear-memory access validation requirements must be applied to store instructions, including alignment and default memory checks.
```

(extending-load-signed)=
#### Extending Load, Signed

| Mnemonic       | Opcode | Immediates                                 | Signature               | Families |
| -------------- | ------ | ------------------------------------------ | ----------------------- | -------- |
| `i32.load8_s`  | 0x2c   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i32)` | [M], [S] |
| `i32.load16_s` | 0x2e   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i32)` | [M], [S] |
|                |        |                                            |                         |          |
| `i64.load8_s`  | 0x30   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i64)` | [M], [S] |
| `i64.load16_s` | 0x32   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i64)` | [M], [S] |
| `i64.load32_s` | 0x34   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i64)` | [M], [S] |

The signed extending load instructions perform a [load](#loading) of narrower
width than their type from the [default linear memory], and return the value
[sign-extended] to their type.
 - `load8_s` loads an 8-bit value.
 - `load16_s` loads a 16-bit value.
 - `load32_s` loads a 32-bit value.

```{req} Signed Extending Load Must Read Narrower Value
:id: REQ_LOAD_SIGNED_EXTEND_READ
:status: open
:tags: semantics, load-instructions
The signed extending load instruction must read a value narrower than its result type from memory (8, 16, or 32 bits).
```

```{req} Signed Extending Load Must Sign Extend
:id: REQ_LOAD_SIGNED_EXTEND
:status: open
:tags: semantics, load-instructions
The signed extending load instruction must sign-extend the loaded narrow value to the full result type, preserving the sign bit.
```

```{req} Signed Extending Load Must Check Bounds
:id: REQ_LOAD_SIGNED_EXTEND_BOUNDS
:status: open
:tags: validation, load-instructions
The signed extending load instruction must validate that all accessed bytes are within the bounds of default linear memory, trapping if any access is out of bounds.
```

**Validation:**
 - [Linear-memory access validation] is required.

```{req} Signed Extending Load Instruction Must Apply Validation
:id: REQ_LOAD_SIGNED_EXTEND_VALIDATION
:status: open
:tags: validation, load-instructions
All linear-memory access validation requirements must be applied to signed extending load instructions.
```

(extending-load-unsigned)=
#### Extending Load, Unsigned

| Mnemonic       | Opcode | Immediates                                 | Signature               | Families |
| -------------- | ------ | ------------------------------------------ | ----------------------- | -------- |
| `i32.load8_u`  | 0x2d   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i32)` | [M], [U] |
| `i32.load16_u` | 0x2f   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i32)` | [M], [U] |
|                |        |                                            |                         |          |
| `i64.load8_u`  | 0x31   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i64)` | [M], [U] |
| `i64.load16_u` | 0x33   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i64)` | [M], [U] |
| `i64.load32_u` | 0x35   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR) : (i64)` | [M], [U] |

The unsigned extending load instructions perform a [load](#loading) of narrower
width than their type from the [default linear memory], and return the value
zero-extended to their type.
 - `load8_u` loads an 8-bit value.
 - `load16_u` loads a 16-bit value.
 - `load32_u` loads a 32-bit value.

```{req} Unsigned Extending Load Must Read Narrower Value
:id: REQ_LOAD_UNSIGNED_EXTEND_READ
:status: open
:tags: semantics, load-instructions
The unsigned extending load instruction must read a value narrower than its result type from memory (8, 16, or 32 bits).
```

```{req} Unsigned Extending Load Must Zero Extend
:id: REQ_LOAD_UNSIGNED_EXTEND
:status: open
:tags: semantics, load-instructions
The unsigned extending load instruction must zero-extend the loaded narrow value to the full result type, filling upper bits with zeros.
```

```{req} Unsigned Extending Load Must Check Bounds
:id: REQ_LOAD_UNSIGNED_EXTEND_BOUNDS
:status: open
:tags: validation, load-instructions
The unsigned extending load instruction must validate that all accessed bytes are within the bounds of default linear memory, trapping if any access is out of bounds.
```

**Validation:**
 - [Linear-memory access validation] is required.

```{req} Unsigned Extending Load Instruction Must Apply Validation
:id: REQ_LOAD_UNSIGNED_EXTEND_VALIDATION
:status: open
:tags: validation, load-instructions
All linear-memory access validation requirements must be applied to unsigned extending load instructions.
```

(wrapping-store)=
#### Wrapping Store

| Mnemonic      | Opcode | Immediates                                 | Signature                         | Families |
| ------------- | ------ | ------------------------------------------ | --------------------------------- | -------- |
| `i32.store8`  | 0x3a   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR, $value: i32) : ()` | [M], [G] |
| `i32.store16` | 0x3b   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR, $value: i32) : ()` | [M], [G] |
|               |        |                                            |                                   |          |
| `i64.store8`  | 0x3c   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR, $value: i64) : ()` | [M], [G] |
| `i64.store16` | 0x3d   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR, $value: i64) : ()` | [M], [G] |
| `i64.store32` | 0x3e   | `$flags`: [memflags], `$offset`: [varuPTR] | `($base: iPTR, $value: i64) : ()` | [M], [G] |

The wrapping store instructions performs a [store](#storing) of `$value` to the
[default linear memory], silently wrapped to a narrower width.
 - `store8` stores an 8-bit value.
 - `store16` stores a 16-bit value.
 - `store32` stores a 32-bit value.

```{req} Wrapping Store Must Write Wrapped Value
:id: REQ_STORE_WRAPPING_WRITE
:status: open
:tags: semantics, store-instructions
The wrapping store instruction must write the operand value to memory after silently wrapping it to the narrower width.
```

```{req} Wrapping Store Must Wrap To Narrow Width
:id: REQ_STORE_WRAP_TO_WIDTH
:status: open
:tags: semantics, store-instructions
The wrapping store instruction must discard high-order bits and write only the low-order bits that fit in the narrow type (8, 16, or 32 bits).
```

```{req} Wrapping Store Must Check Bounds
:id: REQ_STORE_WRAPPING_BOUNDS
:status: open
:tags: validation, store-instructions
The wrapping store instruction must validate that all accessed bytes are within the bounds of default linear memory, trapping before writing if any access is out of bounds.
```

```{req} Wrapping Store Must Not Return Value
:id: REQ_STORE_WRAPPING_NO_RETURN
:status: open
:tags: semantics, store-instructions
The wrapping store instruction must not push a value onto the value stack; it only modifies memory.
```

**Validation:**
 - [Linear-memory access validation] is required.

```{req} Wrapping Store Instruction Must Apply Validation
:id: REQ_STORE_WRAPPING_VALIDATION
:status: open
:tags: validation, store-instructions
All linear-memory access validation requirements must be applied to wrapping store instructions.
```

> See the comment in the [wrap instruction](#integer-wrap) about the meaning of
the name "wrap".

(additional-memory-related-instructions)=
### Additional Memory-Related Instructions

0. [Grow Linear-Memory Size](#grow-linear-memory-size)
0. [Current Linear-Memory Size](#current-linear-memory-size)

(grow-linear-memory-size)=
#### Grow Linear-Memory Size

| Mnemonic      | Opcode | Immediates              | Signature                 | Families |
| ------------- | ------ | ----------------------- | ------------------------- | -------- |
| `memory.grow` | 0x40   | `$reserved`: [varuint1] | `($delta: iPTR) : (iPTR)` | [Z]      |

The `memory.grow` instruction increases the size of the [default linear memory]
by `$delta`, in units of unsigned [pages]. If the index of any byte of the
referenced linear memory would be unrepresentable as unsigned in an `iPTR`, if
allocation fails due to insufficient dynamic resources, or if the linear memory
has a `maximum` length and the actual size would exceed the `maximum` length, it
returns `-1` and the linear-memory size is not increased; otherwise the
linear-memory size is increased, and `memory.grow` returns the previous
linear-memory size, also as an unsigned value in units of [pages]. Newly
allocated bytes are initialized to all zeros.

```{req} Memory Grow Must Increase Linear Memory Size
:id: REQ_MEMORY_GROW_INCREASE_SIZE
:status: open
:tags: semantics, memory-instructions
The `memory.grow` instruction must increase the size of the default linear memory by the number of pages specified in `$delta`.
```

```{req} Memory Grow Must Return Previous Size
:id: REQ_MEMORY_GROW_RETURN_PREVIOUS
:status: open
:tags: semantics, memory-instructions
If memory growth succeeds, `memory.grow` must return the previous memory size in pages; if it fails, it must return -1 (0xFFFFFFFF for i32, etc.).
```

```{req} Memory Grow Must Initialize New Bytes To Zero
:id: REQ_MEMORY_GROW_ZERO_INITIALIZE
:status: open
:tags: semantics, memory-instructions
All newly allocated bytes from `memory.grow` must be initialized to zero.
```

```{req} Memory Grow Must Respect Maximum Size
:id: REQ_MEMORY_GROW_MAX_CONSTRAINT
:status: open
:tags: validation, memory-instructions
If the default linear memory has a declared maximum size, `memory.grow` must not increase the size beyond that maximum; it must return -1 instead.
```

```{req} Memory Grow Must Check Address Representability
:id: REQ_MEMORY_GROW_ADDRESS_REPRESENTABILITY
:status: open
:tags: validation, memory-instructions
If any byte index in the grown memory would be unrepresentable as an unsigned iPTR, `memory.grow` must fail and return -1.
```

```{req} Memory Grow Must Handle Resource Exhaustion
:id: REQ_MEMORY_GROW_RESOURCE_EXHAUSTION
:status: open
:tags: validation, memory-instructions
If allocation fails due to insufficient dynamic resources, `memory.grow` must return -1 and the memory size must not change.
```

**Validation**:
 - [Linear-memory size validation](#linear-memory-size-validation) is required.
 - `$reserved` is required to be `0`.

```{req} Memory Grow Reserved Field Must Be Zero
:id: REQ_MEMORY_GROW_RESERVED_ZERO
:status: open
:tags: validation, memory-instructions
The `$reserved` field of the `memory.grow` instruction must be `0`.
```

> This instruction can fail even when the `maximum` length isn't yet reached,
due to resource exhaustion.

> Since the return value is in units of pages, `-1` isn't otherwise a valid
linear-memory size. Also, note that `-1` isn't the only "negative" value (when
interpreted as signed) that can be returned; other such values can indicate
valid returns.

> `$reserved` is intended for future use.

> This instruction was previously named `grow_memory`, and was briefly proposed
to be named `mem.grow`.

(current-linear-memory-size)=
#### Current Linear-Memory Size

| Mnemonic      | Opcode | Immediates              | Signature              | Families |
| ------------- | ------ | ----------------------- | ---------------------- | -------- |
| `memory.size` | 0x3f   | `$reserved`: [varuint1] | `() : (iPTR)`          | [Z]      |

The `memory.size` instruction returns the size of the [default linear memory],
as an unsigned value in units of [pages].

```{req} Memory Size Must Return Current Memory Size
:id: REQ_MEMORY_SIZE_RETURN_SIZE
:status: open
:tags: semantics, memory-instructions
The `memory.size` instruction must push the current size of the default linear memory (in pages) onto the value stack as an unsigned iPTR value.
```

```{req} Memory Size Returns Pages Not Bytes
:id: REQ_MEMORY_SIZE_UNIT_PAGES
:status: open
:tags: semantics, memory-instructions
The `memory.size` instruction must return the memory size in units of pages (64 KiB), not in bytes.
```

```{req} Memory Size Must Not Modify State
:id: REQ_MEMORY_SIZE_NO_SIDE_EFFECTS
:status: open
:tags: semantics, memory-instructions
The `memory.size` instruction must not modify any state; it is a read-only query of the current memory size.
```

**Validation**:
 - [Linear-memory size validation](#linear-memory-size-validation) is required.
 - `$reserved` is required to be `0`.

```{req} Memory Size Reserved Field Must Be Zero
:id: REQ_MEMORY_SIZE_RESERVED_ZERO
:status: open
:tags: validation, memory-instructions
The `$reserved` field of the `memory.size` instruction must be `0`.
```

> `$reserved` is intended for future use.

> This instruction was previously named `current_memory` and was briefly
proposed to be named `mem.size`.

(instantiation)=
Instantiation
--------------------------------------------------------------------------------

WebAssembly code execution requires an *instance* of a module, which contains a
reference to the module plus additional information added during instantiation,
which consists of the following steps:
 - The entire module is first validated according to the requirements of the
   **Validation** clause of the [top-level module description](#module-contents)
   and all clauses it transitively requires. If there are any failures,
   instantiation aborts and doesn't produce an instance.
 - If a [Linear-Memory Section] is present, each linear memory is
   [instantiated](#linear-memory-instantiation).
 - If a [Table Section] is present, each table is
   [instantiated](#table-instantiation).
 - A finite quantity of [call-stack resources] is allocated.
 - A *globals vector* is allocated, which is a heterogeneous vector of globals
   with an element for each entry in the module's [Global Section], if present.
   The initial value of each global is the value of its
   [instantiation-time initializer](#instantiation-time-initializers), if it has
   one, or an all-zeros bit-pattern otherwise.

**Trap:** Dynamic Resource Exhaustion, if dynamic resources are insufficient to
support creation of the module instance or any of its components.

> The contents of an instance, including functions and their bodies, are outside
any linear-memory address space and not any accessible to applications.
WebAssembly is therefore conceptually a [Harvard Architecture].

[Harvard Architecture]: https://en.wikipedia.org/wiki/Harvard_architecture

(linear-memory-instantiation)=
### Linear-Memory Instantiation

A linear memory is instantiated as follows:

For a linear-memory definition in the [Linear-Memory Section], as opposed to a
[linear-memory import](#import-section), a vector of [bytes] with the length
being the value of the linear memory's `minimum` length field times the
[page size] is created, added to the instance, and initialized to all zeros. For
a linear-memory import, storage for the vector is already allocated.

For each [Data Section] entry with and `index` value equal to the index of the
linear memory, the contents of its `data` field are copied into the linear
memory starting at its `offset` field.

**Trap:** Dynamic Resource Exhaustion, if dynamic resources are insufficient to
support creation of any of the vectors.

(table-instantiation)=
### Table Instantiation

A table is instantiated as follows:

For a table definition in the [Table Section], as opposed to a
[table import](#import-section), a vector of elements is created with the
table's `minimum` length, with elements of the table's element type, and
initialized to all special "null" values specific to the element type. For a
table import, storage for the table is already allocated.

For each table initializer in the [Element Section], for the table identified by
the table index in the [table index space]:
 - A contiguous of elements in the table starting at the table initializer's
   start offset is initialized according to the elements of the table element
   initializers array, which specify an indexed element in their selected index
   space.

**Trap:** Dynamic Resource Exhaustion, if dynamic resources are insufficient to
support creation of any of the tables.

(call-stack-resources)=
### Call-Stack Resources

Call-stack resources are an abstract quantity, with discrete units, of which a
[nondeterministic] amount is allocated during instantiation, belonging to an
instance.

> These resources is used by [call instructions][L].

> The specific resource limit serves as an upper bound only; implementations may
[nondeterministically] perform a trap sooner if they exhaust other dynamic
resources.

(execution)=
Execution
--------------------------------------------------------------------------------

0. [Instance Execution](#instance-execution)
0. [Function Execution](#function-execution)

(instance-execution)=
### Instance Execution

If the module contains a [Start Section], the referenced function is
[executed](#function-execution).

(function-execution)=
### Function Execution

TODO: This section should be improved to be more approachable.

Function execution can be prompted by a [call-family instruction][L] from within
the same module, by [instance execution](#instance-execution), or by a call to
an [exported](#export-section) function from another module or from the
[embedding environment].

The input to execution of a function consists of:
 - the function to be executed.
 - the incoming argument values, one for each parameter [type] of the function.
 - a module instance

For the duration of the execution of a function body, several data structures
are created:
 - A *control-flow stack*, with each entry containing
    - a [label] for reference from branch instructions.
    - a *limit* integer value, which is an index into the value stack indicating
      where to reset it to on a branch to that label.
    - a *signature*, which is a [block signature type] indicating the number and
      types of result values of the region.
 - A *value stack*, which carries values between instructions.
 - A *locals* vector, a heterogeneous vector of values containing an element for
   each type in the function's parameter list, followed by an element for each
   local declaration in the function.
 - A *current position*.

> Implementations needn't create a literal vector to store the locals, or
literal stacks to manage values at execution time.

> These data structures are all allocated outside any linear-memory address
space and are not any accessible to applications.

(function-execution-initialization)=
#### Function Execution Initialization

The current position starts at the first instruction in the function body. The
value stack begins empty. The control-flow stack begins with an entry holding a
[label] bound to the last instruction in the instruction sequence, a limit value
of zero, and a signature corresponding to the function's return types:
 - If the function's return type sequence is empty, its signature is `void`.
 - If the function's return type sequence has exactly one element, the signature
   is that element.

The value of each incoming argument is copied to the local with the
corresponding index, and the rest of the locals are initialized to all-zeros
bit-pattern values.

(function-body-execution)=
#### Function-Body Execution

The instruction at the current position is remembered, and the current position
is incremented to point to the position following it. Then the remembered
instruction is executed as follows:

For each operand [type] in the instruction's signature in reverse order, a value
is popped from the value stack and provided as the corresponding operand value.
The instruction is then executed as described in the
[Instruction](#instructions) description for it. Each of the
instruction's return values are then pushed onto the value stack.

If the current position is now past the end of the sequence,
[function return execution](#function-return-execution) is initiated and
execution of the function is thereafter complete.

Otherwise, [execution](#function-body-execution) is restarted with the new
current position.

**Trap:** Dynamic Resource Exhaustion, if any dynamic resource used by the
implementation is exhausted, at any point during function-body execution.

(labels)=
#### Labels

A label is a value which is either *unbound*, or *bound* to a specific position.

(function-return-execution)=
#### Function Return Execution

One value for each return [type] in the function signature in reverse order is
popped from the value stack. If the function execution was prompted by a
[call instruction][L], these values are provided as the call's return values.
Otherwise, they are provided to the [embedding environment].

(instruction-traps)=
#### Instruction Traps

Instructions may *trap*, in which case the function execution which encountered
the trap is immediately terminated. If the function execution was prompted by a
[call instruction][L], it traps too. Otherwise, abnormal termination is reported
to the [embedding environment].

> Except for the call stack and the state of executing functions, the contents
of an instance, including any linear memories, are left intact after a trap.
This allows inspection by debugging tools and crash reporters. It is also valid
to call exported functions in an instance that has seen a trap.

(text-format)=
Text Format
--------------------------------------------------------------------------------

TODO: Describe the text format.


[B]: #b-branch-instruction-family
[Q]: #q-control-flow-barrier-instruction-family
[L]: #l-call-instruction-family
[G]: #g-generic-integer-instruction-family
[S]: #s-signed-integer-instruction-family
[U]: #u-unsigned-integer-instruction-family
[T]: #t-shift-instruction-family
[R]: #r-remainder-instruction-family
[F]: #f-floating-point-instruction-family
[E]: #e-floating-point-bitwise-instruction-family
[C]: #c-comparison-instruction-family
[M]: #m-linear-memory-access-instruction-family
[Z]: #z-linear-memory-size-instruction-family
[Type Section]: #type-section
[Import Section]: #import-section
[Function Section]: #function-section
[Table Section]: #table-section
[Linear-Memory Section]: #linear-memory-section
[Export Section]: #export-section
[Start Section]: #start-section
[Code Section]: #code-section
[Data Section]: #data-section
[Global Section]: #global-section
[Element Section]: #element-section
[Name Section]: #name-section
[function index space]: #function-index-space
[global index space]: #global-index-space
[linear-memory index space]: #linear-memory-index-space
[table index space]: #table-index-space
[accessed bytes]: #accessed-bytes
[array]: #array
[binary32]: https://en.wikipedia.org/wiki/Single-precision_floating-point_format
[binary64]: https://en.wikipedia.org/wiki/Double-precision_floating-point_format
[bit]: https://en.wikipedia.org/wiki/Bit
[block signature type]: #block-signature-types
[boolean]: #booleans
[byte]: #bytes
[bytes]: #bytes
[byte array]: #byte-array
[call-stack resources]: #call-stack-resources
[default linear memory]: #default-linear-memory
[effective address]: #effective-address
[embedding environment]: #embedding-environment
[embedding environments]: #embedding-environment
[encoding type]: #encoding-types
[encoding types]: #encoding-types
[exported]: #export-section
[external kind]: #external-kinds
[false]: #booleans
[function execution]: #function-execution
[global description]: #global-description
[Floor and Ceiling Functions]: https://en.wikipedia.org/wiki/Floor_and_ceiling_functions
[identifier]: #identifier
[identifiers]: #identifier
[imported]: #import-section
[index space]: #module-index-spaces
[instantiation-time initializer]: #instantiation-time-initializers
[instruction]: #instructions
[instructions]: #instructions
[varuPTR]: #varuptr-immediate-type
[KiB]: https://en.wikipedia.org/wiki/Kibibyte
[known section]: #known-sections
[label]: #labels
[labels]: #labels
[language type]: #language-types
[linear memory]: #linear-memories
[linear memories]: #linear-memories
[linear-memory]: #linear-memories
[linear-memory access validation]: #linear-memory-access-validation
[linear-memory description]: #linear-memory-description
[linear-memory descriptions]: #linear-memory-description
[little-endian byte order]: https://en.wikipedia.org/wiki/Endianness#Little-endian
[memflags]: #memflags-immediate-type
[minimum signed integer value]: https://en.wikipedia.org/wiki/Two%27s_complement#Most_negative_number
[nondeterministic]: #nondeterminism
[nondeterministically]: #nondeterminism
[page]: #pages
[pages]: #pages
[page size]: #pages
[resizable limits]: #resizable-limits
[rotated]: https://en.wikipedia.org/wiki/Bitwise_operation#Rotate_no_carry
[shift count]: #shift-count
[shifted]: https://en.wikipedia.org/wiki/Logical_shift
[sign-extended]: https://en.wikipedia.org/wiki/Sign_extension
[signature type]: #signature-types
[table]: #tables
[tables]: #tables
[table element type]: #table-element-types
[table description]: #table-description
[table descriptions]: #table-description
[text form]: #text-format
[true]: #booleans
[type]: #value-types
[types]: #value-types
[typed]: #value-types
[trap]: #instruction-traps
[two's complement]: https://en.wikipedia.org/wiki/Two%27s_complement
[two's complement difference]: https://en.wikipedia.org/wiki/Two%27s_complement#Subtraction
[two's complement product]: https://en.wikipedia.org/wiki/Two%27s_complement#Multiplication
[two's complement sum]: https://en.wikipedia.org/wiki/Two%27s_complement#Addition
[value type]: #value-types
[value types]: #value-types
[uint32]: #primitive-encoding-types
[UTF-8]: https://en.wikipedia.org/wiki/UTF-8
[valid UTF-8]: https://encoding.spec.whatwg.org/#utf-8-decode-without-bom-or-fail
[varuint1]: #primitive-encoding-types
[varuint7]: #primitive-encoding-types
[varuint32]: #primitive-encoding-types
[varuint64]: #primitive-encoding-types
[varsint7]: #primitive-encoding-types
[varsint32]: #primitive-encoding-types
[varsint64]: #primitive-encoding-types
[float32]: #primitive-encoding-types
[float64]: #primitive-encoding-types
[`$args`]: #named-values
[`$returns`]: #named-values
[`$any`]: #named-values
[`$block_arity`]: #named-values