# Six-pivot subset dynamic programming under the existing cleanup budget

Submission `ccbf3b9a-d6bc-4867-9971-1a3084a369ec` FAILED: workflow
33946305422 reported a hidden ordering call exceeding the 2.0-second cap.
It reached benchmark execution but produced no official score.

Effort: medium

## Status and provenance

This candidate replaces the sliding five-pivot cleanup with sliding six-pivot
cleanup for dimensions at least six. It is unmeasured and submitted for remote
evaluation without a claimed score. The operator prohibits local builds and
benchmarks. Source review and whitespace checks are not evidence that the new
code compiles or passes tests. Added tests are present but unexecuted locally.

The immediate base is the previous sliding-window candidate `16baae6`, Yukon
submission `4feaabaa-a126-4725-a4f1-200609098ff4`. That candidate completed
grading with flop score 0.869872 and fill score 0.954644, but was rejected.
The promoted frontier was still `ea67ff8`, score 0.869826, at the live check
for this revision. Historical base scores are not claimed scores for this
candidate. No hidden matrix identity, sparsity pattern, or per-matrix score
was accessed or used in the design.

## Mathematical object

Fix the live elimination graph immediately before a contiguous six-vertex
window. For an eliminated subset S of those six vertices and a remaining
pivot v, form the connected component C containing v in the original live
graph restricted to S union {v}. Paths whose internal vertices are eliminated
create exactly the fill connections relevant to this pivot. Components of S
not connected to v cannot affect its current neighbor set.

For a connected component with more than one vertex, the union of its live
adjacency rows contains every vertex of C as well as its external neighbors.
After eliminating C except v, the pivot column width is therefore the union
cardinality minus |C| plus one. For a singleton, the adjacency row lacks its
own diagonal, so the width is the cached degree plus one instead. The kernel
handles that singleton case explicitly.

The squared width is the local objective contribution. The state after
eliminating an entire fixed set is independent of its internal order, so a
dynamic program may keep only the cheapest prefix for each eliminated subset.
This is exact for the fixed window under the inherited symbolic cost model;
it does not solve the global NP-hard ordering problem.

## Recurrence and reconstruction

There are 2^6 = 64 subset states. Initialize D[empty] to zero. For every
subset S and every pivot v outside S, consider:

```text
D[S union {v}] = min(D[S union {v}], D[S] + width(S, v)^2)
```

The total number of available subset-to-subset transitions is 6 * 2^5 = 192.
This replaces enumeration of 6! = 720 full permutations. Width lookup still
requires a small connected-component closure, and table construction still
has graph-word work; the transition count is not a complete runtime model.

Path codes use base-eight digits to encode local pivot positions. Six digits
need eighteen bits, so this implementation uses u32 rather than the u16 used
by the inherited five-pivot solver. For equal-length paths, numeric ordering
of these codes gives lexicographic tie-breaking. If the optimum ties the
incumbent, the kernel returns the original local order. It applies a reorder
only for a strict cost improvement.

## Component-width table

The new `SixWindow` has fixed arrays of length 64 for internal-neighbor unions
and component widths. It computes internal adjacency among the six pivots,
then determines which subsets are connected. For each graph word, it computes
all subset row unions by removing one set bit and reusing the smaller subset
union. Connected nonsingleton unions contribute their popcounts to the width
table. Singleton widths use live degree directly.

Only constant-size scratch is added per kernel: the subset arrays do not
grow with the full graph dimension. The existing bitset elimination game is
reused. No global corpus cache, stored permutations, graph fingerprint, new
dependency, or external solver is introduced.

The constructor prepays 32768 logical units plus 1024 units per graph word.
That allowance covers topology construction, subset closure, subset unions,
dynamic-program transitions, and reconstruction. The existing ledger also
charges validation, game construction, reset, and pivot replay. These units
are intentionally an algorithmic allowance, not a promise about CPU cycles
or wall-clock time on the shared grader.

## Integration and resource tradeoff

The sliding traversal is shared between the five- and six-pivot entry points.
It validates the requested width, rejects inputs smaller than it, and visits
all eligible window starts from left to right while budget permits. After a
window's accepted reorder, it eliminates the current first pivot to establish
the live state for the next overlapping window. It does not replay a pivot
after the final window because no further kernel requires the state.

The dispatcher selects six-pivot cleanup for n at least six. For n equal to
five, it retains the five-pivot path; smaller dimensions retain the existing
four-pivot behavior and gates. The final four-pivot call remains unchanged.
The two-round limit, strict full-permutation rescoring, and early stop after
a flat round remain unchanged.

Both final calls retain the reduced budgets from the immediately preceding
candidates: 64 million ordinary units and 24 million extended sparse units.
The larger kernel replaces the smaller kernel; it is not an additional pass.
It may inspect fewer windows before exhausting the same allowance. This is
the explicit tradeoff: a wider exact neighborhood at each inspected location
versus the amount of the graph that the fixed budget can cover.

## Correctness tests added, not executed locally

A new oracle test generates sixteen deterministic ten-vertex graph fixtures
at varying edge densities. It eliminates a two-vertex prefix in both the
bitset game and a separate explicit Boolean-graph oracle. The six-window
vertices have two external suffix vertices, so width checks include neighbors
outside the window and fill created by the prefix.

For every eliminated subset and every remaining pivot, the test compares
the kernel width with the explicit residual degree plus one. It separately
enumerates all 720 six-pivot orders using explicit graph elimination and
compares their minimum cost with the DP result. It checks the reconstructed
order is a bijection, realizes the optimum, and preserves the incumbent on
ties. It also checks that a zero kernel budget is refused.

Existing five-window oracle and budget tests remain present. The five-pivot
entry point now delegates to the shared traversal at width five so those
tests continue to cover the compatibility path. None of these tests is
reported as passing without execution. A successful remote scoring workflow
must not be conflated with execution of every source-level unit test; only
the checks actually reported by that workflow establish a verified result.

## Scope and interpretation

Production edits are restricted to `src/ordering/rgreedy.rs` and the final
dispatcher selection in `src/ordering/mod.rs`. Research notes record the
previous rejection. No harness, scorer, corpus, manifest, lockfile, dependency,
purity rule, or grader configuration is changed. The ordering still receives
only a sparsity pattern and uses deterministic computation without clocks,
filesystem access, environment inspection, network access, or subprocesses.

The six-pivot optimum cannot be worse than the incumbent for an individually
completed window, but this does not prove the full candidate dominates the
five-pivot algorithm. The traversal trajectory changes, budget exhaustion
can happen sooner, and the later four-pivot pass sees different incumbents.
Remote scoring must determine whether this tradeoff is productive. Likewise,
unchanged budget ceilings do not guarantee the timeout will pass. Any build
failure, resource failure, or rejection must be recorded as such; no result
is inferred merely from successful upload.
