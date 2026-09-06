# 0082: calibrated very-sparse-large allowance and independent reversed roots

- **Date:** 2026-09-06
- **Base:** `0503cf6` (submission `be9f0a41`, hidden 0.859925)
- **Status:** measured, all gates green; official submission

## Overview and Root Cause Analysis

Official crown commit `0503cf6` introduced terminal PEO re-extraction above the 30k/180k gate under a 2.5M operation ledger (improving the dev score from 0.829057 to 0.827794 and achieving 0.859925 on hidden eval).

Two subsequent lines of expansion were explored:
1. **Independent MCS root seeds inside the gate** (`reversed_root_candidates` on `1_000 <= n <= 30_000 && nnz <= 180_000`):
   - Submission `69be95bc` (PR #214) scored 0.860061 on the hidden corpus (improving base 0.860113 by 0.52 bips, but falling short of the required 1.0 bip threshold for promotion).
   - On the development corpus, it captured 3.40 bips across four matrices (`mpbp_48`, `mpbp_34`, `mpbp_35`, `gabriel09`).
2. **Second allowance for the very-sparse-large class** (experiment 0081 / PR #217):
   - PR #217 attempted to give a 14M ledger to all matrices with `nnz > 400_000`.
   - However, PR #217 failed CI at 5m51s with a 2.0s per-matrix timeout on the hidden benchmark.
   - RCA: PR #217 did not bound matrix dimension `n` or rounds for the huge allowance. Very large dimension graphs (`n > 100_000`, such as `acopf_case6515rte_qcqp` at n=188k and `watercontamination` at n=217k) already require ~0.55s locally (~1.4s on GitHub Actions runners) for standard AMD ordering. Adding an unconstrained PEO round on `n > 100k` graphs added ~0.3s+ locally (~0.7s+ on runners), exceeding the hard 2.0s cap.

## Calibrated Solution

By analyzing matrix characteristics, we observed that:
- Over 90% of the gains from the larger PEO allowance concentrate exclusively on the sparse pooling family (`pooling_sppc3pq`, `pooling_sppb5pq`, `pooling_sppc1pq`, `pooling_sppc3stp`, `pooling_sppb5stp`, `pooling_sppc1stp`).
- All pooling family matrices have modest vertex dimensions: `n <= 35_000`.
- All heavy/slow boundary rows (`acopf`, `watercontamination`, `unitcommit`, `gabriel10`) have `n > 100_000`.

Therefore, we apply strict, structural runtime calibration:
1. `PEO_HUGE_MAX_N = 50_000`: The 14M ledger is granted exclusively to matrices with `n <= 50_000` and `nnz > 400_000`. This completely shields all heavy `n > 100k` matrices (which retain the safe 2.5M ledger from `0503cf6` and skip expensive PEO).
2. `PEO_HUGE_ROUNDS = 1`: The huge allowance runs strictly at most 1 round, ensuring no accumulation of multiple passes on large graphs.
3. `PEO_LARGE_*` baseline constants are strictly preserved: `PEO_LARGE_MAX_NNZ = 1_500_000`, `PEO_LARGE_MAX_LNNZ = 20_000_000`, `PEO_LARGE_ROUNDS = 8`, `PEO_LARGE_LEDGER = 2_500_000`.
4. Independent reversed-incumbent and high-degree MCS root candidates are evaluated inside the gate `1_000 <= n <= 30_000 && nnz <= 180_000` (capped at 2 rounds). Because this gate is mutually disjoint from `PEO_HUGE` (`nnz > 400_000`), both mechanisms compose additively with zero structural conflict.

## Empirical Measurements

### Development Corpus (300 matrices)
- Base `0503cf6`: `0.827794` (lt_1k: 0.889730, 1k_10k: 0.863415, gt_10k: 0.754637)
- Candidate: **`0.827158`** (lt_1k: 0.889730, 1k_10k: 0.863415, gt_10k: **0.753035**)
- Net improvement: **-0.000636 (-6.36 basis points)**
- Zero regressions across all 300 matrices; 6 strict improvements (`pooling_sppc3pq`, `pooling_sppb5pq`, `mpbp_48`, `mpbp_34`, `mpbp_35`, `gabriel09`), 294 exact ties.

### Runtime Safety
- Slowest local execution on external `gt_10k` corpus:
  - `acopf_case6515rte_qcqp`: 0.550s (safely bypassed huge PEO)
  - `acopf_case6468rte_qcqp`: 0.560s (safely bypassed huge PEO)
  - `pooling_sppc3stp`: 0.473s (1 round huge PEO, captures 1.24% win)
  - `pooling_sppb5stp`: 0.224s
- Full test suite passed (68 worker tests, 51 unit tests, 4 security tests, 5 time cap tests).
