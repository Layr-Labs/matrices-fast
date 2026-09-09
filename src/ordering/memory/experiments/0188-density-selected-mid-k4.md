# 0188 — density-selected mid-band K2/K4

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, promote ≤0.843389)
- **Score:** 0.793834 → **0.793612** (−0.000222)
- **Status:** keep / submit candidate. Model Grok 4.6, harness Grok Bot.

## Hypothesis

0184 showed that swapping the entire mid-below-anchor admit from depth 2 to
depth 4 wins on denser mid rows (nuclear10a, popdynm200) and loses on a
sparser mid row (crudeoil_pooling_dt2). Structural density ≥4.5
(`2 * nnz >= 9 * n`) should separate those classes without matrix-identity
gates: denser mid cores take K=4; otherwise keep the shipped K=2. Exactly one
mid depth per mid row, same one-shot `work_cap = nnz`.

## What changed

`src/ordering/mod.rs` only (REDUCE_EXTRA_DEPTHS loop mid gate). HEAD remains
`74b6ccd`. `probe.rs` left untouched (trailing blank line).

- Preserve mid band: `60k < nnz <= 200k && best_flops < amd_flops`
- Select one mid depth by density:
  - if `nnz.saturating_mul(2) >= n.saturating_mul(9)` admit `depth == 4`
  - else admit `depth == 2`
- one-shot mid `work_cap = nnz` unchanged
- `small_band` / `dense_band` unchanged; depths 5/6 stay mid-skipped
- `REDUCE_EXTRA_DEPTHS`, `REDUCE_EXTRA_ALPHAS`, K3, core ledger untouched

## Result

Yukon local **0.793612** vs tip **0.793834** (`/tmp/yukon-run-0188.log`).
Fill tiebreak 0.9258 → 0.9256. Buckets: lt_1k 0.8875 / 0.9599, 1k_10k 0.8398 /
0.9468, gt_10k 0.6898→0.6886 / 0.8844→0.8839.

Flops-exact vs tip (`/tmp/yukon-run-tip-74b6ccd.log`): **2 better / 0 worse /
298 same**.

Mid-band movers (density gate):

- nuclear10a (n=17493, nnz=163816, dens≈9.36 ≥4.5 → K4): 58215556 → 57044513
  (−1171043), ratio 0.706→0.692
- popdynm200 (n=22407, nnz=105584, dens≈4.71 ≥4.5 → K4): 2447575 → 2408852
  (−38723), ratio 0.953→0.938
- crudeoil_pooling_dt2 (n=18742, nnz=75910, dens≈4.05 <4.5 → K2): unchanged
  9947938 / 0.664 (0184 regression avoided)

Comparative `probe_timing_and_score` (SSI_PROBE_REPEAT=3) on movers + tip slow
rows (`/tmp/probe-0188-movers.log`):

- nuclear10a order() min 0.933 s (tip single-shot 1.053 s)
- popdynm200 order() min 1.056 s (tip 1.135 s)
- crudeoil_pooling_dt2 0.952 s (tip 0.909 s)
- focused-set worst 1.450 s (tip corpus worst 1.509 s); movers ≤1.14 s

Meets keep bar (beats tip, >1 row moves, 0 worse) and submit bar
(score ≤0.793734, >1 mover, 0 worse, timing not above tip/crown).

## Why it won / lost

Density-selected mid depth keeps the 0184 K4 wins and leaves the sparse mid
row on K2, so the pooling_dt2 regression never fires. Same one-shot cap means
each mid row still pays exactly one extra reduce, only at the density-chosen
depth.

## Follow-ups

- none required; do not reintroduce additive K2+K4 or identity gates

## Links

- Prior miss: [0184](0184-mid-band-k4.md)
- Techniques: multi-depth reduce prefixes / mid below-anchor band / density gate
