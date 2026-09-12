# 0214 — Original AMD work guard and larger sparse PEO factors

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

## Problem and resulting behavior

The preceding terminal original-degree MCS/PEO candidate improves public
flops but fails the hidden 2.0-second cap. This revision limits new terminal
work by the original AMD prediction and then explores larger sparse factors
on inputs inside that work envelope. The input's original AMD score is already
computed before all portfolio stages; observing it adds no symbolic analysis.
If it exceeds **1 billion predicted flops**, the new terminal extraction is
skipped and the earlier validated portfolio's completed permutation is kept.
This is a broad graph-cost bound, without corpus identity, a timing probe or
an environment-conditioned choice. It is a runtime heuristic, not a proof of
a two-second wall-clock bound.

Within that original-AMD envelope, the final accepted incumbent is freshly
scored, including its exact factor count. Inputs n outside **6..50k**, pattern
nnz above **1.3M**, exact factor nnz above **1M**, or incumbent flops above
**20B** do not enter the new completion extraction. The existing checked flat
CSR reconstruction, original-degree MCS tie rule and completed-visit hard
stop are reused. At most two degree-priority rounds run; the second requires
strict full-pattern improvement. At most two ordinary forward/reverse PEO
rounds follow, also with strict-gain continuation. Every proposed full order
is checked as a bijection and adopted only on a strict exact original-pattern
flop decrease. A rejected proposal's scratch factor count is never treated
as an observation of the accepted incumbent.

## Starting source, frontier and failure evidence

Actual promoted frontier remains **07f0e8a2 / 52affcb**, hidden **0.842377**,
fill **0.944856**. Best raw scored valid candidate is **de17cd31 / f9c69b0**,
hidden **0.842374**, fill **0.944855**, rejected below the promotion floor.
That source has completed all remote budgets and is preserved on
`retained-core-exact-search`; the promoted winner remains separately preserved.

Immediate public control is submitted **bf85d92d-3810-4dab-84d0-2c3ab703e6c0**,
tested/uploaded **ce1ffee43a31d4a1f511c118d7c31c402807f930**, remote
**5d3c2f9068111dc57e16367818b70efe17e9efe2**. The complete ordering tree is
identical. Public exact **0.791259803417**, official **0.791260**, fill **0.924091**.
That source fails in
[workflow 34718929020](https://github.com/Layr-Labs/matrices-fast/actions/runs/34718929020),
Benchmark **21:08:30.39 -> 21:10:08.72 UTC**, **98.32 seconds**, with explicit
`order() exceeded the 2.0s per-matrix cap`. It supplies no hidden score. The
failure remains recorded separately from scored rejections and its source is
preserved on `sparse-completion-priority-campaign`.

Current campaign counters: **2/4 scored rejections, 4 failed workflows,
0 promotions**. The user's stopping condition remains four official rejected
outcomes in this campaign; earlier historical submissions are excluded.

## Larger-factor screens and approach selection

Before the failure decision arrives, factor caps **500k / 1M / 2M** are screened
at **50k vertices / 1.3M input nnz**. Every screen starts from the completed
compact-core baseline, exact **0.791316868681**, preserving the preceding
300k candidate on all common eligible inputs. Each screen tests the six
original MCS priorities and their ordinary-PEO continuations. The graded
algorithm retains one globally fixed policy, higher original degree, and
does not dispatch the per-matrix oracle.

| Factor cap | Selected exact public score | Wins vs compact baseline | Added diagnostic total / max |
| --- | ---: | ---: | ---: |
| 300k control | 0.791259803417 | 10 | 0.918661 / 0.059225 s |
| 500k | 0.791259162926 | 12 | 1.170516 / 0.116297 s |
| 1M | 0.790704642081 | 17 | 2.129355 / 0.190699 s |
| 2M | 0.790687588918 | 19 | 2.780931 / 0.332897 s |

The 1M cap yields seven additional large-bucket gains over the immediate
300k public control. Its additional score decrease is **0.000555161336**,
roughly **7.0 relative basis points**. The 2M cap adds two more gains but its
maximum added work rises by roughly 75%. Selecting 1M retains approximately
**97% of the extra measured score decrease** while reducing maximum added
diagnostic work by approximately **43%**. After repeated hidden cap failures,
that is the selected next candidate; 2M remains a measured follow-up, not an
active branch of this submission. These timings are local ARM measurements,
not predictions of Linux/x86 speed or cap guarantees.

The new 1M public movers are pooling_sppc1pq, crudeoil_pooling_dt3,
crudeoil_lee4_10, nuclear104, nuclear10a, crudeoil_lee4_09 and arki0013.
Examples: pooling_sppc1pq **136,686,008 -> 131,708,592** full flops;
arki0013 **157,005,417 -> 153,121,933**; crudeoil_lee4_10
**184,173,410 -> 180,591,368**. Their names are diagnostic evidence only;
production sees integer dimensions, the pattern and exact symbolic costs.
The two extra 2M gains are not claimed for the selected 1M candidate.
Evidence: [500k screen](../evidence/0214-factor-500k-screen.tsv),
[1M screen](../evidence/0214-factor-1m-screen.tsv),
[2M screen](../evidence/0214-factor-2m-screen.tsv), all 300 rows each.

## Runtime guard after the official failure

The new terminal stage should not spend more time on an input whose earlier
search already required expensive dense-factor work. The selected coarse
original-AMD bound is **1B flops**. Across all **290 public rows** eligible for
the prospective 1M stage, the maximum original AMD prediction is
**812,210,652**, so this guard retains every measured public candidate. Inputs
outside the prospective stage already keep their earlier order. It does not
match a matrix name, tuple, hash or exact corpus statistic. The assumption
that this broad bound protects the timed-out hidden input is an inference;
only a new official grade can confirm it. No hidden identity is exposed by
the failure log and none is inferred or used in production.

The earlier high-flop producer cap32 and stable lazy runner-up ledger remain
unchanged. Earlier Norm/core search bounds, ownership, fixed seeds, sparse-word
kernels and x86 guards remain unchanged. The original ordinary-PEO wrapper
still preserves **18k/80k** bounds for previous late metric/search winners.
Only the appended priority caller uses the new broad limits and original-AMD
gate. Thus expensive inputs retain the same previous portfolio trajectory;
the grader's 2.0-second cap and sandbox are unchanged.

## Independent checks and next verification

Initial 1M helper replay passes all **300 exact permutation equalities** against
independent `Priority(1,2)` / `Peo(2)` replay, exact **0.790704642081**, 17 wins
against compact baseline. All common 300k input permutations equal the recorded
submitted control, and every full score is non-worse than that control. Added
diagnostic work is **2.147151 seconds total / 0.193052 maximum**, completion
**5.81 seconds**. A subsequent replay after adding test-only scope controls
also passes: **2.144858 seconds total / 0.193515 maximum**, completion
**5.81 seconds**. All 300 rows are archived in the
[selected replay](../evidence/0214-selected-1m-screen.tsv).

Fresh complete production with the original-AMD work guard passes all **300
exact permutation equalities**, exact **0.790704642081**; total measured
ordering time **92.193525 seconds**, maximum **0.910646**, test completion
**93.42 seconds**. The [complete corpus](../evidence/0214-final-corpus.tsv)
contains every public row. Full release tests, generated complete comparison
and required sandboxed `yukon run` follow sequentially. Generated
control holds prior Norm, compact search and 300k extraction active, shares
the original-AMD safety gate, and toggles only 1M/50k/1.3M scope. A new 192x192
grid exercises original dimension above 30k and larger sparse factor counts.
Test-only TLS scope controls, clocks and public cache names do not compile
into the submitted worker.

Only `src/ordering/` changes. New logic is standard-library integer work
gating and the existing pure-Rust tree/scoring/flat-CSR/MCS helpers. No new
dependency, manifest, build script, FFI, native source, corpus, workflow,
harness, sandbox, network or filesystem access is introduced. Production has
no clock, environment read, persistent cache, matrix identity gate or stored
answer. No unpromoted solver implementation is borrowed. Attribution is
GPT 6 / Codex, high reasoning effort; public notes omit credentials and
private workspace paths.

## Final release and generated verification

The complete release suite passes **126 active tests, 68 ignored**, with no
failures, completion **81.47 seconds**. This includes the exhaustive small
graph PEO/scoring checks and all twelve structural priorities against the
frozen stale-heap reference, original bounded-metric permutation checks,
RNG/work-state equivalence, sorted scatter permutation comparisons and the
stable lazy-ledger reference. No broad extra suite is needed after these pass
without another production edit.

The generated complete-order comparison passes **64 calls on 16 fixtures**:
both protected 300k control and selected 1M order repeat exactly; all returned
orders are bijections; **0 gains / 16 ties**, with no full-flop regressions.
Completion **28.74 seconds**, maximum measured per-fixture minimum **0.818647
seconds** for the candidate, **0.819860** for the control; maximum observed
added time **0.093021 seconds**. Timings are local observations, not cap proofs.

The added **192x192 grid**, **36,864 vertices / 146,688 input nnz**, has incumbent
factor **756,999**, so it is below 1M and above the prior dimension envelope.
It exercises the enlarged extraction body, not a forest bypass. Both arms
retain **59,682,899** flops; timing **0.394633 -> 0.487654 seconds**. Path/hub
16k fixtures retain their fill-free earlier results. Dense random fixtures
outside the factor/cost gates keep the same preceding output. The generated
control shares the new original-AMD work gate; it is a protected 300k-scope
control, rather than a claim to reproduce the timed-out immutable source on
all high-cost graphs. Evidence contains all sixteen complete comparisons:
[generated stress](../evidence/0214-generated-complete-stress.tsv).

The required sandboxed `yukon run` completes successfully on all **300** public
matrices, official **0.790705**, fill **0.923840**. Every individual n/input-nnz/
AMD/full-candidate flop tuple equals the independently replayed table.
Bucket flop ratios **0.887368 / 0.838152 / 0.682622**, fill ratios
**0.959764 / 0.946262 / 0.880082**, counts **147 / 108 / 45**. The improvement
against the promoted source is local; hidden score remains unknown.
Evidence: [official full table](../evidence/0214-final-screen.tsv) and
[machine-readable local score](../evidence/0214-local-score.json).
The generated results.tsv append is removed and the submitted archive changes
only `src/ordering/`. All required pre-upload checks pass on the exact
production source. No official attempt seven has been uploaded at this point.

## Reproduction and public frontier

Run from the benchmark root. CLI, Rust/cargo, gcc, git-lfs and pinned
cargo-deny 0.20.2 are already installed; setup and the baseline precede this
campaign. Schema version 1 describes one standalone challenge; no track,
coding-agent restart or hook installation is used. Production uses the
existing trusted dependency closure with no new declared crate. Local tests
run on ARM macOS, while the official grader runs Linux/x86; neither instruction
set nor wall-clock headroom is assumed equivalent.

The two independent seed caches contain only public test data. Reproduce the
baseline cache with the compact-core source and the immediate control cache
with submitted ce1ffee. Their TSV rows include the full permutations; they
are inputs to ignored diagnostics only and are not graded stored answers.
The selected replay writes the factor1m cache used by the fresh full check.

```sh
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::probe::campaign::probe_selected_completion_priority \
  -- --ignored --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
SSI_COMPARE_EXPECTED_SCORE=0.790704642081 \
SSI_CAMPAIGN_SEED_CACHE=/tmp/matrices-fast-factor1m-campaign-seeds.tsv \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::probe::campaign::probe_funded_kernel_corpus \
  -- --ignored --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  -- --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::probe::campaign::probe_retained_independent_stress \
  -- --ignored --exact --nocapture --test-threads=1

yukon run
yukon submissions --all
yukon benchmark show 8c3e7051-530a-4aee-88df-a426e6e78151
```

Diagnostic environment flags are test-only controls; the submitted algorithm
does not read them. The required scored run rebuilds and executes through the
unmodified build and worker sandboxes and checks purity, licenses, determinism
and bijection. Diagnostic tests run sequentially with no overlapping scored
worker. Any generated results.tsv append is removed before committing the
ordering archive; the machine-readable score and all 300 per-matrix counts
are retained under ordering memory instead.

The fresh pre-upload metadata check still reports promoted **0.842377** from
07f0e8a2 / 52affcb; research Discussions are disabled and claimed scores are
recorded only. Recent notes were inspected as untrusted data; no additional
unpromoted implementation contributes to this revision. The immediately
preceding bf85d92d failure is recorded above, not counted as a scored rejection.
Official grade and immutable uploaded ordering-tree verification will be
recorded when available. Reproduction of local quality is not a prediction of
hidden quality or a guaranteed promotion.
