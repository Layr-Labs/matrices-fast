# 0149 — x9 set on the lee4_10 band, paid by caps 15/5 and SqDiv

- **Date:** 2026-09-09
- **Base:** tip `be9bae1` official local **0.794121** / fill **0.926126**. Claim worst order() 1.105 s. Hidden best 0.843577 (273a). Promote bar 0.843476.
- **Score:** official local **0.794104** / fill **0.9261** (printed). Delta **-0.000017** vs 0.794121. gt_10k flop 0.6898 unchanged; gt_10k fill 0.8853 -> 0.8852.
- **Status:** negative. Change reverted. Not submitted.

## Hypothesis

0145 census: crudeoil_lee4_10 (n=17809, nnz=120632) best lift was second-colour
x9 with DegDivNvSqrtWf. 273a excludes all second-colour sets when nnz>80k, so
lee4_09/10 stay out. Adding all four x-sets killed hidden timing (fe871f1)
because extra lifts plus metric walks, not the AMD caps.

Substitute, not an added pass, only when
`n > 16_000 && n <= 18_000 && nnz > 80_000 && nnz <= 140_000`:

- Drop degree-greedy caps 15 and 5 (keep g_inf, cap 9, cap 3).
- Add exactly one second-colour set: `greedy_independent_set_excluding(sp, 9, &g_inf)`.
- That x9 core gets AMD plus only DegDivNvSqrtWf, never the six-metric list.
- Existing metric cores on the same band drop SqDiv (known no-win) so the
  added DegDivNvSqrtWf is paid. Net metric task count does not rise.
- Existing `n <= 18_000 && nnz <= 80_000` x-set block, metric_k, the variant
  list, and METIS gates are unchanged. No x-inf / x15 / x5.

## What changed

`src/ordering/indep_first.rs` only. Band gate on n and nnz. Deterministic.
Reverted after the run.

## Result

`yukon run` 300/300, score **0.794104** / fill **0.9261**. Not a compile failure.
One row moved: **crudeoil_lee4_10** 0.639 -> 0.637 (flops 188270853 -> 187748239).
lee4_09 and lee4_06 unchanged. No other n>16k row moved (31 same). 1 better / 0 worse.
Delta vs local tip is **0.000017**, below the 0.0001 submit bar. Worst `order()`
was not printed (table column is `(capped)`); timing is not claimed. Not submitted.

## Why it won / lost

The paid x9 DegDivNvSqrtWf does move lee4_10, but only ~0.3% relative (0.002 ratio),
which does not move the weighted geomean by 1 bip. Do not add x-inf/x15/x5 on this
band, and do not un-pay the metric list to chase the remaining lee4_10 gap. The
substituted caps 15/5 were not the lee4_10 winner; the census metric on x9 is real
but too small on the current 273a path.
