# 0181 — sparse sloan_order(2,1) replaces AMF α16

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843389)
- **Score:** 0.793834 → **0.793834** (0.00)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

Hand-rolled `sloan_order` exists at `sloan_order(pattern, w1, w2)` but
`SLOAN_MAX_N` is 1000, so it never sees the sparse 1k–25k band
(`nnz≤4n && nnz<130k`). Same-count, different family from 0180: skip the
AMF sweep α16 ticket only on that band and spend that slot on one
`sloan_order(pattern, 2, 1)`. Not `(1, 2)`. Do not call `nd_order`,
`ndfm_order`, or `rcm_order` on this band. `SLOAN_MAX_N` stays 1000 so
n≤1k still tries both weights.

0180 already showed `nd_order` on this band is a flop tie (0/74). This
does not re-check the α16 unique-min question.

## What changed

`src/ordering/mod.rs` only. Reverted after the miss. HEAD remains `74b6ccd`.
475a hunks kept.

- `sloan_ext = n > 1_000 && n <= 25_000 && nnz <= 4 * n && nnz < 130_000`
- AMF sweep `[1.0, 16.0, -1.0]` skipped only `dense_alpha == 16.0` when `sloan_ext`; α1 and α−1 stayed; outside the band the loop was unchanged
- one extra `sloan_order(pattern, 2, 1)` consider when `sloan_ext`; existing `n < SLOAN_MAX_N` consider unchanged (both weights)
- no `nd_order` / `ndfm_order` / `rcm_order` on `sloan_ext`
- `SLOAN_MAX_N`, ND, RCM, NDFM, pair-ext, mid_force, FINAL_FIVE, SUBTREE_CHAIN_MAX_N untouched

## Result

Yukon local **0.793834** vs tip **0.793834** (`/tmp/yukon-run-0181.log`). Fill 0.9258 unchanged. Buckets: lt_1k 0.8875 / 0.9599, 1k_10k 0.8398 / 0.9468, gt_10k 0.6891 / 0.8844 — all unchanged.

Flops-exact vs tip (`/tmp/yukon-run-tip-74b6ccd.log`): **0 better / 0 worse / 300 same**.

`sloan_ext` class (74 public rows): **0 moved**.

Short of score `< 0.793734` (got 0.793834, delta 0). Zero class movers, so not kept. Not submitted. AMF α16 restored in the sweep; Sloan gate restored to `n < SLOAN_MAX_N && nnz < SLOAN_MAX_NNZ`. Do not queue RCM or NDFM on this same α16 payment.

## Why it won / lost

The sparse 1k–25k class is already at a flop count that hand-rolled Sloan (2,1) does not beat. Inserting `sloan_order(pattern, 2, 1)` in place of the AMF α16 ticket changed no final flop count (same 0/74 as 0180's `nd_order` on this band). Leave the sweep and `SLOAN_MAX_N` alone.
