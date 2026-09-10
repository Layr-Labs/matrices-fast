# 0150 — The chain above 45k: it was the geometry, not the gate

## Setup
`SUBTREE_CHAIN_MAX_N = 45_000` exists because iter463a measured the ranked chain
moving the ratio by < 0.01 on rows above it while costing 0.26-0.48 s. 0149 put
the six largest `gt_10k` graphs at 0.92-0.98 against a 0.685 bucket geomean, so
the weakest rows in the heaviest bucket are exactly the ones the chain refuses.

The cap comment already contains the reason to doubt itself: ranked blocks of
`max_s <= 384` cover a vanishing fraction of a six-figure elimination tree. That
is a statement about the geometry, not about whether the technique works there.

## Geometry sweep (`probe_huge_row_geometry`, 6 rows x 32 configs)
`max_s` in {2000, 8000, 32000, n/64}, `min_s` in {64, 512}, `max_blocks` in
{4, 16}, `budget` in {4M, 32M}, ranked, rounds 0 and 2.

`GEO_ROWS_MOVED = 4` of 6, every winner at `min_s = 64`, `max_blocks = 16`,
`budget = 32M`, and a modest `max_s` (1024-2000, i.e. ~n/64) — not the huge
blocks. Best single-pass gains: `unitcommit_200_100_1_mod_8` -0.76 %,
`transswitch2736spr` -0.35 %, `transswitch2383wpr` -0.10 %, at 0.05-0.06 s.

## Cascade (`probe_huge_row_cascade`, all 7 rows above the cap)
Re-postorder and re-rank between rounds, 6 rounds:

| row | n | nnz | ship | after | gain | passes | added |
|---|---:|---:|---:|---:|---:|---:|---:|
| `unitcommit_200_100_1_mod_8` | 146830 | 476332 | 0.976390 | 0.967019 | **0.960%** | 4 | 0.52 s |
| `transswitch2736spr` | 69651 | 331010 | 0.917860 | 0.912574 | **0.576%** | 6 | 0.50 s |
| `transswitch2383wpr` | 59853 | 277562 | 0.978512 | 0.976779 | **0.177%** | 5 | 0.44 s |
| `cont6-qq` | 120395 | 557994 | 0.696163 | — | 0 | 0 | 0.17 s |
| `faclay75` | 272878 | 1379706 | 0.940482 | — | 0 | 0 | 0.38 s |
| `gabriel10` | 244056 | 1148210 | 0.928462 | — | 0 | 0 | 0.38 s |
| `acopf_case9241pegase_qcqp` | 313068 | 1292408 | 0.973719 | — | 0 | 0 | 0.67 s |

The split is clean: the three movers are the smallest rows above the cap, and
every giant pays 0.38-0.67 s for nothing.

## Shipped
`HUGE_CHAIN_MAX_N = 150_000`, `HUGE_CHAIN_MAX_NNZ = 500_000`, 6 rounds, placed
directly after the stage-4 chain so later stages inherit the improved
incumbent. The gate is drawn around where it pays: it admits the three movers,
and both bounds independently exclude the four rows that gained nothing —
`cont6-qq` on nnz, the giants on n. Acceptance is strict-decrease per round.

## Full dev result

| | tip | with 0150 |
|---|---:|---:|
| score | 0.792300 | **0.792249** |
| fill | 0.924373 | 0.924330 |
| gt_10k | 0.685397 | **0.685270** |
| lt_1k / 1k_10k | 0.887390 / 0.839747 | unchanged |

A strict gain of 0.51 bip, entirely inside `gt_10k`, with the other two buckets
bit-identical — the gate did what it says. In-gate worst total on this box is
`unitcommit` at 0.74 s of `order()` plus 0.52 s, ~1.26 s against a 1.4 s corpus
worst, so the change does not touch the timing-critical row.

## Caveat for whoever reads this next
0.51 bip is small and it is measured on the 300-row dev corpus, not the hidden
one. The mechanism generalizes (any graph in the size band gets scaled ranked
blocks instead of none), but the size of the gain does not: it came from three
rows, and a hidden corpus with a different mix above 45k may show more or less.
The three giants prove the technique has a ceiling — above ~250k vertices no
block geometry in the sweep moved anything at all.
