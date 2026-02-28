// C → WebAssembly → Rust example
//
// This program uses a module that was originally written in C, compiled to
// WebAssembly, and then transpiled to memory-safe Rust by herkos.
//
// The generated module (src/fibonacci_wasm.rs) contains no unsafe code and
// enforces memory isolation through the type system at compile time.
//
// Run `./run.sh` to regenerate fibonacci_wasm.rs and execute this program.

#[allow(dead_code)]
mod fibonacci_wasm;

fn main() {
    let mut module = fibonacci_wasm::new().expect("module instantiation failed");

    // Fibonacci sequence
    println!("Fibonacci sequence:");
    for n in 0..=15 {
        let result = module.fibonacci(n).expect("fibonacci trapped");
        println!("  F({n}) = {result}");
    }

    // Factorials
    println!("\nFactorials:");
    for n in [0, 1, 5, 10, 12] {
        let result = module.factorial(n).expect("factorial trapped");
        println!("  {n}! = {result}");
    }

    // GCD
    println!("\nGreatest common divisor:");
    for (a, b) in [(12, 8), (100, 75), (17, 13)] {
        let result = module.gcd(a, b).expect("gcd trapped");
        println!("  gcd({a}, {b}) = {result}");
    }

    // Basic arithmetic
    println!("\nArithmetic:");
    println!("  add(40, 2) = {}", module.add(40, 2).expect("add trapped"));
    println!("  mul(6, 7) = {}", module.mul(6, 7).expect("mul trapped"));
}
