#!/usr/bin/env python3
"""Calibrate the probe frame against the graded frame (0190).

The probe's 25 `phase_mark` sites each evaluate `score(&best_perm)` INSIDE the
interval they report, so every printed phase time is (phase work + one scoring
pass) and every row total is inflated by 25 passes. `phase_mark` is
`#[cfg(test)]`, so none of that exists in the shipped worker.

`SSI_MARK_NOSCORE=1` replaces that argument with `best_flops` -- the same value
the pipeline already maintains, so the printed ratios must be IDENTICAL and the
time difference is exactly the mark overhead.

Usage: mark_cal.py <markon.log> <markoff.log> [--top K]
"""
import re
import sys
from collections import defaultdict

PHASE_RE = re.compile(r"^([0-9a-z._]+)=([0-9.]+)/([0-9.]+)$")


def load(path):
    rows, counts = {}, {}
    for line in open(path, errors="replace"):
        if line.startswith("PHASES\t"):
            f = line.rstrip("\n").split("\t")
            phases = {}
            for kv in f[3:]:
                m = PHASE_RE.match(kv)
                if m:
                    phases[m.group(1)] = (float(m.group(2)), float(m.group(3)))
            rows[f[1]] = (float(f[2]), phases)
        elif line.startswith("COUNTS\t"):
            f = line.rstrip("\n").split("\t")
            counts[f[1]] = tuple(f[2:])
    return rows, counts


def main():
    on, off = sys.argv[1], sys.argv[2]
    top = 12
    if "--top" in sys.argv:
        top = int(sys.argv[sys.argv.index("--top") + 1])
    a, ca = load(on)
    b, cb = load(off)
    common = [k for k in a if k in b]
    print(f"rows markon={len(a)} markoff={len(b)} common={len(common)}")

    # Invariant 1: the same perm/permutation decisions in both frames.
    mism = [k for k in common if a[k][1].get("final") != b[k][1].get("final")]
    print(f"final-ratio mismatches: {len(mism)}")
    cmism = [k for k in common if ca.get(k) != cb.get(k)]
    print(f"COUNTS mismatches: {len(cmism)}")

    deltas = sorted(((a[k][0] - b[k][0], k) for k in common), reverse=True)
    tot_on = sum(a[k][0] for k in common)
    tot_off = sum(b[k][0] for k in common)
    print(f"corpus order(): markon {tot_on:.1f} s  markoff {tot_off:.1f} s "
          f"({100 * (tot_on - tot_off) / tot_off:+.1f} %)")

    print(f"\n---- {top} rows by mark overhead ----")
    print(f"{'row':<28} {'markON':>8} {'markOFF':>8} {'overhead':>9} {'phases>0.002':>12}")
    for d, k in deltas[:top]:
        late = sum(1 for ph, (s, _) in a[k][1].items()
                   if s > 0.002 and b[k][1].get(ph, (0, 0))[0] <= 0.002)
        print(f"{k:<28} {a[k][0]:>8.3f} {b[k][0]:>8.3f} {d:>+9.4f} {late:>12}")

    print(f"\n---- {top} rows by GRADED (markoff) time ----")
    slow = sorted(common, key=lambda k: -b[k][0])[:top]
    for k in slow:
        ph = {p: v[0] for p, v in b[k][1].items() if v[0] > 0.004}
        gain = {p: v[1] for p, v in b[k][1].items()}
        score_on = a[k][0]
        print(f"{k:<28} graded {b[k][0]:.3f} s (probe {score_on:.3f})  biggest: "
              + "  ".join(f"{p}={s:.3f}" for p, s in
                          sorted(ph.items(), key=lambda x: -x[1])[:6]))
        # mark gain per phase: did the cum ratio drop during the phase?
        order = list(gain)
        drops = [(order[i], gain[order[i - 1]] - gain[order[i]])
                 for i in range(1, len(order)) if gain[order[i - 1]] - gain[order[i]] > 1e-9]
        if drops:
            print("      graded yield: " + "  ".join(f"{p}:{v:.4f}" for p, v in drops))


main()
