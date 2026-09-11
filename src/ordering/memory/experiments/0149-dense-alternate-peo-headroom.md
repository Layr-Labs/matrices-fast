# 0149 — Dense alternate-PEO headroom after the third hidden timeout

## Outcome and submission target

This package keeps the complete fixed low-width terminal improvement from
[0148](0148-low-width-terminal-budget.md) and changes one inherited runtime
gate. It scores **0.792241** with a **0.924360** fill tiebreak on all 300
development matrices, exactly matching 0148. A direct full-corpus production
probe measured a **0.712-second** worst complete `order()` call.

The new gate skips the alternate-seed PEO extraction chain only when
`n >= 8,000 && nnz >= 5*n`. The pre-existing skip for the lee1_07 band,
`3,000 <= n < 8,000 && nnz >= 9,000`, remains unchanged. Final-count phase
profiling found no public score contribution from the newly skipped large,
dense branch. The sparse large matrices where this stage is known to win remain
eligible.

This is a runtime-headroom change rather than a new score mechanism. Its
purpose is to carry 0148's strict public improvement through the hidden
two-second limit while preserving every measured ordering gain.

## Why another runtime change was necessary

Three increasingly smaller terminal-window packages failed at the hidden
Benchmark step:

| experiment | submission | development score | added terminal policy | hidden result |
|---|---|---:|---|---|
| 0146 | `42491bf1` | 0.791984 | ten passes through n=12k | 2.0 s timeout |
| 0147 | `a1f928e2` | 0.792076 | ten small passes plus two sparse-medium passes | 2.0 s timeout |
| 0148 | `113841b9` | 0.792241 | three 8M low-width passes below n=3k, plus one tiny pass | 2.0 s timeout |

The third result changes the diagnosis. In 0148, every matrix above 3,000
vertices executes zero new terminal-window work. Those rows follow the same
candidate-generation pipeline as promoted source `62654a5`. Nevertheless,
the hidden run still killed one `order()` call. Removing more terminal work
from medium or large rows therefore could not address that failure: there was
none left to remove. The promoted pipeline was already near the hidden machine's
cap, and the small public improvement needed headroom elsewhere.

The hidden runner does not disclose the matrix or a phase trace. I therefore
profiled final production calls on the slowest development analogues, separated
the major candidate stages, and compared each phase's candidate with the final
accepted flop count. This tests both sides of a safe runtime cut: how much time
the phase consumes and whether deleting it changes the returned ordering.

## Phase diagnosis

The alternate-seed PEO chain was the cleanest removable cost on the slow dense
rows. Before the new guard, representative timings were:

| matrix | n | nnz | complete call | alternate-PEO phase | final flop change from phase |
|---|---:|---:|---:|---:|---:|
| `crudeoil_lee4_10` | 17,809 | 120,632 | 0.7962 s | 0.0814 s | 0 |
| `crudeoil_lee4_09` | 16,119 | 106,836 | 0.7845 s | 0.0837 s | 0 |
| `nuclear104` | 15,862 | 103,468 | 0.7709 s | 0.0859 s | 0 |
| `crudeoil_lee4_06` | 10,429 | 55,492 | 0.7198 s | 0.0845 s | 0 |
| `gams05` | 9,933 | 51,160 | 0.6870 s | 0.0253 s | 0 |
| `arki0016` | 68,958 | 246,592 | 0.6498 s | already skipped | 0 |

The phase costs about 80–86 ms on four of the most relevant public analogues.
Its candidates lose to the incumbent before the terminal exact-window stage,
so the final returned permutations and symbolic counts do not depend on those
calls. This is enough work to matter after three hidden cap failures, while the
phase has a simple structural boundary that retains its known wins.

With only the large-dense alternate-PEO guard applied, the same targeted
final-count probe measured:

| matrix | guarded complete call | final flop count changed? |
|---|---:|---|
| `crudeoil_lee4_10` | 0.7045 s | no |
| `crudeoil_lee4_09` | 0.7052 s | no |
| `nuclear104` | 0.6851 s | no |
| `crudeoil_lee4_06` | 0.6348 s | no |
| `gams05` | 0.6807 s | no |
| `arki0016` | 0.6941 s | no; branch was already skipped |

The worst targeted call fell from 0.7962 to 0.7052 seconds, about an 11%
reduction. The small timing increase on `arki0016` is ordinary run-to-run
noise: the new condition does not alter its already-skipped control flow. A
separate complete 300-row probe then reported **0.712 s** as the global worst.

## Preserving the known alternate-seed winners

Earlier lineage probes identified five important large-matrix beneficiaries of
the alternate-seed chain. All are sparse relative to their dimension:

| matrix | n | nnz | nnz / n | new large-dense guard fires? |
|---|---:|---:|---:|---|
| `mpbp_15` | 9,858 | 31,692 | 3.21 | no |
| `mpbp_35` | 11,120 | 40,790 | 3.67 | no |
| `mpbp_34` | 11,556 | 40,860 | 3.54 | no |
| `gabriel09` | 21,688 | 89,702 | 4.14 | no |
| `arki0013` | 44,909 | 160,172 | 3.57 | no |

The `nnz >= 5*n` boundary separates these documented winners from the dense
lee, nuclear, and gams rows above. The dimension floor is also necessary. An
initial screen applied the density condition at every dimension. It scored
0.792245 rather than 0.792241 because `maxcsp-ehi-85-297-71` at `n=2,372` and
`nnz=207,996` still benefits from the alternate candidate. Restoring the stage
below 8,000 vertices recovered the exact production score. The retained gate
therefore follows measured lineage rather than treating density alone as a
proxy for usefulness.

## Rejected broader runtime cut

I also screened a second dense-row guard around a later PEO-completion and
admission block. That edit looked attractive in an isolated phase timer, but
the final production score worsened to **0.793066**. On `nuclear104`, final
flops regressed from 78,332,024 to 89,038,906 because later candidates depend
on the completion result even when its immediate candidate does not win.

That experiment was reverted completely. No part of its condition, temporary
phase instrumentation, or test switch remains in the candidate. This negative
result is useful because it distinguishes a locally losing candidate from an
intermediate state that enables later improvements. The alternate-seed branch
selected for production has neither role on the guarded public rows: deleting
it preserved every final count in both targeted and full-corpus runs.

## Production policy

The effective alternate-seed exclusion is:

```text
(3,000 <= n < 8,000 and nnz >= 9,000)
or
(n >= 8,000 and nnz >= 5*n)
```

Only the second clause is new. Rows outside those shapes continue to run the
same alternate PEO extraction, exact scoring, and strict acceptance logic as
the promoted source.

The score-producing terminal policy from 0148 is retained exactly:

| eligible rows | width | sweeps | stride | operation allowance |
|---|---:|---:|---:|---:|
| n <= 3,000 and nnz <= 80,000 | 6 | 4 | 1 | 8M |
| n <= 3,000 and nnz <= 80,000 | 7 | 4 | 3 | 8M |
| n <= 3,000 and nnz <= 80,000 | 8 | 4 | 3 | 8M |
| n <= 512 and nnz <= 80,000 | 12 | 4 | 1 | 8M |

These passes improve 11 public rows. Representative strict improvements are
`pooling_sppa0pq` from 1,234,887 to 1,226,861 flops,
`crudeoil_pooling_ct3` from 869,821 to 868,337, and `hydroenergy2` from 55,946
to 55,877. The tiny pass also improves `wastewater05m1` from 8,036 to 8,002.
The large-dense guard is disjoint from every terminal-eligible row, so it cannot
remove these gains.

## Score and timing evidence

The exact candidate has the following aggregate results:

| metric | promoted parent | candidate | change |
|---|---:|---:|---:|
| weighted flop score | 0.792300 | **0.792241** | **-0.000059** |
| weighted fill tiebreak | 0.924373 | **0.924360** | lower |
| worst complete public call | prior 0.8 s band | **0.712 s** | lower |

The weighted score is formed from the benchmark's three dimension buckets.
The production timing probe reproduced the same **0.792241** aggregate score,
so the runtime instrumentation did not expose a scoring discrepancy. The
candidate also returned the same flop count as 0148 for every public row. This
means the runtime guard contributes no speculative public score change; all
aggregate improvement still comes from exact, strict terminal-window
acceptance.

## Correctness, determinism, and scope

The new decision uses only `n`, `nnz`, and integer constants. It performs no
wall-clock measurement and reads no matrix name, corpus position, external
state, hash iteration order, or randomness. A given sparse pattern therefore
takes the same branch on every run.

Skipping a candidate generator cannot invalidate the returned permutation.
The incumbent is already a deterministic bijection, and all remaining
candidates pass the existing exact validation and symbolic scoring machinery.
The full worker suite exercises bijection, determinism, score-workspace,
certificate, and exhaustive small-graph properties.

The implementation changes only `src/ordering/` and uses the Rust standard
library already used by the ordering module. Temporary profiling edits and
screen switches were removed before validation.

## Validation

The exact production tree was checked with:

```sh
yukon run
SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test --release \
  -p ssi-candidate-worker probe_timing_and_score \
  -- --ignored --nocapture --test-threads=1
SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test --release \
  -p ssi-candidate-worker
git diff --check -- src/ordering
```

`yukon run` completed all 300 matrices at **0.792241**, with a **0.924360**
fill tiebreak. The direct timing-and-score probe completed all 300 at the same
score and reported **0.712 seconds** as the worst `order()` call. The worker
suite passed **118 tests**, with 39 explicit measurement probes ignored and no
failures. The final diff check is clean.

This work was performed with GPT 6 through Codex at high reasoning effort. The
approach combined exact score screens, per-phase final-candidate lineage, and
full-corpus validation; it did not infer safety from an isolated phase timer.

## Hidden result and revised diagnosis

Submission `3a4f3ed0` failed the hidden Benchmark step in workflow
`34514415941`. The grader began at 18:30:46 UTC and reported the same two-second
per-matrix timeout at 18:32:26 UTC, about 100.46 seconds into the step. The
large-dense cut therefore did not address the hidden bottleneck, even though it
removed about 90 ms from the matching public rows.

This result changes the useful boundary. Promoted source `62654a5` passed the
hidden corpus, while all four descendants that added terminal work failed.
The fourth descendant removed measurable inherited work from large dense rows
and still failed. The simplest evidence-consistent explanation is that the
timed-out hidden row is eligible for terminal windows and does not match the
large-dense alternate-PEO gate. Further unrelated large-row cuts would spend
public score or complexity without targeting the observed failure class.

The follow-up in [0150](0150-equal-budget-terminal-window-exchange.md) removes
an inherited 32M terminal pass before adding a replacement 32M pass. That keeps
the entire terminal allowance at the promoted parent's hidden-proven 136M and
uses a width-eight exponential state space in place of width twelve. The dense
alternate-PEO guard remains because it is public-score neutral and creates
headroom on rows that match it, but it is no longer the primary timeout
argument.

Effort: high.
