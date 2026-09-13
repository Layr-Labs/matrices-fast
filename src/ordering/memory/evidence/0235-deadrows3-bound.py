#!/usr/bin/env python3
"""Dead-row audit, part 3: how far is the shipped ordering from the *provable*
lower bound on the metric?

For any ordering the metric satisfies SCORE >= n + 3*E + 2*T (the tree's own
`fill_free_certificate_exhaustive` asserts exactly this), with equality iff the
elimination creates no fill. So `gap = SCORE - (n + 3E + 2T)` is an exact,
checkable *distance from the optimum*: gap == 0 proves the shipped ordering is
globally optimal on that row and no device can ever improve it.

Rows: the dev rows whose dev probe ratio is exactly 1.0000 (the pipeline shipped
AMD untouched) plus a few controls. SCORE for the shipped ordering is taken from
the probe's own COUNTS line (`amd == mine` there).
"""
import json
import sys


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
    e = 0
    for j in range(n):
        for k in range(indptr[j], indptr[j + 1]):
            i = indices[k]
            if i != j:
                adj[j] |= 1 << i
                adj[i] |= 1 << j
                if i > j:
                    e += 1
    return n, adj, e


def triangles(adj):
    t = 0
    for u in range(len(adj)):
        nb = adj[u]
        x = nb & ((1 << u) - 1)
        while x:
            b = x & -x
            v = b.bit_length() - 1
            t += (adj[v] & nb).bit_count()
            x ^= b
    return t  # each triangle counted 3 times, over pairs u>v


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


ROWS = [
    # name, probe SCORE (=AMD value; ratio is exactly 1.0 in the dev frame)
    ("chain200", 8182),
    ("chain400", 16382),
    ("hydroenergy1", 14089),
    ("slay08m", 7796),
    ("squfl010-040persp", 22825),
    ("squfl010-080", 27665),
    ("squfl015-060", 36400),
    ("squfl025-030", 42305),
    ("emfl100_3_3", 55506),
    ("squfl025-025persp", 49925),
    ("emfl050_5_5", 155400),
    ("squfl015-080persp", 74520),
    ("squfl020-150", 135020),
    ("squfl030-150", 252605),
    ("emfl100_5_5", 221650),
    ("supplychainr1_053050", 2173490),
    ("kissing2", 156414018),
]
# controls: rows the pipeline DOES beat (ratio < 1)
CONTROLS = [("oil", 0), ("ndcc12", 0)]


def main():
    corpus = ("/home/frosty/angelX/.tmp/live-performance-20260911/"
              "matrices-deepseek/corpus/dev/patterns.jsonl")
    rows = ROWS + CONTROLS
    data = load(corpus, {r[0] for r in rows})
    print("row\tn\tE\ttri\tbound\tscore\tgap\tgap_pct\tnnzL\tfill\tnatural")
    for name, score in rows:
        if name not in data:
            print(f"MISSING\t{name}")
            continue
        n, adj, e = adjacency(data[name])
        T = triangles(adj)
        bound = n + 3 * e + 2 * T
        nat, nat_nnzl = flops_and_nnzl(adj, list(range(n)))
        if score == 0:
            # control: report the natural order only
            print(f"{name}\t{n}\t{e}\t{T}\t{bound}\t-\t-\t-\t-\t-\t{nat}")
            sys.stdout.flush()
            continue
        gap = score - bound
        print(
            f"{name}\t{n}\t{e}\t{T}\t{bound}\t{score}\t{gap}\t"
            f"{100.0 * gap / score:.4f}\t-\t-\t{nat}"
        )
        sys.stdout.flush()


if __name__ == "__main__":
    main()
