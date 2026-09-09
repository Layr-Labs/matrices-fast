# 0186 — heavy-tier dense relabel AMF α 2.5 → 0.5

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843389)
- **Score:** 0.793834 → **0.793834** (0.00)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

Same-count only: in the HEAVY-TIER RELABELLED-AMF block, when
dense (else branch / not sparse — 420k≤nnz<700k and nnz>6n), use
dense_alpha 0.5 instead of 2.5. Sparse branch stays 5.0 (0185 flat).
Pass count, budget, max passes, and seed schedule unchanged. Light
`amf_alphas` and `REDUCE_ALPHAS` untouched. Public dense class:
pooling_sppc1pq, pooling_sppb5pq.

Submit only if score ≤0.793734, or (≥0.00008 better AND 0 worse),
and more than one dense-heavy-relabel row moves, and no row
anywhere is worse. Drop if worst order() rises. Keep only if more
than one dense-heavy-relabel row moves, local beats 0.793834, no
worse rows, and worst order() does not rise.

## What changed

`src/ordering/mod.rs` only, HEAVY-TIER RELABELLED-AMF block.
Reverted after the miss. HEAD remains `74b6ccd`.

- old: `let alpha = if sparse { 5.0 } else { 2.5 };`
- tried: `let alpha = if sparse { 5.0 } else { 0.5 };`
- restored: `let alpha = if sparse { 5.0 } else { 2.5 };`

`HEAVY_RELABEL_AMF_*` nnz gates, budget, max passes, seed loop
unchanged. Light `amf_alphas` stays `[5.0, 2.0, -1.0, 1.0, 16.0]`.
`REDUCE_ALPHAS` stays `[0.5, 2.5, 5.0, 10.0]`. mid_k2 depth==2.
Sparse heavy-relabel alpha stays 5.0. No extrarelbl. probe.rs left
untouched (trailing blank line only). Dense nnz window unchanged
(420k–700k).

## Result

Yukon local **0.793834** vs tip **0.793834** (`/tmp/yukon-run-0186.log`).
Fill 0.9258 → 0.9258 (printed). Buckets: lt_1k 0.8875 / 0.9599,
1k_10k 0.8398 / 0.9468, gt_10k 0.6891 / 0.8844 — printed geomeans
unchanged at 4 d.p.

Flops-exact vs tip (`/tmp/yukon-run-tip-74b6ccd.log`): **0 better / 0 worse / 300 same**.

Dense-heavy class (420k≤nnz<700k, nnz>6n): **2 rows, both tip-flat**

- pooling_sppc1pq (n=14100, nnz=477680): flops 136686008 → 136686008
- pooling_sppb5pq (n=18529, nnz=674470): flops 213571545 → 213571545

Zero dense-heavy movers. Score did not beat 0.793834. Short of
score ≤0.793734 and short of ≥0.00008 better with 0 worse. Not
submitted. Dense alpha restored to 2.5. Did not chase 00d74e4 /
fd5c571 / 6946637.

## Why it won / lost

α0.5 on the dense heavy-relabel path did not displace the incumbent
on either public dense-class row. The α2.5 tickets were not unique
mins that α0.5 could undercut on these seeds/passes, or both alphas
lose to an earlier portfolio candidate. Same-count dense α swap is
closed tip-flat; do not retry this exact change.

## Follow-ups

- Closed: dense heavy-relabel α2.5→0.5 tip-flat (with sparse α5.0
  already tip-flat from 0185). Do not queue further same-count
  heavy-relabel α swaps on this arm without a new seed/pass lever.

## Links

- Prior: [0185](0185-heavy-relabel-sparse-alpha-05.md) (sparse α5.0→0.5 tip-flat)
