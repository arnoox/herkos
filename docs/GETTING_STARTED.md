# Getting Started

## Installation

From crates.io:
```bash
cargo install herkos
```

From source:
```bash
git clone https://github.com/arnoox/herkos.git
cd herkos
cargo install --path crates/herkos
```

## Basic Usage

```bash
herkos input.wasm --output output.rs
herkos input.wasm -O --output output.rs   # with IR optimizations enabled
```

| Option | Description | Required |
|--------|-------------|----------|
| `input.wasm` | Path to WebAssembly module | Yes |
| `--output`, `-o` | Output Rust file path (defaults to stdout) | No |
| `--optimize`, `-O` | Enable IR optimization passes | No |

## Understanding the Output

The transpiler produces a self-contained Rust source file that depends only on `herkos-runtime`. The output contains:

```rust
// Generated output.rs
use herkos_runtime::*;

struct Globals { ... }  // ← mutable globals
const G1: i64 = 42;     // ← immutable

fn func_0(...) { ... }  // ← Wasm functions
fn func_1(...) { ... }

struct Module<MAX_PAGES, TABLE_SIZE> {
    memory: IsolatedMemory<MAX_PAGES>,
    globals: Globals,
    table: Table<TABLE_SIZE>,
}

impl Module { ... }          // ← exports as methods
trait ModuleImports { ... }  // ← required capabilities
```

## Using Transpiled Code

### Direct inclusion

```rust
use herkos_runtime::{IsolatedMemory, WasmResult};

include!("path/to/output.rs");

fn main() -> WasmResult<()> {
    let mut module = Module::<256, 4>::new(
        16,                      // initial pages
        Globals::default(),      // module globals
        Table::default(),        // call table
    )?;

    let result = module.my_function(42)?;
    println!("Result: {}", result);
    Ok(())
}
```

### Via build.rs (recommended for automated workflows)

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    println!("cargo:rerun-if-changed=wasm-modules/math.wasm");

    let wasm_bytes = std::fs::read("wasm-modules/math.wasm").unwrap();
    let options = herkos::TranspileOptions::default();
    let rust_code = herkos::transpile(&wasm_bytes, &options).unwrap();
    std::fs::write(out_path.join("math_module.rs"), rust_code).unwrap();
}
```

```rust
// src/main.rs
use herkos_runtime::WasmResult;
include!(concat!(env!("OUT_DIR"), "/math_module.rs"));

fn main() -> WasmResult<()> {
    let mut module = Module::<16, 0>::new(1, Globals::default(), Table::default())?;
    let result = module.add(5, 3)?;
    println!("Result: {}", result);
    Ok(())
}
```

When including multiple modules, wrap them in Rust modules to avoid name collisions:

```rust
mod math {
    include!(concat!(env!("OUT_DIR"), "/math_module.rs"));
}
mod crypto {
    include!(concat!(env!("OUT_DIR"), "/crypto_module.rs"));
}
```

## Example: C to Rust via Wasm

This example walks through the full pipeline: starting from a C source file,
compiling it to a Wasm binary, and then using `herkos` to transpile that binary
into safe Rust code you can call directly.

Start with a simple C library:

```c
// math.c
int add(int a, int b) { return a + b; }
int multiply(int a, int b) { return a * b; }
```

Compile it to Wasm using `clang` and `wasm-ld`, then transpile with `herkos`:

```bash
clang --target=wasm32 -O2 -c math.c -o math.o
wasm-ld math.o -o math.wasm --no-entry
herkos math.wasm --output math.rs
```

`--no-entry` tells the linker not to require a `main` symbol, since this is a
library. The transpiler reads `math.wasm` and writes `math.rs`, a self-contained
Rust module that only depends on `herkos-runtime`.

Include the generated file and instantiate the module to call its exports:

```rust
use herkos_runtime::WasmResult;
include!("math.rs");

fn main() -> WasmResult<()> {
    let mut module = Module::<16, 0>::new(1, Globals::default(), Table::default())?;
    println!("2 + 3 = {}", module.add(2, 3)?);
    println!("4 * 5 = {}", module.multiply(4, 5)?);
    Ok(())
}
```

The const generics `<16, 0>` set the memory limit to 16 pages (1 MiB) and the
indirect-call table size to 0, since this module makes no indirect calls. Each
exported function returns `WasmResult<T>`, propagating any Wasm traps (such as
out-of-bounds memory access or integer overflow) as `Err(WasmTrap)` rather than
panicking.
