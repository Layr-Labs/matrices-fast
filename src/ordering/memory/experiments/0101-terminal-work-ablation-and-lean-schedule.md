# 0101: isolate terminal gains, then reduce terminal work

Date: 2026-09-07. Effort: max. Status: official local validation passed;
remote submission pending.

This follows [0098](0098-bounded-structural-terminal-portfolio.md) and
[0100](0100-linear-ranked-mcs-and-bounded-continuation.md). The purpose is to
separate demonstrated ordering gains from added work before another remote
submission. The predecessor's remote failure is not diagnosed by these tests.

## Baseline and provenance

The current promoted source checked at the start of this experiment was
`ef9ec4cb00faf0a32d8911b6cf3c0aae17e1eab7`, submission `6fa0167`, with hidden
score **0.851146**. Its independently rebuilt, untouched development control
scored **0.806787941749786**. These numbers refer to different corpora and must
not be compared as though they were measurements of the same workload.

The local starting commit was `ab5624ce2286a0f0ed9182e5df70ac3292b8e5b8`:
development score **0.806316086866332**, 300 official local cases passed, and
80 active candidate tests passed. Its predecessor `58ccecf`, development
score 0.806370063938585, was submitted as
`d21ea6f6-b053-4320-b038-0bfcf306f202` and **failed hidden validation without
a score**. The CLI exposed neither a failure category nor a failing case.
Runtime is a risk to reduce, not an established explanation of that failure.

The setup remains the new Git-LFS-backed matrices-fast repository, isolated
in worktrees. The older SSI challenge and Tungsten checkouts are not modified.
Only `src/ordering/` is edited; the trusted harness, corpus, scorer, manifests,
and dependency versions are unchanged. No new dependency is used. The local
machine is Darwin arm64, with Rust/Cargo 1.98.0, git-lfs 3.8.0, GNU GCC 16.2.0,
cargo-deny 0.20.2, and Yukon v2026.09.05-2. The 300-pattern public corpus has
SHA-256 `faa3ecc29c4ef2c54fe08e4382cee0fef39b04b2cc0efa521d8ecc9c66b7c5b6`.

The manifest is schema v1, with a one-basis-point improvement threshold.
Research Discussions are disabled and claimed scores are recorded only, so
there is no track selection, Discussion post, or claimed-score prefilter.

## What the implementation retains

The inherited AMD/AMF/partitioner/relabel/core/search portfolio remains the
control. The candidate still starts from the exact grader AMD ordering and
admits only exactly scored strict improvements. The canonical extra METIS
seed remains 2, following the default seed 1; it is not selected from a
favorable-seed search. No new seed is introduced here.

The additional terminal mechanisms are:

1. STRIP removes a prefix ranked by original degree, degeneracy core, or
   anchor symbolic column counts, orders the induced remainder, and appends
   the removed vertices. Its incremental induced-graph builder and earned,
   work-bounded two-direction MCS cleanup remain unchanged.
2. TELOS perturbs vertices with high symbolic column counts, either by
   reordering an induced remainder or by a simple tail splice. This experiment
   reduces its schedule, not its structural interpretation of the graph.
3. Terminal ranked MCS uses ID, completed degree, original degree, fill
   surplus, and incumbent column count as total static ranks, in both
   directions. Neighbor rows are prepared by the exact linear rank-ordered
   transpose from 0100. All candidates are independently exact-scored.

MINL scratch reuse and the runner-up admission-before-clone optimization
remain in place. The extra metric-donor continuation rejected in 0098 is
still absent. The component mixer from 0099 remains test-only.

## Full eight-arm phase ablation

The ignored `probe_terminal_phase_ablation` test runs all eight combinations
through the complete pipeline, not just isolated candidate scores. Arm order
alternates between matrices to reduce systematic ordering/warm-up bias.
All arms retain the canonical seed and allocation changes; mask bits select
STRIP, legacy TELOS, and the 0100 terminal MCS policy respectively.

Every arm covers all 300 matrices, with buckets 147 / 108 / 45. Scores are
computed from exact integer predicted flop counts, using the harness weights
0.30 / 0.30 / 0.40. The table's wins/losses compare against mask 0, which is
not quite untouched upstream because it includes the canonical seed fix.

| Mask | Added phases | Exact dev score | Wins / losses | Summed order() seconds |
|---|---|---:|---:|---:|
| 0 | none | 0.806779418773350 | 0 / 0 | 57.602792 |
| 1 | STRIP | 0.806494266130725 | 2 / 0 | 58.229724 |
| 2 | TELOS | 0.806689096540264 | 3 / 0 | 58.856671 |
| 3 | STRIP + TELOS | 0.806403948135740 | 5 / 0 | 59.595596 |
| 4 | ranked MCS | 0.806687852901109 | 13 / 0 | 59.009526 |
| 5 | STRIP + ranked MCS | 0.806406403332319 | 14 / 0 | 59.616978 |
| 6 | TELOS + ranked MCS | 0.806597532252059 | 16 / 0 | 60.459821 |
| 7 | all three | 0.806316086866332 | 17 / 0 | 61.016828 |

STRIP's two final-score gains appear late in corpus order: gabriel09 changes
27484225 to 26932633 flops and popdynm200 changes 2507726 to 2446837. The
first 250 rows alone would incorrectly suggest removing this entire family.
This is a reason to finish the full control, not a reason to gate on those
identities. The production code contains no corpus names or fingerprints.

The marginal contributions do not add exactly: the families alter the
incumbent that subsequent bounded searches receive. The whole-pipeline
ablation is necessary to expose that interaction. All three families are
retained. The gains are empirical; their concentration in a few examples
does not establish broad out-of-sample performance.

The initial combined-arm phase totals were about 0.658 seconds STRIP,
1.464 seconds TELOS, and 1.292 seconds terminal ranked MCS over the corpus.
These are local single-trial phase totals, not hidden-run predictions. The
test now retains the legacy schedules explicitly for reproduction; the
entry-gate/allocation cleanup below also applies to its reference arm.

## Rejected allocation experiment: reusable MCS buckets

A candidate `McsWorkspace` retained weights, visited marks, and bucket
capacities across the ten rank/direction extractions of one completion.
Every logical entry was reset between extractions, so only allocation
capacity was reused. The output was checked against the original fresh
allocation routine for every prepared rank and direction.

The matched microbenchmark uses real completed graphs, precomputes the same
five ranked inputs for both arms, and includes construction of fresh
per-completion scratch in the reuse arm. Each sample performs four batches
of ten MCS extractions. Eight trials alternate arm order; reported times
are medians. Completion reconstruction and neighbor preparation are outside
this MCS-only benchmark; output allocation and scratch allocation are inside.

| Public input | Fresh scratch, ms | Reused scratch, ms | Ratio |
|---|---:|---:|---:|
| sporttournament48 | 5.972 | 5.793 | 1.031x |
| mpbp_48 | 38.808 | 39.104 | 0.992x |
| crudeoil_lee4_10 | 129.224 | 129.071 | 1.001x |
| nuclear10a | 68.874 | 64.692 | 1.065x |

The result is mixed, including a small regression and an effective tie on
two important large completions. The workspace is **not used in production**.
Its implementation, equality checks, and ignored benchmark remain under
`cfg(test)` so the negative result is reproducible. The earlier linear
neighbor-preparation optimization is separate and remains retained.

## Retained TELOS cost reduction

The large schedule could generate up to 16 induced-subgraph orderings plus
15 peels per main round, run two rounds, and visit two runner-up seeds with
smaller schedules. The replacement uses exactly the existing small schedule
throughout the existing gate: one leader round with AMF-alpha-5/remove-1,
no-dense AMD/remove-6, and simple remove-1/remove-2/remove-8 peels. Invalid
prefix sizes are skipped. There are no extra seeds or additional rounds.

The gate remains `16 <= n <= 30000` and `0 < nnz < 400000`. It is checked
before rescoring or copying permutations. The production API now needs only
the input, leader, and leader score; it no longer clones or borrows the
runner-up archive. The test-only legacy arm borrows its seeds read-only.
The gate's dimension, nonzero, zero-input, and boundary cases have a focused
unit test. No new identity-dependent or score-bucket-specific gate is added.

`probe_telos_cost_ablation` runs the complete pipeline with old and new
TELOS schedules, preserving the same STRIP and 0100 ranked-MCS behavior in
both arms. Alternating-arm, full-corpus results:

| Measurement | Old schedule | Five-candidate schedule |
|---|---:|---:|
| Exact weighted flop ratio | 0.806316086866332 | 0.806316086866332 |
| Summed order() seconds | 60.780827 | 59.985292 |
| Summed TELOS phase seconds | 1.388716 | 0.612298 |
| Worst local call, seconds | 0.710337 | 0.694109 |

All **300 returned permutations are byte-identical** between these arms,
not merely equal in flop count. The TELOS phase total falls by about 56%,
while the whole-pipeline total falls by about 1.3% in this local series.
One trial per matrix does not establish a universal wall-time speedup;
other host workloads were present. The fixed candidate-count reduction and
removal of extra rounds/seeds are explicit production-work reductions.

## Ranked-MCS work-budget trial

The next controlled comparison halves the admitted-round work allowance
from 1500000 to 750000 units of `n + input_nnz + factor_nnz` and caps the
chain at two rounds rather than four. A strict win is still required to
continue. Large completions may now be refused before their first extraction;
this deliberately trades some search coverage for headroom. The separate
dimension/input/factor hard limits remain unchanged. Symbolic prechecks are
bounded by the round count but are not counted as primitive operations by
this structural proxy.

The full matched comparison passes all 300 cases:

| Measurement | 1.5M / four rounds | 0.75M / two rounds |
|---|---:|---:|
| Exact weighted flop ratio | 0.806316086866332 | 0.806332496822684 |
| Summed order() seconds | 63.963460 | 63.681181 |
| Summed terminal MCS seconds | 1.374435 | 1.066784 |
| Worst local call, seconds | 0.752130 | 0.712634 |

The smaller allowance deliberately gives back 0.2035 relative dev score
basis points versus the larger local candidate. It remains **5.6452 relative
basis points better than untouched ef9ec4c**: 17 improved inputs, zero
regressions, and 283 unchanged exact flop counts. Its bucket ratios are
0.889699744059344 / 0.845192016257687 / 0.714662421818936.

Six inputs change versus the larger allowance. These are the measured
tradeoffs, not names used by the production policy:

| Public input | Larger-budget flops | Smaller-budget flops | MCS seconds, before / after |
|---|---:|---:|---:|
| mpbp_48 | 16188775 | 16191599 | 0.101864 / 0.050665 |
| chimera_mgw-c16-2031-01 | 2635088 | 2635147 | 0.020052 / 0.011047 |
| crudeoil_lee4_10 | 198557639 | 199013967 | 0.059145 / 0.002004 |
| procurement1large | 7392774 | 7392786 | 0.062935 / 0.033210 |
| crudeoil_lee4_06 | 42814429 | 42818737 | 0.090590 / 0.045625 |
| crudeoil_lee4_09 | 142279294 | 142279687 | 0.098711 / 0.048138 |

The oversized-completion refusal gives back the terminal MCS gain on
crudeoil_lee4_10; it does not make that input worse than upstream. The worst
row in both matched arms is crudeoil_lee4_09. Phase cost drops about 22% in
aggregate, with roughly halved MCS times on the admitted costly continuations.
Total order() time is dominated by inherited phases, so the whole-run saving
is much smaller. The TELOS and MCS comparisons were separate matched series;
their wall-time totals should not be combined into one claimed speedup.

A separate focused check repeats both MCS-budget arms three times on ten
public inputs spanning costly MCS continuations, STRIP winners, and TELOS
winners. Arm order alternates and each arm's returned permutation must be
byte-identical across repetitions. All checks pass. Using the minimum call
time of three trials per input, summed MCS time is 0.549491 -> 0.273672
seconds, and summed whole-pipeline time is 5.717426 -> 5.415095 seconds.
Worst selected call is 0.745492 -> 0.686001 seconds. This is a deliberately
focused timing set, not an independent corpus or a replacement full score.
The selected rows and exact before/after counts remain public test data;
none of their identities enters the production ordering.

The smaller allowance is retained for headroom. No hidden-score improvement
or hidden-failure fix is claimed.

## Final official local gate

The supplied sandboxed `yukon run` passes **all 300 development cases** with
the final smaller schedules, including the harness's bijection, determinism,
and hard per-worker time gates. Exact integer flop counts independently
reconstruct **0.806332496822684** and match the matched-probe candidate on
every row. The emitted six-decimal score is 0.806332; fill-ratio tiebreak is
0.930374. The complete sandboxed candidate test suite passes **81 active
tests**, zero failures, with 35 research/benchmark tests ignored. The ignored
phase-ablation and allocation benchmarks described above were run separately.

These gates establish local correctness and local cap passage, not hidden
acceptance. The promoted source was rechecked as ef9ec4c before the upload;
no new leaderboard improvement is asserted while remote validation is pending.

## Reproduction and validation boundaries

The baseline and final official measurements use `yukon run`, which invokes
the trusted sandboxed candidate-build script and local parent. Candidate
release tests also run inside the same deny-network build sandbox, with
writes restricted to the target/cache/temp directories. No unsandboxed-worker
opt-out is used. Relevant test filters, passed after the standard sandbox
prefix, are:

```text
cargo test --release -p ssi-candidate-worker --offline --locked \
  ordering::probe::probe_terminal_phase_ablation -- --ignored --nocapture --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked \
  ordering::peo_extract::tests::probe_mcs_workspace -- --ignored --nocapture --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked \
  ordering::probe::probe_telos_cost_ablation -- --ignored --nocapture --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked \
  ordering::probe::probe_rank_cost_ablation -- --ignored --nocapture --test-threads=1
```

`SSI_PROBE_ONLY` and `SSI_PROBE_REPEAT` are test-only controls for focused,
repeated timing checks. All family masks and legacy-policy controls are
also test-only and local to the calling test thread. The production ordering
reads no environment, files, time, matrix name, or external cache. It uses
no newly searched random constants, hard-coded permutations, or lookup
tables keyed to corpus identities. Test-only code is not a production path.

Remote acceptance remains the required next gate after local correctness,
scope, determinism, and resource checks. The prior failure is retained in
the record rather than reinterpreted as a successful submission.
