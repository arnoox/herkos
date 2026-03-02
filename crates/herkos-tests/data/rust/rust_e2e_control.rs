#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

include!("common/control.rs");

/// Integer power: base^exp via iterative squaring.
#[no_mangle]
pub extern "C" fn power(mut base: i32, mut exp: i32) -> i32 {
    let mut result: i32 = 1;
    while exp > 0 {
        if exp & 1 != 0 {
            result = result.wrapping_mul(base);
        }
        base = base.wrapping_mul(base);
        exp >>= 1;
    }
    result
}

#[no_mangle]
pub extern "C" fn collatz_steps(n: i32) -> i32 {
    collatz_steps_impl(n)
}

/// Digital root: repeatedly sum digits until single digit.
#[no_mangle]
pub extern "C" fn digital_root(mut n: i32) -> i32 {
    while n >= 10 {
        let mut sum: i32 = 0;
        while n > 0 {
            sum = sum.wrapping_add(n % 10);
            n /= 10;
        }
        n = sum;
    }
    n
}

#[no_mangle]
pub extern "C" fn gcd(a: i32, b: i32) -> i32 {
    gcd_impl(a, b)
}

/// Least common multiple.
#[no_mangle]
pub extern "C" fn lcm(a: i32, b: i32) -> i32 {
    if a == 0 || b == 0 {
        return 0;
    }
    let g = gcd(a, b);
    (a / g).wrapping_mul(b)
}

#[no_mangle]
pub extern "C" fn popcount(n: i32) -> i32 {
    popcount_impl(n)
}

/// Check if n is a power of two.
#[no_mangle]
pub extern "C" fn is_power_of_two(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    if n & (n.wrapping_sub(1)) == 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn isqrt(n: i32) -> i32 {
    isqrt_impl(n)
}
