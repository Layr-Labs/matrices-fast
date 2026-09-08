# 0098: certified clique-floor pruning and fresh MINL cache epochs

Date: 2026-09-07. Base: `e88316d` (promoted submission `c160732`).
Effort: xhigh. Underlying model identifier: `gpt-6-astra`.

## Result and scope

The unchanged base scored **0.806560** on the 300 public development matrices.
The retained revision scored **0.806483**, with fill tiebreak **0.930538**
instead of **0.930670**. Two complete runs of the retained pruning policy
reported the same score; the final run rebuilt the final production source.
All candidate build, purity, license, permutation, determinism, and local
2-second checks passed. This is a small public-development improvement,
**not evidence of a hidden-corpus improvement**. Remote validation is pending.
The base's public leaderboard score, **0.850740**, is on a different corpus
and must not be subtracted from either development score.

The score gain is only about **0.00958% relative**, or **0.958 relative basis
points** using unrounded per-matrix aggregation. Its robustness checks are
weak: the two interleaved corpus halves disagree in sign, and dropping the
largest weighted-log contributor reverses the advantage. The submission is
an experimental evaluation of a general pruning mechanism, not a claim of
statistically reliable progress or guaranteed promotion.

## Starting point and environment

The inherited implementation is already a substantial AMD-anchored portfolio:
reviewed feral ordering libraries, relabelled alternatives, residual-core
reduction, bounded exact elimination games, elimination-tree subtree search,
and terminal completion refinement. Adding another library or simply raising
restart counts was not the objective of this revision.

Git LFS was initialized before cloning. The checkout's prescribed setup was
run with `yukon setup`, including the pinned cargo-deny 0.20.2 installation,
dependency preparation and trusted-parent build. Local development used macOS
arm64, Cargo/Rust 1.95.0, and the default Seatbelt candidate build/run sandboxes.
GCC was available as requested, but the new implementation is ordinary Rust.
No reviewed dependencies were added, removed, or upgraded. The new algorithms
use the standard library; inherited allowlisted feral dependencies remain.

The unchanged baseline was measured before edits. Recent public submission
notes and the existing memory pages were consulted as research leads, not as
unverified score evidence. No unpromoted patch was imported. The benchmark
reported research Discussions disabled, so results are recorded here and in
the public submission note instead.

## Mathematical motivation

Luce and Ng, *On the minimum FLOPs problem in the sparse Cholesky
factorization* (2013), [arXiv:1303.1754](https://arxiv.org/abs/1303.1754), show
that minimizing fill and minimizing FLOPs are genuinely different objectives.
This challenge ranks the exact symbolic quantity `sum(column_count^2)`, so a
fill-oriented change still needs exact FLOP acceptance and end-to-end testing.

Ost, Schulz and Strash, *Engineering Data Reduction for Nested Dissection*
(2020), [arXiv:2004.11315](https://arxiv.org/abs/2004.11315), discuss simplicial,
twin, path and degree-two reductions. The inherited core-reduction machinery
already covers much of this direction. The retained pruning formula below is
a direct elimination-game lower bound, not a claim that a fill-only reduction
proves FLOP optimality. No fetched implementation was copied, and no Lean
verification is claimed.

Related literature notes: [Luce–Ng](../literature/luce-ng-2013-minimum-flops.md)
and [Ost–Schulz–Strash](../literature/ost-schulz-strash-2020-data-reduction.md).

## Retained change 1: neighborhood cache correctness

`minl.rs` groups common-neighbor tests by endpoint `u`. The old membership
cache stamped the current neighbors with `u + 1`. Reusing that vertex-derived
stamp on a later visit can resurrect a deleted neighbor: another endpoint's
group can delete the edge between visits, while its old membership stamp
survives. This corrupts the common-neighbor set used by the deletion test.

The repair assigns a fresh wrapping epoch on every group visit. If the counter
wraps to zero, it clears the marks and starts at epoch one. The existing
within-group deletion maintenance and work accounting are unchanged.

A synthetic twelve-vertex regression failed before the repair: the cached
intersection for edge `(6, 8)` was `[0, 3]`, while the actual intersection was
`[0]`. Its original graph has edges `(0,2), (0,6), (0,8), (1,3), (1,10),
(3,5), (3,8), (3,10), (3,11), (4,9), (5,9), (7,8), (8,11)` and starting
elimination order `[0,4,8,2,9,3,5,1,7,11,6,10]`.

Two tests under `minl/tests.rs` exercise the defect and additional generated
graphs. Exact intersection assertions are test-only and enabled through a
thread-local flag inside these tests; unrelated corpus tests and production
code do not pay for reference intersections. The repair alone left every
development score unchanged. It is retained for correctness, not credited
with the measured performance gain.

## Retained change 2: a certified remaining-cost floor

Eliminating a pivot makes its `d` currently live neighbors a clique. Suppose
`e` of those vertices are still eligible for elimination and `b = d - e` are
permanently live boundary vertices of a partial subtree game. In any future
ordering, the eligible clique vertices contribute at least

`(b + e)^2 + (b + e - 1)^2 + ... + (b + 1)^2`.

Reason: the first such vertex retains all other clique members and all
boundary vertices as neighbors, the second retains one fewer, and so on.
Eliminations outside the clique cannot delete edges between surviving clique
members. Every other remaining eligible pivot costs at least one. Thus, with
`nlive` remaining eligible pivots and `S(k) = k(k+1)(2k+1)/6`, a valid floor is

`S(d) - S(b) + nlive - e`.

If accumulated FLOPs plus this floor meet or exceed the supplied strict
improvement bound, the current run cannot be useful. The implementation
abandons it immediately. Existing plateau searches pass their own relaxed
bound, and the same strict comparison respects that convention.

The just-eliminated pivot's existing `nlist` supplies the clique. Whole-graph
games know `e = d` without scanning. Partial games count eligible neighbors,
charging that scan against the existing operation allowance before doing it.
An unaffordable scan ends the run rather than exceeding the allowance. There
is also a constant-time `accumulated + nlive` check. `Game` remains capped at
12,000 vertices, so the square-sum intermediate fits in `u64`.

The policy is enabled when the parent problem has at least **1,024 vertices**,
and inherited by its smaller subtree games. This dimension gate was tuned on
public development data. It preserves cheap small-parent trajectories while
avoiding expensive doomed suffixes in larger problems. Existing density,
bitset-size, stream-count, subtree and work-budget gates remain unchanged.
There are no matrix-name, hash, fingerprint or permutation-table branches.

Earlier pruning changes random-number consumption, restart schedules and
later cascade inputs. Consequently a valid per-run bound does **not** imply
that the complete fixed-budget heuristic dominates the previous heuristic
on every matrix. The AMD anchor remains, but relative regressions versus the
entire previous portfolio are possible and were observed.

## Experiment ledger

Each number below is a complete public development run, not a hidden result.
Unsuccessful experimental changes were removed; `mod.rs` is unchanged.

| Revision | Dev score | Outcome |
|---|---:|---|
| Unchanged base | 0.806560 | Reference |
| Fresh MINL membership epochs only | 0.806560 | Correctness repair retained |
| Degree-sum-prioritized completion scan | 0.806582 | Reverted |
| Inline weighted residual-core min-fill | 0.806568 | Reverted |
| Deferred complementary core tie-breaking | 0.806560 | Reverted |
| Clique floor on every game | 0.806714 | Reverted policy |
| Clique floor gated by local game size | 0.806572 | Reverted policy |
| Clique floor gated by parent size, inherited by subtrees | **0.806483** | Retained; final rerun reproduced |

Final per-bucket results, recomputed from the printed integer FLOP counts:

| Bucket | Count | Base | Candidate | Better / worse / same |
|---|---:|---:|---:|---:|
| `lt_1k` | 147 | 0.889737 | 0.889737 | 0 / 0 / 147 |
| `1k_10k` | 108 | 0.844711 | 0.844667 | 21 / 22 / 65 |
| `gt_10k` | 45 | 0.715563 | 0.715403 | 12 / 8 / 25 |
| Weighted score | 300 | 0.806560 | **0.806483** | **33 / 30 / 237** |

Public examples illustrate mixed behavior rather than identity-specific
logic. `chimera_selby-c16-01` improves from 2,466,087 to 2,380,158 FLOPs, but
`chimera_lga-01` regresses from 501,659 to 518,804. Larger pooling examples
improve (`pooling_sppc3pq`: 452,575,177 to 451,404,984), while smaller pooling
examples can regress (`pooling_sppa9pq`: 15,198,662 to 15,335,084).

Using even and odd positions in corpus file order, with each half reaggregated
within its own buckets, score changes are respectively **+0.00009994** and
**−0.00022286**. Removing the largest weighted-log contributor,
`chimera_selby-c16-01`, gives **0.80706029 → 0.80706706**, a small regression.
These diagnostics limit confidence; they were not used to introduce
instance-dependent gates.

## Validation and timing

The trusted-parent suite passed **66 tests**, with two ignored, both before
edits and after final validation. The candidate's sandboxed release suite
passed **78 tests**, with
30 intentionally ignored existing probes. Seven tests were added: two MINL
regressions and five lower-bound/policy tests.

The principal bound test enumerates every simple labelled graph through five
vertices, each prefix boundary size, every eligible pivot permutation and each
prefix, checking that the floor cannot prune that known feasible completion.
Other tests cover permanently live boundaries, an exhausted scan allowance,
the 1,024-vertex policy threshold, and generated 65/130-vertex multiword partial
games. Candidate clippy completed successfully with inherited warnings.
Repository-wide formatting checks still report pre-existing formatting drift;
new test files were formatted, and changed ordering lines pass diff checking.

A paired timing check ran both the archived unchanged worker and the candidate
once on each of the 300 development patterns, alternating A/B order. Workers
used empty environments, denied network and home-directory reads, and a single
writable permutation file under the same local Seatbelt policy. Every result
was checked as a bijection and every process had a two-second timeout.

| Worker | Slowest row | Maximum wall time | Sum over 300 |
|---|---|---:|---:|
| Base | `faclay75` | 0.691763 s | 71.020 s |
| Candidate | `faclay75` | 0.692231 s | 71.078 s |

These times include sandbox startup and worker I/O. They show essentially
unchanged local cost, not a speedup or a hidden-hardware guarantee. The managed
Linux grader enforces the 4 GiB address-space limit; macOS local runs do not.
The patch adds no asymptotically new production allocation.

Reproduction uses the prescribed `yukon setup` and `yukon run` commands in the
benchmark work directory. Candidate compilation and tests must remain inside
the build sandbox; a bare workspace build is not equivalent. The trusted parent
alone was tested with `cargo test --release --offline --locked`. For candidate
tests the command inside the build-sandbox profile was
`cargo test --release -p ssi-candidate-worker --offline --locked`.

## Files, limitations and follow-up

Production changes are limited to `minl.rs` and `rgreedy.rs`; regression tests
live in their ordinary child modules. Knowledge-base changes document the
experiments and sources. Dependencies, the public `order` signature, scoring,
corpus, harness and sandbox implementation are unchanged. Generated benchmark
artifacts are not part of the ordering submission.

The durable findings are the stale-cache defect and an inexpensive admissible
floor for partial games with live boundaries. The end-to-end score gain is
small and distribution-sensitive. Hidden validation should determine whether
it merits promotion. Further work could investigate trajectory-preserving
pruning or bounds that reuse more than the most recent clique, but neither is
implemented or claimed here. Any stronger bound would need overlap-safe cost
accounting, exhaustive tests and the same density/work safeguards.
