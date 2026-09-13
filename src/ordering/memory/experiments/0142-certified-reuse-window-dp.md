# 0142: certified structural reuse and exact subset windows

Date: 2026-09-08. Base: `fb851f6b97d1ce0b1b7e1fa61b4746c940dac16e`.

## Verified result

| Metric | Base | Candidate |
|---|---:|---:|
| Full 300-pattern dev score | 0.804851 | **0.804632** |
| Fill ratio | 0.929670 | **0.929561** |
| lt_1k FLOP ratio | 0.887756 | 0.887678 |
| 1k_10k FLOP ratio | 0.842581 | 0.841949 |
| gt_10k FLOP ratio | 0.714375 | 0.714358 |
| Worst local timing-probe call | 1.159 s | **1.027 s** |

52 patterns improve, zero regress, 248 are unchanged. Both the full sandboxed
`yukon run` and the separate timing probe agree. The integrated candidate suite
passes 101 active tests; 31 opt-in probes are ignored by the ordinary invocation.
The relevant timing and window probes were run explicitly. Three scoped
read-only reviews found no significant issues.

These are public-dev observations. Timing is a single-machine comparison
subject to load.

## Official result

Submission **05685a47-a036-4133-bb07-c3278ff7f354** was **promoted** with hidden
score **0.848556** versus **0.848742**, and hidden fill ratio **0.947341**.
Benchmark metadata confirmed this as the current best, at source
`f7f60dc7eee3b7fa9b3ed057dfb53905d4777aef`.
[Grading run](https://github.com/Layr-Labs/matrices-fast/actions/runs/34281380111)
completed successfully; [submission PR](https://github.com/Layr-Labs/matrices-fast/pull/366).
The challenge remains open, so this does not imply a permanent lead.

## Changes

- `candidate_cache.rs`: same-family alpha settings with the same deferred
  dense-vertex set share one generator result. Every original result/error slot
  is retained, including duplicate donor-ledger entries. Seeds and complete
  metric identities remain distinct.
- `parallel.rs`: concurrent identical permutations share one exact symbolic
  score initialization; full equality protects hash collisions.
- `prefix_score.rs`: exact SmallScore replacement based on disjoint sets of
  eliminated-prefix vertices, without constructing fill. For the component C
  containing the newly activated pivot, the column count is
  `|union(N[x], x in C)| - |C| + 1`, using closed neighborhoods.
  Epoch-stamped scratch, sparse absorption, and a complete-clique suffix shortcut
  avoid repeated allocation and replay. Cutoff/neutral-walk semantics are tested.
- `chordal_certificate.rs`: bounded deterministic MCS followed by actual
  linear-time PEO validation. Verified original-graph no-fill orderings are
  globally FLOP-optimal because every chordal completion has
  `F = n + 3*edges + 2*triangles`. Nonchordal graphs continue normally.
- `rgreedy.rs`: known-clique and sparse-word exact elimination kernels.
  Original logical work charges, traversal, bucket behavior and RNG are
  deliberately retained. Six differential tests compare old Game state.
- `rgreedy/window_dp.rs`: exact subset DP within fixed elimination windows,
  factored by connected components of the live induced window. Outside boundary
  vertices are retained. Eliminating the same set leaves the same suffix graph,
  so strict local cost reduction is globally safe. The caller still verifies
  strict improvement with the existing exact scorer.

New code uses standard-library structures and portable integer/bit operations.
No dependency, harness, corpus, compiler policy, sandbox, or timeout was changed.
The inherited reviewed feral dependencies remain unchanged.

## Window experiments

The current schedule uses widths 8, 12, 10 with two offsets each and respective
16M, 32M, 24M precharged work budgets. Structural envelope: n <= 12000 and
directed nnz <= 200000. Completed gains survive budget exhaustion.

| Independent probe | Full-dev projected score | Improved | Worse | Worst added time |
|---|---:|---:|---:|---:|
| width 8 | 0.804739820 | 42 | 0 | 0.016500 s |
| width 10 | 0.804737239 | 46 | 0 | 0.018001 s |
| width 12 | 0.804681472 | 39 | 0 | 0.027320 s |
| chained 8,12,10 | 0.804631542 | 52 | 0 | 0.052730 s |

All starting counts in these probes matched the saved base. A smaller
n <= 4096 / nnz <= 60000 envelope improved only 22-27 rows; the general
work-bounded envelope increased coverage without approaching the memory cap.

## Research and lessons

METIS-style nested dissection is already present; another basic partitioner
duplicates substantial existing work. Exact structural reuse around the
portfolio is a more useful source of headroom here.

Relevant sources:

- Minimum fill and minimum FLOPs differ: <https://arxiv.org/abs/1303.1754>.
- Structural reductions and their qualifications:
  <https://arxiv.org/abs/2004.11315>.
- Recent parallel AMD: <https://arxiv.org/abs/2504.17097>; ordering-time
  improvements do not automatically imply a better FLOP objective.

Important pitfalls: deleting duplicate candidates can change donor trajectories;
reducing logical budget charges changes search behavior; arbitrary degree-two
elimination is not a minimum-FLOP certificate; MCS without PEO verification is
not a chordality certificate. No instance recognition is used.

## Follow-up

An additional CPU research direction is computing window-boundary unions through adjacency-signature histograms and
subset zeta sums instead of materializing every bitset union. That idea is not
implemented or claimed as a measured improvement here.

Links: [index](../index.md), [log](../log.md).
