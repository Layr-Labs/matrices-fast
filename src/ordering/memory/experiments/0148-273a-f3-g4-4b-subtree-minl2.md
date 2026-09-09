# 0148 — 273a leftovers stacked: f3/g4 + 4b-lift subtree + gated second MINL

- **Date:** 2026-09-09
- **Base:** tip `83a9250` / source `be9bae1` (273a), hidden **0.843577**, local **0.794121**, worst **1.105 s**
- **This:** probe **0.793999** (−1.22 bip), worst **1.095 s** (lee4_09; same row as 273a)
- **Status:** aborted before submit — 290a proved second MINL fails hidden even at n≤5k
- **Files:** `indep_first.rs` (f3/g4 on n≥20k, Amf5 on dt3-band small cores), `mod.rs` (`one_subtree_polish` after 4b accept; `apply_minl_descent`; second MINL n≤8k nnz≤40k)

## Why this, not the fail family

| id | result | why |
|---|---|---|
| skip-METIS v12b `0ae0c74`, v5 `fe871f1` | hidden fail | lee4_09/10 band extras |
| 274a `ee626bb` | hidden fail | AmindNorm on the whole x-set band |
| 276a `270bcb7` | rejected 0.00% | tight AmindNorm |
| 302a `b10cd7e` | hidden fail | same tight AmindNorm |
| v21 `d4a7cbf` / 313a `3dc7fdc` | hidden fail | lee4_06 4b hold |
| jonathan 333a `c2caa5d` | hidden fail | hold-only + pair-descent EXT, worst 1.124 s |
| 288a `fda6b8e` | hidden fail | second MINL n≤12k nnz≤70k; lee4_06 in-gate at 1.095 s |
| darthweenies `83941fc` | rejected −0.47 hidden bip | below the 1-bip bar |

273a itself is gated metric top-4 + DegP075 on 265a. Leftovers that still convert and are not that list: unused set constructors on the n≥20k 1b band, subtree on a *winning* 4b lift, and a **tight** second MINL (288a's idea with lee4_06 out).

## Movers vs 273a (11 better / 0 worse)

v23f leftovers (gt_10k, 1b / 4b):

| matrix | n | nnz | 273a | this | note |
|---|---:|---:|---:|---:|---|
| methanol200 | 11999 | 76128 | 0.7246 | **0.7204** | 4b-lift subtree |
| gasprod_sarawak81 | 22536 | 75636 | 0.9135 | **0.9112** | f3/amd 1b, then subtree |
| popdynm200 | 22407 | 105584 | 0.9532 | **0.9518** | g4/ddnsw 1b |

Gated second MINL vs v23f (1k_10k, n≤8k nnz≤40k):

| matrix | n | nnz | v23f | this | Δflops |
|---|---:|---:|---:|---:|---:|
| pooling_sppa0pq | 1649 | 24192 | 0.3297 | **0.3245** | −19331 |
| chimera_selby-c16-01 | 2031 | 10964 | 0.6744 | **0.6710** | −12137 |
| crudeoil_lee1_07 | 3670 | 19322 | 0.7486 | **0.7467** | −9174 |
| maxcsp-langford-3-11 | 660 | 29646 | 0.3127 | **0.3126** | −4861 |
| crudeoil_lee2_06 | 6418 | 34646 | 0.7586 | **0.7585** | −2232 |
| crudeoil_pooling_ct3 | 2644 | 11426 | 0.6282 | **0.6266** | −2167 |
| chimera_rfr-02 | 2032 | 15140 | 0.6537 | **0.6536** | −561 |
| crudeoil_li05 | 4225 | 14628 | 0.7847 | **0.7847** | −21 |

lee4_06 / lee4_09 / lee4_10 / mpbp_34 / mpbp_35 / chp_shorttermplan2a / pooling_digabel19 / pinene200 bit-identical to 273a (or to v23f which was identical there).

## Why the 288a gate was fatal and this one is not

288a re-ran MINL whenever later stages beat the post-MINL incumbent, for `n<=12_000 && nnz<=70_000`. lee4_06 (n=10429, nnz=55492) is in that window. Isolated REPEAT=3: lee4_06 **1.095 s**. Hidden ~1.8× local sits on the 2.0 s SIGKILL. The submission FAILED (no hidden score), same category as the hold family.

This gate is `n<=8_000 && nnz<=40_000`. lee4_06 is out. Full-probe lee4_06 **0.996 s**, worst is still lee4_09 **1.095 s** (273a already promoted at 1.105 s; subset remeasure 1.078 s — noise). Second MINL never touches the 1.07 s critical path.

## Measured

| revision | SCORE | WORST | 1k_10k | gt_10k |
|---|---:|---:|---:|---:|
| 273a | 0.794121 | 1.074–1.105 s | 0.8398 | 0.6898 |
| v23f (f3/g4/4b-subtree) | 0.794061 | 1.078 s | 0.8398 | 0.6897 |
| **this (v23f + MINL2)** | **0.793999** | **1.095 s** | **0.8396** | **0.6897** |

lt_1k 0.8875 control. Weights 0.30 / 0.30 / 0.40. −1.22 local bip vs 273a.
