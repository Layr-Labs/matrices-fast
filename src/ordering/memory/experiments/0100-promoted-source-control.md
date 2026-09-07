# Unchanged promoted-source control after two timeout failures

Effort: high

## Purpose and expected outcome

This submission is an explicit diagnostic control requested by the operator's
continuing competition workflow. It contains the unchanged production source
from promoted commit `e88316db49ddf27aa5b862c41386f9c1fe6aca0f`. It is not
an optimization and is not presented as original algorithmic work. If it
completes with the same official score as the current leader, rejection for
no improvement is the expected outcome and is a useful control result.

The current promoted submission at the pre-submission check was `c1607320`,
owned by rcwrightiii, with official flop score 0.850740. The source includes
the contributions and inherited lineage of that promoted repository. This
control claims no new contribution to those algorithms, no improvement to
their score, and no entitlement to promotion based on reproducing them.

## Why this control is being run

Two recent revisions on this source reached the benchmark step but failed
the enforced two-second ordering cap. Submission `fb204d72`, candidate
`f57001a`, introduced fresh membership epochs in MINL's grouped-neighborhood
scan. Submission `28565936`, candidate `5e678d0`, retained that correction
and used an adaptive exact clique-membership test. Neither produced an
official score. The public failure messages identify a hidden ordering
timeout but do not locate the internal stage responsible.

The failed workflows are:

- https://github.com/Layr-Labs/matrices-fast/actions/runs/34160514564
- https://github.com/Layr-Labs/matrices-fast/actions/runs/34161387368

Repeatedly adding code without a current remote baseline would conflate
candidate effects with baseline execution conditions. This control asks a
narrow question: can the unchanged current promoted production source still
complete the official grading path at this point in the investigation?

A passing control does not prove which change caused either previous failure.
It does establish a current completed execution of the starting source. A
failed control would show that a timeout is possible without either new
production change and would prevent treating every failure as a measured
regression caused by those changes. One run cannot establish a failure rate
or prove the absence of runner variance.

## Exact source state

The checkout began at `e88316d`. Its only modified production file from the
preceding experiment was `src/ordering/minl.rs`. That experiment's patch
was preserved as a documentation artifact and its production file restored
to the bytes recorded at HEAD. A subsequent Git comparison reported no diff
for `src/ordering/minl.rs`. No other production source file was changed in
this source-only checkout.

The remaining differences are research documentation and this submission
note. They record prior failures and the reason for this control. They are
not referenced by production Rust modules or the ordering function. The
preserved patch has a `.patch` suffix and is a historical diff, not a Rust
module, included source, build script, or runtime data file.

The original older experimental checkout remains separate and unchanged.
Its six-pivot and sliding-window experiments are not part of this control.
The control does not graft the failed source onto the promoted baseline,
combine independent experiments, alter a gate, or substitute a different
dependency. It is an unchanged-production-source diagnostic submission.

## Local execution boundary

The operator has prohibited local candidate execution. Accordingly no local
setup, compilation, unit test, benchmark, timing probe, or agent worker has
run for this control. The checkout was obtained with Git LFS smudging disabled;
the development corpus remains a pointer and was not downloaded for tuning.
Only source inspection, small Git comparisons, notes, and Yukon submission
or status commands have been used on the local machine.

This control intentionally has no claimed score. Yukon reports that claimed
scores are recorded only for this benchmark, not an admission prefilter.
The remote workflow owns the rebuild, enforced gates, score, and promotion
decision. A queue receipt is not reported as a completed benchmark result.

No source-level regression test is claimed to pass merely because a prior
submission compiled or the base was historically promoted. The official
workflow may execute a particular subset of repository tests, and only its
reported checks count as current evidence. The exact diagnostic objective
is completion of the trusted remote benchmark for unchanged production code.

## Preserved benchmark contract

The ordering receives a sparsity pattern and returns a permutation. The
trusted grader recomputes the predicted factorization objective and verifies
the required permutation and determinism properties. The hidden evaluation
corpus remains private. This submission does not request its contents,
individual matrix identities, private per-matrix measurements, or any
exception to the evaluation boundary.

No harness, scoring implementation, corpus, manifest, lockfile, dependency,
memory cap, wall-time cap, purity rule, or workflow configuration is changed.
The editable archive remains confined to `src/ordering/`. The submitted
production ordering is precisely the published promoted implementation,
with its existing budgets, gates, portfolio choices, and refinements.

The model and harness attribution identify the agent conducting this control
submission. They do not replace or claim authorship of the promoted algorithm's
existing contributions. The exact source commit and prior submission are
recorded above so the result can be interpreted with the correct provenance.

## Result interpretation and next step

If grading completes at 0.850740 and rejects the submission for lack of an
improvement, the intended control has succeeded. That would confirm current
remote execution of the baseline, while leaving the cause of the two modified
candidate timeouts unresolved. The next optimization should then be judged
against this reproduced baseline, without pretending the control itself won.

If the unchanged source fails the cap, preserve the actual failure message
and workflow link. Do not infer an objective score or declare the runner
broken from one failed run. Baseline instability or an unmeasured execution
condition would become a live possibility requiring further safe diagnosis.
Do not loosen the grader's cap or change the benchmark to force a result.

If the remote score differs despite unchanged production source, first
compare the source, trusted baseline, and reported evaluation metadata before
attributing that difference to an optimization. No such optimization exists
in this control. If another solver advances the leader while validation is
running, report both the control's result and that separate frontier change.

The result of this control will be recorded explicitly as a control, rather
than a winning candidate. Its purpose is to establish trustworthy evidence
for continued algorithm work under the operator's remote-only constraint.
