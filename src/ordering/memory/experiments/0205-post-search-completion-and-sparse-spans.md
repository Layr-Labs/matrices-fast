# 0205 — Revisit the final completion, then refine sparse spans

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

## Submission status

Submission **`026c5a9c-00cb-4a1a-b8ed-03885d3f4bf5`** **failed the hidden
two-second per-matrix cap** in
[workflow 34710571494](https://github.com/Layr-Labs/matrices-fast/actions/runs/34710571494).
Its immutable remote commit is **`c508ad7377dfae5b18e89e12642db34b709a3e0a`**.
The entire remote `src/ordering/` was verified byte-identical to the tested
local submission commit **`a427c32`**. The Benchmark step ran from 18:17:57
to 18:19:44 UTC on 2026-09-12, approximately 106 seconds, before a hidden
matrix exceeded the cap. There is **no hidden score and no promotion**.
Subsequent status updates change only Markdown metadata. The next experiment
adds a factor-nonzero admission bound, removes above-bound eligibility work,
and measures lower-work continuations before any new submission.

## Target and measured starting point

The target is our promoted submission
`c5e6c2ff-f016-45e6-a2ff-31c3cb15cd75`, remote commit `fb35d11`, at hidden
flop score **0.842526**, fill **0.944945**. Its full workflow
[34708709139](https://github.com/Layr-Labs/matrices-fast/actions/runs/34708709139)
succeeded. The exact winning Rust source is saved on `fenced-terminal-window`;
this experiment branches from its documentation checkpoint `f5635cc`.
The recent submission list and benchmark metadata were read before this round.
The benchmark is open, Discussions are disabled, and promotion requires at
least one relative basis point of improvement. Lower is better.

The prior sandboxed 300-matrix baseline is **0.792030**, fill **0.924364**;
the exact diagnostic weighted score is **0.792029985254**. Counts in the
dimension buckets are 147 / 108 / 45, with weights 0.30 / 0.30 / 0.40.
Every new screen independently recomputes the original AMD reference and actual
column-count flop scores. It asserts the same exact starting aggregate and
requires each completed refinement to be a bijection and non-worsening.
The baseline and its understanding are recorded in [0203](0203-exact-kernels-terminal-window.md).

Development uses the existing ARM macOS host, Rust release builds and already
installed gcc, cargo, cargo-deny, git-lfs and Yukon. No tool reinstall, clone,
restart or hook setup is needed. All edits are under `src/ordering/`, with no
dependency additions or changes to the trusted harness or matrix corpus.

## Hypothesis

The promoted terminal greedy search and final exact width-eight pass create
a new completion after all earlier PEO extractions and fill-edge watchers have
finished. Those earlier rounds therefore cannot certify this final completion
as minimal. Re-extract PEOs and run a small watcher on the actual final incumbent,
then polish the resulting ordering. The placement is terminal and every
admission uses the actual original-pattern objective, so earlier portfolio
seeds, donor selection and alternate-seed chains are preserved.

The broader neighborhood is a contiguous span containing several disconnected
components in its live induced graph. A component can be reordered in its own
slots without affecting another component in the span. External vertices stay
live throughout the span. Eliminating the complete span leaves the same suffix
graph regardless of its internal order. This permits an exact subset DP on a
small component whose vertices are spread across a span wider than fourteen.

The new span helper accepts at most 64 positions, but **the exact component
ceiling remains fourteen**. It finds induced connected components with a u64
mask. Singleton and oversized components keep their original slots and order;
components of size two through fourteen use the existing union/signature DP.
Every preparation, component scan, DP and elimination replay is precharged
against the inherited deterministic work ledger. No beam approximation or
unchecked predicted gain is introduced. Completed improvements are retained
when the allowance expires, using the existing partial-gain behavior.

## Preserve the winning prefix and runtime safeguards

The 20-billion-flop producer fence still keeps the first 64 tasks of a batch,
in the same replay order. The complete portfolio and 50k / 4M alternate-PEO
envelope are retained. The terminal greedy allowance remains 50M when
`nnz < 3*n` and 200M otherwise, through `n <= 10,000`, `nnz <= 200,000`.
The exact sorted CSC, flat completion, linked MCS and cached metric kernels
from the promoted solver are unchanged.

The original terminal 8 / four sweeps / step 3 / 64M refinement is retained
under its original density and degree gate. The new continuation uses only
`6 <= n <= 12,000`, `nnz <= 200,000`, and an exact incumbent score at most
**20 billion**. Patterns outside the dimension/nnz bounds run no continuation.
Above the flop bound there are no extra PEO, watcher or window searches;
one exact eligibility score is still needed to determine that bound.
This is a structural predicate on the supplied pattern and its current
factorization work, never an identity or fingerprint.

Inside the continuation:

1. If the original density/degree gate excluded the pattern, apply the same
   width-eight / four-sweep / step-three / 64M refinement. The work allowance
   includes density-dependent preparation and replay. This extends coverage
   to hub patterns without an unbounded high-degree loop.
2. Extract forward/reverse MCS PEO candidates, for at most **two** rounds.
   Stop immediately if a round makes no strict exact improvement. Dimension,
   input-nonzero and factor-nonzero limits are **12k / 200k / 300k**.
3. Apply the inherited certified fill-edge watcher with **4M** operations.
   Its existing **300k factor nonzeros / 100k fill edges** limits still apply.
4. Run span **48**, four sweeps, step **19**, allowance **32M**; then width
   **9**, four sweeps, step **4**, allowance **32M**; then width **8**, eight
   sweeps, step **3**, allowance **64M**. Larger components in the span are
   skipped. The two narrow calls retain the original exact-width semantics.

Each stage compares an independently recomputed candidate score with the
fresh exact incumbent scalar, updating both on a strict decrease. Candidate
permutations are validated. Production uses neither elapsed time nor external
state. The scorer/permutation arenas remain invocation-owned.

## Initial independent window screen: gains were too small alone

All refinements started from the unchanged promoted final ordering. These
screens use the general 12k / 200k envelope; unchanged matrices still count in
the weighted aggregate. Times include scoring completed candidates and are
diagnostic ARM measurements, not a hidden-runner guarantee.

| Width / sweeps / step / allowance | Score | Wins by bucket | Extra CPU | Maximum |
|---|---:|---:|---:|---:|
| 7 / 4 / 2 / 32M | 0.792009187390 | 0 / 13 / 5 | 0.301642 s | 0.004471 s |
| 8 / 4 / 1 / 32M | 0.792017321480 | 0 / 9 / 0 | 0.345109 s | 0.004354 s |
| 10 / 4 / 3 / 32M | 0.792009469269 | 0 / 11 / 4 | 0.607927 s | 0.007314 s |
| 12 / 4 / 5 / 32M | 0.792017851119 | 0 / 5 / 0 | 0.925793 s | 0.008762 s |
| 8 / 8 / 3 / 64M | 0.791997604978 | 0 / 14 / 1 | 0.682929 s | 0.008844 s |
| 9 / 4 / 4 / 32M | 0.792006698948 | 0 / 13 / 4 | 0.457641 s | 0.005477 s |

Wider spans also improved a few previously unchanged small matrices:

| Span / step, four sweeps and 32M | Score | Wins by bucket | Extra CPU |
|---|---:|---:|---:|
| 16 / 7 | 0.792021020229 | 0 / 7 / 1 | 0.802549 s |
| 20 / 9 | 0.792022012456 | 4 / 7 / 2 | 0.720476 s |
| 24 / 11 | 0.792022398432 | 0 / 8 / 1 | 0.691414 s |
| 32 / 13 | 0.792018353843 | 3 / 8 / 2 | 0.656899 s |
| 48 / 19 | 0.791997326002 | 2 / 9 / 1 | 0.612915 s |
| 64 / 27 | 0.792002020166 | 2 / 10 / 3 | 0.582622 s |

No independent window arm cleared a credible promotion margin alone. Taking
the per-matrix minimum across these twelve independent arms would reach
0.791953797440, but that is an unimplemented twelve-arm portfolio, not a
claimed production score. Three-call sequences were measured instead.
The best initial window-only sequence, 48/32M then 9/32M then 8/64M,
reached **0.791961933130**, 2 / 19 / 4 wins, at 1.753114 s aggregate extra
CPU and 0.018734 s maximum. A fourth pass cost more without improving the
aggregate. These small standalone gains were not submitted.

## Completion placement and lower-work alternatives

Two late PEO rounds alone reached **0.791980413451**, 0 / 12 / 4 wins,
at only 0.151729 s extra CPU. The watcher alone reached 0.792010324395 at
4M and 0.792005411338 at 8M. Watcher before the three-call window sequence
reached **0.791897294334**; placing it afterward reached 0.791936975316,
even with a conditional extra width-eight pass. Placement changes the next
search basin while every individual stage remains monotone.

A final screen extended the original window gate, then compared PEO/watcher
placement and two allowance levels on all 300 patterns. "Small" windows are
48/16M then 9/16M then 8/32M (four sweeps each); "full" windows are the
selected 48/32M, 9/32M and eight-sweep 8/64M sequence.

| Continuation | Score | Wins by bucket | Extra CPU | Maximum |
|---|---:|---:|---:|---:|
| Coverage extension + PEO only | 0.791958302257 | 0 / 16 / 4 | 0.216780 s | 0.009691 s |
| PEO → watcher 4M → small windows | 0.791843246929 | 2 / 19 / 4 | 1.729824 s | 0.025964 s |
| **PEO → watcher 4M → full windows** | **0.791769290884** | **2 / 24 / 4** | **2.432096 s** | **0.033987 s** |
| PEO → small windows, no watcher | 0.791847176761 | 2 / 19 / 4 | 1.238228 s | 0.017868 s |
| Watcher 4M → PEO → small windows | 0.791896044187 | 2 / 19 / 4 | 1.688474 s | 0.025612 s |
| Watcher 4M → PEO → full windows | 0.791832621758 | 2 / 24 / 4 | 2.415569 s | 0.033979 s |

The selected continuation improves **30**, worsens **zero**, and leaves
**270** unchanged in this screen. Its public decrease is **0.000260694370**,
approximately **3.29 relative basis points**. This is a larger measured margin
than a single extra window search, while the largest measured continuation
cost remains 34 ms. Both lower-work alternatives remain documented rather
than silently mixed into the selected implementation.

The diagnostic caches only the promoted public incumbent permutations in a
temporary test file so later screens avoid re-running the whole portfolio.
They are independently rescored on every screen, with the exact aggregate
assertion above. This cache is neither included in the submitted archive nor
read by production. No matrix names or cached permutations guide production.

## Verification and reproducibility

Commands use the already approved local diagnostic opt-out for test-only
probes; the scored candidate is rebuilt and executed through `yukon run`'s
normal build and worker sandboxes.

```sh
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  ordering::rgreedy::window_dp::tests -- --test-threads=1
yukon run
```

All **seven active window-DP tests** passed. The new checks include a
64-position span with high mask bits, an eighteen-vertex skipped component,
an external neighbor shared by independent small components, exact suffix
graph equality, malformed span rejection, and 48 configurations confirming
that the generalized helper matches the original API inside the fourteen-
vertex limit. Existing independent exhaustive/boundary/tie/budget oracles
also pass.

The integrated sandboxed **`yukon run` passed all 300 matrices** at
**0.791769**, fill **0.924200**. The exact public weighted flop score is
**0.791769290884**. Every one of the 300 printed original AMD and candidate
flop counts matches the selected diagnostic arm, with **30 wins / 0 losses /
270 unchanged** relative to the promoted baseline. The bucket flop geomeans
are **0.887368 / 0.838074 / 0.685342**; bucket fill geomeans are
**0.959764 / 0.946238 / 0.880998**. The continuation is therefore verified
in the scored production path, rather than inferred from a test seam.

The paired generated stress probe uses eleven deterministic fixtures: sparse
random graphs at 2k/8k/12k dimensions, two denser 2k random graphs, 32-square
and 64-square grids, and a 2k hub. It alternates old/new execution order over
two pairs per fixture, checks identical permutations on repeat, independently
scores both arms, and retains the minimum same-host time. The control disables
only this continuation, leaving the promoted original terminal window active.
**All 44 complete orders pass determinism and bijection checks**, and every
new score is no worse. On the three above-bound cases, complete permutations
are identical, as well as their scores.

| Generated family | n / nnz | Baseline seconds | New seconds | Baseline flops | New flops |
|---|---:|---:|---:|---:|---:|
| Random, 4 links | 2048 / 16356 | 0.676885 | 0.685877 | 287672073 | 287672063 |
| Random, 20 links | 2048 / 81176 | 0.373211 | 0.389034 | 1563953334 | 1563675011 |
| Random, 40 links | 2048 / 160592 | 0.251637 | 0.269883 | 2052595742 | 2052550067 |
| Random, 2 links | 8000 / 31994 | 0.803815 | 0.825033 | 2313914749 | 2313914744 |
| Random, 4 links | 8000 / 63982 | 0.835352 | 0.868134 | 17375182561 | 17375182205 |
| Random, 8 links | 8000 / 127850 | 1.000016 | 0.986936 | 46260820576 | 46260820576 |
| Random, 10 links | 8000 / 159768 | 0.548316 | 0.561175 | 58257439844 | 58257439844 |
| Random, 4 links | 12000 / 95972 | 0.762877 | 0.756329 | 58806390558 | 58806390558 |
| Grid 32 | 1024 / 3968 | 0.324345 | 0.343614 | 195080 | 195080 |
| Grid 64 | 4096 / 16128 | 0.488484 | 0.516345 | 1848964 | 1848950 |
| Hub | 2048 / 4094 | 0.000011 | 0.000010 | 8189 | 8189 |

The largest new minimum time is **0.986936 s**; the largest measured minimum
time increase is **0.032782 s** on the below-bound 8k random graph. Faster
minimum times on a few unchanged arms reflect timing noise, not a claimed
speedup. The above-bound path still computes its exact eligibility score, so
it is not advertised as zero overhead. The hub returns through the unchanged
certificate fast path. These generated fixtures are independent of the dev
corpus, but are a finite stress screen rather than proof for every graph.

```sh
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  probe_terminal_followup_stress -- --ignored --nocapture --test-threads=1
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  -- --test-threads=1
```

The **complete active release suite passed: 123 passed, zero failed, 55 ignored**,
in 78.63 seconds with one test thread. No further production edits followed
the sandboxed scored run; the only later Rust edit clarified a comment.
The hidden grade is not yet available; no hidden improvement or promotion
is inferred from the public results.

Machine-readable evidence is included alongside the note:
[300 selected-arm comparisons](../evidence/0205-final-screen.tsv),
[official local score](../evidence/0205-local-score.json), and
[paired generated stress results](../evidence/0205-generated-stress.tsv).
The baseline permutation cache is deliberately excluded. Generated
`results.tsv` was restored and all changed paths are under `src/ordering/`.
`git diff --check` passes. The fresh full submission list still identifies
`c5e6c2ff` at **0.842526** as the promoted target immediately before packaging.

## Limits and learning

The public host is ARM, while GitHub grading is x86 Linux with an enforced
two-second per-matrix cap and four-GiB address-space cap. A good aggregate
score or small average continuation time cannot prove hidden runtime safety.
The preserved producer fence, structural continuation gate, factor limits,
fixed allowances and non-expanding exact component ceiling address work at
the point it is introduced. Full local and generated checks are necessary;
the actual grader remains decisive.

The useful result is the interaction between late PEO extraction, cleanup
and local refinement. Repeated independent windows alone were too weak.
Revisiting the completion **after** the previous final search exposes larger
gains, and a wider span can solve scattered small components without raising
the exponential component limit. The next decision is to use the official
grade: retain a promoted improvement, or record any cap/threshold rejection
before changing allowances. Identical failed retries are not a strategy.
