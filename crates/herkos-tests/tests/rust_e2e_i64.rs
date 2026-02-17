//! End-to-end tests: Rust → Wasm → Rust (i64 operations).

use herkos_tests::rust_e2e_i64;

fn new_module() -> rust_e2e_i64::WasmModule {
    rust_e2e_i64::new().expect("module instantiation should succeed")
}

// ── Basic i64 arithmetic ──

#[test]
fn test_mul_i64() {
    let mut m = new_module();
    assert_eq!(m.mul_i64(6, 7).unwrap(), 42);
    assert_eq!(m.mul_i64(100_000, 100_000).unwrap(), 10_000_000_000i64);
    assert_eq!(m.mul_i64(0, i64::MAX).unwrap(), 0);
}

#[test]
fn test_sub_i64() {
    let mut m = new_module();
    assert_eq!(m.sub_i64(10, 3).unwrap(), 7);
    assert_eq!(m.sub_i64(0, 0).unwrap(), 0);
    assert_eq!(m.sub_i64(3, 10).unwrap(), -7);
}

// ── Bitwise i64 ──

#[test]
fn test_bitwise_i64() {
    let mut m = new_module();
    assert_eq!(m.bitwise_and_i64(0xFF00, 0x0FF0).unwrap(), 0x0F00);
    assert_eq!(m.bitwise_or_i64(0xF000, 0x000F).unwrap(), 0xF00F);
    assert_eq!(m.bitwise_xor_i64(0xFF, 0xFF).unwrap(), 0);
}

// ── Shifts i64 ──

#[test]
fn test_shift_i64() {
    let mut m = new_module();
    assert_eq!(m.shift_left_i64(1, 32).unwrap(), 1i64 << 32);
    assert_eq!(m.shift_right_s_i64(-1, 1).unwrap(), -1);
}

#[test]
fn test_negate_i64() {
    let mut m = new_module();
    assert_eq!(m.negate_i64(42).unwrap(), -42);
    assert_eq!(m.negate_i64(0).unwrap(), 0);
}

// ── i64 algorithms ──

#[test]
fn test_fib_i64() {
    let mut m = new_module();
    assert_eq!(m.fib_i64(0).unwrap(), 0);
    assert_eq!(m.fib_i64(1).unwrap(), 1);
    assert_eq!(m.fib_i64(10).unwrap(), 55);
    assert_eq!(m.fib_i64(50).unwrap(), 12_586_269_025i64);
}

#[test]
fn test_factorial_i64() {
    let mut m = new_module();
    assert_eq!(m.factorial_i64(0).unwrap(), 1);
    assert_eq!(m.factorial_i64(1).unwrap(), 1);
    assert_eq!(m.factorial_i64(10).unwrap(), 3_628_800);
    assert_eq!(m.factorial_i64(20).unwrap(), 2_432_902_008_176_640_000i64);
}

// ── Cross-validation: Rust E2E matches C E2E ──

#[test]
fn test_fib_matches_c() {
    let mut m = new_module();
    let expected: &[(i64, i64)] = &[(0, 0), (1, 1), (10, 55), (20, 6765), (50, 12_586_269_025)];
    for &(n, fib) in expected {
        assert_eq!(m.fib_i64(n).unwrap(), fib, "fib_i64({n})");
    }
}

#[test]
fn test_factorial_matches_c() {
    let mut m = new_module();
    let expected: &[(i64, i64)] = &[(0, 1), (1, 1), (5, 120), (10, 3_628_800)];
    for &(n, fact) in expected {
        assert_eq!(m.factorial_i64(n).unwrap(), fact, "factorial_i64({n})");
    }
}
