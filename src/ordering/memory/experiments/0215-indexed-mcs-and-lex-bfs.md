# 0215 — Indexed MCS queue and one fixed LexBFS completion extraction

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

## Context, objective and base

The latest appended terminal MCS/PEO source improves the public corpus but
still fails the hidden 2.0-second wall-clock cap. Merely adding an original-AMD
work guard does not resolve it. This revision makes MCS updates cheaper while
preserving its exact output, then adds one different, bounded chordal search
with a measured small incremental cost. It does not increase the 1M factor cap
or weaken the existing work gates. The primary objective remains the weighted
mean of size-bucket geomean LDL factor flop ratios, versus AMD at 1.00; fill is
the secondary tie-break. Any invalid or over-cap matrix fails the whole grade.

Original promoted source **07f0e8a2 / 52affcb**, hidden **0.842377**, public
exact **0.791864560331**, remains the current promoted frontier. The best valid
raw scored source from this campaign is **de17cd31 / bf72aac**, hidden
**0.842374**, fill **0.944855**, rejected because its gain is below the one
relative-basis-point promotion floor. That source remains preserved separately.

Immediate public control is submitted **d895008f-037d-4dc8-a7e6-28400a392d03**,
tested/uploaded **7946bcf8d3875f8fb0c520c48095c55d461027f2**, remote
**6a1cdea71bce22ea506c47dd341ec2ec869b0b02**. Whole ordering trees match.
It scores public exact **0.790704642081**, official **0.790705**, fill **0.923840**,
but [workflow 34720094107](https://github.com/Layr-Labs/matrices-fast/actions/runs/34720094107)
fails with explicit hidden `order() exceeded the 2.0s per-matrix cap`:
Benchmark **21:32:45.37 -> 21:34:28.18 UTC**, **102.81 seconds**. No hidden
score or offending matrix identity is exposed; none is inferred or used.
[0214](0214-amd-work-guard-and-larger-factors.md) records the failed guard
hypothesis and immutable source. Current campaign **2/4 scored rejections,
5 failed workflows, 0 promotions**; failures are not counted as rejected grades.

## Exact-output runtime repair

The old degree-priority MCS puts a new BinaryHeap tuple into the heap on every
neighbor weight increase. Old tuples remain until discarded as stale or visited.
The completed-visit stop already avoids draining the remainder after the last
visit. This revision instead stores one live heap item per unvisited vertex,
with a vertex-to-position inverse map. Weight increases sift that item upward;
removing the maximum marks its inverse position absent and sifts the last item
down. Neighbor traversal and the MCS selection rule stay the same.

The static `(priority, vertex)` tuple is sorted once into a unique rank.
A 64-bit key puts weight in the high 32 bits and static rank in the low bits.
Therefore key comparison preserves the exact old tuple comparison, including
the vertex tie-break. Dimension is bounded at 50k; weights cannot exceed the
number of completion vertices, so neither rank nor weight exhausts 32 bits.
Only standard-library vectors and integer operations are used. The live heap,
inverse positions and keys need **24 bytes per vertex**, approximately 1.2MB
at 50k, rather than allocation proportional to completion weight updates.
Completion storage remains separately bounded. The update work is logarithmic
in live vertices, and stale insertions/removals disappear.

All twelve policies match the frozen pre-stop reference exhaustively for all
graphs and incumbent orders through five vertices. The new queue also matches
the completed-stop old kernel on **292 eligible public completions** under
50k/1.3M/2M diagnostic limits, twice per arm in alternating execution order.
That diagnostic limit includes cases wider than the selected 1M production
stage; it does not activate a 2M graded path. Local kernel totals **1.211223 ->
0.125913 seconds** (about **9.6x**), maximum per-fixture minimum **0.146112 ->
0.009247 seconds**. These are kernel observations on ARM, not Linux/x86
predictions or a guarantee that all preceding portfolio work clears the cap.
[All 292 comparisons](../evidence/0215-indexed-kernel-comparison.tsv).

## LexBFS choice and measured scope

The alternative search is motivated by the primary LexBFS/partition-refinement
discussion in Beisegel et al., ESA 2020. [Literature and provenance](../literature/0215-lex-bfs-partition-refinement.md)
link the publisher paper and the related original Habib et al. bibliography.
The implementation is written independently from the algorithmic idea, not
copied from fetched code. An independent explicit-label oracle validates it.

Our implementation keeps ordered label classes in linked arrays. Per-vertex
previous/next links allow constant-work removal and insertion. A pivot stamp
creates at most one split class for a given original label class; empty IDs
are recycled. Selected neighbors move to the front of their new class in a
fixed traversal order. The graded choice starts with a stable descending
original-degree order, retaining incumbent position for equal degrees, and
visits completion neighbors in reverse of their existing flat-CSR order.
This is one globally fixed traversal policy, with no matrix-conditioned oracle.

All graphs/orders through n=5, three fixed initial orders and both neighbor
directions match the independent label oracle, are PEOs of the independently
completed graph, and never worsen the independently scored original pattern.
This check plus all twelve indexed MCS policies completes in **3.45 seconds**.
The public policy screen starts from all 300 finished 1M-control permutations.
It holds **n<=50k, input nnz<=1.3M, factor nnz<=1M, original AMD<=1B** fixed.
Each optional ordinary-PEO continuation runs only after a strict LexBFS win.

| Initial order / neighbor direction | Raw exact score | With conditional PEO2 | Improved rows |
| --- | ---: | ---: | ---: |
| incumbent / forward | 0.790704642081 | 0.790704642081 | 0 |
| incumbent / reverse | 0.790669567956 | 0.790531769650 | 5 |
| reversed incumbent / forward | 0.790704642081 | 0.790704642081 | 0 |
| reversed incumbent / reverse | 0.790672410335 | 0.790543658623 | 5 |
| original degree / forward | 0.790704642081 | 0.790704642081 | 0 |
| **original degree / reverse** | **0.790594408187** | **0.790519213141** | **6** |

The three fixed adjacent pairs add no quality beyond their reverse member on
this screen. Forward traversal has no public gains in the measured scope,
so the submitted candidate performs one reverse traversal. The whole six-mode
screen totals **1.081575 s**, maximum **0.153614 s**, not the cost of the selected
one-mode candidate. [All 300 screen rows](../evidence/0215-lex-six-mode-screen.tsv).

The selected independent LexBFS5 / conditional PEO2 replay matches the production
helper on all **300 exact permutations**. Exact score **0.790519213141**, six
strict full-flop improvements and no regressions against the immediate public
control. Selected added work **0.396205 s total / 0.047412 max**, test completion
**1.43 s**. [All selected rows](../evidence/0215-selected-lex-screen.tsv).

| Matrix, diagnostic identity only | Control full flops | Fixed LexBFS raw | Final with conditional PEO |
| --- | ---: | ---: | ---: |
| pooling_sppc1pq | 131708592 | 131443992 | 131443992 |
| crudeoil_lee4_10 | 180591368 | 180405890 | 179403641 |
| procurement1large | 7372894 | 7372585 | 7372274 |
| crudeoil_lee4_06 | 32720114 | 32710270 | 32701225 |
| crudeoil_lee4_09 | 129435607 | 129266108 | 129101152 |
| arki0013 | 153121933 | 151071841 | 150283404 |

Fresh complete production, including all preceding portfolio stages, matches
every independently cached public permutation, exact **0.790519213141**,
total ordering time **91.680573 s**, maximum **0.807537 s**, completion **92.88 s**.
The maximum is crudeoil_lee4_10 in both this and the preceding check, previously
**0.910646 s**. The unpaired full-run timing difference is an observation;
the alternating kernel comparison is the stronger isolated speed evidence.
[All fresh full-function rows](../evidence/0215-final-corpus.tsv).

## Graded implementation and constraints

Changed `peo_extract/indexed_priority.rs` and the priority dispatcher implement
the eager queue; the optimized old kernel and frozen reference are test-only.
Changed `peo_extract/lex_bfs.rs` and its bounded dispatcher implement the fixed
LexBFS traversal. `post_core::refine_lex` admits one complete traversal only for
**n=6..50k, input nnz<=1.3M, exact factor<=1M, exact full flops<=20B**. The root
caller shares the existing **original AMD<=1B** work gate. Only a strictly lower
actual original-pattern flop score replaces the finished incumbent; ordinary
PEO gets at most two rounds after that win, inside the same caps. Rejected
candidates and closed gates retain the earlier permutation. No 2M path is active.

The existing Norm/core-search stages, work allowances, producer cap32 above
20B, lazy stable runner-up ledger, sorted scatter permutation, fixed seeds,
x86 popcnt guards and original ordinary-PEO wrappers remain unchanged. There
is no repeat independent-set driver, original-graph copying for every metric,
or extra METIS work. Test-only probes and memory evidence live under ordering.
No manifest, harness, scorer, purity gate, corpus, dependency, native source,
workflow or sandbox changes. Production is standard-library Rust using the
existing trusted graph/scoring helpers; no clock, environment, filesystem,
network, persistent cache, stored answer or corpus identity enters the order.

Local setup is ARM macOS with installed Yukon CLI, gcc, cargo/Rust 1.98.1 and
pinned cargo-deny 0.20.2. Git LFS preceded cloning; setup and the original
baseline were completed earlier. Schema 1 is one standalone benchmark. No
track switch, coding-agent restart or hook setup is used. Linux/x86 timing is
not inferred from local ARM. Two diagnostic-only build issues were corrected:
the new screen needed `crate::corpus` qualification, and Cargo test cwd required
a compile-time manifest-root path to the public evidence table. Neither issue
changed the graded algorithm; all subsequent selected/fresh checks pass.

The newest public 1b764d1 window-extension note was read after this queue and
LexBFS implementation and policy selection, as untrusted data. Its claimed
work/score measurements are not used here; no unpromoted implementation is
borrowed and no coauthor is required. Public notes omit credentials and private
workspace paths. Release, generated and required sandbox checks are recorded
below when complete; no eighth official grade is claimed yet.

## Reproduction

Run from the benchmark root. Diagnostics use explicit opt-out and test-only
controls, while the required scored build and worker use the unchanged
sandbox path. CPU diagnostics, complete release suite, generated calls and
the scored run execute sequentially. The public compact and 1M-control
permutation caches are reproduced on their recorded sources; their existing
experiment pages describe those producers. The selected LexBFS check writes
the cache consumed by the fresh complete-function check. These public caches
never compile into the graded worker.

```sh
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::peo_extract::tests::peo_extraction_exhaustive_graphs_and_orders \
  -- --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::peo_extract::indexed_priority::public_kernel_comparison \
  -- --ignored --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::probe::campaign::probe_lex_completion \
  -- --ignored --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::probe::campaign::probe_selected_lex_completion \
  -- --ignored --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
SSI_COMPARE_EXPECTED_SCORE=0.790519213141 \
SSI_CAMPAIGN_SEED_CACHE=/tmp/matrices-fast-lex-campaign-seeds.tsv \
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

Fresh release suite passes **126 active tests, 71 ignored**, zero failures,
completion **82.77 seconds**. The previously frozen MCS reference, new eager
queue, explicit-label LexBFS oracle and independent full symbolic scores all
agree on their checked domains. Sorted permutation, bounded custom-metric
completion/exhaustion, RNG and work-state equality, and lazy ledger references
also pass. No additional production edits occur after the fresh corpus check.
Generated complete old/new comparisons and required sandboxed score follow.

The complete generated comparison passes **64 calls on 16 fixtures**, both
arms deterministic and bijective, **0 gains / 16 full-flop ties**, completion
**28.75 seconds**. Control is the submitted 1M original-AMD-guarded source with
the completed-stop stale MCS heap and no LexBFS; candidate enables the indexed
queue and fixed LexBFS. Prior Norm/search stages and all geometry/cost gates
are shared. Maximum per-fixture minimum **0.816613 -> 0.815946 seconds**,
maximum observed increase **0.001041 seconds**. The 192x192 grid is below the
1M factor gate (**756,999**) and exercises both complete bodies: **59,682,899**
flops unchanged, **0.490455 -> 0.433045 seconds**. Forest paths/hubs preserve
their early exact certificate. Closed high-factor random cases preserve the
full predecessor output. [All generated rows](../evidence/0215-generated-complete-stress.tsv).

Required sandboxed `yukon run` completes on all **300** public matrices,
official **0.790519**, fill **0.923754**. Every n/input-nnz/AMD/candidate flop
tuple equals the independent selected replay. Bucket flop ratios
**0.887368 / 0.838152 / 0.682158**, fill ratios **0.959764 / 0.946262 / 0.879866**,
counts **147 / 108 / 45**. [Official all-300 table](../evidence/0215-final-screen.tsv)
and [machine-readable local score](../evidence/0215-local-score.json) are archived.
The generated results.tsv append is removed before committing the exact tested
ordering source. All release, generated, independent and required sandbox
checks pass; no production change follows them. Fresh metadata still reports
**0.842377** promoted from **07f0e8a2 / 52affcb**, Discussions disabled and
claimed score recorded only. The newer 1b764d1 window-extension workflow has
failed without a score; its claims contribute no new implementation here.
Final sandboxed table, machine-readable fill metrics and immutable submitted
tree verification will be recorded when available. Hidden outcome remains
unknown and the local kernel speedup alone does not establish a promotion.
