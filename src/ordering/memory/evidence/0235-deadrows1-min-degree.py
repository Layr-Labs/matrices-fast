#!/usr/bin/env python3
"""Dead-row audit: for every dev row whose dev probe ratio is exactly 1.0000
(the pipeline shipped AMD untouched), compute the exact flops metric
(SCORE = sum_j c_j^2, c_j = nnz in column j of L, diagonal included) for a
handful of *legitimate* structure-only orderings and compare against the AMD
value the probe recorded. If any ordering beats `amd`, the row is not pinned:
the pipeline shipped a permutation a benign search would improve.

No grader/hidden state is touched: the corpus is the public dev JSONL and the
metric is the same symbolic elimination the probe uses.
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
            src = o.get("source")
            if src in want:
                out[src] = o
                if len(out) == len(want):
                    break
    return out


def adjacency(o):
    n = o["n"]
    indptr, indices = o["indptr"], o["indices"]
    adj = [0] * n
    e = 0
    for j in range(n):
        for k in range(indptr[j], indptr[j + 1]):
            i = indices[k]
            if i == j:
                continue
            if i > j:
                e += 1
            adj[j] |= 1 << i
            adj[i] |= 1 << j
    return n, adj, e


def flops_and_nnzl(adj, order):
    live = adj[:]
    total = 0
    nnzl = 0
    for v in order:
        bv = 1 << v
        nb = live[v] & ~bv
        c = nb.bit_count() + 1
        total += c * c
        nnzl += c
        x = nb
        while x:
            b = x & -x
            u = b.bit_length() - 1
            live[u] = (live[u] | nb) & ~b & ~bv
            x ^= b
        live[v] = 0
    return total, nnzl


def mindeg_order(adj, alpha=0.0):
    n = len(adj)
    live = adj[:]
    deg = [a.bit_count() for a in adj]
    heap = [(deg[v], v) for v in range(n)]
    heapq.heapify(heap)
    order = []
    done = [False] * n
    while heap:
        d, v = heapq.heappop(heap)
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
            live[u] |= nb
            live[u] &= ~b & ~bv
            deg[u] = live[u].bit_count()
            heapq.heappush(heap, (deg[u], u))
            x ^= b
        live[v] = 0
    return order


def mcs_order(adj):
    n = len(adj)
    wt = [0] * n
    done = [False] * n
    order = []
    heap = [(0, -v) for v in range(n)]
    heapq.heapify(heap)
    while heap:
        w, nv = heapq.heappop(heap)
        v = -nv
        if done[v] or w != wt[v]:
            continue
        done[v] = True
        order.append(v)
        x = adj[v]
        while x:
            b = x & -x
            u = b.bit_length() - 1
            if not done[u]:
                wt[u] += 1
                heapq.heappush(heap, (wt[u], -u))
            x ^= b
    return order


ROWS = [
    ("emfl100_5_5", 221650),
    ("squfl030-150", 252605),
    ("supplychainr1_053050", 2173490),
    ("emfl050_5_5", 155400),
    ("squfl020-150", 135020),
    ("squfl015-080persp", 74520),
    ("emfl100_3_3", 55506),
    ("squfl025-025persp", 49925),
    ("squfl015-060", 36400),
    ("squfl010-080", 27665),
    ("squfl025-030", 42305),
    ("squfl010-040persp", 22825),
    ("chain400", 16382),
    ("hydroenergy1", 14089),
]


def main():
    corpus = ("/home/frosty/angelX/.tmp/live-performance-20260911/"
              "matrices-deepseek/corpus/dev/patterns.jsonl")
    want = {r[0] for r in ROWS}
    data = load(corpus, want)
    print("row\tn\tedges\tamd\tmindeg\tmcs\tmindeg_delta\tmcs_delta\t"
          "md_fill\tmcs_fill\tnnzL_amd_implied")
    for name, amd in ROWS:
        if name not in data:
            print(f"MISSING\t{name}")
            continue
        n, adj, e = adjacency(data[name])
        md = mindeg_order(adj)
        mc = mcs_order(adj)
        f_md, z_md = flops_and_nnzl(adj, md)
        f_mc, z_mc = flops_and_nnzl(adj, mc)
        print(
            f"{name}\t{n}\t{e}\t{amd}\t{f_md}\t{f_mc}\t"
            f"{(f_md - amd) / amd:+.6f}\t{(f_mc - amd) / amd:+.6f}\t"
            f"{z_md - (n + e)}\t{z_mc - (n + e)}\t-"
        )
        sys.stdout.flush()


if __name__ == "__main__":
    main()
