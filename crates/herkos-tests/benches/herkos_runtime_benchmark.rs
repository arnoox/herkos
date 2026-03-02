use criterion::{criterion_group, criterion_main, Criterion};
use herkos_tests::*;
use std::hint::black_box;

fn fibo_5_wasm_bench(c: &mut Criterion) {
    let mut m = rust_e2e_arith::new().unwrap();
    c.bench_function("fib 5 wasm transpiled to rust", |b| {
        b.iter(|| m.fibo(black_box(5)))
    });
}

fn fibo_5_orig_bench(c: &mut Criterion) {
    c.bench_function("fib 5 plain rust", |b| b.iter(|| fibo_orig(black_box(5))));
}

fn fibo_20_wasm_bench(c: &mut Criterion) {
    let mut m = rust_e2e_arith::new().unwrap();
    c.bench_function("fib 20 wasm transpiled to rust", |b| {
        b.iter(|| m.fibo(black_box(20)))
    });
}

fn fibo_20_orig_bench(c: &mut Criterion) {
    c.bench_function("fib 20 plain rust", |b| b.iter(|| fibo_orig(black_box(20))));
}

// ─── Memory-intensive benchmarks ─────────────────────────────────────────────

fn memsort_100_wasm_bench(c: &mut Criterion) {
    let mut m = rust_e2e_memory_bench::new().unwrap();
    c.bench_function("memsort 100 wasm transpiled to rust", |b| {
        b.iter(|| m.mem_fill_sort_sum(black_box(100), black_box(42)))
    });
}

fn memsort_100_orig_bench(c: &mut Criterion) {
    c.bench_function("memsort 100 plain rust", |b| {
        b.iter(|| mem_fill_sort_sum_orig(black_box(100), black_box(42)))
    });
}

// ─── Control-flow-heavy benchmarks ──────────────────────────────────────────

fn collatz_27_wasm_bench(c: &mut Criterion) {
    let mut m = rust_e2e_control::new().unwrap();
    c.bench_function("collatz 27 wasm transpiled to rust", |b| {
        b.iter(|| m.collatz_steps(black_box(27)))
    });
}

fn collatz_27_orig_bench(c: &mut Criterion) {
    c.bench_function("collatz 27 plain rust", |b| {
        b.iter(|| collatz_steps_orig(black_box(27)))
    });
}

fn collatz_871_wasm_bench(c: &mut Criterion) {
    let mut m = rust_e2e_control::new().unwrap();
    c.bench_function("collatz 871 wasm transpiled to rust", |b| {
        b.iter(|| m.collatz_steps(black_box(871)))
    });
}

fn collatz_871_orig_bench(c: &mut Criterion) {
    c.bench_function("collatz 871 plain rust", |b| {
        b.iter(|| collatz_steps_orig(black_box(871)))
    });
}

// ─── Search / math benchmarks ───────────────────────────────────────────────

fn isqrt_wasm_bench(c: &mut Criterion) {
    let mut m = rust_e2e_control::new().unwrap();
    c.bench_function("isqrt 1000000 wasm transpiled to rust", |b| {
        b.iter(|| m.isqrt(black_box(1_000_000)))
    });
}

fn isqrt_orig_bench(c: &mut Criterion) {
    c.bench_function("isqrt 1000000 plain rust", |b| {
        b.iter(|| isqrt_orig(black_box(1_000_000)))
    });
}

fn gcd_wasm_bench(c: &mut Criterion) {
    let mut m = rust_e2e_control::new().unwrap();
    c.bench_function("gcd(46368,28657) wasm transpiled to rust", |b| {
        b.iter(|| m.gcd(black_box(46368), black_box(28657)))
    });
}

fn gcd_orig_bench(c: &mut Criterion) {
    c.bench_function("gcd(46368,28657) plain rust", |b| {
        b.iter(|| gcd_orig(black_box(46368), black_box(28657)))
    });
}

// ─── Bitwise benchmarks ─────────────────────────────────────────────────────

fn popcount_wasm_bench(c: &mut Criterion) {
    let mut m = rust_e2e_control::new().unwrap();
    c.bench_function("popcount 0xDEADBEEF wasm transpiled to rust", |b| {
        b.iter(|| m.popcount(black_box(0xDEADBEEFu32 as i32)))
    });
}

fn popcount_orig_bench(c: &mut Criterion) {
    c.bench_function("popcount 0xDEADBEEF plain rust", |b| {
        b.iter(|| popcount_orig(black_box(0xDEADBEEFu32 as i32)))
    });
}

// ─── Recursive call overhead ────────────────────────────────────────────────

fn sum_recursive_100_wasm_bench(c: &mut Criterion) {
    let mut m = rust_e2e_arith::new().unwrap();
    c.bench_function("sum_recursive 100 wasm transpiled to rust", |b| {
        b.iter(|| m.sum_recursive(black_box(100)))
    });
}

fn sum_recursive_100_orig_bench(c: &mut Criterion) {
    c.bench_function("sum_recursive 100 plain rust", |b| {
        b.iter(|| sum_recursive_orig(black_box(100)))
    });
}

criterion_group!(
    benches,
    // Fibonacci (pure computation)
    fibo_5_wasm_bench,
    fibo_5_orig_bench,
    fibo_20_wasm_bench,
    fibo_20_orig_bench,
    // Memory-intensive (bounds-checking overhead)
    memsort_100_wasm_bench,
    memsort_100_orig_bench,
    // Branchy control flow (division + conditionals)
    collatz_27_wasm_bench,
    collatz_27_orig_bench,
    collatz_871_wasm_bench,
    collatz_871_orig_bench,
    // Binary search loop (multiplication + comparison)
    isqrt_wasm_bench,
    isqrt_orig_bench,
    // Euclidean algorithm (modular arithmetic loop)
    gcd_wasm_bench,
    gcd_orig_bench,
    // Bitwise tight loop
    popcount_wasm_bench,
    popcount_orig_bench,
    // Recursive function call overhead
    sum_recursive_100_wasm_bench,
    sum_recursive_100_orig_bench,
);
criterion_main!(benches);
