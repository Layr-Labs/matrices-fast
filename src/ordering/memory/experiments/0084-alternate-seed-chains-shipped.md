# A second starting ordering, priced by measured round cost

## Baseline

Promoted tip `a2614af` (submission `1a4183b7`, hidden 0.859834). Local dev
baseline on this host, worker rebuilt from that tree: **0.827195**, fill
0.937411 (lt_1k 0.889730 / 1k_10k 0.863415 / gt_10k 0.753129).

Two host caveats. `src/ordering` compiles only into `ssi-candidate-worker`, so
every number below follows `cargo build --release --workspace`; and this host is
slower than the runner, so per-matrix times are used as deltas against a
baseline measured the same way, never as absolute headroom.

## Why the terminal chain had stopped paying

The terminal PEO chain re-extracts a perfect elimination order of the incumbent
completion `H` and re-eliminates `G` along it. That can only shrink the
completion, and its fixed point is a **minimal** triangulation: no proper
subgraph of it that contains `G` is chordal. Instrumenting every round over the
dev corpus shows the chain is already at that fixed point almost everywhere —
502 of 652 rounds end with no gain. Consistent with that, three attempts to buy
more out of the chain all failed: quadrupling the oversize allowance moved the
score by 0.02 bip, a third traversal tie policy (FIFO buckets instead of LIFO)
unstuck a stalled chain zero times out of 652 rounds, and raising the round cap
from 8 to 32 was worth 0.39 bip.

Which minimal triangulation you land on is decided entirely by the ordering you
start from. So the lead is not more cleanup, it is a second starting point.

## The seeds are free; only the rounds cost anything

The portfolio scores about thirty orderings per matrix and keeps one. The
others are already built and already scored, and today they are dropped. The
`consider` funnel now retains the best few displaced orderings, which adds one
clone of a permutation and no ordering work at all.

Each retained seed is then run through the same chain, and the result replaces
the leader only when its exact score is strictly lower.

## Pricing the rounds honestly

The above-gate chain in the inherited tree charges a round as
`5 * (n + nnz) + Lnnz`. We measured that law rather than inherit it. Timing the
three phases of 528 real chain rounds on this corpus and fitting
`time ~ a*(n + nnz) + b*Lnnz` with no intercept gives

```
a = 0.034 us per (n + nnz)
b = 0.039 us per Lnnz        ratio 0.86 : 1
```

The two terms cost the **same** per unit; the inherited 5:1 overcharges
`(n + nnz)` by about a factor of five. The reason is visible in the phase
split: the reconstruction dominates and it is driven by the factor, not the
input. On the largest sampled round (n=17364, nnz=252910, Lnnz=3198854) the
round takes 138.2 ms, of which the reconstruction is 125.8 ms against 6.0 ms of
column counts and 6.4 ms of scoring.

So the alternate-seed chains charge `n + nnz + Lnnz` against one allowance
shared across all seeds of a matrix:

```
PEO_ALT_LEDGER    = 4_000_000 units   (about 140 ms on this host at 0.035 us/unit)
PEO_ALT_MAX_LNNZ  = 4_000_000
PEO_ALT_SEEDS     = 4 seeds, up to 8 rounds each, first-come out of the shared allowance
```

A round is paid for before it runs and the block is skipped outright when
`n + nnz` alone exceeds the allowance, so a large matrix cannot even pay the
entry fee. The allowance is a function of `n`, `nnz` and `Lnnz` only — no clock
is read, so the ordering stays a function of the pattern and two runs agree.

## Result

Full 300-matrix trusted run, baseline then candidate:

| | `a2614af` | candidate |
|---|---:|---:|
| weighted flop geomean | 0.827195 | **0.826784** |
| fill tiebreak | 0.937411 | 0.937368 |
| lt_1k | 0.889730 | 0.889730 |
| 1k_10k | 0.863415 | 0.863292 |
| gt_10k | 0.753129 | 0.752193 |

Six strict movers, zero regressions, 294 ties. Exact flop counts:

- `pooling_sppb5pq` 218002440 -> 213571545 (2.03%)
- `faclay35` 32576490 -> 32021838 (1.70%)
- `faclay30` 13086823 -> 12850431 (1.81%)
- `ringpack_20_3` 1261104 -> 1244772 (1.30%)
- `maxcsp-ehi-85-297-71` 1112735756 -> 1111065177 (0.15%)
- `syn40hfsg` 8169 -> 8162 (0.09%)

Five of the six are in the weighted buckets that were not moving at all under
the chain work of the last several submissions, and `pooling_sppb5pq` is
outside the in-gate limits entirely — it is reachable only because the
allowance, not a size constant, decides what runs.

## Timing, and the variant that was rejected for it

68 tests pass. Timing probe on this host, slowest rows first:

| build | worst rows |
|---|---|
| baseline | `multiplants_stg1b` 1.3674 s |
| **candidate (4 seeds, 4M)** | `crudeoil_lee4_10` 1.3688 s, `multiplants_stg1b` 1.3629 s, `unitcommit_200_100_1_mod_8` 1.3525 s |
| rejected variant (4 seeds, 8M) | `unitcommit_200_100_1_mod_8` **1.8237 s**, `transswitch2383wpr` 1.5845 s |

Doubling the allowance to 8M scored the same 4.1 bip and pushed the largest dev
matrix to 1.82 s against an enforced 2 s cap. It was discarded on that basis
alone: the shipped candidate holds the corpus tail exactly where the baseline
has it, and the whole point of the ledger is that its worst case is a function
of the matrix rather than of what the dev corpus happens to contain. A two-seed
4M variant was also measured, at 1.74 bip and the same timing; the shipped
four-seed version gets the extra seeds for free because they share one
allowance rather than each getting their own.

## Caveats and next

The seeds come only from candidates that flow through `consider`; the stages
that set `best_perm` directly are not represented, so the seed set is a subset
of what the portfolio actually built. Widening it means plumbing, not more
compute, and is the obvious next step. The cost law was fitted on rows with
`n <= 40_000` and `nnz <= 800_000` and should be refitted before it is used to
justify a larger allowance for wider rows.
