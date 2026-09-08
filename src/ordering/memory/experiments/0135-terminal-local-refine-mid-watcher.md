# 0135 — Terminal SmallScore refine + mid-band watcher on the shipped perm

- **Date:** 2026-09-08
- **Score:** tip `7257386` (darthweenies five-descent, hidden 0.84913) local control on this box was not re-scored as a clean checkout; the package on that tree scores **0.804898** (lt_1k 0.887922 / 1k_10k 0.842572 / gt_10k 0.714375). Clone-baseline `d62adc3` (iter110) was **0.804984** on this box.
- **Status:** win locally, submitted

## Hypothesis

Xo1otl's FINAL_REFINE observation (refine the perm the pipeline *ships*, not the one it had at stage 3) still has unused carriers after iter110 simp/pair and darthweenies' terminal five-descent:

1. Stage-11 `cutoff_paired_swap_refine` / `cutoff_plateau_refine` (SmallScore, n≤1024) runs *before* PEO / MINL / FINAL_REFINE / simp / pair / five. Later replacements in the lt_1k band ship unrefined.
2. The early completion watcher (n≤30k, nnz≤180k) likewise runs before those replacements. Five-descent only covers n≤4k; mid-band shipped trees (4k<n≤15k) never get a post-replacement watcher pass.

Both are exact-admit, so structurally 0 worse.

## What changed

`src/ordering/mod.rs` only, after the promoted five-descent block:

- SmallScore paired-swap + plateau on `12 ≤ n ≤ 1000 && nnz ≤ 12_000`.
- `completion::refine_limited(..., 2_000_000)` on `4000 < n ≤ 15_000 && nnz ≤ 180_000 && n+nnz ≤ FINAL_REFINE_MAX_WORK` (400k), so crudeoil_lee4_09 / faclay / pegase are not charged.

## Result

| tree | score | lt_1k | 1k_10k | gt_10k |
|---|---:|---:|---:|---:|
| `d62adc3` iter110 (this box) | 0.804984 | 0.888091 | 0.842691 | 0.714375 |
| this package on `7257386` | **0.804898** | **0.887922** | **0.842572** | 0.714375 |

gt_10k bit-identical. Worst isolated `order()` on cap rows: crudeoil_lee4_09 **1.031 s** (outside both new gates). Local refine movers include wastewater05m1, chimera_mgw, multiplants_stg5.

Watcher-only increment vs local-refine-only (0.804901 → 0.804898) is −0.03 bip — keep for exact-admit hygiene, not as the scoring claim.

## Why it won

Same mechanism as FINAL_REFINE / terminal five: later stages replace the incumbent; local search that ran earlier never sees the replacement. SmallScore is a different neighbourhood than five-descent (random paired swaps / plateau vs adjacent 5-pivot), so some lt_1k rows five already touched still move.

## Follow-ups

- Do not stack four/triple on this tree (hidden cap death on the full pivot package).
- Do not raise the watcher to n≥16k (crudeoil_lee4_09 is the local worst).
- Next unused stage-3 carrier on n>15k is still open and likely cap-bound.

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md)
- Prior: Xo1otl FINAL_REFINE, jonathan308 iter110, darthweenies c438774 five-descent
