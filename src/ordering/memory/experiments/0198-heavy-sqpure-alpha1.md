# 0198 — HEAVY_METRIC_ORDER cm_sqpure α5.0→1.0

- **Date:** 2026-09-09
- **Base:** clean tip `74b6ccd` (local 0.793834). 0197 DegP125 reverted. Closed: mid-K4, dens≤4, heavy last-ticket flats, depth-7, 4th-core/SqDiv-drop, DegP075→DegP125 on n>16k. Exact-search seeds / metric_k / SqDiv list tip; keep 475a hunks; no extrarelbl.
- **Score:** 0.793834 → **0.793834** (0.00)
- **Status:** miss / slot[1] alpha reverted to 5.0. Not submitted. Model Grok 4.6, harness Grok Bot. Benchmark `8c3e7051-530a-4aee-88df-a426e6e78151`.

## Hypothesis

Same-count change on `HEAVY_METRIC_ORDER` slot[1] only: `("cm_sqpure", 5.0)` → `("cm_sqpure", 1.0)`. Tip comment notes SqPure α10 vs α5 on crudeoil_pooling_dt3; try α1 while leaving tickets 0/2/3, names, band caps, budget, giant/sparse_hub untouched. Keep bar: beat 0.793834, >1 heavy-live mover, 0 worse, uncapped worst not rising vs tip (~1.65s; prefer ≲1.14s).

## What changed

`src/ordering/mod.rs` only (`HEAVY_METRIC_ORDER` second ticket alpha), then restored after the local run:

1. `extra_deg2_div_nv_wf05` / 10.0
2. `cm_sqpure` / 1.0  (was 5.0; reverted)
3. `extra_deg2_div_nv_wf05` / 2.5
4. `extra_deg2_div_nv_wf002` / 10.0

No name / match-arm / budget / dens≤4 / mid / indep_first edits. `probe.rs` untouched. Reverted; slot[1] is `cm_sqpure` / 5.0 again.

## Result

Yukon local **0.793834** vs tip **0.793834** (`/tmp/yukon-run-0198.log`).
Buckets unchanged: lt_1k 0.8875 / 1k_10k 0.8398 / gt_10k 0.6891. Fill 0.9258.

Flops-exact vs tip `/tmp/yukon-run-tip-74b6ccd.log`: **0 better / 0 worse / 300 same**.
No heavy-live movers. Keep bar missed.

## Why it won / lost

Null like prior heavy-metric flats ([0166](0166-heavy-metric-cm-sqdiv.md), [0193](0193-heavy-metric-ammf.md), [0194](0194-heavy-metric-wf01.md)). SqPure α1 either never displaces the α5 incumbent on rows that take the second heavy-metric ticket, or ties flops exactly. No score or row movement; not worth the α change.

## Follow-ups

- Do not retry this exact cm_sqpure α5→1 swap.
- Tip ordering restored (slot[1] cm_sqpure@5.0).
- dens≤4 / mid-K4 / heavy last-ticket flats remain closed for hidden.
