#!/usr/bin/env python3
"""
collect_bench.py <criterion_dir> <output_csv> [--comment "text"]

Walks Criterion JSON output, extracts timing statistics, and appends
enriched rows to a CSV file. Creates the file with header on first run.

Environment variables consumed:
  BRANCH        - git branch name
  COMMIT_SHA    - full commit SHA
  TIMESTAMP     - ISO-8601 timestamp
"""

import sys
import os
import json
import csv
import pathlib
import argparse

FIELDS = [
    "timestamp", "branch", "commit_sha",
    "benchmark_name",
    "mean_ns", "stddev_ns", "median_ns", "lower_ns", "upper_ns",
    "comments",
]


def parse_estimates(path: pathlib.Path) -> dict:
    """Extract timing statistics from a Criterion estimates.json file."""
    with path.open() as f:
        data = json.load(f)
    return {
        "mean_ns":   data["mean"]["point_estimate"],
        "stddev_ns": data["std_dev"]["point_estimate"],
        "median_ns": data["median"]["point_estimate"],
        "lower_ns":  data["mean"]["confidence_interval"]["lower_bound"],
        "upper_ns":  data["mean"]["confidence_interval"]["upper_bound"],
    }


def collect_rows(criterion_dir: pathlib.Path, metadata: dict) -> list[dict]:
    """Walk criterion_dir and build one row per benchmark."""
    rows = []
    for estimates_path in sorted(criterion_dir.rglob("new/estimates.json")):
        # Path: <criterion_dir>/<group>/<bench_name>/new/estimates.json
        bench_name = estimates_path.parent.parent.name
        try:
            stats = parse_estimates(estimates_path)
        except (KeyError, json.JSONDecodeError) as e:
            print(f"WARNING: skipping {estimates_path}: {e}", file=sys.stderr)
            continue
        rows.append({**metadata, "benchmark_name": bench_name, **stats})
    return rows


def write_csv(output_path: pathlib.Path, rows: list[dict]) -> None:
    """Append rows to CSV, writing header only if file is new."""
    file_exists = output_path.exists() and output_path.stat().st_size > 0
    with output_path.open("a", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=FIELDS)
        if not file_exists:
            writer.writeheader()
        writer.writerows(rows)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Parse Criterion benchmark results into CSV rows"
    )
    parser.add_argument("criterion_dir", help="Root Criterion output directory")
    parser.add_argument("output_csv", help="Output CSV file path")
    parser.add_argument(
        "--comment",
        default="",
        help="Optional comment to append to each row"
    )

    args = parser.parse_args()

    criterion_dir = pathlib.Path(args.criterion_dir)
    output_csv = pathlib.Path(args.output_csv)

    if not criterion_dir.is_dir():
        print(f"ERROR: criterion directory not found: {criterion_dir}", file=sys.stderr)
        return 1

    metadata = {
        "timestamp":   os.environ.get("TIMESTAMP", ""),
        "branch":      os.environ.get("BRANCH", ""),
        "commit_sha":  os.environ.get("COMMIT_SHA", ""),
        "comments":    args.comment,
    }

    rows = collect_rows(criterion_dir, metadata)

    if not rows:
        print("WARNING: no benchmark results found in criterion directory", file=sys.stderr)
        return 0

    write_csv(output_csv, rows)
    print(f"Wrote {len(rows)} rows to {output_csv}")
    for r in rows:
        print(f"  {r['benchmark_name']}: mean={r['mean_ns']:.1f}ns")
    return 0


if __name__ == "__main__":
    sys.exit(main())
