# Sliding five-pivot cleanup with persistent elimination state

Submission `4feaabaa-a126-4725-a4f1-200609098ff4` completed grading but was
REJECTED: flop score 0.869872, fill 0.954644, versus frontier 0.869826.
This supersedes the pending receipt. No timing improvement was measured.

Effort: medium

## Status and base

This is a remote-only experiment, not a measured improvement. The immediate
base is submission `75ba74fc-3963-4e9c-b6c4-015f64411e89`, candidate commit
`94fafeb`, which completed the official workflow but was rejected with flop
score 0.869875 and fill score 0.954639. The current promoted frontier remains
`ea67ff8`, with official flop score 0.869826 at the latest check. The immediate
base is therefore a resource-valid candidate, not the leader.

The immediate base retained interleaved five-pivot alignment sweeps and used
half the frontier's final cleanup budgets. This candidate keeps those reduced
budgets but replaces the five separate alignment sweeps with one overlapping,
left-to-right sliding-window pass. It changes the search traversal and reduces
repeated elimination-state replay. It does not add a new portfolio member,
additional cleanup round, dependency, or larger operation limit.

No build, unit test, benchmark, timing probe, or agent worker has run locally
for this candidate. The operator requested remote-only candidate execution.
Source inspection and `git diff --check -- src/ordering` were the available
pre-submission checks. The new regression test is present but unexecuted.
Yukon's remote workflow determines compilation, enforced correctness gates,
runtime compliance, and the official score. No claimed score is supplied.

## Motivation

The existing five-pivot descent constructs an exact local kernel for a window
of five consecutive positions. Across its five alignment sweeps, it visits
each possible window start once, grouped by start position modulo five. Each
alignment resets the elimination game and replays the prefix and intervening
pivots to reach that alignment's later windows.

This organization repeats much of the prefix replay. The kernel needs the
elimination state before its current window, but it does not require that the
next window begin five positions later. Once the kernel has selected a local
permutation, the first pivot can be finalized and the next window can begin
one position later. The other four vertices remain live and can participate
in the next overlapping window.

The candidate implements that traversal. It resets the game once, visits
starts zero through n minus five in ascending order, and eliminates only the
accepted pivot at the current start before constructing the next window.
This is a structural reuse of elimination state, rather than another arbitrary
permutation of the alignment schedule. It may also propagate useful vertex
movements through adjacent windows within one pass.

## State invariant

Immediately before processing window start k, the game represents the graph
after eliminating `cur[0..k]` in that order. The current five vertices are
`cur[k..k+5]`; all remain live. The exact kernel compares their possible
orders using the existing component-width dynamic program.

If the kernel finds a strict improvement, only those five positions are
permuted. The already eliminated prefix is untouched. The candidate then
eliminates `cur[k]`, using its identity after the accepted reorder, so the
state required by the next start is available. Later windows never contain
that finalized position. Replaying the original seed identity here would
be incorrect; the code explicitly uses the current permutation.

For the final window, no next kernel is needed, so its first pivot is not
replayed. The function returns the current full permutation. The caller
continues to recompute the complete predicted flop count and retains it only
on strict improvement. No partial elimination state leaves the function.

## Budget behavior and complexity limits

The input validation, adjacency construction, game allocation, metadata
charges, exact kernel work, and pivot elimination charges are unchanged.
The existing reset charge is paid once before the sliding loop. If it cannot
be paid, there is no completed gain and the function returns `None`.

If kernel construction or pivot replay exhausts the ledger later, completed
strict gains are returned through the existing `changed.then_some(cur)` path.
The current permutation remains complete even if the game cannot advance.
No uncharged extra sweep or fallback computation is introduced.

With sufficient budget, the pass examines n minus four windows and replays
n minus five pivots. The former multi-alignment implementation examined the
same set of window start positions but repeatedly reconstructed elimination
prefixes across alignments. Both methods can terminate earlier under budget.
The candidate reduces this repeated replay; it does not claim a particular
wall-clock speedup, since the graph trajectory and kernel work also change.

The final five- and four-pivot calls retain the immediate base's 64-million
ordinary budget and 24-million extended sparse budget. The two-round cap,
early stop after a flat round, and strict full-cost acceptance are unchanged.
The earlier pair-descent calls and all portfolio/subtree stages keep their
existing budgets. No memory cap or time cap is relaxed.

## Expected quality tradeoff

Visiting overlapping windows in ascending order is not equivalent to visiting
them by residue class. Accepted changes alter subsequent windows, so the
output may be better or worse even if every individual local change improves
its current ordering. There is no dominance claim over the promoted frontier
or over the immediate rejected base.

The opportunity is to spend less of the fixed budget reconstructing states
and more completing useful windows. A hub delayed by one window may be
delayed again by the immediately overlapping next window. Conversely, the
residue-class traversal may expose useful combinations that this traversal
misses. Only the independent score can determine which effect dominates on
the evaluated distribution. The candidate contains no matrix identity checks,
corpus lookup tables, hidden-case gates, or evaluation-pattern recognition.

## Regression coverage and verification limits

A new test constructs star graphs for dimensions five through twelve and
starts with the hub first. It checks that the result is a bijection, improves
the canonical cost, and moves the hub into the final two positions. The final
two positions can tie in elimination cost, so the test does not demand a
particular tie-breaking order between the last hub and leaf.

Existing tests for five-window exact costs, budget exhaustion, malformed
inputs, deterministic output, and monotonicity remain unchanged. The new test
is not reported as passing: it was not executed locally. The remote scoring
workflow's enforced gates also must not be conflated with execution of every
unit test in the repository; only explicitly reported remote checks count.

Production edits are limited to `src/ordering/rgreedy.rs` relative to the
immediate base, with the prior terminal-budget change in `mod.rs` retained.
No harness, corpus, manifest, dependency, lockfile, or scoring implementation
is modified. Research notes record the preceding timeout and rejection so
future work does not mistake either candidate for a promoted improvement.

## Remote evaluation and interpretation

Submit the editable directory with this note and no claimed score. A queue
receipt is not a successful run. A successful workflow must still produce a
score that beats the current frontier by the required promotion margin.
If it times out, the intended replay reduction has not established sufficient
resource safety for the full inherited ordering function. If it completes but
loses on score, the changed traversal has not established a scoring win.
Neither outcome justifies inferring anything about individual hidden matrices.
