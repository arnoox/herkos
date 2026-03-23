#!/usr/bin/env python3
"""
collect_size.py <metrics_json> <output_csv> [--comment "text"]

Collects file size metrics from examples and appends enriched rows to a CSV file.
Creates the file with header on first run.

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
    "metric_name",
    "value",
    "comments",
]


def parse_metrics(path: pathlib.Path) -> list[dict]:
    """Load metrics from a JSON file."""
    with path.open() as f:
        data = json.load(f)
    return data


def collect_rows(metrics_file: pathlib.Path, metadata: dict) -> list[dict]:
    """Parse metrics JSON and build one row per metric."""
    rows = []
    try:
        metrics = parse_metrics(metrics_file)
    except (KeyError, json.JSONDecodeError) as e:
        print(f"ERROR: failed to parse {metrics_file}: {e}", file=sys.stderr)
        return rows

    for metric_name, value in metrics.items():
        rows.append({
            **metadata,
            "metric_name": metric_name,
            "value": value,
        })

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
        description="Collect file size metrics into CSV rows"
    )
    parser.add_argument("metrics_json", help="Metrics JSON file (e.g., /tmp/size_metrics.json)")
    parser.add_argument("output_csv", help="Output CSV file path")
    parser.add_argument(
        "--comment",
        default="",
        help="Optional comment to append to each row"
    )

    args = parser.parse_args()

    metrics_file = pathlib.Path(args.metrics_json)
    output_csv = pathlib.Path(args.output_csv)

    if not metrics_file.is_file():
        print(f"ERROR: metrics file not found: {metrics_file}", file=sys.stderr)
        return 1

    metadata = {
        "timestamp":   os.environ.get("TIMESTAMP", ""),
        "branch":      os.environ.get("BRANCH", ""),
        "commit_sha":  os.environ.get("COMMIT_SHA", ""),
        "comments":    args.comment,
    }

    rows = collect_rows(metrics_file, metadata)

    if not rows:
        print("WARNING: no metrics found", file=sys.stderr)
        return 0

    write_csv(output_csv, rows)
    print(f"Wrote {len(rows)} rows to {output_csv}")
    for r in rows:
        print(f"  {r['metric_name']}: {r['value']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
