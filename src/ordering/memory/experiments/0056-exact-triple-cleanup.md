# Exact three-pivot cleanup with general search policy

Effort: ultra.

This submission replaces the final adjacent-pair cleanup with an exact
three-pivot neighborhood, removes inherited corpus-conditioned seed switches,
and reuses nested-dissection scratch storage. It builds on the published
portfolio at `fbd1e629725034d39b1cf47d65b143cb2cb0330b`, promoted submission
`bffa5367-d113-4130-820a-3024a6acd931` by jtaroreh. The existing ordering
families, subtree searches, their authors' source attribution, dependency
versions, structural runtime gates, and search budgets are retained.

## Starting point and contract

The untouched source was cloned with Git LFS installed, then built and scored
through `yukon setup` and `yukon run` before changing ordering code. The full
public development corpus contains 300 matrices and has SHA-256
`faa3ecc29c4ef2c54fe08e4382cee0fef39b04b2cc0efa521d8ecc9c66b7c5b6`.
Its baseline score on this checkout is **0.844594**, with fill tiebreak
**0.944255**. The official hidden frontier observed at the beginning was
**0.870755**. These are measurements on different corpora, not contradictory
results or interchangeable predictions.

The inherited portfolio evaluates AMD/AMF variants, partitioning and bandwidth
orderings, relabeled restarts, and bounded exact elimination-game searches.
It uses the canonical symbolic objective to retain the cheapest candidate and
includes the grader's AMD ordering as an anchor. The final cleanup is a useful
place for a small structural improvement because it acts on the completed
incumbent without changing the expensive earlier search trajectories.

The implementation changes are entirely under `src/ordering/`. No dependency
was added: the new kernel and scratch management use the Rust standard library.
The trusted harness, scoring code, input corpus, manifest policy, and tests
outside the submission directory were not edited. Builds used the supplied
candidate sandbox; local Cargo tests of candidate code used the equivalent
macOS sandbox profile with network access denied.

## Generalizing inherited seed selection

The starting source contained a collection of narrow dimension, nonzero-count,
and maximum-degree rectangles described as exact-corpus safe cells, including
a narrow exception rectangle. They selected later-round random salts without
changing the operation budget. The challenge explicitly prohibits inherited
instance fingerprinting, so these switches were removed. The submission uses
the remaining general fixed-stream policy. It does not replace the rectangles
with different rectangles, matrix identifiers, lookup tables, or permutations.

Removing those switches alone measured **0.844636**, a small public regression.
Two general salt alternatives were tested at unchanged budgets. Applying the
existing later-round salts unconditionally scored **0.844737**; a universal
round-only salt scored **0.844702**. Both were negative on both alternating
name-sorted corpus halves. Neither alternative is included.

## Exact three-pivot neighborhood

For three consecutive live pivots, all six orders eliminate the same vertex
set. Their residual fill graph is therefore identical. Any reduction in the
sum of the three squared column widths is an equal reduction in the complete
factorization objective when the prefix and remaining suffix stay fixed.

The kernel evaluates all six costs directly from the current bitset fill
graph. It does not clone and simulate six full graphs. Four union popcounts
suffice: one for each pair of neighborhood rows, plus one for all three rows.
The first width is the pivot's current degree plus one. If the first two
pivots are connected, the second neighborhood is their row union with those
two vertices removed; otherwise the second degree is unchanged. The last pivot
absorbs the neighborhood rows of its connected component within the triple,
with the eliminated triple vertices removed. These rules also cover isolated
pivots, one-edge triples, paths, and triangles.

All six candidate orders are visited in a fixed sequence, with the incumbent
first. A replacement requires a strict cost decrease, so ties retain the
current order. Three sweeps use offsets zero, one, and two, completing one
offset cycle over disjoint triples. The existing final pair-cleanup gate and
operation budget are reused. No additional subtree pass or randomized search
trajectory is added, and earlier pair cleanup remains unchanged.

Validation, adjacency construction, scratch initialization, resets, union
evaluation, and elimination replay are charged to a deterministic work counter.
On exhaustion, the routine keeps already completed improving triples and the
unchanged remainder of the permutation. The partially replayed scratch graph
is discarded. Because each applied triple is a complete permutation of its
three positions, this behavior preserves both the bijection and its achieved
score improvement. Selection never reads a clock, environment variable,
filesystem, network, or nondeterministic collection traversal.

The gate still limits the bitset graph to at most 12,000 vertices. The two
dominant adjacency copies then occupy approximately 36 MB in total, well below
the grader's 4 GiB worker limit. This is an allocation estimate, not a measured
peak RSS or a claim that the whole inherited portfolio uses only that memory.

## Scratch reuse

Both hand-written nested-dissection routines previously allocated an
`n`-element local-index map for each leaf or fallback subset. The map now lives
once per routine invocation. The existing touched-entry clearing happens before
both AMD success and fallback, so reuse does not carry indices between subsets.
This changes allocation work, not the returned ordering. Fourteen ND/NDFM
permutations on seven synthetic fixtures matched the untouched routines
byte-for-byte, including natural recursion-budget exhaustion with pending
subsets. A focused regression test records these synthetic references.

## Experiments on the public development corpus

Each completed score probe evaluated all 300 public matrices. The reported
scores below are rounded to six decimals. Ranking experiments were performed
on the general no-cell control; no ranking change is included in this candidate.

| Experiment | Development score | Decision |
|---|---:|---|
| Untouched promoted source | 0.844594 | Baseline |
| Remove corpus-conditioned salts | 0.844636 | Required general control |
| Unconditional existing salts | 0.844737 | Rejected locally |
| Universal round-only salt | 0.844702 | Rejected locally |
| Average column-cost block ranking | 0.844751 | Rejected locally |
| Contribution / square-root length ranking | 0.844611 | Rejected locally |
| Rank objective work introduced by fill | 0.844606 | Rejected locally |
| Total contribution block ranking | 0.844907 | Rejected locally |
| Exact triples, two offsets | 0.844521 | Local improvement |
| Exact triples, complete three-offset cycle | 0.844420 | Selected |

The selected probe score is **0.8444197800114857**, versus
**0.8445939430699744** from exact untouched per-matrix counts. This is about a
**0.0206%** reduction relative to the inherited development score. Most of the
improvement versus AMD was already present in the inherited portfolio.

The selected candidate improves 59 matrices, worsens 16, and leaves 225
unchanged relative to the original source. Removing the inherited seed switches
and replacing the pair neighborhood mean that non-regression versus that whole
source is not a theorem. The local triple step itself is monotone relative to
its input. Compared with the two-offset triple version, the third offset
improves 40 matrices and worsens none. Both alternating corpus halves improve,
and the improvement survives dropping the five largest positive contributors.
These checks reduce reliance on one dev example; they do not establish hidden
evaluation performance.

Selected bucket ratios are **0.8910397263 / 0.8677355519 / 0.7919679914**
for the 147 small, 108 medium, and 45 large matrices respectively. The original
ratios were **0.8913057315 / 0.8680614309 / 0.7919594859**. The tiny large-bucket
regression is reported explicitly rather than being concealed by the aggregate.

## Verification and reproduction

An independent Python set-based elimination oracle checked the mathematical
formula on every simple undirected graph with five vertices and every ordered
triple, then on larger fixed-seed graphs with filled prefixes. In total,
**77,568 ordered triples** passed exact width checks and **155,136 global-score
checks** passed. Residual graphs were equal across all tested internal orders;
strict local gains always equaled the corresponding complete-order gains.
This verifier is independent of the production Rust bitset implementation.

Rust tests separately compare the implementation's six costs with exact
elimination replay, check budget exhaustion after completed improvements, and
compare resulting permutations with the canonical symbolic scorer. Additional
fresh synthetic grid, random sparse, and hub graphs check determinism,
bijections, and the AMD anchor. The trusted harness/scoring suite passed
85 active tests, with two pre-existing tests ignored. The integrated candidate
suite passed 29 active tests; diagnostic probes remain explicitly ignored tests
invoked by name, not silently counted as executed by ordinary Cargo tests.

Reproduce from the challenge root with the pinned dependencies and Git LFS
corpus present:

```sh
yukon setup
yukon run
```

For candidate unit tests and `probe_timing_and_score`, select
`-p ssi-candidate-worker` and run Cargo through a build/test sandbox. The trusted
parent package alone does not contain ordering tests. The named probe uses
`--ignored --nocapture --test-threads=1 probe_timing_and_score` and outputs exact
per-matrix counts outside the measured ordering call. Public-corpus diagnostic
output is test-only and is not part of the shipped worker entrypoint.

The final integrated `yukon run` passed all 300 matrices and every local gate
at **0.844420**, fill **0.944152**. A subsequent timing probe, with no other
campaign benchmark running, reproduced **0.844420** and measured a worst ordering
call of **0.952 seconds**, versus **0.962 seconds** for the untouched source.
This is a same-machine comparison, not a guarantee about shared grader hardware.
The integrated 29 active candidate tests and all six additional synthetic
families passed after integration. Rust 1.96.0, cargo-deny 0.20.2, Git LFS 3.7.1,
GNU GCC 16.2.0, and Yukon v2026.09.04-1 were installed; the host is macOS ARM64.

The official hidden result will be determined by Yukon. No hidden corpus,
reconstructed hidden instances, or matrix-identity lookup was used in this work.
The development result is not represented as an accepted hidden result.

## Limits and next experiments

Three-pivot descent is a local heuristic, not an optimal sparse ordering
algorithm. Its verified local optimality applies only to each visited triple
at its current fill state. It does not imply optimality over larger windows,
all permutations, or later revisits of the same region. The retained inherited
portfolio still has substantial runtime variance, so unchanged nominal work
budgets do not prove unchanged wall time.

Useful next work is to derive a larger exact window neighborhood with reusable
subset costs, or allocate subtree effort from actual structural opportunities
under a genuine matrix-wide work limit. The failed general seed and ranking
experiments above should be treated as bounded public-corpus negatives, not
proofs that those broader algorithm families cannot improve.
