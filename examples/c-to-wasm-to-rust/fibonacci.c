// fibonacci.c — Freestanding C for wasm32 (no libc, no stdlib)
//
// This file demonstrates plain C functions that get compiled to WebAssembly
// and then transpiled to memory-safe Rust by herkos.

// Iterative Fibonacci: returns the n-th Fibonacci number.
// F(0)=0, F(1)=1, F(2)=1, F(3)=2, F(4)=3, F(5)=5, ...
int fibonacci(int n) {
    if (n <= 0) return 0;
    if (n == 1) return 1;

    int a = 0;
    int b = 1;
    int i = 2;
    while (i <= n) {
        int tmp = a + b;
        a = b;
        b = tmp;
        i++;
    }
    return b;
}

// Factorial: returns n! (iterative)
int factorial(int n) {
    int result = 1;
    while (n > 1) {
        result *= n;
        n--;
    }
    return result;
}

// Greatest common divisor (Euclidean algorithm)
int gcd(int a, int b) {
    while (b != 0) {
        int tmp = b;
        b = a % b;
        a = tmp;
    }
    return a;
}

// Simple arithmetic for demonstration
int add(int a, int b) { return a + b; }
int mul(int a, int b) { return a * b; }
