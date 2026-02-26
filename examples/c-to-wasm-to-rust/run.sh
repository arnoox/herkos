#!/usr/bin/env bash
#
# C → WebAssembly → Rust example pipeline
#
# Prerequisites:
#   - clang with wasm32 target support (apt-get install clang lld)
#   - Rust toolchain (cargo)
#   - herkos CLI (cargo install --path ../../crates/herkos)
#
# Usage:
#   ./run.sh          # build and run
#   ./run.sh --clean  # remove generated artifacts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

WASM_FILE="fibonacci.wasm"
GENERATED_RS="src/fibonacci_wasm.rs"

if [[ "${1:-}" == "--clean" ]]; then
    rm -f "$WASM_FILE" "$GENERATED_RS"
    cargo clean 2>/dev/null || true
    echo "Cleaned generated artifacts."
    exit 0
fi

# Step 1: Compile C to WebAssembly
echo "==> Compiling fibonacci.c to WebAssembly..."

CLANG=""
if command -v clang-19 &>/dev/null; then
    CLANG="clang-19"
elif command -v clang &>/dev/null; then
    CLANG="clang"
else
    echo "Error: clang not found. Install with: apt-get install clang lld" >&2
    exit 1
fi

$CLANG --target=wasm32-unknown-unknown -nostdlib -Oz \
    -Wl,--no-entry \
    -Wl,--export-all \
    -Wl,-zstack-size=65536 \
    -Wl,--initial-memory=131072 \
    -Wl,--max-memory=131072 \
    fibonacci.c -o "$WASM_FILE"

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

# Step 3: Build and run the Rust project
echo "==> Building and running Rust project..."
echo ""
cargo run --release
