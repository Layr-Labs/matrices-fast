# 0076 — Exact randomized LNS on the K = 3 residual core (core-LNS)

- **Date:** 2026-09-06
- **Base:** `a07cc9c` (submission `3bd5c872`, hidden **0.861203**; dev **0.831772** on this box)
- **Score:** dev 0.831772 → **0.831264** (−6.1 bips); buckets lt_1k 0.889764 → **0.888958**, `1k_10k` 0.864765 → **0.862724**, gt_10k 0.764398 unchanged; fill 0.938803 → 0.938683
- **Status:** **WIN locally** — full trusted sandboxed 300-matrix run OK (`results.tsv` 1788686321), 66 active tests pass; submitted for hidden validation

## Hypothesis

The K = 3 residual core of `core_lift::reduce` is the exact fill graph after eliminating every
live-degree-≤3 vertex: a smaller, denser instance of the same problem with an exactly split
objective (`Σ c_j² = prefix_flops + flops(core)`). The full-graph exact LNS (`rgreedy::search`,
350–400M ops on n ≤ 1000, 150–250M on the medium gate) walks plateaus of the FULL graph's
elimination game; the core's landscape is a different one, and the exact-ranked AMF/AMD argmin on
the core seeds a basin the full-graph pipeline never visits. Because bitset words scale with the
core size, a 60M-op walk on a ≤ 1000-node core costs ≤ 0.04 s wherever it runs.

## What changed

`mod.rs` only:

- consts `CORE_LNS_MIN_N = 8`, `CORE_LNS_MAX_N = 1_000`, `CORE_LNS_MAX_NNZ = 60_000`,
  `CORE_LNS_STREAMS = [(30M, 0x5EED_0001), (30M, 0x5EED_0002)]`, `CORE_LNS_RECURSE_MARGIN = (2, 1)`.
- `refine_core`: after the deep subtree pass and before the simplicial / five- / four-pivot
  cleanup, run the two chained `rgreedy::search` streams on the core from the refined incumbent;
  each is admitted only on a strict decrease of `flops_of(core_pat)`. Applies to every
  `refine_core` call site (K = 3 pass, winning extra-depth cores, the medium terminal-core winner).
- K = 3 `order_core`: the 11/10 margin that decides whether a raw core splice is worth polishing
  becomes 2/1 when the core is LNS-eligible (cheap). Measured reason: two movers start at 1.18 ×
  and 1.375 × the incumbent and finish below it.

Deterministic (fixed seeds, op budgets, no wall-clock), no threads, structural gates only.

## Measurement before shipping (`probe_core_families`, `probe_core_lns_design`; 2-core pin)

Increment over the finished incumbent, rows n < 10k with 8 ≤ cn ≤ 1000, core_nnz ≤ 60k (150 rows):

| variant | dev delta | movers | Σ secs | max secs | half0 / half1 / drop-1 |
|---|---|---|---|---|---|
| 2 × 30M chained | **−4.85 bips** | 8 | 4.69 | 0.040 | −7.46 / −1.90 / −3.42 |
| 2 × 50M chained | −4.36 | 8 | 7.74 | 0.060 | −8.38 / +0.00 / −2.93 |
| 1 × 100M | −5.31 | 7 | 7.75 | 0.059 | −8.37 / −1.89 / −3.44 |
| 30M + 30M + 100M | −5.15 | 10 | 12.43 | 0.102 | −7.97 / −1.97 / −3.71 |
| 2 × 30M then core descents | −5.03 | 9 | +0.19 | +0.014 | −7.65 / −2.09 / −3.61 |

Movers: pooling_adhya4tp 0.8595 → 0.8048, batchs201210m 0.9154 → 0.8720, wastewater05m1
0.7274 → 0.6964, edgecross10-090 0.8420 → 0.8181, st_e31 0.9491 → 0.9418, blend718, syn15m04m,
qspp_0_11_0_1_10_1. Negative controls measured at the same time: relabelled core MinFill (8 seeds,
cn ≤ 300 / 600): 0 / 1 movers for 2–4 s of corpus time — retired; the medium-band ticket portfolio
on the K = 5/4/2/6 cores: −0.74 bips, 2 movers; tickets or LNS on n ≥ 10k cores: 0 movers.

## Result

Full trusted run (`bash scripts/local-candidate-build.sh && cargo run --release --offline --locked`):
300 matrices OK, score **0.831772 → 0.831264** (−0.000508, −6.1 bips), fill 0.938803 → 0.938683.
**14 rows better / 0 worse / 286 identical**:

| matrix | bucket | n | nnz | before | after |
|---|---|---|---|---|---|
| pooling_adhya4tp | lt_1k | 170 | 936 | 0.8595 | **0.8044** |
| wastewater05m1 | lt_1k | 98 | 536 | 0.7274 | **0.6930** |
| batchs201210m | 1k_10k | 5188 | 17910 | 0.9154 | **0.8847** |
| edgecross10-090 | 1k_10k | 1053 | 5204 | 0.8420 | **0.8181** |
| rsyn0820m04m | 1k_10k | 6028 | 17372 | 0.8357 | 0.8206 |
| rsyn0815m04m | 1k_10k | 5456 | 15756 | 0.8522 | 0.8391 |
| st_e31 | lt_1k | 301 | 744 | 0.9491 | 0.9389 |
| rsyn0830m04m | 1k_10k | 7252 | 20752 | 0.8226 | 0.8153 |
| blend718 | 1k_10k | 1386 | 4310 | 0.8923 | 0.8863 |
| syn15m04m | 1k_10k | 1908 | 5288 | 0.9663 | 0.9645 |
| qspp_0_11_0_1_10_1 | lt_1k | 341 | 44264 | 0.9926 | 0.9913 |
| graphpart_3g-0244-0244 | lt_1k | 128 | 672 | 0.9719 | 0.9709 |
| rsyn0805m03m | 1k_10k | 2868 | 8270 | 0.9045 | 0.9035 |
| hydroenergy2 | 1k_10k | 2092 | 6236 | 0.8146 | 0.8145 |

Production found six movers the probe did not (the rsyn family, graphpart, hydroenergy2): inside
`refine_core` the walk starts from the subtree-refined core (not the raw argmin), and the widened
margin lets 48 more rows enter `refine_core` at all.

Timing (`probe_timing_and_score`, 2-core pin, same box): clean 2-core pin, candidate alone: worst **1.185 s** (`multiplants_stg1b`, +0.030 s) vs frontier 1.171 s (`crudeoil_lee4_10`, unchanged at 1.183 s); mean row delta +0.021 s, 42 rows faster; largest row increase +0.110 s (`rsyn0810m04m` 0.839 → 0.949 s, a row the 2/1 margin newly admits to `refine_core`); no row above 1.19 s.

## Why it won / lost

The gain is a basin change on a different exact instance, not a better heuristic: every mover is a
row whose K = 3 core is a few hundred vertices, where 60M ops of exact plateau walking is an enormous
search relative to the core (30 ms), and where the full-graph LNS had already converged. The
objective split makes each core win exact, and the strict `<` admission keeps the AMD floor. It
did not move gt_10k: large rows with a ≤ 1000-vertex core are rare (8 on dev) and already tied or
optimal there.

## Follow-ups

- LNS on the extra-depth cores (K = 5/4/2/6) when they are LNS-eligible but not yet winners
  (today only winners are refined): up to four more 60M walks per small row — needs a time offset.
- Budget shape: the 30M/50M/100M curve is flat within noise; a second PRNG seed pair as a
  well-below-anchor extra ticket is the cheapest untested lever.

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md)
- [0062](0062-reduce-then-amf-terminal.md), [0065](0065-core-recursion-on-the-k3-core.md),
  [0075](0075-residual-core-minfill.md)
