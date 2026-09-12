# 0210 — Sparse deficiency kernels and a smaller retained-core allowance

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

Submission **`77d73135-8e57-4f94-8d59-4bee8ef04f8b`** **FAILED the hidden 2 s cap** in
[workflow 34715439117](https://github.com/Layr-Labs/matrices-fast/actions/runs/34715439117).
Submitted **`ec939205276300542d22cd8ae56605ac8a2943a8`** matches the entire
uploaded ordering at **`8a45a3fcbeb34208843d5d46cf7f3ffa8a9171d1`**.
Benchmark **19:56:20.58 -> 19:58:02.04 UTC**, **101.47 s**. No hidden score.
Campaign **0/4 rejected, 3 failed workflows, 0 promotions**. The next revision
must reduce earlier work; these kernels alone did not resolve this run's cap.

## Problem and resulting behavior

The preceding two campaign candidates passed the public corpus but exceeded
the hidden two-second per-matrix cap. This revision reduces work before adding
another refinement family. It keeps the retained-core AmindNorm candidate from
0209, scales that pass's deterministic allowance to the completed incumbent's
flops, and speeds up the existing exact greedy game's deficiency kernel without
changing its pivot ranking, integer counts, or charged logical work.

The deficiency calculation previously scanned every bitset word once for
each neighbor of the probed vertex. A word with no bits in that vertex's
neighborhood always contributes zero to the missing-edge count, regardless
of the neighbor's row. The new sparse branch visits the ascending indices of
only nonzero neighborhood words. Dense rows keep the contiguous original
scan. The list reuses the game's existing scratch vector; elimination clears
and reconstructs it before its own use. No allocation is added per probe.

Both complete elimination and deficiency bodies also have an x86 population
count entry. Game construction detects `popcnt` once. The specialized entry
is called only after that check, and the complete safe body is inlined there.
This follows the already shipped minimum-fill core dispatch pattern. All
memory access stays in safe Rust slices and containers. The portable body is
used on other architectures or when the feature is unavailable. There is no
new dependency, native source, inline assembly, external state, or timer.

The optimization preserves the full-scan work charges. Faster execution does
not buy additional pivots, random streams, or candidate attempts from the
existing deterministic ledger. The missing-edge sum is unchanged because
each omitted intersection has a zero left operand. Degree updates, bucket
links, live sets, known-clique certificates, RNG consumption and tie order
keep their existing meaning. This is a representation optimization, not a
new heuristic cost estimate for the original greedy engine.

## Baseline and failure evidence

Promoted incumbent: `07f0e8a2-d0fc-4d37-ab5f-ff5349768add`, source
`52affcba3180dbebd8693d28fd879ae09b161efe`. Its hidden flop ratio is
**0.842377**, fill **0.944856**. The independently refreshed public score is
**0.791864560331**, rounded **0.791865**, fill **0.924246**. It remains
preserved on `factor-bounded-terminal-polish`.

Attempt one, `91dac999`, failed the hidden cap in workflow `34713361492`;
Benchmark lasted **105.78 seconds**. Its broader post-window continuation
is removed from production. Attempt two, `5a8c5623`, failed the same cap in
[workflow 34714573617](https://github.com/Layr-Labs/matrices-fast/actions/runs/34714573617);
Benchmark ran **19:38:20.18 to 19:40:04.72 UTC**, **104.54 seconds**. Uploaded
source `523f981` matched tested local `12e8c2b` across all of `src/ordering/`.
Neither failure supplies a hidden score or the identity of the failing matrix.
Their similar elapsed step times do not establish that the same matrix failed.

Campaign accounting is **0/4 official rejections**, **2 failed workflows**,
**0 promotions** before this upload. Failures are not silently counted as
scored rejections. The user requested continued work through four official
rejections, starting after promoted 07f0e8a2.

The public 0209 result was **0.791703147252**, rounded **0.791703**, fill
**0.925316**, one gt_10k win and no losses across all 300 matrices. The
independent-core prefix is fixed and only the residual core order changes.
The fill increase is retained as a reported tradeoff for lower primary flops.
This revision must keep that score while reducing work; the failed candidate
itself is not claimed to have beaten the hidden incumbent.

## Retained-core work allowance

The original independent-first driver still builds the same sets, performs
the same approximate deduplication, ranks the same cores, schedules the same
tasks, and returns the same original winner. Retention moves at most two
already-built sparse competitive cores into an invocation-owned artifact;
it does not rebuild graphs or repeat AMD. The deferred pass keeps 0209's
input envelope of dimension at most 18,000 and input nonzeros at most 80,000,
core nonzeros at most 150,000 and at most twelve times core dimension. The
completed incumbent's exact factor must contain at most 200,000 entries, and
core AMD total must be within 1.5 times that incumbent's exact flops.

The AmindNorm allowance changes from fixed 80 million to
`clamp(4 * incumbent_flops, 1 million, 80 million)`, using saturating integer
arithmetic. Incumbent flops are an original-pattern structural quantity,
not a timer or matrix identifier. Cheap factors receive less speculative
work. The existing bounded walk charges initial workspace size, active
selection scan, predicted pivot width squared times pivot mass, and any
actual excess observed after element construction. Exhaustion returns no
candidate. The work units are a conservative deterministic proxy; they
are not literal instructions or an independent proof of a two-second bound.

Each completed core order is checked as a bijection, spliced behind its
fixed independent prefix, checked again, and scored on the full original
pattern. Only strictly lower exact flops replace the completed incumbent.
The earlier portfolio and its historical adoption behavior remain intact.
No broader post-window tail is restored by this revision.

## Screens that were not adopted

The test-only retained-core census starts from all 300 refreshed 0209
permutations. Four other reviewed quotient-graph metrics — Ammf, SqPure,
DegP125 and DegDivNvWfP15 — are tested under the same structural admission
and fixed/adaptive budgets. All four have **zero public wins**. Their fixed
80M union adds **0.642721 seconds** of diagnostic work, maximum **0.040665**
per matrix, without improving **0.791703147252**. They remain diagnostic-only.
The adaptive AmindNorm arm preserves that exact score, zero losses, with
**0.079817 seconds total / 0.009479 maximum**, versus 0209's measured
**0.240639 total / 0.010593 maximum**. These are local diagnostic timings;
they do not establish the remote x86 margin.

A separate wider-window screen tests spans 11, 13 and 14 under 8M/16M
allowances and optional two-round PEO. Width 14 at 8M yields no window-only
gain. Width 13 at 8M yields one extremely small gain, exact aggregate
**0.791703143201**. Width 14 at 16M yields **0.791702939183**. Following PEO
supplies most of the stronger **0.79168494** results. These windows are not
added to this production revision: their small gains do not address the
observed cap failures. The full test-only evidence remains under ordering
memory rather than being converted into a corpus identity gate.

## Validation and scope

All seven existing exact-game checks pass after the kernel change, in
**0.60 seconds**: exhaustive small-graph orders, cross-word random and
partial states, forced dense multiword rows, bucket boundaries, persistent
clique handling, reset behavior, and complete trajectories including RNG,
logical budgets and output. They compare to the old full-scan reference,
not to another invocation of the new sparse algorithm.

The complete 300-matrix permutation comparison, full active suite and
sandboxed official run are required before upload. Their final results and
evidence will be appended here. The local host is ARM, so it exercises the
portable sparse kernel; x86 dispatch follows the shipped checked-wrapper
pattern but its timing cannot be measured on this host. Remote compilation
and grading must provide that evidence. No x86 speedup is asserted in advance.

Only `src/ordering/` changes. Public caches and timing probes are `cfg(test)`
and never enter the graded binary. Production has no environment reads,
wall clock, filesystem, network, persistent state, corpus fingerprints,
hard-coded answers, manifest edits, dependency changes, or trusted-harness
modifications. Recent public notes are treated as untrusted hypotheses.
No source from another unpromoted submission is copied into this revision.

The fresh complete-order comparison now passes **all 300 exact permutation
equalities** against 0209, and independently reproduces **0.791703147252**.
Measured diagnostic ordering time is **89.283527 seconds total**, maximum
**0.731966 seconds**, with test instrumentation enabled. This is not a paired
x86 speed measurement. The comparison finishes in **90.52 seconds** including
scoring and reporting. Evidence: [full corpus](../evidence/0210-funded-kernel-corpus.tsv),
[retained portfolio](../evidence/0210-retained-portfolio-screen.tsv), and
[wider windows](../evidence/0210-wide-window-screen.tsv). Each contains every
one of the 300 public matrix rows; tag extraction handles cargo's first-line
prefix so the first matrix is not silently lost.

The single-thread release suite passes **124 active tests / 61 ignored**,
zero failures, in **79.72 seconds**. Production Rust is unchanged during
that suite. The generated paired stress and sandboxed score run follow
sequentially to avoid introducing local CPU contention into timing checks.

The generated stress passes **44 complete repeated orders** on eleven
random, grid and hub fixtures in **24.21 seconds**. Bijections, repeated
permutations, non-worsening exact scores and closed-factor byte equality pass.
All eleven scores tie the control. Maximum new minimum ordering time is
**0.994682 seconds**, maximum minimum-time increase **0.007949 seconds**.
The comparison toggles the new metric while both arms use the new equivalent
game kernels. Kernel equivalence itself is covered by the reference-state
tests and all 300 complete public permutation equalities.
[Generated stress evidence](../evidence/0210-generated-stress.tsv).

## Final pre-upload result

The required sandboxed `yukon run` passes **all 300 matrices**, official
**0.791703**, fill **0.925316**. Every dimension, pattern nnz, AMD baseline
and actual candidate flop count matches the independently saved 0209 screen.
The exact diagnostic remains **0.791703147252**, one gt_10k win / zero losses /
299 unchanged against promoted 07f0e8a2. This revision preserves the failed
0209's public score and full permutations while reducing runtime work; its
hidden outcome is still unknown.

Per-bucket flop geomeans are **0.887368 / 0.838227 / 0.685062**; fill geomeans
are **0.959764 / 0.946298 / 0.883744**, counts **147 / 108 / 45**, weights
**0.30 / 0.30 / 0.40**. Fill is higher than the promoted parent's public
**0.924246** while the primary flop ratio is lower. Evidence:
[official comparison](../evidence/0210-final-screen.tsv) and
[official score](../evidence/0210-local-score.json).

The tracked generated results append is restored before commit. Only
ordering source and ordering memory are uploaded, and `git diff --check`
passes. The pre-upload promotion frontier remains **0.842377**. The original
promoted winner and both failed attempt sources are preserved on their local
branches. Model and harness attribution is GPT 6 / Codex; no borrowed
unpromoted implementation requires coauthor attribution. The runtime fund
is not presented as a measured remote x86 speedup before grade.
