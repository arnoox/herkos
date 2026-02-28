fn fill_sort_sum_impl(buf: &mut [i32; 1024], n: i32, seed: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    let n = (n as usize).min(buf.len());

    // Fill with LCG pseudo-random values
    let mut rng = seed;
    for item in buf.iter_mut().take(n) {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        *item = rng;
    }

    // Bubble sort — indexing required for random-access swaps
    for i in 0..n {
        for j in 0..(n - 1 - i) {
            if buf[j] > buf[j + 1] {
                buf.swap(j, j + 1);
            }
        }
    }

    // Wrapping checksum
    let mut sum: i32 = 0;
    for item in buf.iter().take(n) {
        sum = sum.wrapping_add(*item);
    }
    sum
}
