//! End-to-end tests: C → Wasm → Rust (i64 arithmetic).

use herkos_tests::c_e2e_i64;

fn new_module() -> c_e2e_i64::WasmModule {
    c_e2e_i64::new().expect("module instantiation should succeed")
}

// ── Basic i64 arithmetic ──

#[test]
fn test_mul_i64() {
    let mut m = new_module();
    assert_eq!(m.mul_i64(6, 7).unwrap(), 42);
    assert_eq!(m.mul_i64(0, 999).unwrap(), 0);
    assert_eq!(m.mul_i64(-3, 4).unwrap(), -12);
    assert_eq!(m.mul_i64(100_000, 100_000).unwrap(), 10_000_000_000i64);
}

#[test]
fn test_sub_i64() {
    let mut m = new_module();
    assert_eq!(m.sub_i64(10, 3).unwrap(), 7);
    assert_eq!(m.sub_i64(0, 0).unwrap(), 0);
    assert_eq!(m.sub_i64(3, 10).unwrap(), -7);
}

#[test]
fn test_div_i64_s() {
    let mut m = new_module();
    assert_eq!(m.div_i64_s(10, 3).unwrap(), 3);
    assert_eq!(m.div_i64_s(-10, 3).unwrap(), -3);
    assert_eq!(m.div_i64_s(100, -10).unwrap(), -10);
}

#[test]
fn test_div_i64_traps_on_zero() {
    let mut m = new_module();
    assert!(m.div_i64_s(1, 0).is_err());
}

#[test]
fn test_rem_i64_s() {
    let mut m = new_module();
    assert_eq!(m.rem_i64_s(10, 3).unwrap(), 1);
    assert_eq!(m.rem_i64_s(-10, 3).unwrap(), -1);
    assert_eq!(m.rem_i64_s(7, 7).unwrap(), 0);
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
    assert_eq!(m.shift_left_i64(1, 0).unwrap(), 1);
    assert_eq!(m.shift_right_s_i64(-1, 1).unwrap(), -1);
    assert_eq!(
        m.shift_right_u_i64(-1, 1).unwrap(),
        ((-1i64 as u64) >> 1) as i64,
    );
}

#[test]
fn test_negate_i64() {
    let mut m = new_module();
    assert_eq!(m.negate_i64(42).unwrap(), -42);
    assert_eq!(m.negate_i64(0).unwrap(), 0);
    assert_eq!(m.negate_i64(-1).unwrap(), 1);
}

// ── i64 algorithms ──

#[test]
fn test_fib_i64() {
    let mut m = new_module();
    assert_eq!(m.fib_i64(0).unwrap(), 0);
    assert_eq!(m.fib_i64(1).unwrap(), 1);
    assert_eq!(m.fib_i64(10).unwrap(), 55);
    assert_eq!(m.fib_i64(20).unwrap(), 6765);
    assert_eq!(m.fib_i64(50).unwrap(), 12586269025i64);
}

#[test]
fn test_factorial_i64() {
    let mut m = new_module();
    assert_eq!(m.factorial_i64(0).unwrap(), 1);
    assert_eq!(m.factorial_i64(1).unwrap(), 1);
    assert_eq!(m.factorial_i64(10).unwrap(), 3_628_800);
    assert_eq!(m.factorial_i64(20).unwrap(), 2_432_902_008_176_640_000i64);
}

// ── Cross-validation ──

#[test]
fn test_division_invariant_i64() {
    let mut m = new_module();
    for &(a, b) in &[(10i64, 3), (-10, 3), (10, -3), (100, 7)] {
        let q = m.div_i64_s(a, b).unwrap();
        let r = m.rem_i64_s(a, b).unwrap();
        assert_eq!(q * b + r, a, "invariant: ({a}/{b})*{b} + ({a}%{b}) == {a}");
    }
}
