# Adaptive exact clique membership with preserved logical charges

Effort: high

## Candidate and evidence

This revision follows `fb204d72-33a8-4c8c-90b4-a3db606b5e43`, candidate
commit `f57001a`, which corrected stale neighbor stamps in completion-lattice
descent. That candidate reached benchmark execution but failed the two-second
per-matrix cap. The failure log is available at:
https://github.com/Layr-Labs/matrices-fast/actions/runs/34160514564 .

No objective score was produced for that candidate. It is therefore unknown
whether the corrected search improves the official aggregate. The promoted
baseline remains `e88316d`, with official score 0.850740 at the latest check.
This revision retains the stamp correction and reduces actual work inside
the exact clique test, without changing the logical charges or search limits.

The operator requires remote-only execution. No local setup, compilation,
unit tests, benchmarks, profiling, or agent workers were run. Source review
and `git diff --check` passed, but compilation, regression tests, runtime,
and official scoring remain unverified at submission time. No claimed score
is supplied. The remote grader owns the outcome.

## Exact predicate

Let C be the common neighborhood of the endpoints of a proposed fill-edge
deletion. The inherited chordality-preserving deletion test requires C to
be a clique. For each member a of C, this is equivalent to requiring every
vertex of C other than a to occur in the sorted adjacency row N(a).

The previous implementation verified that condition by scanning N(a) and
counting entries stamped as members of C. That is a useful strategy when the
adjacency row is small compared with the required set. But when a has a large
external neighborhood and C is small, most visited entries cannot contribute
to the proof. The new helper chooses between two exact implementations of
the same set-containment predicate.

The sparse-required-set route iterates through C, skips a itself, and performs
binary search in the sorted adjacency row for every other required vertex.
It returns false on the first missing required edge. The scan route retains
stamped membership counting, with the existing impossibility early exit and
a new success exit once all required neighbors have been found.

Neither route samples edges or approximates clique membership. It is still
necessary to establish every required adjacency. The matrix graph is a simple
graph with sorted unique neighbor lists and no self loop in these rows, which
is the existing completion-graph representation used by the caller.

## Choice of route

For adjacency length d and required count k minus one, the helper compares
`(k - 1) * bit_length(max(d, 1))` with d. Saturating multiplication prevents
the route estimate from overflowing. If the binary-search estimate is smaller,
it probes the required set; otherwise it scans the adjacency row.

This estimate is a deterministic work heuristic, not a calibrated CPU model.
The exact predicate is independent of which route it selects. The threshold
depends only on current graph sizes and contains no matrix names, corpus
recognition, hidden-instance gates, or environmental inputs.

In the scan route, reaching k minus one matching entries is sufficient to
return true: all required distinct neighbors have been witnessed. Scanning
the remainder cannot invalidate a set-inclusion certificate. If too few
entries remain to reach that count, the helper returns false as before.

## Budget preservation

The caller continues to subtract the entire adjacency-row length from its
logical ledger before calling the helper. It also retains the budget check
at the same point after a successful membership test. The helper itself does
not refund credits for binary-search probes or early success.

This choice is intentional. It aims to make an already selected workload
cheaper, not to spend the saved time on additional search. Under the same
graph state, the exact predicate and the charged amount are intended to match
the predecessor, so subsequent edge decisions and budget cutoffs should not
change merely because this predicate uses a cheaper implementation.

The broader epoch-stamp correction, retained from the failed predecessor,
can change search decisions relative to the promoted baseline. This revision
does not claim score neutrality relative to the leader. It seeks a feasible
implementation of the corrected search while preserving the predecessor's
logical workload.

The unchanged per-stage allowance is not a proof of wall-clock safety. Other
stages dominate some inputs; binary search has different constant factors;
the route estimate is not a timing measurement. A remote timeout remains a
failure even if the predicate is mathematically exact. No time cap, memory
cap, work limit, or gate is relaxed.

## Retained stamp correction

Each rebuild of the grouped common-neighborhood membership table receives a
fresh epoch instead of reusing an endpoint-derived stamp. This prevents marks
for deleted former neighbors from surviving a switch away and back to the
same endpoint. Consecutive edges in the active group continue to reuse its
table, and the existing deletion invalidation still clears the removed
neighbor. Counter wrap clears marks with an explicit ledger charge.

That correction remains relevant because an exact clique predicate applied
to a stale common-neighbor set is not an exact test of the intended graph.
The two helper changes serve different purposes: current membership state
and cheaper verification of the resulting set, respectively.

## Tests added and limits

A new helper test enumerates subsets of an eight-vertex universe with at
least two members. For each member it generates sorted adjacency rows with
a possible missing required vertex and several amounts of external padding.
It compares the adaptive helper with direct set inclusion using `contains`.
The cases are designed to exercise both binary-search and scanning routes,
successful inclusion, and missing-edge rejection.

The earlier stamp-rebuild and epoch-wrap tests are retained. These tests
operate on synthetic arrays and do not require the matrix corpus. They have
not been executed locally and are not reported as passing. The official
scoring workflow may not execute these particular unit tests; only checks
actually reported by the workflow establish verified test results.

## Scope and reproducibility

Starting from `e88316d`, retain experiment 0098's fresh group epochs, then
replace the per-member stamped counting loop with the adaptive exact helper.
Keep the full-row charge and subsequent exhaustion check in the caller.
Preserve all other scan ordering, edge deletion, completion realization,
portfolio generation, and refinement settings. Submit the editable directory
without a claimed score and with this note.

Production changes are confined to `src/ordering/minl.rs`. Notes and index
entries remain under `src/ordering/`. No corpus, scorer, harness, dependency,
manifest, lockfile, or grader setting is changed. The source-only checkout
contains no downloaded matrix corpus and the earlier experimental checkout
is preserved separately. Nothing is committed or force-reset by this task.

A queue receipt means only that validation was requested. If the workflow
completes, compare the official score with the current frontier and required
promotion margin. If it fails, report the actual failure category and do not
infer an unobserved score. The purpose is exact work reduction supported by
remote evidence, not an unsupported promise of a new leaderboard rank.
