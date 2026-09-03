# 0022 — Bounded subtree work

- **Date:** 2026-09-02
- **Accepted-base score:** 0.859116 → **0.852938**
- **Fill:** 0.955319 → **0.950102**
- **Status:** public pass; hidden submission pending

## Problem

Experiment 0021 scored 0.851513 on the public corpus but timed out on one hidden
matrix. Its `budget` was per stream and per block. The 32-block, two-stream,
2M-operation configuration therefore requested up to 128M search operations per
matrix. It was not the matrix-wide limit that its local timing suggested.

## Fix

The production configuration now has at most 32 ranked blocks, one stream per
block, and 1M requested operations per stream. The resulting matrix-wide search
ceiling is 32M requested operations, one quarter of the failed configuration.
All existing structure gates, exact scoring, bijection checks, and strict
improvement checks remain unchanged.

A regression test calculates the upper bound from the exact production
configuration and requires it to stay at or below 32M. Before the fix, the same
test reported 128M. The test does not use shared counters, so parallel tests
cannot change its result.

## Equal-work check

Two layouts with the same 32M requested work were measured:

| Layout | Public score | Worst local time |
|---|---:|---:|
| 32 blocks × 1 stream × 1M | **0.852938** | 0.767–0.777 s |
| 16 blocks × 2 streams × 1M | 0.853575 | 0.766 s |

The 32-block layout is better at the same work and time. It keeps the main
subtree gain, including a 0.2121 ratio on `pooling_sppc1pq`.

## Result

The complete 300-matrix Yukon run passed:

| Bucket | Accepted base | Bounded subtree work |
|---|---:|---:|
| `lt_1k` | 0.896482 | 0.896482 |
| `1k_10k` | 0.884759 | **0.8803** |
| `gt_10k` | 0.811860 | **0.7997** |
| weighted score | 0.859116 | **0.852938** |

The candidate gives up 0.001425 of the failed public score but keeps a 0.006178
gain over the accepted local base. The full candidate test suite passed.

## Links

- Failed predecessor: [0021 exact subtree refinement](0021-exact-subtree-refinement.md)
- Technique: [best-of portfolio](../techniques/best-of-portfolio.md)
