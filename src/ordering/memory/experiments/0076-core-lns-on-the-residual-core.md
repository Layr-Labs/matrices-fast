# 0076 — Exact randomized LNS on the K = 3 residual core (core-LNS)

- **Date:** 2026-09-06
- **Base:** `a07cc9c` (submission `3bd5c872`, hidden **0.861203**; dev **0.831772** on this box)
- **Score:** dev 0.831772 → **0.831457** (−3.8 bips); buckets lt_1k 0.889764 → **0.8888**, `1k_10k` 0.864765 → 0.8635, gt_10k 0.764398 unchanged
- **Status:** time-neutral shape submitted for hidden validation (full trusted 300-matrix run OK, 66 tests). The earlier ADDITIVE shape (−6.1 bips dev, +0.014 s worst row, up to +0.11 s on 0.8 s rows) **failed hidden validation** as `f118a2c5`

## Hypothesis

The K = 3 residual core of `core_lift::reduce` is the exact fill graph after eliminating every
live-degree-≤3 vertex: a smaller, denser instance of the same problem with an exactly split
objective (`Σ c_j² = prefix_flops + flops(core)`). The full-graph exact LNS (`rgreedy::search`,
350–400M ops on n ≤ 1000, 150–250M on the medium gate) walks plateaus of the FULL graph's
elimination game; the core's landscape is a different one, and the exact-ranked AMF/AMD argmin on
the core seeds a basin the full-graph pipeline never visits. Because bitset words scale with the
core size, a 60M-op walk on a ≤ 1000-node core costs ≤ 0.04 s wherever it runs.

## What changed

`mod.rs` only (+103 / −30 vs `a07cc9c`):

- consts `CORE_LNS_MIN_N = 8`, `CORE_LNS_MAX_N = 1_000`, `CORE_LNS_MAX_NNZ = 60_000`,
  `CORE_LNS_STREAMS = [(30M, 0x5EED_0001), (30M, 0x5EED_0002)]`, `CORE_LNS_TRIM = (1, 1)`,
  `CORE_LNS_POLISH_MARGIN = (2, 1)`; helper `trim_streams`.
- **Reduce once, decide early:** for 50 ≤ n < 10k the terminal block's K = 3 `core_lift::reduce` runs
  before the full-graph LNS section and is reused by the terminal block (`early_core`).
  `lns_eligible` = 8 ≤ cn ≤ 1000 and core_nnz ≤ 60k; `lns_paid` = eligible and the row is in the
  n ≤ 1000 LNS gate or `medium_exact_gate`.
- **Pay first:** on paid rows `trim_streams` removes Σ`CORE_LNS_STREAMS` = 60M ops from the TAIL of
  the row's full-graph LNS stream list (small rows: the fifth 100M stream → 40M; medium `(100M, 50M)`
  → `(90M)`), so no row runs more exact-search ops than the frontier.
- **Walk once:** `refine_core(…, full, lns)`; the K = 3 path calls it with `full = raw ≤ 11/10 ×
  incumbent` (the frontier's chain) or, on paid rows within 2×, `full = false` (LNS-only polish: walk
  + the cheap simplicial / five- / four-pivot cleanup, no core subtree chain). Extra-depth winners and
  the medium terminal-core winner pass `lns = false`.
- Strict `<` on `flops_of(core_pat)` per stream; the splice is admitted through the existing strict
  `<`. Deterministic, no threads, structural gates only.

Closed variant (candidate 1, `f118a2c5`): the walk inside every `refine_core` call plus a 2/1 margin
for the FULL chain on eligible cores — additive; the widened chain cost up to +0.11 s (+13 %) on
0.8–0.9 s medium rows (rsyn0810m04m, rsyn0815m04m, rsyn0805m03m, ndcc13, mpbp_46, oil).

## Measurement before shipping (`probe_core_families`, `probe_core_lns_design`; 2-core pin)

Increment over the finished incumbent, rows n < 10k with 8 ≤ cn ≤ 1000, core_nnz ≤ 60k (150 rows):

| variant | dev delta | movers | Σ secs | max secs | half0 / half1 / drop-1 |
|---|---|---|---|---|---|
| 2 × 30M chained | **−4.85 bips** | 8 | 4.69 | 0.040 | −7.46 / −1.90 / −3.42 |
| 2 × 50M chained | −4.36 | 8 | 7.74 | 0.060 | −8.38 / +0.00 / −2.93 |
| 1 × 100M | −5.31 | 7 | 7.75 | 0.059 | −8.37 / −1.89 / −3.44 |
| 30M + 30M + 100M | −5.15 | 10 | 12.43 | 0.102 | −7.97 / −1.97 / −3.71 |
| 2 × 30M then core descents | −5.03 | 9 | +0.19 | +0.014 | −7.65 / −2.09 / −3.61 |
| 2 × 15M | −2.35 | 3 | 2.35 | 0.021 | −4.60 / +0.00 / — |
| 2 × 10M / 1 × 20M | −0.95 / −0.77 | 2 / 2 | 1.55 | 0.014 | — |

Movers: pooling_adhya4tp 0.8595 → 0.8048, batchs201210m 0.9154 → 0.8720, wastewater05m1
0.7274 → 0.6964, edgecross10-090 0.8420 → 0.8181, st_e31 0.9491 → 0.9418, blend718, syn15m04m,
qspp_0_11_0_1_10_1. Negative controls measured at the same time: relabelled core MinFill (8 seeds,
cn ≤ 300 / 600): 0 / 1 movers for 2–4 s of corpus time — retired; the medium-band ticket portfolio
on the K = 5/4/2/6 cores: −0.74 bips, 2 movers; tickets or LNS on n ≥ 10k cores: 0 movers.

## Result

Time-neutral shape, full trusted run (`results.tsv`, 300 OK): **0.831772 → 0.831457** (−3.8 bips).
**12 better / 5 worse / 283 identical.** Better: pooling_adhya4tp 0.8595 → 0.8044, wastewater05m1
0.7274 → 0.6938, batchs201210m 0.9154 → 0.8847, edgecross10-090 0.8420 → 0.8181, waterund11
0.7284 → 0.7151, st_e31 0.9491 → 0.9389, pooling_adhya4pq, sfacloc2_4_90, korcns, syn15m04m, blend718,
graphpart_3g-0244-0244. Worse (the replaced tail tickets): blend721 0.8573 → 0.8858, netmod_kar1
0.8048 → 0.8174, slay09h, rsyn0830hfsg, multiplants_mtg1b (≤ 0.2 %).

Timing (2-core pin, alone): worst 1.180 s vs frontier 1.171 s (same row, crudeoil_lee4_10, untouched
by the change); mean row **−0.7 ms**; 116 rows faster; every row ≥ 1.0 s within +0.03 s; largest
row increase +0.21 s on rsyn0840m02m (0.46 → 0.67 s: a different LNS incumbent sent the conditional
subtree chain down a longer path to the same flops).

For reference, the additive shape measured 0.831264 (−6.1 bips, 14 / 0 / 286) with worst 1.185 s,
mean +0.021 s, and +0.11 s on its worst-affected row — and failed hidden validation.

## Why it won / lost

The gain is a basin change on a different exact instance, not a better heuristic: every mover is a
row whose K = 3 core is a few hundred vertices, where 60M ops of exact plateau walking is an enormous
search relative to the core (30 ms), and where the full-graph LNS had already converged. The
objective split makes each core win exact, and the strict `<` admission keeps the AMD floor. It
did not move gt_10k: large rows with a ≤ 1000-vertex core are rare (8 on dev) and already tied or
optimal there.

## Follow-ups

- Priced on the additive tree (`probe_core_lns_next`): the walk on the K = 5/4/2/6 cores −0.79 bips
  (7 movers, +0.11 s/row), a second seed pair −0.54, seeding from the plain AMD core ordering −0.42;
  the two K = 3 variants do not survive drop-1. The family is near its ceiling; any further ticket
  needs its own offset.
- Rows with an eligible core but no full-graph LNS (6000 < n < 10k, or nnz > 30k) get nothing; the
  additive tree showed movers there (rsyn0830m04m, rsyn0820m04m) — an offset on those rows (e.g. a
  relabel restart or two) is the cheapest untested trade.

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md)
- [0062](0062-reduce-then-amf-terminal.md), [0065](0065-core-recursion-on-the-k3-core.md),
  [0075](0075-residual-core-minfill.md)
