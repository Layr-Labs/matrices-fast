# 0194 — same-count HEAVY_METRIC_ORDER last ticket → extra_deg_div_nv_wf01

- **Date:** 2026-09-09
- **Base:** clean tip `74b6ccd` (local 0.793834). mid_k2 tip form; dens≤4 / mid-K4 / Ammf closed for hidden.
- **Score:** 0.793834 → **0.793834** (0.00)
- **Status:** miss / last ticket reverted. Not submitted. Model Grok 4.6, harness Grok Bot.

## Hypothesis

Same-count swap on `HEAVY_METRIC_ORDER`: replace the last ticket
`("extra_deg2_div_nv_wf002", 10.0)` with `("extra_deg_div_nv_wf01", 10.0)`.
That name already exists in `metric_sweep::EXTRA_METRICS` and is handled by the
generic `spec_name` match arm — no new arm. The fourth ticket is only taken
when `k == 4`. Prefer heavy-live rows moving. Band caps / budget / giant /
sparse_hub unchanged. First three tickets untouched.

## What changed

`src/ordering/mod.rs` only (`HEAVY_METRIC_ORDER` last entry), then restored
after the local run:

1. `extra_deg2_div_nv_wf05` / 10.0
2. `cm_sqpure` / 5.0
3. `extra_deg2_div_nv_wf05` / 2.5
4. `extra_deg_div_nv_wf01` / 10.0  (was `extra_deg2_div_nv_wf002` / 10.0)

No new match arm. No dens≤4 / mid_k4 / Ammf / cm_sqdiv edits. Reverted after
run; last ticket is `extra_deg2_div_nv_wf002` / 10.0 again.

## Result

Yukon local **0.793834** vs tip **0.793834** (`/tmp/yukon-run-0194.log`).
Buckets unchanged: lt_1k 0.8875 / 1k_10k 0.8398 / gt_10k 0.6891. Fill 0.9258.

Flops-exact vs tip `/tmp/yukon-run-tip-74b6ccd.log`: **0 better / 0 worse / 300 same**.
No heavy-live movers. Keep bar missed (need beat 0.793834, >1 mover, 0 worse).

## Why it won / lost

Null like [0166](0166-heavy-metric-cm-sqdiv.md) and [0193](0193-heavy-metric-ammf.md).
The fourth ticket either never fires on a row where wf01 α10 beats the
incumbent, or wf01 ties the removed wf002 basin. No new dispatcher cost on
rows that skip k=4.

## Follow-ups

- Do not retry this exact last-ticket wf01 swap.
- dens≤4 / mid-K4 / Ammf remain closed for hidden.
- Tip ordering logic restored (last ticket wf002@10).

## Links

- Prior heavy-metric last-ticket misses: [0166](0166-heavy-metric-cm-sqdiv.md), [0193](0193-heavy-metric-ammf.md)

## Submission

Not submitted. Ticket name reverted only.
