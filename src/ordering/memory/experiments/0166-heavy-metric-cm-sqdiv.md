# 0166 — cm_sqdiv into the heavy-metric 4-ticket prefix

- **Date:** 2026-09-09
- **Score:** 0.793834 → **0.793834** (0.00 bip)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

Same-count swap on `HEAVY_METRIC_ORDER`: drop the last ticket `extra_deg2_div_nv_wf002` / 10.0 and put `cm_sqdiv` at dense_alpha 10.0. The heavy-metric dispatcher already has a `cm_sqdiv` arm (`ScoreVariant::SqDiv`); this is a ticket rename, not a new caller. The fourth ticket is only taken when `k == 4` (roughly 500k < nnz < 700k, outside the dead window and below the giant trim).

## What changed

`HEAVY_METRIC_ORDER` last ticket only, then restored after the local run:

1. `extra_deg2_div_nv_wf05` / 10.0
2. `cm_sqpure` / 5.0
3. `extra_deg2_div_nv_wf05` / 2.5
4. `cm_sqdiv` / 10.0  (was `extra_deg2_div_nv_wf002` / 10.0)

`cm_sqdiv` was absent from the list. Count stayed 4. No new match arm. `SUBTREE_CHAIN_MAX_N` stays 45_000. HEAD remains `74b6ccd`.

Code was reverted after the local run. The last ticket is `extra_deg2_div_nv_wf002` / 10.0 again.

## Result

Yukon local **0.793834** vs tip rebaseline **0.793834** (`/tmp/yukon-run-0166.log`). Short of the submit bar (need `< 0.793734`, more than one row better, none worse). Buckets unchanged: lt_1k 0.8875, 1k_10k 0.8398, gt_10k 0.6891. Fill 0.9258.

**0 better / 0 worse** / 300 same versus `/tmp/yukon-run-tip-74b6ccd.log`. Not submitted. `src/ordering/mod.rs` restored to the tip.

## Why it won / lost

The fourth ticket never moved a row. Either no matrix in the k=4 band admitted SqDiv α10 over the incumbent, or that band did not include a row where dropping `extra_deg2_div_nv_wf002` / 10.0 changed the best-of. Net is a null. Do not resubmit this swap, and do not invent another heavy-metric caller to force the variant in.

## Follow-ups

- Do not invent a new dispatcher arm. A later heavy-metric swap must use a name the existing match already dispatches (`cm_sqdiv`, `cm_sqpure`, or an `EXTRA_METRICS` name).
- Do not retry this exact last-ticket replacement.

## Links

- Code: `HEAVY_METRIC_ORDER` and the existing `cm_sqdiv` arm in `src/ordering/mod.rs`
- Related: [0165](0165-heavy-metric-degsqrt.md), [0092](0092-ported-heavy-tier-metrics-parallel-portfolio.md)
