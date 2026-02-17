// Loop-heavy algorithms: power, collatz, primality, divisor sum

// Exponentiation by repeated multiplication
int power(int base, int exp) {
    int result = 1;
    while (exp > 0) {
        result *= base;
        exp--;
    }
    return result;
}

// Count steps in the Collatz sequence until reaching 1
int collatz_steps(int n) {
    int steps = 0;
    while (n > 1) {
        if (n & 1) {
            n = 3 * n + 1;
        } else {
            n = n >> 1;
        }
        steps++;
    }
    return steps;
}

// Trial division primality test. Returns 1 if prime, 0 if not.
// Avoids ternary / select by using a flag variable.
int is_prime(int n) {
    if (n < 2) { return 0; }
    int i = 2;
    while (i * i <= n) {
        if (n % i == 0) { return 0; }
        i++;
    }
    return 1;
}

// Count the number of primes <= n (simple sieve-free approach)
int count_primes(int n) {
    int count = 0;
    int i = 2;
    while (i <= n) {
        count += is_prime(i);
        i++;
    }
    return count;
}

// Sum of proper divisors of n (excluding n itself)
int sum_of_divisors(int n) {
    int sum = 0;
    int i = 1;
    while (i * i <= n) {
        if (n % i == 0) {
            sum += i;
            if (i != 1 && i != n / i) {
                sum += n / i;
            }
        }
        i++;
    }
    return sum;
}

// Check if n is a perfect number (sum of divisors == n)
int is_perfect(int n) {
    if (n < 2) { return 0; }
    int s = sum_of_divisors(n);
    // Avoid ternary: use subtraction trick
    // Returns 1 if s == n, 0 otherwise
    int diff = s - n;
    // (diff == 0) => !(diff | (-diff)) >> 31 & 1
    // Simpler: just use equality via subtraction and or
    return (diff == 0) & 1;
}

// Digital root: repeatedly sum digits until single digit
int digital_root(int n) {
    while (n >= 10) {
        int sum = 0;
        while (n > 0) {
            sum += n % 10;
            n /= 10;
        }
        n = sum;
    }
    return n;
}
