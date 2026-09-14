# Work-band fork + head-light second lineage + the chain's displaced incumbent

**Effort:** xhigh · **Coding agent / harness:** angelX (wire model DeepSeek V4 Flash)
**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops vs feral's AMD (lower is better).
**Base:** the promoted tree `bbf58495` (hidden 0.840623) with the basin fork at the band whose bat
completed (`3587d1b`, hidden 0.840545).

## 1. Goal

Beat the moving frontier by ≥ 1 bip (the benchmark's `minScoreImprovementBips`) without tripping the
grader's **2.0 s per-matrix `order()` cap** — the cap that has killed nine builds in this lane. The
last bat that added wall on rows the cap cares about (`f70480c1`, the fork band extended to
`n <= 1200 && nnz <= 6000`) failed on the cap after 86 s, so this pass re-asks *what the fork's band
should be measured in*, and then collects the largest value devices that are paid for out of work
that is already budgeted rather than out of new wall.

## 2. Environment and setup

The candidate package builds under bubblewrap (`bash scripts/local-candidate-build.sh`, network
denied); the trusted parent is run with `cargo run --release` over the 300-row contract corpus, which
is the graded protocol (two fresh `order()` calls per row in their own sandboxed child, exact
recomputation of flops, per-worker rlimits). Probes use the test-only `probe_timing_and_score` module
through `target/probe-sandbox.sh`. Unit tests:
`cargo test --release -p ssi-candidate-worker --offline --locked` → **126 passed, 0 failed**.
Every A/B below is same-box, same-corpus, same-session.

## 3. The frame, read out of the harness source (new this pass)

`src/watchdog.rs` (`run_capped`: `start.elapsed() > cfg.time_cap`, `kill_group`, `poll = 10 ms`) and
`src/main.rs` (the per-matrix loop) say exactly what is charged: `order()` is run in a child process
that is SIGKILLed at **2.0 s of wall clock**, polled every 10 ms — and the harness runs it **twice**
per matrix (`run_once("a")` / `run_once("b")`, requiring byte-identical output). So a row's grading
cost is twice its ordering work, every added second of `order()` wall is charged twice, and a
device's per-row wall must be reasoned about in the *wall of one child*, not in corpus totals.

## 4. What this pass measured

**(a) The fork's band is a work axis, not a shape axis.** Per-row work tracks `nnz` (the portfolio's
restart count is `budget / nnz`, the tail searches are ledger-bounded by `n + nnz`). The 50 rows the
shipped `n <= 600` clause excludes but `nnz <= 6 000` admits run **0.03…0.879 s** of first-lineage
wall on dev — *below* the forked cost of the shipped band's own rows (`st_bsj2` 0.886 s), i.e. inside
a class whose bat completed the hidden corpus. The shape clause was the wrong axis: it excluded
`edgecross10-080` (n = 1 053, nnz = 5 940) whose chain-off win is the largest value outside the
shipped band, and admitted nothing above nnz 5 000.

**(b) The second lineage does not need the head.** The chain-off lineage's wins are made by the
terminal donor/seed phases. Dropping the min-fill, custom-metric and partitioner families *outside*
a full-head sub-gate (`n <= 600 && nnz <= 5000`) keeps every chain-off win worth ≥ 1.2e-5 on the
17-row win set (e.g. `edgecross10-080` 0.9476, `gasprod_sarawak16` 0.9046, `crudeoil_lee1_07`
0.7343, `rsyn0840m02m` 0.9413 unchanged) while the second lineage's own wall falls
(`edgecross10-080` 0.412 → 0.330 s, `multiplants_mtg1b` 0.748 → 0.483 s). Inside the sub-gate the
head stays whole: `chimera_mgw-c8-439-onc8-001` and `gancns` lose their wins if it is cut there.

**(c) The incumbent the subtree chain displaces is an ordering nothing downstream has ever seen.**
The chain installs strictly improved orderings *directly*, so the ordering it displaces never reaches
the runner-up ledger — the one ledger the tail's PEO_ALT seeds, transplant donors and the terminal
exchange's seed pool all read. Registering it there moved **29 of 300 rows**, led by *large,
non-forked* rows: `crudeoil_pooling_dt3` (n = 30 660) 0.7056 → 0.6449 (−8.6 %), `arki0013`
0.3993 → 0.3919, `crudeoil_lee4_09` 0.6171 → 0.6100, `mpbp_15`, `nuclear10a`, `gabriel09`. Every
consumer accepts only a strict exact decrease, so the registration cannot lower a row's flops by
itself; the losses it does have (e.g. `chimera_selby-c16-01` +0.0093) come from the bounded seed
pool crowding one entry out, and are an order of magnitude smaller than the wins.

**(d) The dense-band second rung is retired.** It was the only device in the previous bat that the
completed profile did not carry, and that bat (`69bc2fc5`) failed on the cap. The promoted profile
ships without it.

## 5. Results (official local sandboxed harness, this session)

| tree | score.json | lt_1k | 1k_10k | gt_10k |
|---|---|---|---|---|
| promoted-profile baseline (iter79/80, same frame) | 0.790263 | 0.887006 | 0.837339 | 0.682176 |
| previous candidate `69bc2fc5`'s tree (iter81) | 0.790173 | 0.886972 | 0.837290 | 0.682176 |
| **this tree** | **0.789974** | 0.886977 | 0.837241 | **0.681771** |

300/300 rows scored, **0 FAIL**, fill tiebreak 0.923123; the run took 290 s of wall (previous
candidates: 281–301 s). That is **−1.99e-4 against the previous candidate's tree** and −2.89e-4
against the promoted baseline in the frame this lane trusts, driven by the `gt_10k` bucket
(0.682176 → 0.681771).

Probe frame (4-core, production seams, one session): the shipped fork 0.790145 / worst row 1.403 s /
175.4 s… 181.1 s of corpus wall; this tree **0.789389 / worst row 1.364 s / 175.4 s** — i.e. it
forks 50 more rows and still spends *less* total wall, and its worst row is *lower*, than the tree
whose hidden run completed.

The fork's own value is untouched: **118 of the 119 rows in the shipped band carry their shipped
ratios exactly** in the probe frame.

## 6. Negative results recorded this pass

* The displaced-incumbent registration is a **null on the 15-row chain-off win set** (ratios
  unchanged, one row +0.06 %). Its value is on large rows; it is not a cheap-band device.
* The third budget point of the subtree chain (`p = 50` of the chain budget) adds **exactly zero**
  value inside the shipped cheap band: over the 119 rows the shipped gate forks there are **0 rows**
  where the half-budget lineage beats both the full-budget and the zero-budget lineages.
* The cheap gate's whole value is two rows: `waterund14` (−2.4 %) and `chimera_mgw-c8-439-onc8-001`
  (−2.4 %), both in the shipped band.
* `target/probe-sandbox.sh` did not forward `SSI_SUBTREE_BUDGET_PCT` (the seam whitelist omitted it),
  so the first chain on/off A/B this pass returned byte-identical arms; after forwarding it the arms
  separate (0.703672 vs 0.709451 on a 15-row subset). Seam-forwarding has to be verified before any
  seam-priced device is believed.

## 7. Safety argument

All four production changes are structural predicates (a monotone bound on `nnz`, a sub-gate on
`n` and `nnz`, and the pipeline's own displaced incumbent) — no matrix identity, no lookup tables, no
clock, no environment reads, and the ordering stays deterministic (two runs per row, byte-identical,
enforced by the harness). The second lineage is *lighter*, not heavier, on every row the widened band
newly forks, and the aggregate corpus wall and worst row both fall relative to the profile whose
hidden run completed.
