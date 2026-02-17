// Freestanding C for wasm32 — no libc, no stdlib
// Pure arithmetic, bitwise, shift, loop, and recursive functions.

// ── Basic arithmetic ──

int add_i32(int a, int b) { return a + b; }
int sub_i32(int a, int b) { return a - b; }
int mul_i32(int a, int b) { return a * b; }
int negate(int a) { return -a; }
int const_42(void) { return 42; }
int diff_of_squares(int a, int b) { return (a + b) * (a - b); }

// ── Division and remainder ──

int div_s(int a, int b) { return a / b; }
int rem_s(int a, int b) { return a % b; }

// ── Bitwise ──

int bitwise_and(int a, int b) { return a & b; }
int bitwise_or(int a, int b)  { return a | b; }
int bitwise_xor(int a, int b) { return a ^ b; }

// ── Shifts ──

int shift_left(int a, int b)    { return a << b; }
// Unsigned shift right: cast to unsigned, shift, cast back
int shift_right_u(int a, int b) { return (int)((unsigned)a >> (unsigned)b); }

// ── i64 (long long) ──

long long add_i64(long long a, long long b) { return a + b; }

// ── Loops ──

int factorial(int n) {
    int result = 1;
    while (n > 1) {
        result *= n;
        n--;
    }
    return result;
}

int sum_1_to_n(int n) {
    int sum = 0;
    while (n > 0) {
        sum += n;
        n--;
    }
    return sum;
}

// ── Recursive ──

int gcd(int a, int b) {
    while (b != 0) {
        int tmp = b;
        b = a % b;
        a = tmp;
    }
    return a;
}
