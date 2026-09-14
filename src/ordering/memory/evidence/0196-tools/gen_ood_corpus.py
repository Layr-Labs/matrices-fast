#!/usr/bin/env python3
"""0196: out-of-distribution structural stress corpus.

Why: the dev corpus has a structural hole -- above n=10 000 its median nnz/n is
5.7 and its max is 39.6 (all lt_1k rows), so every n-shaped gate/scope decision
this run has made was blind to the variable that actually sets per-row work.
This generator samples STRUCTURAL FAMILIES (never corpus instances): 2D/3D grid
graphs, uniform random sparse at fixed average degree, block-angular KKT-like
systems (dense diagonal blocks + coupling rows, the shape optimization solvers
produce), random geometric graphs and scale-free (hub) graphs.

Output: corpus JSONL in the same schema as corpus/dev/patterns.jsonl
(n, nnz, indptr, indices, hash, source) with symmetric CSC patterns including
the diagonal (the loader drops it).  Instrument only -- never shipped, never
part of a submission, no identity is read by any ordering code.
"""
import hashlib
import json
import random
import sys

OUT = sys.argv[1] if len(sys.argv) > 1 else "/tmp/ood/patterns.jsonl"
ONLY = sys.argv[2:] if len(sys.argv) > 2 else None


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


def grid2d(k):
    n = k * k
    e = []
    for i in range(k):
        for j in range(k):
            v = i * k + j
            if j + 1 < k:
                e.append((v, v + 1))
            if i + 1 < k:
                e.append((v, v + k))
    return n, e


def grid3d(k):
    n = k ** 3
    e = []
    idx = lambda i, j, l: (i * k + j) * k + l
    for i in range(k):
        for j in range(k):
            for l in range(k):
                v = idx(i, j, l)
                if l + 1 < k:
                    e.append((v, idx(i, j, l + 1)))
                if j + 1 < k:
                    e.append((v, idx(i, j + 1, l)))
                if i + 1 < k:
                    e.append((v, idx(i + 1, j, l)))
    return n, e


def rand_deg(n, d, seed):
    rng = random.Random(seed)
    seen = set()
    e = []
    target = n * d // 2
    while len(e) < target:
        i = rng.randrange(n)
        j = rng.randrange(n)
        if i == j:
            continue
        key = (min(i, j), max(i, j))
        if key in seen:
            continue
        seen.add(key)
        e.append(key)
    return n, e


def kkt_blocks(nb, bs, coupling, intra, seed):
    """Block-angular KKT-like pattern: nb dense-ish blocks of size bs plus
    `coupling` border rows connected into every block (the shape a
    block-decomposed optimization KKT system has)."""
    rng = random.Random(seed)
    n = nb * bs + coupling
    e = []
    for b in range(nb):
        base = b * bs
        target = bs * intra // 2
        seen = set()
        while len(seen) < target:
            i = rng.randrange(bs)
            j = rng.randrange(bs)
            if i == j:
                continue
            key = (min(i, j), max(i, j))
            if key in seen:
                continue
            seen.add(key)
            e.append((base + key[0], base + key[1]))
    border = nb * bs
    for c in range(coupling):
        for b in range(nb):
            base = b * bs
            for _ in range(3):
                e.append((border + c, base + rng.randrange(bs)))
    return n, e


def geo(n, k, seed):
    """Random geometric graph: n points in a k x k box joined when within
    radius r chosen for average degree ~ 12."""
    rng = random.Random(seed)
    pts = [(rng.random() * k, rng.random() * k) for _ in range(n)]
    import math
    r = math.sqrt(12.0 * k * k / (math.pi * n)) / 1.35
    cells = {}
    cs = r
    for i, (x, y) in enumerate(pts):
        cells.setdefault((int(x / cs), int(y / cs)), []).append(i)
    e = []
    for i, (x, y) in enumerate(pts):
        cx, cy = int(x / cs), int(y / cs)
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                for j in cells.get((cx + dx, cy + dy), ()):
                    if j <= i:
                        continue
                    xj, yj = pts[j]
                    if (x - xj) ** 2 + (y - yj) ** 2 <= r * r:
                        e.append((i, j))
    return n, e


def scale_free(n, m, seed):
    """Barabasi-Albert: m edges per new vertex, hub-heavy (large-degree
    vertices are the stress case for degree-ordered elimination)."""
    rng = random.Random(seed)
    targets = list(range(m + 1))
    e = []
    for v in range(m + 1):
        for u in range(v):
            e.append((u, v))
    repeated = []
    for v in range(m + 1, n):
        choices = set()
        while len(choices) < m:
            choices.add(repeated[rng.randrange(len(repeated))] if repeated and rng.random() < 0.95
                        else rng.randrange(v))
        for t in choices:
            e.append((t, v))
        targets.extend(choices)
        repeated.extend(choices)
    return n, e


def band_banded(n, w):
    """Symmetric banded pattern of half-width w (stiffness-matrix shape:
    large n, small degree, deep elimination)."""
    e = []
    for i in range(n):
        for k in range(1, w + 1):
            if i + k < n:
                e.append((i, i + k))
    return n, e


def main():
    rows = []
    for k in (40, 80, 130, 200, 320):
        n, e = grid2d(k)
        rows.append((f"ood_grid2d_{k}", n, sym(n, e)))
    for k in (16, 26, 40):
        n, e = grid3d(k)
        rows.append((f"ood_grid3d_{k}", n, sym(n, e)))
    for n, d, sd in ((2000, 12, 1), (6000, 12, 2), (20000, 10, 3), (20000, 20, 4),
                     (40000, 10, 5), (40000, 20, 6), (40000, 40, 7), (20000, 40, 8),
                     (60000, 8, 9), (300000, 6, 10)):
        nn, e = rand_deg(n, d, sd)
        rows.append((f"ood_rand_d{d}_n{n}", nn, sym(nn, e)))
    for nb, bs, cp, intra, sd in ((40, 200, 200, 12, 11), (100, 300, 300, 10, 12),
                                  (200, 200, 400, 8, 13)):
        n, e = kkt_blocks(nb, bs, cp, intra, sd)
        rows.append((f"ood_kkt_b{nb}x{bs}+{cp}", n, sym(n, e)))
    for n, k, sd in ((20000, 400, 14), (40000, 600, 15)):
        nn, e = geo(n, k, sd)
        rows.append((f"ood_geo_n{n}", nn, sym(nn, e)))
    for n, m, sd in ((20000, 6, 16), (40000, 6, 17)):
        nn, e = scale_free(n, m, sd)
        rows.append((f"ood_ba_m{m}_n{n}", nn, sym(nn, e)))
    for n, w in ((50000, 8), (200000, 4)):
        nn, e = band_banded(n, w)
        rows.append((f"ood_band_w{w}_n{n}", nn, sym(nn, e)))
    if ONLY:
        keep = set(ONLY)
        rows = [r for r in rows if r[0] in keep]
    emit(rows, OUT)


main()
