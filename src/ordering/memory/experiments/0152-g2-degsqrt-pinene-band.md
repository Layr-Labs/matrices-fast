# 0152 — paid g2/DegSqrt on the pinene200 band only

- **Date:** 2026-09-09
- **Base:** tip `be9bae1` official local **0.794121** / fill **0.926126**. Hidden best 0.843577. Promote bar 0.843476.
- **Score:** official local **0.794121** / fill **0.926126** (exact tie). 0 movers vs tip.
- **Status:** null. Reverted to tip `be9bae1` ordering sources. Not submitted.

## Hypothesis

0149's paid x9 moved only lee4_10 by 0.002 and 0.000017 overall. Do not restore
that n<=18k gate, and do not export g2/DegSqrt onto gabriel09. The 0145 census
pairing g2/DegSqrt (0.9357 -> 0.9234) is gabriel09's result, not a pinene200
result. It is probed here on pinene200 only because the SqDiv payment is already
score-flat. A flat pinene200 row is not evidence the census failed.

Substitute, not an added six-metric list, only when
`18_000 < n <= 20_000 && 80_000 < nnz <= 120_000` (dev row: pinene200 n=19995,
nnz=97990). gabriel09 (n=21688) and popdynm200 (n=22407) stay outside. gasprod
stays out. n<=18k, including lee4_09/10, is unchanged.

- Do not drop degree-greedy caps 15 and 5.
- Add exactly one g2 set: `greedy_independent_set(sp, 2)`.
- That g2 core gets AMD plus DegSqrt only if `cn <= METRIC_CORE_MAX_N` and
  `cnnz <= METRIC_CORE_MAX_NNZ`; otherwise AMD-only. No AMF, METIS, or
  six-metric list on that core.
- Existing metric cores on that same band drop SqDiv so the added DegSqrt is
  paid. SqDiv is not touched on n<=16k.

## What changed

`src/ordering/indep_first.rs` only. Structural n/nnz gate. Deterministic.
Yukon run logged to `/tmp/yukon-run-0152.log`.

## Result

Official local **0.794121** / fill **0.926126**. Exact bit-identical to tip on
all 300 rows (0 movers). pinene200 stayed 0.836; gabriel09 0.913; popdynm200
0.953; lee4_09 0.645; lee4_10 0.639. Buckets unchanged
(lt_1k 0.8875 / 1k_10k 0.8398 / gt_10k 0.6898).

## Why it won / lost

Lost / null. The pinene200 band already had a competitive core portfolio; the
extra g2 set either failed admission / size-dedup or its AMD+DegSqrt walk did
not beat the incumbent AMD/metric winner on that row. SqDiv drop on other
metric cores in-gate was also score-flat (no regression, no gain). Mechanism
does not move the only gated row. Reverted.
