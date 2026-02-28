// Inter-module lending example
//
// This program demonstrates the herkos memory-lending pattern:
// - A host program owns memory (IsolatedMemory)
// - A transpiled WebAssembly "library module" borrows that memory
// - The host writes data, the library processes it, the host reads results
//
// The generated module (src/math_library_wasm.rs) contains no unsafe code.
// Memory isolation is enforced through the type system at compile time.
//
// Run `./run.sh` to regenerate math_library_wasm.rs and execute this program.

#[allow(dead_code)]
mod math_library_wasm;

use herkos_runtime::{IsolatedMemory, WasmResult};

/// Host that owns memory and provides import functions to the library module.
struct MathHost {
    results_log: Vec<i32>,
}

impl MathHost {
    fn new() -> Self {
        MathHost {
            results_log: Vec::new(),
        }
    }
}

impl math_library_wasm::EnvImports for MathHost {
    fn log_result(&mut self, value: i32) -> WasmResult<()> {
        self.results_log.push(value);
        println!("  [library logged: {}]", value);
        Ok(())
    }
}

fn main() {
    println!("=== Inter-Module Lending Example ===\n");

    // Step 1: Host creates owned memory (simulates a Module's IsolatedMemory)
    let mut memory = Box::new(IsolatedMemory::<4>::try_new(2).unwrap());
    let mut host = MathHost::new();
    let mut library = math_library_wasm::new().expect("library instantiation failed");

    // Step 2: Host writes an array of integers into memory
    let data = [10, 20, 30, 40, 50];
    println!("Host writes data to memory: {:?}", data);
    for (i, &val) in data.iter().enumerate() {
        memory.store_i32(i * 4, val).unwrap();
    }

    // Step 3: Library borrows memory to compute the sum
    let sum = library
        .sum_array(0, data.len() as i32, &mut *memory)
        .expect("sum_array trapped");
    println!("Library computed sum: {}", sum);

    // Step 4: Library doubles all values in-place
    println!("\nLibrary doubles array in-place...");
    library
        .double_array(0, data.len() as i32, &mut *memory)
        .expect("double_array trapped");

    // Step 5: Host reads back the modified values
    print!("Host reads back: [");
    for i in 0..data.len() {
        let val: i32 = memory.load_i32(i * 4).unwrap();
        if i > 0 {
            print!(", ");
        }
        print!("{}", val);
    }
    println!("]");

    // Step 6: Library sums and logs (uses both memory and host import)
    println!("\nLibrary sums doubled values and logs result:");
    let doubled_sum = library
        .sum_and_log(0, data.len() as i32, &mut *memory, &mut host)
        .expect("sum_and_log trapped");
    println!("Doubled sum: {}", doubled_sum);

    // Step 7: Show memory info
    let pages = library
        .memory_info(&mut *memory)
        .expect("memory_info trapped");
    println!("\nMemory size: {} pages ({} bytes)", pages, pages * 65536);

    println!("\n=== Done ===");
}
