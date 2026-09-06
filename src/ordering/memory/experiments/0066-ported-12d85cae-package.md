# 0066 — Ported package: the small-graph and margin delta of submission `12d85cae` (jtaroreh, PR #171)

- **Date:** 2026-09-05
- **Base:** S6 ([0064](0064-multi-depth-prefixes.md) + [0065](0065-core-recursion-on-the-k3-core.md)) on the `2a28517` tree
- **Source:** github.com/Layr-Labs/matrices-fast/pull/171, head `08a99353`, graded 0.867023 vs the 0.867211 crown (−1.9 bip); code delta +52 / −22 lines in `mod.rs` and `rgreedy.rs`
- **Status:** SUBMITTED in S10b (S9b + 0067). Whole-corpus pinned min-of-3 on both corpora: dev worst 1.290 s -> 0.978 s, out-of-sample 1.115 s -> 1.157 s (one small-band row +0.042 s); zero rows of the >= 0.8 s class slower than the slower of the two graded trees (crown 2a28517, 12d85cae) by more than 0.05 s; flops identical on every row across three runs; 53 tests

## What the package does

Score side (graphs up to 1,000 nodes): a fifth randomised plateau stream, MinFill restarts 12 → 24 (nnz ≤ 5k), subtree
`min_s` 16 → 8, the below-anchor gate loosened from 4/5 to 17/20 with a second tier of 8 extra relabel tickets (small graphs
always get the tier). Time side: relabel restarts floored at 4 instead of 8 on sparse graphs with n ≥ 40k and nnz ≤ 200k,
AMF restarts capped at 8 for n ≥ 10k and nnz ≥ 100k, the 384-node pooling window removed from the refiner (its author
traced three earlier remote kills to it).

## What is NOT carried

Its outer density gate on the robust AMD envelope (`nnz <= 12 n || nnz <= 150k`). On this tree the envelope is already
gated per variant above 150k nnz (0064); the outer gate switches all five variants off on dense mid-size graphs and costs
arki0005 (n 7,522, nnz 255k, density 34) 15.4% out-of-sample. Dropping that one line restores it (+0.2%).

## Measured on S6 (pinned to two cores)

Their package alone, judged by our two corpora: dev −1.5 bip (their claim −1.55), out-of-sample ≈ 0 (38 rows better, 11
worse by ≤ 1%: small-graph rows moved both ways by the `min_s` change; the extra tickets and streams are floors). Time:
+0.06–0.3 s on small and medium rows (0.5–0.9 s class), −0.2 s on the crown's slowest class (dev worst 1.289 → 1.081 s in
the ship check). The grader carried exactly that small-row cost in `12d85cae`, so the time law for this package is: no row
slower than the slower of the two graded trees (crown, theirs) by more than 0.05 s, on min-of-N — measured: their tree on this box, pinned, worst 1.508 s dev / 1.371 s out-of-sample (dev 0.838933, out-of-sample 0.856672, i.e. +2.5 bip WORSE than the crown out-of-sample - the package is a dev-side gain that the hidden grader nevertheless paid 1.9 bip for); S9b violates that envelope on 0 rows of either corpus.

| corpus | crown 2a28517 | S6 | S9b (= S6 + this package) | better / worse vs crown | worst order() (single pinned run) |
|---|---|---|---|---|---|
| dev (300) | 0.839063 | 0.835979 | **0.835848** | 23 / 4 (max +0.6%) | 1.289 s -> 1.081 s |
| out-of-sample (591) | 0.856455 | 0.854948 | **0.854961** | 38 / 11 (max +0.99%) | 1.115 s -> 1.157 s |

Pinned interleaved min-of-3 on the 32 slowest rows: crown worst 1.297 s -> S9b 1.107 s, no row slower by more than 0.05 s
(arki0013 1.297 -> 0.826, gams05 -0.42, unitcommit -0.28, crudeoil_lee4_10 -0.26). 53 tests.

## Links

- [0064](0064-multi-depth-prefixes.md), [0065](0065-core-recursion-on-the-k3-core.md); diff kept at `matrices_mage/competitors/pr171_jtaroreh_12d85cae.diff`
