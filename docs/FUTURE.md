# Future Extensions

This document describes features that are **planned but not yet implemented**. For the current specification, see [SPECIFICATION.md](SPECIFICATION.md).

---

## 1. Verified Backend

The verified backend emits `unsafe` Rust for memory accesses and arithmetic where **formal proofs** guarantee safety. It is the performance-optimal backend for fully analyzed modules.

### 1.1 Design

- Every memory access and arithmetic operation must have an associated proof
- Each `unsafe` block carries a machine-checkable proof reference: `// PROOF: bounds_check_0x1234`
- **Compilation fails** if any operation lacks a proof — no silent fallback
- Target overhead: **0–5%** (function call indirection only)

```rust
// Verified backend: unchecked load justified by formal proof
// PROOF: bounds_check_0x1234 — offset ∈ [0, MEM_SIZE - 4] proven by static analysis
let value = unsafe { memory.load_i32_unchecked(offset as usize) };
```

### 1.2 Proof Obligations

- **Spatial**: the access `[offset, offset + size)` is within linear memory bounds
- **Alignment**: the offset satisfies the type's alignment requirement
- **Arithmetic**: integer operations cannot overflow in the proven value ranges

### 1.3 Unchecked Memory Access API

The runtime already provides unsafe unchecked methods for future use:

```rust
impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
    unsafe fn load_i32_unchecked(&self, offset: usize) -> i32;
    unsafe fn load_i64_unchecked(&self, offset: usize) -> i64;
    unsafe fn store_i32_unchecked(&mut self, offset: usize, value: i32);
    unsafe fn store_i64_unchecked(&mut self, offset: usize, value: i64);
}
```

### 1.4 Stack Frame Isolation

The shadow stack pattern enables **stack frame isolation proofs**:

1. Identify `__stack_pointer` adjustments at function entry/exit
2. Prove all memory accesses target `[__stack_pointer, __stack_pointer + frame_size)`
3. Prove the stack pointer is restored on exit

This enables:
- **Stack frame purity**: if a function only writes to its own stack frame, the verified backend can prove no side effects on heap/global data
- **Stack bounds elimination**: if the stack pointer stays within the stack region, all stack-relative accesses can be proven in-bounds without per-access checks

---

## 2. Hybrid Backend

The hybrid backend accepts **partial** verification metadata. Proven accesses emit `unsafe` with proof references; unproven accesses fall back to runtime checks.

### 2.1 Design

```rust
// Proven access — unchecked
// PROOF: bounds_check_0x1234
let a = unsafe { memory.load_i32_unchecked(proven_offset as usize) };

// Unproven access — runtime check
// UNPROVEN: fallback to runtime check
let b = memory.load_i32(dynamic_offset as usize)?;
```

- The transpiler annotates each fallback site so developers can iteratively improve proof coverage
- Target overhead: **5–15%** depending on proof coverage
- Practical choice for production use: maximize performance where provable, stay safe elsewhere

### 2.2 Backend Selection Summary

| Backend  | `unsafe` in output | Proof requirement | Overhead | Use case |
|----------|--------------------|-------------------|----------|----------|
| Safe     | None               | None              | 15–30%   | Migration, testing, non-critical modules |
| Verified | All accesses       | Complete          | 0–5%     | Performance-critical, fully analyzed |
| Hybrid   | Proven accesses    | Partial           | 5–15%    | Production — iterative proof improvement |

---

## 3. Temporal Isolation (Fuel-Based Execution)

Spatial isolation (implemented) prevents memory corruption. **Temporal isolation** prevents CPU time starvation — equally critical for safety standards (ISO 26262 requires both).

### 3.1 The Fuel Model

Each module call receives a **fuel budget**. The transpiler inserts fuel decrements at loop headers and function calls. When fuel reaches zero, the function returns `WasmTrap::FuelExhausted`:

```rust
struct Fuel {
    remaining: u64,
}

impl Fuel {
    #[inline(always)]
    fn consume(&mut self, cost: u64) -> WasmResult<()> {
        if self.remaining < cost {
            return Err(WasmTrap::FuelExhausted);
        }
        self.remaining -= cost;
        Ok(())
    }
}
```

### 3.2 Three Temporal Guarantee Levels

| Level | Mechanism | Overhead | Guarantee |
|-------|-----------|----------|-----------|
| Fuel-checked (safe) | Runtime fuel decrements | ~3–5% | Guaranteed termination within budget |
| WCET-proven (verified) | Static analysis of loop bounds | 0% | Proven worst-case execution time |
| Hybrid | Mix of proven and checked | 1–3% | Static bounds where provable, runtime elsewhere |

### 3.3 Symmetry with Spatial Isolation

| Dimension | Spatial (memory) | Temporal (CPU time) |
|-----------|-----------------|---------------------|
| What's protected | Memory regions | CPU cycles |
| Safe backend | Bounds check at every access | Fuel check at every back edge |
| Verified backend | Proven in-bounds → unchecked | Proven bounded → no fuel check |
| Trap on violation | `WasmTrap::OutOfBounds` | `WasmTrap::FuelExhausted` |

---

## 4. Contract-Based Verification

Instead of external analysis metadata, the verified and hybrid backends can emit Rust code annotated with **formal verification contracts** checked during compilation.

### 4.1 Kani (Bounded Model Checking)

Kani is the most practical choice today. The transpiler can emit proof harnesses alongside generated functions:

```rust
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(64)]
fn verify_sum_array() {
    let memory = IsolatedMemory::<256>::new(16);
    let arr: u32 = kani::any();
    let len: u32 = kani::any();
    kani::assume(arr as u64 + len as u64 * 4 <= 16 * 65536);
    let result = sum_array(&memory, arr, len);
}
```

### 4.2 Flux-rs (Refinement Types)

More ambitious: bounds constraints encoded directly in the type system:

```rust
#[flux_rs::sig(fn(memory: &IsolatedMemory<MAX_PAGES>,
                   offset: u32{v: v + 3 < MAX_PAGES * 65536}) -> i32)]
fn load_i32_verified(memory: &IsolatedMemory<MAX_PAGES>, offset: u32) -> i32 {
    memory.load_proven::<i32>(offset as usize)
}
```

### 4.3 Comparison

| Aspect | External metadata | Contract-based (Kani/Flux) |
|--------|------------------|---------------------------|
| Proof location | Separate TOML file | Embedded in Rust source |
| Synchronization | Requires hash check | Inherently in sync |
| Toolchain | Analysis tool + SMT solver | `cargo kani` or Flux plugin |
| `no_std` compat | N/A (offline) | Kani: yes; Flux: partial |
| Maturity | Custom (to be developed) | Kani: production; Flux: research |

**Recommended strategy**: Use both complementarily:
1. **Kani** for verifying `herkos-runtime` itself (already partially done)
2. **External metadata** for transpiled code initially
3. **Contract-based** as a future evolution when Flux-rs matures

---

## 5. Other Planned Features

- **Automated refactoring suggestions** for better Rust idioms in generated code
- **DWARF debug info preservation** for source-level debugging of transpiled code
- **Proof coverage reports**: per-function and per-module percentage of accesses that are proven vs. runtime-checked
- **Dynamic linking** of transpiled modules (open question — see [SPECIFICATION.md §8](SPECIFICATION.md#8-open-questions))
