# Kani Formal Verification

This crate uses [Kani](https://model-checking.github.io/kani/), Amazon Web Services's Rust verification tool based on bounded model checking (CBMC), to formally verify core invariants of the runtime library.

## What is Verified

The Kani proof harnesses exhaustively verify that:

### Memory (`IsolatedMemory`)
- **No panics**: All load/store operations return `Ok` or `Err(WasmTrap)`, never panic
- **Bounds checking**: Successful loads/stores are always within `[0, active_size)`
- **Overflow handling**: Offset overflow is detected and returns `OutOfBounds`
- **Grow semantics**:
  - `active_pages` never exceeds `MAX_PAGES`
  - Failed grow returns -1 and leaves state unchanged
  - Successful grow zero-initializes new pages
- **Store/load roundtrips**: Storing then loading returns the original value
- **Active region**: Accesses beyond `active_pages` (but within `MAX_PAGES`) are rejected

### Table (`Table`)
- **No panics**: All get/set/grow operations return `Ok` or `Err(WasmTrap)`, never panic
- **Bounds checking**: Operations only succeed when `index < active_size`
- **Grow semantics**:
  - `active_size` never exceeds `MAX_SIZE`
  - Failed grow returns -1 and leaves state unchanged
  - Successful grow initializes new slots with the init value
- **Set/get roundtrips**: Setting then getting returns the same entry
- **Empty slots**: Getting an empty slot returns `UndefinedElement`

## Installation

Install Kani following the [official instructions](https://model-checking.github.io/kani/install-guide.html):

```bash
cargo install --locked kani-verifier
cargo kani setup
```

## Running Verification

Verify all proofs in this crate:

```bash
cargo kani -p herkos-runtime
```

Verify a specific proof harness:

```bash
cargo kani -p herkos-runtime --harness load_i32_never_panics
```

Run with verbose output:

```bash
cargo kani -p herkos-runtime --verbose
```

## Proof Statistics

To see verification coverage and proof counts:

```bash
cargo kani -p herkos-runtime --verbose | grep "VERIFICATION"
```

## Continuous Integration

The Kani proofs should be run in CI on every commit. Add to `.github/workflows/ci.yml`:

```yaml
- name: Install Kani
  run: |
    cargo install --locked kani-verifier
    cargo kani setup

- name: Run Kani proofs
  run: cargo kani -p herkos-runtime
```

## Understanding Proof Results

Kani will report:
- **VERIFICATION SUCCESSFUL**: All assertions passed for all possible inputs
- **VERIFICATION FAILED**: A counterexample was found
- **UNREACHABLE**: Some code paths are proven unreachable (good for panic paths)

If a proof fails, Kani provides:
1. The failing assertion
2. A concrete counterexample (input values that trigger the failure)
3. A trace showing the execution path to the failure

## Performance Notes

- Verification time scales with the size of `MAX_PAGES` / `MAX_SIZE` const generics
- The proofs use small values (1-4 pages, 4-8 table slots) to keep verification tractable
- This is sound: if the code is correct for small values, const generics ensure it's correct for all values

## Adding New Proofs

When adding new runtime functionality:

1. Add a `#[kani::proof]` harness in the `#[cfg(kani)] mod proofs` section
2. Use `kani::any()` to generate symbolic inputs covering all possible values
3. Use `kani::assert()` to state invariants that must hold
4. Add `#[kani::unwind(N)]` if the proof involves loops (set N to loop bound + 1)

Example:

```rust
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    #[kani::unwind(1)]
    fn my_operation_never_panics() {
        let mut mem = IsolatedMemory::<4>::new(1);
        let input: i32 = kani::any();
        let _ = mem.my_operation(input);
        // Kani verifies no panic occurs for all possible i32 values
    }
}
```

## References

- [Kani Tutorial](https://model-checking.github.io/kani/kani-tutorial.html)
- [Kani Rust Book](https://model-checking.github.io/kani/)
- [FUTURE.md §4](../../docs/FUTURE.md) — Contract-based verification design
- [SPECIFICATION.md §3.2](../../docs/SPECIFICATION.md) — Runtime architecture
