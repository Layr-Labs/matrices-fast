# 0193 — same-count HEAVY_METRIC_ORDER last ticket → cm_ammf

- **Date:** 2026-09-09
- **Base:** 0192 package (dens≤4 else-floor seed on tip mid_k2; local 0.793811). Mid closed tip mid_k2.
- **Score:** 0.793811 → **0.793811** (0.00)
- **Status:** miss / Ammf reverted; 0192 dens≤4 left intact. Not submitted. Model Grok 4.6, harness Grok Bot.

## Hypothesis

Same-count swap on `HEAVY_METRIC_ORDER`: replace the last ticket
`("extra_deg2_div_nv_wf002", 10.0)` with `("cm_ammf", 10.0)`. Ammf is the
approximate minimum mean local fill variant already implemented in
`custom_metrics` (same shape as `cm_sqpure`). The fourth ticket is only taken
when `k == 4`. Prefer heavy-live rows moving. No AmindNorm. Band caps / budget /
giant / sparse_hub unchanged. dens≤4 seed and mid_k2 untouched.

## What changed

`src/ordering/mod.rs` only (`HEAVY_METRIC_ORDER` + one match arm), then restored
after the local run:

1. `extra_deg2_div_nv_wf05` / 10.0
2. `cm_sqpure` / 5.0
3. `extra_deg2_div_nv_wf05` / 2.5
4. `cm_ammf` / 10.0  (was `extra_deg2_div_nv_wf002` / 10.0)

Match arm `"cm_ammf"` → `ScoreVariant::Ammf` (mirror of `cm_sqpure`). Reverted
after run; last ticket is `extra_deg2_div_nv_wf002` / 10.0 again. dens≤4 seed
and mid_k2 unchanged throughout.

## Result

Yukon local **0.793811** vs 0192 baseline **0.793811** (`/tmp/yukon-run-0193.log`).
Buckets unchanged: lt_1k 0.8875 / 1k_10k 0.8397 / gt_10k 0.6891. Fill 0.9258.

Flops-exact vs 0192: **0 better / 0 worse / 300 same**. No heavy-live movers.
Vs tip still 2/0 (the 0192 dens≤4 wins). Keep bar missed (need clearly better
than 0.793811, >1 mover, 0 worse).

## Why it won / lost

Null like [0166](0166-heavy-metric-cm-sqdiv.md). The fourth ticket either never
fires on a row where Ammf α10 beats the incumbent, or Ammf ties the removed
wf002 basin. No new dispatcher cost on rows that skip k=4.

## Follow-ups

- Do not retry this exact last-ticket Ammf swap.
- dens≤4 package left in tree; `268e91c` **failed** (official n/a) after this run.
- Mid remains closed (tip mid_k2).

## Links

- Prior heavy-metric swap miss: [0166](0166-heavy-metric-cm-sqdiv.md)
- Base package: [0192](0192-dens4-else-seed-on-tip.md)

## Submission

Not submitted. Ammf reverted only.
