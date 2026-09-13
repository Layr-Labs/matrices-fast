# 0094 — Dense-giant α2.5 twin + terminal count-ranked peel; sparse-giant floors

**Date:** 2026-09-07. **Base:** sub6 / `5d2aeab9` (dev 0.811703).
**Result:** dev **0.810282** (−14.2 bips vs 0.811703); worst same-box
`order()` 1.232 (faclay75 0.85 → 1.09, pooling_sppc3pq 1.13 → 1.20) s (2-vCPU pod, min-of-2).

## Pieces (each attributed by the old tree's phase trail or by `probe_census`)

1. **Dense-giant twin.** On dense giants (`nnz >= 700k`, `nnz >= 20 n`) the heavy
   block queues the α2.5 twin of its first variant beside the α10 pass: on
   `pooling_sppc3pq` that is a distinct basin (0.4846 vs 0.5338 at the
   portfolio) and the old tree's trail reached its 0.3925 from exactly there.
   Selected EXPLICITLY (entries 0 and 2 of `HEAVY_METRIC_ORDER`): an earlier
   draft reordered the list instead and silently dropped `SqPure` α5 out of the
   sparse band's two-entry prefix, costing `crudeoil_pooling_dt3` 18 bips.
   Never reorder that list — it is a prefix per band.
2. **Terminal count-ranked peel on dense giants.** Rank vertices by the
   finished incumbent's exact column counts, splice the top-k to the END in
   ascending true degree, accept on strict exact decrease; k ∈ {2, 16, 96},
   four exact evaluations, ~40 ms. `pooling_sppc3pq` 0.4254 → 0.3922. Every
   other terminal stage is gated off these rows, so this is their only
   post-portfolio refinement.
3. **Light tier + `extra_deg2_div_nv_wf05`.** The one `metric_sweep` spec that
   the census found as the best single generator on the light tier
   (`gabriel09` 0.9446 vs the finished 0.9568; final 0.9378 with it).
4. **Sparse-giant floors.** `HEAVY_METRIC_MAX_NNZ` 1.2M → 1.4M admits the
   sparse-hub giant class (`faclay75`, max degree 2777) to its hub-scale variant
   (census: `DegDivNvWfP15` 0.9667 → 0.9405, 0.19 s), and hub-free sparse giants
   in [500k, 1.5M] get a floor of ONE relabelled-AMD restart where the budget
   gave zero (census: `acopf` 1.0000 → 0.9737, 0.13 s).

## Negative results

- A fixed-α5 relabelled-AMF pass for the seeds the cycled loop assigns other
  α (census favourite on procurement1large / chp_shorttermplan2d): zero changed
  rows, +3 s corpus time.
- METIS seeds 2/21/26/55 on the dense band: win only where the shapes (0093)
  win.
