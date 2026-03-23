#!/usr/bin/env bash
set -euo pipefail

COMMENT="${1:-}"

# Ensure we're in the repo root
if [ ! -f Cargo.toml ]; then
    echo "ERROR: Cargo.toml not found. Please run this script from the repo root."
    exit 1
fi

echo "=== herkos benchmark recorder ==="
echo "Branch: $(git rev-parse --abbrev-ref HEAD)"
echo "Commit: $(git rev-parse --short HEAD)"
[ -n "$COMMENT" ] && echo "Comment: $COMMENT"
echo

# Run benchmarks
echo "Running benchmarks (HERKOS_OPTIMIZE=1)..."
HERKOS_OPTIMIZE=1 cargo bench -p herkos-tests

echo
echo "Collecting results..."

# Capture metadata
export BRANCH=$(git rev-parse --abbrev-ref HEAD)
export COMMIT_SHA=$(git rev-parse HEAD)
export TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Parse Criterion output to CSV
python3 scripts/collect_bench.py target/criterion /tmp/new_bench_rows.csv ${COMMENT:+--comment "$COMMENT"}

echo
echo "Pushing to metrics branch..."

# Configure git if not already done
git config user.email "bench@local" || true
git config user.name "bench" || true

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Fetch or create metrics branch
if git fetch origin metrics 2>/dev/null; then
    git checkout metrics
else
    echo "Creating orphan metrics branch..."
    git checkout --orphan metrics
    git rm -rf . --quiet 2>/dev/null || true
    printf '# metrics branch\n\nManual benchmark history for herkos.\n\nSee bench_history.csv for data.\n' > README.md
    git add README.md
    git commit -m "chore: init metrics branch"
fi

# Append rows to bench_history.csv
if [ -f bench_history.csv ]; then
    # Skip header row when appending
    tail -n +2 /tmp/new_bench_rows.csv >> bench_history.csv
else
    # First time: copy entire file with header
    cp /tmp/new_bench_rows.csv bench_history.csv
fi

git add bench_history.csv
if ! git diff --cached --quiet; then
    git commit -m "bench: results for ${COMMIT_SHA:0:12}"
    git push origin metrics
else
    echo "No changes to bench_history.csv (likely zero benchmarks found)"
fi

# Return to original branch
git checkout "$CURRENT_BRANCH"

echo "✓ Done. Pushed to metrics branch."
