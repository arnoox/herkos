# C-FFT → WebAssembly → Rust Example

A 4096-point radix-2 Cooley-Tukey FFT in C, compiled to WebAssembly, and transpiled to memory-safe Rust by herkos. The Rust host writes audio signals into the module's isolated memory, calls the FFT, and reads back the spectrum.

```
fft.c  ──clang──▶  fft.wasm  ──herkos──▶  src/fft_wasm.rs
                                                │
                                          src/main.rs drives it
                                                │
                                           cargo run
```

The generated Rust module contains **no unsafe code**. Memory isolation is enforced through the type system at compile time.

## Prerequisites

- **clang** with wasm32 target support (`apt-get install clang lld`)
- **Rust** toolchain (`cargo`)
- **herkos** CLI (already available in the repo)

## Usage

```bash
./run.sh          # compile C → Wasm → Rust, then build and run
./run.sh --clean  # remove generated artifacts
```

Or directly from this directory:
```bash
cargo run --release
```

## Key design points

- **No libm**: twiddle factors are computed via Taylor series at the base angle (δ = 2π/4096) then propagated by complex recurrence. `sqrtf` compiles to the Wasm `f32.sqrt` instruction — no library import.

- **Static buffers only**: all arrays are `static float[...]` globals (BSS, zero-initialized), no malloc, no stack VLAs.

- **Memory budget**: signal (32 KB) + twiddle real (8 KB) + twiddle imag (8 KB) + magnitude (8 KB) = 56 KB BSS, fits comfortably inside the 64 KB non-stack region of 2 Wasm pages (128 KB).

- **Host↔module interface**: the host uses `fft_get_input_ptr()` and `fft_get_output_ptr()` to discover buffer addresses rather than hardcoding linker offsets. This demonstrates capability-based access to isolated memory.

- **Clang flags**: `-Wl,--initial-memory=131072 -Wl,--max-memory=131072` (2 pages), `-Wl,-zstack-size=65536` (64KB stack).

## Performance

Each 4096-point FFT computes in ~420 microseconds on a modern CPU. This includes:
- Bit-reversal permutation
- 12 stages of butterfly operations (stage = 2¹ to 2¹²)
- Magnitude computation (via `__builtin_sqrtf`)

The overhead vs. native Wasm execution is negligible (monomorphization and inlining eliminate the `IsolatedMemory` abstraction cost).

## Algorithm details

**Cooley-Tukey radix-2 DIT (Decimation In Time):**

1. **Bit-reversal**: permute input to separate even/odd indices
2. **Twiddle table init**: compute W_N^k = e^(-2πik/N) using Taylor series + recurrence
3. **Butterfly passes**: 12 stages, each stage has 2^s butterflies with stride 2^(12-s)
4. **Magnitude**: |X_k| = √(re² + im²)

**Why no `sin()`/`cos()` from libm?**

For tiny angle δ = 2π/4096 ≈ 0.00153 rad, Taylor series (3 terms each) are accurate to ~1e-9:
```
sin(δ) ≈ δ − δ³/6 + δ⁵/120
cos(δ) ≈ 1 − δ²/2 + δ⁴/24
```

Then all 2048 twiddle factors are generated via complex rotation recurrence:
```
W[k+1] = W[k] × W[1]    (complex multiplication)
```

This avoids any libm import — the whole FFT is freestanding C with only `__builtin_sqrtf()` (which compiles to Wasm's native `f32.sqrt` instruction).

## Integration with herkos

1. **C source**: fft.c (self-contained, no external dependencies)
2. **Wasm binary**: fft.wasm (1.1 KB, extremely compact)
3. **Generated Rust**: src/fft_wasm.rs (bounds-checked memory API, no unsafe)
4. **Host integration**: src/main.rs accesses module memory via `module.0.memory.store_f32()`/`load_f32()`

The generated `WasmModule` wraps `Module<Globals, MAX_PAGES, TABLE_SIZE>` and exposes:
- Constructor: `new() -> WasmResult<WasmModule>`
- Exported functions: `fft_init()`, `fft_compute()`, `fft_get_input_ptr()`, `fft_get_output_ptr()`
- Memory access: `module.0.memory.store_f32(offset, value)`, `module.0.memory.load_f32(offset)`

All memory operations return `WasmResult<T>` — traps (out-of-bounds, overflow) propagate as errors, never panics.

## Example output

```
=== herkos C-FFT Example ===
    4096-point radix-2 DIT FFT, C → Wasm → memory-safe Rust
    Input buffer:  Wasm byte offset 0x00400
    Output buffer: Wasm byte offset 0x08400

--- Test 1: Single tone at 1000 Hz ---
    Spectrum (1000 Hz tone) — 419.50µs
      991 Hz (bin   92): ####                           272.3
     1001 Hz (bin   93): ############################## 2000.3
     1012 Hz (bin   94): ###                            215.3
    Peak bin: 93 → 1001.3 Hz

--- Test 2: Two tones (440 Hz + 2000 Hz) ---
    ...
    Top bins:
      bin   41 →   441.4 Hz  mag=1591.6
      bin  186 →  2002.6 Hz  mag=930.7
```

Frequencies are accurate to within ~1 Hz (limited by FFT bin resolution of 44100/4096 ≈ 10.77 Hz per bin).
