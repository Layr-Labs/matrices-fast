# 0009 — Robust AMD Nonzero Envelope Expansion (130k → 600k)

**Date:** 2026-09-02
**Score:** 0.870261 → **0.868096** (−0.002165, −24.9 bips). Buckets: lt_1k 0.9064,
1k_10k 0.8963, gt_10k **0.8237→0.8182** (−0.0055). Fill tiebreak 0.960309→0.958983.
**Status:** WIN, shipped.

## Hypothesis

In `src/ordering/mod.rs`, five non-aggressive and dense-detection disabled AMD variants (`amd_robust`, `amd_robust5`, `amd_robust2`, `amd_nodense`, `amd_nodense_agg`) provide distinct minimum-degree tie-breaking orders that are pure upside over the baseline.

Historically, `ROBUST_MAX_NNZ` was capped at `130_000` under the assumption that high-nonzero matrices might approach the 2 s time cap. However:
1. AMD execution time scales almost strictly with nonzeros, and at $nnz \le 600,000$, an AMD pass requires only tens of milliseconds.
2. High-dimensional systems in `gt_10k` ($n \ge 10,000$) often have $nnz$ in the 130k–600k range (e.g., `cont6-qq` $n=120,395, nnz=557,994$ and `methanol400` $n=23,999, nnz=151,728$).
3. Because these matrices were gated out of robust AMD, they missed dense-row deferral optimizations and non-aggressive supervariable absorption.

By raising `ROBUST_MAX_NNZ` from `130_000` to `600_000`, we safely expose matrices with up to 600k nonzeros to these 5 AMD passes. Matrices with $nnz > 600,000$ (such as `faclay75` $nnz = 1.38M$) remain safely excluded, preventing any worst-case latency inflation.

## Results & Verification

- Score on dev corpus: **0.868096** (fill tiebreak **0.958983**).
- `gt_10k` bucket dropped from 0.8237 to **0.8182** (−55 basis points!).
- Key matrix wins:
  - `methanol400` ($n = 23,999, nnz = 151,728$): flop ratio plummeted from 0.965 to **0.737** (−23.6% flops reduction!).
  - `cont6-qq` ($n = 120,395, nnz = 557,994$): flop ratio reduced from 0.9145 to **0.8899** (−2.7% reduction).
- Worst-case runtime: **0.823 s**, completely within the 2.000 s limit.
