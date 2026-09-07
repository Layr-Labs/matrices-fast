# 0094 — Subtree-refined basins into the donor pool (fixed 8)

- **Date:** 2026-09-07
- **Score:** 0.813331 → 0.812890 (−4.4 bips dev; fill 0.933055 → 0.933063)
- **Status:** WIN locally (11 movers / 7 regressions, net −4.4); submitted.

## Hypothesis

Transplant assembles winning segments from donors and PEO chains converge
from donor basins, but the pool holds only portfolio-displaced outputs.
Subtree-refined orderings carry strictly better segments (they are refinements
of the leader), so retaining them changes pool composition toward refined
basins at fixed pool size — a source change, not a count change (cf. 0079's
kill: count is width-exposed, composition is not).

## What changed

`src/ordering/mod.rs`: new `retain_alt` closure mirroring flush_batch's
push/sort/dedup/truncate-to-8 protocol; called at the five main-chain
subtree round sites (rounds 1–5) before best-update. Six insertions + closure.

## Result

| bucket | before | after |
|---|---:|---:|
| `lt_1k` | 0.8896 | 0.8896 |
| `1k_10k` | 0.8631 | 0.8556 |
| `gt_10k` | 0.7241 | 0.7234 |

11 movers (pooling_sppc3pq 0.460→0.408, batchs201210m 0.915→0.869,
rsyn0820m04m 0.834→0.820, 8 small) against 7 regressions (worst
procurement1large 0.535→0.575). Net −4.4 bips.

## Why regressions happen (understood, disclosed)

Pool composition feeds two consumers (alt chains iterate all seeds;
transplant assembles from all donors) under a shared first-come ledger, so
a changed pool reshuffles chains/assemblies downstream even where every
stage is strict-accept locally. The 7 regressions are that cascade, not a
bug: same mechanism that finds the 11 wins. No new cap exposure anywhere
(pool count, ledgers, gates all unchanged) — worst case is score noise,
never a timeout.

## Follow-ups

- Hidden result decides: confirm → widen to terminal/extra chains' rounds;
  reject → composition variance dominates and donor-pool work closes.

## Links

- Research queue: [open-questions](../open-questions.md)
