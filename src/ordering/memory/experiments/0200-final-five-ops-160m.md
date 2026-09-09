# 0200 — FINAL_FIVE_OPS 128M→160M

- **Date:** 2026-09-09
- **Base:** clean tip `74b6ccd` (local 0.793834). Closed: mid-K4, dens≤4, HEAVY_METRIC redraws, depth-7, 4th-core, DegP125 n>16k, midband relabel floor16, extrarelbl, pairdesc widen, KaHIP Eco chimera, Scotch, ND/Sloan α16. Leave exact-search seeds and relabel_restarts_tuned alone. Keep 475a hunks (pairdesc EXT nnz≤60k etc). No FINAL_FIVE_OPS >192M. Do not raise FINAL_FIVE_MAX_N or MAX_NNZ.
- **Score:** 0.793834 → **0.793825** (−0.000009)
- **Status:** KEEP / submitted. Model Grok 4.6, harness Grok Bot. Benchmark `8c3e7051-530a-4aee-88df-a426e6e78151`.

## Hypothesis

Terminal `adjacent_five_descent` on the shipped incumbent is capped at `FINAL_FIVE_OPS = 128_000_000` for n≤12k ∩ nnz≤80k. Raising the op budget to 160M (still under the 192M ceiling; MAX_N/MAX_NNZ unchanged) may find additional strict flop improvements on final-five class rows without widening the gate or stacking mid_force/REDUCE/pairdesc.

Keep bar: beat tip 0.793834, >1 final-five class row moves, 0 worse, uncapped worst does not rise vs tip (~1.65s; prefer ≲1.14s).

## What changed

`src/ordering/mod.rs` only — the terminal five-descent budget constant:

```
const FINAL_FIVE_MAX_N: usize = 12_000;
const FINAL_FIVE_MAX_NNZ: usize = 80_000;
const FINAL_FIVE_OPS: i64 = 160_000_000; // was 128_000_000
```

No n/nnz widen, no OPS>192M, no mid_force/REDUCE/pairdesc stack. `probe.rs` untouched.

## Result

Yukon local **0.793825** vs tip **0.793834** (`/tmp/yukon-run-0200.log`).
Buckets: lt_1k 0.8875 / 1k_10k 0.8398→0.839787 / gt_10k 0.6891→0.689117. Fill 0.9258→0.925767.

Flops-exact vs tip `/tmp/yukon-run-tip-74b6ccd.log`: **9 better / 0 worse / 291 same**.
All 9 movers in final-five class (n≤12k ∩ nnz≤80k):
- mpbp_15 −1869, crudeoil_lee2_06 −1195, mpbp_34 −238, mpbp_35 −181,
- chp_partload −88, transswitch0300p −58, powerflow0300p −36, rsyn0840m04m −18, glider400 −13.

Uncapped `probe_timing_and_score`: WORST **1.442 s** (`/tmp/probe-timing-0200.log`) vs tip **1.654 s** (`/tmp/probe-timing-tip-0190ab.log`). Does not rise.

Keep bar met.

## Why it won / lost

Extra 32M ops on the terminal five-descent found strict improvements on nine mid-size rows already inside the existing n/nnz gate; no outside-gate row moved and none regressed. Timing stayed under tip uncapped worst.

## Follow-ups

- Do not raise FINAL_FIVE_MAX_N / MAX_NNZ or OPS above 192M in follow-ups without a fresh timing bar.
- Leave exact-search seeds and relabel_restarts_tuned alone.
