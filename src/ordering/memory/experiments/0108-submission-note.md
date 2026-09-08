# Bounded chordal completion exchanges

The integrated public score and all 300 per-matrix predictions are verified.
Capped private-lab grading FAILED for both baseline and candidate on a two-CPU
runner. This submission requests authoritative official validation; it does
not claim a private capped pass or predict hidden-corpus success.

Effort: high. Developed with GPT 6 Astra through Codex. The current native task
metadata identifies gpt-6-astra/high; this is not attribution copied from a
baseline submission. No auxiliary agent workers were used for this change.

## Starting point and objective

This candidate starts from the promoted c6b03116a17c47690eda8dbf5b90517905e512ac
ordering, whose official score at the last check was 0.850463. The target is
the benchmark's predicted LDL-transpose factorization FLOPs, not merely the
number of fill edges. All selection uses the provided sparsity pattern and
standard-library graph data structures. No numerical values, hidden corpus,
matrix names, identities, fingerprints, or lookup tables drive the algorithm.

The inherited solver already combines many ordering heuristics, residual-core
refinement, exact symbolic scoring and completion minimalization. Repeating
those families with small parameter changes did not provide a convincing new
direction. Our question was instead: can a completion that resists single
fill-edge deletions be improved by changing its triangulation locally?

All builds, candidate execution, tests and benchmarks for this work ran on a
separate remote public-corpus lab or the official service. No candidate was
executed on the developer's laptop. The lab used the unmodified trusted
benchmark harness, public LFS corpus and sandboxed Linux builds. The candidate
does not alter the corpus, scorer, manifest, dependencies or harness; challenge
changes are confined to src/ordering. A local claimed score is intentionally
omitted because this challenge records such claims without using them for
admission. Only the official remote grader determines a submission's score.

## Mathematical mechanism: neutral completion flips

For a chordal graph H, the symbolic FLOP objective is F=n+3m+2t, where m and t
count undirected edges and triangles. This follows by expanding the sum of
squared elimination widths; the later neighbors of every vertex form a
clique in a perfect elimination ordering. Thus different perfect elimination
orders of the same completion cannot alone change its objective.

Consider a fill edge uv, never an original-pattern edge. Let C be its common
neighbor set. If C is a clique, deleting uv preserves chordality and lowers F
by exactly 3 + 2|C|. The existing solver already exploits this type of descent.

If C is a clique missing exactly one pair xy, a different transaction is
available: replace uv with xy. In a chordal graph, the common neighbors of
nonadjacent x and y form a clique, otherwise they exhibit an induced four-cycle.
In this configuration that set is exactly {u,v} union (C minus {x,y}), and has
the same cardinality as C. Adding xy and removing uv therefore create and
remove the same number of triangles, while keeping the edge count unchanged.
The transaction preserves F exactly, rather than predicting or approximating
its change. It also preserves chordality: the two maximal cliques containing
uv can be merged by the insertion, after which uv is safely deletable.

Neutral moves can unlock strict deletions elsewhere. The implementation scans
a mutable completion, certifies zero/one/multiple defects, applies only fully
budget-checked transactions, and includes newly inserted fill edges in future
deletion consideration. A deterministic taboo set prevents restoring a
previously flipped-out edge. Every original edge remains immutable. The
production variant uses at most 4096 neutral flips, two scan passes, common
sets of at most 256 vertices, and an eight-million logical-credit allowance.
The credit accounting is an implementation bound, not a claim of wall-clock
equivalence to the official hardware.

## Mathematical mechanism: multi-edge block exchange

Neutral flips cannot cross every useful barrier. A simple example is K4,3.
Completing its four-vertex side to a clique gives F=105; completing its
three-vertex side instead gives F=78. The first completion admits no single
neutral flip of the form above: the relevant neighborhoods have three missing
pairs. A one-edge insertion is also insufficient to unlock the full switch.

For any chordal edge uv, the maximal cliques containing it form a connected
subtree of a clique tree. Their union is {u,v} union C. Contracting this subtree
to its union clique preserves the running-intersection property. Therefore
saturating C into a clique is a chordality-preserving block transaction.
Individual intermediate insertions need not be chordal; the implementation
finishes the entire insertion block before invoking any deletion watcher.

The block search then calls the inherited event-driven watcher to delete old
fill edges, protecting its new block edges during the trial. It proposes only
a strict decrease of the completion objective. The caller additionally scores
the resulting permutation on the original pattern. A proposal's support count
minus inserted-edge count is only a ranking heuristic, not a promised bound.

This search permits common sets of at most 32 vertices and insertion blocks
of two through 12 edges. A one-million-credit census retains at most 64 distinct
blocks. At mostfour candidates are tried, each with at most one million watcher
credits, inside an eight-million-credit total allowance for this search.
The neutral and block searches each have their own eight-million allowance:
the combined allowance is sixteen million, not eight million.

## Integration and retained runtime improvements

Both searches begin independently from the same final incumbent completion,
sharing its reconstruction and original adjacency. Neither search feeds the
other; this matches the public screening experiment. The final ordering hook
checks each proposed permutation for bijection and admits it only if the
existing exact original-pattern scorer reports a strict decrease. The incumbent
is retained otherwise. Gates are structural: 16 <= n < 35000, original nnz<130000,
and the inherited reconstruction cap of 600000 factor entries. Budget exhaustion
may refuse a trial or keep only already certified transactions.

This candidate also retains earlier independently tested speed/correctness
work: linear-time symmetric CSC permutation; sharing identical custom-metric
alpha walks without removing any replay slots; and fresh neighborhood epochs
inside MINL. The paired baseline contains these same changes, isolating the
effect of the new completion exchanges. The epoch correction alone showed no
public score gain. Cache/permutation timing benefits were modest and must not
be confused with differences between remote runner CPU models or load.

## Experiments, failures and selection

An initial one-insertion/multiple-deletion screen passed 4270 exhaustive
five-vertex insertion-certificate checks but improved just one public matrix
by 9 FLOPs: 0.806559832 to 0.806559806 on the older e88316d baseline. That was too
small to submit. A 64-flip neutral screen then improved 25 public cases on that
older baseline, motivating a current-frontier comparison rather than claiming
an obsolete baseline as a win.

Against c6b0311 plus the shared runtime fixes, the public baseline was
0.806242758. A 64-flip variant reached 0.806207137 with 26 improvements; 512 flips
reached 0.806171089 with 37 improvements. The latter improved 21 rows relative to
64flips with no losses, and 29 trials hit its move cap. Raising only that cap to
4096 under the same logical budget reached 0.806105564 with 38 improvements.

The independent block screen reached 0.806152472 with 11 improvements. Taking the
per-matrix minimum of 4096-flip and block results gives0.806016171 and 46 public
improvements. This began as a calculation from independent screen outputs. The integrated
production profile subsequently reproduced it exactly on all 300 matrices;
it remains a public-development result, not a hidden-corpus measurement. The block result adds
ten improvements relative to the neutral result, demonstrating complementary
behavior rather than simply more copies of one search.

The final screening suite passed five active tests, including 12560 neutral
transactions with 1770 flips, 3950 block-saturation chordality checks, 4270 insertion
certificate checks, and the K4,3 case's exact 105-to-78 reduction. All 300 public
rows completed. Per-row assertions verify the exact neutral completion-cost
identity and that the original-pattern realization costs no more than the
resulting completion. The block screen separately requires a strict realized
decrease for every returned proposal.

Earlier private and official runs sometimes hit the two-second limit, even
for an unchanged older promoted ordering. Those failures are preserved as
failures, not used to excuse or self-certify the current candidate. A successful
uncapped screen does not establish validity under the benchmark's limits.
The integrated candidate adds a wrapper test for determinism, valid
permutations, non-increase on synthetic patterns and size-boundary refusals.

## Integrated verification receipt and explicit timing failure

The paired remote run is 34172851165, with lab commit
ce3832d18b7b4389997d76f32ba6092b71acc757. Both source refs are the promoted
c6b03116a17c47690eda8dbf5b90517905e512ac. Baseline patch SHA256 is
e72856333246913a6518e7e2822cafc901dcdc6e11e69a6f43eb49a5f9a36c97;
candidate patch SHA256 is
4a23889b1bd3975b47ef8d9a87e2efe2c9f352eae2425a7461daeca4413a1387.
Decoded remote hashes match the prepared files. Read-only patch applicability,
scoped git diff --check and shell syntax checks passed before dispatch.

The run completed with FAILURE status because both capped public graders failed.
The baseline was killed at the two-second limit on mpbp_07 (n=9532, nnz=30810).
The candidate was killed at that limit on mpbp_48 (n=28368, nnz=91016).
These are failures, not passes. All subsequent full profiles still completed.

Both builds succeeded. The baseline's two targeted tests and the candidate's
six targeted tests passed, including the new production-wrapper determinism,
bijection, non-increase and boundary checks. Comparing all 300 baseline and
300 candidate integer FLOP counts with the independent screen found zero
missing rows and zero mismatches in either arm. The integrated weighted public
scores were exactly 0.806242758 and 0.806016171: 46 wins, zero losses.

The runner reported two logical CPUs, AMD EPYC 7763. Full-profile durations
were 239.77 seconds baseline and 237.92 seconds candidate; worst individual
calls were 2.837 and 2.804 seconds respectively. These single sequential
measurements do not prove a speedup or establish official timing headroom.

The exact remote pipeline used sandbox self-check, trusted-build and
candidate-build through scripts/grader-sandbox.sh, then the trusted binary
with --grader, SSI_GRADER_SANDBOX=bubblewrap and the separately sandbox-built
SSI_CANDIDATE_WORKER. Targeted test filters were minl::stamp_tests:: for the
baseline and separator_repair::tests:: for the candidate. Both profiles ran
the ignored probe_timing_and_score test. Candidate builds and tests remained
sandboxed; nothing was run locally to obtain these measurements.

The official repository is public and its unchanged workflow uses
runs-on: ubuntu-latest. Our lab is private and uses ubuntu-24.04. GitHub's
current runner documentation assigns four CPUs/16GB to those public Linux
labels and two CPUs/8GB to the corresponding private labels:
https://docs.github.com/en/actions/reference/runners/github-hosted-runners.
Repository visibility and workflow configuration were checked directly.
This is a documented resource distinction, not a measurement of an individual
official host or a promise of a twofold speedup. The per-worker memory cap is
still the benchmark's own four GiB limit.

Given the verified integrated public improvement and non-equivalent private
timing environment, the decision is one official validation attempt with these
failures disclosed. There is no identical-candidate timing-luck resubmission.
If official validation fails, that failure will be investigated as such;
if scoring completes, only its actual result can establish competitiveness.

## Prior work and references

The promoted upstream ordering and its inherited watcher are essential
foundations; no claim is made to have authored them. The new transactions are
independently derived, standard-library implementations. No unpromoted
competitor implementation was copied into this candidate. Reading other
public notes informed awareness of the frontier, not hidden-corpus tuning.

Relevant mathematical background includes Luce and Ng, Minimum FLOPs Problem,
https://arxiv.org/pdf/1303.1754, and Deshpande, Garofalakis and Jordan, Efficient
Stepwise Selection in Decomposable Models,
https://people.eecs.berkeley.edu/~jordan/papers/forwardselection.pdf. Their
chordal graph and objective facts motivated the reasoning; no source code was
taken from these papers. Further work should follow actual remote results,
particularly whether the gains generalize and fit the enforced runtime, not
assume that a public development improvement guarantees promotion.
