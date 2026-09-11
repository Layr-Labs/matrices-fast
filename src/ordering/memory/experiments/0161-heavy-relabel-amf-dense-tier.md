# 0161 — Delete the heavy relabelled-AMF dense sub-tier and the hole beside it

- **Date:** 2026-09-11
- **Score:** 0.792573 → **0.792573** (no change; 0 of 300 rows move)
- **Status:** compliance re-derivation with zero dev effect

## Hypothesis

The heavy-tier relabelled-AMF multistart ran on two `nnz` sub-tiers with a
**20 000-wide hole** between them:

```rust
let sparse = (200_000..400_000).contains(&nnz) && nnz <= 6 * n;   // alpha 5
let dense  = (420_000..700_000).contains(&nnz) && nnz >  6 * n;   // alpha 2.5
```

The constants' own doc comment says what the hole is for: *"the gap holds
kissing2-class rows where 24 seeds all return 1.0000"*. Against the corpus, in
`Pattern::nnz()` units, that is accurate — `kissing2` is `n = 20 772`,
`nnz = 401 024`, dense, and the `400_000` edge sits **1 024 below it**. So the
hole is a hole cut around one named corpus matrix, which is the clearest case of
instance special-casing in a structural costume anywhere in this tree. A gap has
no cost reading at all.

Deleting the dense sub-tier removes the hole, the `420_000` edge and the
`700_000` ceiling in one hunk, leaves a ladder that is monotone non-increasing in
`nnz`, and removes work.

## What changed

`src/ordering/mod.rs`: the `dense` sub-tier and its two constants are deleted;
the block runs on sparse patterns only, at α5.

## Result

**Dev score unchanged to the last digit** and **0 of 300 rows move** — exact flop
counts identical. Two dev rows lose the passes they were queueing:
`pooling_sppc1pq` (n = 14 100, nnz = 477 680) and `pooling_sppb5pq`
(n = 18 529, nnz = 674 470), 2–3 relabelled-AMF passes each. Neither of them was
winning with one.

## Why it won / lost

Neither on score. It is a compliance re-derivation that is free on dev and
strictly work-reducing at the call site: the predicate is pointwise ≤ the one it
replaces, so no pattern gains a pass.

## Caveats

`HEAVY_RELABEL_AMF_SPARSE_MAX_NNZ = 400_000` survives. It is now the point where
the ladder reaches zero rather than the lower lip of a hole, which is a weaker
objection, but it is still a number and not a law.

## Links

- [0158](0158-heavy-metric-density-law.md) — the same treatment one block earlier
