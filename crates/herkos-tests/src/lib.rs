// Include generated modules from build.rs (in OUT_DIR)
include!(concat!(env!("OUT_DIR"), "/mod.rs"));

// Shared algorithm implementations used by both the transpiled Wasm modules
// and the native Rust baselines below.
include!("../data/rust/common/fibo.rs");
include!("../data/rust/common/fill_sort_sum.rs");
include!("../data/rust/common/control.rs");
include!("../data/rust/common/sum_recursive.rs");

pub fn fibo_orig(n: i32) -> i32 {
    fibo_impl(n)
}

/// Native Rust baseline for the memory-intensive fill+sort+sum benchmark.
///
/// Uses a stack-allocated array with direct indexing (no bounds checks in
/// release mode). This is the "best case" that the transpiled Wasm version
/// is compared against.
pub fn mem_fill_sort_sum_orig(n: i32, seed: i32) -> i32 {
    let mut buf = [0i32; 1024];
    fill_sort_sum_impl(&mut buf, n, seed)
}

pub fn collatz_steps_orig(n: i32) -> i32 {
    collatz_steps_impl(n)
}

pub fn isqrt_orig(n: i32) -> i32 {
    isqrt_impl(n)
}

pub fn gcd_orig(a: i32, b: i32) -> i32 {
    gcd_impl(a, b)
}

pub fn popcount_orig(n: i32) -> i32 {
    popcount_impl(n)
}

pub fn sum_recursive_orig(n: i32) -> i32 {
    sum_recursive_impl(n)
}
