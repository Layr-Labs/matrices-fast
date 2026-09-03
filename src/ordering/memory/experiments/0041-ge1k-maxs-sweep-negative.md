# 0041 — The `n >= 1000` `max_s` lead is CLOSED: 384 is already optimal

**Date:** 2026-09-03
**Base:** frontier `344a5d2` (submission `26932eba`, board 0.874601), dev 0.849487
**Result:** no change shipped; the strongest open lead from
[0040](0040-lt1k-block-size-sweep.md) measured NEGATIVE for the large buckets.
Base re-measured at 0.849487 (unchanged, receipt `results.tsv` row
`1788459754`).

## Where this came from

0040's "Next" section named `SUBTREE_CFG.max_s = 384` for the `n >= 1000`
chain as "the strongest open lead in the repo right now" — lt_1k had just won
−0.001045 by DROPPING max_s 768→256, and `1k_10k`+`gt_10k` carry 0.70 of the
weight. The obstacle was cost: no bucket-scoped probe existed for the two
large buckets, so the sweep needed a new instrument.

## Instrument

`probe_ge1k` (new, test-only, ~64 s): scores only the 153 `n >= 1000`
matrices, prints both bucket geomeans, the weighted partial
`0.30*mid + 0.40*big` (moves by exactly the score delta while lt_1k is
untouched), and dumps per-matrix ROWs so the 0040 robustness bar
(disjoint halves + drop-top-3) can be computed offline. Verified: the
baseline partial reproduces the full-run bucket numbers exactly
(0.875176 / 0.796916).

## The sweep

| change | 1k_10k | gt_10k | weighted partial | verdict |
|---|---|---|---|---|
| base (max_s 384) | 0.875176 | 0.796916 | 0.581319 | — |
| SUBTREE_CFG.max_s 256 | 0.874198 | **0.799074** | **0.581889** | REGRESSION |
| round-2 chain max_s 320 | 0.874813 | 0.797115 | 0.581290 | fails robustness |

`max_s` flips sign between buckets: what helps `1k_10k` (−0.000978) hurts
`gt_10k` (+0.002158) more, and gt_10k carries 0.40. Per-matrix, the 256
variant loses `pooling_sppc1pq` +6.7% and `pooling_sppc3pq` +5.6% — the
pooling family's blocks are larger than 256 and the chain stops reaching
them.

Round-2 max_s 320 is nominally better (−0.000029) but FAILS the 0004 bar:
drop-top-3 flips it to +0.000102 and its gains sit on 3 matrices
(`chimera_mgw-c16-2031-01` −2.5%, `chimera_selby-c16-01` −2.3%,
`crudeoil_pooling_ct3` −0.5%). Half A/B also disagree (−0.000101/+0.000044).
Reverted.

## Conclusion

`SUBTREE_CFG.max_s = 384` is a measured local optimum for the `n >= 1000`
chain, not an oversight. The lt_1k "smaller is better" reading does NOT
transfer upward. Do not re-sweep max_s on this axis; remaining sign-flipping
knobs would have to be swept per-bucket with the chain split by `n`, which
the code does not currently structure (rounds 2-5 share `subtree_cfg_for`).

## Next

- 21 `1k_10k` ties and 10 `gt_10k` ties remain (31 of 153) — the same
  untouched-upside list 0040 flagged for lt_1k (55 ties there).
- `max_blocks`/`budget` were NOT swept at fixed 32M ceiling for n >= 1000;
  max_s is closed but the budget-shape axis is not.
