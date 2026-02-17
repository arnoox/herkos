//! End-to-end tests: C → Wasm → Rust (loop-heavy algorithms).

use herkos_tests::c_e2e_loops;

fn new_module() -> c_e2e_loops::WasmModule {
    c_e2e_loops::new().expect("module instantiation should succeed")
}

// ── Power ──

#[test]
fn test_power() {
    let mut m = new_module();
    assert_eq!(m.power(2, 0).unwrap(), 1);
    assert_eq!(m.power(2, 1).unwrap(), 2);
    assert_eq!(m.power(2, 10).unwrap(), 1024);
    assert_eq!(m.power(3, 5).unwrap(), 243);
    assert_eq!(m.power(1, 100).unwrap(), 1);
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
    assert_eq!(m.collatz_steps(6).unwrap(), 8);
    assert_eq!(m.collatz_steps(27).unwrap(), 111);
}

// ── Primality ──

#[test]
fn test_is_prime() {
    let mut m = new_module();
    assert_eq!(m.is_prime(0).unwrap(), 0);
    assert_eq!(m.is_prime(1).unwrap(), 0);
    assert_eq!(m.is_prime(2).unwrap(), 1);
    assert_eq!(m.is_prime(3).unwrap(), 1);
    assert_eq!(m.is_prime(4).unwrap(), 0);
    assert_eq!(m.is_prime(5).unwrap(), 1);
    assert_eq!(m.is_prime(17).unwrap(), 1);
    assert_eq!(m.is_prime(18).unwrap(), 0);
    assert_eq!(m.is_prime(97).unwrap(), 1);
    assert_eq!(m.is_prime(100).unwrap(), 0);
}

#[test]
fn test_count_primes() {
    let mut m = new_module();
    assert_eq!(m.count_primes(1).unwrap(), 0);
    assert_eq!(m.count_primes(10).unwrap(), 4); // 2,3,5,7
    assert_eq!(m.count_primes(20).unwrap(), 8); // 2,3,5,7,11,13,17,19
    assert_eq!(m.count_primes(100).unwrap(), 25);
}

// ── Divisors ──

#[test]
fn test_sum_of_divisors() {
    let mut m = new_module();
    // sum_of_divisors returns sum of proper divisors (excluding n)
    assert_eq!(m.sum_of_divisors(1).unwrap(), 1); // just 1
    assert_eq!(m.sum_of_divisors(6).unwrap(), 6); // 1+2+3
    assert_eq!(m.sum_of_divisors(28).unwrap(), 28); // 1+2+4+7+14
    assert_eq!(m.sum_of_divisors(12).unwrap(), 16); // 1+2+3+4+6
}

#[test]
fn test_is_perfect() {
    let mut m = new_module();
    assert_eq!(m.is_perfect(6).unwrap(), 1);
    assert_eq!(m.is_perfect(28).unwrap(), 1);
    assert_eq!(m.is_perfect(496).unwrap(), 1);
    assert_eq!(m.is_perfect(10).unwrap(), 0);
    assert_eq!(m.is_perfect(1).unwrap(), 0);
}

// ── Digital root ──

#[test]
fn test_digital_root() {
    let mut m = new_module();
    assert_eq!(m.digital_root(0).unwrap(), 0);
    assert_eq!(m.digital_root(5).unwrap(), 5);
    assert_eq!(m.digital_root(9).unwrap(), 9);
    assert_eq!(m.digital_root(10).unwrap(), 1);
    assert_eq!(m.digital_root(99).unwrap(), 9);
    assert_eq!(m.digital_root(123).unwrap(), 6); // 1+2+3=6
    assert_eq!(m.digital_root(9999).unwrap(), 9);
}
