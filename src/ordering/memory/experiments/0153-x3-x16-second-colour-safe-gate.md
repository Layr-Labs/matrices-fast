# 0153 — x3/x16 second-colour caps on the safe nnz gate

- **Date:** 2026-09-09
- **Base:** tip `be9bae1` official local **0.794121** / fill **0.926126**. Hidden best 0.843577. Promote bar 0.843476.
- **Score:** official local **0.794121** / fill **0.926126** (exact tie). 0 movers vs tip.
- **Status:** null. Reverted to tip `be9bae1` ordering sources. Not submitted.

## Hypothesis

0145 follow-up explicitly left "x-sets at further caps (x3, x16)" untested.
The shipped second-colour block already runs under
`n <= 18_000 && nnz <= 80_000` (excludes lee4_09/10 timing band; keeps lee4_06
and lee2_06). Caps today: x-inf, x15, x9, x5.

Add **x3** and **x16** via `greedy_independent_set_excluding` on that same gate
only. These are second-colour classes at different degree caps — family-level,
structural, not one-row tickets. Admission still budget-trims / size-dedups /
refuses lifts that exceed the ledger share, so unpaid work is structurally
bounded. Do not widen the gate onto lee4_09/10. Do not retread 0147–0152
(metric-core DegP125/SqDiv swaps, paid x9 on lee4, DegSqrt on 5th AMD core,
pinene g2).

## What changed

`src/ordering/indep_first.rs` only: two extra excluding-set candidates inside
the existing second-colour gate. Deterministic. Yukon run logged to
`/tmp/yukon-run-0153.log`.

## Result

Official local **0.794121** / fill **0.926126**. Exact bit-identical to tip on
all 300 rows (0 movers). Buckets unchanged (lt_1k 0.8875 / 1k_10k 0.8398 /
gt_10k 0.6898). lee4_06 0.507, lee4_09 0.645, lee4_10 0.639 unchanged.

## Why it won / lost

Lost / null. Extra x3/x16 second-colour sets either size-deduped against the
existing x-inf/x15/x9/x5 family or their AMD phase-1 cores never beat the
incumbent under the competitive-margin + metric/METIS phase-2 portfolio. No
score movement and no regression. Reverted. Tip-clean `indep_first.rs`.

## Next leads (not started this turn)

Prefer non-retreads still open after 0147–0153:
1. **DegP125 as ADD** on a carefully paid gate that does not touch n<=16k SqDiv
   (e.g. n>16k: add DegP125, drop DegP075 — not SqDiv; 0148 was SqDiv↔DegP125
   swap = exact tie).
2. Residual-core multi-depth prefixes K∈{2,4,5} (0062/0075 open question).
3. Relabelled-AMF `dense_alpha` is already multi-α on tip (`[5,2,-1,1,16]`);
   do not treat the open-questions line as untested without re-reading tip.
