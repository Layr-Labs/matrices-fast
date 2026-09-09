# 0148 — n>16k substitute DegP125 for SqDiv

- **Date:** 2026-09-09
- **Base:** tip `be9bae1` official local **0.794121** / fill **0.926126** (300/300). Claim worst order() 1.105 s.
- **Score:** official local **0.794121**, fill **0.926127**. Buckets lt_1k 0.8875 / 1k_10k 0.8398 / gt_10k 0.6898. Exact tie with the 273a local claim at 6 decimals (delta 0.000000).
- **Status:** null. DegP125 swap reverted. Not submitted.

## Hypothesis

0147's 5/4 fourth-core gate was a null. On n>16k the metric schedule still
walks SqDiv, which 0143 found no core wins for. DegP125 is the unused win-F
sibling of DegP075 (`deg^1.25`). Replace SqDiv with DegP125 only when
`n > 16_000`. Same task count, `metric_k` stays top-3, census trio
DegDivNvSqrtWf / DegPlusDegme / DegSqrt stays, n<=16k list including SqDiv
and DegP075 is unchanged. No fourth core, no second-colour widening.

## What changed

`src/ordering/indep_first.rs` metric variant list only. Reverted after the run.

## Result

`yukon run` 300/300, status OK, score **0.794121** / fill **0.926127**.
Does not beat 0.794121 by 0.0001. Worst `order()` was not printed (table
column is `(capped)`); timing is not claimed. Not submitted.

## Why it won / lost

Null. Swapping one non-winner metric for its unused degree-power sibling on
the n>16k cores does not change any accepted permutation enough to move the
weighted geomean. Do not grid more DegP* substitutions on this list without
a new census that names a distinct n>16k winner. Do not add a fourth core
or widen second-colour to chase this band.
