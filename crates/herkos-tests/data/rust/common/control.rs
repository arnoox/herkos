fn collatz_steps_impl(mut n: i32) -> i32 {
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

fn gcd_impl(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn popcount_impl(mut n: i32) -> i32 {
    let mut count: i32 = 0;
    let mut i: i32 = 0;
    while i < 32 {
        if n & 1 != 0 {
            count = count.wrapping_add(1);
        }
        n = ((n as u32) >> 1) as i32;
        i = i.wrapping_add(1);
    }
    count
}

fn isqrt_impl(n: i32) -> i32 {
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
