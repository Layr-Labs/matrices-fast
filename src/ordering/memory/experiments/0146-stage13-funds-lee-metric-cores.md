# 0146 — Stage 13 (alternate-seed PEO) pays for v5's lee-row quality

- **Date:** 2026-09-08
- **Score:** tip `ea7618f` (submission `62d660a4`, hidden 0.844081) official local
  **0.795871 → 0.794956** (−9.15 bips); gt_10k 0.694015 → **0.691727**;
  worst `order()` **0.907 s → 0.838 s** (this box)
- **Status:** win (dev); hidden pending
- **Files:** `indep_first.rs` (`X_SET_MAX_N` new at 20_000, `METRIC_CORE_MAX_N`
  12_000 → 16_000, `METRIC_TOP_CORES` 1 → 3, `DegSqrt` back in the metric menu),
  `mod.rs` (`PEO_ALT_MAX_N` 50_000 → 10_000)

## Hypothesis

Two facts in the notes sit next to each other and had not been put together.

1. [0145](0145-second-colour-class-metric-cores.md)/v11 is a *timing cut*. v5
   (`fe871f1`) scored the lee rows at 0.6473 / 0.6374 and was killed by the
   hidden 2 s cap; v11 bought the margin back by gating the x-sets at n ≤ 12_000
   and the metric passes to one core. The quality was demonstrated and then
   abandoned for wall-clock, not because it failed.
2. The pipeline had never been audited for score-per-millisecond. Wall-clock is
   the binding constraint on this problem (233 of the last 400 public
   submissions failed, mostly on the cap), so a stage that costs much and wins
   little is not merely waste — it is the budget for something better.

So: price every stage, find the worst payer, and spend its time on the quality
v11 could not afford.

## Method (and a measurement trap)

`SSI_PROBE_PHASES=1 … probe_timing_and_score` gives per-row wall-clock and each
phase's mark. Aggregating time and apparent ratio movement per phase over 300
rows nominates `13.alt` (alternate-seed PEO chains): 6.27 s corpus-wide, 0.10 s
on each of the slowest rows, and no visible ratio movement anywhere.

**That aggregate is not evidence, and nearly sent this the wrong way.**
`phase_mark` reports `best_flops`, but stages 12/13 improve `best_perm` WITHOUT
updating `best_flops` (the code says so), so their marks are flat by
construction — on `crudeoil_lee4_06` the last mark reads 0.5398 while the row
finishes at 0.5044. The only sound instrument is an A/B on the final per-row
ratio. Disabling the stage outright is what produced the number below.

## Result

| revision | score | worst | rows changed |
|---|---:|---:|---|
| tip `ea7618f` | 0.795871 | 0.907 s | — |
| stage 13 disabled entirely | 0.795890 | 0.821 s | 4 worse (its whole value) |
| x-sets ≤ 20k + metric cores ≤ 16k | 0.795746 | 0.972 s | 1 better (`pooling_dt3`) |
| + `DegSqrt`, metric top-3 | 0.794956 | 0.966 s | 4 better / 0 worse |
| **+ stage 13 gated n ≤ 10_000** | **0.794956** | **0.838 s** | **4 better / 0 worse** |

Rows improved, all `gt_10k` or its edge: `crudeoil_lee4_10` 0.6755 → **0.6374**
(−3.81 pp), `crudeoil_lee4_09` 0.6814 → **0.6473** (−3.41), `crudeoil_pooling_dt3`
0.7367 → **0.7219** (−1.48), `mpbp_34` 0.3089 → **0.3031** (−0.58). Nothing
regresses. The two lee rows are the corpus's slowest and end up *faster* than the
frontier: 0.907 s → 0.822 / 0.838 s.

**Stage 13's total value is 0.19 bips.** It changes 4 of 300 rows — `mpbp_15`
−0.45 pp, `maxcsp-ehi-85-297-71` −0.13, `syn40hfsg` −0.08,
`kall_circlesrectangles_c6r39` −0.03 — and costs 0.13–0.19 s on a dozen rows.
Every one of the four is at n < 10_000, so `PEO_ALT_MAX_N = 10_000` keeps all of
them and reclaims the rest.

## Why it won

Three separable pieces, and the order of the measurements matters:

- The **x-set envelope alone was worth nothing on the lee rows** (one row moved,
  `pooling_dt3`, and that came from the metric envelope). The winning passes are
  metric walks on the lifted core, so the set has to be paired with a metric
  envelope that reaches cores just above 12k and with `METRIC_TOP_CORES > 1` —
  the x9 core the lee rows win on is not the lowest-AMD core, so top-1 never
  evaluates it. This is why 0145's follow-up list ("x-sets at further caps
  untested") would not have found it either.
- `METRIC_TOP_CORES = 3` is the expensive half (+0.12 to +0.18 s on
  `pooling_sppc3pq`, `nuclear104`, `lee4_06`, `mpbp_48`) — and those are exactly
  the rows carrying 0.09–0.13 s of stage 13. The two changes are almost
  perfectly complementary on the critical path, which is why the combination is
  both better and faster rather than a trade.
- Stage 13's recorded wins (`mpbp_34` −0.19, `mpbp_35` −0.08, `arki0013` −0.05,
  `gabriel09` −0.03, from 0142-era measurement) **no longer reproduce**: with the
  stage off those rows are bit-identical. Later stages (MINL, transplant, window
  DP) evidently now reach the same completions by other routes. A stage's value
  decays as the pipeline around it grows, and nothing re-prices stages.

## Caveats

- Hidden pending. Local worst 0.838 s against the frontier's 0.907 s on the same
  box is the margin argument; this box runs ~1.13× faster than the box 0145 was
  measured on (their v11 worst 1.026 s), so quote ratios to the frontier, never
  absolute seconds, when comparing against another solver's page.
- `PEO_ALT_MAX_N = 10_000` is fitted to where stage 13's four surviving dev wins
  sit. The structural half of the argument is sound (cost scales with
  n + nnz + lnnz per round per seed, the wins do not), but a hidden row above 10k
  that stage 13 would have helped loses ~0.19-bip-class value. Given the stage
  costs 0.10 s on the rows nearest the cap, that is the right side of the trade.

## Follow-ups

- Price the REST of the pipeline the same way. `9.reduce` (13.24 s corpus-wide,
  29 apparent wins) and `1.portfolio` (30.36 s) have never been audited for
  score-per-millisecond, and the same `best_flops`-staleness trap applies to any
  stage after 11 — use final-ratio A/B, not phase marks.
- With 0.07 s of margin now in hand on the worst row, v5's remaining cut pieces
  are affordable again: METIS top-2 on cores, and the x-sets at nnz ≥ 300k.
- 0145's own follow-up (a subtree round on an accepted lift) is still open, but
  note there is already a terminal deep subtree pass right after point 4b
  (`mod.rs`, gate n ≤ 80_000 / nnz ≤ 250_000), so an accepted lift is not
  unpolished — only the stage-4 chain is skipped.

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md), [amd](../techniques/amd.md)
- Related: [0145](0145-second-colour-class-metric-cores.md) (the timing cut this
  refunds), [0143](0143-independent-set-first-lift.md) (the lift itself),
  [0142](0142-certified-reuse-window-dp.md) (stage 13's origin)
