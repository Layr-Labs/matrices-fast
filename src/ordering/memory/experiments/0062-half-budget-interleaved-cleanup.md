# Half-budget interleaved terminal cleanup

Effort: medium

## Candidate status

This is an unmeasured, remote-only candidate for `Layr-Labs/matrices-fast`.
It follows failed submission `e6fd7fab-6511-4516-9bb1-30657b3a54b4`, which
changed five-pivot sweep alignment order relative to promoted base `ea67ff8`.
The previous candidate compiled far enough to reach the benchmark execution
step, but the remote grader terminated an ordering call for exceeding the
two-second per-matrix wall-clock cap. It produced no official score.

This revision retains that deterministic alignment schedule and halves the
operation budget supplied to both kernels in the final cleanup chain. It is
a resource-reduction experiment, not a claim that the timeout is fixed or
that the interleaved schedule improves the official score. No local build,
test suite, development benchmark, or timing probe has been executed for this
revision. The operator requires all candidate execution on the remote grader.

## Verified prior result and current frontier

The failed workflow is:
https://github.com/Layr-Labs/matrices-fast/actions/runs/33943192983

The benchmark step reported: `order() exceeded the 2.0s per-matrix cap`.
That report establishes a resource failure, not an objective regression. It
does not identify the hidden matrix or which internal stage consumed the
time. No hidden pattern, matrix identity, or per-matrix private measurement
was requested or used to design this revision.

At the pre-submission check, Yukon still listed `ea67ff8` as the promoted
frontier, with official score 0.869826. The benchmark remained open and
claimed scores were recorded only. This candidate therefore omits a claimed
score and relies on independent remote evaluation. Historical development
and official scores belong to their respective corpora and do not constitute
a measured result for this revision.

## Change relative to the failed candidate

Inside the final `if pair_descent_gate` block in `src/ordering/mod.rs`, define:

```rust
let terminal_ops_budget = pair_descent_ops_budget / 2;
```

Pass that value to the existing final `adjacent_five_descent` and
`adjacent_four_descent` calls. Keep the maximum of two rounds, the five-then-four
composition, exact full-permutation rescoring, strict acceptance condition,
and early exit after a round with no accepted improvement.

The ordinary final-call budget becomes 64 million charged operations instead
of 128 million. The existing extended sparse gate uses 24 million instead
of 48 million. Across four possible final kernel calls, the sum of their
configured limits consequently falls from 512 million to 256 million in the
ordinary path, and from 192 million to 96 million in the extended path.
These sums describe this cleanup stage only, not the total ordering function.

The earlier adjacent-pair passes continue receiving their original budgets.
Subtree search, relabelled ordering, portfolio generation, simplicial promotion,
and all other prior stages are unchanged. This isolates the resource change
to the terminal stage whose sweep behavior was changed by experiment 0061.

## Retained alignment experiment

The five-pivot descent visits offsets `[0, 2, 4, 1, 3]` instead of increasing
offsets `[0, 1, 2, 3, 4]`. It still visits each eligible alignment at most
once and starts at zero. Different alignments operate on overlapping windows
of the current permutation; accepted changes can therefore alter the local
minimum reached by subsequent alignments. Interleaving is a deterministic
alternative visitation schedule, not an optimality guarantee.

An unavailable offset uses `continue`, not `break`, because an unavailable
offset four may occur before an available offset one on a small input. The
eligible offset set remains unchanged for n between five and eight. Inputs
smaller than five retain the existing early return. None of these decisions
depend on a matrix identity or corpus membership.

Reducing the budget can cause fewer eligible alignments or windows to be
completed. That is intentional. The existing budget-exhaustion behavior
returns completed changes through the established path, and the caller
rescans their full predicted cost before accepting them. The candidate does
not install a new fallback ordering or modify the budget ledger internals.

## Correctness and tradeoffs

All accepted local changes remain permutations of existing window vertices.
No vertex is introduced or dropped by the source changes. The inherited
input validation, deterministic seed, elimination-game construction, and
full-permutation objective remain intact. The added arithmetic divides a
positive existing budget by two; both configured values remain positive.

Strict acceptance means an individual terminal proposal is accepted only if
it improves its immediate incumbent. It does not prove that the overall
candidate matches or improves the promoted algorithm. A reduced search
budget can lose improvements found by the base. A changed visitation order
can lead to a different result, and changed incumbents can affect whether
the second cleanup round executes. Quality loss is an explicit risk.

Likewise, halving charged limits does not prove that measured wall-clock
time halves. Some calls may already finish well below their limits. The
ordering also spends time in stages outside this patch, and the prior
failure does not isolate the offending stage. This revision tests a bounded
resource reduction; it makes no assertion of verified timing headroom.

## Scope and review

Production changes relative to promoted `ea67ff8` are restricted to
`src/ordering/rgreedy.rs` for the retained alignment order and
`src/ordering/mod.rs` for the terminal budget division and two arguments.
Research notes are under `src/ordering/memory/experiments/`. No dependency,
lockfile, manifest, corpus, harness, scoring implementation, or grader setting
is changed. No new package, external process, network access, clock access,
environment access, or filesystem access is introduced into the ordering.

Verification before submission is source inspection and a scoped whitespace
check only. Compilation, tests, deterministic execution, memory compliance,
timeout compliance, and scoring are not claimed to have passed. The remote
workflow is responsible for the enforced checks; a queue receipt alone must
not be reported as a successful benchmark run.

## Reproduction and result handling

Start with the promoted production files at `ea67ff8`, apply experiment
0061's five-offset iterator change and small-input `continue`, then apply
the terminal budget division described above. Submit only the editable
directory with this public note and the actual model and harness attribution.
Do not supply the base's historical score as this candidate's claimed score.

A completed official score should be compared with the current frontier and
the required promotion margin. A timeout should be recorded as a timeout,
not treated as evidence that the score would have improved. If this reduced
revision still fails, the interleaved schedule has not established a viable
candidate and should not be retried unchanged. If it passes but is rejected,
its measured score establishes the quality cost or benefit of this particular
resource allocation, not a general law about window ordering.
