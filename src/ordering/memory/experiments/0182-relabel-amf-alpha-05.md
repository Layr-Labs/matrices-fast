# 0182 — light relabel AMF cycle α16 → 0.5

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843389)
- **Score:** 0.793834 → **0.793837** (+0.000003)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

The light relabelled-AMF cycle inside `nnz <= RELABEL_AMF_MAX_NNZ` is
`amf_alphas = [5.0, 2.0, -1.0, 1.0, 16.0]`. Same-count swap of the last
entry 16.0 → 0.5, length stays 5. α0.5 is already used by `REDUCE_ALPHAS`
on a different path; this tests that dense factor only on the light
relabel lottery. Do not add a sixth alpha. Sweep `[1.0, 16.0, -1.0]` and
HEAVY_RELABEL_AMF alpha stay. The n<5000 extra dense_alpha −1.0 paired
ticket stays.

Keep only if more than one row moves, local beats 0.793834, no worse
rows, and worst order() does not rise. Submit only if score ≤0.793734,
or (≥0.00008 better AND 0 worse). Drop if worst order() rises.

## What changed

`src/ordering/mod.rs` only, inside `leader_order` light relabel cycle.
Reverted after the miss. HEAD remains `74b6ccd`.

- old: `let amf_alphas = [5.0f64, 2.0, -1.0, 1.0, 16.0];`
- tried: `let amf_alphas = [5.0f64, 2.0, -1.0, 1.0, 0.5];`
- restored: `[5.0, 2.0, -1.0, 1.0, 16.0]`

Length stayed 5. Restart count unchanged. AMF sweep loop `[1.0, 16.0, -1.0]`
unchanged. `HEAVY_RELABEL_AMF` alpha unchanged. `REDUCE_ALPHAS` unchanged
(`[0.5, 2.5, 5.0, 10.0]`). n<5000 extra `dense_alpha: -1.0` paired ticket
unchanged. probe.rs left untouched (trailing blank line only).

## Result

Yukon local **0.793837** vs tip **0.793834** (`/tmp/yukon-run-0182.log`).
Fill 0.925772 → 0.925771. Buckets: lt_1k 0.8875 / 0.9599, 1k_10k 0.8398 / 0.9468,
gt_10k 0.6891 / 0.8844 — printed geomeans unchanged at 4 d.p.

Flops-exact vs tip (`/tmp/yukon-run-tip-74b6ccd.log`): **0 better / 1 worse / 299 same**.

- edgecross14-156 (n=3097, nnz=19664): flops 1376964 → 1378748, ratio 0.925 → 0.927

One row moved, and it moved worse. Score rose. Does not beat 0.793834.
Short of score ≤0.793734 and short of ≥0.00008 better with 0 worse.
Public table redacts all 300 times as capped; no per-matrix cap kill.
Not submitted. Light `amf_alphas` restored to `[5.0, 2.0, -1.0, 1.0, 16.0]`.
Leave that list alone.

## Why it won / lost

Replacing the light-cycle 16.0 ticket with 0.5 did not open a cheaper
best-of path on the relabel lottery. The only moved public row
(edgecross14-156) got worse, so the dropped α16 slot was earning a
strictly better flop count than α0.5 on that seed pattern. Do not retry
this same-count last-entry swap.
