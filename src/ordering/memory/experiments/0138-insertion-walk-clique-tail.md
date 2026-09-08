# Bounded Insertion Walk and Exact Clique-Tail Scoring

## Context and Scope

This experiment starts from promoted source `fbef927`, submission
`767130fa-c3ae-43d8-92d4-70db8ed3b9d6`. The repository was clean before setup.
The task was to improve the existing ordering, test it against the starting
checkout, and submit a measured improvement. The existing portfolio, its
dependencies, and the trusted harness were retained. Algorithm and test edits
are confined to `src/ordering/`. No dependency was added or upgraded.

The underlying session model is `openai/gpt-6-astra`, running in OpenCode.
No separately configurable effort setting was recorded. Public submission
metadata uses these actual identities with the user's approval.

The benchmark is schema version 1, identifier
`8c3e7051-530a-4aee-88df-a426e6e78151`, named
`layr-labs/matrices-fast`. Research Discussions were disabled when inspected.
The current remote best was 0.848883 at both initial inspection and the
pre-submission check. That number is a hidden-corpus result and must not be
compared directly with the development numbers below.

## Baseline and Environment

Setup used `yukon setup`, including the pinned cargo-deny 0.20.2 check,
manifest generation, vendoring, native-code scan, and trusted release build.
The candidate was built using the repository's sandboxed build script through
`yukon run`. The host is macOS and the candidate build used Seatbelt with
network access denied. Subsequent candidate test builds and executions also
ran under a Seatbelt profile restricting writes to build/cache/temp roots.

The unmodified checkout completed the full 300-pattern public development
corpus with score 0.804873 and fill tiebreak 0.929679. This was a new measured
baseline, not an assumed number from historical notes. The development AMD
anchor remains 1.0. The unmodified timing probe reported a worst call of
1.0292 seconds on `qapw`. Other slow calls included 0.8563 seconds on
`maxcsp-langford-3-11` and 0.6178 seconds on `crudeoil_lee4_09`.

## Implementation Rationale

The existing final small-graph polish samples paired swaps and single swaps.
A vertex insertion explores a different neighborhood: remove a pivot from one
position and insert it at another while preserving all intervening pivots'
relative order. A swap also displaces the destination pivot to the origin;
that extra displacement can make the swap unhelpful even when insertion helps.
The experiment therefore reuses the exact SmallScore evaluator rather than
adding another ordering dependency or rewriting the large-matrix pipeline.

`insertion_refine` is called on the result of `leader_order` by the public
`order` entrypoint, after the existing forest certificate. It returns the
original order immediately unless the dimension is between 12 and 1,000
inclusive and the supplied pattern has at most 12,000 nonzeros. These are
structural cost gates, not matrix identities. All larger or denser graphs keep
the original candidate selection; only the exact scorer shortcut can affect
their runtime when used by existing small-graph routines.

The search uses a fixed xorshift stream initialized to
`0xd1b54a32d192ed03`. There are exactly 4,096 draw attempts. Alternating attempts
choose either an unrestricted destination or a forward offset from 1 to 32,
wrapped modulo the dimension. Equal endpoints are skipped. Rust slice
`rotate_left(1)` and `rotate_right(1)` implement the insertion without deleting
or duplicating a vertex. There is no external randomness or wall-clock input.

Every proposed order is scored with the existing `flops_bounded` evaluator.
Strict improvements update both the best permutation and the current walk.
Equal-cost moves may advance the current walk, but never overwrite the saved
strict-best output. Worsening moves are reversed with the inverse rotation.
Consequently, the returned permutation has no greater predicted flop cost
than the input permutation, including when the walk ends on a neutral state.
The new helper is intentionally small and uses only already-present types
and standard-library vector/slice operations.

## Exact Scorer Shortcut

After eliminating a pivot of live degree d, its neighbors form a clique.
The existing bounded scorer already uses the sum of squares from 1 through d
as a suffix lower bound, plus one for every other remaining vertex. If d
equals the entire remaining vertex count, there are no other vertices: the
bound is the exact cost of the suffix, regardless of its elimination order.
The new two-line fast path returns this exact value immediately after the
existing rejection check. It avoids materializing and eliminating a clique
whose cost is already known. This preserves comparison outcomes and all
accepted scores, rather than trading scoring accuracy for speed.

Primary-source API research confirmed that the existing feral-amd 0.2.1
dependency already supplies quotient-graph AMD and its configurable variants:
https://docs.rs/feral-amd/0.2.1/feral_amd/
The current portfolio already uses that family extensively. This experiment
therefore stays within the repository's existing exact-scoring/local-search
approach. No fetched implementation was copied into the source tree.

## Experiments

The test-only `probe_insertions` starts from `leader_order` so it can compare
the inherited pipeline with the additional pass even after production wiring.
It checks bijection, repeats the search to check exact determinism, and uses
the independent feral symbolic scoring path to check non-increasing flops.

| Candidate | Strictly improved matrices | Regressions | Worst isolated added time |
| --- | ---: | ---: | ---: |
| 1,024 insertion attempts, original bounded scorer | 9 | 0 | 0.032662 s |
| 4,096 insertion attempts, exact clique-tail shortcut | 14 | 0 | 0.125770 s |

The larger budget was retained because it found additional improvements and
the integrated timing run remained below the starting worst-call time. This
is not a clean timing ablation of just the budget: the second probe also
includes the exact clique-tail shortcut. No inference about its isolated
speedup should be made from that table.

The 14 strict development improvements at the selected budget were:

| Public development pattern | Starting flops | Candidate flops |
| --- | ---: | ---: |
| pooling_adhya4tp | 20581 | 20540 |
| chimera_mgw-c8-439-onc8-002 | 87247 | 86918 |
| wastewater05m1 | 8042 | 8039 |
| graphpart_3g-0244-0244 | 32506 | 32474 |
| waterund14 | 72569 | 71330 |
| gancns | 58863 | 58844 |
| multiplants_stg1 | 155624 | 155496 |
| sonet23v4 | 94348 | 94278 |
| waterund11 | 10759 | 10705 |
| korcns | 6629 | 6608 |
| rocket50 | 19265 | 19224 |
| multiplants_stg5 | 158934 | 158574 |
| rsyn0815m | 7745 | 7727 |
| pooling_digabel19 | 277303 | 275668 |

These names are measurement labels only. Neither production code nor its
gating logic reads names, identifies corpus members, or stores permutations.

## Full-Corpus Results

| Metric | Starting checkout | Candidate |
| --- | ---: | ---: |
| Weighted development flop score | 0.804873 | 0.804788 |
| Weighted development fill tiebreak | 0.929679 | 0.929659 |
| lt_1k flop geomean, 147 patterns | 0.887829 | 0.887544 |
| 1k_10k flop geomean, 108 patterns | 0.842581 | 0.842581 |
| gt_10k flop geomean, 45 patterns | 0.714375 | 0.714375 |
| Worst measured complete order call | 1.0292 s | 0.8952 s |

The rounded total score delta is -0.000085, approximately a 0.0106 percent
relative improvement over this checkout. All 300 development matrices pass
the full benchmark, including the repeated-permutation determinism gate and
the hard two-second process deadline. Fourteen improve and 286 retain the
same flop counts. Both larger size buckets retain their exact reported scores.

The candidate timing probe's slowest calls were `qapw` at 0.8952 seconds,
`maxcsp-langford-3-11` at 0.8153, `ndcc13` at 0.6362, and
`multiplants_stg1b` at 0.6313. Small sparse matrices do pay additional search
time even where the search fails to improve. The worst overall call improved,
but aggregate runtime is not claimed to improve: the complete probes lasted
76.62 and 77.92 seconds respectively. The independent full probe reproduces
the candidate score of 0.804788.

## Verification and Reproduction

The trusted commands used were:

```sh
yukon setup
yukon run
cargo test --release --offline --locked
```

For candidate tests on macOS, use the same Seatbelt write roots and network
denial as the repository's local candidate build script. The commands inside
that sandbox were:

```sh
cargo test --release -p ssi-candidate-worker --offline --locked probe_insertions -- --ignored --nocapture
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked probe_timing_and_score -- --ignored --nocapture
```

The candidate suite passed 72 active tests, with 31 explicitly ignored
research probes. This includes the new insertion test on synthetic graphs
covering empty/singleton inputs, the lower gate boundary, and a bitset word
boundary. It also includes the existing exhaustive clique-cutoff comparison
on all five-vertex graphs and all permutations, plus the existing differential
bounded-versus-unbounded scorer checks. The trusted harness suite passed 66
active tests; two optional sandbox tests remained ignored. No test failure was
hidden or waived. Existing unused-code and documentation warnings remain.

Only `mod.rs`, `probe.rs`, and research documentation under `src/ordering/`
are intended for the submission archive. `results.tsv` was appended by the
trusted harness; its generated empty-note trailing tabs are not manually
edited and are outside the editable submission directory.

## Limitations and Follow-Up

The public development gain is modest and does not establish hidden-corpus
promotion. The hidden set is disjoint and its score and timing remain the
server's responsibility. Timing is wall-clock on one local macOS host and
can vary with load or architecture. Structural gates bound the dimensions,
input storage, and number of attempts, but do not prove that every unknown
graph completes inside the grader's deadline together with the inherited
portfolio. No larger-graph expansion or unbounded convergence loop is added.

The useful reusable result is a different monotone neighborhood using the
existing evaluator, coupled with an exact complete-suffix shortcut. Future
work could reuse unchanged elimination prefixes between insertions or reject
obviously commuting moves before full scoring. Either should first be tested
against the current exact evaluator and measured on an independent corpus.
Increasing the search budget or lifting the size gate without that evidence
would add runtime risk for a small development-score gain.
