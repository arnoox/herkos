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

criterion_group!(
    benches,
    fibo_5_wasm_bench,
    fibo_5_orig_bench,
    fibo_20_wasm_bench,
    fibo_20_orig_bench
);
criterion_main!(benches);
