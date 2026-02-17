//! End-to-end tests: Rust → Wasm → Rust (control flow and algorithms).

use herkos_tests::rust_e2e_control;

fn new_module() -> rust_e2e_control::WasmModule {
    rust_e2e_control::new().expect("module instantiation should succeed")
}

// ── Power ──

#[test]
fn test_power() {
    let mut m = new_module();
    assert_eq!(m.power(2, 0).unwrap(), 1);
    assert_eq!(m.power(2, 10).unwrap(), 1024);
    assert_eq!(m.power(3, 5).unwrap(), 243);
    assert_eq!(m.power(0, 5).unwrap(), 0);
    assert_eq!(m.power(-1, 3).unwrap(), -1);
    assert_eq!(m.power(-1, 4).unwrap(), 1);
}

// ── Collatz ──

#[test]
fn test_collatz_steps() {
    let mut m = new_module();
    assert_eq!(m.collatz_steps(1).unwrap(), 0);
    assert_eq!(m.collatz_steps(2).unwrap(), 1);
    assert_eq!(m.collatz_steps(3).unwrap(), 7);
    assert_eq!(m.collatz_steps(27).unwrap(), 111);
}

// ── Digital root ──

#[test]
fn test_digital_root() {
    let mut m = new_module();
    assert_eq!(m.digital_root(0).unwrap(), 0);
    assert_eq!(m.digital_root(5).unwrap(), 5);
    assert_eq!(m.digital_root(123).unwrap(), 6);
    assert_eq!(m.digital_root(9999).unwrap(), 9);
}

// ── GCD and LCM ──

#[test]
fn test_gcd() {
    let mut m = new_module();
    assert_eq!(m.gcd(12, 8).unwrap(), 4);
    assert_eq!(m.gcd(17, 13).unwrap(), 1);
    assert_eq!(m.gcd(100, 25).unwrap(), 25);
    assert_eq!(m.gcd(7, 0).unwrap(), 7);
}

#[test]
fn test_lcm() {
    let mut m = new_module();
    assert_eq!(m.lcm(4, 6).unwrap(), 12);
    assert_eq!(m.lcm(3, 5).unwrap(), 15);
    assert_eq!(m.lcm(7, 7).unwrap(), 7);
    assert_eq!(m.lcm(0, 5).unwrap(), 0);
}

// ── Popcount ──

#[test]
fn test_popcount() {
    let mut m = new_module();
    assert_eq!(m.popcount(0).unwrap(), 0);
    assert_eq!(m.popcount(1).unwrap(), 1);
    assert_eq!(m.popcount(0xFF).unwrap(), 8);
    assert_eq!(m.popcount(-1).unwrap(), 32); // all bits set
    assert_eq!(m.popcount(0x5555_5555).unwrap(), 16);
}

// ── Is power of two ──

#[test]
fn test_is_power_of_two() {
    let mut m = new_module();
    assert_eq!(m.is_power_of_two(0).unwrap(), 0);
    assert_eq!(m.is_power_of_two(1).unwrap(), 1);
    assert_eq!(m.is_power_of_two(2).unwrap(), 1);
    assert_eq!(m.is_power_of_two(3).unwrap(), 0);
    assert_eq!(m.is_power_of_two(4).unwrap(), 1);
    assert_eq!(m.is_power_of_two(1024).unwrap(), 1);
    assert_eq!(m.is_power_of_two(-1).unwrap(), 0);
}

// ── Integer square root ──

#[test]
fn test_isqrt() {
    let mut m = new_module();
    assert_eq!(m.isqrt(0).unwrap(), 0);
    assert_eq!(m.isqrt(1).unwrap(), 1);
    assert_eq!(m.isqrt(4).unwrap(), 2);
    assert_eq!(m.isqrt(9).unwrap(), 3);
    assert_eq!(m.isqrt(10).unwrap(), 3); // floor(sqrt(10))
    assert_eq!(m.isqrt(100).unwrap(), 10);
    assert_eq!(m.isqrt(10000).unwrap(), 100);
}

// ── Cross-validation with C E2E ──

#[test]
fn test_matches_c_collatz() {
    // Known Collatz steps
    let expected = [(1, 0), (2, 1), (3, 7), (6, 8), (27, 111)];
    let mut m = new_module();
    for &(n, steps) in &expected {
        assert_eq!(m.collatz_steps(n).unwrap(), steps, "collatz_steps({n})");
    }
}
