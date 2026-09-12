# 0212 — Compact exact search on retained independent-set cores

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

## Problem and resulting behavior

The original independent-first portfolio and the deferred AmindNorm candidate
use approximate quotient-graph orderers on their Schur complements. The full
matrix can exceed the exact greedy game's dimension limit while its retained
residual core fits. This revision explores that different elimination game
inside at most one already built, competitive sparse core, without changing
the original stage's winner or the preceding search trajectory.

After the complete 0211 ordering, the finished permutation is projected onto
each of at most two retained core vertex sets. The core vertices keep their
relative order and follow the fixed independent prefix. These projected
orders are actual new full-matrix candidates, scored exactly and adopted only
on a strict decrease. The inverse map is invocation-local, indexed by original
vertex id; it contains no corpus names or precomputed answers.

The core seed is symbolically scored on its own residual pattern. Independent
prefix cost is fixed, so `prefix_flops + core_flops` equals the spliced full
candidate's exact score. A fill-free core reaches `n + 3*edges + 2*triangles`;
the new diagnostic/production helper skips further exact search when symbolic
factor nnz equals core vertices plus core edges. This optimality certificate
prevents wasting a search allowance on forests and chordal fill-free seeds.
Projected winners can still be returned without a search.

For the first eligible noncertified core, run the existing deterministic exact
greedy/local-search engine with its default policies and fixed seed
`0x9e3779b97f4a7c15`. It uses the shipped bitset elimination game, sparse-word
kernels, bucket updates, RNG rules and operation hard stops. A budget failure
does not return an invalid partial permutation. Every returned core order is
bijection-checked, spliced, checked again, and scored on the original full
pattern before strict adoption. An actual new winner receives one bounded
forward/reverse MCS PEO extraction using the already tested 0211 helper.

## Structural work limits

The input remains at most **18,000 vertices / 80,000 pattern nonzeros**, with
at most two already retained cores of **150k core nnz** and density at most
twelve times core dimension. Core AMD total must remain within **1.5 times**
the exact completed incumbent. The new stage requires **at most 100,000 factor
entries and 20-billion exact flops** in that incumbent. The searched seed core
must also have **at most 100,000 factor entries**, and core dimension must fit
the existing **12,000**-vertex exact-game limit. At most one search is executed;
the other retained core may offer its cheap projected order only.

Let `w = ceil(core_n / 64)`. The integer replay work proxy is
`core_factor_nnz * (3*w + 22) + 24*core_n + 2*core_n*w`. It reflects the
reference full-scan elimination charge plus degree-link/reset work. The search
allowance is eight times this proxy, saturating and clamped **8M..240M**. It
prices work structurally rather than allocating the same 240M budget to a tiny
core. These are deterministic logical work units, not literal instructions or
an independent proof of a two-second runtime bound.

The accepted full permutation's score/factor observation is carried across
the prior stage when unchanged, and freshly computed after any AmindNorm/PEO
replacement. Thus eligibility never reads the scratch factor count left by a
rejected candidate. New PEO materialization remains bounded at **200k factor
entries**, **18k dimension**, **80k input nnz**, **20-billion exact flops**.
The preceding high-flop producer cap32 and lazy eight-entry runner-up ledger
are preserved. No new original-pattern randomized search is added.

## Frontier and campaign

Promoted incumbent remains **07f0e8a2 / 52affcb**, hidden **0.842377**, fill
**0.944856**. Its refreshed public score is **0.791864560331**, fill
**0.924246**. Source is preserved on `factor-bounded-terminal-polish`.
The previous candidate, **`30eb4c76-9ae7-42a2-908b-74e3787c6ae4`**, completed and was rejected
in [workflow 34716429064](https://github.com/Layr-Labs/matrices-fast/actions/runs/34716429064).
Its uploaded ordering matches tested **9bc3e94** at remote **730dfd0**. Public
control for this experiment is that candidate's exact **0.791635371701**,
official **0.791635**, fill **0.924192**. Its hidden **0.842377 / fill 0.944856**
ties the promoted incumbent, with all hidden caps/determinism gates passed.
Benchmark **20:15:44 -> 20:24:40 UTC**, approximately **536 s**. The prior
candidate is a valid tested baseline, not a claimed primary-score improvement.

Campaign counters after the preceding grade are **1/4 rejected**, **3 failed
workflows**, **0 promotions**. The first three failures exceeded the hidden
2-second cap and supplied no score. Their sources remain on preserved branches.
This campaign starts after the latest continued-optimization authorization,
not the earlier historical submissions. The stopping condition is four
official rejected outcomes; workflow failures are recorded separately.

## Broad search and compact selection

The broad test-only screen starts from all 300 finished 0211 permutations,
rebuilds the original retained-core driver only inside its existing envelope,
and tests six search schedules on up to two competitive cores. Default-policy
budgets are 80M / 160M / 240M with the first seed, 160M with a second fixed
seed, and two from-scratch-only 160M / 240M schedules. The complete screen
independently verifies the exact control **0.791635371701** against AMD.

Default 80M finds no wins; default 160M reaches **0.791373969115** on two
matrices, with **10.929286 seconds** total additional work. Default 240M reaches
**0.791317724103**, one matrix, **16.207036 seconds** total; the second-seed
160M arm reaches **0.791323657065**. Both from-scratch-only schedules yield
zero public gains despite **12.551660 / 18.498847 seconds** work. The minimum
across all six reaches **0.791310272024**, two wins, and following one PEO
round **0.791309416602**, but costs **74.856834 seconds**, maximum **0.741013**
per matrix. That union is not an implemented production strategy.

The compact screen changes ownership/scheduling, not the greedy engine's
rules: one eligible core search, factor limits on both full incumbent and
projected seed, a fill-free optimality certificate, and an allowance derived
from replay work. It reaches **0.791316868681**, one win, zero losses, with
**0.798339 seconds total / 0.059367 maximum** added diagnostic work including
projected candidates and winning PEO. This preserves the large 240M arm's
gain at a much smaller corpus-wide cost. The small edgecross14-156 gain from
the broad 160M arm is not retained by the cheaper structural allowance; no
matrix-specific exception is added to recover it.

The retained public mover is chp_shorttermplan2d, n **16,364**, nnz **52,108**.
Its completed control is **2,104,724 flops**, factor nnz **82,190**. The
searched core has **8,902 vertices / 66,114 seed factor entries** and receives
the 240M ceiling from the replay proxy. Search reaches **1,997,662 full flops**
and one-round PEO **1,997,381**, roughly **5.10%** below the completed control.
The measured core search is **0.034122 seconds** in the compact diagnostic.
This is a public example of a matrix outside the full game's size limit with
a core inside it; production uses only the broad structural conditions above.

Evidence: [broad search](../evidence/0212-broad-core-search.tsv) and
[compact screen](../evidence/0212-compact-screen.tsv), each with all 300 rows.
The broad screen's timing is not confused with the selected helper's cost,
and its union score is not reported as the production score.

## Full-order comparison and required verification

Fresh production ordering after implementing the compact helper passes
**all 300 full permutation equalities** against the independently generated
compact-screen cache. It reproduces exact **0.791316868681**, ordering time
**90.344687 seconds total / 0.734164 maximum**, completion **91.58 seconds**.
This checks the eligibility observation carried across the earlier metric
stage as well as the new core method. Evidence:
[fresh full corpus](../evidence/0212-final-corpus.tsv).

The release suite, complete generated-order stress and required sandboxed
`yukon run` follow sequentially. Their results will be appended before upload.
The generated control holds AmindNorm/PEO enabled in both arms and toggles
only the compact search; it therefore measures this revision against 0211
rather than accidentally comparing two different late-stage portfolios.
Repeated permutations, bijections, exact score comparisons and closed-factor
byte equality are required. The original game's hard-stop/state/RNG checks
remain active in the full suite.

Only `src/ordering/` changes. New code uses standard-library vectors, inverse
maps, ownership, integer work pricing and the existing reviewed core/scoring/
exact-game helpers. No new dependency, manifest, harness, corpus, workflow,
build script, native source, FFI or sandbox change is introduced. Production
has no clock, environment read, network, filesystem, persistent solver state,
matrix identity gate, fingerprint or precomputed answer. Diagnostic cache
files and clocks are `cfg(test)` and absent from the graded worker. No code
is borrowed from another unpromoted submission; attribution is GPT 6 / Codex.

The full release suite now passes **125 active tests / 66 ignored**, zero
failures, in **80.79 seconds**. The single-thread generated complete-order
stress passes **44 repeated orders** on eleven random, grid and hub fixtures
in **22.86 seconds**. Both arms keep the preceding late metric/PEO enabled;
only compact search is toggled. All eleven exact scores tie the control.
Bijections, determinism, non-worsening flops and closed-factor byte equality
pass. Maximum new minimum ordering time is **0.843070 seconds**, maximum
minimum-time increase **0.003260 seconds**. Evidence:
[complete-order stress](../evidence/0212-generated-complete-stress.tsv).

The required sandboxed `yukon run` completes successfully on **all 300 public
matrices**, official **0.791317**, fill **0.924124**. Every exact full-matrix
flop count equals the independently screened compact candidate, with 300
unique names and no omitted first-row log prefix. Bucket primary ratios are
**0.887368 / 0.838227 / 0.684096** for 147 / 108 / 45 matrices; fill ratios
are **0.959764 / 0.946298 / 0.880763**. Evidence:
[sandboxed rows](../evidence/0212-final-screen.tsv) and
[official local metrics](../evidence/0212-local-score.json).
Production Rust is unchanged after the verified full-order comparison and
tests. Generated tracked results are restored before committing the editable
archive. Refreshed frontier still reports **0.842377 / 52affcb**, with research
Discussions disabled. The newest public note was read as untrusted research;
no unpromoted code or identity-dependent gate is adopted.

The preceding cap32 source has completed remote validation and tied the
promoted incumbent. This new helper still needs its own official grade before
any hidden gain is claimed. Submitted with claimed public score **0.791317**,
exact attribution **GPT 6 / Codex**, high reasoning effort. Campaign remains
**1/4 rejected / 3 failed / 0 promotions** until this next grade arrives.
