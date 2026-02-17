#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn mul_i64(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}

#[no_mangle]
pub extern "C" fn sub_i64(a: i64, b: i64) -> i64 {
    a.wrapping_sub(b)
}

#[no_mangle]
pub extern "C" fn bitwise_and_i64(a: i64, b: i64) -> i64 {
    a & b
}

#[no_mangle]
pub extern "C" fn bitwise_or_i64(a: i64, b: i64) -> i64 {
    a | b
}

#[no_mangle]
pub extern "C" fn bitwise_xor_i64(a: i64, b: i64) -> i64 {
    a ^ b
}

#[no_mangle]
pub extern "C" fn shift_left_i64(a: i64, b: i64) -> i64 {
    a.wrapping_shl(b as u32)
}

#[no_mangle]
pub extern "C" fn shift_right_s_i64(a: i64, b: i64) -> i64 {
    a.wrapping_shr(b as u32)
}

#[no_mangle]
pub extern "C" fn negate_i64(a: i64) -> i64 {
    (0i64).wrapping_sub(a)
}

/// Iterative Fibonacci returning i64.
#[no_mangle]
pub extern "C" fn fib_i64(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut a: i64 = 0;
    let mut b: i64 = 1;
    let mut i: i64 = 1;
    while i < n {
        let tmp = b;
        b = a.wrapping_add(b);
        a = tmp;
        i = i.wrapping_add(1);
    }
    b
}

/// Iterative factorial returning i64.
#[no_mangle]
pub extern "C" fn factorial_i64(n: i64) -> i64 {
    let mut result: i64 = 1;
    let mut i: i64 = 2;
    while i <= n {
        result = result.wrapping_mul(i);
        i = i.wrapping_add(1);
    }
    result
}
