#!/usr/bin/env python3
"""Graded-frame phase census + per-phase mark-overhead attribution (0190).

Inputs: the markON / markOFF probe logs from `SSI_MARK_NOSCORE` A/B.
- `census`  : per-phase seconds summed over the corpus, in the graded frame.
- `chain`   : for the wide band 10 000 < n <= 50 000 (the rows the 0187 gate
              removed the chain from) the chain's own phase seconds in each
              frame, i.e. how much of the "0.11-0.20 s/row" price was the
              probe's mark.
- `bands`   : the per-row mark overhead by n band.
"""
import re
import sys
from collections import defaultdict

PHASE_RE = re.compile(r"^([0-9a-z._]+)=([0-9.]+)/([0-9.]+)$")
BANDS = [(0, 1000), (1000, 10000), (10000, 50000), (50000, 100000), (100000, 10**9)]


def load(path):
    rows, counts = {}, {}
    for line in open(path, errors="replace"):
        if line.startswith("PHASES\t"):
            f = line.rstrip("\n").split("\t")
            ph = {}
            for kv in f[3:]:
                m = PHASE_RE.match(kv)
                if m:
                    ph[m.group(1)] = (float(m.group(2)), float(m.group(3)))
            rows[f[1]] = (float(f[2]), ph)
        elif line.startswith("COUNTS\t"):
            f = line.rstrip("\n").split("\t")
            counts[f[1]] = (int(f[2]), int(f[3]))
    return rows, counts


a, ca = load(sys.argv[1])
b, cb = load(sys.argv[2])
rows = [k for k in a if k in b]
PHASES = [p for p in b[rows[0]][1] if not p.startswith("13p.")]

print("--- graded-frame phase census (markoff), corpus totals ---")
tot = {p: sum(b[k][1].get(p, (0, 0))[0] for k in rows) for p in PHASES}
probe = {p: sum(a[k][1].get(p, (0, 0))[0] for k in rows) for p in PHASES}
print(f"{'phase':<18} {'graded s':>10} {'probe s':>10} {'mark s':>9} {'mark %':>8}")
for p, s in sorted(tot.items(), key=lambda x: -x[1])[:14]:
    pr = probe[p]
    mk = pr - s
    print(f"{p:<18} {s:>10.2f} {pr:>10.2f} {mk:>9.2f} {100 * mk / pr if pr else 0:>8.1f}")
print(f"{'TOTAL':<18} {sum(tot.values()):>10.2f} {sum(probe.values()):>10.2f}")

print("\n--- wide band 10 000 < n <= 50 000: the chain's own cost ---")
wg = [k for k in rows if 10000 < ca[k][0] <= 50000]
print(f"{len(wg)} rows; chain (13.alt) probe vs graded, per row:")
print(f"{'row':<28} {'n':>8} {'13.alt probe':>13} {'13.alt graded':>14} {'row mark':>9}")
for k in sorted(wg, key=lambda k: -a[k][1].get("13.alt", (0, 0))[0]):
    pa = a[k][1].get("13.alt", (0, 0))[0]
    pb = b[k][1].get("13.alt", (0, 0))[0]
    print(f"{k:<28} {ca[k][0]:>8} {pa:>13.4f} {pb:>14.4f} {a[k][0] - b[k][0]:>+9.4f}")
sa = sum(a[k][1].get("13.alt", (0, 0))[0] for k in wg)
sb = sum(b[k][1].get("13.alt", (0, 0))[0] for k in wg)
print(f"band 13.alt total: probe {sa:.3f} s -> graded {sb:.3f} s  "
      f"({sa / len(wg):.4f} -> {sb / len(wg):.4f} s/row)")

print("\n--- per-row mark overhead by n band ---")
for lo, hi in BANDS:
    sel = [k for k in rows if lo <= ca[k][0] < hi]
    if not sel:
        continue
    d = sorted(a[k][0] - b[k][0] for k in sel)
    print(f"n in [{lo},{hi}): rows={len(sel):>3}  median {d[len(d)//2]:+.4f}  "
          f"max {d[-1]:+.4f}  sum {sum(d):+.2f}")
