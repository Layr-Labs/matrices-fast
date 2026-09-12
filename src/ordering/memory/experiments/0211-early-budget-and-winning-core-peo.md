# 0211 — Earlier high-flop budget, lazy runner-up retention and winning-core PEO

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

Submission **`30eb4c76-9ae7-42a2-908b-74e3787c6ae4`** **REJECTED, tied incumbent**, in
[workflow 34716429064](https://github.com/Layr-Labs/matrices-fast/actions/runs/34716429064).
Submitted **`9bc3e9452423db39ee476705bd09d1788153633b`** matches the entire
uploaded ordering at **`730dfd052df11500a3d73c1ff37806147c69314c`**.
Workflow SUCCESS, Benchmark **20:15:44 -> 20:24:40 UTC**, approximately **536 s**.
Hidden **0.842377**, fill **0.944856**, same as promoted 07f0e8a2. All hidden
caps and determinism gates passed; no primary score improvement. This is the
campaign's first scored rejection: **1/4 rejected, 3 failed, 0 promotions**.

## Problem and resulting behavior

Three continued-optimization candidates passed the public development corpus
but failed the hidden two-second per-matrix cap. Their opaque receipts give
neither the failing matrix nor phase costs. This revision reduces an earlier
portfolio budget instead of assuming a small terminal refinement is the
cause. It also removes doomed runner-up permutation clones while preserving
that ledger exactly, and polishes strictly winning retained-core metrics
with one bounded PEO extraction from their newly produced completion.

The incumbent high-flop fence changes from the first **64** deferred producers
per batch to the first **32**, still only when the batch-entry incumbent's
predicted flops exceed **20 billion**. Threshold, source-order replay, batch
head and downstream gates remain the same. The highest AMD cost in the public
300-matrix corpus is **6,179,750,712**, so this change does not buy a public
score by recognizing an evaluation instance. It is a broad structural budget
for factors whose work greatly exceeds the input graph's apparent sparsity.
It can reduce ordering quality on unknown high-flop graphs; that tradeoff
must be assessed by a completed official grade, not inferred from failed runs.

The runner-up ledger previously cloned each losing producer's entire
permutation, appended it, stable-sorted by score, removed equal-score duplicates,
and kept eight entries. The new helper binary-searches the maintained sorted
score list before asking a closure to clone the permutation. Equal scores
keep the already retained first entry, matching stable sorting and deduplication.
An insertion below the eight-entry cutoff preserves source-order tie behavior;
an insertion beyond the cutoff is discarded without allocating a permutation.
When a producer improves the best, the same displaced incumbent is offered
to the ledger. Every downstream alternate PEO/donor consumer sees the same
score/permutation list. This change does not prune candidate generation or
alter the batch's winner selection.

The deferred AmindNorm pass from 0210 remains: reuse at most two sparse
independent-set cores already built by the original driver, input dimension
at most 18,000 and pattern nonzeros at most 80,000; core nonzeros at most
150,000 and twelve times core dimension; completed incumbent factor at most
200,000 entries; core AMD total within 1.5 times exact final incumbent flops.
Its deterministic allowance is `clamp(4 * incumbent_flops, 1M, 80M)`. A failed
or exhausted bounded metric yields no candidate. Completed orders must be
bijections and strictly reduce full original-pattern flops before adoption.

Only when that late metric wins, the new PEO helper reconstructs its chordal
completion, extracts forward and reverse MCS PEOs, and accepts strict exact
improvements. Materialization is bounded by **18k dimension / 80k pattern nnz /
200k factor entries / 20-billion exact flops**. No subsequent full search is
started. The helper refreshes the accepted permutation's score immediately
before reading the scratch workspace's factor count, so a rejected candidate
cannot supply eligibility data. One round is selected after measuring that
two and four rounds do not improve the public score further.

## Frontier and campaign evidence

Current promoted winner: **`07f0e8a2-d0fc-4d37-ab5f-ff5349768add`**, source
**`52affcba3180dbebd8693d28fd879ae09b161efe`**, hidden **0.842377**, fill
**0.944856**. Public baseline is independently refreshed at
**0.791864560331**, official **0.791865**, fill **0.924246**. The winning
source is preserved on `factor-bounded-terminal-polish` throughout this campaign.

- Attempt 1, `91dac999`: hidden cap failure in `34713361492`, Benchmark
  **105.78 s**. Its broader post-window tail is removed from production.
- Attempt 2, `5a8c5623`: hidden cap failure in `34714573617`, Benchmark
  **104.54 s**. Its retained-core metric improved the public score but supplied
  no hidden score.
- Attempt 3, `77d73135`: hidden cap failure in
  [workflow 34715439117](https://github.com/Layr-Labs/matrices-fast/actions/runs/34715439117),
  **19:56:20.58 to 19:58:02.04 UTC Benchmark**,
  **101.47 s**. Entire uploaded ordering `8a45a3f` matched tested `ec93920`.
  Sparse-word deficiency and checked x86 popcnt entries compiled successfully
  remotely, but those kernels alone did not resolve this run's timeout.

There is no hidden score for any of these failures, and their similar elapsed
step times do not identify a failing matrix. Campaign accounting before this
upload is **0/4 official rejected outcomes, 3 failed workflows, 0 promotions**.
Historical outcomes before the latest authorization are not counted. The user
requested continued optimization until four official rejections in this campaign.

## Measured postprocessing screen

The current retained-core public control is **0.791703147252**, one gt_10k
win and zero losses against promoted 07f0e8a2. The screen freshly scores all
300 cached permutations and constructs the original retained cores only under
the existing structural envelope. Caches are diagnostic input, never solver
answers embedded in production. It tests reverse labels, ascending and
descending degree labels, three fixed xorshift shuffle seeds, and original
labels at AmindNorm dense alpha 1, 2.5 and -1. Each candidate is spliced,
bijection-checked and rescored on the full original pattern before comparison.

One-round PEO on the winning control reaches **0.791635371701**, one further
gt_10k win, zero losses. Two and four rounds reach exactly the same aggregate.
The one-round screen measures **0.006370 seconds** extra work on its sole
winning control. Two rounds measure **0.010956 seconds**. These are local
diagnostic timings, not a remote cap guarantee. The measured choice is one
round, not the maximum explored depth.

The public mover is edgecross24-115, n 16,747 / pattern nnz 77,352. Promoted
07f0e8a2 flops are **28,685,704**; the deferred metric reaches **27,935,538**
with **185,122 factor entries**, and one-round PEO reaches **27,626,301**.
This observation motivates completion polishing of any late winner under
the same broad structural limits, not a branch on this matrix's identity.

The best raw relabeled/threshold arm is original labels at alpha 2.5,
**0.791676188056**, one win against the raw metric control. Its union with
the other arms followed by PEO reaches **0.791641578999**, which is worse
than polishing the original raw winner. Reverse and ascending-degree labels,
two of the three shuffles, alpha 1 and alpha -1 have zero public gains.
The descending-degree arm reaches **0.791702924252**, and the third shuffle
**0.791686799843**. These alternatives are not added to production. Raw
candidate ranking and completed-candidate ranking need not have the same best
basin; keeping the measured polished control avoids paying all nine passes.

Evidence includes [all postprocessing rows](../evidence/0211-postprocessing-screen.tsv),
[one-round selection](../evidence/0211-round-one-screen.tsv), and
[four-round selection](../evidence/0211-round-four-screen.tsv). Every screen
contains all 300 matrix rows, with cargo's first-line prefix handled explicitly.

## Equivalence, cap validation and scope

The new lazy runner-up helper is compared after every event with the original
stable append/sort/dedup/truncate policy across capacities 0 through 9 and
2,000 descending/random/repeated-score offers at each capacity. It covers
the production capacity eight. Duplicate and noncompetitive offers use
panicking clone closures to prove those closures are never evaluated. Equal
scores retain the old first permutation rather than whichever job finishes
first. Candidate replay order remains the established source order.

Fresh full-order comparison, active release suite, standalone generated PEO
stress, generated complete-order stress and official sandboxed run are
required before this upload. Final results will be appended when available.
Unlike the prior pure representation candidate, the 64-to-32 high-flop
budget changes the search set on unknown expensive factors. It deliberately
seeks runtime margin, with ordering quality assessed by completed grading.
The last promoted source is retained and is not overwritten by an ungraded
local hypothesis.

Only `src/ordering/` changes. New implementation uses Rust standard-library
containers, lazy closures, integer structural limits and the existing reviewed
ordering/scoring helpers. No dependency, manifest, build script, native code,
trusted harness, corpus, workflow or sandbox setting changes. Production has
no wall-clock gate, environment read, filesystem, network, persistent state,
matrix fingerprint, cached answer or identity-specific branch. Public cache
files and timing probes are `cfg(test)` and absent from the graded binary.
No implementation is copied from another unpromoted submission; recent notes
are treated as untrusted research claims. Model/harness is GPT 6 / Codex.

Fresh complete-order comparison now passes all **300 full permutation
equalities** against the independently derived one-round polished control.
It reproduces **0.791635371701**, one further win / zero losses against 0210;
total diagnostic ordering **89.521700 seconds**, maximum **0.729791 seconds**,
test completion **90.97 seconds**. Evidence:
[complete public corpus](../evidence/0211-early32-final-corpus.tsv).

The full single-thread release suite passes **125 active tests / 64 ignored**,
zero failures, in **80.23 seconds**. Production Rust is unchanged during the
suite. The standalone PEO stress covers six path, hub, square-grid and
rectangular-grid fixtures, AMD and reversed AMD, and both one and two rounds:
**24 cases / 48 repeated helper calls**, finishing in **0.13 seconds**.
Bijections, determinism, strict adoption and exact factor/flop cutoff skips
pass. Fourteen materializations are observed, seven above the old 12k
dimension ceiling; thus the higher-dimension path is exercised directly.

The complete-order generated stress passes **44 repeated orders** on eleven
random, grid and hub patterns in **22.97 seconds**. Bijections, determinism,
non-worsening late refinement and closed-factor byte equality pass. All scores
tie the control. Maximum new minimum time is **0.855383 seconds**; maximum
minimum-time increase **0.007518 seconds**. Both arms share the early cap32
and equivalent kernels while toggling only the late retained-core stage.

An independent full-order budget comparison toggles early cap **64 versus 32**
while keeping the new late stage enabled in both arms. It passes **44 repeated
orders**, **0 wins / 0 losses / 11 same scores**, in **23.56 seconds**.
The high-flop fixtures exercise the fence rather than only its closed side:
random n8000/links8, **46,260,820,576 flops**, minimum **0.998492 -> 0.799507 s**;
random n12000/links4, **58,806,390,558 flops**, **0.746160 -> 0.554489 s**.
Random n8000/links10, **58,257,439,844 flops**, **0.557260 -> 0.550664 s**.
These local repeated comparisons show real headroom on two large-fill
generated cases without score loss. They do not prove hidden quality or cap
safety on an arbitrary unseen graph. Evidence:
[direct PEO stress](../evidence/0211-generated-post-core-stress.tsv),
[complete-order stress](../evidence/0211-generated-complete-stress.tsv), and
[64/32 budget comparison](../evidence/0211-generated-cap-stress.tsv).

## Final pre-upload result

The required sandboxed `yukon run` passes **all 300 matrices** at official
**0.791635**, exact diagnostic **0.791635371701**, fill **0.924192**. Every
dimension, input nnz, AMD value and candidate flop count matches the saved
independent full-order screen. Compared with 0210 there is one further gt_10k
win, zero losses and 299 unchanged; compared with promoted 07f0e8a2 the
public score improves from **0.791864560331**, roughly **2.89 relative basis
points**. Fill improves from 0210's **0.925316** and the promoted parent's
public **0.924246**. The new completion drops unneeded filled edges in this
public winner; the result is not just a lower squared-width cost with more fill.

Bucket flop geomeans are **0.887368 / 0.838227 / 0.684893** and fill geomeans
**0.959764 / 0.946298 / 0.880934**, counts **147 / 108 / 45**, weights
**0.30 / 0.30 / 0.40**. Evidence:
[official all-matrix comparison](../evidence/0211-final-screen.tsv) and
[official score](../evidence/0211-local-score.json).

The current hidden frontier is rechecked immediately before upload and remains
**0.842377** at promoted 07f0e8a2. No hidden improvement or remote cap safety is
claimed in advance. Only ordering source and evidence are committed; the
generated tracked results append is restored and `git diff --check` passes.
The note reports both the public gain and the general high-flop budget tradeoff.
