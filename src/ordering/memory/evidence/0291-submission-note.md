# Submission note — re-arm the basin fork, band widened to `n <= 1000 && nnz <= 10000`

**Tree:** iter82, on top of iter81 (`b21879f`), whose own submission
`849643ab` (PR #747) **completed** the hidden Benchmark step in 8 min 02 s at
**0.840510** against the frontier **0.840511** — an improvement of 1e-6, short
of the 1 bip bar.

**This tree:** public dev probe **0.790253862267 → 0.790161** — **−9.3e-5
dev (−0.93 bips)**, 4 movers, **0 regressions**, 300/300 `COUNTS` records.
Production candidate and trusted parent builds clean; release suite
**127 passed / 0 failed / 56 ignored**; determinism re-run byte-identical.

---

## 1. What changed

Three production deltas on top of the iter81 tree.

**(a) The shared-prefix basin fork is re-armed.** The pipeline runs once through
`3.search` and then executes only the divergent **suffix** twice, from that
identical checkpoint — one lineage with the stage-4 subtree cascade, one with it
withheld — and returns the lineage with the strictly smaller *exact* `Σ c_j²`.
The base lineage is one of the two candidates, so the device is **monotone by
construction**: it cannot perturb any earlier stage and cannot regress a row.
The prefix is not duplicated: the portfolio, the reductions and every search
stage before the checkpoint run once.

**(b) The structural band moves from the ported `n <= 600 && nnz <= 5 000` to
`n <= 1 000 && nnz <= 10 000`.** Priced row by row, not guessed — §3.

**(c) The anchor margin stays 20 %**, compared as
`best_flops * 100 < amd_flops * (100 - 20)`.

The shipped gate is therefore, in full and with its exact constants:

```
n >= 6 && n <= 1_000 && nnz <= 10_000
  && best_flops.saturating_mul(100) < amd_flops.saturating_mul(100 - 20)
```

It is a pure function of `(n, nnz, best_flops, amd_flops)`: no identity, no
clock, no environment, no hash order. `best_flops` is the incumbent's exact
score and `amd_flops` is the grader's own AMD anchor score, both already
computed by the pipeline.

**(d) One predicate for both frames.** The retired form carried a
`#[cfg(test)]` arm defaulting to *off* behind `SSI_SHARED_BASIN_FORK=1` and a
`#[cfg(not(test))]` arm returning `false`. That is a probe-vs-graded
divergence: a plain local probe would have run a different program from the
graded worker. The shipped form is a single function whose test-only seams
(`SSI_FORK_MAX_N`, `SSI_FORK_MAX_NNZ`, `SSI_FORK_MARGIN_PCT`, and
`SSI_SHARED_BASIN_FORK=0` as the off-switch) all **default to the shipped
constants**, so a plain probe reproduces the graded worker bit for bit while
the whole device remains priceable in one binary.

---

## 2. Why the 2026-09-14/15 retirement does not apply

The fork was removed for a **cap** reason, never a value one. The lane
established the real cause afterwards, in this order:

| iteration | tree | hidden Benchmark step |
|---|---|---|
| iter74–iter77 | fork present (margins 0 / 10 / 14 / 20 %) | cap-killed, 83.4–93.4 s |
| iter78 `782a26d` | **fork removed entirely** | still cap-killed, **84.681 s** |
| iter79 `26f00f8` | fork scaffolding removed | still cap-killed, **84.360 s** |
| iter80 `65c2e9d` | last buffer divergence removed | still cap-killed, **84.106 s** |
| iter81 `849643ab` | **promoted `Game` buffer lifecycle restored** | **COMPLETED, 8 min 02 s** |

Removing the fork did not remove the kill. Restoring four `rgreedy.rs`
buffer-image behaviours — a retained 1 GiB sparse image, a recycled-image
`new_sparse`, the reset-first constructor, and a first-reset fast path — did.
None of the four is on any path the fork exercises. The fork's cap record is
therefore explained by a cause this tree no longer contains, and the tree that
carries the device has already demonstrated a completed hidden Benchmark step
with a strictly larger wall budget than the device needs.

Independently, the public board's own version of this device — submission
`3587d1b`, PR #689, the same `n <= 600 && nnz <= 5 000` fence — **completed** a
hidden corpus at 0.840545.

---

## 3. The band widening is measured row by row

The ported fence admits **119** dev rows, of which **23** clear the 20 % margin.
The rows immediately outside it are `615..=969` vertices and `2 580..=7 260`
nonzeros, all already at ratio < 0.80, so they are the same shape the fence was
built for. Each was priced against the baseline before the constants moved:

| row | n | nnz | base ratio | narrow fence (600 / 5 000) | wide fence (1 000 / 10 000) |
|---|---:|---:|---:|---|---|
| `waternd2` | 615 | 2 580 | 0.4806 | — | unchanged |
| `multiplants_mtg1b` | 645 | 4 404 | 0.6627 | not admitted | **152 043 → 151 569 (−0.312 %)** |
| `multiplants_stg1` | 736 | 3 594 | 0.7853 | not admitted | unchanged |
| `multiplants_stg1b` | 814 | 3 992 | 0.6079 | not admitted | unchanged |
| `multiplants_stg5` | 827 | 3 804 | 0.4122 | not admitted | unchanged |
| `sonet23v4` | 803 | 6 462 | 0.6753 | not admitted | unchanged |
| `sonet24v5` | 874 | 7 260 | 0.5940 | not admitted | **110 276 → 110 162 (−0.103 %)** |
| `ndcc13` | 969 | 5 882 | 0.6183 | not admitted | unchanged |

Two of the eight move, none regresses, and every newly admitted row is still
under 1 000 vertices, so the added wall is bounded by the same argument that
made the narrow fence affordable.

---

## 4. Result — one binary, one session, 300 rows

`SSI_MARK_NOSCORE=1` (the graded-closest probe frame) throughout:

| arm | SCORE | lt_1k | 1k_10k | gt_10k |
|---|---:|---:|---:|---:|
| baseline `b21879f` | 0.790254 | 0.8873 | 0.8373 | 0.6822 |
| fork scaffolding restored, fork **off** | 0.790254 | 0.8870 | 0.8373 | 0.6822 |
| fork on, narrow fence, margin 0 % | 0.790165 | — | — | — |
| fork on, narrow fence, margin 20 % | 0.790168 | 0.8870 | 0.8373 | 0.6822 |
| **fork on, wide fence, margin 20 % (shipped)** | **0.790161** | 0.8870 | 0.8373 | 0.6822 |

Whole-corpus movers, shipped form:

| row | n | nnz | base flops | shipped flops | gain |
|---|---:|---:|---:|---:|---:|
| `waterund14` | 333 | 2 204 | 200 484 | 72 184 → **70 488** | **−2.350 %** |
| `chimera_mgw-c8-439-onc8-001` | 440 | 3 040 | 106 479 | 80 090 → **78 229** | **−2.324 %** |
| `multiplants_mtg1b` | 645 | 4 404 | 229 415 | 152 043 → **151 569** | **−0.312 %** |
| `sonet24v5` | 874 | 7 260 | 185 654 | 110 276 → **110 162** | **−0.103 %** |
| **4 rows** | | | | | **0 regressions / 296 identical** |

Every mover is in `lt_1k`; `1k_10k` and `gt_10k` are bit-identical to the base
(0.8373 / 0.6822). That is the honest signature of a cheap-row device, and it is
why the margin is part of the safety argument rather than a tuning knob.

**Margin 0 vs margin 20, measured.** At the narrow fence the margin-0 arm reads
0.790165 against margin 20's 0.790168: a **3.4e-6** difference bought by the
single row `gancns` (ratio 0.842, 58 299 → 58 202). The 20 % fence is kept
because `gancns` carries the largest measured fork wall in the band (+0.35 s)
for 97 flops; it is the row the 14 %→20 % step was introduced to retire.

---

## 5. The cap trade

* **Where the device spends.** Only inside
  `n <= 1_000 && nnz <= 10_000 && best_flops * 100 < amd_flops * 80` — that is,
  on rows the pipeline has already taken 20 % below the AMD anchor. The fence is
  structural (`n`, `nnz` only).
* **What it duplicates.** The *suffix* from the `3.search` checkpoint, not the
  prefix. The portfolio (the wall leader), the reductions, the subtree cascade's
  input construction and every search stage before the checkpoint run once.
* **The rows the original wall receipt charged are outside the fence by
  construction.** The iter66 receipt blamed `himmel11` (ratio 1.0000, +0.68 s)
  and `syn15hfsg` (ratio 0.9915, +0.83 s). Neither clears a 20 % margin, so
  neither is admitted — in either fence. Requiring the margin is the same
  argument the terminal class block uses for its own `past_anchor` gate: a fork
  re-runs the suffix on an ordering the portfolio already displaced, so on a row
  the pipeline never moved off the anchor there is no second basin and the
  duplicate suffix is pure cost.
* **Measured wall on the admitted set** (30 admitted dev rows plus the two rows
  the original receipt charged, three alternating `on`/`off` repetitions in one
  session, same binary, `SSI_MARK_NOSCORE=1`, off arm via
  `SSI_SHARED_BASIN_FORK=0`):

  | arm | rep1 | rep2 | rep3 | median |
  |---|---:|---:|---:|---:|
  | fork **on** (shipped) | 57.549 s | 55.931 s | 50.667 s | **55.931 s** |
  | fork **off** | 48.754 s | 52.728 s | 46.473 s | **48.754 s** |

  **+14.7 % on 33 rows**, i.e. on the device's own footprint — 30 of 300 corpus
  rows — with the two rows the original receipt charged (`himmel11` ratio 1.0000
  and `syn15hfsg` 0.9915) reading **+3.3 % and +1.9 %**, i.e. noise, because
  neither clears the 20 % margin. Per-row medians are in the tree's experiment
  page. Absolute seconds there are a shared-host frame, not a graded-host
  prediction, and **no speedup is claimed anywhere**; the only claim is the
  ratio between the two arms measured in the same session on the same rows.
* **The wall budget is not the binding constraint on this tree.** The tree this
  change sits on completed the hidden Benchmark step in 8 min 02 s, and the
  2.0 s per-matrix kill that ended eight earlier submissions was traced to a
  buffer-lifecycle divergence that is no longer present.
* **The retreat is one line.** If the verdict is a cap kill, restoring
  `MAX_N = 600` / `MAX_NNZ = 5_000` is the fence the public analogue `3587d1b`
  completed with, and it costs 7e-6 of the 9.3e-5.

---

## 6. What was measured and rejected in the same pass

* **The simplicial reduction, at any degree.** A new `core_lift` rule that
  eliminates any vertex whose live neighbourhood is *already* a clique — a legal
  exact elimination step at any degree, verified under a charge-before-attempt
  pair-check ledger — fires on most rows and shrinks cores by **20–98 %**
  (`squfl030-150`: 13 680 → **180**; `emfl050_5_5`: 13 175 → **650**;
  `mpbp_34`: 11 556 → **2 610**). Full 300-row arm: **every flop record
  bit-identical, SCORE 0.790254 both ways, 0 differing rows.** Adding a widened
  `order_core` small-core quality gate (`1000..10_000` → `n <= 60 000`) on top of
  it is also identical on every sampled row. A smaller core is not a better
  ordering: the core search's output is a whole-graph value and the pipeline
  already reaches the same value on the original core. **Sealed.**
* **The five `gt_10k` anchor ties are not search failures.**
  `probe_tie_forensics` shows all five carry fill (`fillfree = 0`), but
  `probe_tie_headroom` finds **21 diverse candidates** — six AMD option pairs,
  three AMF α, METIS default/tuned/high-trial, Scotch, RCM, Sloan, ND and six
  custom metrics — all landing on **exactly 1.00000000**, i.e. AMD's own value.
  With a 180-vertex core also landing there, the reading is *optimum*, not
  failure: `fillfree = 0` is evidence about chordality, not about headroom.
* **The stale-`best_flops` family has no dev headroom.** `probe_eval_audit`:
  `rows=300 leak_rows=0 scored_candidates=3121`, `recoverable_bips = 0.00` — the
  pipeline ships the best permutation it scored on every row.
* **Margin 0** (any strict win over the anchor): +3.4e-6 of value for a wider
  admission set. Not taken.

---

## 7. Verification

* production candidate build (`bash scripts/local-candidate-build.sh`): **clean**;
  trusted parent `cargo build --release -p matrices-fast --offline --locked`:
  **clean**.
* release suite `cargo test --release -p ssi-candidate-worker --offline --locked`:
  **127 passed / 0 failed / 56 ignored**.
* full 300-row dev probe with **shipped defaults** (fork on, both seams unset):
  `SCORE = 0.790161`, **300/300 `COUNTS`**.
* **determinism:** the ten-row control set (the four movers, `gancns`,
  `himmel11`, `syn15hfsg`, and three rows outside the fence) was run **twice**
  under the shipped defaults and produced **byte-identical `COUNTS`** records
  both times. The two lineages run on `std::thread::scope`, but the merge is a
  strict `<` on the exact score, so no thread schedule can reach the output.
* every changed path is under `src/ordering/`.

---

## 8. Provenance

* **This lane's tree:** `b21879f` (iter81), submission `849643ab`, PR #747.
* **The frontier this is measured against:** `f1782ef` (Xo1otl), PR #719,
  "sparse-pristine terminal exchange", hidden 0.840511.
* **The device's own history in this lane:** PR #707 (`7febce96`, cap-killed),
  PR #689 (`3587d1b`, hidden 0.840545, completed), PR #710 (`6506934a`),
  PR #743–#746 (fork-off controls, all cap-killed at the same corpus position
  as their fork-bearing predecessors).
* **The cap-cause receipt:** PR #747, workflow `34997790290`, Benchmark step
  `16:55:45.0196689Z → 17:03:47.0960286Z`.
