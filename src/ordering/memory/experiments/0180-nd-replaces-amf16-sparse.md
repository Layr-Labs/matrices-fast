# 0180 — sparse nd_order replaces AMF α16

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843389)
- **Score:** 0.793834 → **0.793834** (0.00)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

Hand-rolled `nd_order` exists but `ND_MAX_N` is 1000, so it never sees the
sparse 1k–25k exact-tie class (emfl/squfl/supplychain and the public rows
with `nnz≤4n`). Same-count replacement: skip the AMF sweep α16 ticket only
on that band and spend that slot on one `nd_order`. `ND_MAX_N` stays 1000
so denser n>1000 with `nnz<130k` stay out.

Precondition: no public row in `1k<n≤25k && nnz≤4n && nnz<130k` has AMF
α16 as a unique min. Proved on all 74 public band rows: α16 flop-ties AMF
α1 or AMF α−1, or AMD is already strictly better (`unique=0`).

## What changed

`src/ordering/mod.rs` only. Reverted after the miss. HEAD remains `74b6ccd`.
475a hunks kept.

- `nd_ext = n > 1_000 && n <= 25_000 && nnz <= 4 * n && nnz < 130_000`
- AMF sweep `[1.0, 16.0, -1.0]` skipped only `dense_alpha == 16.0` when `nd_ext`; α1 and α−1 stayed; outside the band the loop was unchanged
- one extra `nd_order` consider when `nd_ext`; existing `n < ND_MAX_N && nnz < ND_MAX_NNZ` consider unchanged
- no `ndfm_order` / `rcm_order` / `sloan_order` on `nd_ext`
- `ND_MAX_N`, RCM, Sloan, NDFM, pair-ext, mid_force, FINAL_FIVE, SUBTREE_CHAIN_MAX_N untouched

## Result

Yukon local **0.793834** vs tip **0.793834** (`/tmp/yukon-run-0180.log`). Fill 0.925772 unchanged. Buckets: lt_1k 0.8875 / 0.9599, 1k_10k 0.8398 / 0.9468, gt_10k 0.6891 / 0.8844 — all unchanged.

Flops-exact vs tip (`/tmp/yukon-run-tip-74b6ccd.log`): **0 better / 0 worse / 300 same**.

`nd_ext` class (74 public rows): **0 moved**.

Short of score `< 0.793734` (got 0.793834, delta 0). Zero class movers. Not submitted. AMF α16 restored in the sweep; `nd_order` gate restored to `n < ND_MAX_N && nnz < ND_MAX_NNZ`.

## Why it won / lost

The sparse 1k–25k class is already at a flop count that hand-rolled nested dissection does not beat. On every public band row the AMF α16 ticket was already redundant (flop-tie with α1 or α−1, or worse than AMD), and inserting `nd_order` in its place did not change any final flop count. Leave the sweep and `ND_MAX_N` alone.
