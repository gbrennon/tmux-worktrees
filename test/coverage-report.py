#!/usr/bin/env python3
"""Print terminal coverage report from kcov JSON output."""

import json
import glob
import sys
import os


def load_coverage(kcov_dir):
    cov_files = glob.glob(os.path.join(kcov_dir, "*/coverage.json"))
    if not cov_files:
        # Maybe kcov_dir is directly the run dir
        direct = os.path.join(kcov_dir, "coverage.json")
        if os.path.exists(direct):
            cov_files = [direct]
    if not cov_files:
        print(f"  No coverage data found in {kcov_dir}", file=sys.stderr)
        sys.exit(1)
    return json.load(open(cov_files[0]))


def main():
    kcov_dirs = sys.argv[1:] if len(sys.argv) > 1 else ["/tmp/kcov-output"]

    scripts_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "scripts")

    # Merge coverage from multiple kcov output directories
    # Take the max covered_lines across runs (not sum, to avoid double-counting)
    kcov_by_path = {}
    for kcov_dir in kcov_dirs:
        data = load_coverage(kcov_dir)
        for f in data.get("files", []):
            name = f.get("file", "")
            if name in kcov_by_path:
                existing = kcov_by_path[name]
                existing["covered_lines"] = max(int(existing.get("covered_lines", 0)), int(f.get("covered_lines", 0)))
                existing["total_lines"] = max(int(existing.get("total_lines", 0)), int(f.get("total_lines", 0)))
                existing["percent_covered"] = (
                    existing["covered_lines"] / existing["total_lines"] * 100
                ) if existing["total_lines"] else 0
            else:
                kcov_by_path[name] = f

    total_covered = 0
    total_lines = 0
    rows = []

    for root, dirs, files in os.walk(scripts_dir):
        for fn in sorted(files):
            if not fn.endswith(".sh"):
                continue
            full = os.path.join(root, fn)
            rel = os.path.relpath(full, os.path.dirname(os.path.dirname(full)))
            kf = kcov_by_path.get(full)
            if kf is not None:
                covered = int(kf.get("covered_lines", 0))
                total = int(kf.get("total_lines", 0))
                pct = float(kf.get("percent_covered", 0))
            else:
                covered = 0
                total = 0
                pct = 0.0
            total_covered += covered
            total_lines += total
            rows.append((rel, covered, total, pct))

    if not rows:
        print("  No .sh files found under scripts/")
        return

    max_label = max(len(r[0]) for r in rows)

    print()
    print(f"  {'File':<{max_label}}  {'Covered':>7}  {'Total':>5}  {'Pct':>5}")
    print(f"  {'-' * max_label}  {'-------':>7}  {'-----':>5}  {'-----':>5}")
    for label, covered, total, pct in sorted(rows):
        bar_len = min(int(pct / 10), 10)
        bar = "#" * bar_len + " " * (10 - bar_len)
        print(f"  {label:<{max_label}}  {covered:>7}  {total:>5}  {pct:>5.1f}%  |{bar}|")

    print(f"  {'-' * max_label}  {'-------':>7}  {'-----':>5}  {'-----':>5}")
    overall_pct = (total_covered / total_lines * 100) if total_lines else 0
    bar_len = min(int(overall_pct / 10), 10)
    bar = "#" * bar_len + " " * (10 - bar_len)
    print(f"  {'TOTAL':<{max_label}}  {total_covered:>7}  {total_lines:>5}  {overall_pct:>5.1f}%  |{bar}|")
    print()


if __name__ == "__main__":
    main()
