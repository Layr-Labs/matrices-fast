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
