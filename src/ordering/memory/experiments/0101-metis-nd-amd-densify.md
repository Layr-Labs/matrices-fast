# 0101 — METIS nd_to_amd densify under n<10k + max EXTRA_METRICS densify

**Date:** 2026-09-07 ~19:48 PT. **Base tip:** `c6b0311` / `2e6f2ee` (hidden **0.850463**; local **0.806243**).
**Result:** local **0.805853** (−3.90 bip). gt_10k 0.7150 identical. 1k_10k 0.8444→**0.8431**. nuclear25a 0.635→**0.561**.
**Timing:** probe running.

## Package
1. Max unused EXTRA_METRICS under n<10k (14 specs × α{10,5,2.5,1}) + medium exact +2 tickets.
2. METIS nd_to_amd_switch {50,150,300,800} + max_imbalance 0.15 under n<10k only (part_extra2 gate).
3. Tip-strict MINL (`!core_path_improved`); tip MinFill 3k/12k; PEO_OVERSIZE 2.5M.

## Ablations
| pkg | score | note |
|-----|------:|------|
| densify only | 0.806143 | −1.0 bip |
| densify+MinFill8k+METIS+lot | 0.805888 | −3.55 but worst 1.375s |
| densify+lotteries | 0.806183 | lotteries regress |
| densify+MinFill nnz18k | 0.806143 | MinFill ≠ nuclear25a |
| **densify+METIS** | **0.805853** | nuclear25a confirmed |

## Tip bans respected
nested K3→6; AMF α0.5/2.5 n<10k lottery; DegDivNvDegme@5; thin/MCS/near-cap.
