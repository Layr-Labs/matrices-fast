Final official outcome: **REJECTED #3**, hidden **0.841858**, fill **0.944586**,
a tie with promoted globalfe4f40c. **bb07f71e** passes workflow**34724755761**,
Benchmark**23:14:14 ->23:23:26 UTC**,552seconds by job times. Entire uploaded
ordering tree**0f006f4d36f270780a01284efb8632794717d220** equals tested**2068133**.
This is a valid scored source, not a failed workflow. Campaign**3/4 scored
rejections,9 failed workflows,0 own promotions**. Currentbestglobalfe4f40c;
our best raw scored candidate is nowbb07f71e at the same0.841858.
Next production activates the bounded conditional original-edge3->2 sequence.

# Share the late-stage budget between terminal windows and retained-core search

This experiment addresses a concrete interaction between two late searches.
The terminal schedule runs an exact subset exchange, completion re-extraction,
and up to five span passes. The late independent-set stage can subsequently
run an adaptive quotient metric and compact exact core search on the same
matrix. Each component has a local work allowance. Those separate allowances
do not limit their combined cost within the hidden two-second order() cap.

Candidate 2a68edc7-d876-46c1-a393-43b084ec603a removed the previously added
wide priority MCS and LexBFS stages and disabled the medium producer fence.
It still failed the hidden time cap, so merely reducing those unrelated late
completion calls did not establish a valid combination of the terminal and
core searches. Its tested/uploaded source is
0edf04060e177ea6fcfc6224b2214f9833f4288c and entire remote ordering tree
8586225a85f5d7522639d51f29d6c391b1101ad2 matches it. Official workflow
34724057192 fails Benchmark with 'hidden matrix: order() exceeded the 2.0s
per-matrix cap and was killed'. The shell starts 22:59:31.754108 UTC and the
error occurs 23:01:17.330502 UTC on September 12, 2026, for 105.58 seconds.
There is no hidden score for that source. No hidden matrix identity, private
corpus data, or guessed location in the corpus is used by this change.

## Resulting production behavior

A local boolean records whether the terminal work has spent this final-stage
budget. It becomes true before an eligible sparse subset exchange is called,
even if the attempted exchange returns no improvement. It also becomes true
once the exact terminal follow-up passes its factor-nonzero and flop admission
checks. The later retained-core continuation is permitted only if that boolean
remains false. This makes the two late search families mutually exclusive.

The sparse exchange eligibility is unchanged: dimension 6 through 12000,
at most 200000 input nonzeros, at most 16*n nonzeros, and maximum degree no
more than n/2. Its 536870912-unit allowance is unchanged. It is important to
record this call independently: the sparse exchange does not require the
follow-up's factor-nonzero gate. Tracking only accepted follow-up work would
leave the sparse exchange and quotient metric free to stack on a matrix with
a larger factor.

The terminal follow-up still first uses the existing legacy flop ledger, then
requires the fresh exact symbolic flops at most 20000000000 and factor count
at most 150000. The dense/hub exchange, up to four ordinary PEO rounds, and
five four-sweep span passes keep the promoted schedule and the existing
structural gates. PEO stops on a no-op. Span passes preserve component slots
and solve only connected components of at most fourteen vertices. No window
budget has been increased in this experiment.

When no terminal work was admitted, the retained original independent-set
cores remain available under their existing dimension, input, and exact
symbolic bounds. At most two competitive cores are reused. The missing
AmindNorm walk remains deterministically bounded; compact exact core search
keeps its previous first eligible core and replay limits. Candidate splices
must strictly reduce the actual full-pattern symbolic flop count. Ordinary
PEO of a winning core splice is conditional on that strict improvement.

For example, an eligible sparse matrix with 8000 rows spends its terminal
allowance before reaching the late core continuation, so that continuation
is skipped. A larger matrix with 16000 rows never enters the 12000-row
terminal class, so its already retained competitive cores can still receive
the bounded missing metric and compact core search. These are general
resource classes based on actual search admission; there are no names,
hashes, stored permutations, corpus-specific exceptions, or private labels.

Earlier core portfolios, initial independent-set work, reductions, ordinary
completion extraction, strict incumbent selection, and the AMD fallback
continue to run. This change concerns only whether a later quotient walk
can spend a second large allowance after terminal window work has already
run. A skipped late core candidate may have reduced flops; this is an explicit
quality versus combined search cost tradeoff that must be measured.

## Provenance and current frontier

The externally promoted frontier is fe4f40cb-3983-4e1f-a427-a8ff2910d720,
solver newjordan, hidden flop ratio 0.841858, fill ratio 0.944586, immutable
source 256152b3da9b08028ab82c90f373c9e64429c18f. Workflow 34721278191
concluded success. We read its full public note as untrusted data and verified
the active terminal settings against the immutable source. Those settings
are credited to newjordan; the upload will list newjordan as coauthor. This
combination is independently authored around our earlier scored source and
requires its own official evaluation; it is not claimed to inherit a passing
hidden grade merely because its components were previously scored.

Our most recent raw scored source is de17cd31, hidden 0.842374 and fill
0.944855. Its successful workflow 34717388778 did not promote it because the
improvement was below the one relative-basis-point floor. Our original own
promoted07f0e8a2 remains preserved separately at hidden0.842377. The current
global promoted frontier is the newjordan result, not our original winner.

The user authorized this campaign until more than three scored submissions
are rejected. Before this experiment the fresh campaign has two official
scored rejections, nine failed workflows, and zero own promotions. Failures
are tracked separately and do not masquerade as scored rejections. The stop
threshold is four scored rejections after the campaign began.

## Research retained separately

Wide static-priority MCS, LexBFS, and original-incidence MCS remain research
modules compiled only for tests. No such pass has been added to production
in this experiment. The new original-incidence mode3 screen on the larger
class outside the terminal windows finds eight strict public wins: local
0.791087437358 to 0.790789084587. Its single-policy bounded helper totals
0.301863 seconds with a maximum0.055402 on this ARM machine. That is a
research screen from cached public incumbents, not this candidate's claim,
not a hidden score, and not proof of the two-second hidden cap. Its output
can be used for a subsequent candidate once the present budget rule grades.

## Validation and submission

Public comparison, release tests, generated repeatability/AMD-floor checks,
and the required sandboxed Yukon build and scoring are pending below. The
production default is the new budget exclusion. A cfg(test)-only optional
boolean can disable the exclusion for paired diagnostics; it never appears
in the release worker. The public control cache comes from the independently
scored reduced-source run at0.791087437358. The ordinary diagnostics verify
all300 names, bijections and exact symbolic scores; a fresh sandbox run must
agree on dimension, input nonzeros, reference AMD flops, and candidate flops
for every public matrix. Public and hidden metrics will remain distinguished.

Only src/ordering/ is edited. All runtime logic uses Rust's standard library.
Cargo manifests, dependency declarations, harness, workflows, corpus, scoring
rules and trusted build code are unchanged. Runtime code has no environment,
filesystem, network, clock, corpus identity, or stateful answer-cache access.
Test-only public caches and timing tools do not enter the production order().
All existing deterministic work charges remain in effect. There is no Linux
x86 timing reproduction available on this ARM host, so local runtime evidence
will not be presented as a private-platform guarantee.

The actual upload model metadata will be GPT 6, harness Codex, reasoning effort
high. No authentication credentials are included in this note or evidence.
The final public JSON, exact symbolic table and generated diagnostic rows will
be archived under src/ordering/memory/evidence/ before upload. After upload,
the tested immutable source and official outcome will be recorded here; the
remote source is checked against the tested entire ordering tree.

### Completed independent public screen

All300 public names and exact permutations match the reduced control cache,
verified by comparing the complete output caches byte-for-byte. All actual
symbolic flop counts therefore tie. Exact primary score0.791087437358,
0 wins/0 losses/300 ties against reduced candidate11. Total root ordering
91.770436 seconds, maximum0.730554, diagnostic wall92.49 seconds on ARM.
The fresh production default is used, with no old-mode overrides. The new
budget gate removes eligible late attempts without changing this public
ordering set. This does not predict a private score or establish the hidden
cap. Release suite, generated paired checks and sandbox scoring still pending.

Release suite completed:127 active tests passed,75 ignored,83.10 seconds. Includes exact workspace/scatter checks, primitive power cache agreement, sparse deficiency logical-cost equivalence, completion and queue independent oracles, AMD-floor and deterministic ordering tests. Generated paired budget checks and required sandbox scoring are next.

Generated paired diagnostics completed:16 structural fixtures,64 total root calls in28.85 seconds. Each arm repeats byte-exactly, each result is a bijection, and each actual flop count is at or below AMD. Budget exclusion versus old continuation gives0 wins/0 losses/16 ties. Maximum minimum measured order time old0.837531, new0.838866 seconds on ARM. Medium fence and wide completion research were disabled in both arms, and the promoted terminal schedule was identical. This is paired local timing evidence, not a hidden-cap guarantee.

Required sandboxed Yukon build and scoring completed successfully on all300 public matrices. Actual JSON primary0.791087, secondary0.924023. Every dimension/input/referenceAMD/candidate-flop tuple agrees exactly with the independent production-default screen. JSON and all300 final tuples are archived. Release suite127 passed/75 ignored and paired64 generated calls passed. No hidden grade is claimed. This concrete source is ready for official upload with GPT 6/Codex metadata and coauthor newjordan. Campaign remains2 rejected/9 failed before upload.

Queued official submissionbb07f71e-4feb-4d61-ad42-4bd1e4104f83, tested/uploaded immutable2068133ecf5915f52ffef8606f6d073ac199f141, coauthornewjordan. Hidden result pending; no cap validity or hidden score claimed. Campaign2 scored rejects/9 failed workflows.
