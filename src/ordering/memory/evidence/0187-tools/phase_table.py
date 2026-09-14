#!/usr/bin/env python3
"""Phase cost/value table from a probe log that was run with SSI_PROBE_PHASES=1.

A PHASES line is::

    PHASES <name> <total> <phase>=<secs>/<cum_ratio> ... final=<ratio>

`cum_ratio` is the running-best flops of the row divided by the frontier's
flops for that row (the probe's own ratio, <= 1 means better).  The per-phase
*value* is therefore the drop of `cum_ratio` across that phase: prev - cur.
Cost is the printed seconds for that phase.

Usage: phase_table.py <log>
"""
import re
import sys
from collections import defaultdict

PHASE_RE = re.compile(r"^([0-9a-z._]+)=([0-9.]+)/([0-9.]+)$")


def parse(path):
    rows = []
    counts = {}
    with open(path, errors="replace") as fh:
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if parts[0] == "PHASES" and len(parts) >= 3:
                name = parts[1]
                try:
                    total = float(parts[2])
                except ValueError:
                    continue
                seq = []
                final = None
                for p in parts[3:]:
                    m = PHASE_RE.match(p)
                    if m:
                        seq.append((m.group(1), float(m.group(2)), float(m.group(3))))
                    elif p.startswith("final="):
                        final = float(p.split("=", 1)[1])
                rows.append((name, total, seq, final))
            elif parts[0] == "COUNTS" and len(parts) >= 5:
                try:
                    counts[parts[1]] = (int(parts[2]), int(parts[3]))
                except ValueError:
                    pass
    return rows, counts


def bucket(n):
    return 0 if n < 1000 else (1 if n < 10000 else 2)


def main():
    path = sys.argv[1]
    rows, counts = parse(path)
    if not rows:
        print("no PHASES lines (run the probe with SSI_PROBE_PHASES=1)")
        return
    secs = defaultdict(float)
    gain = defaultdict(float)
    seen = defaultdict(int)
    gained = defaultdict(int)
    maxg = defaultdict(float)
    seen_rows = []
    seq_of = {}
    for name, total, seq, final in rows:
        n, nnz = counts.get(name, (0, 0))
        seen_rows.append((name, n, nnz, total, final))
        seq_of.setdefault(name, seq)
        prev = 1.0
        for ph, t, r in seq:
            g = prev - r
            secs[ph] += t
            gain[ph] += g
            seen[ph] += 1
            if g > 1e-9:
                gained[ph] += 1
                maxg[ph] = max(maxg[ph], g)
            prev = r
    tot_secs = sum(secs.values())
    print(f"{path}: {len(rows)} rows, sum(phase secs) = {tot_secs:.1f} s")
    print(f"{'phase':<18}{'secs':>9}{'%tot':>7}{'gain':>10}{'rows':>7}{'maxg':>9}{'gain/s':>10}")
    for ph in sorted(secs, key=lambda p: -secs[p]):
        print(f"{ph:<18}{secs[ph]:>9.2f}{100*secs[ph]/tot_secs:>6.1f}%"
              f"{gain[ph]:>10.4f}{gained[ph]:>7}{maxg[ph]:>9.4f}"
              f"{gain[ph]/max(secs[ph],1e-9):>10.4f}")
    print("---- by bucket ----")
    for b in range(3):
        bs = defaultdict(float)
        bg = defaultdict(float)
        nb = 0
        for name, n, nnz, total, final in seen_rows:
            if bucket(n) != b:
                continue
            nb += 1
            prev = 1.0
            for ph, t, r in seq_of[name]:
                bs[ph] += t
                bg[ph] += prev - r
                prev = r
        top = sorted(bs, key=lambda p: -bs[p])[:8]
        print(f"bucket {b} ({nb} rows): " + "  ".join(
            f"{p}={bs[p]:.1f}s/{bg[p]:.4f}" for p in top))


if __name__ == "__main__":
    main()
