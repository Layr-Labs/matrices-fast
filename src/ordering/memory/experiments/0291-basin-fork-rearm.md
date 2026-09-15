# 0291 — re-arm the shared-prefix basin fork (cap cause retired, band widened)

- **Date:** 2026-09-15 (iter82)
- **Base:** iter81 `b21879f` — dev probe **0.790253862267** (300/300 `COUNTS`), hidden `849643ab` = 0.840510 vs the frontier 0.840511.
- **Change:** restore the shared-prefix **basin fork** and ship it in production, with the structural band widened from the ported `n <= 600 && nnz <= 5 000` to **`n <= 1 000 && nnz <= 10 000`** and the anchor margin held at **20 %**.
- **Public score:** **0.790161** (300/300), i.e. **−9.3e-5 dev (−0.93 bips)**, 4 movers, **0 regressions**, all inside `lt_1k`.
- **Status:** shipped and submitted.

## 1. Why the retirement no longer applies

The fork was removed from production by iter78 (`938598a`) and its scaffolding by
iter80 (`84fd4c0`), on a **cap** argument, never a value one. That argument is
now known to be wrong, and the lane proved it in the order it happened:

| iteration | tree | hidden Benchmark outcome |
|---|---|---|
| iter74–iter77 | fork present (margins 0 / 10 / 14 / 20 %) | killed at 2.0 s cap, 83.4–93.4 s in |
| iter78 (`782a26d`) | **fork removed entirely** | still killed, **84.681 s** |
| iter79 (`26f00f8`) | fork scaffolding removed | still killed, **84.360 s** |
| iter80 (`65c2e9d`) | last buffer divergence removed | still killed, **84.106 s** |
| iter81 (`849643ab`) | **promoted `Game` buffer lifecycle restored** | **COMPLETED, 8 min 02 s** |

iter81's receipt ([0289](0289-restore-promoted-buffer-lifecycle.md)) identifies
the cause as four `rgreedy.rs` image behaviours — a retained 1 GiB sparse image,
a recycled-image `new_sparse`, the reset-first constructor and a first-reset
fast path — none of which the fork touches. Removing the fork did not make the
kill go away; restoring those four behaviours did. So the fork's own cap record
is explained by a cause that no longer exists.

Independently, the public board's own version of this device (submission
`3587d1b`, PR #689, the same `n <= 600 && nnz <= 5 000` fence) **completed** a
hidden corpus at 0.840545.

## 2. The change

The iter80 removal is reversed verbatim against the current tree
(`git show 84fd4c0 -- src/ordering/mod.rs | git apply -R`): the two band
constants, the margin constant, `shared_basin_fork_band`, and the
`run_suffix` split at the `3.search` checkpoint with its
`prefix_score_workspace` / `runner_up: Arc<RefCell<..>>` ownership transfer, the
`stage4_signal` channel, and the strict-`<` merge over the two lineages.

Then three production deltas:

1. **Band `600 / 5 000` → `1 000 / 10 000`.** See §3.
2. **Margin stays 20 %**, compared as `best_flops * 100 < amd_flops * (100 - 20)`.
3. **One predicate for both frames.** The retired form had a `#[cfg(test)]` arm
   defaulting to *off* behind `SSI_SHARED_BASIN_FORK=1` and a `#[cfg(not(test))]`
   arm returning `false`; that is exactly the probe-vs-graded divergence trap.
   The shipped form is a single `shared_basin_fork_band` whose test-only seams
   (`SSI_FORK_MAX_N`, `SSI_FORK_MAX_NNZ`, `SSI_FORK_MARGIN_PCT`,
   `SSI_SHARED_BASIN_FORK=0`) all **default to the shipped constants**, so a
   plain probe reproduces the graded worker bit for bit and the whole device can
   still be priced in one binary.

What the fork does, unchanged: run the pipeline once through `3.search`, then
run the divergent **suffix** twice from that identical checkpoint — one lineage
with the stage-4 subtree cascade, one with it withheld — and return the
lineage with the strictly smaller **exact** `Σ c_j²`. It is monotone by
construction (the base is one of the two candidates) and appending it cannot
perturb any earlier stage.

## 3. The band widening is priced, not guessed

The ported fence admits 119 dev rows; 23 clear the 20 % margin. The next rows of
the same shape are `615..=969` vertices and `2 580..=7 260` nonzeros, all
already at ratio < 0.80:

| row | n | nnz | base ratio | fork@20 % narrow (600/5 000) | fork@20 % wide (1 000/10 000) |
|---|---:|---:|---:|---|---|
| `waternd2` | 615 | 2 580 | 0.4806 | — | unchanged |
| `multiplants_mtg1b` | 645 | 4 404 | 0.6627 | not admitted | **152 043 → 151 569 (−0.312 %)** |
| `multiplants_stg1` | 736 | 3 594 | 0.7853 | not admitted | unchanged |
| `multiplants_stg1b` | 814 | 3 992 | 0.6079 | not admitted | unchanged |
| `multiplants_stg5` | 827 | 3 804 | 0.4122 | not admitted | unchanged |
| `sonet23v4` | 803 | 6 462 | 0.6753 | not admitted | unchanged |
| `sonet24v5` | 874 | 7 260 | 0.5940 | not admitted | **110 276 → 110 162 (−0.103 %)** |
| `ndcc13` | 969 | 5 882 | 0.6183 | not admitted | unchanged |

Two of the eight move, no row regresses, and every admitted row is still under
1 000 vertices, so the added wall is bounded by the same argument that made the
narrow fence affordable. The fence remains a pure function of `(n, nnz)`.

## 4. Result

One binary, one session, 300/300 `COUNTS`, `SSI_MARK_NOSCORE=1`:

| arm | SCORE | lt_1k | 1k_10k | gt_10k |
|---|---:|---:|---:|---:|
| baseline `b21879f` | 0.790254 | 0.8873 | 0.8373 | 0.6822 |
| scaffolding restored, fork **off** | 0.790254 | — | — | — |
| fork on, band `600 / 5 000`, margin 0 % | 0.790165 | — | — | — |
| fork on, band `600 / 5 000`, margin 20 % | 0.790168 | 0.8870 | 0.8373 | 0.6822 |
| **fork on, band `1 000 / 10 000`, margin 20 % (shipped)** | **0.790161** | 0.8870 | 0.8373 | 0.6822 |

The movers, whole-corpus, shipped form:

| row | n | nnz | base flops | shipped flops | gain |
|---|---:|---:|---:|---:|---:|
| `waterund14` | 333 | 2 204 | 200 484 | **72 184 → 70 488** | −2.350 % |
| `chimera_mgw-c8-439-onc8-001` | 440 | 3 040 | 106 479 | **80 090 → 78 229** | −2.324 % |
| `multiplants_mtg1b` | 645 | 4 404 | 229 415 | **152 043 → 151 569** | −0.312 % |
| `sonet24v5` | 874 | 7 260 | 185 654 | **110 276 → 110 162** | −0.103 % |
| **4 rows** | | | | | **0 regressions** |

Every mover is in `lt_1k`; `1k_10k` and `gt_10k` are untouched
(0.8373 / 0.6822 both ways). That is the expected signature of a
cheap-row device and it is the honest reading of the transfer risk.

**Margin 0 vs margin 20.** At the narrow band, margin 0 measures 0.790165
against margin 20's 0.790168 — a **3.4e-6** difference bought by the single
row `gancns` (ratio 0.842, 58 299 → 58 202). The 20 % fence is kept because
`gancns` is the row the 14 %→20 % step was introduced to retire: it carries the
largest measured fork wall (+0.35 s) in the band for 97 flops.

**The scaffold is score-neutral.** Before the restore, the six known
fork-mover/wall rows plus controls produce byte-identical `COUNTS` records with
and without the `run_suffix` split in place, which is iter80's own finding
reproduced.

## 5. Cap trade

The device adds work only inside `n <= 1 000 && nnz <= 10 000 && best_flops*100 < amd_flops*80`, i.e. on rows the pipeline has already taken 20 % below the
AMD anchor. It **duplicates a suffix, not a prefix**: the portfolio, the
reductions, the search stages and the whole common prefix run once. The iter66
receipt charged the wall to the *near-anchor* rows (`himmel11` ratio 1.0000,
`syn15hfsg` 0.9915) — and neither row is admitted at a 20 % margin, in either
band, so the two rows that produced the original wall receipt are outside the
shipped fence by construction.

This is the same argument the class block's `past_anchor` gate uses, and it is
why the margin is not a tuning knob here but part of the safety argument.

### Measured wall, interleaved, one session

The admitted set is 30 dev rows (the wide fence at a 20 % margin) plus the two
rows the original receipt charged (`himmel11`, `syn15hfsg`). Three repetitions
of `on` and `off`, alternating, same binary, `SSI_MARK_NOSCORE=1`,
`SSI_SHARED_BASIN_FORK=0` as the off arm, `SSI_PROBE_ONLY` fixed:

| arm | rep1 | rep2 | rep3 | **median** |
|---|---:|---:|---:|---:|
| fork **on** (shipped) | 57.549 s | 55.931 s | 50.667 s | **55.931 s** |
| fork **off** | 48.754 s | 52.728 s | 46.473 s | **48.754 s** |

**+7.18 s over 33 rows = +14.7 % on the admitted set** — 30 rows of 300, so the
corpus-wide add is a fraction of that. Per-row medians, slowest adds first:

| row | n | nnz | off | on | delta |
|---|---:|---:|---:|---:|---:|
| `multiplants_mtg1b` | 645 | 4 404 | 2.767 s | 3.917 s | +41.6 % |
| `sonet24v5` | 874 | 7 260 | 2.869 s | 3.972 s | +38.4 % |
| `sonet23v4` | 803 | 6 462 | 2.728 s | 3.611 s | +32.3 % |
| `chimera_mgw-c8-439-onc8-001` | 440 | 3 040 | 2.218 s | 2.919 s | +31.6 % |
| `multiplants_stg5` | 827 | 3 804 | 2.903 s | 3.739 s | +28.8 % |
| `waterund14` | 333 | 2 204 | 1.578 s | 1.827 s | +15.8 % |
| `multiplants_stg1b` | 814 | 3 992 | 3.312 s | 3.541 s | +6.9 % |
| `multiplants_stg1` | 736 | 3 594 | 3.038 s | 3.260 s | +7.3 % |
| `ndcc13` | 969 | 5 882 | 2.951 s | 2.953 s | +0.1 % |
| `himmel11` (**not** admitted) | 14 | 60 | 0.720 s | 0.744 s | +3.3 % (noise) |
| `syn15hfsg` (**not** admitted) | 399 | 1 022 | 1.140 s | 1.162 s | +1.9 % (noise) |

`himmel11` and `syn15hfsg` are **outside the fence** and show no delta at all,
which is the fence doing its job: the two rows the original wall receipt
charged are not spent on. The absolute seconds here are a shared-host frame,
not a graded-host prediction, and are **not** claimed as a speedup; the only
claim is the ratio between the two arms measured in the same session on the same
rows. The admitted rows are all `n <= 1 000`, so their graded budget is small.

**The narrow fence is the fallback.** If the hidden verdict is a cap kill, the
one-line retreat is `SHARED_BASIN_FORK_MAX_N = 600` /
`SHARED_BASIN_FORK_MAX_NNZ = 5_000`, which is the fence the public analogue
`3587d1b` completed with; it costs 7e-6 of the 9.3e-5.

## 6. Refuted in the same pass

- **The simplicial reduction at any degree.** [0290](0290-simplicial-reduction-negative.md):
  a new exact core_lift rule (eliminate a vertex whose live neighbourhood is
  already a clique, at ANY degree, verified under a pair-check ledger) fires on
  most rows and shrinks cores by 20–98 %, and **300/300 flop records are
  bit-identical** — SCORE 0.790254 both ways. Widening `order_core`'s
  small-core quality gate from `1000..10_000` to `n <= 60 000` on top of it is
  also identical. A smaller core is not a better ordering; the core-size axis is
  closed as a value lever.
- **The five `gt_10k` anchor ties are not search failures.** `probe_tie_forensics`
  shows all five carry fill (`fillfree = 0`), but `probe_tie_headroom` finds 21
  diverse candidates (six AMD option pairs, three AMF α, METIS/tuned/high-trial,
  Scotch, RCM, Sloan, ND, six custom metrics) landing on **exactly
  1.00000000**. With a 180-vertex core (`squfl030-150`) also landing there, the
  reading is optimum, not failure: `fillfree = 0` is evidence about chordality,
  not about headroom.
- **The exchange sweep axis above the shipped 12** was started and abandoned as
  not worth the CPU once the fork's own measurement was in hand;
  `PRODUCTION_XCH_PLATEAU = 1` already stops the sweep loop at the first no-change
  sweep for every `n >= 10 000` class row, so the axis only reaches `n < 10 000`.
  Re-open it deliberately, not by accident.

## 7. Verification

- **Production candidate build** (`SSI_ALLOW_UNSANDBOXED_WORKER=1 bash scripts/local-candidate-build.sh`) — clean; trusted parent `cargo build --release -p matrices-fast --offline --locked` clean.
- **Determinism:** the ten-row control set (four movers, `gancns`, `himmel11`,
  `syn15hfsg` and three controls outside the band) run twice under the shipped
  defaults gives **byte-identical `COUNTS`** both times. The two lineages run on
  `std::thread::scope` but the merge is a strict `<` on the exact score, so the
  thread schedule cannot reach the output.
- **Release suite:** `cargo test --release -p ssi-candidate-worker --offline --locked`.
- **Full corpus:** `SCORE = 0.790161`, 300/300 `COUNTS`, 4 differing records,
  0 regressions.

## 8. Evidence

- iter81 completion receipt: `849643ab-f0cc-41fb-91bf-5d06eecacac9`, PR #747,
  Benchmark 16:55:45.0196689Z → 17:03:47.0960286Z.
- public fork analogue: `3587d1b`, PR #689, hidden 0.840545 (completed).
- local arms: `.session-backup/iter82-base-full.log` (0.790254),
  `.session-backup/iter82-fork-m0.log` (0.790165),
  `.session-backup/iter82-fork-m20.log` (0.790168),
  `.session-backup/iter82-fork-wide.log` (0.790161),
  `.session-backup/iter82-ship-full.log` (shipped defaults),
  `.session-backup/probe-iter82ship`.
- iter80 removal reversed: `git show 84fd4c0 -- src/ordering/mod.rs`.

## Links

- [0290 the simplicial reduction is a null](0290-simplicial-reduction-negative.md)
- [0289 restore the promoted `Game` buffer lifecycle](0289-restore-promoted-buffer-lifecycle.md)
- [0286 retire the basin fork](0286-retire-basin-fork.md)
- [0276 shared-prefix fork and the bounded dense-twin shape](0276-shared-prefix-fork-and-bounded-twin.md)
- [0271 basin fork on the cheap tier, and the exchange's width band](0271-basin-fork-and-width-band.md)
