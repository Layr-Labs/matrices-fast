#!/usr/bin/env python3
"""Per-row phase breakdown of a PHASES probe log.

Usage: phase_rows.py <log> [min_n] [--top K]
Prints n-range aggregates and the K slowest rows with their phase seconds
(only phases > 0.005 s), marked '+' when the phase improved that row's ratio.
"""
import re
import sys
from collections import defaultdict

PHASE_RE = re.compile(r"^([0-9a-z._]+)=([0-9.]+)/([0-9.]+)$")
RANGES = [(0, 1000, "n<1k"), (1000, 10000, "1k-10k"), (10000, 20000, "10k-20k"),
          (20000, 100000, "20k-100k"), (100000, 10**9, "n>=100k")]


def load(path):
    rows, counts, order = {}, {}, []
    with open(path, errors="replace") as fh:
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if parts[0] == "PHASES" and len(parts) >= 3:
                seq = []
                for p in parts[3:]:
                    m = PHASE_RE.match(p)
                    if m:
                        seq.append((m.group(1), float(m.group(2)), float(m.group(3))))
                if parts[1] not in rows:
                    order.append(parts[1])
                rows[parts[1]] = (float(parts[2]), seq)
            elif parts[0] == "COUNTS" and len(parts) >= 5:
                counts[parts[1]] = (int(parts[2]), int(parts[3]))
    return rows, counts, order


def main():
    path = sys.argv[1]
    topk = 12
    if "--top" in sys.argv:
        topk = int(sys.argv[sys.argv.index("--top") + 1])
    rows, counts, order = load(path)
    print(f"{len(rows)} rows with PHASES")
    print(f"{'range':<10}{'rows':>5}{'secs':>8}{'gain':>9}   top phases (secs/gain)")
    for lo, hi, label in RANGES:
        secs, gain, nb = defaultdict(float), defaultdict(float), 0
        for name in order:
            n, nnz = counts.get(name, (0, 0))
            if not (lo <= n < hi):
                continue
            nb += 1
            prev = 1.0
            for ph, t, r in rows[name][1]:
                secs[ph] += t
                gain[ph] += prev - r
                prev = r
        tot = sum(secs.values())
        gg = sum(gain.values())
        top = sorted(secs, key=lambda p: -secs[p])[:6]
        print(f"{label:<10}{nb:>5}{tot:>8.1f}{gg:>9.4f}   " +
              "  ".join(f"{p}={secs[p]:.2f}/{gain[p]:.4f}" for p in top))
    print(f"\n---- {topk} slowest rows ----")
    ranked = sorted((rows[n][0], n) for n in order)[::-1][:topk]
    for t, name in ranked:
        n, nnz = counts.get(name, (0, 0))
        seq = rows[name][1]
        prev = 1.0
        bits = []
        for ph, ts, r in seq:
            mark = "+" if prev - r > 1e-9 else " "
            if ts >= 0.005:
                bits.append(f"{ph}={ts:.3f}{mark}")
            prev = r
        print(f"{name:<28} n={n:<7} nnz={nnz:<9} {t:.3f}s  " + " ".join(bits))


if __name__ == "__main__":
    main()
