#!/usr/bin/env bash
#
# WAT → WebAssembly → Rust inter-module lending example
#
# Prerequisites:
#   - wabt (for wat2wasm): apt-get install wabt
#   - Rust toolchain (cargo)
#   - herkos CLI (cargo install --path ../../crates/herkos)
#
# Usage:
#   ./run.sh          # build and run
#   ./run.sh --clean  # remove generated artifacts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

WAT_FILE="math_library.wat"
WASM_FILE="math_library.wasm"
GENERATED_RS="src/math_library_wasm.rs"

if [[ "${1:-}" == "--clean" ]]; then
    rm -f "$WASM_FILE" "$GENERATED_RS"
    cargo clean 2>/dev/null || true
    echo "Cleaned generated artifacts."
    exit 0
fi

# Step 1: Assemble WAT to Wasm
echo "==> Assembling $WAT_FILE to WebAssembly..."
if command -v wat2wasm &>/dev/null; then
    wat2wasm "$WAT_FILE" -o "$WASM_FILE"
else
    echo "Error: wat2wasm not found. Install with: apt-get install wabt" >&2
    echo "Falling back to pre-compiled $WASM_FILE if present." >&2
    if [[ ! -f "$WASM_FILE" ]]; then
        exit 1
    fi
fi
echo "    Created $WASM_FILE ($(wc -c < "$WASM_FILE") bytes)"

# Step 2: Transpile WebAssembly to Rust using herkos
echo "==> Transpiling WebAssembly to Rust..."
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

if command -v herkos &>/dev/null; then
    herkos "$WASM_FILE" --output "$GENERATED_RS"
else
    cargo run --manifest-path "$REPO_ROOT/Cargo.toml" -p herkos -- \
        "$SCRIPT_DIR/$WASM_FILE" --output "$SCRIPT_DIR/$GENERATED_RS"
fi
echo "    Created $GENERATED_RS"

# Step 3: Build and run
echo "==> Building and running Rust project..."
echo ""
cargo run --release
