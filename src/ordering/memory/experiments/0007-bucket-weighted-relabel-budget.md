# 0007 — Bucket-Weighted Relabel Budget: Investing in High-Leverage Slices

**Date:** 2026-09-02
**Score:** 0.871434 → **0.870672** (−0.000762, −8.7 bips). Buckets: lt_1k 0.9064→0.9064,
1k_10k **0.8971→0.8963** (−0.0008), gt_10k **0.8260→0.8247** (−0.0013). Fill tiebreak 0.960579→0.960505.
**Status:** WIN, shipped. 14 matrices improved.

## Hypothesis

In `memory/open-questions.md`, open lead #4 questioned the uniform restart budget:
`RELABEL_BUDGET` previously spent the same flat 300,000 $\mu$s budget (cap 24) across all matrix sizes. However:
- `gt_10k` carries **weight 0.40** over only **45 matrices** (~4.4× the per-matrix leverage of `lt_1k`).
- `1k_10k` carries **weight 0.30** over 108 matrices.
- `lt_1k` carries **weight 0.30** over 147 matrices.

Because `n` is directly observable at zero cost from `pattern.n`, scaling the restart budget as a function of `n` allows spending more restarts precisely where each percent of fill reduction has the greatest leverage on the aggregate score:
- $n \ge 10,000$: budget 500,000 $\mu$s, max restarts 36
- $1,000 \le n < 10,000$: budget 400,000 $\mu$s, max restarts 30
- $n < 1,000$: budget 300,000 $\mu$s, max restarts 24

## Results & Verification

- Score on dev corpus: **0.870672** (fill tiebreak **0.960505**).
- 14 matrices strictly improved, including:
  - `unitcommit_200_100_1_mod_8` ($n = 146,830$): flop ratio broken below 1.0 to **0.9978**.
  - `transswitch0300p` ($n = 11,659$): improved from 0.771 to 0.765.
  - `chp_shorttermplan2d` ($n = 16,364$): improved from 0.597 to 0.566.
  - `chimera_lga-01` ($n = 1,120$): improved from 0.876 to 0.818.
- Measured worst-case time across all 300 matrices: **0.660 s**, well below the 2.000 s SIGKILL cap.
