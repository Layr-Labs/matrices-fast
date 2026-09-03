# 0025 — Spend the final subtree budget on fewer, deeper searches

- **Date:** 2026-09-03
- **Score:** frontier source 0.851168 → **0.849622** (−0.001546;
  fill 0.948675 → **0.947714**)
- **Status:** win; submission pending
- **Parent:** `9ec9816` (promoted hidden score 0.876991)

## Hypothesis

The five-round subtree chain still leaves useful local improvements, but another
32-block × 1M pass spreads its fixed matrix-wide budget too thinly. Ranking the
subtrees by incumbent contribution and spending the same requested work on fewer,
deeper trajectories should escape more local minima. Medium and large matrices
need different allocations because their bucket leverage and useful subtree sizes
differ.

## What changed

One terminal `subtree_refine` pass now runs after the complete existing chain,
whether or not the chain's last round improved:

- `1,000 <= n < 10,000`: 4 ranked blocks × 8M requested operations,
  `min_s=16`, `max_s=768`.
- `10,000 <= n <= 350,000`: 12 ranked blocks × 2.666M requested operations,
  `min_s=16`, `max_s=1,200`.
- Both branches use one stream, `max_sub=1,200`, and `round=5` seed
  diversification. The existing `nnz <= 1,500,000` gate remains.

Each branch requests at most 32M operations per matrix. The searcher's 1.25×
guard means the corresponding internal hard-cap allowance is at most 40M, plus
tree setup and canonical scoring. A candidate is retained only if it is a valid
bijection and strictly reduces the trusted `sum(c_j^2)` score.

## Result

| configuration, one terminal pass | dev score | delta |
|---|---:|---:|
| 32 blocks × 1M, max_s 768 | 0.850973 | −0.000195 |
| 16 blocks × 2M, max_s 768 | 0.850130 | −0.001038 |
| 8 blocks × 4M, max_s 768 | 0.849954 | −0.001214 |
| 6 blocks × 5.333M, max_s 768 | 0.849825 | −0.001343 |
| 4 blocks × 8M, max_s 768 | 0.849890 | −0.001278 |
| **bucket-adaptive 4×8M / 12×2.666M** | **0.849622** | **−0.001546** |

Final buckets are **0.896482 / 0.875268 / 0.795242** for
`lt_1k / 1k_10k / gt_10k`. The small bucket is unchanged because the subtree
gate starts at 1,000 vertices. On the regression fixture `rsyn0815m04m`, exact
flops fell from 170,169 to 164,459.

The full 300-matrix Yukon run passed. `probe_timing_and_score` measured a worst
`order()` time of **0.886 s** (`gams05`), below the 2 s limit and close to the
promoted parent's measured envelope.

## Why it won

At fixed aggregate work, search depth matters more than block count after five
shallow rounds have already harvested easy improvements. The ranker identifies
the high-contribution blocks; four long trajectories improve the medium bucket
more than 32 short trajectories. Large matrices benefit from a wider 1,200-node
window and twelve trajectories, which gives better coverage while still making
each search substantially deeper than the prior 1M passes.

The terminal placement also matters. It searches the final incumbent even when
an earlier conditional round did not improve, instead of depending on the whole
nested chain to fire.

## Follow-ups

- Measure whether a second allocation can improve the final incumbent without
  increasing the 32M per-phase hidden timing risk.
- Test ranking features that predict which four medium blocks deserve deep work;
  unranked selection lost badly in this sweep.

## Links

- Earlier bounded subtree pass: [0022](0022-bounded-subtree-work.md)
- Earlier widening chain: [0024](0024-subtree-round-4-chain.md)
- Portfolio and timing policy: [best-of-portfolio](../techniques/best-of-portfolio.md)
