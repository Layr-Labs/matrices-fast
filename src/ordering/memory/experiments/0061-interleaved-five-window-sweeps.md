# Interleaved five-pivot sweep alignments

Submission: `e6fd7fab-6511-4516-9bb1-30657b3a54b4` FAILED remote grading.
Workflow 33943192983 reported that a hidden matrix exceeded the 2.0-second
per-matrix cap. No score was produced. This supersedes the pending status.

Effort: medium

## Summary and verification status

This is a remote-evaluation candidate based on promoted frontier `ea67ff8`
in `Layr-Labs/matrices-fast`. It changes the alignment visitation order inside
`adjacent_five_descent`, from `[0, 1, 2, 3, 4]` to `[0, 2, 4, 1, 3]`.
The number of possible alignments, work budget, exact window kernel, and
strict-improvement acceptance rules are unchanged. This is a search-schedule
experiment, not a claim of an already measured improvement.

No build, unit test, development-corpus benchmark, or timing measurement was
run for this candidate. The operator requested remote-only execution. Yukon
reports that claimed scores are recorded only for this benchmark, so this
submission intentionally supplies no claimed score. The remote grader must
establish compilation, correctness, resource compliance, and the official
score. A submission being accepted into the queue is not evidence of passing.

## Base and prior evidence

The promoted base is submission `56ee74d`, commit `ea67ff8`, with the current
official score 0.869826 and fill ratio 0.954591 when inspected for this attempt.
These are base results, not measurements of this candidate. A previous
fresh build of the base passed all 300 public development matrices with
flop score 0.843978 and fill score 0.943905. The public development score and
official hidden score refer to different corpora and must not be compared as
if they were competing measurements of the same sample.

Experiment 0060 reversed the final four-pivot and five-pivot calls. It passed
the development run but scored 0.843979 with fill 0.943906, slightly worse than
the base. That change was reverted and is not included here. The terminal
composition remains five-pivot then four-pivot, with at most two rounds and
the existing early exit when a round accepts no improvement.

The recent public submission `23f4d41` tested a third conditional terminal
round and scored 0.869810, but was rejected because the gain did not meet the
promotion threshold. This candidate does not add that third round. It changes
how a fixed five-alignment pass visits its windows rather than increasing the
number of cleanup rounds. No code from that rejected candidate was copied.

## Mechanism

The five-pivot kernel processes disjoint windows at a selected alignment.
Each alignment starts by resetting the elimination-game state, eliminating
the prefix before that alignment, and then scanning five-vertex windows.
The current permutation is retained between alignments, so an improvement
accepted during one alignment changes the windows seen during the next.

Visiting alignments in increasing order is one deterministic choice, not a
requirement of the exact local objective. Interleaving alignments changes
which overlapping windows are exposed next. A shift by two positions shares
three positions with the previous five-position alignment rather than four.
This can change the search trajectory even though each local kernel still
selects its result using the same exact objective and acceptance condition.

The chosen schedule visits every residue modulo five exactly once. It starts
with alignment zero, preserving the first sweep and its early-budget behavior.
It then visits two, four, one, and three. This schedule is constant for every
input; it does not encode matrix identities, corpus membership, dimensions
associated with particular examples, or hidden-evaluation observations.

There is no claim that this visitation order dominates increasing order.
Local search is path-dependent. Both schedules can improve on their starting
permutation while ending at different objective values. The candidate may
score better, worse, or identically to the base. Remote results decide whether
the changed trajectory is useful.

## Small-input correctness detail

The original increasing loop could stop with `break` when an alignment no
longer had five positions available. That assumption is invalid after
interleaving: an unavailable alignment four can precede an available alignment
one. The candidate therefore uses `continue` for an unavailable alignment.

For n=5, only offset zero is processed. For n=6, offsets zero and one are
processed. For n=7, offsets zero, two, and one are processed. For n=8, offsets
zero, two, one, and three are processed. For n>=9, all five offsets are
eligible. Thus the eligible set is preserved even though its order changes.
The function's existing n<5 rejection is unchanged. These cases were checked
by source review, not by executing tests, and remain subject to remote
verification.

## Scope and invariants

The sole production-code change is in `src/ordering/rgreedy.rs`, inside
`adjacent_five_descent`: the offset iterator, the unavailable-offset control
flow, and the explanatory comment. `src/ordering/mod.rs` remains the base's
production dispatcher. The four-pivot kernel, five-pivot exact solver,
elimination-game implementation, scoring code, and all operation-budget
constants are unchanged.

No dependency is added or changed. The candidate adds only Rust standard
library syntax over the existing implementation. It does not access values,
right-hand sides, files, environment variables, clocks, network resources,
external processes, or random entropy from the ordering function. The
permutation is still produced entirely from the input sparsity pattern and
the deterministic incumbent generated by the inherited portfolio.

Every accepted window reorder still permutes only its own existing vertices.
The caller still recomputes the complete predicted cost and accepts only a
strict improvement against its incumbent. Budget exhaustion still uses the
existing return path. No purity gate, harness, manifest, corpus, lockfile, or
scorer is changed. Existing historical research files remain in the editable
tree, but they are not an input to the ordering function.

## Resource caveats

The candidate retains at most five alignments and the same precharged work
ledger per invocation. It adds no kernel call, cleanup round, search stream,
or allocation. Nevertheless, unchanged configured budgets are not proof of
unchanged runtime. Different accepted orders can change realized elimination
work, budget exhaustion points, and later cleanup activity. The authoritative
two-second and memory-limit checks must still pass on the remote runner.

Prior experiments established that apparently modest changes can fail hidden
timing even after successful development runs. This submission therefore makes
no timing-headroom claim and does not treat a passing historical base as a
guarantee for the candidate. No hidden per-matrix data was accessed or used.

## Reproduction and interpretation

Start from the promoted production files at `ea67ff8`. In the five-pivot
descent routine, replace the increasing offset range with `[0, 2, 4, 1, 3]`,
and replace the unavailable-offset `break` with `continue`. Keep the final
five-then-four composition and every numeric budget unchanged. Submit the
editable directory through Yukon with this public note and without a claimed
score. The grader owns the build and score; do not infer success from upload.

If the remote run fails, record the actual reported failure before considering
another candidate. If it scores worse or fails the required improvement
threshold, this specific schedule has not established a useful contribution.
If it is promoted, the result supports this deterministic alignment schedule
on the evaluated distribution, not a general proof of optimality. Subsequent
work should continue to use structural algorithms rather than identifying
evaluation instances or tuning to private per-matrix behavior.
