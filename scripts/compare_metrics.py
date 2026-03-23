#!/usr/bin/env python3
"""
compare_metrics.py [--format {table|sparkline|html}]

Fetches bench_history.csv and size_history.csv from the metrics branch,
groups by benchmark/metric name, and displays evolution over time.

Output formats:
  table      - Simple ASCII table (default)
  sparkline  - Compact sparkline visualization
  html       - Interactive HTML report
"""

import sys
import os
import csv
import json
import subprocess
import tempfile
import argparse
from pathlib import Path
from collections import defaultdict
from datetime import datetime
from typing import List, Dict, Tuple


def fetch_csv_from_branch(filename: str, branch: str = "metrics") -> List[Dict]:
    """Fetch and parse CSV from metrics branch."""
    try:
        result = subprocess.run(
            ["git", "show", f"{branch}:{filename}"],
            capture_output=True,
            text=True,
            check=True,
        )
        rows = []
        reader = csv.DictReader(result.stdout.splitlines())
        for row in reader:
            rows.append(row)
        return rows
    except subprocess.CalledProcessError as e:
        print(f"Warning: Could not fetch {filename} from {branch}: {e.stderr}", file=sys.stderr)
        return []


def parse_timestamp(ts: str) -> datetime:
    """Parse ISO timestamp."""
    try:
        return datetime.fromisoformat(ts.replace("Z", "+00:00"))
    except (ValueError, AttributeError):
        return datetime.min


def group_metrics(bench_rows: List[Dict], size_rows: List[Dict]) -> Dict[str, List[Dict]]:
    """Group metrics by name (benchmark_name or metric_name) with timestamps."""
    groups = defaultdict(list)

    # Group benchmarks
    for row in bench_rows:
        name = row.get("benchmark_name", "").strip()
        if name:
            groups[f"bench/{name}"].append({
                "type": "benchmark",
                "timestamp": row.get("timestamp", ""),
                "commit": row.get("commit_sha", "")[:12],
                "value": float(row.get("mean_ns", 0)),
                "unit": "ns",
                "stddev": float(row.get("stddev_ns", 0)),
            })

    # Group file sizes
    for row in size_rows:
        name = row.get("metric_name", "").strip()
        if name:
            groups[f"size/{name}"].append({
                "type": "size",
                "timestamp": row.get("timestamp", ""),
                "commit": row.get("commit_sha", "")[:12],
                "value": float(row.get("value", 0)),
                "unit": "bytes" if "bytes" in name else "lines",
            })

    # Sort each group by timestamp
    for name in groups:
        groups[name].sort(key=lambda x: parse_timestamp(x["timestamp"]))

    return groups


def format_value(value: float, unit: str) -> str:
    """Format value with appropriate precision."""
    if unit == "ns":
        if value > 1e9:
            return f"{value/1e9:.2f}s"
        elif value > 1e6:
            return f"{value/1e6:.2f}ms"
        elif value > 1e3:
            return f"{value/1e3:.2f}µs"
        else:
            return f"{value:.0f}ns"
    elif unit == "bytes":
        if value > 1e6:
            return f"{value/1e6:.2f}MB"
        elif value > 1e3:
            return f"{value/1e3:.2f}KB"
        else:
            return f"{value:.0f}B"
    else:
        return f"{value:.0f}"


def sparkline(values: List[float], width: int = 10) -> str:
    """Generate a simple sparkline from values."""
    if not values or len(values) < 2:
        return "─" * width

    min_v = min(values)
    max_v = max(values)
    range_v = max_v - min_v

    if range_v == 0:
        return "─" * width

    chars = "▁▂▃▄▅▆▇█"
    sparkline_str = ""
    step = len(values) / width
    for i in range(width):
        idx = min(int(i * step), len(values) - 1)
        val = values[idx]
        normalized = (val - min_v) / range_v
        char_idx = min(int(normalized * (len(chars) - 1)), len(chars) - 1)
        sparkline_str += chars[char_idx]

    return sparkline_str


def display_table(groups: Dict[str, List[Dict]]) -> None:
    """Display metrics as ASCII table."""
    print("\n" + "=" * 120)
    print(f"{'Metric':<40} {'Latest':<20} {'Prev':<20} {'Change':<15} {'Trend':<20}")
    print("=" * 120)

    for name in sorted(groups.keys()):
        entries = groups[name]
        if not entries:
            continue

        latest = entries[-1]
        prev = entries[-2] if len(entries) > 1 else None

        latest_str = format_value(latest["value"], latest["unit"])

        if prev:
            change = latest["value"] - prev["value"]
            pct_change = (change / prev["value"] * 100) if prev["value"] != 0 else 0
            change_str = f"{change:+.1f} ({pct_change:+.1f}%)"
            direction = "↑" if change > 0 else "↓" if change < 0 else "="
            trend_str = sparkline([e["value"] for e in entries[-10:]])
        else:
            change_str = "—"
            direction = "—"
            trend_str = "N/A"

        print(f"{name:<40} {latest_str:<20} {change_str:<20} {direction:<15} {trend_str:<20}")

    print("=" * 120)


def display_sparkline(groups: Dict[str, List[Dict]]) -> None:
    """Display metrics with sparklines for compact view."""
    print("\n" + "─" * 100)
    print(f"{'Metric':<45} {'Sparkline':<40} {'Latest':<15}")
    print("─" * 100)

    for name in sorted(groups.keys()):
        entries = groups[name]
        if not entries:
            continue

        values = [e["value"] for e in entries]
        latest_str = format_value(entries[-1]["value"], entries[-1]["unit"])
        spark = sparkline(values, width=35)

        print(f"{name:<45} {spark:<40} {latest_str:<15}")

    print("─" * 100)


def display_html(groups: Dict[str, List[Dict]], output_path: str = "metrics_report.html") -> None:
    """Generate an interactive HTML report (requires plotly)."""
    try:
        import plotly.graph_objects as go
        from plotly.subplots import make_subplots
    except ImportError:
        print("Error: plotly required for HTML output. Install with: pip install plotly", file=sys.stderr)
        return

    num_metrics = len(groups)
    fig = make_subplots(
        rows=(num_metrics + 1) // 2,
        cols=2,
        subplot_titles=sorted(groups.keys()),
        specs=[[{"secondary_y": False}] * 2] * ((num_metrics + 1) // 2),
    )

    for idx, name in enumerate(sorted(groups.keys())):
        entries = groups[name]
        if not entries:
            continue

        row = (idx // 2) + 1
        col = (idx % 2) + 1

        timestamps = [parse_timestamp(e["timestamp"]).isoformat() for e in entries]
        values = [e["value"] for e in entries]

        fig.add_trace(
            go.Scatter(
                x=timestamps,
                y=values,
                mode="lines+markers",
                name=name,
                hovertemplate=f"<b>{name}</b><br>Timestamp: %{{x}}<br>Value: %{{y:.2f}}<extra></extra>",
            ),
            row=row,
            col=col,
        )

    fig.update_layout(
        title_text="herkos Metrics Evolution",
        height=300 * ((num_metrics + 1) // 2),
        showlegend=False,
    )

    fig.write_html(output_path)
    print(f"✓ HTML report written to {output_path}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Display metric evolution from metrics branch"
    )
    parser.add_argument(
        "--format",
        choices=["table", "sparkline", "html"],
        default="table",
        help="Output format (default: table)",
    )
    parser.add_argument(
        "--output",
        default="metrics_report.html",
        help="Output file for HTML format (default: metrics_report.html)",
    )

    args = parser.parse_args()

    print("Fetching metrics from 'metrics' branch...")
    bench_rows = fetch_csv_from_branch("bench_history.csv")
    size_rows = fetch_csv_from_branch("size_history.csv")

    if not bench_rows and not size_rows:
        print("Error: Could not fetch any metrics. Ensure metrics branch exists.", file=sys.stderr)
        return 1

    print(f"  Loaded {len(bench_rows)} benchmark entries, {len(size_rows)} size entries")

    groups = group_metrics(bench_rows, size_rows)

    if not groups:
        print("No metrics found.", file=sys.stderr)
        return 1

    if args.format == "table":
        display_table(groups)
    elif args.format == "sparkline":
        display_sparkline(groups)
    elif args.format == "html":
        display_html(groups, args.output)

    return 0


if __name__ == "__main__":
    sys.exit(main())
