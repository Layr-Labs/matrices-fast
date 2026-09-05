# 0010 — Relabelled-MinFill Multi-Start: Randomized Deficiency Elimination on Small Graphs

**Date:** 2026-09-02
**Score:** 0.868096 → **0.867686** (−0.000410, −4.7 bips). Buckets: lt_1k **0.9064→0.9053** (−0.0011),
1k_10k **0.8963→0.8960** (−0.0003), gt_10k 0.8182. Fill tiebreak 0.958983→0.958763.
**Status:** WIN, shipped. 19 matrices strictly improved.

## Hypothesis

In `memory/open-questions.md`, open lead #1 proposed:
*"Relabel the OTHER numbering-sensitive routines: the hand-rolled RCM, Sloan, `nd_order` / `ndfm_order`... and MinFill."*

`minfill_order` is an exact minimum-deficiency heuristic: at each step, it eliminates the vertex with minimum local fill (the smallest number of edges added between its uneliminated neighbours). Tied vertices are broken by degree and vertex index. Because index tie-breaking is sensitive to initial vertex numbering, $B = Q A Q^T$ yields distinct deficiency elimination orders at the cost of a single fast MinFill pass.

For small combinatorial graphs ($n < 2,000$ and $nnz < 10,000$), each MinFill pass consumes under 0.3 milliseconds. Running 6 randomized restarts requires less than 2 milliseconds per matrix, presenting zero worst-case latency risk while providing a third distinct elimination objective alongside AMD (min-degree) and AMF (approximate min-fill).

## Results & Verification

- Score on dev corpus: **0.867686** (fill tiebreak **0.958763**).
- **19 matrices strictly improved**, concentrated in small network and optimization models where exact deficiency elimination outperforms degree bounds:
  - `multiplants_stg1`: 178,193 → **170,933** (−4.1%)
  - `chimera_mgw-c8-439-onc8-002`: 94,124 → **92,600** (−1.6%)
  - `st_bpaf1a`: 610 → **580** (−4.9%)
  - `clay0204hfsg`: 6,808 → **6,712** (−1.4%)
  - `syn40m`: 4,965 → **4,893** (−1.5%)
  - `rsyn0840m`: 10,827 → **10,732** (−0.9%)
- Worst-case latency across all 300 matrices: **0.745 s**, comfortably below the 2.000 s SIGKILL cap.
