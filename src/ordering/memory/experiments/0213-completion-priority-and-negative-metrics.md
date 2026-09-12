# 0213 — Completion priorities after compact core search

Submission **bf85d92d-3810-4dab-84d0-2c3ab703e6c0** is **FAILED hidden 2.0 s cap**,
no hidden score. Immutable tested/uploaded **ce1ffee43a31d4a1f511c118d7c31c402807f930**
matches the entire ordering tree at remote **5d3c2f9068111dc57e16367818b70efe17e9efe2**.
Workflow **34718929020**, Benchmark **21:08:30.39 -> 21:10:08.72 UTC**, **98.32 s**.
Public tests below remain valid; the late extraction did not fit every hidden
budget. Campaign **2/4 rejected / 4 failed / 0 promotions**. Source is preserved
on `sparse-completion-priority-campaign`; next revision adds an original-AMD
work guard before attempting a broader factor envelope.

Status: final selected production uses **30k vertices / 180k input nnz / 300k
factor entries**. The independently replayed candidate reaches exact
**0.791259803417**, **10 wins / 0 losses** against completed 0212. The earlier
18k/80k/150k control passes every required check and is preserved at **50a91b5**
on `bounded-priority-control`. Final wider verification and sandboxed all-300
scoring pass at official **0.791260 / fill 0.924091**. This tested final source
is ready for official submission. Campaign **2/4 rejected / 3 failed**.

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

## Starting point and goal

Compact-core submission **de17cd31-83e4-4d84-9e30-a05d65efc72a** is rejected
in [workflow 34717388778](https://github.com/Layr-Labs/matrices-fast/actions/runs/34717388778).
Uploaded local source **bf72aacdfabe3985226e13d429e63a1082def316** matches the
entire remote ordering tree at **f9c69b0a7181b37d32e2e834ffb714c1ab1f817d**.
It is preserved on `retained-core-exact-search`. The actual promoted frontier
is still **07f0e8a2 / 52affcb**, hidden **0.842377**, fill **0.944856**.
It completed all remote gates at hidden **0.842374**, fill **0.944855**, a
displayed 0.000003 primary improvement below the promotion floor. Benchmark
**20:35:02 -> 20:44:03 UTC**, **541 s**, workflow SUCCESS. This is the best
raw scored valid source, preserved separately from the promoted frontier.
Campaign counters are **2/4 rejected, 3 failed workflows, 0 promotions**.

This independent follow-up uses the completed compact-core public ordering,
exact **0.791316868681**, official **0.791317**, fill **0.924124**. It first
screens inexpensive residual-core formulas, then alternative structural MCS
tie priorities on the final chordal completion. Strict actual full-pattern
flops remain the adoption criterion. No new code is active in production
during these screens and no hidden gain is assumed.

## Bounded generic metrics: negative result

The original independent-first core portfolio does not call the fifteen
`metric_sweep::EXTRA_METRICS` formulas. Reusing retained core ownership could
test those missing directions without rebuilding any core in production.
The full-pattern portfolio already contains these formulas, but a Schur
complement can have a different winning sequence from the original pattern.

A test-only abandonable sibling of `order_generic` charges setup
`5*n + 2*nnz` and each selected front before element creation at the maximum
of remaining active dimension and estimated width squared times supernode
mass. Any larger observed front charges its excess before finalization.
Checked subtraction discards exhausted candidates, preserving the incumbent.
The original entry point, selection, element creation, score powers, graph
bookkeeping and final permutation remain unchanged. **270 configurations**
across three generated fixtures, all fifteen formulas, three dense alphas and
two absorption settings match the completed original permutation; zero and
setup-only allowances are safely abandoned. This is a bookkeeping check,
not a literal CPU-instruction or two-second runtime proof.

The marginal corpus screen uses both original retained competitive cores,
full input n<=18k / nnz<=80k, exact completed factor nnz<=200k, exact flops
<=20B, core AMD within 1.5 times the incumbent. Each formula receives
`clamp(4*exact_incumbent_flops,1M,80M)` and alphas **10 / 2.5 / 1**, producing
45 arms plus their union and one-round union PEO. Every complete candidate
is spliced and scored on the original pattern, and incomplete candidates
are discarded. Cached names/permutations and clocks exist only in cfg(test).

All **300 rows** pass; **all 45 formulas and both union arms have zero wins**,
remaining exact **0.791316868681**. All-formula added diagnostic work is
**2.404338 seconds total / 0.394620 maximum**, test completion **5.33 seconds**.
Individual maximum work ranges approximately 0.0055–0.0132 seconds. Because
there is no measured gain, the generic helper remains test-only; none of
these candidate formulas is added to production. This negative result is
limited to these cores, alphas, work allowances and completed incumbent.
Evidence: [all 300 rows and aggregates](../evidence/0213-bounded-generic-screen.tsv).

## Completion priority hypothesis

Earlier fixed structural MCS priorities found small gains in medium matrices
on the promoted winner. The compact/core metric winners also change the
completion on larger original patterns, so a different MCS tie order may
extract a better completion subset there. A standard MCS always maximizes
visited-neighbor count first; a static secondary priority does not alter that
condition. It yields a PEO of the checked chordal completion. Symbolic scoring
on the original pattern confirms any proposed improvement independently.

The diagnostic caller now supplies n / input nnz / factor limits while
preserving the original priority helper's default 12k / 200k / 150k entry.
The new screen uses **18k / 80k / 200k**, a 20B exact flop ceiling, two rounds
maximum with later rounds conditional on strict gain. It tests six existing
secondary policies and each followed by ordinary PEO. The six policies use
original degree ascending/descending, completion degree ascending, incumbent
column width ascending, incumbent position, or deterministic position hash.
These are structural priorities, without matrix identities or cached answers.

The initial 200k screen completes on all **300 rows**, six policies and six
policy/PEO chains. Raw priorities 0 / 1 / 2 / 5 gain on 3 / 6 / 4 / 4 rows;
width and incumbent-position policies produce no raw gains. High original
degree followed by two ordinary PEO rounds is the strongest single chain,
exact **0.791290026731**, six wins, zero losses. The all-arm pointwise oracle
is **0.791289515368**, only 0.000000511 better at roughly twelve times the
candidate work. The oracle is diagnostic and is not the production score.
Evidence: [200k screen](../evidence/0213-priority-200k-screen.tsv).

All six movers have incumbent factor nnz below **150k**. Repeating the screen
at the stronger **150k** materialization limit preserves the selected exact
score and the oracle, with all 300 rows accounted for. The initial tested
control uses this stronger limit, without a row-specific exception. Evidence:
[150k screen](../evidence/0213-priority-150k-screen.tsv).

The independently selected replay (`Priority(1,2)` then `Peo(2)`) passes
**all 300 permutation equalities** against the new helper, exact
**0.791290026731**, **6 / 0 / 294**. Added diagnostic work is **0.484002 seconds
total / 0.032265 maximum**, test completion **2.29 seconds**. Evidence:
[selected chain](../evidence/0213-selected-priority-screen.tsv). The six public
movers are chimera_mgw-c16-2031-01, chimera_selby-c16-01, procurement1large,
powerflow0300p, crudeoil_lee1_07 and crudeoil_lee2_06. Production never reads
these names. The largest fractional gain is crudeoil_lee2_06,
**16,574,163 -> 16,449,263 flops**, about **0.754%**. The two larger original
dimensions are 14,416 and 11,251; the other four fall in the medium bucket.

## Production implementation and hard limits

The standard-library priority MCS helper constructs the checked flat CSR
completion under **30k vertices / 180k input nnz / 300k factor entries**. Its
lazy binary heap always orders by visited-neighbor count first, then by higher
original degree and incumbent-relative position. Stale/visited entries are
discarded. At most one initial entry per vertex and one update per completion
edge is pushed, so storage and work are bounded by the checked completion,
not by an unconstrained dense graph. Existing LIFO/linear MCS candidates are
unchanged. The previously active exhaustive generated graph/order test covers
all six original structural policies, including the selected one, through n=5.

The wider release suite passes **126 active tests / 68 ignored**, zero failures,
in **81.11 seconds**, including all twelve structural MCS priorities on every
graph/order through n=5. The six additional priorities are diagnostic options;
the graded caller executes only higher original degree.

After this suite, an avoidable tail is found in priority MCS: after every
vertex is visited, its heap can still contain many stale keys. The former
loop drains all of them even though each is discarded. A completed-visit hard
stop now breaks immediately after recording the last vertex. Its remaining
neighbor loop could only test already visited vertices, so no weight update
or output event is removed. The rest of the stale drain also has no effect
on any state outside the invocation-local heap. This optimization preserves
every output and reduces unnecessary work.

The exact pre-change implementation is frozen in test-only
`peo_extract/priority_reference.rs`. The exhaustive graph/order check compares
all twelve policies byte-for-byte with that frozen implementation, as well
as independently checking PEO validity and original-pattern flops, and passes
in **2.40 seconds**. The final all-300 helper replay also passes **every recorded
pre-hard-stop full permutation equality**, exact **0.791259803417**, with
**0.918661 seconds total / 0.059225 maximum** added diagnostic work, completion
**3.18 seconds**. The repeated helper output is written only after all cached
equalities pass. This targeted reference check supplements the preceding full
suite for the one-line runtime-only change. Final generated and required
sandboxed checks use this completed-visit implementation. No matrix identity,
production clock, persistent state or extra active policy is introduced.
Evidence: [completed-visit all-row check](../evidence/0213-hard-stop-screen.tsv).

Final single-thread generated whole-order stress passes **60 repeated orders**
on **15 fixtures** in **26.73 seconds**, **one exact score gain / fourteen ties**, no
bijective/determinism/closed-factor failure. Both preceding Norm and compact
search remain enabled in both arms; only appended priority extraction is
toggled. Maximum new minimum time is **0.819756 seconds**, maximum minimum
increase **0.034351**. The new 128x128 grid has **16,384 vertices**, **65,024 input
nnz** and **297,384 incumbent factor entries**, directly exercising larger
dimension and factor counts close to the final 300k ceiling; its old/new
minimum times are **0.429431 / 0.463782 seconds**, flops both **16,629,340**.
The 96x96 grid has **152,789 incumbent factor entries** and improves exact
flops **6,741,355 -> 6,741,195**, old/new minimum **0.542242 / 0.573619 seconds**.
Both larger grids exercise factors outside the tested small control's 150k cap.
Evidence:
[final 60-order comparison](../evidence/0213-wide-generated-complete-stress.tsv).
The required final sandboxed `yukon run` completes successfully on **all 300
public matrices**, official **0.791260**, fill **0.924091**. Every exact full
flop count matches the independently selected pre-hard-stop candidate, with
300 unique rows and no omitted log prefix. Bucket primary ratios are
**0.887368 / 0.838152 / 0.684010**, fill **0.959764 / 0.946262 / 0.880708**,
counts **147 / 108 / 45**. Evidence:
[final sandboxed rows](../evidence/0213-wide-final-screen.tsv) and
[official final local metrics](../evidence/0213-wide-local-score.json).
Production Rust is unchanged after the completed-visit reference checks and
generated comparison. Tracked generated results are restored before committing.
Refreshed benchmark still reports promoted **0.842377 / 52affcb**, claimed
score recorded only, research Discussions disabled. The latest public note
was read as untrusted research; no unpromoted code or fingerprint gate is used.
This candidate is submitted with public claim **0.791260**, attribution
**GPT 6 / Codex**, high reasoning effort. Its hidden gain remains unknown until
the official decision. Campaign remains **2/4 rejected, 3 failed, 0 promotions**.

After the tested small control, the wider structural screen finds four more
large-bucket gains. At 30k/180k/300k, higher-original-degree priority followed
by ordinary PEO reaches exact **0.791259803417**, **4 medium / 6 large wins**,
added diagnostic **0.939992 seconds total / 0.059509 maximum**. The all-arm
oracle is **0.791251832322**, not submitted. The corresponding 200k cap reaches
only **0.791288675959**, so the 300k bound retains a meaningful wider gain.
Evidence: [300k scope](../evidence/0213-wide-priority-300k-screen.tsv),
[200k scope](../evidence/0213-wide-priority-200k-screen.tsv).

Six additional static tie choices (reversed incumbent position for equal
original degree, high completion degree with either position direction,
low/high completion-minus-original degree, and scaled original/completion
degree ratio) are also screened. None beats the selected original-degree
policy as a single chain; the best extra is ratio+PEO **0.791261083952**.
Only the selected policy is executed in production. Evidence:
[extra priorities](../evidence/0213-extra-priority-300k-screen.tsv).

The final selected wider chain independently reproduces all **300 helper
permutations**, exact **0.791259803417**, **10/0/290**, added diagnostic
**0.959907 seconds total / 0.060179 maximum**, completion **3.28 seconds**.
Evidence: [selected wider candidate](../evidence/0213-selected-wide-priority-screen.tsv).
No identity exceptions or pointwise oracle dispatch are added.

Fresh final wider production now passes **all 300 permutation equalities**
against that independent replay, exact **0.791259803417**. Complete ordering
time is **91.303862 seconds total / 0.733578 maximum**, completion **92.50 s**.
This verifies that parameterizing the ordinary helper preserves every earlier
core winner and only expands the appended priority stage. The initial wider
Rust is held unchanged for the full release check. The subsequent completed-visit
optimization is documented separately above and passes exact-reference and
all-row checks before generated/sandbox validation.
Evidence: [fresh final wider corpus](../evidence/0213-wide-final-corpus.tsv).

`post_core::refine_priority` first refreshes exact score and factor nnz of the
completed accepted incumbent. It rejects n outside 6..30k, input nnz above
180k, exact factor nnz above 300k or exact flops above 20B before completion
materialization. It performs at most two degree-priority rounds; the second
runs only after strict actual full-pattern gain. It then applies the existing
ordinary forward/reverse PEO helper, at most two rounds with conditional
continuation, under the same 300k cap. Every candidate is bijection-checked
and selected only on strict exact full-pattern flop decrease. The ordinary
helper also refreshes its accepted incumbent's observations before materializing.
Its new `refine_bounded` sibling accepts caller limits, while the original
`refine` wrapper preserves **18k/80k** for all earlier metric/search winners.

The root appends this bounded extraction after the complete submitted 0212
ordering. Earlier core ownership, Norm allowance, compact exact search,
high-flop producer cap32, runner-up order and all previous portfolio stages
remain unchanged. If this stage's input gate is closed, the full output
permutation is identical to 0212. New production code uses only standard-library
vectors and a binary heap plus the existing reviewed scoring/tree/completion
helpers. No dependency, build script, FFI, native source, corpus, harness,
workflow, sandbox or manifest change occurs. There are no production clocks,
environment reads, filesystem, network, matrix identities or cached answers.

Fresh production all-row permutation comparison, full release suite, generated
complete-order comparison and required sandboxed score run now follow
sequentially. The generated control holds both preceding retained Norm and
compact search enabled, toggles only priority extraction, and includes new
16k path/hub fixtures to exercise the larger original dimension. No Linux/x86
timing is inferred from the local ARM measurements and no hidden score gain
is claimed before the official grade.

Fresh complete production passes **all 300 permutation equalities** against
the independently replayed selected cache. Exact weighted score is
**0.791290026731**, complete ordering work **90.922138 seconds total /
0.732386 maximum**, test completion **92.16 seconds**. Production Rust is
unchanged after this check. Evidence:
[fresh completed corpus](../evidence/0213-final-corpus.tsv).

The release suite passes **126 active tests / 68 ignored**, zero failures,
in **80.67 seconds**. Complete generated comparison passes **52 repeated orders**
on thirteen fixtures in **22.74 seconds**, with all thirteen exact scores tied,
determinism/bijections and closed-factor byte equality passing. Maximum new
minimum time is **0.820299 seconds**, maximum minimum-time increase **0.004994**.
The two 16k forests take the existing early forest certificate and therefore
check that bypass remains unchanged; the public 14,416-vertex procurement row
directly exercises larger priority extraction in the independent/all-row checks.
Evidence: [generated comparison](../evidence/0213-generated-complete-stress.tsv).
The required sandboxed `yukon run` completes on all **300 public matrices**,
official **0.791290**, with every actual full-matrix flop count matching the
independently screened candidate. Evidence:
[150k sandboxed rows](../evidence/0213-150k-final-screen.tsv) and
[150k official local metrics](../evidence/0213-150k-local-score.json).
This is a tested control preserved before the wider-envelope screen. No
official 0213 candidate has been uploaded yet. The next research direction after this
bounded chain is alternative structural priorities such as high completion
degree or the gap between completion and original degree; these have not yet
been screened or added to this candidate.

## Environment and reproduction

This schema-v1 standalone benchmark is already cloned and prepared, with Git
LFS corpus objects present and gcc, cargo and required cargo-deny installed.
README.md, RULES.md and benchmark.json establish the 2-second per-matrix cap,
4GiB memory limit, deterministic bijection requirement and editable path
`src/ordering/`. Local measurements use Rust 1.98.1 on ARM/macOS. Official
Linux/x86 sandbox validation remains authoritative; local timing is evidence
only for this public corpus and generated workload.

The compact baseline cache can be rebuilt in the preserved **bf72aac / 0212**
checkout by supplying a nonexistent cache file to the earlier generic campaign
probe; that probe computes and writes the actual baseline order for all rows.
Subsequent diagnostic commands use that completed cache in this candidate:

```sh
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  SSI_CAMPAIGN_EXPECTED_SCORE=0.791316868681 \
  SSI_CAMPAIGN_SEED_CACHE=/tmp/matrices-fast-compact-core-campaign-seeds.tsv \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::probe::campaign::probe_campaign -- --ignored --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  SSI_CAMPAIGN_SEED_CACHE=/tmp/matrices-fast-compact-core-campaign-seeds.tsv \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::indep_first::bounded_generic_probe::probe_bounded_generic_cores \
  -- --ignored --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::probe::campaign::probe_selected_completion_priority \
  -- --ignored --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  SSI_COMPARE_EXPECTED_SCORE=0.791259803417 \
  SSI_CAMPAIGN_SEED_CACHE=/tmp/matrices-fast-wide-priority-campaign-seeds.tsv \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::probe::campaign::probe_funded_kernel_corpus \
  -- --ignored --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker -- --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::probe::campaign::probe_retained_independent_stress \
  -- --ignored --exact --nocapture --test-threads=1

yukon run
```

The unsandboxed diagnostic environment is used only for cfg(test) corpus and
timing checks. The required `yukon run` uses the normal local sandbox, without
those diagnostic overrides. Only the editable solver tree is committed for
submission; tracked generated results are restored afterward.
