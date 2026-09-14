#!/usr/bin/env python3
"""0202: scaled fill-work corpus -- hunt the row class that decides the 2 s cap.

Why: the promoted build (4e45ee63, hidden 0.842716) is a build whose fence bought
~0.2 s on the fill-heavy class, and the killed builds around it died at a fixed
corpus position. Dev contains no row in that class at all (max AMD fill 6.18e9),
so the only way to price the next shape is to *reproduce* the class locally and
scale it until a row crosses 2 s with the SHIPPED profile.

The 0200 sweep at fixed n = 8 000 found a non-monotone step of nnz with a spike
at nnz/n ~ 16 (1.42 s). This generator sweeps (n, avg degree) jointly across the
window the draw actually covers (n <= 9 950) and past it, at the degrees where
the incumbent's exact fill reaches 1e10..1e11.

Pure structure, same JSONL schema as corpus/dev/patterns.jsonl, never shipped.
"""
import hashlib
import json
import random
import sys

OUT = sys.argv[1] if len(sys.argv) > 1 else "/tmp/scale/patterns.jsonl"


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
    for n in (8000, 9000, 9950):
        for d in (10, 14, 16, 20, 24, 28):
            name = f"sc_n{n}_d{d}"
            rows.append((name, n, sym(n, fixed_degree(n, d, seed=1000 * d + n))))
    emit(rows, OUT)


if __name__ == "__main__":
    main()
