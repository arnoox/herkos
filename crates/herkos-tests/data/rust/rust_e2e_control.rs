#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

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

/// Collatz conjecture: count steps until n reaches 1.
#[no_mangle]
pub extern "C" fn collatz_steps(mut n: i32) -> i32 {
    let mut steps: i32 = 0;
    while n != 1 {
        if n & 1 == 0 {
            n >>= 1;
        } else {
            n = n.wrapping_mul(3).wrapping_add(1);
        }
        steps = steps.wrapping_add(1);
    }
    steps
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

/// Greatest common divisor via Euclidean algorithm.
#[no_mangle]
pub extern "C" fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
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

/// Population count (number of set bits).
#[no_mangle]
pub extern "C" fn popcount(mut n: i32) -> i32 {
    let mut count: i32 = 0;
    // Process all 32 bits by iterating 32 times
    let mut i: i32 = 0;
    while i < 32 {
        if n & 1 != 0 {
            count = count.wrapping_add(1);
        }
        // Use unsigned shift right to avoid sign extension
        n = ((n as u32) >> 1) as i32;
        i = i.wrapping_add(1);
    }
    count
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

/// Integer square root via binary search.
#[no_mangle]
pub extern "C" fn isqrt(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    let mut low: i32 = 1;
    let mut high: i32 = n;
    let mut result: i32 = 0;
    while low <= high {
        let mid = low.wrapping_add((high.wrapping_sub(low)) >> 1);
        let sq = mid.wrapping_mul(mid);
        if sq == n {
            return mid;
        } else if sq < n {
            result = mid;
            low = mid.wrapping_add(1);
        } else {
            high = mid.wrapping_sub(1);
        }
    }
    result
}
