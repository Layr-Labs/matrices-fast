# Seeding the terminal chain from the reduction and relabel stages

## Baseline

Promoted tip `4d86414` (submission `e5310ad6`, hidden 0.859573), remeasured on
this host after a clean `cargo build --release --workspace`: dev **0.826784**,
fill 0.937368, buckets lt_1k 0.889730 / 1k_10k 0.863292 / gt_10k 0.752193.

## What the previous submission established

The terminal PEO chain re-extracts a perfect elimination order of the incumbent
completion and re-eliminates along it, so its fixed point is a *minimal*
triangulation: nothing further can be removed. Instrumenting the chain showed
it is already at that fixed point almost everywhere — 502 of 652 rounds end
with no gain — which is why four separate attempts to buy more out of it (a 4x
larger oversize allowance, a third traversal tie policy, dropping either
direction, a 4x round cap) were each worth well under one basis point.

The previous submission acted on the other half of that statement: which
minimal triangulation you land on is decided by the ordering you start from. It
retained the best few orderings the portfolio builds and discards, converged
each through the same chain under one shared allowance priced by a measured
cost law, and was promoted for 2.61 basis points.

## The gap this closes

Those seeds came from exactly one place: the `consider` funnel, which is where
the ordering *portfolio* — AMD and AMF variants, METIS, KaHIP, Scotch — offers
its candidates. Two later stages build orderings by a completely different
route and are not represented at all:

- **the reduction / core-lift stages.** `core_lift::reduce` peels low-degree
  structure, orders the residual core (AMD and AMF at several dense alphas,
  and exact MinFill on small cores), then splices the core ordering back under
  the peeled prefix. Two of these run: one at the default row-degree depth and
  a bounded sequence at extra depths.
- **the independent relabel candidate** (`terminal_core_candidate`), which is
  built on a relabelled core and admitted only after every inherited pass.

Each of these produces a complete, valid, scored ordering, and each is dropped
outright the moment it fails to take the lead. They are the seeds most worth
having, because a spliced core ordering is structurally unlike anything the
portfolio produces: its elimination prefix comes from peeling, not from a
degree heuristic, so the completion it induces is different in a way that two
AMD variants never are.

This change routes all three losing paths into the same seed list through one
`keep_seed` closure. The seed list is unchanged in size and the allowance is
unchanged, so a stronger seed does not add work — it displaces a weaker one.

## Cost

Nothing is ordered that was not already ordered. `keep_seed` clones a
permutation, inserts it into a four-element list, and sorts it. The chain
rounds run under the same 4,000,000-unit shared allowance introduced with the
previous submission, charged as `n + nnz + Lnnz` per round from the fitted cost
law (0.034 us per `(n + nnz)` against 0.039 us per `Lnnz` over 528 timed
rounds; the inherited 5:1 weighting was wrong by about a factor of five). A
round is paid for before it runs and the whole block is skipped when
`n + nnz` alone exceeds the allowance.

No clock is read anywhere in the decision path — the allowance is a function of
`n`, `nnz` and `Lnnz` only — so the ordering remains a deterministic function
of the pattern and two runs of the same matrix agree exactly.

## Result

Full 300-matrix trusted run, baseline then candidate:

| | `4d86414` | candidate |
|---|---:|---:|
| weighted flop geomean | 0.826784 | **0.826728** |
| fill tiebreak | 0.937368 | 0.937346 |
| lt_1k | 0.889730 | 0.889730 |
| 1k_10k | 0.863292 | **0.863104** |
| gt_10k | 0.752193 | 0.752193 |

0.56 basis points, zero regressions, all of it in the middle bucket. That
concentration is itself the finding: the reduction path only produces a seed
that differs materially from the leader when the graph has enough peelable
structure to leave a small core but is not so small that the portfolio already
found the same ordering, and on this corpus that is a mid-size phenomenon.

68 tests pass. Timing probe against the baseline's 1.3674 s worst row:
`crudeoil_lee4_10` 1.3813 s, `multiplants_stg1b` 1.3616 s,
`unitcommit_200_100_1_mod_8` 1.3196 s. The corpus tail is where the baseline
has it, roughly 0.6 s clear of the enforced 2 s cap on this host.

## Variant rejected

Raising the seed count from four to six with the richer source was worth a
further 0.02 basis points and pushed the worst row to 1.4305 s. Not taken: more
seeds from the same families converge to nearby triangulations, so the extra
allowance buys duplicates rather than diversity. The seed *source* is not the
bottleneck; the set of triangulations the chain can reach from any
heuristic-built ordering is. That is the next thing to attack — a deliberately
perturbed seed, derived deterministically from the pattern, rather than another
ordering the same heuristics already built.

## Caveats

The gain is small and sits in one bucket, so it is worth exactly what it
measures and no more. The cost law was fitted on rows with `n <= 40_000` and
`nnz <= 800_000` and should be refitted before it is used to argue for a wider
allowance. The relabel candidate is seeded only when it exists, which is a
minority of rows; on the rest the seed list is unchanged from the previous
submission.
