# 0151 — DegSqrt on the 5th AMD core, paid by dropping SqDiv on n<=16k

- **Date:** 2026-09-09
- **Base:** tip `be9bae1` official local **0.794121** / fill **0.926126**. Hidden best 0.843577. Promote bar 0.843476.
- **Score:** official local **0.794155** / fill **0.926131**. Delta **+0.000034** vs 0.794121. lt_1k flop 0.8875 -> 0.8876; 1k_10k and gt_10k flop unchanged (0.8398 / 0.6898).
- **Status:** negative. Change reverted. Not submitted.

## Hypothesis

0150's x9 widen onto lee4_09 was killed before a run: lee4_09 is n<=16k and
already the 273a win (metric top-4 + DegP075). Second-colour still requires
nnz<=80k, so x9 is not admitted there. Census x9/DegSqrt 0.6512 is not a
reachable target of this experiment.

Substitute, not an added pass, and not a new set. On `n <= 16_000` only,
drop SqDiv from the existing metric cores (0143: no core wins) and spend one
freed slot on DegSqrt on the already-admitted 5th-lowest AMD core, and only
when that core is inside the metric envelope and its AMD total is within 5/4
of the best. DegSqrt stays on the current metric_k cores. The extra DegSqrt
is scheduled only when at least two SqDiv walks are actually dropped, so the
metric task count falls (-k + 1, k>=2). n>16k unchanged. No x9, no caps
changes, no METIS/AMF gate changes.

## What changed

`src/ordering/indep_first.rs` only, inside `run` phase-2 metric scheduling.
Reverted after the run.

## Result

`yukon run` 300/300, score **0.794155** / fill **0.926131**. Not a compile failure.

lee4_09 (n=15904, nnz=101792) **0.645** unchanged (flops 134613233).
lee4_10 (n=17809, nnz=120632) **0.639** unchanged (flops 188270853).
lee4_06 unchanged at 0.507.

Only two rows moved, both n<1k: ndcc12 0.953->0.953 (flops 329556->329342) and
ndcc13 0.618->0.630 (flops 324727->331007). 1 better / 1 worse / 298 same.
No n>12k band row moved. Worst `order()` was not printed (table column is
`(capped)`); timing is not claimed. Not submitted.

## Why it won / lost

The 5th-core DegSqrt did not touch the n<=16k band that 273a already metrics
(lee4_09 stayed at 0.645). Dropping SqDiv was not free: ndcc13 got worse by
0.012 ratio, which lifted the lt_1k geomean and the official score. SqDiv is
not uniformly dead on n<=16k. Do not retry this as a 0147 lookalike, and do
not drop SqDiv on the n<=16k metric list to pay a 5th-core metric. Do not add
x9 on lee4_09.
