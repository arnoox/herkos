#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn add_i32(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

#[no_mangle]
pub extern "C" fn sum_recursive(n: i32) -> i32 {
  if n <= 0 {
    0
  } else {
    n + sum_recursive(n - 1)
  }
}

#[no_mangle]
pub extern "C" fn fibo(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        let mut a: i32 = 0;
        let mut b: i32 = 1;
        for _ in 2..=n {
            let tmp = a.wrapping_add(b);
            a = b;
            b = tmp;
        }
        b
    }
}

#[no_mangle]
pub extern "C" fn sub_i32(a: i32, b: i32) -> i32 {
    a.wrapping_sub(b)
}

#[no_mangle]
pub extern "C" fn mul_i32(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}

#[no_mangle]
pub extern "C" fn add_i64(a: i64, b: i64) -> i64 {
    a.wrapping_add(b)
}

#[no_mangle]
pub extern "C" fn bitwise_and(a: i32, b: i32) -> i32 {
    a & b
}

#[no_mangle]
pub extern "C" fn bitwise_or(a: i32, b: i32) -> i32 {
    a | b
}

#[no_mangle]
pub extern "C" fn bitwise_xor(a: i32, b: i32) -> i32 {
    a ^ b
}

#[no_mangle]
pub extern "C" fn shift_left(a: i32, b: i32) -> i32 {
    a.wrapping_shl(b as u32)
}

#[no_mangle]
pub extern "C" fn shift_right_u(a: i32, b: i32) -> i32 {
    ((a as u32).wrapping_shr(b as u32)) as i32
}

#[no_mangle]
pub extern "C" fn negate(a: i32) -> i32 {
    a.wrapping_neg()
}

#[no_mangle]
pub extern "C" fn const_42() -> i32 {
    42
}

/// (a + b) * (a - b) — tests that LLVM composes multiple operations.
#[no_mangle]
pub extern "C" fn diff_of_squares(a: i32, b: i32) -> i32 {
    a.wrapping_add(b).wrapping_mul(a.wrapping_sub(b))
}
