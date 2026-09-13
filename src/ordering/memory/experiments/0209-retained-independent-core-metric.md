# 0209 — Bounded late metric on retained independent-set cores

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

Submission **`5a8c5623-8104-47e8-9833-7903495f7caa`** **FAILED the hidden 2 s cap** in
[workflow 34714573617](https://github.com/Layr-Labs/matrices-fast/actions/runs/34714573617).
Submitted source **`12e8c2b909653efcd836fe681883efbc44a6060a`** matches the
entire uploaded ordering at **`523f981fd6fb79a3c1ab985a659ce9d81c05501d`**.
Benchmark **19:38:20.18 -> 19:40:04.72 UTC**, **104.54 s**; no hidden score.
Campaign counters are **0/4 rejected, 2 failed workflows, 0 promotions**.
The next revision must reduce runtime before adding further work.

## Problem and resulting behavior

The independent-set-first stage already builds and ranks several exact Schur
complements. Its metric portfolio omits AmindNorm. Rebuilding all alternative
sets after the complete pipeline would repeat expensive lift and AMD work.
This candidate instead retains at most two competitive cores already built
by the promoted stage and applies a bounded AmindNorm walk after the winning
pipeline has completed. It adopts only a strict decrease in exact original
matrix predicted LDLᵀ flops.

The prior campaign tail is removed from production. This starts from the
promoted `07f0e8a2` Rust, preserves its ordering trajectory, and adds deferred
ownership of existing core artifacts plus the final bounded metric. No new
independent-set construction, METIS run, randomized search, or dependency is
added to production. Existing set deduplication and earlier core rankings are
preserved. The wider exact-distinct census was a diagnostic lead, not a source
change adopted without measurement.

## Baseline and failed first attempt

Promoted target: **`07f0e8a2-d0fc-4d37-ab5f-ff5349768add`**, landed source
**`52affcba3180dbebd8693d28fd879ae09b161efe`**, hidden flop ratio **0.842377**,
fill **0.944856**. The public target is freshly verified at
**0.791864560331**, rounded **0.791865**, fill **0.924246**. All 300 matrices
are included; bucket counts are 147 / 108 / 45 with weights 0.30 / 0.30 / 0.40.
The promoted winner remains preserved on `factor-bounded-terminal-polish`.

The first campaign submission, `91dac999`, passed all 300 public matrices at
0.791821 and its release/stress checks, but failed the hidden 2 s cap in
[workflow 34713361492](https://github.com/Layr-Labs/matrices-fast/actions/runs/34713361492).
Benchmark lasted **105.78 s**, 19:13:34.69 -> 19:15:20.47 UTC on September 12.
There is no hidden score. That exact failed source is preserved on
`post-window-completion-campaign`; its tail module is now test-only and is
not executed by this production candidate.

Campaign accounting remains **0 / 4 official rejections**, **1 failed
workflow**, **0 promotions**. These counts start after the user's latest
continued-optimization authorization. Earlier historical rejections are not
included. The campaign continues until four official rejected outcomes.

## Diagnostic hypothesis and cost reduction

A recent public submission note suggested additional core metrics, but used
specific instance bands. We copied neither code nor instance gates. Our
screen uses the parent's full light-input envelope, **32 <= n <= 18,000**,
**input nnz <= 80,000**, plus explicit work bounds. The current benchmark is
schema one, lower is better, claimed score recorded only, Discussions
disabled, and requires one relative basis point for promotion.

An independent census reconstructs exact-distinct independent sets, ranks
their cores by AMD total, and compares additional METIS and quotient metrics.
It verifies every spliced score against full-pattern symbolic analysis.
The larger census finds a public score **0.791660025456** relative to the
failed-tail diagnostic baseline 0.791821436276. Its added gain comes from
AmindNorm on the second-ranked existing core. Additional METIS passes produce
no public gain. Rebuilding the core portfolio costs several seconds across
the public corpus, so that reconstruction is not shipped.

The retained-core driver is paired against the original driver on all **218**
public inputs in the light envelope. Every original returned flop total and
permutation is byte-identical. Core retention occurs after the original
phase-two result has been selected; no earlier candidate, score, tie order,
thread reduction, or adoption rule changes. The two retained cores are the
lowest AMD totals, within the original **1.5×** competitive margin, with
**core nnz <= 150,000** and **core nnz <= 12 × core dimension**.

The finished full-pattern factor must contain **at most 200,000 entries**.
The current completed incumbent, rather than an earlier portfolio scalar,
defines the **1.5× AMD competitiveness check** for the additional walk.
The existing 20-billion scalar gate remains a conservative early rejection;
it is never sufficient to adopt a candidate. Every adoption uses exact
original-pattern scoring.

## Bounded metric execution

Unbounded late walks retain the measured gain but have a 0.134746 s public
outlier. Sparse-core and competitiveness filters alone do not remove it:
AmindNorm can produce a poor completion even when the same core's AMD ordering
is competitive. The new entry point therefore supplies a fixed **80M
work-unit allowance** for each of at most two additional metric walks.

The existing `order_variant` entry point and elimination loop are unchanged.
The bounded sibling initializes the same workspace and pivot-score caches.
It charges setup from dimension and input nnz. Before creating an element,
it charges the estimated pivot width squared times supervariable mass, or
the remaining-vertex scan allowance, whichever is larger. After element
creation it charges any larger actual pivot-width cost before finalization.
Checked integer subtraction abandons exhausted candidates. These units are
a deterministic work proxy, not a claim of exact machine instructions or a
wall-clock guarantee. There is no clock gate or machine-load-dependent choice.

Exhaustion returns no candidate and the caller keeps its valid incumbent.
A completed walk uses the same pivot selection, element construction,
score-update function and final permutation as the original metric. The
full suite includes a meaningful comparison across five generated graph
families, eleven metric variants, both aggressive settings and two density
settings: **220 configurations**, checking unbounded and completed bounded
permutations against the original. Zero allowance and post-setup exhaustion
are checked on a cycle, followed by a fresh successful run.

The initial 64M estimated-width allowance kept the gain and cut maximum added
time to 0.010318 s. Stronger scan/actual-width accounting at 64M dropped the
winning candidate. The selected **80M** accounting restores it. On the
independent public screen it costs **0.240639 s total**, **0.010593 s maximum**,
versus the 0.13–0.16 s unbounded outliers. These are ARM diagnostic measurements
that include candidate scoring; no x86 speedup or hidden-cap guarantee is
inferred from them.

## Exact terminal observations and result

The original final continuation already computes the exact accepted
incumbent's flops and factor count. The candidate carries those observations
forward as a pair, updating it only after strict accepted replacements.
Rejected candidate scores do not overwrite the accepted pair. This avoids
another eligibility score where the original continuation already observed
the incumbent. Other eligible inputs receive a fresh symbolic score. Test
builds independently re-score and assert both cached values before use.
The original legacy scalar and earlier adoption trajectory remain unchanged.

Selected independent public result: **0.791864560331 -> 0.791703147252**,
**one gt_10k win / zero losses / 299 unchanged**, a **0.000161413079**
decrease, approximately **2.04 relative basis points**. All 300 original AMD
ratios contribute to the aggregate, including closed inputs. Evidence is in
[0209 retained final screen](../evidence/0209-retained-final-screen.tsv).
The larger census and unbounded cost measurements are linked from
[0207](0207-optimization-campaign.md). No hidden result is claimed before grade.

The final paired stress toggles only the new metric and preserves core
retention/admission in both arms. Eleven generated random, grid and hub
fixtures produce **44 complete repeated orders**. Bijections, determinism,
non-worsening scores and closed-factor byte equality all pass. All eleven
scores equal the parent on this stress suite. Maximum new minimum ordering
time is **0.988731 s**, maximum minimum-time increase **0.011252 s**. The
stress completes in **24.22 s** and is saved in
[0209 generated stress](../evidence/0209-generated-stress.tsv).

Only `src/ordering/` changes. New code uses standard-library ownership,
integer accounting and containers with the existing reviewed ordering and
scoring helpers. No dependency, manifest, trusted harness, corpus, workflow,
native code, persistent solver state, external data, identity gate, or cached
answer is added. Retained graphs belong to one invocation, with at most two
small CSR copies already built by the original stage. Test-only public seed
files never enter the graded binary.

## Required pre-upload checks

The final single-thread release suite passes **124 active tests / 59 ignored**,
zero failures, in **80.03 s**. The bounded-metric equivalence/exhaustion check
passes. The required **sandboxed `yukon run` passes all 300 matrices** at
**0.791703**, exact diagnostic **0.791703147252**. Every actual flop count
matches the independent screen: one gt_10k win, zero losses, 299 unchanged.
The fill tiebreak **increases from 0.924246 to 0.925316**; the candidate trades
some additional factor entries for lower squared column-width cost. Flops are
the primary objective, so this is not reported as a fill improvement.

Per-bucket flop geomeans are **0.887368 / 0.838227 / 0.685062**; fill geomeans
are **0.959764 / 0.946298 / 0.883744**. Counts remain 147 / 108 / 45.
[All-matrix comparison](../evidence/0209-final-screen.tsv) and
[official score](../evidence/0209-local-score.json) are saved. The generated
tracked results log is restored before commit; only ordering source and
ordering evidence are included. `git diff --check` passes. The pre-upload
frontier remains **0.842377** at promoted `07f0e8a2`; no hidden result is
claimed before grading.
