# 0148: the deferred independent-set lift gets its own pipeline pass

- **Date:** 2026-09-10
- **Score:** dev **0.792455 → 0.792370** (+1.072 relative bip), fill 0.924479 → 0.9245
- **Status:** win, shipped. Ceiling of the same mechanism is **2.796 relative bip**;
  1.724 of it is behind a cost gate.

Base: `8a12bda` (descendant of promoted frontier `62654a5`). All bips here are
**relative** (`Δ/base × 10⁴`) unless marked `abs`; `log.md` historically quotes
absolute bips, and at this dev score `rel = abs × 1.262`.

## Hypothesis

Stage 1b builds an independent-set lift. When it is not adopted at once (size
gate `n >= INDEP_FORCE_MIN_N`, or a 40 % `INDEP_IMMEDIATE_MARGIN` lead) it is
*deferred*, and stage 4b then compared it **raw** against an incumbent fresh out
of the 7-round subtree chain. That comparison is biased by a quantity neither
side knows — how much stages 2-4 will gain on the incumbent — so real lift wins
were thrown away. 0146 removed the `digabel` / `hydro` / `mid_force` `n`-windows
that had been papering over the bias with instance special-casing, at a cost of
1.7 dev bip. This tests the general rule those windows were a lookup table for.

## The deferral population is small, and that is what makes this affordable

One production pass per dev row, reporting stage 1b's decision
(`probe_indep_defer`):

| stage-1b outcome | rows |
|---|---|
| no lift, or none better than the incumbent | 261 |
| adopted at once by `n >= INDEP_FORCE_MIN_N` | 10 |
| adopted at once by `INDEP_IMMEDIATE_MARGIN` | 7 |
| **deferred** | **22** |

Of the 22 deferred lifts, 11 were accepted by the old raw comparison at 4b. Note
this is **not** the 210 rows `1b.indep` fires on — a fact worth having before
sizing any change to that stage.

## The ceiling: 2.796 relative bip, zero dev losers

Both arms run to completion on each deferring row (`probe_indep_arms`): the
shipped pass, and the pass with the lift installed as the stage-1b incumbent.
The lift arm wins 10, loses 5, ties 7. Taking the better **final** score per row
moves the corpus **0.792455 → 0.792233**.

| row | `n` | `nnz` | rel bip | reachable under the shipped gate |
|---|---|---|---|---|
| `methanol200` | 11 999 | 76 128 | 0.662 | no |
| `crudeoil_lee4_10` | 17 809 | 120 632 | 0.580 | no |
| `waterund14` | 333 | 2 204 | 0.537 | **yes** |
| `pooling_digabel19` | 514 | 5 340 | 0.473 | **yes** |
| `popdynm25` | 2 807 | 13 904 | 0.381 | no |
| `crudeoil_lee4_09` | 15 904 | 101 792 | 0.047 | no |
| `graphpart_3g-0244-0244` | 128 | 672 | 0.042 | **yes** |
| `torsion50` | 2 508 | 29 808 | 0.033 | no |
| `glider400` | 10 017 | 49 624 | 0.014 | no |
| `wastewater05m1` | 98 | 536 | 0.009 | **yes** |

(The column sums to 2.78 rather than 2.796 because the score is a weighted mean
of per-bucket geomeans, not a sum.) The gain survives dropping its largest
mover — 2.13 bip remain without `methanol200` — which is worth checking on any
candidate here, because a gain that evaporates when its top row is removed has
translated badly on hidden every time we have measured it.

## What changed

`src/ordering/mod.rs` only, plus `parallel.rs`/`probe.rs` for `#[cfg(test)]`
instrumentation.

`leader_order` becomes `leader_pass(pattern, forced_lift) -> (perm, score(perm),
deferred_lift)`. `order()` calls it twice:

1. `leader_pass(pat, None)` — the pipeline **unchanged**, including the old raw
   comparison at 4b. It also reports the lift stage 1b deferred.
2. `leader_pass(pat, Some(lift))` — the lift is installed as the stage-1b
   incumbent and gets stages 2-15.

The better **final** score wins. Gate on the second pass:
`CHAIN_BRANCH_MAX_N = 600`, `CHAIN_BRANCH_MAX_NNZ = 30_000`.

Also fixed: the terminal `rgreedy::subset_window_descent_step` acceptance did
not write its score back to `best_flops`. It was the fifth such site and the
only one the stage-14 `STALE_BEST_FLOPS` reporter never covered (it sits after
stage 14). Harmless while nothing read the value; an observable defect now that
`leader_pass` returns it.

## Result

- dev **0.792455 → 0.792370**, official 300-pattern harness run **OK**
  (bijection + determinism gates clean).
- Four rows improve: `waterund14` −234.96, `pooling_digabel19` −206.92,
  `graphpart_3g-0244-0244` −18.44, `wastewater05m1` −3.73 on-row bip.
- **Zero rows get worse.** Per-row diff over all 300 patterns: 4 changed, 0
  regressed.
- Cap, isolated `taskset -c 0-3`, median of 5: worst branched row
  `pooling_digabel19` **1.051 s**, below five rows already in the tail. Corpus
  worst stays `crudeoil_lee4_09` **1.180 s** — unchanged.

**Zero dev losers holds by construction, not by tuning.** Pass 1 is
byte-identical to the shipped pipeline and pass 2's permutation is returned only
when its score is strictly lower. The second pass is an *option*, never a
substitution. This matters twice: it is why the mechanism is safe, and it is why
its gate may be calibrated on **seconds** without selecting on the score
outcome — shrinking the gate forgoes gains and cannot make a row worse.

## The comparison must be on the FINAL score — the cheap version costs 13.2 bip

"Run the polish chain on both candidates and choose there" is the obvious
cheaper design, and it is **wrong**. Implemented faithfully (stages 2-4 as a
two-arm loop, `min_by_key` on the polished flops, then stages 5-15 on the
winner) it scores **0.793502 — 13.2 relative bip worse than doing nothing.** It
captures 9 of the 10 available wins and then throws away more than all of them
on one row:

| `mpbp_35` | after `4.subtree` | final |
|---|---|---|
| incumbent arm | 0.4227 | **0.3219** |
| lift arm | **0.4013** | 0.3942 |

`10.completion` takes the incumbent's basin 0.4225 → 0.3639 and `12.peo` takes
it to 0.3342. The same two stages, same row, same budgets, gain 0.0007 and
0.0035 on the lift's basin. **A candidate's flops at an intermediate stage says
nothing about what the later stages can do with its basin.**

Nor is there a later-but-still-early place to stand. The stage at which the two
arms cross varies by row: `3.search` (`chimera_lga-01`, `pooling_digabel19`),
`7.telos` (`popdynm25`), `9.reduce` (`mpbp_35`), `14.transplant`
(`pinene200`). Only the final score decides. This is the same mechanism that
made 0146's one-round polish at 4b cost 21 `abs` bip, and it generalizes: that
result was not about *how much* polish, it was about *where the comparison
stands*.

## Deciding before stage 4 is unsupported

The natural predictor — the lift's raw lead `f / best_flops` at stage 1b — does
not separate. Leads on rows the old 4b accepted: 0.9134, 0.9436, 0.9495, 0.9687,
0.9840, 0.9857, 0.9874, 0.9892, 0.9927, 0.9965, 0.9994. Leads on rows it
rejected: 0.9122, 0.9221, 0.9380, 0.9653, 0.9785, 0.9850, 0.9919, 0.9934,
0.9942, 0.9959, 0.9967. Complete overlap, and the extremes invert the
hypothesis: the corpus's *largest* raw lead (0.9122, `waterund14`) is a genuine
final win while 0.9221 (`mpbp_35`) is the catastrophic loss.

The labelled set is also tiny — 22 rows, 7 of them exact ties, leaves 16
informative rows — so a sweep over structural statistics here would be
chance-dominated. Do not spend a session on it; the two-arm branch makes the
predictor unnecessary.

## Why the gate stops at `n = 600`

A second pass roughly doubles a row's cost, so the gate must bound the cost of
the **class**, not of the dev rows currently in it — the corpus is hidden and
refreshed, and the risk is a row that both defers a lift and is expensive. The
dev cost envelope is a staircase and it steps out of reach immediately above
600:

| `n ≤` | slowest dev row in the class | doubled |
|---|---|---|
| 600 | 0.576 s (`chimera_mgw-c8-439-onc8-002`) | **1.153 s** |
| 1 000 | 0.786 s | 1.573 s |
| 3 000 | 0.950 s (`rsyn0805m03m`) | 1.900 s |
| 12 000 | 1.169 s (`arki0016`) | 2.338 s |
| 20 000 | 1.187 s (`crudeoil_lee4_09`) | 2.374 s |

An `nnz` bound does not help: the envelope is unchanged at every `nnz` cut from
30 k upward, because the expensive small and medium rows are cheap in `nnz` and
expensive in fixed LNS op budgets. So 600 is not a fitted number — it is the
last rung whose doubled worst case stays under the tree's existing worst row,
and the next rung up buys **no** additional dev gain while costing 0.4 s.

## Negative: trimming the second pass to afford a wider gate

Skipping `1.portfolio` and `3.search` in the second arm cuts its cost to
0.128–0.520 s per row and lets the gate reach `n ≤ 12 000, nnz ≤ 80 000`. The
corpus then scores **0.792365** — 0.06 bip better than the safe gate, for five
times the exposure:

- `methanol200`'s 0.662 bip vanishes completely,
- `pooling_digabel19` falls from −206.9 to −160.9 on-row bip,
- `graphpart_3g-0244-0244` and `wastewater05m1` lose theirs entirely,
- `popdynm25` gains −74.8 where the full pass gives −129.6.

**`1.portfolio`'s displaced candidates are load-bearing, not just its winner.**
`flush_batch` pushes every displaced ordering into `runner_up`, which is where
the stage-12/13 alternate seeds come from. A lift arm without them is much
weaker — even though every portfolio candidate on those rows scores *worse* than
the lift that replaced it. Its census `gained = 212` counts only half of what
that stage buys.

## Follow-ups

The remaining **1.724 bip** needs a second pass whose cost is bounded
*independently of the row's class* — i.e. one that can be **aborted** on a
deterministic work counter checked at stage boundaries. Aborting is free here:
pass 1 already holds a complete answer, so a truncated pass 2 forgoes a gain and
cannot lose a row. Per-stage cost on the deferring rows tops out near 0.35 s, so
a stage-boundary check bounds the overshoot to one stage.

Build the counter **inside the kernels**, not at the call sites:
`rgreedy::{search, subtree_refine, adjacent_pair_descent, simplicial_promotion,
subset_window_descent, subset_window_descent_step}`, `minl::*` and `core_lift::*`
all already receive an explicit `budget`, so ~8-10 functions give structurally
complete coverage where ~30 call sites give coverage that can silently drift.
Granting-budget accounting over-estimates consumed work, which is the safe
direction for a cap gate. Screen the counter against per-row seconds *before*
building a gate on it.

Second-pass cost with the stage-1/1b prefix shared (worth taking once the gate
widens — 0.19–0.61 s on the gt_10k rows — and worth nothing at `n ≤ 600`, where
`1.portfolio` costs 0.012–0.147 s):

| row | pass 1 | + pass 2, prefix shared |
|---|---|---|
| `crudeoil_lee4_09` | 1.170 s | 1.690 s |
| `crudeoil_lee4_10` | 1.160 s | 1.546 s |
| `ndcc12` | 0.739 s | 1.254 s |
| `mpbp_35` | 0.780 s | 1.213 s |
| `popdynm25` | 0.714 s | 1.137 s |
| `torsion50` | 0.595 s | 0.928 s |
| `pooling_digabel19` | 0.531 s | 0.922 s |
| `methanol200` | 0.590 s | 0.912 s |
| `waterund14` | 0.416 s | 0.752 s |
| `glider400` | 0.415 s | 0.695 s |

`crudeoil_lee4_09/10` stay out of reach either way.

## Measurement note that supersedes 0147's absolute seconds

0147 reported isolated `taskset -c 0-3` medians of `acopf_case9241pegase_qcqp`
1.4135 s, `gabriel10` 1.2572 s, `crudeoil_lee4_09` 1.2446 s, `arki0016`
1.2219 s. On production-identical code this session's readings in the *same*
instrument are 0.992, 1.041, 1.180 and 1.103 s — **1.2–1.4× lower, with a
different ranking** — while within-run spread in both sessions was only
1.00–1.16×. So the dispersion that matters when comparing two figures is
*between sessions*, not within a run, and it is large enough to reorder the
tail. **Re-measure the base in the same session as the change; never difference
against an inherited seconds figure.** What both sessions agree on is that the
tail is a flat band of 8-16 rows inside 1.0–1.4 s, so no targeting decision may
rest on which row leads it.

## Instruments added (all `#[cfg(test)]`, `src/ordering/probe.rs`)

- `probe_indep_arms` — both arms on every deferring row, per row and per bucket.
- `probe_indep_defer` — the deferral census plus the polish chain's seconds.
- `probe_indep_trajectory` — both arms' per-stage `(ratio)/(seconds)`
  trajectories. This is the one that shows where the arms cross, and the pattern
  to copy for any question about two candidates' whole-pipeline outcomes: one
  process, both arms, no env-var switch in production code.

```sh
cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_indep_arms
```

## Links

- Previous: [0146](0146-stale-incumbent-and-stage13-retirement.md) (the removed
  `n`-windows and the one-round 4b polish),
  [0147](0147-grader-core-count-and-indep-window.md) (the cap instrument),
  [0143](0143-independent-set-first-lift.md),
  [0145](0145-second-colour-class-metric-cores.md) (the lift itself).
