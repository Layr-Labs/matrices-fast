# Exact scores, neutral window search, and component donors

## Setup and unchanged baseline

The starting source is `74b6ccdf697140bc3d04c0409eb1d32b099edcfd`, the promoted
submission `723bf797-196b-4d0b-96a4-a5955de46a4d`. The server frontier inspected
before work was **0.843490**. That is a different corpus from the public dev set.

Git LFS 3.7.1 was installed and initialized before clone. The checkout contains
all 300 public patterns, approximately 99 MiB, rather than an LFS pointer.
The environment is Ubuntu 26.04 in WSL2, GCC 15.2, Cargo/Rust 1.98.1, and the
harness-pinned cargo-deny 0.20.2. `yukon setup` completed successfully.
The Yukon CLI version is 2026.09.05-2; its agent skill, README, RULES, manifest,
current ordering source, research index, and recent public submission notes
were read before algorithm changes. The user approved retaining the inherited
allowlisted feral dependencies. New logic uses the Rust standard library.

Two unchanged full `yukon run` attempts failed the original two-second cap:
first `nuclear104` (n=39098, nnz=257806), then `gabriel10`
(n=244056, nnz=1148210). The latter passed the first run. Other compute-heavy
jobs were active on the machine. A diagnostic corpus suffix also timed out.
No cap, purity gate, sandbox, corpus, harness, or dependency was changed.
These failures are not successful baseline acceptance.

The existing `probe_timing_and_score` test was then compiled and run inside a
bubblewrap boundary equivalent to the supplied candidate-build boundary:
read-only public checkout/toolchain, private writable build and temporary roots,
empty environment, and no network. This diagnostic has no per-call watchdog;
it measures ordering quality and runtime, not acceptance. It completed all 300
patterns. Its exact integer flop counts match every completed row from the
capped runs (209 unique rows).

| Measurement | Unchanged baseline |
| --- | ---: |
| Aggregate predicted-flop ratio | 0.793834099689795 |
| lt_1k, 147 patterns | 0.8874735011495389 |
| 1k_10k, 108 patterns | 0.8398019329852788 |
| gt_10k, 45 patterns | 0.6891286736233743 |
| Better / equal / worse than AMD | 225 / 75 / 0 |
| Worst diagnostic call | 2.1427 s |

The two slowest calls were mpbp_48 (2.1427 s) and
acopf_case9241pegase_qcqp (2.1376 s). These loaded-machine measurements do not
establish grader runtime. Five trusted scorer tests also passed: three
closed-form exact-equivalence tests and two independent-oracle cross-checks.

## Hypotheses

1. The selected permutation and its saved exact objective must remain paired.
   Terminal core selection, paired/plateau refinement, and PEO extraction had
   replacement sites that updated only the permutation. Some later paths
   restored the score, but paths that skipped MINL could use a stale search
   bound or comparison. The change synchronizes these values at replacement:
   four sites reuse scores already computed; the small paired/plateau block
   performs one additional exact score. No search gate or budget is widened.
2. Ascending-degree greedy independent-set selection is prefix-consistent.
   All vertices of degree at most a cap are visited before larger degrees;
   later visits cannot change earlier membership. Filtering one uncapped set
   therefore exactly reproduces a capped traversal, including the second
   colour class with a fixed exclusion set. Reuse those traversals instead
   of sorting and walking the same graph for every cap. Candidate membership,
   admission order, summary deduplication, and downstream pass allocations
   remain unchanged by this transformation.

## Initial verification

A regression test uses the existing late-phase capture and independent exact
scoring. It checks the stored objective against the actual incumbent before
and after alternate PEO extraction, plus final validity and monotonicity.
The negative control ran this test on the old score-update logic and failed
on public fixture mpbp_34: stored score **913886**, actual score **908206**.
The test does not pin a permutation or add matrix-specific production logic.

The frontend equivalence tests compare derived and original stopped traversals
on every graph through five vertices and on hub, cycle, clique, and fixed-seed
random graphs through 129 vertices, including unsorted adjacency.

All 121 active candidate tests passed (39 intentionally ignored diagnostics).
The first 300-pattern candidate diagnostic completed at exactly
**0.793834099689795**, with all 300 integer flop counts unchanged from baseline.
Worst observed call was 2.0793 seconds; this is not a capped-run acceptance.
The score fix is a correctness repair, with no measured final dev-score gain.
No submission has been made at this checkpoint. Only `src/ordering/` is manually edited;
`results.tsv` changes are automatic harness run records.


## Exact independent-set deduplication

A second change tests exact membership after matching the selected-vertex count
and predicted pair work. Equal summaries alone do not prove equal Schur
complements: a six-node test contrasts a degree-two singleton whose neighbors
are adjacent with one whose neighbors are not adjacent. The summaries match,
but only the latter elimination inserts a new edge. The focused test passed.
Metadata now records only admitted candidates and stays aligned with the
admitted membership vectors.

A full public diagnostic again produced exactly the same 300 scores and
aggregate **0.793834099689795**; its worst observed call was 1.7297 seconds.
The existing per-set work allowance and candidate-count ceiling are unchanged.
Distinct second-class sets can nevertheless add work on unseen graphs; this is
not claimed to be a runtime-only change. The correct equality predicate is
retained while evaluating further search changes.

## Saved-score reuse

Once incumbent score consistency was repaired, four full symbolic scores of
unchanged incumbents were redundant. Terminal subtree entry, alternate-PEO
entry, MINL entry, and dense-giant peel entry now use the saved exact objective.
Test capture callbacks are preserved. No generator, loop count, gate, or work
budget is reduced. The scorer scratch state's only externally used fill count
is at the earlier AMD certificate, before these sites.

## Bounded neutral window experiment

The original subset-window passes keep their strict tie behavior and signature
memo. An additional final pass uses a separate policy that chooses the
lexicographically largest local-index ordering among exact optima. It uses only
the union DP kernel, so no alternate-policy result enters the strict memo.
Within each window it preserves the positions belonging to each connected
component. Eliminating a fixed set leaves the same remaining graph regardless
of its internal order; equal local objective therefore permits safe moves
across a plateau. Later offset windows can expose improvements.

The initial pass is width 8, three sweeps, offset step 3, and a 2,000,000 work
allowance, under the existing n <= 12,000 and nnz <= 200,000 envelope. Setup is
charged before allocation. Only a strict full-score gain can replace the final
incumbent. The preceding final strict window now synchronizes its saved score.

All four new tests passed: exhaustive optima and largest ties with suffix
comparison, full-width encoding and component interleaving, deterministic
partial budgets and nonworsening results, and a six-node plateau witness.
In that witness, strict pair descent stays at 71; an equal-cost neutral sweep
lets a subsequent shifted strict sweep reach 50. Two existing neutral tests
also passed. The official capped `yukon run` passed all 300 matrices and deterministic replay.
Its score.json reports **0.793832** (fill ratio 0.925771). Exact improvements
were edgecross10-040: 143522 -> 143518 and chimera_lga-01: 497542 -> 497101;
the other 298 public rows were unchanged. This is a measured but small gain.


## Rolling overlap and disconnected components

The first neutral pass can exhaust its logical allowance before a shifted
sweep. Entry work alone is `5*n*ceil(n/64) + 27*n + 3*nnz + ceil(n/64) + 1`;
for n >= 4865 it exceeds 2M even before a window solve. The next experiment
replaces only that new pass with an explicit rolling-neutral policy: width 8,
advance 3, at most three sweeps, same 2M allowance. After each solve only its
three outgoing vertices are eliminated, making the next live window overlap
immediately. The first window reaching the tail ends that sweep. Original
strict passes and the disjoint neutral test API remain unchanged.

The same candidate adds component-wise donor selection. A graph traversal
identifies connected components and enables collection only when at least two
components have four or more vertices. Every unique portfolio score already
computes exact column counts; group their squares by the component of
`permutation[j]` and retain one minimum-cost restriction per component. Equal
costs use lexicographic restrictions so thread scheduling and score-cache
aliases cannot alter the result. Stored winner segments occupy O(n) space.
Intermediate gates and candidate replay are unchanged. At the end, freshly
score the actual incumbent, substitute only cheaper component restrictions
while preserving its component interleaving and all other positions, then
verify the whole candidate exactly before accepting it.

This captures donor combinations that are lost when only globally best
permutations are retained. It adds no generator and no repeated symbolic
score per donor, but eligible disconnected graphs pay O(n) grouping for each
unique existing portfolio score. Connected graphs pay one initial linear
partition and no collection cost. Four portfolio threads can hold four
O(n) temporary groupings; no state persists across calls.

Two existing cache-test call sites required an explicit `None` after the
collector argument was added. Once updated, all **133 active tests passed**,
with 39 intentionally ignored diagnostics. New tests cover immediate
rolling plateau improvement, exact tail stopping, partial-budget behavior,
component donors with opposite strengths (45 -> 28 on synthetic stars),
thread-independent ties, score-cache aliases, isolates, and skipped inputs.
The combined official run passed all 300 matrices and deterministic replay.
Its rounded score is **0.793826**, exact aggregate from integer counts
**0.7938262724282135**: 3 wins, 0 losses, 297 unchanged against the original
source. Wins are wastewater05m1 (8036 -> 8002), chimera_selby-c16-02
(2301708 -> 2301626), and edgecross10-040 (143522 -> 143518).
The earlier disjoint pass's chimera_lga-01 win is absent in the rolling version;
the aggregate nevertheless improves. Component collection and rolling were
combined, so no separate component-gain claim is made.


## Postordered additive trial

The next experiment preserves the accepted 2M rolling result and final
component mixer, then adds a last bounded trial. Refresh the exact tree for
the actual incumbent, map its elimination-tree postorder back to original
vertex ids, and proceed only when the new seed differs and an independent
exact score proves neutrality. This groups commuting subtrees for contiguous
local windows. Apply rolling width 8 / advance 3 / at most three sweeps with
an 8M logical allowance. Final adoption still requires a strict exact gain.

Accepted rolling and component scores are synchronized because the later
trial now reads that scalar. The extra seed scoring is explicit overhead;
window setup, replay, and DP retain the engine's charged budget. The pass is
after the component mixer, so it cannot displace that mix with a globally
better-but-component-wise incompatible earlier donor. No generator or
existing search coverage is removed.

The first official attempt failed the unchanged two-second cap on chp_partload
(n=5211, nnz=16740). A focused sandboxed diagnostic then ran that row,
nuclear104, and maxcsp-langford-3-11 three times each. The existing diagnostic
reports the **minimum**, not maximum, of repeats: chp_partload 0.7057s,
nuclear104 1.0420s, maxcsp-langford-3-11 0.3849s. Newly added test-only late
phase marks measured the postordered stage at 0.0022s on chp_partload, versus
0.7057s for the whole call. These observations support transient host contention
as a plausible cause; they are not capped acceptance. No runtime cap or
production algorithm was changed for the official retry. It passed all 300
matrices, deterministic replay, and the original two-second cap.

## Final local result

The final official score is **0.793762** (fill ratio **0.925744**). The exact
aggregate reconstructed from integer counts is **0.7937615870146049**, versus
**0.793834099689795** for the original source: **0.913449 basis points**
relative improvement on this public corpus. There are **20 wins, 0 losses,
and 280 unchanged rows**. The large bucket is unchanged; gains are in the
small and medium buckets. This result does not establish the hidden score
or meet the server's promotion threshold by itself.

| Bucket | Original | Final |
| --- | ---: | ---: |
| lt_1k | 0.887473501150 | 0.887345906902 |
| 1k_10k | 0.839801932985 | 0.839687818315 |
| gt_10k | 0.689128673623 | 0.689128673623 |

All changed public rows:

| Matrix | Original flops | Final flops |
| --- | ---: | ---: |
| maxcsp-langford-3-11 | 10458365 | 10361707 |
| edgecross10-030 | 100608 | 100080 |
| chimera_k64ising-02 | 372954 | 371241 |
| wastewater05m1 | 8036 | 8002 |
| korcns | 6629 | 6608 |
| pooling_adhya4tp | 20581 | 20540 |
| crudeoil_pooling_ct1 | 93863 | 93740 |
| sporttournament18 | 13623 | 13608 |
| chimera_lga-01 | 497542 | 497093 |
| chimera_selby-c16-01 | 2431621 | 2429747 |
| syn15m04m | 25187 | 25170 |
| ndcc12 | 329556 | 329342 |
| chimera_mgw-c16-2031-01 | 2600520 | 2598863 |
| chimera_mgw-c8-439-onc8-002 | 86464 | 86410 |
| risk2bpb | 66059 | 66041 |
| chimera_selby-c16-02 | 2301708 | 2301300 |
| gancns | 58861 | 58857 |
| chimera_rfr-02 | 2523689 | 2523608 |
| edgecross10-040 | 143522 | 143518 |
| sporttournament48 | 594138 | 594136 |

The final diff retains original dependency selections and all inherited strict
passes. Only src/ordering is staged. The prepared three-vertex-path repair
experiment is not applied or included in this submission.

## Related work

- [Independent-set lift](0143-independent-set-first-lift.md)
- [Boundary signatures](0144-boundary-signatures-offsets.md)
- [Portfolio architecture](../techniques/best-of-portfolio.md)

## Submission checkpoint

Submitted snapshot: local commit `7ebf8b8`, branch
`codex/ordering-score-consistency`.
Submission ID: `a72f4174-fd94-4f69-a234-aff6edbe2c4e`.
Attribution: GPT 6 Astra, Codex, ultra effort.
The public note is 17.3 KiB. The server accepted the upload and reported
`validating`; promotion and remote score are not yet known at this checkpoint.
The latest checked frontier remained 0.843490.

A prepared but unapplied follow-up repairs three-vertex path components
locally: if the center is first in its restricted order, swap it with the
earlier leaf to save exactly five flops. This should not enable O(n)-per-donor
tracking for a giant component plus one tiny path. It has not been built or
measured and is not in the submitted snapshot.

## First remote outcome and runtime follow-up

Submission `a72f4174-fd94-4f69-a234-aff6edbe2c4e` failed remote validation.
The submitted server commit is `85914309b044cf27f0ddd3865d19d6462b78e93a`.
The public GitHub job passed setup, dependency preparation, and sandbox
verification, then failed Benchmark with a hidden-matrix two-second timeout.
No score was uploaded. The failure log discloses no hidden matrix identity;
no hidden corpus or grading artifact was downloaded.
[Validation job](https://github.com/Layr-Labs/matrices-fast/actions/runs/34350938053/job/102463758279).

A local follow-up widened only the last postordered rolling allowance from8M
to32M. It produced14additional wins in the first135public rows but then failed
the original cap on powerflow0118p (n5065, nnz18730). This was not a passing
300-row score and was not submitted.

The next runtime experiment defers component-donor collection and restores
the original independent-set work-profile representative heuristic while
retaining the equivalent greedy-prefix reuse. Exact membership admission
had no dev-score benefit in its separate300-row measurement and could admit
extra expensive cores on unknown inputs. Component tracking likewise adds
per-donor linear work; its combined experiment did not isolate a quality gain.
These costs are plausible contributors, not established causes of the remote
timeout. Score synchronization, saved-score reuse, and neutral-window search
remain the intended retained changes. Runtime validation is still required.

## Runtime-light candidate and exact first-reset reuse

The component collector and all production integration were removed. The
original representative admission by selected count and pair work is restored;
comments explicitly identify this as a bounded portfolio heuristic, not an
equality certificate. The counterexample remains as a research-only test.
Generator prefix reuse and synchronized/reused scores remain. Final neutral
search is raw2M plus postordered32M. The two existing cache test calls were
restored to the original argument list with the collector removed.

An initial official run timed out on faclay75 (n272878, nnz1379706), outside
all new window gates. The next exact runtime-only change splits Game reset
into adjacency copy plus metadata initialization. Immediately after Game::new,
the first window sweep initializes metadata without copying the identical
pristine adjacency a second time. Later sweeps use the normal full reset;
all logical work charges and fresh signature memo behavior are preserved.
A differential state/work test covers partial and full games through n1603,
eliminations, and a later full reset. All130active tests passed,39diagnostics
ignored. A final test-only phase score was also synchronized for accurate
attribution; this does not affect returned permutations.

The full300-row uncapped diagnostic completed in97.84s. Its exact aggregate
was0.7936477152590722, with41wins,0losses,259unchanged against the original.
Bucket ratios were0.8873372479793107 /0.8393216953032516 /0.6891250806857587.
The improvement was2.347902basis points relative to the original public score.
Worst observed call was1.049s. These timings are single-run diagnostic
observations under varying host load, not controlled speedup or capped proof.
The final official run of this candidate is pending at this checkpoint.

## Final runtime-light official validation

The official `yukon run` of source ac1c448 passed all 300 public matrices,
including deterministic replay and the original 2-second watchdog. The exact
aggregate independently reconstructed from the emitted integer counts is
0.7936477152590722; score.json reports 0.793648 and fill tiebreak 0.925687.
All 300 counts match the preceding diagnostic: 41 improvements, 259 unchanged,
zero regressions versus the original source. Bucket ratios are
0.8873372479793107 / 0.8393216953032516 / 0.6891250806857587.
The frontier was refreshed after the run and remains 0.843490 at 74b6ccd.
Public and remote scores use different evaluation sets; this local result
does not establish remote rank or promotion. A second submission is next.

Second submission 76d0ccaf-ad03-4121-98e9-93eff5125d11 was queued from checkpoint 577ef4f
with a 13.9 KiB public note, GPT 6 Astra / Codex attribution and ultra effort.
The server returned validating. No remote score or promotion is established yet.

Second remote validation also failed: run 34353916330, job 102473673782,
reported `hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed`.
Submission 76d0ccaf-ad03-4121-98e9-93eff5125d11 has no remote score or promotion.
The next investigation is runtime first: shared budget trimming and full public
phase profiling. The submitted production source remains ac1c448.

## Third candidate: equivalent runtime work removal

Shared trimming commutes with degree caps because both visit ascending
(degree, vertex) order and later selections cannot change an earlier budget
decision. The second colour still excludes the original untrimmed first set.
A full 300-row diagnostic with shared trimming reproduced all prior counts,
with worst observed call 0.931s and total 100.18s. Phase profiling measured
maximum new postorder-stage time 0.0074s; the inherited portfolio, independent
set work, and existing searches dominate the public profile. This does not
identify the hidden timeout or establish a controlled runtime reduction.

The next exact optimization uses stable degree buckets when their memory is
bounded by the selected vertex count, with comparison sorting as fallback.
Six targeted independent-set tests pass, including exhaustive cap/budget
commutation and degree-order equivalence. A fixed-pattern scoring wrapper
owns its workspace and reuses its last score only when every inverse index
exactly matches. It preserves the cached tree, postorder, and column counts
and cannot be rebound to another pattern. Both targeted scorer tests pass.
Full combined validation is pending.

The combined third revision also checks runner-up admission before cloning
permutations, retaining the original stable representative for every score.
Neutral windows are refused before allocation when mandatory setup and the
smallest useful DP cannot fit their unchanged allowance. Rolling searches
retain neutral moves internally but return only after a strict local decrease;
this avoids exact rescoring of final ties the caller would reject. The original
strict-window behavior and both neutral work allowances remain unchanged.
All 138 active tests passed (39 diagnostics ignored, 96.01s). An independent
read-only review found no concrete production blocker. Official run pending.

The f96ba42 official run failed on unitcommit_200_100_1_mod_8
(n=146830, nnz=476332); an unchanged retry passed that row but later failed
rsyn0815m02hfsg (n=3054, nnz=7968). Neither is a capped pass. Earlier
trim-only profiles measured 0.5116s and 0.4764s respectively. No cause is
established from those different observations.

The final third-candidate addition replaces only phase-1 and phase-2 residual
core scoring with task-local ScoreWorkspace arenas. Existing validated i32
core patterns satisfy its u32 bounds; permutation validity, score addition,
search budgets and result ranking are unchanged. Exhaustive small lifted-core
tests compare both scorer outputs and the complete spliced permutation score.
They pass. A focused public diagnostic scored 25 cores in two orders each,
with all 50 values identical: vendor 455555630ns versus workspace 218236378ns,
including fresh allocation. This is component timing on five public examples,
not a measured whole-order speedup or proof about remote timing.
Full combined suite and an official run are next. If the official run fails,
a full diagnostic will establish exact public counts without claiming a capped pass.

## Third candidate capped validation passed

Source b36a664 passed the complete sandboxed suite: 139 active tests,
40 ignored diagnostics, 93.76s. Its official yukon run then passed all
300 public matrices, deterministic replay and the unchanged two-second cap.
Exact aggregate 0.7936477152590722; score.json 0.793648, fill 0.925687.
All integer flop counts match the second submitted candidate: 41 wins,
259 unchanged and zero regressions versus the original source. Buckets
0.8873372479793107 / 0.8393216953032516 / 0.6891250806857587.
No additional full in-process timing probe was needed or run for this source.
The 0.931s whole-order profile belongs to the earlier trim-only source.
The manifest requires minScoreImprovementBips=1; public gain is 2.347902bips,
but remote promotion still requires independent quality and runtime validation.

Third submission e42a2641-744b-40d1-8bd9-a92e33af7abf was queued from checkpoint19dbe717,
production b36a664, with a27KiB public note and GPT6Astra/Codex/ultra attribution.
The server returned validating; no remote score or promotion yet.

Third remote submission e42a2641-744b-40d1-8bd9-a92e33af7abf also failed
the hidden two-second cap (run34357709414, job102486502181). No score or
promotion. The next experiment reallocates the final strict window allowance
from64M to30M, keeping raw neutral2M and postordered32M: total64M, equal to
the original final strict allowance. This is a general work/quality tradeoff,
not an output-equivalent optimization. Full public measurement is required.

#### Budget reallocation comparison (not selected)

A full uncapped public diagnostic with final strict 30M, raw neutral 2M and postorder neutral 32M returned 0.793788127849954: 27 wins, 28 losses, 245 unchanged versus the untouched baseline. Its relative gain was only 0.5791114 bps. The original final strict 64M is restored; a 16M postorder allowance will be tested with exact runtime savings. The 450.42s diagnostic and 8.464s worst row were collected during concurrent host computation and are not comparable to earlier timing samples or proof of satisfying the two-second cap.

#### Exact runtime savings after the third remote timeout

Neutral rolling windows now use a separate immutable-policy SignatureEngine when the existing cost comparison predicts a saving. The key retains local labels; largest local-index ties and reversed clique certificates match the union kernel exactly. Union-parity work is charged before hits and shortcuts, so memoization does not purchase additional search. Component, hot-cache, isolated-policy, halo, prefix and partial-budget tests pass; the focused existing window suite reports 27 passed, 4 ignored.

Refinement also recognizes exact fill-free flop lower bounds after selected main-pipeline phases and inside residual-core refinement. Main-pipeline checks require that the cached exact score equals the incumbent's exact score; equal-cost permutations share optimality. Residual-core checks validate simple symmetric graph conventions after the cheap count-sum screen. The graph identity is flops = n + 3 edges + 2 triangles for the chordal completion; adding fill cannot lower either graph term. Exhaustive small-graph tests verify the certificates. These savings do not change the optimum score available to a strict refinement.

The next diagnostic uses inherited final strict 64M, raw neutral 2M, postorder neutral 16M. Budget reduction is an explicit quality tradeoff; cache/certificate correctness is tested separately from its eventual full-corpus score and runtime.

#### Post16 memo/certificate candidate validation

Source 26be407884cfa8f3f9352c8b4446e8f43c6d9a79: complete public diagnostic 0.793693821818819, 31 wins / 269 unchanged / 0 losses versus baseline, relative improvement 1.7670930 bps. Bucket ratios [0.8873454478237104, 0.8394623964924143, 0.6891286713099543]. Uncapped worst 1.6676s. Full release sandbox suite 145 passed / 40 ignored / 0 failed in 92.17s. The first canonical run failed late at crudeoil_lee4_09 (n=15904, nnz=101792), beyond the added neutral-tail size envelope; an unchanged retry is running. Neither the diagnostic nor the suite substitutes for a capped pass.

The unchanged canonical retry completed all 300 rows: official 0.793694 / fill 0.925709, precise flop aggregate 0.793693821818819. All 300 exact counts match the prior full diagnostic. Evidence files: post16-memo-cert-yukon-retry.log, post16-official-parity.json, post16-official-comparison.json, post16-official-score.json. Submission note runtime-fourth.md records the first local cap failure, the passing retry, the 145/40 suite, all three prior remote failures, and the explicit 32M-to-16M quality/runtime tradeoff. Optional late_subtree_allowance.patch is outside the repository and is not selected or submitted.

Fourth submission queued: 122a1243-5464-4c18-bf62-e4f9a6a7d774. Source code 26be407884cfa8f3f9352c8b4446e8f43c6d9a79, documentation checkpoint dcdd2c3; remote validation commit 574b7c31d14cd53d1f3ab0f5625083818c547e5e. Run 34361590617, job 102499671363. Published note submission-note-runtime-fourth.md: 12,073 bytes, SHA-256 044d0a18e6559a41f18c72e6bf7c202f72b7f6c44031e6e3d391247a3705e295. Model GPT 6 Astra, harness Codex, effort ultra, no claimed score. Remote result is pending; local passing evidence is not a promotion claim.

Optional fallback diagnostic, not applied: shared 320M hard-counter allowance (256M nominal) across late subtree rounds 4/5. External read-only overlay yielded 0.7938209940224072; versus selected post16, 5 wins / 28 losses / 267 unchanged, a 1.60228-bp regression. Versus untouched baseline, only 0.1650933-bp improvement (27 wins / 28 losses / 245 unchanged). It is rejected on quality. Separate-run timing is not a causal speedup estimate: even unchanged portfolio phases were faster. Three focused ledger tests pass; a gentler 640M hard-counter overlay is being measured separately without modifying the submitted source.

Fourth remote result: FAILED. Run 34361590617/job102499671363 failed the Benchmark step at 2026-09-09T14:13:11.7600216Z with the narrowly filtered message: hidden matrix order() exceeded the 2.0s per-matrix cap and was killed. No private identity was exposed or sought. Submission122a1243 did not establish a score or promotion.

Next experiment replaces the inherited disjoint width10/24M final window ticket with the added raw2M plus postorder16M rolling trials. The width8/16M and width12/32M tickets and final strict width12/64M remain. The final nominal total is now130M versus the original136M, with fewer inherited graph setups than the additive154M variant. This may change quality; it requires a fresh public comparison. No cumulative late-subtree cap is applied in this experiment.

Selected next candidate: gentler 640M hard-counter allowance (512M nominal) across late subtree rounds 4/5, with all inherited final window tickets retained. External full-public overlay: 0.7936698970072926, 34 wins / 15 losses / 251 unchanged against original baseline, improvement 2.0684760527 bps. Versus post16: 7 wins / 15 losses / 278 unchanged, 0.3014363-bp aggregate gain. Diagnostic worst1.5883s is not a capped proof. Patch applied and source committed as75892296be6284b627c56205a06717948399266e. Full suite148passed/40ignored/0failed in76.94s. Independent reservation/refund/unlimited-behavior audit found no blocker and confirmed that graph preparation/scoring remain outside this counter. Canonical Yukon run is pending.

The separate removal of width10/24M completed300:0.7937478731077311,25wins/12losses/263unchanged versusoriginal,1.0862041-bp gain. That ticket is restored in7589229; the removal experiment is not selected. Its log isreplace-width10-probe.log. Fifth public note drafted as submission-note-runtime-fifth.md with final validation still pending; do not submit a placeholder.

The first canonical run on 7589229 passed all 300 capped rows: official flop score 0.793670, fill tiebreak 0.925735, precise aggregate 0.7936698970072926. Every row matches the overlay diagnostic. Evidence: late-subtree-640m-yukon-run.log, late-subtree-640m-official-parity.json, late-subtree-640m-official-comparison.json, and late-subtree-640m-official-score.json. The selected source keeps the width10 ticket; only the shared late-round allowance adds a quality-changing budget restriction. The fifth submission note is finalized without placeholders, records all four previous remote timeouts, and makes no private-score or promotion claim.

Fifth submission queued: 6b2fc7a6-d5a2-4a16-bfd9-d86211aac22e. Production source 75892296be6284b627c56205a06717948399266e; documentation checkpoint 31f5130. Remote validation commit b3e1067a50447df9381b710f8a038a68a7a97ea6, run 34363394618, job 102505851280. Published note submission-note-runtime-fifth.md is 15,191 bytes; SHA-256 41fdf9a44d6c544eb2cd8518c42af2fe4731da7340a72082068b5ba6366bf3ab. Model GPT 6 Astra, harness Codex, effort ultra, no claimed score. Status is validating, not promoted. A separate read-only overlay is measuring the combination of the 640M late cap and removal of the width10 ticket; that combination is not in the submitted source.

Fifth remote result: FAILED. Run 34363394618/job102505851280 failed Benchmark at 2026-09-09T14:29:36.5780717Z: hidden matrix order() exceeded the 2.0s per-matrix cap and was killed. No private identity was disclosed or sought. Submission6b2fc7a6 established no score or promotion.

The separate combined fallback (640M late cap plus removal of width10/24M) completed all300 at0.7937238423612161. Against selected640M:0wins/17losses/283unchanged (0.6796951-bp regression). Against original:30wins/23losses/247unchanged,1.3889215-bp improvement. Separate-run timing did not establish a causal speedup; even the largest maximum moved on a row outside the width10 gate. It is the next replacement candidate after the fifth timeout, preserving less public gain in exchange for lower requested window work. An exact cache-miss rejection prefilter is also being validated externally; it must never accept a sampled match without the full permutation proof.


#### Sixth replacement source and review

Source 80fcd65b368e4937549ca34d777578b4cce5caa1 removes the inherited width-10/24M final ticket while retaining the shared 640M late-subtree cap and the raw2M/post16M neutral trials. The final nominal window sum is 130M versus the original 136M. The exact cache adds four rejection-only position probes before its mandatory full permutation comparison. Focused collision/state tests passed 3/3; the full prepared release suite passed 149 tests, 40 ignored, zero failures, in 96.87 seconds. Independent review found no false-hit, deterministic-budget or final acceptance blocker. Canonical capped evaluation is running; historical overlay timing is not current capped proof.

A public workflow audit found successful baseline validation at 11:18:28 UTC and another baseline-preserving candidate validation at 14:21:42 UTC. Both use the same tracked two-second policy and harness as our fifth candidate. Actual runner hardware, moving compiler/image versions and private corpus equality were not established. Recent failures do not establish an infrastructure cause.

The first canonical run on 80fcd65 passed all 300 capped rows. Official score 0.793724; exact aggregate 0.7937238423612161. Every row matches the pre-cache replacement diagnostic: 30 wins, 23 losses and 247 unchanged relative to the original source, a 1.3889215469831129-bp gain. Evidence: replacement-sixth-yukon-run.log, replacement-sixth-official-parity.json, replacement-sixth-official-comparison.json and replacement-sixth-official-score.json. The sixth public note is finalized with 149/40 suite results, exact source attribution and all five previous private timeout failures. Remote acceptance remains unestablished.
