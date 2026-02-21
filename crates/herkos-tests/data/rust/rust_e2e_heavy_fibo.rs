#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};

// ── Panic handler ─────────────────────────────────────────────────────────────

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// ── Bump allocator ────────────────────────────────────────────────────────────
//
// A simple single-threaded bump allocator backed by a static byte array.
// Memory is carved out by advancing a cursor; individual frees are no-ops.
// This is appropriate for a `no_std` Wasm module that runs in a single thread
// and has bounded, predictable allocation needs.

const HEAP_SIZE: usize = 16 * 1024; // 16 KiB — ample for thousands of cached i32 values

static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];
static mut HEAP_CURSOR: usize = 0;

struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        // Round cursor up to the required alignment.
        let aligned = (HEAP_CURSOR + align - 1) & !(align - 1);
        let new_cursor = aligned + size;
        if new_cursor > HEAP_SIZE {
            return core::ptr::null_mut(); // out of heap space
        }
        HEAP_CURSOR = new_cursor;
        HEAP.as_mut_ptr().add(aligned)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocators never free individual allocations.
    }
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

// ── Fibonacci cache ───────────────────────────────────────────────────────────
//
// `CACHE` is a lazily-initialised `Vec` that stores:
//   CACHE[0] = fibo(0), CACHE[1] = fibo(1), …, CACHE[k-1] = fibo(k-1)
//
// On each call to `fibo(n)`:
//   • If k > n  — the value is already cached; return it directly  (O(1)).
//   • If k <= n — extend the cache from index k up to n, then return (O(m)).
//
// All arithmetic uses wrapping i32 semantics to match WebAssembly behaviour.

static mut CACHE: Option<Vec<i32>> = None;

/// Return a mutable reference to the cache, initialising it on first access.
fn cache() -> &'static mut Vec<i32> {
    // SAFETY: WebAssembly modules are single-threaded; there is no concurrent
    // access to `CACHE`.  We only call this function from exported entry points
    // that Wasm guarantees are not re-entered.
    unsafe {
        if CACHE.is_none() {
            let mut v = Vec::with_capacity(64);
            v.push(0i32); // fibo(0)
            v.push(1i32); // fibo(1)
            CACHE = Some(v);
        }
        // SAFETY: we just guaranteed `CACHE` is `Some`.
        CACHE.as_mut().unwrap_unchecked()
    }
}

/// Return `fibo(n)` using wrapping i32 arithmetic (matches Wasm semantics).
///
/// All previously unseen values from `fibo(cache_len)` up to `fibo(n)` are
/// computed once and appended to the cache.  Subsequent calls with the same
/// or a smaller `n` are served directly from the cache without recomputation.
#[no_mangle]
pub extern "C" fn fibo(n: i32) -> i32 {
    if n < 0 {
        return 0;
    }
    let n = n as usize;
    let v = cache();
    while v.len() <= n {
        let k = v.len();
        let next = v[k - 2].wrapping_add(v[k - 1]);
        v.push(next);
    }
    v[n]
}

/// Return the number of Fibonacci values currently stored in the cache.
///
/// Calling this function before any `fibo` call will return 2 because the
/// cache is pre-seeded with `fibo(0) = 0` and `fibo(1) = 1`.
#[no_mangle]
pub extern "C" fn fibo_cache_len() -> i32 {
    cache().len() as i32
}
