#!/usr/bin/env python3
"""Dead-row audit, part 4: do *traversal* orderings (BFS / RCM / DFS / level
order) beat AMD on the ratio=1.0000 rows? The tree's candidate portfolio is
partition- and degree-based (METIS shapes, AMF alphas, relabelled AMF, PEO
chains); a bandwidth-oriented traversal order is a different generator class and
costs O(E) to build, so it would be cap-cheap.

Metric frame: SCORE = sum_j c_j^2, checked against the probe's recorded AMD value
on every row before any conclusion is drawn.
"""
import json
import sys
from collections import deque


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
    adj = [[] for _ in range(n)]
    for j in range(n):
        for k in range(indptr[j], indptr[j + 1]):
            i = indices[k]
            if i != j and i not in adj[j]:
                adj[j].append(i)
    return n, adj


def flops_of(adjl, order):
    n = len(adjl)
    live = [0] * n
    for v in range(n):
        m = 0
        for u in adjl[v]:
            m |= 1 << u
        live[v] = m
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


def bfs_order(adjl, start, sort_neighbors=False):
    n = len(adjl)
    seen = [False] * n
    order = []
    q = deque([start])
    seen[start] = True
    while q:
        v = q.popleft()
        order.append(v)
        nbrs = adjl[v]
        if sort_neighbors:
            nbrs = sorted(nbrs, key=lambda u: (len(adjl[u]), u))
        for u in nbrs:
            if not seen[u]:
                seen[u] = True
                q.append(u)
    for v in range(n):  # disconnected parts
        if not seen[v]:
            seen[v] = True
            q.append(v)
            while q:
                w = q.popleft()
                order.append(w)
                for u in adjl[w]:
                    if not seen[u]:
                        seen[u] = True
                        q.append(u)
    return order


def dfs_order(adjl, start):
    n = len(adjl)
    seen = [False] * n
    order = []
    stack = [start]
    while stack:
        v = stack.pop()
        if seen[v]:
            continue
        seen[v] = True
        order.append(v)
        for u in reversed(adjl[v]):
            if not seen[u]:
                stack.append(u)
    for v in range(n):
        if not seen[v]:
            stack.append(v)
            while stack:
                w = stack.pop()
                if seen[w]:
                    continue
                seen[w] = True
                order.append(w)
                for u in reversed(adjl[w]):
                    if not seen[u]:
                        stack.append(u)
    return order


def peripheral(adjl, seed):
    # double sweep: farthest node from `seed`, then farthest from that
    def sweep(s):
        n = len(adjl)
        dist = [-1] * n
        dist[s] = 0
        q = deque([s])
        last = s
        while q:
            v = q.popleft()
            last = v
            for u in adjl[v]:
                if dist[u] < 0:
                    dist[u] = dist[v] + 1
                    q.append(u)
        return last, dist
    a, _ = sweep(seed)
    b, dist = sweep(a)
    return b, dist


ROWS = [
    ("chain200", 8182),
    ("chain400", 16382),
    ("hydroenergy1", 14089),
    ("squfl010-040persp", 22825),
    ("squfl010-080", 27665),
    ("squfl015-060", 36400),
    ("squfl025-030", 42305),
    ("emfl100_3_3", 55506),
]


def main():
    corpus = ("/home/frosty/angelX/.tmp/live-performance-20260911/"
              "matrices-deepseek/corpus/dev/patterns.jsonl")
    data = load(corpus, {r[0] for r in ROWS})
    print("row\tn\tamd\tbfs0\trcm\trcm_fwd\tdfs0\tlvl_desc\tlvl_asc\trcm_delta\tdfs_delta")
    for name, amd in ROWS:
        if name not in data:
            print(f"MISSING\t{name}")
            continue
        n, adjl = adjacency(data[name])
        deg0 = min(range(n), key=lambda v: (len(adjl[v]), v))
        b0 = flops_of(adjl, bfs_order(adjl, 0))
        bd = flops_of(adjl, bfs_order(adjl, deg0))
        rcm = bfs_order(adjl, deg0, sort_neighbors=True)
        f_rcm_rev = flops_of(adjl, list(reversed(rcm)))
        f_rcm_fwd = flops_of(adjl, rcm)
        d0 = flops_of(adjl, dfs_order(adjl, deg0))
        per, dist = peripheral(adjl, deg0)
        lvl_desc = sorted(range(n), key=lambda v: (-dist[v], v))
        lvl_asc = sorted(range(n), key=lambda v: (dist[v], v))
        f_ld = flops_of(adjl, lvl_desc)
        f_la = flops_of(adjl, lvl_asc)
        print(
            f"{name}\t{n}\t{amd}\t{b0}\t{f_rcm_rev}\t{f_rcm_fwd}\t{d0}\t{f_ld}\t{f_la}\t"
            f"{(f_rcm_rev - amd) / amd:+.6f}\t{(d0 - amd) / amd:+.6f}"
        )
        sys.stdout.flush()


if __name__ == "__main__":
    main()
