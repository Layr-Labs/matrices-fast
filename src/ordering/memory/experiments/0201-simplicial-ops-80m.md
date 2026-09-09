# 0201 — SIMPLICIAL_PROMOTION_OPS_BUDGET 64M→80M

- **Date:** 2026-09-09
- **Base:** 0200 package on tip `74b6ccd` (FINAL_FIVE_OPS stays **160M**; local package 0.793825). Scoreboard-locked. Closed: 0177 simplicial MAX_N 6k→10k, mid-K4, dens≤4, HEAVY_METRIC redraws, DEPTHS 6→7, 4th-core, DegP125 n>16k, midband floor16, extrarelbl, pair sweeps, 488a stack, KaHIP/Scotch, ND/Sloan α16.
- **Score:** 0.793825 → **0.793825** (0.00)
- **Status:** MISS / OPS reverted to 64M; FINAL_FIVE_OPS **160M** left intact. Not submitted (bar missed; e188522 failed). Model Grok 4.6, harness Grok Bot. Benchmark `8c3e7051-530a-4aee-88df-a426e6e78151`.

## Hypothesis

Terminal + post-terminal `simplicial_promotion` shares `SIMPLICIAL_PROMOTION_OPS_BUDGET = 64_000_000` under the shipped gate n≤6k ∩ nnz≤100k ∩ dens≤24. Raising only the op budget to 80M (MAX_N/MAX_NNZ/density unchanged; no 0177 widen) may find additional strict flop improvements on simplicial-gate rows beyond the 0200 FINAL_FIVE_OPS package.

Keep bar (all required vs **0200 package** 0.793825): beat 0.793825; >1 simplicial-gate row moves; 0 worse; uncapped worst ≤1.442 s. Submit only if e188522 promotes (rebase+restack); if e188522 fails/rejects first, stop before submit.

## What changed

`src/ordering/mod.rs` only — then restored after the local run:

```
const SIMPLICIAL_PROMOTION_OPS_BUDGET: i64 = 80_000_000; // was 64_000_000; reverted
```

MAX_N=6_000, MAX_NNZ=100_000, density≤24 unchanged. `FINAL_FIVE_OPS` stayed 160M throughout. `FINAL_SIMP_OPS` stayed 64M (not in scope). `probe.rs` untouched.

## Result

Yukon local **0.793825** (= package) (`/tmp/yukon-run-0201.log`).
Buckets: lt_1k 0.8875 / 1k_10k 0.8398 / gt_10k 0.6891. Fill 0.9258.

Flops-exact vs 0200 `/tmp/yukon-run-0200.log`: **0 better / 0 worse / 300 same**.
Simplicial-gate movers: **0 / 0**. Tip-flat on every row.

Uncapped probe: **not run** (score/movers bar already missed).

e188522 (0200 FINAL_FIVE_OPS 160M submit): **failed** (n/a) — no rebase/restack; no 0201 submit.

Keep bar missed. Reverted OPS to 64M; left 0200 FINAL_FIVE_OPS 160M intact.

## Why it won / lost

Extra 16M ops on the existing simplicial gate found no new strict improvement beyond the 0200 package — every matrix bit-identical. The 64M budget was already saturating the profitable simplicial neighborhoods on this corpus; a pure OPS deepen without gate widen is a null here (same shape as several HEAVY_METRIC α redraws).

## Follow-ups

- Do not retry SIMPLICIAL_PROMOTION_OPS_BUDGET 64→80 (or nearby deepenings) alone.
- Do not reopen MAX_N 6k→10k (0177 closed).
- e188522 failed: Scoreboard should reassess 0200 FINAL_FIVE_OPS 160M keep-vs-revert before the next submit; 0201 leaves that package in the tree as found.
