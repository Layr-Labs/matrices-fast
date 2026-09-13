#!/usr/bin/env python3
"""0201: small-n / high-fill structural corpus.

Why: the 2 s cap is decided by rows whose *fill* work is an order of magnitude
above anything dev contains (dev max AMD fill 6.18e9; the hidden killing row is
somewhere above that, since the frontier's own margin there is < 0.1 s).  The
0200 fence is keyed on an ABSOLUTE fill threshold (2e10) plus a fixed count
(64) -- a window, exactly the shape the frontier's only bar-clearing step
removed at stage 1b.  To replace it with a budget-shaped guard the calibration
needs the joint distribution of (incumbent exact fill, batch length) on dev and
on the class where those batches are 5-10x dearer.

Generator: uniform random sparse graphs at fixed average degree, symmetric
CSC with the diagonal, same JSONL schema as corpus/dev/patterns.jsonl.  Pure
structure, no corpus identity, never shipped.
"""
import hashlib
import json
import random
import sys

OUT = sys.argv[1] if len(sys.argv) > 1 else "/tmp/cl3/patterns.jsonl"


def emit(rows, path):
    with open(path, "w") as fh:
        for name, n, adj in rows:
            nz = sum(len(a) for a in adj)
            indptr = [0]
            indices = []
            for a in adj:
                indices.extend(sorted(a))
                indptr.append(len(indices))
            h = hashlib.sha256(name.encode()).hexdigest()
            fh.write(json.dumps({
                "n": n, "nnz": nz, "indptr": indptr, "indices": indices,
                "hash": h, "source": name}) + "\n")
            sys.stderr.write(f"wrote {name} n={n} nnz={nz} nnz/n={nz/n:.2f}\n")


def sym(n, edges):
    adj = [set() for _ in range(n)]
    for i in range(n):
        adj[i].add(i)
    for (i, j) in edges:
        adj[i].add(j)
        adj[j].add(i)
    return adj


def fixed_degree(n, d, seed):
    """Random simple graph with average degree ~d (dedup, no self loops)."""
    rng = random.Random(seed)
    seen = set()
    edges = []
    target = (n * d) // 2
    guard = 0
    while len(edges) < target and guard < 20 * target + 1000:
        guard += 1
        i = rng.randrange(n)
        j = rng.randrange(n)
        if i == j:
            continue
        k = (i, j) if i < j else (j, i)
        if k in seen:
            continue
        seen.add(k)
        edges.append(k)
    return edges


def main():
    rows = []
    for (n, d) in [(2000, 8), (4000, 8), (6000, 8), (8000, 8), (9500, 8),
                   (9950, 7), (9950, 9), (9950, 11)]:
        name = f"fd_n{n}_d{d}"
        edges = fixed_degree(n, d, seed=1000 * d + n)
        rows.append((name, n, sym(n, edges)))
    emit(rows, OUT)


if __name__ == "__main__":
    main()
