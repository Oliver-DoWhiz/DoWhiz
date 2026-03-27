#!/usr/bin/env python3
"""
Analyze task timing data from JSONL file.
Generates bar plots and summary statistics table.

Usage:
    python analyze_timings.py [path_to_jsonl]

Default path: ./task_timings.jsonl
"""

import json
import sys
from pathlib import Path
from collections import defaultdict

try:
    import matplotlib.pyplot as plt
    import numpy as np
except ImportError:
    print("Please install: pip install matplotlib numpy")
    sys.exit(1)

STAGES = [
    ("queue_latency_ms", "Queue Latency"),
    ("setup_latency_ms", "Setup"),
    ("ephemeral_share_create_ms", "Ephemeral Create"),
    ("ephemeral_share_upload_ms", "Ephemeral Upload"),
    ("aci_cold_start_ms", "ACI Cold Start"),
    ("codex_execution_ms", "Codex Execution"),
    ("result_download_ms", "Result Download"),
]


def load_timings(path: Path) -> list[dict]:
    """Load JSONL timing records."""
    records = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line:
                records.append(json.loads(line))
    return records


def compute_stats(values: list[float]) -> dict:
    """Compute summary statistics."""
    if not values:
        return {"count": 0, "mean": 0, "median": 0, "p95": 0, "std": 0}
    arr = np.array(values)
    return {
        "count": len(arr),
        "mean": np.mean(arr),
        "median": np.median(arr),
        "p95": np.percentile(arr, 95),
        "std": np.std(arr),
    }


def print_summary_table(records: list[dict]):
    """Print summary statistics table."""
    print("\n" + "=" * 80)
    print(f"TIMING SUMMARY ({len(records)} tasks)")
    print("=" * 80)
    print(f"{'Stage':<20} {'Count':>8} {'Mean':>10} {'Median':>10} {'P95':>10} {'StdDev':>10}")
    print("-" * 80)

    for key, name in STAGES:
        values = [r[key] for r in records if r.get(key) is not None]
        stats = compute_stats(values)
        if stats["count"] > 0:
            print(f"{name:<20} {stats['count']:>8} {stats['mean']:>10.1f} {stats['median']:>10.1f} {stats['p95']:>10.1f} {stats['std']:>10.1f}")

    # Total
    values = [r["total_ms"] for r in records if r.get("total_ms") is not None]
    stats = compute_stats(values)
    print("-" * 80)
    print(f"{'TOTAL':<20} {stats['count']:>8} {stats['mean']:>10.1f} {stats['median']:>10.1f} {stats['p95']:>10.1f} {stats['std']:>10.1f}")
    print("=" * 80)
    print("(All times in milliseconds)")


def plot_mean_breakdown(records: list[dict], output_path: Path):
    """Bar plot of mean time per stage."""
    means = []
    labels = []

    for key, name in STAGES:
        values = [r[key] for r in records if r.get(key) is not None]
        if values:
            means.append(np.mean(values))
            labels.append(name)

    if not means:
        print("No data to plot")
        return

    fig, ax = plt.subplots(figsize=(10, 6))
    bars = ax.barh(labels, means, color='steelblue')
    ax.set_xlabel('Time (ms)')
    ax.set_title(f'Mean Latency by Stage (n={len(records)} tasks)')

    # Add value labels
    for bar, val in zip(bars, means):
        ax.text(val + max(means) * 0.01, bar.get_y() + bar.get_height()/2,
                f'{val:.0f}ms', va='center', fontsize=9)

    plt.tight_layout()
    plt.savefig(output_path / "mean_breakdown.png", dpi=150)
    print(f"Saved: {output_path / 'mean_breakdown.png'}")


def plot_stacked_timeline(records: list[dict], output_path: Path, last_n: int = 20):
    """Stacked bar showing breakdown per task."""
    recent = records[-last_n:] if len(records) > last_n else records

    fig, ax = plt.subplots(figsize=(12, 6))

    x = np.arange(len(recent))
    bottom = np.zeros(len(recent))
    colors = plt.cm.tab10(np.linspace(0, 1, len(STAGES)))

    for (key, name), color in zip(STAGES, colors):
        values = [r.get(key, 0) or 0 for r in recent]
        ax.bar(x, values, bottom=bottom, label=name, color=color, width=0.8)
        bottom += np.array(values)

    ax.set_xlabel('Task')
    ax.set_ylabel('Time (ms)')
    ax.set_title(f'Per-Task Timing Breakdown (last {len(recent)} tasks)')
    ax.legend(loc='upper left', bbox_to_anchor=(1, 1))
    ax.set_xticks(x)
    ax.set_xticklabels([r.get('task_id', '')[:12] for r in recent], rotation=45, ha='right', fontsize=7)

    plt.tight_layout()
    plt.savefig(output_path / "stacked_timeline.png", dpi=150)
    print(f"Saved: {output_path / 'stacked_timeline.png'}")


def plot_distribution(records: list[dict], output_path: Path):
    """Box plot showing distribution per stage."""
    data = []
    labels = []

    for key, name in STAGES:
        values = [r[key] for r in records if r.get(key) is not None]
        if values:
            data.append(values)
            labels.append(name)

    if not data:
        return

    fig, ax = plt.subplots(figsize=(10, 6))
    ax.boxplot(data, labels=labels, vert=False)
    ax.set_xlabel('Time (ms)')
    ax.set_title(f'Latency Distribution by Stage (n={len(records)} tasks)')

    plt.tight_layout()
    plt.savefig(output_path / "distribution.png", dpi=150)
    print(f"Saved: {output_path / 'distribution.png'}")


def main():
    # Get input path
    if len(sys.argv) > 1:
        input_path = Path(sys.argv[1])
    else:
        input_path = Path("./task_timings.jsonl")

    if not input_path.exists():
        print(f"File not found: {input_path}")
        sys.exit(1)

    # Load data
    records = load_timings(input_path)
    if not records:
        print("No timing records found")
        sys.exit(1)

    print(f"Loaded {len(records)} timing records from {input_path}")

    # Print summary
    print_summary_table(records)

    # Generate plots
    output_path = input_path.parent
    plot_mean_breakdown(records, output_path)
    plot_stacked_timeline(records, output_path)
    plot_distribution(records, output_path)

    print(f"\nPlots saved to: {output_path}")


if __name__ == "__main__":
    main()
