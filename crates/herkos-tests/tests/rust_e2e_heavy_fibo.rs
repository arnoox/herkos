//! End-to-end tests: Rust → Wasm → Rust (memoised Fibonacci with custom allocator).
//!
//! The source module (`data/rust/rust_e2e_heavy_fibo.rs`) uses a bump allocator
//! to provide `liballoc` in a `no_std` Wasm environment.  It caches computed
//! Fibonacci values in a `Vec<i32>` so that:
//!
//! * Calling `fibo(n)` when `n` is already cached costs O(1).
//! * Calling `fibo(n + m)` after a previous `fibo(n)` only computes the
//!   `m` new values — all earlier results are served from the cache.
//!
//! `fibo_cache_len()` exposes the current cache size so tests can verify the
//! memoisation invariants without inspecting internal state directly.

use herkos_tests::rust_e2e_heavy_fibo;

fn new_module() -> rust_e2e_heavy_fibo::WasmModule {
    rust_e2e_heavy_fibo::new().expect("module instantiation should succeed")
}

// ── Reference implementation ──────────────────────────────────────────────────

/// Ground-truth `fibo(n)` using the same wrapping i32 arithmetic as Wasm.
fn fibo_ref(n: usize) -> i32 {
    let mut a: i32 = 0;
    let mut b: i32 = 1;
    for _ in 0..n {
        let tmp = a.wrapping_add(b);
        a = b;
        b = tmp;
    }
    a
}

// ── Base cases ────────────────────────────────────────────────────────────────

#[test]
fn test_fibo_base_cases() {
    let mut m = new_module();
    assert_eq!(m.fibo(0).unwrap(), 0);
    assert_eq!(m.fibo(1).unwrap(), 1);
}

// ── Small known values ────────────────────────────────────────────────────────

#[test]
fn test_fibo_small_values() {
    let mut m = new_module();
    // Classical Fibonacci sequence: 0,1,1,2,3,5,8,13,21,34,55,89,144
    let expected: &[i32] = &[0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144];
    for (n, &want) in expected.iter().enumerate() {
        assert_eq!(m.fibo(n as i32).unwrap(), want, "fibo({n})");
    }
}

// ── Cross-validation against reference ───────────────────────────────────────

#[test]
fn test_fibo_matches_reference() {
    let mut m = new_module();
    for n in 0i32..50 {
        assert_eq!(m.fibo(n).unwrap(), fibo_ref(n as usize), "fibo({n})");
    }
}

// ── Negative input guard ──────────────────────────────────────────────────────

#[test]
fn test_fibo_negative_returns_zero() {
    let mut m = new_module();
    assert_eq!(m.fibo(-1).unwrap(), 0);
    assert_eq!(m.fibo(-100).unwrap(), 0);
}

// ── Cache growth ──────────────────────────────────────────────────────────────

#[test]
fn test_cache_seeded_with_two_entries() {
    let mut m = new_module();
    // The cache is pre-seeded with fibo(0) = 0 and fibo(1) = 1 on first access.
    assert_eq!(m.fibo_cache_len().unwrap(), 2);
}

#[test]
fn test_cache_grows_to_cover_requested_index() {
    let mut m = new_module();
    m.fibo(10).unwrap();
    // Cache must cover indices 0..=10, i.e. 11 entries.
    assert_eq!(m.fibo_cache_len().unwrap(), 11);
}

#[test]
fn test_cache_extends_incrementally() {
    let mut m = new_module();

    m.fibo(10).unwrap();
    assert_eq!(m.fibo_cache_len().unwrap(), 11);

    // Requesting fibo(20) only needs to compute 10 new values.
    m.fibo(20).unwrap();
    assert_eq!(m.fibo_cache_len().unwrap(), 21);

    m.fibo(30).unwrap();
    assert_eq!(m.fibo_cache_len().unwrap(), 31);
}

#[test]
fn test_cache_does_not_shrink_on_smaller_query() {
    let mut m = new_module();
    m.fibo(20).unwrap();
    let len_after = m.fibo_cache_len().unwrap();

    // A smaller query must not evict cached entries.
    m.fibo(5).unwrap();
    assert_eq!(
        m.fibo_cache_len().unwrap(),
        len_after,
        "cache must not shrink after a smaller query"
    );
}

// ── Monotone cache invariant ──────────────────────────────────────────────────

#[test]
fn test_cache_len_is_monotone() {
    let mut m = new_module();
    let mut prev_len = m.fibo_cache_len().unwrap();

    // Mix of increasing and decreasing n values; cache must never shrink.
    for n in [0i32, 3, 1, 10, 7, 20, 20, 25, 24, 30] {
        m.fibo(n).unwrap();
        let new_len = m.fibo_cache_len().unwrap();
        assert!(
            new_len >= prev_len,
            "cache len should be monotone non-decreasing (was {prev_len}, now {new_len} after fibo({n}))"
        );
        prev_len = new_len;
    }
}

// ── Wrapping arithmetic beyond i32 overflow ───────────────────────────────────

#[test]
fn test_fibo_wrapping_overflow() {
    let mut m = new_module();
    // fibo(46) = 1_836_311_903 is the last value that fits in i32.
    // fibo(47) and beyond overflow — wrapping behaviour must match the reference.
    for n in 44i32..=55 {
        assert_eq!(
            m.fibo(n).unwrap(),
            fibo_ref(n as usize),
            "fibo({n}) wrapping"
        );
    }
}

// ── Idempotency: repeated calls return identical results ──────────────────────

#[test]
fn test_fibo_idempotent() {
    let mut m = new_module();
    // Prime the cache up to n=30.
    for n in 0i32..=30 {
        m.fibo(n).unwrap();
    }
    // All values must be stable on repeated queries.
    for n in 0i32..=30 {
        assert_eq!(
            m.fibo(n).unwrap(),
            fibo_ref(n as usize),
            "fibo({n}) should be idempotent"
        );
    }
}
