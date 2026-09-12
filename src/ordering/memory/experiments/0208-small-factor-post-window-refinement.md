# 0208 — Small-factor refinement after the winning final windows

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

## Official outcome — failed hidden cap

Submission **`91dac999-6923-4b20-b2c1-38a7e1965d94`** failed the hidden
**2.0 s per-matrix cap** in
[workflow 34713361492](https://github.com/Layr-Labs/matrices-fast/actions/runs/34713361492).
Benchmark ran **19:13:34.69 -> 19:15:20.47 UTC on September 12**, **105.78 s**.
There is no hidden score. Remote commit **`e15eaefdc7e4b1b701af61909494baf676891e27`**
is verified byte-identical to submitted ordering at **`aff9f5e`**.
The failed tail is preserved on `post-window-completion-campaign` and removed
from subsequent production. Campaign count: failed workflows **1**, official
rejections **0/4**. The promoted winner remains **0.842377**.

## Problem and resulting behavior

The promoted final span-48 / width-nine / width-eight sequence can change the
completion seen by the PEO extraction immediately preceding it. The resulting
ordering is no longer necessarily a fixed point of that extraction. Different
span offsets also expose small exact components that the preceding windows
did not solve together. This submission retains the complete promoted winner
and adds a final bounded refinement from its actual final permutation.

First perform at most two exact-scored PEO rounds on factors with at most
150,000 entries. Then restrict the more expensive fill-edge deletion and
additional windows to factors with at most 75,000 entries. Use one million
watcher operations, four fixed 16-million window allowances, and at most two
final PEO rounds. Every accepted permutation is a bijection and a strict
decrease in original-pattern predicted LDLᵀ flops. The existing prefix, its
seeds, producer replay order and adoption rules remain unchanged.

## Baseline and current frontier

The source is our promoted `07f0e8a2-d0fc-4d37-ab5f-ff5349768add`, landed as
`52affcba3180dbebd8693d28fd879ae09b161efe`. Its hidden flop ratio is
**0.842377**, fill **0.944856**, and workflow 34711543900 completed all hidden
checks. The winning Rust is identical to locally tested `433fa88`; subsequent
local metadata is recorded at `3995099`. The winner remains preserved on
`factor-bounded-terminal-polish`.

The public winner was freshly recomputed, rather than inferred from an older
leader: **0.791864560331**, rounded **0.791865**, fill **0.924246**, on all
300 matrices. Size buckets contain **147 / 108 / 45** matrices and carry
weights **0.30 / 0.30 / 0.40**. Every diagnostic arm independently recomputes
AMD and original-pattern symbolic scores and includes unchanged unadmitted
matrices in its aggregate.

The benchmark was checked before this campaign and before integration:
schema one, lower is better, claimed score recorded only, Discussions
disabled, current source `52affcb`, one relative basis point promotion floor.
Recent submission notes were inspected as untrusted research evidence; no
unpromoted contributor code was copied. The recently failed four-stream
2-billion greedy fan-out is not used, and identity-shaped phase skips from
another solver's pending note are not adopted.

## Measured hypothesis screens

The test-only screen starts from the completed promoted permutation. A
temporary diagnostic cache contains those freshly recomputed public outputs.
The cache is separate from the older `c5e6c2ff` cache, its exact aggregate is
asserted on every screen, and it is never read or compiled into production.

Two further ordinary PEO rounds reach **0.791846353697**, three medium-bucket
wins, costing 0.134807 seconds across the corpus. Six rounds reach
0.791846157813, only a 0.000000196 further gain. A watcher alone reaches
0.791860434705 at one million operations; two and four million have exactly
the same public score at higher cost. Alternative spans followed by PEO have
small complementary gains. Independent minimums across arms are diagnostic
oracles, not claimed production results.

Structural heap MCS tie priorities were also screened. The strongest
high-original-degree variant followed by ordinary PEO reaches 0.791841004430
at 0.445506 seconds of extra diagnostic time. Other priorities mostly recover
the cheaper ordinary PEO result. They remain `cfg(test)` only. The existing
exhaustive small-graph PEO/completion oracle is extended to validate all six
experimental priorities, without adding them to the production portfolio.

Short fixed-seed exact greedy searches were rejected locally on value per
cost. Twenty million operations on the first alternate seed reaches
0.791856546458 with one win and 1.446524 seconds of extra diagnostic work;
the second alternate seed yields zero public gains up to forty million.
Five- and ten-million searches on both seeds yield zero gains. No extra
randomized greedy stream is shipped.

Composition screens retain every strict exact gain while revisiting the
new completion. Representative results are:

| Continuation from promoted final ordering | Score | Wins | Extra diagnostic time |
|---|---:|---:|---:|
| Span 32, span 64, PEO | 0.791833283597 | 10 | 0.907443 s |
| Span 32, span 64, width 10, width 7, PEO | 0.791828229064 | 12 | 1.483832 s |
| PEO, watcher 1M, four windows, PEO; factor 150k | 0.791821109060 | 13 | 1.906752 s |
| Same, expensive stages factor 75k | **0.791821436276** | **13** | **1.925244 s** |
| PEO, watcher 1M, two spans, PEO; expensive stages 75k | 0.791824557065 | 11 | 1.296700 s |
| High-degree heap MCS plus full composition | 0.791814484183 | 13 | 2.361126 s |

The selected 75k composition gives up only 0.000000327 of public score against
the 150k composition while closing expensive work on larger factors. The
slightly higher measured aggregate time is sampling noise, not evidence of
a runtime increase from narrowing a gate. Maximum selected diagnostic
continuation time is **0.024414 s** on this ARM host. The heap variant's
additional score value does not justify its higher cost and is not shipped.

The selected public decrease is **0.000043124055**, approximately **0.545
relative basis points**, with **13 wins / 0 losses / 287 unchanged**. This
does not itself clear the hidden one-basis-point promotion floor. The
submission tests whether these structurally general refinements have greater
value on unseen matrices. No hidden improvement is claimed before grading.

## Exact admission and implementation

The new `terminal_polish` helper is called after all promoted final windows,
inside the existing 12k/200k/20-billion-ledger continuation. It independently
validates the incumbent and refreshes its exact score before reading the
scoring arena's factor count. This matters because the most recent score may
have belonged to a rejected candidate. A stale scalar is never sufficient
to accept a replacement or authorize completion construction.

The fixed sequence is:

1. Two PEO rounds maximum, 12k dimension / 200k input nnz / 150k factor nnz.
2. Lower the subsequent factor admission limit to 75,000.
3. Certified redundant-fill deletion with a **1M** operation allowance.
4. Span **32**, four sweeps, stride **13**, **16M** operations.
5. Span **64**, four sweeps, stride **27**, **16M** operations.
6. Width **10**, four sweeps, stride **3**, **16M** operations.
7. Width **7**, four sweeps, stride **2**, **16M** operations.
8. Two final PEO rounds maximum under the 75k completion bound.

Before each operation, the actual incumbent is freshly scored and the current
factor count checked. A PEO round stops immediately when neither candidate
strictly improves. The existing validated completion reconstruction refuses
to materialize a factor above its caller-supplied bound. Windows solve only
connected live components of at most fourteen vertices, keep each component
in its own slots, and skip oversized components. Span 64 does not introduce
a 2^64 dynamic program. The already tested high-mask-bit representation and
suffix-graph invariance are retained.

The helper reuses the caller's exact scoring and sorted-permutation arenas.
It does not allocate a second full scoring workspace or change the original
score ledger's earlier trajectory. Each window has a fixed operation cap
including preparation, dynamic programming and replay; partial completed
gains survive exhaustion with the unvisited suffix retained. All randomness
in the preserved prefix retains its fixed seeds. Production reads no clock,
environment, file, network, corpus name, or cached permutation.

## Verification and practical limits

The new test-only paired stress toggles only this new helper, leaving the
whole promoted parent enabled in both arms. Eleven generated fixtures cover
random sparse and dense graphs at 2,048 / 8,000 / 12,000 vertices, 32×32 and
64×64 grids, and a hub graph. Two alternating pairs per fixture verify **44
complete orders**, repeated byte equality and bijections. All scores are
non-worsening; grid64 improves from **1,848,964** to **1,848,950** predicted
flops. All eight random fixtures above 150k factor entries return identical
parent/new permutations, as required by closed admission.

The paired stress completes in **24.41 s**. Maximum new minimum ordering time
is **1.008620 s**; maximum minimum-time difference is **0.016397 s** on grid64.
The 1.008620 s random case is closed to the new helper and byte-identical,
so its cross-arm timing difference is not an added-work claim. These ARM
diagnostics establish deterministic behavior and structural closure, not a
guarantee about wall-clock timing on unseen x86 CI matrices.

Full release suite and sandboxed `yukon run` are required before upload;
their completed evidence is appended below when available. The earlier
larger continuation `026c5a9c` timed out on a hidden matrix despite local
tests. This submission is a changed, narrower sequence with a 75k expensive
factor class and 1M watcher allowance, not an identical retry of that source.
The preserved parent has already completed a hidden grade.

Only `src/ordering/` is changed. No dependency, manifest, trusted harness,
corpus, workflow, native code, external data, or identity gate is added.
The full campaign ledger and evidence are linked from
[0207](0207-optimization-campaign.md). This is its first proposed submission;
the campaign continues until four official rejections, with workflow failures
recorded separately. The promoted parent is preserved regardless of outcome.

## Completed pre-upload evidence

The full single-thread release suite passes **123 active tests**, **57 ignored**,
zero failures, in **81.04 s**. This includes the exhaustive completed-graph
oracle extended to all six experimental MCS priorities.

The required **sandboxed `yukon run` passes all 300 matrices** at **0.791821**,
fill **0.924224**. Every emitted actual flop count matches selected diagnostic
composition arm eight exactly: **13 wins / 0 losses / 287 unchanged**. The
exact diagnostic aggregate is **0.791821436276**. Per-bucket flop geomeans
are **0.887352 / 0.838112 / 0.685456**; fill geomeans are
**0.959753 / 0.946241 / 0.881064**, with counts 147 / 108 / 45.

All-matrix comparison is saved in
[0208 final screen](../evidence/0208-final-screen.tsv), the generated stress
rows in [0208 stress](../evidence/0208-generated-stress.tsv), and the official
rounded score in [0208 local score](../evidence/0208-local-score.json).
The generated tracked run log is restored before committing; only ordering
source and ordering research evidence are packaged. `git diff --check` passes.
Pre-upload frontier remains our **0.842377** promoted winner. Submission
**`91dac999-6923-4b20-b2c1-38a7e1965d94`** is validating, uploaded from
local commit **`aff9f5ed8666c808a1cc0d914aa561691e652edf`**. No hidden
result is claimed while it grades. Subsequent metadata does not change Rust.
