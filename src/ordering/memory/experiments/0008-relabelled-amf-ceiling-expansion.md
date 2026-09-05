# 0008 — Relabelled-AMF Nonzero Ceiling Expansion (130k → 200k)

**Date:** 2026-09-02
**Score:** 0.870672 → **0.870261** (−0.000411, −4.7 bips). Buckets: lt_1k 0.9064,
1k_10k 0.8963, gt_10k **0.8247→0.8237** (−0.0010). Fill tiebreak 0.960505→0.960309.
**Status:** WIN, shipped.

## Hypothesis

In `memory/open-questions.md`, open lead #3 asked:
*"Is `RELABEL_AMF_MAX_NNZ = 130_000` leaving anything above it? The ceiling is a cost bound, not a measured optimum."*

Analysis of the 130,000 to 200,000 nonzero band revealed high-leverage matrices in the `gt_10k` bucket that were previously excluded from the relabelled AMF multi-start purely due to the conservative 130,000 threshold:
- `crudeoil_lee4_10` ($nnz = 138,441$)
- `ringpack_30_2` ($nnz = 139,457$)
- `methanol400` ($nnz = 175,727$)
- `arki0013` ($nnz = 160,172$)
- `faclay35` ($nnz = 159,158$)

Because the restart count is governed by `budget / nnz`, at $nnz = 150,000$ with `budget = 500,000`, the algorithm executes at most 3 restarts. Each pass takes ~0.03s, adding at most ~0.09s of compute time. Raising the ceiling from 130,000 to 200,000 is thus computationally safe while exposing high-dimensional matrices to randomized minimum-fill pivoting.

## Results & Verification

- Score on dev corpus: **0.870261** (fill tiebreak **0.960309**).
- `gt_10k` bucket dropped from 0.8247 to **0.8237**.
- Key breakthroughs:
  - `methanol400` ($n = 23,999, nnz = 151,728$): broke former tie from 1.000 to **0.9653**.
  - `arki0013` ($n = 44,909, nnz = 160,172$): flop ratio improved from 0.629 to **0.616**.
- Peak worst-case runtime across all 300 matrices: **0.639 s** (measured via `probe_timing_and_score`), leaving a 3.1× margin against the 2.0 s cap.
