# 0182 — One draw, and never a new slowest row

Iteration 23. Two submissions on the 0167 terminal ladder had already been
killed by the grader's 2 s per-matrix cap; this page finishes that experiment
(pricing every rung), adds the board-wide kill-position census that frames the
failure, and ships the smallest rung that still buys > 2 dev bips.

## 1. The two cap kills, and where they landed

| submission | commit | Action run | grader start | kill | in-benchmark |
|---|---|---|---|---|---|
| `c13df7a2-67e4-…` (iter16, two 2e8 draws on every `n <= 12 000` row) | `100594a` | 34679317718 | 06:57:24.14 | 06:59:18.57 | 114 s |
| `bc0e0b6c-c8f9-…` (iter20, tiered 2×2e8 / one 5e7) | `4b7d480` | 34681315504 | 07:43:18.34 | 07:45:10.13 | 112 s |

Both logs end in exactly one failure line: `RUN FAILED: hidden matrix: order()
exceeded the 2.0s per-matrix cap and was killed`. A *completing* submission's
Benchmark step is ~495 s (`34681000152`: 07:36:06 → 07:44:21, score 0.8432;
`34681740480` queued 717 s wall). So both kills landed ~22 % into the corpus.
`src/main.rs:74` (`TIME_CAP_PER_MATRIX`) and `src/watchdog.rs:19` show what the
cap covers: `watchdog::run_capped` times the **whole sandboxed worker
subprocess** — spawn, pattern read, `order()`, and the permutation write — so an
add of `x` seconds per row is an add against a 2 s process budget, not against a
2 s `order()` budget.

## 2. Kill position is board-wide and near-deterministic

Reading the `Benchmark` step start line and the `RUN FAILED` line out of each
failed job log (`gh api /repos/Layr-Labs/matrices-fast/actions/jobs/<id>/logs`):

| run | submission (solver) | kill position |
|---|---|---|
| 34681315504 | bc0e0b6c (ours) | 112 s |
| 34679317718 | c13df7a2 (ours) | 114 s |
| 34679300793 | 1e251ca7 (other) | 101 s |
| 34665838580 | f7525b4b (other) | 101 s |
| 34665268892 | 7b67b90d (other) | 102 s |
| 34664436632 | 0b422df5 (other) | 101 s |
| 34650758923 | ea67f01b (other) | 99 s |
| 34649385295 | 3ed87513 (other) | 103 s |
| 34646761746 | 5cc9e038 (other) | 99 s |
| 34668584952 | 09bd34a9 (other) | **558 s** |

Nine of ten kills cluster at 99–114 s; one ran 558 s (longer than a completing
run's whole Benchmark step) and then died at the cap too. So the hidden corpus
does **not** contain one fixed killer row: the cap is reachable at many corpus
positions, and a candidate that adds per-row time anywhere inside the window it
covers can be the one that tips a row over. 5 of the last 24 benchmark runs
produced a score at all (`gh run list -L 100`); the same solver both completed
and died within minutes (`34562389425` 674 s vs `34561902002` 323 s).

## 3. Price list of the ladder (all 300 dev rows, probe `probe_timing_and_score`)

Every row below is the *same* mechanism in the *same* terminal placement,
seeded from the finished incumbent; only budget and window change. `raw gain` =
sum of per-row flop-ratio reductions vs the 0151 frontier page (0.792439).

| config | log | dev SCORE | bips | movers | raw gain |
|---|---|---|---|---|---|
| one 5e7 draw, `n <= 12 000` | `0179-wide-1x50m.log` | 0.792354 | −0.85 | 7 | 0.0247 |
| four 5e7 draws | `0181-four-seeds-4x50m.log` | 0.792299 | −1.40 | 12 | 0.0443 |
| four 1e8 draws | `0181-four-seeds-4x100m.log` | 0.792256 | −1.83 | 21 | 0.0555 |
| **one 2e8 draw, `n <= 12 000`** | `0179-wide-1x200m.log` | **0.792215** | **−2.24** | 15 | 0.0693 |
| two 2e8 draws, `n <= 12 000` | `0167-terminal-ladder-2x2e8.log` | 0.792188 | −2.51 | 19 | 0.0757 |
| one 5e8 draw, `n <= 7 000` | `0180-one-rung-5e8.log` | 0.792195 | −2.44 | 17 | 0.0699 |
| tiered (iter20, killed) | `0176-tiered-ladder.log` | 0.792199 | −2.40 | 17 | 0.0739 |

The budget is *saturated*: 2.5× the ops on the same seed and window
(2e8 → 5e8) buys +0.0009 of raw gain, while a second 2e8 draw on a **different
seed** buys +0.0064. The first draw is worth 91 % of the ungated two-draw
config's gain. Most of it is one row: `crudeoil_lee2_06` carries 0.0423 of
0.0693 (61 %).

The fixed price of a draw, measured per row as the difference between the
ladder log and the frontier log (same probe, same box):

* two draws: mean **+0.087 s**, p50 +0.097, p90 +0.171, max +0.237;
* one draw: mean **+0.023 s** over the 263 in-window rows, p90 +0.069;
* `corr(add, base_time) = −0.33`: the biggest adds land on the *cheapest* rows
  (`ex9_2_6` n=28 +0.237 s, `wastewater05m1` n=98 +0.206 s), because a draw
  costs ~0.05 s per 2e8 ops whatever the matrix is. Mean add on rows with base
  > 0.5 s is 0.054 s, on rows with base < 0.3 s it is 0.108 s.
* corpus-wide: frontier 118.1 s, tiered 144.2 s (**+22 %**), ungated 2×2e8
  135.4 s (+15 %), one draw 122.5 s (**+3.7 %**).

This is the shape the cap punishes: the add is a constant, and it is paid on
rows whose base time is unknown (hidden corpus) — including the rows the
pipeline is already slowest on.

## 4. Value sits on the *heavy* rows

Gain by the frontier's base time on the same row (config: one 2e8 draw):
`< 0.4 s` 0.0056 / `0.4–0.7 s` 0.0160 / `>= 0.7 s` **0.0477**. A gate that
excludes slow rows therefore throws away the mechanism, which is why the fix is
to shrink the *price*, not to hunt for a time predictor: over 300 dev rows the
best single-feature predictor of `order()` time is `log n` (r = +0.82 on
log-time), while flops is nearly useless (r = +0.38 raw; `transswitch0300p`
n=11 659 / 404 940 flops takes 1.017 s, `faclay75` with 3.2e9 flops takes
1.024 s). No deterministic in-pipeline meter was built this iteration; the
shipped answer is the smallest rung, plus the window the cap itself suggests.

## 5. Shipped shape (this iteration's candidate)

`src/ordering/mod.rs` — terminal ladder, one draw, gate `n <= 12 000 &&
nnz <= 200 000` (the second clause mirrors the two `nnz <= 200_000` gates that
already guard `21.comp`/`22.win` in the same function, and the harness's own
failure text: *"gate expensive paths by BOTH n and nnz"*). The clause is
*vacuous on dev* — all 15 movers have `nnz <= 44 264` — and the probe score is
byte-identical to the no-clause config, so it is pure hidden-corpus insurance
against dense in-window rows where both the pattern read and the pipeline's
`O(nnz)` stages are largest.

Safety argument, stated as the constraint the two kills impose: **the candidate
must not create a new slowest row.** Frontier global worst row 1.132 s
(`acopf_case9241pegase_qcqp`, n=313 068) is outside the window and is therefore
untouched by either build; the frontier's worst *in-window* row is 1.017 s
(`transswitch0300p`) and the candidate's is 1.029 s (`crudeoil_lee4_06`) — the
two are within the cross-run noise this box shows (±0.17 s on that row between
runs), and the candidate's corpus total is +3.7 % over the frontier against
+22 % for the killed tiered build.

### Measured this iteration

* production path (no env overrides), sandboxed probe: **SCORE 0.792215**,
  worst `order()` 1.135 s — `0182-probe-seedA-1x2e8.log`; identical to the
  test-switch config `0179-wide-1x200m.log`, i.e. the nnz clause costs 0.
* official sandboxed harness, full dev corpus: **300/300 OK, score 0.792215,
  fill 0.924449**, `results.tsv` row `1789201317` — `0182-local-harness-seedA.log`.
* seed sweep at the shipped price (one 2e8 draw, gate `n <= 12 000 && nnz <=
  200 000`): seed A `0x9E37_79B9_7F4A_7C15` **0.792215**, seed B
  `0xD1B5_4A32_D192_ED03` 0.792343, seed C `0xA24B_AED4_963E_E407` 0.792318
  (`0182-probe-seed{B,C}-1x2e8.log`). The mover *sets* are only partly shared
  (|A∩B| = 6, |A∩C| = 10, |B∩C| = 9, |A∪B∪C| = 22), so seed choice is worth
  more than the second rung — and A, already the first entry, is the best of
  the three.

## 6. Open, unproven

* Whether the hidden corpus scales our dev deltas: the ladder has never been
  *scored* remotely (both runs died before the upload step).
* Whether the kills were *caused* by the ladder or by the runner's own margin:
  a same-window submission from another solver completed at 0.842833
  (0.24 bips over the frontier) while two of theirs died at ~101 s. Reducing
  the price is the action that is correct under either reading.
* No deterministic in-pipeline work meter exists yet; `log n` is only r = 0.82.
