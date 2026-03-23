#!/usr/bin/env bash
set -euo pipefail

COMMENT="${1:-}"

# Ensure we're in the repo root
if [ ! -f Cargo.toml ]; then
    echo "ERROR: Cargo.toml not found. Please run this script from the repo root."
    exit 1
fi

echo "=== herkos file size recorder ==="
echo "Branch: $(git rev-parse --abbrev-ref HEAD)"
echo "Commit: $(git rev-parse --short HEAD)"
[ -n "$COMMENT" ] && echo "Comment: $COMMENT"
echo

# Run FFT example to generate fft_wasm.rs
echo "Running FFT example (C → Wasm → Rust)..."
cd examples/c-fft
./run.sh
cd ../..

echo
echo "Collecting metrics..."

# Extract metrics from generated files
GENERATED_RS="examples/c-fft/src/fft_wasm.rs"

if [ ! -f "$GENERATED_RS" ]; then
    echo "ERROR: Generated file not found: $GENERATED_RS"
    exit 1
fi

# Measure file size and line count
FILE_SIZE=$(wc -c < "$GENERATED_RS")
LINE_COUNT=$(wc -l < "$GENERATED_RS")

echo "  fft_wasm.rs: $FILE_SIZE bytes, $LINE_COUNT lines"

# Create JSON metrics file
cat > /tmp/size_metrics.json << EOF
{
    "fft_wasm_bytes": $FILE_SIZE,
    "fft_wasm_lines": $LINE_COUNT
}
EOF

# Capture metadata
export BRANCH=$(git rev-parse --abbrev-ref HEAD)
export COMMIT_SHA=$(git rev-parse HEAD)
export TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

echo
echo "Pushing to metrics branch..."

# Configure git if not already done
git config user.email "metrics@local" || true
git config user.name "metrics" || true

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Fetch or create metrics branch
if git fetch origin metrics 2>/dev/null; then
    git checkout metrics
else
    echo "Creating orphan metrics branch..."
    git checkout --orphan metrics
    git rm -rf . --quiet 2>/dev/null || true
    printf '# metrics branch\n\nManual benchmark and file size history for herkos.\n\nSee bench_history.csv and size_history.csv for data.\n' > README.md
    git add README.md
    git commit -m "chore: init metrics branch"
fi

# Parse metrics to CSV
python3 scripts/collect_size.py /tmp/size_metrics.json /tmp/new_size_rows.csv ${COMMENT:+--comment "$COMMENT"}

# Append rows to size_history.csv
if [ -f size_history.csv ]; then
    # Skip header row when appending
    tail -n +2 /tmp/new_size_rows.csv >> size_history.csv
else
    # First time: copy entire file with header
    cp /tmp/new_size_rows.csv size_history.csv
fi

git add size_history.csv
if ! git diff --cached --quiet; then
    git commit -m "metrics: file size for ${COMMIT_SHA:0:12}"
    git push origin metrics
else
    echo "No changes to size_history.csv"
fi

# Return to original branch
git checkout "$CURRENT_BRANCH"

echo "✓ Done. Pushed to metrics branch."
