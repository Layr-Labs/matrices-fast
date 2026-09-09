# 0185 — heavy-tier sparse relabel AMF α 5.0 → 0.5

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843389)
- **Score:** 0.793834 → **0.793834** (0.00)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

Same-count only: in the HEAVY-TIER RELABELLED-AMF block, when
`sparse` is true (200k≤nnz<400k and nnz≤6n), use dense_alpha 0.5
instead of 5.0. Dense branch stays 2.5. Pass count, budget, max
passes, and seed schedule unchanged. Light `amf_alphas` and
`REDUCE_ALPHAS` untouched. Public sparse class includes at least
transswitch2736spr and transswitch2383wpr.

Keep only if more than one sparse-heavy-relabel row moves, local
beats 0.793834, no worse rows, and worst order() does not rise.
Submit only if score ≤0.793734, or (≥0.00008 better AND 0 worse),
and more than one sparse-heavy-relabel row moves, and no row
anywhere is worse. Drop if worst order() rises. Do not chase
validating fd5c571 or 6946637.

## What changed

`src/ordering/mod.rs` only, HEAVY-TIER RELABELLED-AMF block.
Reverted after the miss. HEAD remains `74b6ccd`.

- old: `let alpha = if sparse { 5.0 } else { 2.5 };`
- tried: `let alpha = if sparse { 0.5 } else { 2.5 };`
- restored: `let alpha = if sparse { 5.0 } else { 2.5 };`

`HEAVY_RELABEL_AMF_*` nnz gates, budget, max passes, seed loop
unchanged. Light `amf_alphas` stays `[5.0, 2.0, -1.0, 1.0, 16.0]`.
`REDUCE_ALPHAS` stays `[0.5, 2.5, 5.0, 10.0]`. mid_k2 depth==2.
No extrarelbl. probe.rs left untouched (trailing blank line only).

## Result

Yukon local **0.793834** vs tip **0.793834** (`/tmp/yukon-run-0185.log`).
Fill 0.925772 → 0.925772. Buckets: lt_1k 0.8875 / 0.9599, 1k_10k 0.8398 / 0.9468,
gt_10k 0.6891 / 0.8844 — printed geomeans unchanged at 4 d.p.

Flops-exact vs tip (`/tmp/yukon-run-tip-74b6ccd.log`): **0 better / 0 worse / 300 same**.

Sparse-heavy class (200k≤nnz<400k, nnz≤6n): **2 rows, both tip-flat**

- transswitch2383wpr (n=59853, nnz=277562): flops 3861509 → 3861509
- transswitch2736spr (n=69651, nnz=331010): flops 7281507 → 7281507

Zero sparse-heavy movers. Score did not beat 0.793834. Short of
score ≤0.793734 and short of ≥0.00008 better with 0 worse. Not
submitted. Sparse alpha restored to 5.0.

## Why it won / lost

α0.5 on the sparse heavy-relabel path did not displace the incumbent
on either public sparse-class row. The α5.0 tickets were not unique
mins that α0.5 could undercut on these seeds/passes, or both alphas
lose to an earlier portfolio candidate. Same-count sparse α swap is
closed tip-flat; do not retry this exact change.
