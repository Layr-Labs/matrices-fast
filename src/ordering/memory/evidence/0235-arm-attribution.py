#!/usr/bin/env python3
"""Attribute the iter51 arm sweep (0230): which arm pays the +0.041 s worst-row
cost, and which arm buys the dev value? Ratios are deterministic (exact output),
so per-row ratio deltas are exact; secs are single-draw and are used only for
the worst-row question."""
import sys
import os

BASE = ("/home/frosty/angelX/.tmp/live-performance-20260911/matrices-deepseek/"
        "src/ordering/memory/evidence/")
ARMS = {
    "P": "0230-armP-4cpu.log",
    "L": "0230-armL-4cpu.log",
    "L2": "0230-armL2-4cpu.log",
    "X": "0230-armX-4cpu.log",
    "XL": "0230-armXL-4cpu.log",
    "XLP": "0230-armXLP-4cpu.log",
}


def parse(path):
    rows = {}
    worst = None
    score = None
    in_tsv = False
    with open(path, errors="replace") as fh:
        for line in fh:
            if line.startswith("SCORE = "):
                score = float(line.split("=")[1])
            if line.startswith("WORST order()"):
                worst = float(line.split("=")[1].split("s")[0])
            if line.startswith("secs\tmatrix"):
                in_tsv = True
                continue
            if in_tsv:
                parts = line.rstrip("\n").split("\t")
                if len(parts) != 5:
                    continue
                try:
                    secs = float(parts[0])
                    n = int(parts[2])
                    ratio = float(parts[4])
                except ValueError:
                    continue
                rows[parts[1]] = (secs, n, ratio)
    return rows, worst, score


def main():
    data = {}
    for k, f in ARMS.items():
        p = os.path.join(BASE, f)
        if os.path.exists(p):
            data[k] = parse(p)
    print("arm\tscore\tworst_row_s\tn_rows")
    for k, (rows, worst, score) in data.items():
        print(f"{k}\t{score}\t{worst}\t{len(rows)}")
    base = data.get("P")
    if not base:
        return
    base_rows = base[0]
    for k in ["L", "L2", "X", "XL", "XLP"]:
        if k not in data:
            continue
        rows = data[k][0]
        better = worse = same = 0
        worst_delta = (0.0, None)
        for name, (secs, n, ratio) in rows.items():
            if name not in base_rows:
                continue
            b = base_rows[name][2]
            if ratio < b:
                better += 1
            elif ratio > b:
                worse += 1
            else:
                same += 1
            d = secs - base_rows[name][0]
            if d > worst_delta[0]:
                worst_delta = (d, name)
        print(
            f"{k}\tbetter={better}\tworse={worse}\tsame={same}\t"
            f"max_row_secs_delta={worst_delta[0]:+.4f}@{worst_delta[1]}"
        )


if __name__ == "__main__":
    main()
