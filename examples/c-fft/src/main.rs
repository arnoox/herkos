// C → WebAssembly → Rust FFT example
//
// This program drives a 4096-point radix-2 FFT that was originally written in C,
// compiled to WebAssembly, and then transpiled to memory-safe Rust by herkos.
//
// The generated module (src/fft_wasm.rs) contains no unsafe code.
// Memory access is bounds-checked; isolation is enforced by the Rust type system.
//
// Run `./run.sh` to regenerate fft_wasm.rs and execute this program.

#[allow(dead_code)]
mod fft_wasm;

use std::time::Instant;

const N: usize = 4096;
const N_HALF: usize = N / 2;
const SAMPLE_RATE: f32 = 44100.0;

fn main() {
    let mut module = fft_wasm::new().expect("FFT module instantiation failed");

    // Initialize twiddle table (called once)
    module.fft_init(N as i32).expect("fft_init trapped");

    // Get the pointer (Wasm byte offset) to the input buffer
    let input_ptr = module.fft_get_input_ptr().expect("fft_get_input_ptr trapped") as usize;
    let output_ptr = module.fft_get_output_ptr().expect("fft_get_output_ptr trapped") as usize;

    println!("=== herkos C-FFT Example ===");
    println!("    4096-point radix-2 DIT FFT, C → Wasm → memory-safe Rust");
    println!("    Input buffer:  Wasm byte offset 0x{:05X}", input_ptr);
    println!("    Output buffer: Wasm byte offset 0x{:05X}", output_ptr);
    println!();

    // ── Test 1: Single tone at 1 kHz ─────────────────────────────────────────
    println!("--- Test 1: Single tone at 1000 Hz ---");
    write_tone(&mut module, input_ptr, &[(1000.0, 1.0)]);
    let elapsed = run_fft(&mut module);
    let magnitudes = read_magnitudes(&module, output_ptr);
    print_spectrum(&magnitudes, "1000 Hz tone", elapsed);
    let peak = find_peak_bin(&magnitudes);
    println!(
        "    Peak bin: {} → {:.1} Hz",
        peak,
        bin_to_hz(peak)
    );
    println!();

    // ── Test 2: Two tones at 440 Hz and 2000 Hz ───────────────────────────────
    println!("--- Test 2: Two tones (440 Hz + 2000 Hz) ---");
    write_tone(&mut module, input_ptr, &[(440.0, 0.8), (2000.0, 0.5)]);
    let elapsed = run_fft(&mut module);
    let magnitudes = read_magnitudes(&module, output_ptr);
    print_spectrum(&magnitudes, "440 Hz + 2000 Hz", elapsed);
    let peaks = find_top_bins(&magnitudes, 3);
    println!("    Top bins:");
    for (bin, mag) in &peaks {
        println!("      bin {:4} → {:7.1} Hz  mag={:.1}", bin, bin_to_hz(*bin), mag);
    }
    println!();

    // ── Test 3: Tone + harmonic ──────────────────────────────────────────────
    println!("--- Test 3: 3520 Hz tone + 880 Hz harmonic ---");
    write_tone(&mut module, input_ptr, &[(3520.0, 1.0), (880.0, 0.3)]);
    let elapsed = run_fft(&mut module);
    let magnitudes = read_magnitudes(&module, output_ptr);
    print_spectrum(&magnitudes, "3520 Hz + 880 Hz", elapsed);
    println!();
}

/// Write a sum of sinusoids into the FFT input buffer.
/// `freqs`: slice of (frequency_hz, amplitude) pairs.
/// Input is interleaved complex: [re_0, im_0, re_1, im_1, ...], imaginary parts = 0.
fn write_tone(module: &mut fft_wasm::WasmModule, input_ptr: usize, freqs: &[(f32, f32)]) {
    use std::f32::consts::PI;
    for i in 0..N {
        let t = i as f32 / SAMPLE_RATE;
        let mut sample = 0.0f32;
        for &(freq, amp) in freqs {
            sample += amp * (2.0 * PI * freq * t).sin();
        }
        let re_offset = input_ptr + i * 8;      // 8 bytes per complex (re+im f32)
        let im_offset = input_ptr + i * 8 + 4;
        module.0.memory.store_f32(re_offset, sample).expect("store real");
        module.0.memory.store_f32(im_offset, 0.0f32).expect("store imag");
    }
}

/// Run the FFT and return elapsed wall-clock time.
fn run_fft(module: &mut fft_wasm::WasmModule) -> std::time::Duration {
    let start = Instant::now();
    module.fft_compute(N as i32).expect("fft_compute trapped");
    start.elapsed()
}

/// Read N/2 magnitude values from the output buffer.
fn read_magnitudes(module: &fft_wasm::WasmModule, output_ptr: usize) -> Vec<f32> {
    (0..N_HALF)
        .map(|k| {
            module
                .0
                .memory
                .load_f32(output_ptr + k * 4)
                .expect("load magnitude")
        })
        .collect()
}

/// Find the bin with the highest magnitude.
fn find_peak_bin(magnitudes: &[f32]) -> usize {
    magnitudes
        .iter()
        .enumerate()
        .skip(1) // skip DC
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Find the top `n` bins by magnitude, sorted descending.
fn find_top_bins(magnitudes: &[f32], n: usize) -> Vec<(usize, f32)> {
    let mut indexed: Vec<(usize, f32)> = magnitudes
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &m)| (i, m))
        .collect();
    indexed.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    indexed.truncate(n);
    indexed
}

/// Convert bin index to frequency in Hz.
fn bin_to_hz(bin: usize) -> f32 {
    bin as f32 * SAMPLE_RATE / N as f32
}

/// Print a compact ASCII spectrum showing the top bins.
fn print_spectrum(magnitudes: &[f32], label: &str, elapsed: std::time::Duration) {
    println!("    Spectrum ({label}) — {:.2?}", elapsed);

    // Find global max for normalization (skip DC bin 0)
    let max_mag = magnitudes[1..].iter().cloned().fold(0.0f32, f32::max);
    if max_mag <= 0.0 {
        println!("    (empty spectrum)");
        return;
    }

    // Collect top 12 bins
    let mut top: Vec<(usize, f32)> = magnitudes
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &m)| (i, m))
        .filter(|(_, m)| *m > max_mag * 0.05) // threshold at 5% of peak
        .collect();
    top.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    top.truncate(12);
    top.sort_by_key(|(bin, _)| *bin); // re-sort by frequency for display

    const BAR_WIDTH: usize = 30;
    for (bin, mag) in &top {
        let bar_len = ((mag / max_mag) * BAR_WIDTH as f32) as usize;
        let bar: String = "#".repeat(bar_len);
        println!(
            "    {:5.0} Hz (bin {:4}): {:30} {:.1}",
            bin_to_hz(*bin),
            bin,
            bar,
            mag
        );
    }
}
