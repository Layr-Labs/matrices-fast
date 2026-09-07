# 0077 — Third stream on the sub-1k subtree retry

- **Date:** 2026-09-06
- **Score:** 0.827195 → 0.827195 (exact tie; fill 0.937411 both)
- **Status:** NEGATIVE locally; reverted.

## Hypothesis

Open-questions `lt_1k` lead: 55 ties remain in-bucket and only one
reallocation was ever tested. A third diversified stream on the n < 1000
retry ticket costs one more 1M budget per block on cheap small graphs.

## What changed

`src/ordering/mod.rs` retry ticket: `cfg1.streams = 2` → `3` for `n < 1_000`
only (plus comment). Same gates, same seeds otherwise, strict `<` admission.

## Result

Full trusted 300-matrix run: byte-identical score to the base, all buckets
unchanged. Zero movers — extends 0056 ("extra search does not move ties") to
below-anchor smalls on this corpus: there is nothing left for a third
trajectory to find there.

## Follow-ups

- Do not add streams to the sub-1k retry; close this branch of the `lt_1k`
  lead. Remaining `lt_1k` headroom, if any, is in max_blocks/budget/max_s
  reallocation within the 32M ceiling, not in stream count.
- Next: residual-core multi-depth prefixes K ∈ {2,4,5}, or probe_ties for a
  fresh target list.

## Links

- Research queue: [open-questions](../open-questions.md)
