# 0177 — simplicial promotion on 6k–10k, pair-ext payment

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843406)
- **Score:** 0.793834 → **0.793842** (+0.08 bip)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

Terminal `simplicial_promotion` stops at `n <= 6000`. The public class
`6000 < n <= 10000`, `nnz <= 100000`, `nnz <= 24*n` has 15 rows. Raise only
`SIMPLICIAL_PROMOTION_MAX_N` to 10_000 so the existing terminal call covers
that band. Keep nnz≤100k, density ≤24, and the 64M ops budget. Do not add
a second simplicial call (post-terminal cleanup stays at n≤6000; FINAL_SIMP
stays at 6000).

Payment, same count: drop `PAIR_DESCENT_EXT` only on the intersection of
that band with the existing pair-ext predicate (`nnz<=60k` and
`max_deg*50<=n`). Pair-ext stays on for 4k<n≤6k and 10k<n≤12k.
Intersection is not empty: 11 public rows.

## What changed

`src/ordering/mod.rs` only. Reverted after the miss. HEAD remains `74b6ccd`.

- `SIMPLICIAL_PROMOTION_MAX_N`: 6_000 → 10_000. nnz, density, 64M budget unchanged.
- Post-terminal simplicial site pinned to `n <= 6_000` so the raised constant does not add a second call.
- `pair_descent_ext` false when `6_000 < n <= 10_000 && nnz <= 100_000 && nnz <= 24*n` and the existing ext predicate holds.

Replaced: pair-ext (the `pair_descent_ext` gate, including the pair-descent call it enables) on the 11-row intersection, with one terminal `simplicial_promotion` at the existing 64M budget.

## Result

Yukon local **0.793842** vs tip **0.793834** (`/tmp/yukon-run-0177.log`). Fill 0.9258. Buckets: lt_1k 0.8875 unchanged, 1k_10k 0.8398 → 0.8398, gt_10k 0.6891 unchanged.

Flops-exact vs tip: **2 better / 4 worse / 294 same**. All movers are inside the 6k–10k class.

Class `6000 < n <= 10000`, `nnz <= 100000`, `nnz <= 24*n` (15 rows): **2 better / 4 worse / 9 same**.

Better: `mpbp_07` 1248947 → 1248515 (−432), `rsyn0840m04m` 186634 → 186553 (−81).

Worse: `crudeoil_lee2_06` +4029, `rsyn0830m04m` +246, `syn40m04hfsg` +163, `rsyn0820m04m` +46.

Short of score `< 0.793734` (got +0.000008). Class has regressions. Timing not probed (bar already missed; harness prints `(capped)`). Not submitted. `mod.rs` restored.

## Why it won / lost

Dropping pair-ext to pay for one simplicial call on this band is not free: four intersection rows got worse and only two improved. Simplicial on the shipped incumbent is not a substitute for the ext pair-descent those rows already used. Do not raise `SIMPLICIAL_PROMOTION_MAX_N` past 6000 without a different payment, and do not add a second simplicial call.

## Follow-ups

- Do not repeat this MAX_N raise with a pair-ext drop.
- Do not widen FINAL_SIMP or the post-terminal simplicial site onto 6k–10k.
- Do not raise the 64M simplicial ops budget.

## Links

- Terminal simplicial promotion [0013](0013-terminal-simplicial-promotion.md)
