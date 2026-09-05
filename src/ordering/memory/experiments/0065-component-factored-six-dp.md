# Component-factored exact six-pivot dynamic programming

Effort: medium

## Status

This is an unmeasured remote-only revision of six-pivot candidate `1a03c8a`,
submission `ccbf3b9a-d6bc-4867-9971-1a3084a369ec`. That submission reached
benchmark execution but failed the two-second per-matrix cap. Its workflow
was https://github.com/Layr-Labs/matrices-fast/actions/runs/33946305422 .
No score was produced, and neither an objective improvement nor a correctness
pass for the whole corpus can be inferred from that failed run.

This revision factors independent components inside each six-vertex window.
It removes redundant subset transitions and cross-component row unions.
It does not raise or lower the configured cleanup allowance, add a pass,
change the six-vertex window size, or alter the final four-pivot cleanup.
No local build, test suite, benchmark, timing probe, or agent worker is run.
The user requires remote-only candidate execution. Source checks are the
only pre-submission verification; remote grading owns the authoritative result.

## Independence condition

Take the live elimination graph immediately before a six-vertex window and
form its induced graph on those six vertices. Distinct connected components
of that induced graph have no direct edges between them. Eliminating vertices
of one such component can add edges among that component's remaining vertices
and its external neighbors, but cannot connect a vertex in another component
to it: that would require an initial path through eliminated window vertices
between the two components.

The qualification is important: external vertices are still live during this
window. Two window components may share an external neighbor, but a path via
that live neighbor does not create elimination fill between them. It would
matter only if the external neighbor were eliminated, which the fixed-window
kernel does not do. The graph is formed after any preceding prefix has been
eliminated, so fill caused by that prefix is already included in connectivity.

Consequently each component's pivot widths depend only on which vertices in
that component have been eliminated. Interleavings with other components do
not change its costs. The optimum window cost is the sum of the independent
component optima. This factors the exact problem without approximating it.

## Dynamic-program reduction

The unfactored six-vertex DP has 64 subset states and 192 available transitions.
For a component of size s, its DP needs s times 2^(s-1) transitions. The new
implementation solves only subsets belonging to each component, then adds
their optima. Singleton components bypass the DP entirely and use the cached
degree-plus-one width directly.

For two three-vertex components, the transition count is 12 plus 12, or 24,
rather than 192. For a fully connected six-vertex window, it remains 192.
These are transition counts, not wall-clock measurements. Discovering the
components and merging their solutions also costs work. There is no claimed
speedup on connected windows and no guarantee that the prior timeout is fixed.

The cost recurrence inside each component is unchanged: the next state's
cost is the current optimum plus the square of the next pivot's exact width.
The path remains encoded with base-eight digits in u32, sufficient for six
positions. Equal-cost local paths retain lexicographic tie-breaking.

## Reconstruction

Each component reconstructs an optimal sequence of its own pivot positions.
The implementation then merges these sequences by repeatedly selecting the
smallest available head position. Components commute, so any merge preserving
their internal optimal orders realizes the same total cost. The smallest-head
merge supplies deterministic global lexicographic tie-breaking rather than
depending on incidental array or container iteration order.

If the optimal total cost equals the incumbent cost, the original six-position
order is returned unchanged. A local reorder is applied only on strict
improvement. The enclosing caller still recomputes the whole permutation's
predicted flop count before accepting it. The factorization is intended to
preserve the exact local optimum and tie behavior, not to trade quality for
speed; that intent remains subject to the unexecuted tests and remote checks.

## Row-union reduction

The previous constructor built every subset row union for every graph word.
Many of these subsets span different connected components and cannot be used
by a pivot-width query. The new constructor first builds a fixed list of
needed nonsingleton subsets contained in individual components. The list is
ascending, so each remove-lowest-bit predecessor is available when needed.

For each graph word, singleton unions are seeded from the six adjacency rows.
Only the listed larger subsets are then formed. Connected subset popcounts
feed the existing component-width table. A subset's predecessor may itself
be disconnected, so all within-component subsets are retained in the union
list, not just the connected ones. This distinction avoids using an absent
intermediate union when computing a connected subset.

If every window vertex is isolated in the induced graph, there are no needed
nonsingletons and no row-word scanning is performed. Singleton widths still
come from the cached live degrees. The fixed arrays remain small and no
graph-sized allocation is added by this factorization.

## Resource accounting and integration

The existing conservative six-window charge is retained: 32768 logical units
plus 1024 per graph word, in addition to the surrounding game and replay
charges. Savings in realized work do not grant additional budget tickets.
The final ordinary allowance remains 64 million units per kernel call and
the extended sparse allowance remains 24 million. The two-round cap, early
exit on a flat round, sliding traversal, and full-permutation acceptance stay
unchanged. No clock-based behavior or environment-dependent decision is added.

This is deliberately not a claim that unchanged logical budgets imply safe
wall-clock runtime. The unfactored predecessor demonstrated otherwise. Some
inputs have fully connected windows, and time in other ordering stages is
unaffected by this patch. The remote cap is the only authoritative resource
test available in this workflow.

## Tests and limitations

The six-window oracle test is extended with an explicit pair of disconnected
triples sharing live external neighbors and an edgeless fixture. The existing
sixteen deterministic density-varied fixtures remain. For each fixture the
test compares all subset/pivot widths with explicit graph elimination, checks
the DP optimum against enumeration of all 720 orders, validates the returned
bijection and realized cost, checks incumbent preservation on ties, and checks
zero-budget refusal. These tests were added but not executed locally.

Only source inspection and a whitespace check precede submission. The grader
may execute a different set of gates than the repository's unit tests, so
its result must not be described as proof that all source tests ran. No score
is claimed for this candidate. An upload receipt establishes only that it
was queued, not that it compiled or passed evaluation.

## Scope and result handling

The production delta from the preceding candidate is within `SixWindow` in
`src/ordering/rgreedy.rs`; its oracle fixtures are updated in that same file.
The dispatcher, earlier ordering portfolio, budgets, dependencies, harness,
scorer, corpus, manifests, and lockfiles are unchanged. Research notes record
the prior timeout rather than leaving a stale pending result.

There is no hidden-instance identification or private-pattern feedback in
this revision. The decision to factor is based solely on connectivity of the
current window in the supplied sparsity graph. A timeout would mean that the
whole candidate remains unviable under the enforced cap. A scored rejection
would establish a completed measurement without a promotion. Only the actual
official result can establish a leaderboard improvement.
