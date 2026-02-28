use anyhow::{Context, Result};
use clap::Parser;
use herkos::{transpile, TranspileOptions};
use std::fs;
use std::path::PathBuf;

/// herkos — WebAssembly to Rust transpiler with compile-time isolation guarantees.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Input WebAssembly binary (.wasm)
    input: PathBuf,

    /// Output Rust source file
    #[arg(long, short)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    eprintln!("herkos: transpiling {}", cli.input.display(),);

    // Read WASM file
    let wasm_bytes =
        fs::read(&cli.input).with_context(|| format!("failed to read {}", cli.input.display()))?;

    // Configure transpilation options
    let options = TranspileOptions {
        mode: "safe".to_string(),
        max_pages: 256,
    };

    // Transpile using library function
    let rust_code = transpile(&wasm_bytes, &options).context("transpilation failed")?;

    // Write output
    if let Some(output_path) = cli.output {
        fs::write(&output_path, &rust_code)
            .with_context(|| format!("failed to write {}", output_path.display()))?;
        eprintln!("herkos: wrote {}", output_path.display());
    } else {
        // Print to stdout if no output file specified
        print!("{}", rust_code);
    }

    eprintln!("herkos: transpilation complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_defaults() {
        let cli = Cli::parse_from(["herkos", "input.wasm"]);
        assert_eq!(cli.input, PathBuf::from("input.wasm"));
        assert!(cli.output.is_none());
    }
}
