# Linear canonical permutation using symmetry

Effort: high. Source reasoning and remote validation, no local candidate execution.

## Prior evidence

An isolated two-vCPU AMD EPYC 7763 GitHub runner compared promoted source
e88316db49ddf27aa5b862c41386f9c1fe6aca0f against RMS source
3d73845fd4e8baab96ce38df3aa1b1cba248624a on the 300 public matrices.
The phase probe completed for both: 0.806560 and 0.806556 respectively.
Only pooling_rt2tp improved (5843 to 5829 predicted flops), while waste
regressed slightly (1270656 to 1270684); 298 matrices were unchanged.
Both public grader runs failed the enforced 2-second cap on mpbp_48.
Uncapped probe worst times were 2.588 and 2.592 seconds, respectively.
All two baseline and three RMS targeted custom-metric tests passed.
These are public-corpus diagnostics, not official hidden scores.

RMS is parked, including a patch archive, rather than carried into this
speed-only experiment. The public benefit was concentrated and too small
to justify a repeated official submission without another improvement.

## Mathematical invariant

For a full symmetric pattern A, transposition leaves the pattern unchanged:
P A P^T = P A^T P^T. Enumerating original columns in increasing new-index
order can therefore be interpreted as enumerating the new rows in order.
Scatter each neighbor into its new column; each destination receives
monotonically increasing row indices and is already canonical.

The old helper maps entries and comparison-sorts each resulting column.
The new helper constructs exactly the same CSC arrays in O(n + nnz),
removing all column sorts. It retains both triangles and existing diagonal
entries, adds no fill and removes no entries. The same full-symmetry and
bijective new-to-old permutation preconditions apply to both helpers.
This is not valid for arbitrary nonsymmetric CSC, which is outside these
call sites' contract.

## Scope

Added symmetric_permute.rs and replaced one imported helper in mod.rs.
All order-producing heuristics, candidate counts, score comparisons,
search budgets, and acceptance order remain unchanged. Direct qualified
uses of the vendor helper elsewhere are untouched, including test oracles.
No dependency changes and no trusted harness changes.

## Verification plan and current boundary

The new helper is checked against the qualified vendor implementation on:

- all 64 undirected four-vertex graphs, all 24 permutations, with and
  without diagonals, deliberately unsorted input adjacency;
- the empty pattern;
- all 300 public matrices under identity, reversal and two deterministic
  relabelings (1200 public-array equality checks).

An alternating-order test reports aggregate helper timings to reduce
systematic warm-cache ordering bias. Full baseline/candidate timing probes
and the unchanged trusted public grader run in the separate remote lab.
Candidate build and all tests are network-isolated by bubblewrap.

Remote run 34165012200 completed: both equivalence tests passed, including
1200 public CSC-array comparisons, and all 300 full-probe objective counts
matched exactly. Helper aggregate time was 452625905 ns for the reference
and 340426138 ns for linear construction (about 24.8% less). Full probe
duration fell only from 237.64 to 234.95 seconds; score stayed 0.806560.
Worst timing was 2.583 versus 2.608 seconds. Both public grader runs still
timed out on mpbp_48, so this is not a runtime-cap solution or an official
score improvement. Kept as a validated equivalent primitive while a larger
redundancy-removal hypothesis is tested separately in experiment0103.
Source diff whitespace check passed. Shell syntax validation
of the separate lab passed; an initial diff check accidentally inspected
the parent checkout and reported old results.tsv whitespace. That parent
file was not modified, and the scoped checkout's check then passed.

## Next decision

Require byte-identical arrays in every equivalence test and equal full
public objective counts before claiming an output-preserving speedup.
Use measured timings to decide whether the speedup is material enough to
create headroom for one separate mathematical improvement. A speed-only
candidate is not expected to meet the leaderboard's strict score threshold.
