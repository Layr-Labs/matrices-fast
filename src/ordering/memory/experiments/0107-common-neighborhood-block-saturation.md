# Common-neighborhood block saturation

COMPLETED: remote34172289321 SUCCESS. Five active tests passed, including3950
block-saturation chordality checks and the exact K4,3 reduction105->78. Block
screen score0.806152472 vs baseline0.806242758,11public wins. It adds10rows
relative to the4096-neutral variant; joint per-row minimum0.806016171,46wins.
These independent screen results led to integrated capped validation0108,
run34172851165. No integrated or official result is claimed here.

## Historical dispatch and preparation record

DISPATCHED remote34172289321 after34171632067 completed successfully.
Lab commit344fb47c0be9c6b28b8c086be18984b568fa0055, sourcec6b0311,
patch1f1f28d5ed26d257ef9d075a01a385b0900a52426309edeb2b46e6eb054d04bd.
Byte verification and reverse applicability passed; no local execution.
Five active tests then300 public rows. Independent LONG_REPAIR is now4096
flips at unchanged8M credits because29 prior512 trials hit their move limit.
Prior512 screen improved37cases,0.806242758->0.806171089;4096 counts can be
compared to it but cross-run timing cannot. BLOCK_REPAIR is separate.
No block result claimed yet; all material below describes the hypothesis.

Prepared while remote34171632067 tests64/512 neutral flips on promotedc6b0311.
NOT in that dispatched patch. No local or remote execution of this block code
has occurred yet. Current separator_repair.rs differs from candidate.patch.

## Mathematical motivation

For the original complete bipartite graph K4,3, a completion that makes the
four-vertex side a clique has F=3*25+(16+9+4+1)=105. Making the three-vertex
side a clique instead gives F=4*16+(9+4+1)=78. Every old fill edge has a common
neighborhood with three missing pairs, so our single-defect neutral move cannot
start. One edge insertion alone is also insufficient. This example motivates
a bounded block transaction crossing a positive-cost barrier.

## Chordality derivation

Let uv be an edge in a chordal completion H and C=N(u) intersect N(v). The
maximal cliques containing uv form a connected subtree of a clique tree;
their union is {u,v} union C. Replace that subtree with one clique equal to
its union. The running-intersection property survives this contraction: any
vertex appearing both inside and outside occurred on their connecting path.
The resulting chordal graph is exactly H with C saturated into a clique.
Intermediate individual insertions need not be chordal; no watcher or PEO is
run until the entire block has been inserted.

After insertion, the existing watcher can delete certified old fill edges.
Only a strict completion-cost decrease is proposed; the public test also
requires the original-pattern scorer to realize a strict decrease. New block
edges are protected during each trial. These are independently derived graph
transactions, not copied solver code or matrix-specific lookup logic.

## Bounded prototype

Test-only block_repair uses C size<=32, two through12 missing pairs, at most64
distinct block proposals from a1M-credit census, at mostfour watcher trials.
Identical missing-edge blocks accumulate support from old fill edges. Ranking
by support minus insertion count is explicitly a heuristic, NOT a mathematical
lower bound or guaranteed gain. Only proposals with support>insertions proceed.
The whole operation has8M logical credits, each watcher at most1M. Clone,
realization, sorting and mutation work are reserved. Logical accounting does
not establish wall-clock validity, and the bounds may discard useful moves.

New unrun tests: K4,3 must exhibit105->78 and no neutral move; exhaustive
five-vertex chordal graphs must remain chordal after saturating the common
neighborhood of every edge. Public probe adds independent BLOCK_REPAIR rows
and BLOCK_TIME seconds. REPAIR combined times will include allthree variants
in this future screen; they are NOT production timing. Existing public
neutral variants retain their exact-objective assertions.

Next: wait for34171632067; inspect its tests and all30064/512 results first.
Then publish a separate byte-verified block-screen patch and dispatch once.
Only after convincing public gains should production integration and capped
remote A/B verification occur. No official submission or #1 claim yet.
