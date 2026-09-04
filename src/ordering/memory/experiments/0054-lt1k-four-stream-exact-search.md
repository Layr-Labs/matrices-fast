# Experiment 0054: lt_1k four-stream exact search

## Status

Verified win locally. Starting from accepted commit `7177486` (with 0053 selective medium depth, dev baseline `0.845469`), this experiment expands the exact randomized greedy elimination search on small graphs (`n <= 1,000 && nnz <= 30,000`) from 2 streams to 4 streams by adding two 50M-operation trajectories with independent fixed seeds `0x27BB_2EE6_87B0_B0FD` and `0x45A1_89C3_F208_7314`.

The candidate development result across all 300 matrices is:

```text
score = 0.845048
fill  = 0.944611
```

The parent scored 0.845469 / 0.944729 locally. The public reduction is **-0.000421** (**-4.98 basis points** relative to the parent).

## Per-Bucket Breakdown

| Bucket | Count | Parent Flop Ratio | 0054 Flop Ratio | Flop Delta |
| :--- | :--- | :--- | :--- | :--- |
| **`lt_1k`** | 147 | 0.893893 | **0.891820** | **-0.002073** (-20.7 bips) |
| **`1k_10k`** | 108 | 0.868912 | 0.868912 | 0.000000 (control) |
| **`gt_10k`** | 45 | 0.792071 | 0.792071 | 0.000000 (control) |
| **Total** | 300 | 0.845469 | **0.845048** | **-0.000421** (-4.98 bips) |

The small bucket dropped from 0.893893 to 0.891820, while `1k_10k` and `gt_10k` are byte-identical controls.

## Timing Analysis

On small graphs (n <= 1,000), all matrices complete within 0.05s - 0.20s. The global worst-case matrix runtime on the entire corpus remains entirely governed by large and upper-medium matrices (crudeoil_lee4_10 at 0.99s, arki0013 at 0.96s - 1.05s). The four-stream expansion on n <= 1,000 adds zero worst-case latency to the critical path.
