# 0184 — mid-band below-anchor gate K=2 → K=4

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843389)
- **Score:** 0.793834 → **0.793881** (+0.000047)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

In the `REDUCE_EXTRA_DEPTHS` loop, the mid-below-anchor gate currently admits
only `depth == 2` when `60k < nnz ≤ 200k` and `best_flops < amd_flops`, with a
one-shot `work_cap = nnz`. Same-count only: change that admit to `depth == 4`
(`mid_k4`). Depth 2 still runs when `small_band` or `dense_band` is true.
Depths 5 and 6 stay mid-skipped. Shipped primary `REDUCE_ROW_DEG = 3` stays.
`REDUCE_EXTRA_DEPTHS` array unchanged (`[5, 4, 2, 6]`). No second mid depth,
no nested K=4, no mid_force widen, no min-fill spend gate.

Keep only if more than one mid-band row (60k<nnz≤200k with best under AMD)
moves, local beats 0.793834, no worse rows, and worst order() does not rise.
Submit only if score ≤0.793734, or (≥0.00008 better AND 0 worse), and more
than one mid-band row moves, and no row anywhere is worse. Drop if worst
order() rises.

## What changed

`src/ordering/mod.rs` only (REDUCE_EXTRA_DEPTHS loop mid gate). Reverted after
the miss. HEAD remains `74b6ccd`. probe.rs left untouched (trailing blank line).

- `mid_k2` / `depth == 2` → `mid_k4` / `depth == 4`
- nnz band and `best_flops < amd_flops` unchanged
- one-shot mid `work_cap = nnz` unchanged
- comments updated from K=2-only mid band to K=4-only

## Result

Yukon local **0.793881** vs tip **0.793834** (`/tmp/yukon-run-0184.log`).
Fill tiebreak 0.9258 → 0.9258. Buckets: lt_1k 0.8875 / 0.9599 (unchanged),
1k_10k 0.8398 / 0.9468 (unchanged), gt_10k 0.6891→0.6892 / 0.8844→0.8846.

Flops-exact vs tip (`/tmp/yukon-run-tip-74b6ccd.log`): **2 better / 1 worse / 297 same**.

Mid-band (60k<nnz≤200k, tip under AMD): **3 movers** (2 better / 1 worse):

- nuclear10a (n=17493, nnz=163816): 58215556 → 57044513 (−1171043), ratio 0.706→0.692
- popdynm200 (n=22407, nnz=105584): 2447575 → 2408852 (−38723), ratio 0.953→0.938
- crudeoil_pooling_dt2 (n=18742, nnz=75910): 9947938 → 10395148 (+447210), ratio 0.664→0.694 **worse**

Does not beat 0.793834. Short of score ≤0.793734 and short of ≥0.00008 better
with 0 worse. One worse row anywhere fails keep. Public table redacts all 300
times as capped; no per-matrix cap kill observed. Not submitted. `mod.rs`
reverted to mid_k2 / depth==2.

## Why it won / lost

Swapping the mid admit from K=2 to K=4 does move more than one under-AMD mid
row, and two of them improve, but the same swap regresses crudeoil_pooling_dt2
enough to lift the geomean (+0.047 bip). K=4 is not a free substitute for the
shipped K=2 mid ticket: the one-shot work_cap still fires once, but the deeper
prefix finds a different (sometimes worse) incumbent before later stages can
recover. Do not nest K=4 on top of K=2 in this band (0158 worse); do not retry
this same-count K=2→K=4 swap.

## Follow-ups

- none for this same-count swap

## Links

- Techniques: multi-depth reduce prefixes / mid below-anchor band
