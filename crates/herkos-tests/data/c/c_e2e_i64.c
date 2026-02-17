// 64-bit arithmetic operations

long long mul_i64(long long a, long long b) { return a * b; }
long long sub_i64(long long a, long long b) { return a - b; }
long long div_i64_s(long long a, long long b) { return a / b; }
long long rem_i64_s(long long a, long long b) { return a % b; }

long long bitwise_and_i64(long long a, long long b) { return a & b; }
long long bitwise_or_i64(long long a, long long b) { return a | b; }
long long bitwise_xor_i64(long long a, long long b) { return a ^ b; }

long long shift_left_i64(long long a, long long b) { return a << b; }
long long shift_right_s_i64(long long a, long long b) { return a >> b; }
long long shift_right_u_i64(long long a, long long b) {
    return (long long)((unsigned long long)a >> (unsigned long long)b);
}

long long negate_i64(long long a) { return -a; }

// Fibonacci using i64 to test larger values
long long fib_i64(int n) {
    long long a = 0, b = 1;
    while (n > 0) {
        long long tmp = a + b;
        a = b;
        b = tmp;
        n--;
    }
    return a;
}

// i64 factorial
long long factorial_i64(int n) {
    long long result = 1;
    while (n > 1) {
        result *= (long long)n;
        n--;
    }
    return result;
}
