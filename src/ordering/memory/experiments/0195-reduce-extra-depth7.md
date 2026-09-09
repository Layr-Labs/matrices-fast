# 0195 — same-count REDUCE_EXTRA_DEPTHS last entry 6→7

- **Date:** 2026-09-09
- **Base:** clean tip `74b6ccd` (local 0.793834). mid_k2 tip form; dens≤4 / mid-K4 / Ammf / wf01 closed for hidden.
- **Score:** 0.793834 → **0.793834** (0.00)
- **Status:** miss / DEPTHS last entry reverted. Not submitted. Model Grok 4.6, harness Grok Bot.

## Hypothesis

Same-count change on `REDUCE_EXTRA_DEPTHS`: last entry `6 → 7` → `[5, 4, 2, 7]`.
Depth 7 is a deeper from-scratch reduction on `small_band` / `dense_band` only
(uses `reduce_checked` because depth > `REDUCE_ROW_DEG` = 3). Mid band still
only admits `depth == 2` (`mid_k2`); no mid-K4 reopen. Prefer small/dense-band
movers without regressing uncapped worst vs tip (~1.65s).

## What changed

`src/ordering/mod.rs` only (`REDUCE_EXTRA_DEPTHS` last entry), then restored
after the local run:

- was `[5, 4, 2, 6]`
- trial `[5, 4, 2, 7]`
- restored `[5, 4, 2, 6]`

Untouched: `mid_k2` (`depth == 2`), `REDUCE_ROW_DEG` (3), bands, work caps,
alphas, recurse gates, `HEAVY_METRIC_ORDER`, dens≤4 seeds, `indep_first.rs`,
`probe.rs`. No 4th-metric-core / SqDiv edits.

## Result

Yukon local **0.793834** vs tip **0.793834** (`/tmp/yukon-run-0195.log`).
Buckets unchanged: lt_1k 0.8875 / 1k_10k 0.8398 / gt_10k 0.6891. Fill 0.9258.

Flops-exact vs tip `/tmp/yukon-run-tip-74b6ccd.log`: **0 better / 0 worse / 300 same**.
No movers. Keep bar missed (need beat 0.793834, >1 mover, 0 worse).

## Why it won / lost

Null: depth-7 `reduce_checked` on small/dense bands produced no flops-exact
change vs depth 6 on the public set (pair budget / fail-closed likely rejects
the deeper reduction, or it never improves AMD-tied crowns). Same pattern as
extra-depth lottery nulls like [0120](0120-iter93-reduce-depth3.md).
