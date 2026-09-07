# Fresh neighborhood epochs for completion-lattice descent

Effort: high

## Status, baseline, and execution boundary

This is an unmeasured remote-evaluation candidate on promoted source
`e88316db49ddf27aa5b862c41386f9c1fe6aca0f`, Yukon submission `c1607320`.
At the live check for this attempt, the benchmark was open, the current
official best was 0.850740, and claimed scores were recorded only rather
than used as an admission prefilter. This submission omits a claimed score.
The base's official score is context, not a measurement of this patch.

The operator requires candidate execution to remain on the official remote
grader. A fresh source-only checkout was created with Git LFS smudging
disabled; the matrix corpus remains an LFS pointer. No setup, local build,
unit test, development benchmark, timing probe, or agent worker was run.
Source review and `git diff --check` passed. Those checks do not establish
compilation, test success, runtime safety, or an objective improvement.

The earlier six-pivot experiments used an obsolete frontier. Their latest
component-factored candidate, submission `8da17485`, failed the two-second
ordering cap in workflow 33948134085. None of that candidate's production
changes is carried into this checkout. This patch starts from the current
promoted production source and changes only the MINL neighborhood marks.

## Mechanism under review

The terminal completion-lattice descent constructs the incumbent's filled
graph and considers deleting fill edges while retaining chordality. Its
local deletion test uses the common neighborhood of the candidate endpoints:
the inherited code checks whether that common neighborhood is a clique.
The scan groups work by the lower endpoint u so that its adjacency can be
marked once, then intersected with each candidate endpoint v by traversing
only N(v). This is an important reuse optimization; it should not require
rebuilding a membership table separately for every candidate edge.

The inherited implementation assigned the numeric mark `u + 1` to every
neighbor of u. A group switch rebuilt the marks for the next endpoint, and
later rounds reset which endpoint group was considered current. But a return
to the same endpoint u reused the same numeric mark. Rebuilding writes marks
for current neighbors; it does not necessarily erase marks for former ones.

This matters because the live adjacency changes during deletion. A neighbor
can be removed from N(u) while processing an edge whose lower endpoint is
some other vertex. The inline invalidation `umark[vv] = 0` maintains the
currently active group's table, but it is not a persistent reverse index
that invalidates every historical mark for every endpoint. A subsequent
revisit of u must therefore distinguish its current marks from historical
marks left by an earlier visit.

## Concrete stale-state scenario

Suppose a visit to u marked vertices {1, 3, 5}. Later another group removes
the edge between u and vertex 1. When the scan next returns to u, its actual
adjacency is {3, 5}. Reapplying the old endpoint-derived stamp to {3, 5}
leaves the historical stamp on vertex 1 unless it happened to be overwritten
or cleared by unrelated intermediate work.

If a candidate endpoint v is still adjacent to vertex 1, intersection by
that reused stamp can report vertex 1 as a common neighbor even though it
is absent from N(u). The exact clique test is then being applied to the
wrong set. This can obstruct a valid deletion or otherwise change the scan's
decisions. The full ordering scorer remains an independent acceptance gate,
but it cannot correct the membership table used by the search itself.

The source scenario is a correctness argument about cache invalidation, not
a claim that a particular public or hidden matrix has been reproduced. No
hidden pattern, matrix identity, or private per-matrix feedback was used.
The impact on completed deletions and final score requires remote evidence.

## Patch

Each neighborhood rebuild now receives a fresh monotonically advancing
epoch. A small helper, `minl_stamp_neighbors`, advances the epoch, charges
the row traversal, and marks exactly the current adjacency entries with that
epoch. Membership during intersection compares against the current epoch
instead of the endpoint identity.

The current-group reuse optimization is retained. Consecutive candidates
sharing u reuse the same stamp. A deletion in that active group still clears
the removed neighbor's membership with the existing `umark[vv] = 0` update.
A switch away and back, or a rebuild in a later round, gets a different epoch,
so older marks cannot represent current membership.

The epoch is a u32 with explicit wrap handling. On wrap to zero, all marks
are cleared, the clear is charged to the operation ledger, and epoch one is
used. Zero remains the unmarked sentinel. Wrap handling is deterministic and
does not consult time, process state, entropy, or external resources.

## Work and scope

For the ordinary non-wrapping rebuild, the neighbor traversal and its logical
charge are the same as before. The additional work is a counter increment
and comparison. No ordinary O(n) clear is introduced, no adjacency list is
copied, and no per-edge allocation is added. The exceptional full clear only
occurs after wrapping the epoch counter and is explicitly charged.

The deletion budget, number of rounds, fill gate, input gate, common-neighbor
cap, edge scan ordering, clique checker, MCS realization, AMD realization,
subsequent refinement, and all portfolio choices are unchanged. The patch
does not increase a configured work limit. It is not a new random seed or a
gate tuned to a corpus identity.

This still does not prove unchanged wall-clock runtime. Correcting membership
can change which edges are deleted, how many subsequent checks occur, whether
a descent is considered completed, and which later refinements are reached.
The remote two-second cap remains a required check. Any reported timeout is
a failure of the candidate regardless of the correctness motivation.

## Regression tests added, not executed

One helper test marks a neighborhood, processes an intervening neighborhood,
and rebuilds the first with a former neighbor absent. It asserts that only
the current neighbors match the new epoch and checks the accumulated row
charges. This exercises the specific invariant needed for repeated groups.

A second test starts at the maximum epoch with stale mark values present.
It checks that wrap clears all old entries, assigns epoch one only to the
new neighbor, and charges both the clear and the row traversal. These tests
use tiny synthetic arrays rather than the matrix corpus.

Neither test has been run locally. They are included as reviewable regression
coverage, not a claimed green test result or a mutation-proven reproduction.
The official scoring workflow may not execute these particular unit tests;
only checks actually reported by the workflow should be described as passed.

## Reproducibility and result interpretation

Start from `e88316d`. Change the MINL per-group neighbor-mark identity from
the endpoint number to a fresh epoch, preserving active-group invalidation
and adding wrap handling. Add the helper regression tests under the same
editable directory. Submit the intended source archive through Yukon without
a claimed score. The remote workflow owns compilation and official scoring.

Production changes are confined to `src/ordering/minl.rs`. The experiment
note and its knowledge-base index entry are documentation. No harness,
scorer, corpus, manifest, lockfile, dependency, or grader setting is edited.
The old experimental checkout is preserved separately and is not packaged.

A successful queue receipt means only that the archive was accepted for
validation. A completed score below the frontier must still meet the required
promotion margin. A rejection would mean this candidate did not establish
a leaderboard improvement; a failure must be reported with the actual
grader category when available. There is no promise of a new rank based on
the source argument alone.
