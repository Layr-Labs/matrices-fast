# 0080: terminal PEO re-extraction above the 30k/180k gate, under a work ledger

- **Model:** Claude Fable 5.1
- **Harness:** Claude Code
- **Date:** 2026-09-06
- **Base:** `386b89b` (submission `4bbe2d07`, hidden 0.860113)
- **Status:** measured, all gates green; official pending

## The gap

[0077](0077-terminal-peo-re-extraction.md) introduced terminal PEO re-extraction and
[0079](0079-peo-round-chain-depth.md) let its strict-gain chain run to eight rounds. Both sit behind
`16 <= n <= 30_000 && nnz <= 180_000`, and the reconstruction refuses a factor above
`Lnnz > 300_000`. The bounded completion watcher of `completion::refine_limited` shares the same
dimension and nonzero limits.

Every matrix outside that gate therefore reaches the end of `leader_order` with whatever completion
the partitioner, AMD, AMF and reduction passes left, and receives no completion cleanup of any kind.
On the two corpora used here that is 17 of 300 development rows and 22 of 591 held-out rows, and
they sit almost entirely in the `gt_10k` bucket that carries weight 0.40. Measured on the base tree,
none of the seven submissions promoted between `5734aba` and `386b89b` moved any of them.

## Why the gain is there, and why it is bounded

For a chordal completion `H`, the objective `sum_v (1 + d(v))^2` equals `n + 3|E(H)| + 2T(H)`. It is
a function of `H` alone, so every perfect elimination order of `H` scores the same. Eliminating the
original graph `G` under a PEO `q` of `H` produces a completion `K` contained in `H`, so a
re-extracted candidate is never worse, and is strictly better exactly when the incumbent's own
completion is not a minimal triangulation along its own order. Nothing in that argument refers to
the size of the graph. Cost is what stopped at the gate, so cost is what this experiment bounds.

## The change

`peo_extract::candidates_bounded` takes the structural limits as parameters and `candidates` calls it
with the existing constants, so the inside-gate path is unchanged. A second branch at the end of
`leader_order` runs the same chain for rows outside the gate, under a per-matrix work ledger:

```
cost(round) = 5 * (n + nnz) + Lnnz
ledger      = 2_500_000 units
```

Per-round work is one reconstruction, two linear bucket-MCS passes and two exact scores, each linear
in `n + nnz + Lnnz`. Regressing measured per-round milliseconds on those two terms over 35 sampled
rounds gives `89.7 ms per M(n + nnz) + 12.9 ms per M(Lnnz)`, a ratio of about 7:1; the shipped law
uses 5:1, which is the same shape and slightly conservative for factor-heavy rows. Charging each
round against a fixed allowance bounds the added time per matrix by structure alone, for inputs that
appear in no local corpus as much as for those that do. A matrix whose first round already exceeds
the allowance receives no rounds and is left exactly as the base leaves it.

Every gate is structural: dimension, nonzeros, factor nonzeros. No matrix identity, no clock, no
environment variable and no filesystem input reaches the production path. Acceptance is unchanged:
a candidate must be a bijection and must strictly lower the exact score, with the incumbent's score
re-derived from its own column counts each round.

## Measurement

| corpus | crown `386b89b` | candidate | rows better | rows worse |
| --- | --- | --- | --- | --- |
| development (300) | 0.829057 | **0.827794** | 5 | 0 |
| held-out (591) | 0.852396 | **0.852280** | 5 | 0 |

Interleaved min-of-3 on the 32 slowest rows, pinned to two cores on an idle box: **no row is slower
by more than 0.05 s**, and the worst call is **1.047 s against the base 1.061 s**. Development gains:
nuclear104 12.79%, arki0013 4.83%, maxcsp-ehi-85-297-71 0.23%, and two smaller. Held-out gains:
arki0005 1.23%, arki0006 1.21%, pooling_sppb2stp 0.44%, and two smaller. All 68 tests pass, and the
trusted 300-case run completes with bijection, determinism and watchdog checks green.

## Choosing the allowance

The allowance was swept on one build with the ledger read from the environment (a measurement build
only; the shipped constant is compiled in). Lower is better.

| ledger | dev | held-out | verdict |
| --- | --- | --- | --- |
| unbounded | 0.826332 | 0.851620 | worst call 1.958 s / 1.841 s -- far outside the envelope |
| 10,000,000 | 0.826613 | 0.851950 | rejected: a slow-class row +0.127 s, and one row above the base worst |
| 5,000,000 | 0.826850 | 0.852112 | rejected: a slow-class row +0.103 s |
| 4,500,000 | 0.827651 | 0.852277 | rejected on re-measurement: two boundary rows +0.066 s and +0.087 s |
| 4,000,000 | 0.827789 | 0.852277 | not re-measured |
| 3,500,000 | 0.827794 | 0.852278 | not re-measured |
| **2,500,000** | **0.827794** | **0.852280** | **accepted** |

Two allowances were rejected for time, not for score. At 5,000,000 and above, one row of the slow
class gains more than 0.05 s outright. At 4,500,000 the whole-corpus min-of-3 passed, but the
interleaved 32-row measurement on a quiet box showed two rows sitting at the 0.80 s boundary taking
a second round and gaining 0.066 s and 0.087 s. The accepted allowance gives each of those rows one
round instead of two, costing 0.0002 on development and 0.000003 on held-out while halving the added
time. The remaining difference between allowances is small because the largest single gain, 12.79%
on one row, is captured by its first round.

The chain costs less worst-case time than the base rather than more: the rows it touches are
not the rows that set the worst call, and the ledger keeps every admitted round small.

## What this does not do

The three most expensive candidates are excluded outright: their unbounded chains gained 0.4% to
2.3% each while costing 1.3 s to 1.8 s per matrix. Recovering them needs a cheaper reconstruction,
or one reconstruction shared between the watcher and the chain, rather than a larger allowance.
