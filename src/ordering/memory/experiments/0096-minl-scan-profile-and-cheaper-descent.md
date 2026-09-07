# 0096 — MINL scan profiled: cheaper descent, exact re-testing, kept partial rounds, one refinement round

**Date:** 2026-09-07. **Base:** `7f5a20d` (dukemawex `82d8d6e`, hidden 0.85171 = my `d74ab4b` + four relabelled-metric lottery families; dev 0.808139 on the 2-vCPU pod, worst 1.561 s).
**Result:** dev **0.807259** (−10.9 bips), 30 rows better / 2 worse; worst same-box `order()` 1.634 s (tip 1.561 s). Harness 300/300 rows OK, score 0.8073 (0.8896 / 0.8468 / 0.7158, fill 0.9307), 4 min 20 s on the pod.

## Profile (test-only counters in `minl.rs`, 14 rows, budget 120M to see the whole scan)

| row | n | fill edges | tests | first removal at test | removed | rounds | budget left |
|---|---|---|---|---|---|---|---|
| crudeoil_lee4_10 | 17809 | 634834 | 112367 | 1 | (dropped) | 0 | exhausted in round 1 |
| crudeoil_lee4_09 | 15904 | 506610 | 107983 | 1 | (dropped) | 0 | exhausted |
| mpbp_48 | 28368 | 145057 | 123041 | 27491 | (dropped) | 0 | exhausted |
| gabriel09 | 21688 | 166691 | 83637 | 4664 | (dropped) | 0 | exhausted |
| transswitch2736spr | 69651 | 205576 | 375851 | 1 | 1145 | 1 | exhausted |
| procurement1large | 14416 | 74036 | 118668 | 24394 | 31 | 1 | exhausted |
| chimera_selby-c16-01 | 2031 | 43811 | 190452 | 1915 | 184 | 4 | exhausted |
| mpbp_35 | 11120 | 32373 | 288219 | 199 | 440 | 9 | 50M left |
| mpbp_34 | 11556 | 31826 | 158284 | 13644 | 254 | 5 | 83M left |
| rsyn0820m04m | 6028 | 7457 | 14866 | 3709 | 48 | 2 | 117M left |
| ringpack_30_2 | 17999 | 29763 | 6943 | never | 0 | 0 | exhausted (clique checks 117M: hub degrees) |

Ops split roughly 55 % common-neighbourhood merges, 45 % clique checks; the
common-neighbour cap (2000) never binds (500 and 4000 change nothing), rounds
beyond 8 change nothing. The only binding resource is the round-1 budget.

## Changes

1. **Kept partial rounds** (bug): the budget break left the round before
   `removed_total += removed_this`, so every big-fill row's partial descent
   was discarded as "nothing removable". On dev the kept partial descents move fourteen rows against the cheaper scan alone (`crudeoil_pooling_dt2` −2.1 bips, `mpbp_48` −0.5, `crudeoil_lee4_06` −0.4, `gabriel09` −0.2, `arki0013`, `pooling_sppc1pq`, `nuclear10a`, `crudeoil_lee2_06`, …), none loses.
2. **Grouped-stamp scan**: coarse cheapest-first (log2 buckets of the degree
   sum), grouped by the lower endpoint inside a bucket; `N(u)` stamped once per
   group, intersect walks `N(v)` only, clique check counts stamped members per
   adjacency poorest-first with an early exit. ~½ the ops per edge → the same
   40M budget reaches ~2× the edges: pinene200 0.8465 → 0.8396,
   transswitch2736spr 0.9238 → 0.9179, procurement1large 0.5353 → 0.5335
   (−1.65 bips, 5 rows, 0 losses; MINL stage time 3.2 → 3.9 s corpus-wide,
   worst row +0.02 s).
3. **Dirty-endpoint re-test** (exact): later rounds re-test only edges with
   an endpoint whose adjacency changed; −20 % stage time on mpbp/chp rows,
   three small extra wins.
4. **One `subtree_refine` round on a strict MINL win** (32 blocks, 8M, n ≤ 35k):
   −0.75 bips, 9 rows, 0 losses, ≤ 0.08 s per winner. 32M: −0.6 more for
   +0.23 s on the winners — not taken.

5. **Three more relabelled-metric lottery families** (`SqPure@10`, `DegP125@1`,
   `DegDivNvSqrtWf@10`, same 120k budget and 1–6 pass clamp): −5.6 bips,
   7 wins / 2 losses (rsyn0820/0830/0840m04m, crudeoil_lee4_06), +0.03 s on
   the slowest row. dukemawex's four families translated −1.0 dev → −9.5
   hidden bips, so breadth in this loop is the best-translating lever seen.
   Not taken: budget 240k (−3.5 bips on one row, +0.14 s near the cap).
6. **Refinement round only after a COMPLETED descent**: a budget-cut descent
   sits on the big-fill rows (crudeoil_lee4_*, mpbp_48, gabriel09), where the
   round costs ~0.1 s for a 0.01 % win. Measured: +0.68 dev bips given back (ten losses of ≤ 0.27 bip, crudeoil_lee4_06 the largest) for −0.08 to −0.09 s on each of the four slowest rows (worst 1.692 → 1.609 s). Taken: the cap is a lottery and those rows are the ticket.

## Negative results (full 300-row pod runs, each vs its parent)

- Early MINL (before search/subtree): +0.5 bips, 20/23 rows, +2 s. Mid MINL
  (after cleanup): −0.56 bips with 2 wins / 3 losses, +3 s, worst +0.09 s.
- Chained terminal MINL (≤ 3 passes): −0.13 bips, 3 rows.
- MINL from ≤ 2 runner-up seeds (nnz < 60k): 0 rows, +1.5 s.
- AMF realization of M: 0 rows. Rounds 16 / common 4000 / common 500: 0 rows.
- Budget 120M (before the cheaper scan): −1.0 bips, 5 rows, worst row +0.1 s.
- +3 small-tier exact-search streams: −0.02 bips, +6.4 s. +2 medium-tier
  streams: −1.9 bips, 6/3 rows, +2.6 s (batchs201210m 0.915 → 0.869 flips
  under ANY perturbation — a basin, not a mechanism).

## Movers vs the base

`crudeoil_lee4_06` 0.6848 → 0.6620, `crudeoil_pooling_dt2` 0.7106 → 0.6943, `rsyn0830m04m` 0.8226 → 0.7920, `rsyn0840m04m` 0.8583 → 0.8305, `rsyn0820m04m` 0.8189 → 0.7926, `pinene200` 0.8465 → 0.8396, `transswitch2736spr` 0.9238 → 0.9179, `mpbp_48` 0.4811 → 0.4786, `procurement1large` 0.5353 → 0.5335, `chimera_mgw-c16-2031-01` 0.7997 → 0.7920, `gabriel09` 0.9378 → 0.9357, `crudeoil_lee4_09` 0.6966 → 0.6951, `chimera_selby-c16-01` 0.6884 → 0.6839, `arki0013`, `mpbp_34`, `crudeoil_lee1_07`, `crudeoil_lee2_06`, `mpbp_35`, then twelve rows under 0.06 bip; the two losses are `ndcc13` and `crudeoil_li05` at +0.00 and +0.01 bip
