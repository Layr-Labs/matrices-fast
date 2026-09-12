#!/usr/bin/env python3
"""Per-row comparison of two probe logs against each other (ratio = flops/anchor).

Both logs must carry COUNTS lines `COUNTS name n nnz anchor flops`. The script
reports, for every row whose ours/anchor ratio moved, the band, n, nnz and the
absolute + relative ratio delta, plus bucket scores over the intersection.
"""
import sys, math

W = [0.30, 0.30, 0.40]


def load(path):
    d = {}
    for line in open(path):
        if line.startswith("COUNTS"):
            p = line.rstrip("\n").split("\t")
            _, name, n, nnz, anchor, flops = p[:6]
            d[name] = (int(n), int(nnz), int(anchor), int(flops))
    return d


def bucket(n):
    return 0 if n < 1000 else (1 if n < 10000 else 2)


def score(d, names=None):
    logs = [0.0] * 3
    cnt = [0] * 3
    for name, (n, nnz, a, f) in d.items():
        if names is not None and name not in names:
            continue
        b = bucket(n)
        logs[b] += math.log(f / a)
        cnt[b] += 1
    num = den = 0.0
    for b in range(3):
        if cnt[b]:
            num += W[b] * math.exp(logs[b] / cnt[b])
            den += W[b]
    return num / den, [math.exp(logs[b] / cnt[b]) if cnt[b] else float("nan") for b in range(3)], cnt


A = load(sys.argv[1])
B = load(sys.argv[2])
common = set(A) & set(B)
onlyA = sorted(set(A) - set(B))
onlyB = sorted(set(B) - set(A))
sa, ba, ca = score(A, common)
sb, bb, cb = score(B, common)
print(f"A {sys.argv[1]}")
print(f"  score={sa:.6f} buckets={[f'{x:.6f}' for x in ba]} counts={ca}")
print(f"B {sys.argv[2]}")
print(f"  score={sb:.6f} buckets={[f'{x:.6f}' for x in bb]} counts={cb}")
print(f"delta = {sb - sa:+.6f}  ({(sb - sa) / sa * 1e4:+.2f} bips relative)  common rows={len(common)}")
print(f"only in A: {onlyA}")
print(f"only in B: {onlyB}")
print()
print("band\tname\tn\tnnz\tratioA\tratioB\td_ratio_rel_pct\td_log_score_contrib")
rows = []
for name in sorted(common, key=lambda k: A[k][0]):
    n, nnz, a1, f1 = A[name]
    _, _, a2, f2 = B[name]
    r1, r2 = f1 / a1, f2 / a2
    if f1 == f2:
        continue
    b = bucket(n)
    contrib = W[b] * (math.log(r2) - math.log(r1)) / cb[b]
    rows.append((n, name, nnz, r1, r2, (r2 - r1) / r1 * 100.0, contrib))
rows.sort(key=lambda t: -abs(t[6]))
for n, name, nnz, r1, r2, dpct, contrib in rows:
    band = "<1k" if n < 1000 else ("1k-10k" if n < 10000 else "gt10k")
    print(f"{band}\t{name}\t{n}\t{nnz}\t{r1:.6f}\t{r2:.6f}\t{dpct:+.4f}\t{contrib:+.3e}")
tot = sum(r[6] for r in rows)
print(f"changed rows: {len(rows)}   total bucket-weighted log-delta = {tot:+.3e}")
by = {}
for n, name, nnz, r1, r2, dpct, contrib in rows:
    b = 0 if n < 1000 else (1 if n < 10000 else 2)
    by.setdefault(("lt_1k", "1k_10k", "gt_10k")[b], []).append((dpct, contrib))
for k, v in by.items():
    print(f"  {k}: {len(v)} rows, sum contrib {sum(c for _, c in v):+.3e}")
