#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

include!("common/fill_sort_sum.rs");

// Work buffer in the Wasm data segment (zero-initialized, goes to BSS).
// 1024 i32s = 4 KiB — fits comfortably in 2-page memory (128 KiB).
static mut BUF: [i32; 1024] = [0i32; 1024];

/// Fill the work buffer with pseudo-random values, bubble sort them in-place,
/// and return a wrapping checksum (sum of all sorted values).
///
/// Memory access profile (for `n` elements):
///   - Fill:  n stores
///   - Sort:  O(n²) loads + conditional stores  (bubble sort)
///   - Sum:   n loads
///
/// `n`:    number of i32 elements to process (capped at 1024)
/// `seed`: initial value for the LCG pseudo-random generator
#[no_mangle]
pub extern "C" fn mem_fill_sort_sum(n: i32, seed: i32) -> i32 {
    unsafe { fill_sort_sum_impl(&mut BUF, n, seed) }
}
