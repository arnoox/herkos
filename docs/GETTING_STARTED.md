# Getting Started

Welcome to `herkos`! This guide will walk you through installing the transpiler and converting your first WebAssembly module to Rust.

`herkos` is a tool that converts WebAssembly modules into safe, memory-isolated Rust code. Instead of relying on runtime hardware memory protection (MMU/MPU), isolation is enforced by the Rust type system at compile time.

**Key benefits:**
- **Memory safe by default**: All transpiled code is memory-safe Rust
- **Isolation enforced at compile time**: No runtime overhead for proven accesses
- **Production-ready**: Three code generation backends (safe, hybrid, verified) for different performance/assurance needs

## Installation

> Note: it is currently only possible to build from source.

Clone the repository and build:

```bash
git clone https://github.com/anthropics/herkos.git
cd herkos
cargo install --path crates/herkos
```

## Basic Usage

### Transpile a WebAssembly Module

```bash
herkos input.wasm --mode safe --output output.rs
```

This creates `output.rs` containing your transpiled code.

### Command-Line Options

| Option | Description | Required |
|--------|-------------|----------|
| `input.wasm` | Path to WebAssembly module | Yes |
| `--mode` | Code generation mode: `safe`, `hybrid`, or `verified` (default: `safe`) | No |
| `--output` | Output Rust file path | No |
| `--max-pages` | Maximum memory pages when module declares no maximum | No |

Current limitations:
- The current state of herkos only supports `safe` mode, all other modes have no effect and will behave like default. More on modes can be found here: TODO.
- `--max-pages` has no effect.

### Example: Simple Translation

Given a WebAssembly module `math.wasm`:

```bash
herkos math.wasm --output math.rs
```

This produces `math.rs` with:
- A `Module` struct containing your transpiled functions
- Memory operations via `IsolatedMemory`
- All imports/exports translated to Rust traits

## Understanding the Output

### Module Structure

The transpiled code defines a `Module` struct that encapsulates your Wasm module:

```rust
// Your transpiled module
pub struct Module<MAX_PAGES: usize, TABLE_SIZE: usize> {
    // Memory for the module
    memory: IsolatedMemory<MAX_PAGES>,
    // Indirect call table
    table: Table<TABLE_SIZE>,
    // Mutable globals
    globals: Globals,
}
```

### Memory Access

All memory operations are safe and bounds-checked:

```rust
// Safe: returns WasmResult<i32>
let value = module.memory.load_i32(offset)?;

// Store returns WasmResult<()>
module.memory.store_i32(offset, value)?;
```

### Calling Exported Functions

Functions exported by the Wasm module become methods on the `Module` struct:

```rust
// If your Wasm module exports a function "add"
let result = module.add(3, 4)?;
```

### Handling Errors

All operations return `WasmResult<T>`, not `Option` or panics:

```rust
match module.process_data(ptr, len) {
    Ok(output) => println!("Result: {}", output),
    Err(e) => eprintln!("Wasm error: {:?}", e),
}
```

## Using Transpiled Code in Your Project

### Step 1: Add herkos-runtime Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
herkos-runtime = "0.1"
```

Or, if you prefer a path dependency:

```toml
[dependencies]
herkos-runtime = { path = "path/to/herkos-runtime" }
```

### Step 2: Include the Transpiled Code

In your Rust project:

```rust
use herkos_runtime::{IsolatedMemory, WasmResult};

// Include the transpiled module
include!("path/to/output.rs");

fn main() -> WasmResult<()> {
    // Instantiate your module
    let mut module = Module::<256, 4>::new(
        16,                          // initial pages
        Globals::default(),          // module globals
        Table::default(),            // call table
    )?;

    // Call exported functions
    let result = module.my_function(42)?;
    println!("Result: {}", result);

    Ok(())
}
```

### Step 3: Compile and Run

```bash
cargo build
cargo run
```

Your transpiled Wasm code runs as native Rust with full type safety.

## Advanced: Using herkos as a Library in build.rs

For a more automated workflow, you can invoke `herkos` programmatically from your build script. This generates Rust code during compilation and includes it automatically, without manual transpilation steps.

### When to Use This Pattern

- **Embedded Wasm modules**: Your `.wasm` files are part of your source tree
- **Automated workflows**: Regenerate Rust code whenever Wasm modules change
- **Clean distribution**: Generated code isn't committed to git (lives in `target/`)
- **Single compilation step**: `cargo build` handles everything

### Step 1: Add herkos as a Build Dependency

Update your `Cargo.toml`:

```toml
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
herkos-runtime = "0.1"

[build-dependencies]
herkos = "0.1"
```

### Step 2: Create a Build Script (build.rs)

Create `build.rs` at the root of your project (same level as `Cargo.toml`):

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    // Get the output directory where generated code will be placed
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    // Path to your WebAssembly module (in your source tree)
    let wasm_path = "wasm-modules/math.wasm";

    // Tell Cargo to rebuild when the Wasm module changes
    println!("cargo:rerun-if-changed={}", wasm_path);

    // Transpile the Wasm module
    let output_file = out_path.join("math_module.rs");
    herkos::transpile_file(
        wasm_path,
        output_file.to_str().unwrap(),
        herkos::Mode::Safe,
    ).expect("Failed to transpile Wasm module");

    println!("Generated Rust code at: {:?}", out_path);
}
```

> **Note**: The exact API depends on the `herkos` crate. See the crate documentation for available functions. The basic pattern is to call a transpilation function that takes input path, output path, and mode.

### Step 3: Include Generated Code at Runtime

In your `src/main.rs` or library file:

```rust
use herkos_runtime::WasmResult;

// Include the generated module code
// include_str! reads the file contents as a string at compile time
// The generated file lives in target/debug/build/<package>/out/
include!(concat!(env!("OUT_DIR"), "/math_module.rs"));

fn main() -> WasmResult<()> {
    // Now you can use the transpiled module
    let mut module = Module::<16, 0>::new(1, Globals::default(), Table::default())?;

    let result = module.add(5, 3)?;
    println!("Result: {}", result);

    Ok(())
}
```

### Step 4: Build and Run

```bash
cargo build
cargo run
```

Cargo automatically:
1. Runs `build.rs` before compilation
2. Generates Rust code in `target/debug/build/<package>/out/`
3. Includes it via `include!()` macro
4. Compiles everything together

### Full Example Project Structure

```
my-app/
├── Cargo.toml              # Include build-dependencies
├── build.rs                # Transpilation script
├── src/
│   └── main.rs             # Include generated code
└── wasm-modules/
    └── math.wasm           # Your WebAssembly module
```

### Advanced: Transpiling Multiple Modules

To transpile several Wasm modules:

```rust
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    // List of (input.wasm, output.rs)
    let modules = vec![
        ("wasm-modules/math.wasm", "math_module.rs"),
        ("wasm-modules/crypto.wasm", "crypto_module.rs"),
        ("wasm-modules/compression.wasm", "compression_module.rs"),
    ];

    for (input, output) in modules {
        // Rebuild when any module changes
        println!("cargo:rerun-if-changed={}", input);

        // Transpile each module
        let output_file = out_path.join(output);
        herkos::transpile_file(
            input,
            output_file.to_str().unwrap(),
            herkos::Mode::Safe,
        ).expect(&format!("Failed to transpile {}", input));
    }
}
```

Then include them all:

```rust
use herkos_runtime::WasmResult;

include!(concat!(env!("OUT_DIR"), "/math_module.rs"));
include!(concat!(env!("OUT_DIR"), "/crypto_module.rs"));
include!(concat!(env!("OUT_DIR"), "/compression_module.rs"));

fn main() -> WasmResult<()> {
    let mut math = MathModule::<16, 0>::new(1, Globals::default(), Table::default())?;
    let mut crypto = CryptoModule::<16, 0>::new(1, Globals::default(), Table::default())?;

    // Use both modules...
    Ok(())
}
```

### Important Notes on `include!()`

The `include!()` macro:
- **Reads the file at compile time** — the generated file must exist before Rust compilation
- **Path is relative to the source file** — use `concat!(env!("OUT_DIR"), "/file.rs")` for files in the output directory
- **No module namespacing** — everything from the included file (structs, functions, etc.) is in the same namespace. If you include multiple modules, use different struct names or wrap them in modules:

```rust
mod math {
    include!(concat!(env!("OUT_DIR"), "/math_module.rs"));
}

mod crypto {
    include!(concat!(env!("OUT_DIR"), "/crypto_module.rs"));
}

fn main() -> WasmResult<()> {
    let mut m = math::Module::<16, 0>::new(1, Globals::default(), Table::default())?;
    let mut c = crypto::Module::<16, 0>::new(1, Globals::default(), Table::default())?;
    Ok(())
}
```

### Handling Regeneration and Caching

Cargo caches build scripts. To force regeneration:

```bash
# Clean build
cargo clean

# Or rebuild only if Wasm modules changed (Cargo tracks this)
cargo build
```

The `println!("cargo:rerun-if-changed=...")` directives in `build.rs` tell Cargo to re-run the script only when those files change. This keeps builds fast.


## Example: Transpiling C Code

### Original C Code (math.c)

```c
int add(int a, int b) {
    return a + b;
}

int multiply(int a, int b) {
    return a * b;
}
```

### Compile to WebAssembly

Using Clang:

```bash
clang --target=wasm32 -O2 -c math.c -o math.o
wasm-ld math.o -o math.wasm --no-entry
```

### Transpile to Rust

```bash
herkos math.wasm --output math.rs
```

### Use in Your Rust Project

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

## Common Tasks

### Check What's in Your Wasm Module

Use `wasmprinter` or `wasm-tools`:

```bash
# List exports
wasm-tools metadata wasm input.wasm
```

Or just transpile it and look at the generated `output.rs`.

### Set Memory Size

If your Wasm module doesn't declare a maximum memory size, use `--max-pages`:

```bash
herkos input.wasm --max-pages 256 --output output.rs
```

This sets the maximum to 256 pages (16 MiB).

### Reduce Binary Size

Use release builds:

```bash
cargo build --release
```

The Rust compiler and linker will optimize away unused monomorphizations.

### Debug Memory Access Issues

When a bounds check fails, you get a `WasmTrap::MemoryOutOfBounds` error. Check:
1. Your module's memory declarations in the Wasm binary
2. That memory accesses stay within `(active_pages * 65536)` bytes
3. That you're using the right `MAX_PAGES` value

---

Happy transpiling! 🦀
