# 0092 — Ported SSI-challenge families: quotient metrics on both tiers, heavy relabelled AMF, a byte-identical parallel portfolio, two cap-margin trims

**Date:** 2026-09-07. **Base:** `081af15` (hidden 0.857182, dev 0.824625).
**Result:** dev **0.814072** (−105.5 bips; lt_1k 0.8896 / 1k_10k 0.8564 / gt_10k 0.7257),
fill 0.9333; 23 rows better / 9 worse / 267 identical. Worst same-box `order()`
1.208 s (`crudeoil_lee4_10`) against the base's 1.146 s on the same quiet 2-vCPU
x86 pod, min-of-2; corpus total 116.8 s → 107.4 s. Real harness run: 300/300 OK
(bijection + determinism).

## Where the headroom was

The `ssi-ordering-challenge` tree (same dev corpus, same scorer) scores 0.810054
here against this crown's 0.824625, and the whole gap is `gt_10k` (0.7147 vs
0.7518). A per-phase incumbent trail on its twelve largest `gt_10k` movers put
the gain in the CANDIDATE PORTFOLIO — generators this crown never runs — not in
its post-hoc phases (its TELOS descent, ported and re-tested here at a 400M-unit
ledger, is worth ~1 bip on this crown: the subtree chains, completion watcher
and PEO re-extraction already harvest what it used to find).

## What shipped

1. **`parallel.rs`** — every `consider(...)` producer is queued
   (`consider!(move || ..)`), a batch is generated and scored on ≤4 threads, and
   `flush!()` replays the sequential acceptance + runner-up ledger in source
   order. Flushed at every gate that reads the incumbent (`flops_before_part`,
   `part_extra`, `part_extra2`, `extra_relabel`, the terminal descents).
   Verified bit-identical on 299/299 rows. A first draft that flushed one line
   after `flops_before_part` scored 0.824094: better, but a different program.
2. **Heavy tier (130k–1.2M nnz)**: `metric_sweep.rs` (`MetricSpec` generic
   pivot score on `feral_ordering_core::quotient_graph`) + `HEAVY_METRIC_ORDER`
   (`extra_deg2_div_nv_wf05` α10, `SqPure` α5, `..wf05` α2.5, `..wf002` α10; a
   `2.8M/nnz` prefix), dead window 200k–500k, sparse-only below 200k, one variant
   above 700k with the hub-scale variant on sparse-hub giants; plus no-dense AMD
   on sparse 250k–700k and AMF α{2.5,0.5} (<400k) / α{2.5,0.5,1,1.5,2} in the
   dense band 20<nnz/n<50 up to 700k. Movers: pooling_sppc3pq 0.63→0.46,
   crudeoil_pooling_dt3 0.90→0.74, cont6-qq 0.89→0.80, gabriel10 0.98→0.95.
3. **Light tier (<130k nnz)**: ten `ScoreVariant`s at α∈{10,1} (NOT AmindNorm,
   NOT the 15 extra specs — see below), queued AFTER the partitioner cascade.
   Movers: ringpack_30_2 0.41→0.23 (`DegSqrt` α1 alone), edgecross24-115
   0.91→0.82, crudeoil_lee4_06 0.80→0.74, crudeoil_lee4_09 0.78→0.73, glider400
   0.93→0.89.
4. **Heavy relabelled AMF** (200k–400k sparse α5, 420k–700k dense α2.5,
   `1.5M/nnz` passes ≤4): transswitch2736spr 0.987→0.924, pooling_sppc1pq.
5. **Trims**: ranked-subtree chain only for n ≤ 35k (`SUBTREE_CHAIN_MAX_N`),
   PEO alternate-seed chains only for n ≤ 50k (`PEO_ALT_MAX_N`). Measured with
   the new phase marks: on every dev row above those sizes the stages cost
   0.2–0.5 s and moved the ratio by < 0.01.

## Negative results (do not re-derive)

- Queueing the metric families BEFORE the cascade: −5 bips net but +22 bips on
  `mpbp_34` — the metric incumbent raises the bar `part_extra` compares METIS
  against, so KaHIP-Eco (the mpbp family's best generator) never runs.
- `AmindNorm` on the light tier: 0.72–0.80 s per pass on `popdynm200` (hub,
  max-degree 1200) for a 68585x ordering; every other variant 12 ms there.
- The 15 `metric_sweep` specs on the light tier: no wins, 15 symbolic scoring
  passes (0.3–0.5 s at 100k nnz).
- TELOS (main + top-K extras + 8-move-set multi-start), terminal, 400M ledger:
  4 wins on a 38-row subset (procurement1large −6.5 bips) for +0.2–0.5 s per row.
- D_WIDE above 700k: 0.13 s on the slowest row, no win the metric block lacked.

## Tooling added (test-only)

`SSI_PROBE_ONLY`, `SSI_PROBE_REPEAT`, `SSI_PROBE_PHASES` on
`probe_timing_and_score` (per-row phase time AND incumbent flops), and
`probe_census` (every generator alone on named rows, `SSI_CENSUS_ALL` for the
full list with seconds).
