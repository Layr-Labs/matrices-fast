# 0149 — 273a leftovers + pair-descent EXT (no MINL2, no hold)

- **Date:** 2026-09-09
- **Base:** tip `83a9250` / source `be9bae1` (273a), hidden **0.843577**, local **0.794121**
- **This:** probe **0.794048** (−0.73 bip), worst **1.085 s**
- **Status:** submitting
- **Files:** `indep_first.rs` (f3/g4 n≥20k), `mod.rs` (4b-lift subtree; pair_descent_ext nnz 30k→80k)

## Why this stack

After v21 failed, 273a is still #1. Closed since then:

| id | result | lesson |
|---|---|---|
| 288a `fda6b8e` | hidden fail | second MINL n≤12k nnz≤70k, lee4_06 in-gate 1.095 s |
| 290a `1ab6be1` | hidden fail | second MINL n≤5k nnz≤50k — **lee4_06 out, still FAIL** |
| 333a `c2caa5d` | hidden fail | 40% hold + pairdesc EXT |
| 330a `4f49891` | rejected 0.843545 | pairdesc EXT **alone** (−0.32 hidden bip, below 1-bip) |
| v21 / 313a | hidden fail | lee4_06 4b-hold |

0148 stacked MINL2 n≤8k on the v23f leftovers. 290a proved that family dies on hidden even with lee4_06 excluded. 0148 aborted.

330a proved pairdesc EXT (nnz 30k→80k, n∈(4k,12k], no hold) **gets a hidden score** rather than SIGKILL. Alone it is thin. Stack it on the v23f leftovers (f3/g4/4b-subtree) so the hop is 8 movers / two mechanisms, not a 330a twin.

## Movers vs 273a (8 better / 0 worse)

| matrix | n | nnz | 273a | this | mechanism |
|---|---:|---:|---:|---:|---|
| methanol200 | 11999 | 76128 | 0.7246 | **0.7204** | 4b-lift subtree |
| gasprod_sarawak81 | 22536 | 75636 | 0.9135 | **0.9112** | fill-greedy cap-3 |
| popdynm200 | 22407 | 105584 | 0.9532 | **0.9518** | degree-greedy cap-4 |
| powerflow0300p | 11251 | 41918 | 0.9652 | **0.9640** | pairdesc EXT |
| mpbp_35 | 11120 | 40790 | 0.3226 | **0.3224** | pairdesc EXT |
| crudeoil_lee2_06 | 6418 | 34646 | 0.7586 | **0.7584** | pairdesc EXT |
| transswitch0300p | 11659 | 48446 | 0.9377 | **0.9376** | pairdesc EXT |
| crudeoil_lee4_06 | 10429 | 55492 | 0.5074 | 0.5074 | pairdesc EXT −1053 flops |

pinene200 0.8357, pooling_digabel19 0.8330, lee4_09 0.6447, mpbp_34 0.2892 unchanged.

## Measured

| revision | SCORE | WORST | 1k_10k | gt_10k |
|---|---:|---:|---:|---:|
| 273a | 0.794121 | 1.105 s | 0.8398 | 0.6898 |
| v23f leftovers only | 0.794061 | 1.078 s | 0.8398 | 0.6897 |
| 330a pairdesc only | 0.794108 | 1.107 s | — | — |
| **this** | **0.794048** | **1.085 s** | 0.8398 | **0.6896** |

lt_1k 0.8875 control.
