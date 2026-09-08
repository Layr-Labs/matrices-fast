# Flop-neutral completion flips followed by fill deletion

Test-only screen dispatched as remote34171018836, lab commit
a37321e5ad00bd6e747ad9e7cfde781155ae15cf. Candidate patch SHA256
ec7667fa99a0c94b0de4b6862d5c939cec47d7030b0bb6acbb0d37afe56fc21c.
Decoded remote hash verified; reverse patch applicability and diff checks pass.
No local compilation or execution; no production order() integration.

## Motivation from measured evidence

The broader insertion/watcher screen0105 passed its certificate tests but
improved only arki0016,836603->836594. Aggregate0.806559832->0.806559806 is far
too small to submit. Its witness-frequency census could prioritize pairs that
do not actually repair a blocked edge. This follow-up instead checks an exact
local condition and allows successive neutral moves, without a global watcher
trial per insertion.

## Derivation

Let H be chordal and uv a fill edge whose common-neighbor graph C is a clique
except for exactly one missing pair xy. Write |C|=k. Since x,y are nonadjacent
in a chordal graph, S=N(x) intersect N(y) is a clique: two nonadjacent vertices
in S would form an induced four-cycle with x,y. Thus S is exactly
{u,v} union (C minus {x,y}) and has size k.

There are exactly two maximal cliques of H containing uv: {u,v} union
(C minus {x}) and {u,v} union (C minus {y}). The cliques containing uv form a
connected subtree of a clique tree, so these two are adjacent. Merging them by
adding xy preserves a clique-tree representation and hence chordality.
Now C is a clique, so deleting uv also preserves chordality.

The insertion creates k triangles and the deletion removes k triangles.
Edge count is unchanged. Therefore F=n+3m+2t is exactly unchanged by this
atomic exchange. This sharpens0105's nonnegative one-swap bound to equality
under the explicit single-defect condition. It is an independent derivation
using the standard chordal facts documented in the linked literature.

Such a neutral swap may make other fill edges individually removable.
Each ordinary deletion with q common neighbors reduces F by exactly3+2q.
In particular, two certified fill edges sharing a center and the same missing
pair permit one neutral swap followed by a strict deletion. The search is
therefore a non-increasing walk on completion cost, not merely more PEOs of
the same completion. Reapplying the final PEO to the original graph can remove
additional fill, which is why the public screen checks the original scorer.

## Implementation and limits

The test module retains0105's validated code. New plateau_descent scans a
mutable completion and its fill-edge list. A zero-defect neighborhood permits
deletion; one defect permits a flip; multiple defects are skipped. Newly added
fill edges can later be deleted. A bounded taboo set prevents restoring a
flipped-out edge, and there are at most64 flips and two passes. Every mutation
is fully budget-checked before it starts. Original pattern edges never enter
the removable list.

The single-defect test uses one reusable epoch array to count common-set
adjacencies, finding a missing endpoint only when exactly one is absent. This
avoids an unbounded all-pairs membership check. The existing eight-million
logical-credit setting remains provisional; structural initialization and
realization are reserved up front. Logical credits are not a two-second proof.

The public screening gate expands to16<=n<35000 with nnz<130000 and fill<=600000.
Results must distinguish gains inside the old n<10000 gate from newly admitted
large cases; a global delta cannot be attributed solely to the algorithm when
the screening envelope also changes.

## Verification status

Current-frontier run34171632067 SUCCESS:3tests passed, all300 public rows and
the exact completion/realization identities passed. Baseline0.806242758;
64flips0.806207137 (26wins),512flips0.806171089 (37wins). The512 variant beats64
on21rows, loses on0, and is equal on279.29 trials hit the512 limit. Isolated
successful512 trials total0.890339s,max22.632ms transswitch0300p, excluding
shared graph construction and None results. Combined64+512 worst50.996ms.
Fullscreen241.61s,2logical CPUs AMD EPYC9V74. No cappedgrader was run.
Next34172289321 independently tests4096 under the SAME8M credits plus block
exchange0107. Its baseline is unchangedc6b0311; compare512 counts, not timing,
between those runs. Aim next for production integration/cappedremoteA/B if
the gain remains worthwhile rather than an unbounded parameter sweep.

Remote34171018836 completed successfully: all three tests passed, including
12560 plateau transactions and1770 flips, plus4270 insertion-certificate checks.
All300 public rows completed. Score0.806559832->0.806525511;25 matrices improved,
16 with n<10000 and9 in the expanded large band. Added1.513660s aggregate,
27.835ms maximum; full screen229.68s on2logical CPUs AMD EPYC7763.
This original screen kept min(old,new), so it is a gain screen, not proof that
every raw proposal was non-regressing. It did not run a capped production grader.

Meanwhile rcwrightiii2e6f2ee was officially promoted at0.850463, source
c6b03116a17c47690eda8dbf5b90517905e512ac. Its38 inserted/3 deleted mod.rs lines
were incorporated without discarding our changes. New remote34171632067 pins
that frontier and independently compares64 vs512 flips with the same8M-credit
budget. Both public proposals now assert old-completion == certified deletion
gain and original-pattern realization <= completion, instead of a min-filter.
This tests whether a longer neutral path pays off; no production integration.
REPAIR time includes BOTH trial variants and setup; LONG_PLATEAU reports the
isolated512-flip trial duration. Do not interpret combined time as a single call.
Patchfeb438a84d3a5793e2b19cd2661c30c8f377b1fb6fa3e649f149104341b72beb;
lab commit2825e737505999662b00520f323a6438ea0a197c; byte hashes verified.

### Original dispatch plan (now completed)

Three active tests are queued: the two0105 tests plus exhaustive five-vertex
chordal graphs with every choice of one or two designated fill edges. The new
test checks chordality, preservation of original edges, permutation validity,
and the exact identity F_before-F_after=sum(certified deletion gains).
No result is claimed yet. The ignored public screen then measures counts and
isolated added runtime on300 public inputs. This workflow does not run a capped
production grader, and success would not establish submission validity.

References: [0105](0105-separator-insertion-repair-screen.md),
[chordal insertion certificates](../literature/deshpande-garofalakis-jordan-stepwise.md),
[FLOP objective](../literature/luce-ng-minimum-flops.md).
