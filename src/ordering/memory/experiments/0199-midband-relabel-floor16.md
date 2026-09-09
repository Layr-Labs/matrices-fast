# 0199 — mid-band non-hub relabel floor 12→16

- **Date:** 2026-09-09
- **Base:** clean tip `74b6ccd` (local 0.793834). 0198 HEAVY_METRIC slot[1] α1 reverted. Closed: mid-K4, dens≤4, HEAVY_METRIC last-ticket and slot[1] α flats, depth-7, 4th-core/SqDiv, DegP125 n>16k, extrarelbl, pairdesc widen, KaHIP Eco chimera, Scotch, ND/Sloan α16 band. Exact-search seeds / metric_k / SqDiv list tip; keep 475a hunks; no linson adaptive windows; do not raise gt_10k relabel 500k/36→600k/40. Do not chase 1393c71.
- **Score:** 0.793834 → **0.793841** (+0.000007)
- **Status:** miss / floor reverted to 12. Not submitted. Model Grok 4.6, harness Grok Bot. Benchmark `8c3e7051-530a-4aee-88df-a426e6e78151`.

## Hypothesis

In `relabel_restarts_tuned`, the mid-band non-hub branch (`nnz <= 150_000 && max_deg * 50 <= n`) floors restarts at `base_r.max(12)`. Raising the floor to 16 spends four more AMD-relabel restarts on that band only — same gate, same budget/cap/AMF — aiming for >1 strict flop movers with 0 worse vs tip 0.793834, without lifting uncapped worst (~1.65s; prefer ≲1.14s).

## What changed

`src/ordering/mod.rs` only (`relabel_restarts_tuned` mid-band non-hub floor), then restored after the local run:

```
} else if nnz <= 150_000 && max_deg * 50 <= n {
    base_r.max(16) // Mid-band non-hub floor  (was 12; reverted)
```

Hub guard, low-nnz, gt_10k / giant floors, `RELABEL_BUDGET`, `relabel_budget_and_cap`, and AMF relabel untouched. `probe.rs` untouched. Reverted; floor is 12 again.

## Result

Yukon local **0.793841** vs tip **0.793834** (`/tmp/yukon-run-0199.log`).
Buckets: lt_1k 0.8875 / 1k_10k 0.8398→0.839825 / gt_10k 0.6891. Fill 0.9258→0.925769.

Flops-exact vs tip `/tmp/yukon-run-tip-74b6ccd.log`: **0 better / 1 worse / 299 same**.
Worse: `mpbp_15` (n=9858, nnz=31692) 1198160 → 1201770. Mid-band class movers: 0 better / 1 worse. Keep bar missed (need beat tip, >1 class mover, 0 worse).

## Why it won / lost

Extra mid-band restarts did not find a better AMD-relabel incumbent on any row in the gate; the one class mover (`mpbp_15`) landed worse, lifting 1k_10k enough to push the weighted score +7e-6. Floor 16 is not free even when budget/cap already allow ≥12.

## Follow-ups

- Do not retry this exact mid-band floor 12→16.
- Tip ordering restored (`base_r.max(12)`).
- Leave hub / low-nnz / gt_10k / giant floors and `relabel_budget_and_cap` alone; do not chase 1393c71.
