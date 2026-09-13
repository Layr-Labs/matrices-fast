# 0100 — MinFill envelope + METIS switches + orthogonal n<10k lotteries (timing fail)

**Date:** 2026-09-07 ~19:05 PT. **Base:** iter15 densify (0.806143) on tip `c6b0311`.
**Result:** local **0.805888** (−3.55 bip vs tip 0.806243). gt_10k 0.7150 identical. 1k_10k 0.8441→**0.8432**.
**Timing:** WORST order() **1.375 s** (crudeoil_lee1_07) vs failed-submit probes ~1.19 s — **too slow, not submitted**.

## Package (on top of 0099 densify)
1. MinFill `MAX_N/NNZ` 3k/12k → 8k/40k; relabel restarts to n<8k nnz<25k.
2. Orthogonal n<10k lotteries: DegDivNvDegme@{2.5,1}, DegSqrt@10, Ammf/AmindNorm@{10,2.5} (not DegDivNvDegme@5).
3. METIS nd_to_amd {50,150,300,800} + imb 0.15 under n<10k.

## Diagnosis
Slowdown concentrated on mid `1k_10k` rows that entered the expanded MinFill gate (crudeoil_lee1_07 +187 ms, chp_partload +285 ms). faclay75 unchanged ~1.11 s.

## Follow-up
iter17: revert MinFill + METIS densify; keep densify + orthogonal lotteries (timing bisect).
