fn sum_recursive_impl(n: i32) -> i32 {
    if n <= 0 {
        0
    } else {
        n + sum_recursive_impl(n - 1)
    }
}
