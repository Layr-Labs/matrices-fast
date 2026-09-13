#!/usr/bin/env python3
"""iter58 (0240): admission-key census of the terminal class block.

Computes, for every dev row, the structural clauses of the two gates that
decide which exact-window device (if any) a row receives:

  class gate      n >= 6 && n <= class_n(25 000) && nnz <= K && nnz <= 16n && max_deg <= n/2
  follow-up gate  n >= 6 && n <= class_n        && nnz <= K && best_flops <= 2e10
  (inside the follow-up: dense/hub site)  nnz > 16n || max_deg > n/2
  (and inside that: the block's real inner bound)  nnz_l <= 150 000

`nnz` = off-diagonal nonzeros (the harness contract drops the diagonal),
`max_deg` = largest column length after dropping the diagonal.

Usage: python3 0240-gate-census.py [K]   (K defaults to 200000)
"""
import collections
import json
import sys

K = int(sys.argv[1]) if len(sys.argv) > 1 else 200_000
CLASS_N = 25_000
PATH = "corpus/dev/patterns.jsonl"


def bucket(n):
    return "lt_1k" if n < 1000 else ("1k_10k" if n <= 10_000 else "gt_10k")


rows = []
with open(PATH) as fh:
    for line in fh:
        o = json.loads(line)
        n, indptr, idx = o["n"], o["indptr"], o["indices"]
        deg = [
            sum(1 for k in range(indptr[j], indptr[j + 1]) if idx[k] != j)
            for j in range(n)
        ]
        rows.append((n, sum(deg), max(deg), o.get("source", "?")))

print(f"corpus rows: {len(rows)}   key K = {K}   class_n = {CLASS_N}")
print("bucket counts:", dict(collections.Counter(bucket(r[0]) for r in rows)))

above = [r for r in rows if r[1] > K]
print(f"\nrows above the key (nnz > {K}): {len(above)}")
print("  by bucket:", dict(collections.Counter(bucket(r[0]) for r in above)))
for n, nnz, mx, src in sorted(above, key=lambda r: -r[1]):
    print(
        f"   {src:38s} n={n:7d} nnz={nnz:9d} nnz/n={nnz / n:7.2f} "
        f"maxdeg={mx:6d} {bucket(n):7s} "
        f"n>class_n={'Y' if n > CLASS_N else 'n'} "
        f"dense={'Y' if nnz > 16 * n else 'n'} hub={'Y' if mx > n // 2 else 'n'}"
    )

reach_class = [r for r in above if r[0] <= CLASS_N and r[1] <= 16 * r[0] and r[2] <= r[0] // 2]
reach_follow = [r for r in above if r[0] <= CLASS_N and (r[1] > 16 * r[0] or r[2] > r[0] // 2)]
print(f"\nfail the class-gate key alone (armed by raising only this key): {len(reach_class)}")
for r in reach_class:
    print(f"   {r[3]} n={r[0]} nnz={r[1]}")
print(f"fail the follow-up key (routed to the dense/hub site once armed): {len(reach_follow)}")
for r in reach_follow:
    print(f"   {r[3]} n={r[0]} nnz={r[1]}")

for clause, pred in (
    ("n > class_n", lambda n, nnz, mx: n > CLASS_N),
    ("nnz > K", lambda n, nnz, mx: nnz > K),
    ("nnz > 16n", lambda n, nnz, mx: nnz > 16 * n),
    ("max_deg > n/2", lambda n, nnz, mx: mx > n // 2),
):
    sel = [r for r in rows if pred(r[0], r[1], r[2])]
    print(
        f"\nclause {clause:16s} excludes {len(sel):3d} rows "
        f"{dict(collections.Counter(bucket(r[0]) for r in sel))}"
    )

hub = [r for r in rows if r[2] > r[0] // 2]
print(f"\nhub class (max_deg > n/2): {len(hub)} rows, "
      f"{sum(1 for r in hub if r[0] < 1000)} of them lt_1k")
print("   n, maxdeg/n  -- the class is dominated by tiny saturated graphs:")
for n, nnz, mx, src in sorted(hub, key=lambda r: r[0])[:40]:
    print(f"   {src:34s} n={n:6d} nnz={nnz:7d} maxdeg={mx:4d} d/n={mx / n:5.2f} "
          f"dense={'Y' if nnz > 16 * n else 'n'}")
