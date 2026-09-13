#!/usr/bin/env python3
"""Dead-row audit, part 5: is the *tie-break inside min-degree* the missing
construction? The pipeline's exact greedy is pure MIN-FILL (`minfill_order`),
which is far worse than AMD on this class. The classic unrepresented rule is
lexicographic (degree, exact fill): pick any minimum-degree vertex, and among
them the one whose elimination creates the fewest fill edges. Cost is O(E log n)
plus fill evaluation on the few min-degree candidates — cap-cheap.

Frame: SCORE = sum_j c_j^2; the probe's recorded AMD value is reproduced first
(my plain min-degree equals it on 12/13 rows), so the metric is the grader's.
"""
import json
import sys
import heapq


def load(path, want):
    out = {}
    with open(path) as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            o = json.loads(line)
            if o.get("source") in want:
                out[o["source"]] = o
                if len(out) == len(want):
                    break
    return out


def adjacency(o):
    n = o["n"]
    indptr, indices = o["indptr"], o["indices"]
    adj = [0] * n
    for j in range(n):
        for k in range(indptr[j], indptr[j + 1]):
            i = indices[k]
            if i != j:
                adj[j] |= 1 << i
                adj[i] |= 1 << j
    return n, adj


def flops_of(adj, order):
    live = adj[:]
    total = 0
    for v in order:
        bv = 1 << v
        nb = live[v] & ~bv
        c = nb.bit_count() + 1
        total += c * c
        x = nb
        while x:
            b = x & -x
            u = b.bit_length() - 1
            live[u] = (live[u] | nb) & ~b & ~bv
            x ^= b
        live[v] = 0
    return total


def fill_of(live, v):
    nb = live[v]
    k = nb.bit_count()
    if k < 2:
        return 0
    pair = 0
    x = nb
    while x:
        b = x & -x
        u = b.bit_length() - 1
        pair += (live[u] & nb).bit_count()
        x ^= b
    return (k * (k - 1) - pair) // 2


def greedy(adj, key):
    """key = (deg, fill) or (fill, deg); lazy heap with exact re-evaluation."""
    n = len(adj)
    live = adj[:]
    deg = [a.bit_count() for a in adj]
    heap = [(deg[v], fill_of(live, v), v) for v in range(n)]
    heapq.heapify(heap)
    done = [False] * n
    order = []
    while heap:
        d, f, v = heapq.heappop(heap)
        if done[v]:
            continue
        fd = deg[v]
        ff = fill_of(live, v)
        cur = (d, f)
        want = (fd, ff) if key == "deg_fill" else (ff, fd)
        if cur != want:
            heapq.heappush(heap, (want[0], want[1], v))
            continue
        done[v] = True
        order.append(v)
        nb = live[v] & ~(1 << v)
        bv = 1 << v
        x = nb
        while x:
            b = x & -x
            u = b.bit_length() - 1
            live[u] = (live[u] | nb) & ~b & ~bv
            deg[u] = live[u].bit_count()
            x ^= b
        live[v] = 0
        x = nb
        while x:
            b = x & -x
            u = b.bit_length() - 1
            fu = fill_of(live, u)
            heapq.heappush(heap, (deg[u], fu, u))
            x ^= b
    return order


ROWS = [
    ("chain200", 8182),
    ("chain400", 16382),
    ("hydroenergy1", 14089),
    ("squfl010-040persp", 22825),
    ("squfl010-080", 27665),
    ("squfl015-060", 36400),
    ("squfl025-030", 42305),
    ("emfl100_3_3", 55506),
    ("squfl025-025persp", 49925),
    ("squfl015-080persp", 74520),
    ("squfl020-150", 135020),
    ("emfl050_5_5", 155400),
    ("emfl100_5_5", 221650),
    ("squfl030-150", 252605),
]


def main():
    corpus = ("/home/frosty/angelX/.tmp/live-performance-20260911/"
              "matrices-deepseek/corpus/dev/patterns.jsonl")
    data = load(corpus, {r[0] for r in ROWS})
    print("row\tn\tamd\tdeg_fill\tdelta\ttri_free")
    for name, amd in ROWS:
        if name not in data:
            print(f"MISSING\t{name}")
            continue
        n, adj = adjacency(data[name])
        f = flops_of(adj, greedy(adj, "deg_fill"))
        print(f"{name}\t{n}\t{amd}\t{f}\t{(f - amd) / amd:+.6f}\t-")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
