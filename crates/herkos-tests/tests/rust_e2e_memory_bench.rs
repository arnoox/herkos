//! End-to-end tests: Rust → Wasm → Rust (fill / bubble-sort / sum over Wasm memory).
//!
//! The source module (`data/rust/rust_e2e_memory_bench.rs`) exports a single
//! function:
//!
//! ```text
//! mem_fill_sort_sum(n: i32, seed: i32) -> i32
//! ```
//!
//! It fills the first `n` elements of a 1024-element static buffer with LCG
//! pseudo-random values, bubble-sorts them in place, then returns a wrapping
//! checksum.  Tests verify the transpiled output against a native reference.

use herkos_tests::rust_e2e_memory_bench;

fn new_module() -> rust_e2e_memory_bench::WasmModule {
    rust_e2e_memory_bench::new().expect("module instantiation should succeed")
}

// ── Reference implementation ──────────────────────────────────────────────────

include!("../data/rust/common/fill_sort_sum.rs");

fn fill_sort_sum_ref(n: i32, seed: i32) -> i32 {
    let mut buf = [0i32; 1024];
    fill_sort_sum_impl(&mut buf, n, seed)
}

// ── Edge cases ────────────────────────────────────────────────────────────────

#[test]
fn test_zero_elements_returns_zero() {
    let mut m = new_module();
    assert_eq!(m.mem_fill_sort_sum(0, 0).unwrap(), 0);
    assert_eq!(m.mem_fill_sort_sum(0, 42).unwrap(), 0);
}

#[test]
fn test_negative_n_returns_zero() {
    let mut m = new_module();
    assert_eq!(m.mem_fill_sort_sum(-1, 0).unwrap(), 0);
    assert_eq!(m.mem_fill_sort_sum(i32::MIN, 1).unwrap(), 0);
}

#[test]
fn test_single_element() {
    let mut m = new_module();
    // With n=1 there is nothing to sort; checksum is just the one LCG value.
    let seed: i32 = 1;
    let expected = seed.wrapping_mul(1103515245_i32).wrapping_add(12345);
    assert_eq!(m.mem_fill_sort_sum(1, seed).unwrap(), expected);
}

// ── Cross-validation against reference ───────────────────────────────────────

#[test]
fn test_matches_reference_small_n() {
    let mut m = new_module();
    for n in 1i32..=16 {
        assert_eq!(
            m.mem_fill_sort_sum(n, 0).unwrap(),
            fill_sort_sum_ref(n, 0),
            "n={n} seed=0"
        );
    }
}

#[test]
fn test_matches_reference_various_seeds() {
    let mut m = new_module();
    let cases: &[(i32, i32)] = &[
        (10, 0),
        (10, 1),
        (10, -1),
        (10, i32::MAX),
        (10, i32::MIN),
        (32, 42),
        (64, 12345),
        (128, -99999),
    ];
    for &(n, seed) in cases {
        assert_eq!(
            m.mem_fill_sort_sum(n, seed).unwrap(),
            fill_sort_sum_ref(n, seed),
            "n={n} seed={seed}"
        );
    }
}

#[test]
fn test_matches_reference_full_buffer() {
    let mut m = new_module();
    // n=1024 exercises every element of the static buffer.
    assert_eq!(
        m.mem_fill_sort_sum(1024, 7).unwrap(),
        fill_sort_sum_ref(1024, 7)
    );
}

#[test]
fn test_n_capped_at_1024() {
    let mut m = new_module();
    // Values beyond 1024 must be clamped; result should equal n=1024.
    assert_eq!(
        m.mem_fill_sort_sum(2048, 7).unwrap(),
        fill_sort_sum_ref(1024, 7),
        "n>1024 must be clamped to 1024"
    );
}

// ── Sorting invariant ─────────────────────────────────────────────────────────
//
// We cannot read the sorted buffer directly through the transpiled API, but we
// can verify that the checksum is independent of argument order (i.e. the sort
// produces a deterministic total regardless of how the LCG happens to seed).
// The key property is: two calls with the same (n, seed) must always agree.

#[test]
fn test_deterministic_across_calls() {
    let mut m = new_module();
    let first = m.mem_fill_sort_sum(64, 9999).unwrap();
    let second = m.mem_fill_sort_sum(64, 9999).unwrap();
    assert_eq!(
        first, second,
        "same (n, seed) must always yield the same checksum"
    );
}

// ── Sequential calls with distinct inputs remain independent ──────────────────
//
// The module keeps a static buffer; verify that successive calls with different
// seeds still match the reference (no stale state from a previous call leaks
// into the next fill).

#[test]
fn test_sequential_calls_independent() {
    let mut m = new_module();
    let seeds = [0i32, 1, -1, 100, 999_999, i32::MAX, i32::MIN];
    for seed in seeds {
        assert_eq!(
            m.mem_fill_sort_sum(32, seed).unwrap(),
            fill_sort_sum_ref(32, seed),
            "seed={seed}"
        );
    }
}
