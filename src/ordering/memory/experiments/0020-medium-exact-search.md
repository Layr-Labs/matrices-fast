# 0020 — Bounded exact search for medium sparse graphs

- **Date:** 2026-09-02
- **Score:** 0.860780 → **0.859116** (fill 0.955916 → **0.955319**)
- **Status:** win

## Hypothesis

The exact elimination-game search already gives the largest small-bucket gains,
but its shipped gate ends at `n = 1,000`. The degree-bucket path in `rgreedy`
supports larger graphs, and an operation budget bounds search work independently
of graph dimension. A serial search on sparse graphs just above the old gate
should improve the `1k_10k` bucket without touching the accepted small-graph
path or the expensive large-matrix tier.

## What changed

`order()` now runs two serial exact-search stages when
`1,000 < n <= 6,000 && nnz <= 30,000`. The stages use fixed nominal budgets of
100M and 50M word operations and the fixed seed `0xD1B54A32D192ED03`. Each stage
starts from the current best permutation and is accepted only on a strict
exact-flop improvement. No threads, clock reads, new dependencies, or matrix
identities are used. When the existing pair-descent gate allows it, the existing
four-sweep adjacent-pair pass is also repeated after this search so it can refine
the new incumbent.

The 100M and 50M values are nominal budgets in the inherited search. Its internal
guard allows 25% extra work and checks at pivot boundaries, so one final pivot
can pass that guard. The limits remain deterministic operation bounds rather
than clock limits.

The original `n <= 1,000` search branch is unchanged. This is important because
recent hidden failures showed that extra work on that branch can breach the
2-second cap.

## Result

The full 300-matrix Yukon run passed:

| Bucket | Before | After |
|---|---:|---:|
| `lt_1k` | 0.896482 | 0.896482 |
| `1k_10k` | 0.890302 | **0.884759** |
| `gt_10k` | 0.811860 | 0.811860 |
| weighted score | 0.860780 | **0.859116** |

Representative exact-flop changes against the parent were:

- `nuclear25a`: 1,314,803 → 1,192,314
- `waste`: 1,433,519 → 1,333,705
- `pooling_sppa0pq`: 1,496,775 → 1,437,861
- `batchs121208m`: 138,030 → 130,839
- `risk2bpb`: 71,424 → 68,025

The score path is monotone: each search result and each pair-descent result is
re-scored by the trusted scorer and accepted only if it is lower than the
incumbent.

The final timing probe measured a 0.755 s worst `order()` call. The first
candidate probe measured 0.843 s, and the synced parent measured 0.776 s on the
same machine. The spread confirms that wall-clock timing is noisy, but all three
runs remain below the recommended 1 s local target and the enforced 2 s cap.

## Gate and budget sweep

- One 50M stage through `n <= 2,000`: projected 0.860385; worst added local
  search time 0.016 s.
- One 50M stage through `n <= 4,000`: projected 0.860065.
- One 50M stage through `n <= 6,000`: projected 0.859841; worst added local
  search time 0.021 s.
- Extending the same stage through `n <= 10,000` produced no further wins and
  raised worst added time to 0.047 s, so it was rejected.
- A 100M first stage through the final gate projected 0.859461. A repeated 50M
  stage reached 0.859153. The final post-search pair pass reached 0.859116 in
  the complete harness.
- A third 50M search could reach about 0.858996 locally, but it was not shipped:
  the extra gain did not justify more hidden-cap risk.

## Negative result

One relabelled pass of each remaining numbering-sensitive pure-Rust family
(RCM, both Sloan weights, BFS nested dissection, and GGGP nested dissection)
produced zero wins over the current portfolio. Their combined worst added local
time was 0.071 s. More seeds remain untested, but a broad production fan is not
supported by this result.

## Why it won

The existing medium portfolio uses approximate quotient graphs and separator
heuristics. The bounded search maintains the exact fill graph and accumulates
the scored `sum(c_j^2)` objective during elimination. It therefore finds
orderings outside the approximate AMD/AMF candidate set. Two stages help because
the lower first-stage incumbent increases pruning in the second stage and gives
the local search a better starting basin.

## Follow-ups

- Keep the medium gate at or below 6,000 vertices unless a larger budget can
  complete useful trajectories; the 10,000-vertex probe added cost but no wins.
- A third stage is measurable but should be reconsidered only with stronger
  hidden-time evidence.
- Relabelled RCM/Sloan/ND needs a multi-seed probe before any production change.

## Links

- Technique: [best-of portfolio](../techniques/best-of-portfolio.md)
- Predecessor: [0017 exact small-graph search](0017-exact-small-graph-lns-search.md)
