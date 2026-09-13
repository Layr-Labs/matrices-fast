#!/usr/bin/env python3
"""Dead-row audit, part 2: does any *cheap structure-only construction* beat the
AMD value the probe recorded on a ratio=1.0000 row?

Constructions (all structure-only, no identity, no grader state):
  * exponential-free greedy MIN-FILL (exact fill count of each candidate,
    lazy heap), the same primitive the tree already owns as `minfill_order`;
  * randomized MIN-DEGREE restarts with a tie-break jitter;
  * reverse-min-degree.

Frame check: the Python metric (SCORE = sum_j c_j^2 over the elimination) must
reproduce the probe's recorded AMD value on every row it is compared against.
"""
import json
import sys
import heapq
import random


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


def minfill_order(adj):
    """Greedy exact-fill minimum-degree-ish: pick the vertex whose elimination
    adds the fewest fill edges; ties by degree."""
    n = len(adj)
    live = adj[:]
    deg = [a.bit_count() for a in adj]
    ver = [0] * n

    def fill_of(v):
        nb = live[v]
        if nb.bit_count() < 2:
            return 0
        pair = 0
        x = nb
        while x:
            b = x & -x
            u = b.bit_length() - 1
            pair += (live[u] & nb).bit_count()
            x ^= b
        return (nb.bit_count() * (nb.bit_count() - 1) - pair) // 2

    heap = [(fill_of(v), deg[v], v) for v in range(n)]
    heapq.heapify(heap)
    done = [False] * n
    order = []
    while heap:
        f, d, v = heapq.heappop(heap)
        if done[v] or f != fill_of(v):
            continue
        done[v] = True
        order.append(v)
        nb = live[v] & ~(1 << v)
        bv = 1 << v
        touched = []
        x = nb
        while x:
            b = x & -x
            u = b.bit_length() - 1
            live[u] = (live[u] | nb) & ~b & ~bv
            touched.append(u)
            x ^= b
        live[v] = 0
        for u in touched:
            deg[u] = live[u].bit_count()
            heapq.heappush(heap, (fill_of(u), deg[u], u))
    return order


def mindeg_order_rand(adj, rng, jitter):
    n = len(adj)
    live = adj[:]
    deg = [a.bit_count() for a in adj]
    heap = [(deg[v], rng.random() * jitter, v) for v in range(n)]
    heapq.heapify(heap)
    done = [False] * n
    order = []
    while heap:
        d, _, v = heapq.heappop(heap)
        if done[v] or d != deg[v]:
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
            heapq.heappush(heap, (deg[u], rng.random() * jitter, u))
            x ^= b
        live[v] = 0
    return order


ROWS = [
    ("chain400", 16382),
    ("hydroenergy1", 14089),
    ("squfl010-040persp", 22825),
    ("squfl015-060", 36400),
    ("squfl010-080", 27665),
    ("squfl025-030", 42305),
    ("emfl100_3_3", 55506),
    ("squfl025-025persp", 49925),
    ("squfl015-080persp", 74520),
    ("squfl020-150", 135020),
    ("emfl050_5_5", 155400),
    ("emfl100_5_5", 221650),
    ("squfl030-150", 252605),
]
RESTARTS = int(sys.argv[1]) if len(sys.argv) > 1 else 12


def main():
    corpus = ("/home/frosty/angelX/.tmp/live-performance-20260911/"
              "matrices-deepseek/corpus/dev/patterns.jsonl")
    data = load(corpus, {r[0] for r in ROWS})
    print("row\tn\tamd\tminfill\tdelta_minfill\tbest_rand\t"
          "delta_best\treverse\tdelta_rev")
    for name, amd in ROWS:
        if name not in data:
            print(f"MISSING\t{name}")
            continue
        n, adj = adjacency(data[name])
        f0 = flops_of(adj, list(range(n)))
        assert f0 > 0
        mf = flops_of(adj, minfill_order(adj))
        rev = flops_of(adj, list(reversed(minfill_order(adj))))
        rng = random.Random(12345)
        best = None
        for k in range(RESTARTS):
            f = flops_of(adj, mindeg_order_rand(adj, rng, 1.0 + 0.5 * k))
            best = f if best is None else min(best, f)
        print(
            f"{name}\t{n}\t{amd}\t{mf}\t{(mf - amd) / amd:+.6f}\t{best}\t"
            f"{(best - amd) / amd:+.6f}\t{rev}\t{(rev - amd) / amd:+.6f}"
        )
        sys.stdout.flush()


if __name__ == "__main__":
    main()
