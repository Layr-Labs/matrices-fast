# 0166 — Terminal engine ladder (0154 census, priced by a budget ladder)

**Iteration 16.** Hypothesis: the 0154 engine census found the largest remaining
single-row gains by aiming `rgreedy::search` at the *finished* incumbent, and it
did so on only 37 rows — the "engine-affordable ungated" class. Priced over the
whole corpus that class is worth 3.33 bips, and 2.2 of those come from two rows,
which is the shape the log says does not carry. So instead of shipping the
census's 37-row class, this iteration **priced the same mechanism as a terminal
stage over every row with `n <= 12 000`** and shipped the affordable rungs.

## 1. Instrument: the self-inflicted-loss audit (`probe_eval_audit`)

`alt_lineage::note_scored` already sees every full-pattern score the pipeline
pays (`score()` in `leader_order`). Arming it around one `order()` call turns
that into a bound: **the shipped permutation is never better than the best
permutation the pipeline itself priced.**

Result over all 300 dev rows (`evidence/0164-eval-audit.log`):

```
AUDIT-SUMMARY rows=300  leak_rows=0  scored_candidates=3081
lt_1k  147  shipped=0.8875 min=0.8875
1k_10k 108  shipped=0.8397 min=0.8397
gt_10k  45  shipped=0.6857 min=0.6857
SCORE_SHIPPED = 0.792439  SCORE_MIN_EVALUATED = 0.792439  recoverable_bips = 0.00
```

Zero leaks, on every row, with 3081 scored candidates. The
best_flops-bookkeeping family (0163's stale-`best_flops` repair) has no
remaining *dev* headroom; what it has is an invariant, not a score. This also
reproduces the frozen-frontier score exactly with a second, independent binary.

## 2. The in-flight experiment: the `1800..=2500` substitution window

The window is a *substitution*, not an adoption band: `indep_first::run` returns
the 180a sequential arm instead of the general open-set path for that `n`. The
0151 mix measured `min()` over both arms and found no change; that cannot
distinguish "the general arm is equal in the band" from "the general arm is
worse in the band". A test-only counterfactual switch (`SSI_INDEP_NO_WINDOW`)
settles it directly, on the 17 dev rows with `1800 <= n <= 2500`
(`evidence/0165-window-on.log`, `evidence/0165-window-off.log`):

- 16 of 17 rows: byte-identical flops.
- `hydroenergy2` (n=2092, nnz=6236): 55946 -> 56014 flops, **+0.12 %**.
- band score 0.849147 -> 0.849208, i.e. **0.61 bips on the band**, which is
  **0.028 bips corpus-wide** (one row of 108 in `1k_10k`).

So `min()` did hide the general arm's value — on exactly one row — and the
window is worth 0.03 dev bips. It is a fingerprint-shaped predicate whose whole
measured value is one row; removing it is a compliance change, not a score
candidate, and it is *not* part of this submission.

## 3. Phase attribution (why more budget is the remaining lever)

`evidence/0161-tail-attribution-300.log` carries the per-phase ratio trajectory
for 276 rows; attributing a win to the phase in which the ratio strictly fell:

| phase | time (s) | rows won | of which > 1 % | last win |
|---|---:|---:|---:|---:|
| 4.subtree | 9.7 | 112 | 53 | 10 |
| 3.search | 21.0 | 68 | 27 | 12 |
| 14.transplant | 1.0 | 70 | 11 | 12 |
| 8.cleanup | 1.2 | 70 | 7 | 13 |
| 17.final | 2.2 | 52 | 1 | 23 |
| 19.five | 0.8 | 47 | 0 | 20 |
| 22.win | 2.1 | 39 | 0 | 39 |
| 13.alt | 6.7 | 3 | 0 | 1 |
| 20.lt1k | 2.8 | 3 | 0 | 1 |

Total `order()` time 115.6 s over 276 rows — 0.42 s/row against a 2 s cap. The
pipeline is **mechanism-starved on ~8 rows in 9, not time-starved**, and its
wins are dominated by `rgreedy`'s own neighbourhoods (subtree/search), which is
where an extra terminal draw of the same engine has the most to add.

## 4. The ladder measurement (`probe_engine_census` extended)

`SSI_CENSUS_BUDGETS` + `SSI_CENSUS_CLASS=all` + `SSI_CENSUS_MAX_N=12000` price
each rung from the finished incumbent, cumulatively, with per-row wall time:

```
LEVEL k=0 budget=200000000  wins=15/263 added_total_s=14.8  mean_added=0.056 worst_tip_plus_added=1.000
LEVEL k=1 budget=500000000  wins=24/263 added_total_s=52.0  mean_added=0.198 worst_tip_plus_added=1.124
LEVEL k=2 budget=2000000000 wins=28/263 added_total_s=200.8 mean_added=0.763 worst_tip_plus_added=2.102
```

Projected onto the full 300-row corpus (baseline 0.792439) and then *built into
`order()`* as a terminal stage, with the sandboxed-probe A/B as the check:

| shipped config | rows moved | probe SCORE | dev bips | mean add | worst affected-row add |
|---|---:|---:|---:|---:|---:|
| one 2e8 (`0167-terminal-ladder-2e8-only.log`) | 15 | 0.792215 | **2.20** | 0.042 s | 0.231 s |
| two 2e8 (`0167-terminal-ladder-2x2e8.log`) | 19 | 0.792188 | **2.51** | 0.080 s | 0.196 s |
| 2e8 + 5e8 (`0167-terminal-ladder-300.log`) | 24 | 0.792133 | **3.06** | 0.216 s | 0.470 s |

The prediction is exactly reproducible: the ladder's own per-row values appear
in the built binary. The 19 movers of the shipped config are spread over all
three buckets (`crudeoil_lee2_06` 0.75820 -> 0.71590 at 5.58 %, `chimera_rfr-02`
1.06 %, `multiplants_stg5` 0.75 %, `pooling_adhya4pq` 0.50 %, `pooling_sppa9tp`
0.50 %, `wastewater05m1` 0.38 %, …). Acceptance is `min` against the exact
score, so no row can regress; the gate is the monotone predicate `n <= 12 000`.

The 5e8 rung adds 0.55 bips for 2.7x the wall time, and the sandboxed harness
multiplies per-row cost (below), so **two 2e8 draws ship**.

Also fixed at the same site: the final `subset_window_descent_step` block wrote
`best_perm` from a local score without writing `best_flops` back. The audit
proves that gap cost nothing on dev (0 leaks), but it is the same stale-incumbent
window the 0163 repair closed elsewhere, and it now has to hold because the
terminal ladder prices the incumbent after it.

## 5. Sandboxed-harness runs (both FAILed, attributably)

```
1789195807 FAIL  demo7            (n=155,   nnz=618)     order() >= 2.0 s
1789195935 FAIL  pooling_sppb5pq  (n=18529, nnz=674470)  order() >= 2.0 s
```

The second row has `n > 12 000` — the ladder cannot touch it. The frozen
frontier failed **3 of 3** local sandboxed runs the same way, each on a
different row (`pooling_sppc1pq`, `p_ball_10b_7p_3d_h`, `sonet24v5`), and those
rows' in-process times are 0.42-0.55 s. Both new failures sit on rows whose
in-process time is 0.29-0.83 s, so this host's sandboxed run needs a factor of
2.4-3.4x over the in-process probe before the cap bites; the remote grader
cannot be doing that to the frontier, which is promoted with a 1.085 s
in-process worst row. The local cap verdict therefore has no discriminating
power here and the submission is decided by the probe: score, per-row movers,
and per-row added time.

## 6. Remote verdict (2026-09-12) — the ungated ladder is dead

The submission that carried this ladder (`c13df7a2`) came back `failed`; the
grader job log (Actions run `34679317718`) ends `RUN FAILED: hidden matrix:
order() exceeded the 2.0s per-matrix cap`, 114 s into a step that takes ~540 s
for a submission that completes. The ladder's added wall clock is therefore a
lost submission on the graded runner, not a local artifact. Fixed in
[0170](0170-structural-window-terminal-ladder.md): the same draws behind
deterministic `n` windows (2 x 2e8 for `n <= 7 000`, one 5e7 above), dev
0.792199 = 96 % of this page's value with the 2e8 draws removed from the 17
widest in-window rows.
