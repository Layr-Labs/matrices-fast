#!/usr/bin/env python3
"""Audit the public submission ledger for improving-but-under-threshold rejections.

Input:  the plain-text table printed by `yukon submissions --all` (ANSI stripped).
Output: (1) the count table, (2) one line per improving rejection with its gap to
        the best-known-at-the-time, (3) the max-gap-rejected / min-gap-accepted
        bracket that calibrates `minScoreImprovementBips` RELATIVE.

Usage: yukon submissions --all > ledger.txt; python3 audit_below_threshold.py ledger.txt
"""
import re
import sys
import collections

ROW = re.compile(
    r"^([0-9a-f]{7})\s+(\S+)\s+(promoted|rejected|failed|pending|cancelled)\s+"
    r"(\S+)\s+(\{.*?\}|n/a)\s+(\S+)\s+(\S+)\s+(.+?)\s*$"
)


def parse(path):
    rows = []
    for line in open(path, encoding="utf-8", errors="replace"):
        line = re.sub(r"\x1b\[[0-9;]*m", "", line)
        m = ROW.match(line)
        if m:
            rows.append(dict(
                sub=m.group(1), solver=m.group(2), status=m.group(3), score=m.group(4),
                delta=m.group(6), commit=m.group(7), created=m.group(8),
            ))
    return rows


def signed(text):
    m = re.match(r"([-+])([0-9.]+)", text)
    return None if not m else float(m.group(2)) * (1 if m.group(1) == "+" else -1)


def main(path):
    rows = parse(path)
    print("parsed rows:", len(rows))
    print("status:", dict(collections.Counter(r["status"] for r in rows)))

    # Best-known-at-the-time, walked in ledger order (the ledger is chronological).
    best = None
    improving_rejects, improving_accepts = [], []
    for r in rows:
        if r["score"] == "n/a":
            continue
        s = float(r["score"])
        if r["status"] == "promoted" and (best is None or s < best):
            best = s
        elif r["status"] == "rejected" and best is not None:
            d = signed(r["delta"])
            if d is not None and d < 0:                      # beat its own tip
                improving_rejects.append((s - best, r, best))
    # accepts: promotions that improved the then-best, with their gap
    best = None
    for r in rows:
        if r["score"] == "n/a":
            continue
        s = float(r["score"])
        if r["status"] == "promoted":
            if best is not None and s < best:
                improving_accepts.append((s - best, r, best))
            if best is None or s < best:
                best = s

    improving_rejects.sort(key=lambda t: t[0])
    improving_accepts.sort(key=lambda t: t[0], reverse=True)

    print()
    print("improving-but-rejected (beat the best known at the time, still closed):",
          len(improving_rejects))
    print("distinct solvers:", len(set(r["solver"] for _, r, _ in improving_rejects)))
    print("gap distribution (score - best):")
    gaps = [g for g, _, _ in improving_rejects]
    if gaps:
        print("  worst (largest gain) %+.7f   median %+.7f   smallest %+.7f"
              % (gaps[0], gaps[len(gaps) // 2], gaps[-1]))
    print()
    print("=== improving rejections, sorted by size of gain ===")
    for g, r, b in improving_rejects:
        print("gap %+.7f  %s  %-16s score %-9s best %-9s %s"
              % (g, r["sub"], r["solver"], r["score"], b, r["created"]))

    print()
    print("=== CALIBRATION: the bar is RELATIVE (fraction of the current best) ===")
    print("max gap still rejected: %+.7f" % (improving_rejects[0][0] if improving_rejects else 0))
    if improving_accepts:
        g, r, b = improving_accepts[0]
        print("min gap still accepted: %+.7f  (%s, score %s vs best %s)"
              % (g, r["sub"], r["score"], b))
        print("implied relative bar: %.3e < bar < %.3e"
              % (abs(improving_rejects[0][0]) / float(improving_rejects[0][2]),
                 abs(g) / float(b)))
        print()
        print("acceptances with the smallest gain over the then-best:")
        for g, r, b in improving_accepts[:6]:
            print("gap %+.7f  %s  %-16s score %-9s best %-9s   rel %.2f bip"
                  % (g, r["sub"], r["solver"], r["score"], b, abs(g) / float(b) * 1e4))
    print()
    print("=== control: every 9/14-scored entry cited in 0278 (same-day corpus) ===")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "ledger.txt")
