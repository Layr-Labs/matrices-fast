# 0222 — the sparse-span schedule extended by four measured widths

- **Date:** 2026-09-12 (iter47)
- **Score:** production frame, one session, one binary, 4-vCPU (`taskset -c 0-3`), 300 dev rows:
  shipped 5 windows **0.791635** (worst 1.221 s) -> 9 windows **0.791586** (worst 1.266 s);
  **14 rows better, 0 worse, 286 identical**; corpus wall +5.0 % (132.9 -> 139.6 s).
- **Status:** shipped (`PRODUCTION_SPAN_WINDOWS` = 9), official local harness 300/300 at
  0.791586 / 0.924134 (`results.tsv:1789259342`), submitted **e07fe7ae** (validating).
- **Page:** [0222-span-schedule-extension.md](0222-span-schedule-extension.md)

## Hypothesis

Every width appended to the sparse-span schedule so far has paid on the shipped point
(3 -> 5 widths = -3.0e-5, iter43, now entries 4-5 of `PRODUCTION_SPAN_WINDOWS`), and each
pass accepts only a strict exact decrease. So the schedule is a schedule, not a fixpoint,
and the residual value of the *next* widths is exactly measurable in-frame.

## What changed

`PRODUCTION_SPAN_WINDOWS` 5 -> 9 entries: appended `(10,4,4,64M)`, `(11,4,4,64M)`,
`(14,4,5,64M)`, `(6,4,3,64M)`. Measured first behind the test-only append seam
`SSI_SPAN_WINDOWS_EXTRA`, which was then removed (the array is the single source of truth
again; a no-env test build reproduces the shipped `order()`).

## Result (all arms `SSI_INDEP_FORCE=1`, same session)

| arm | env | SCORE | worst order() | corpus |
|---|---|---:|---:|---:|
| P | prod 5 windows | 0.791635 | 1.221 s | 132.9 s |
| X | +4 widths | **0.791586** | 1.266 s | 139.6 s |
| X22 | + 4 widths + `SSI_TERM_CLASS_N=22000` | 0.791583 | 1.288 s | 142.3 s |

Movers: crudeoil_lee2_06 -0.56 %, wastewater05m1 -0.42 %, sporttournament48 -0.34 %,
rsyn0840m04m -0.12 %, powerflow0300p -0.09 %, chp_shorttermplan1a -0.09 %, syn10m04m
-0.07 %, transswitch0300p -0.05 %, then six below 0.05 %.
[0222-span-widths-extra-4cpu.log]

## Why the two rejected arms are structural negatives (not just small)

- `SSI_TERM_CLASS_N` 12000 -> 16000/22000 = **-3e-6** (+0.11..0.15 s worst row). The
  window/span family is gated *inside* `rgreedy` (`MAX_N = 12000` in `Game::build_adj` /
  `Game::new`), so on the ten newly admitted rows the CLASSTRACE trace shows the exchange
  returning `candidate=0` on every row and the followup's spans leaving the value unchanged
  on all five rows that pass the 150 k factor key. The band's rows with structure have
  factor-nonzero 365 k-690 k and stay behind the key.
  [0220-classn-coverage-4cpu.log, src/ordering/rgreedy.rs:88, rgreedy/window_dp.rs:285]
- `SSI_FOLLOWUP_FACTOR` 150 k -> 1 M = **-1.0e-5** (+0.106 s worst row). Rejected: a tenth
  of a bip is not worth relaxing the exact admission bound the recorded frontier credits
  for surviving the 2 s cap. [0221-factor-key-ceiling-4cpu.log]

## Side measurements recorded this iteration (tools for later)

- Cross-build per-row oracle against the public leader's probe (`0208-leader-probe-4cpu.log`
  vs our `0209b-shipped-composite-mirror-4cpu.log`): **280 of 300 rows are identical**; the
  leader beats us on 2 rows, total recoverable 8.6e-6. Cross-build porting is exhausted.
- The class block's own dev value, recovered as the `22.win` -> `final` delta over the
  300-row phase dump (`0204-full-phases.log`): **4.71e-4 weighted over 41 rows**, all
  `n <= 12 000`.
- Per-stage cost/yield table over the same dump: `1b.indep` 3.43 nats / 6.2 s (best yield),
  `4.subtree` 2.32 / 9.4, `3.search` 1.44 / 20.5, `9.reduce` 1.06 / 14.2, while
  `13.alt` + its `13p.*` sub-phases spend **13.9 s of corpus time for 0.005 nats on 2 rows**
  (`20.lt1k` 0.017 nats / 2.8 s). Candidate time-negative deletion sites.
- The five `gt_10k` rows that tie at AMD (`emfl100_5_5`, `kissing2`, `supplychainr1_053050`,
  `squfl030-150`, `emfl050_5_5`) stay pinned: a 4000-eval insertion search finds
  `strict=0` on all five and the tie-headroom battery's best is 1.0000.
  [0159-insertion-gt10k-ties.log, 0153-tie-headroom-battery.log]

## Follow-ups

1. Price a fifth width pair at a *reduced* ledger (value-per-second; the pass budget is the
   only cost axis left inside the class).
2. Delete or curtail `13.alt`/`13p.*` (measured dead weight) and spend the freed time inside
   the class's schedule.
3. The class's `n`-band is only reachable by lifting `rgreedy::MAX_N` for the window-DP
   family alone (bitset adjacency ~n^2/64 words, ~60 MB at n = 22 000) — and only for rows
   whose factor key admits them.

## 0222b — the NEXT width group is measured and prepped (not shipped yet)

Same frame, one session, one binary, after the nine-width build was shipped:
`P` (shipped 9) **0.791586** (worst 1.362 s) vs `E2` (9 + `13/4/6, 16/4/6, 5/4/3, 24/4/11`)
**0.791572** (worst 1.403 s) = **−1.4e-5**, **8 rows better / 0 worse**, corpus +2.9 %.
Movers: `chimera_mgw-c16-2031-01` −0.13 %, `crudeoil_lee1_07` −0.09 %,
`rsyn0840m04m` −0.07 %, `crudeoil_lee2_06` −0.07 %, `chimera_mgw-c8-439-onc8-002` −0.07 %,
`chimera_selby-c16-01` −0.04 %, `crudeoil_pooling_ct1` −0.04 %, `sporttournament48` −0.03 %.
[0226-next-width-group-4cpu.log]

Residual value is diminishing (4 widths = −4.9e-5, next 4 = −1.4e-5), so the width group is
**kept in the test seam** and is *not* the next submission on its own: −1.4e-5 dev cannot
clear a 1-bip promotion floor. It is the payload for a candidate that first buys back its
wall time by deleting the measured dead weight (`13.alt` + `13p.*`: 13.9 s of corpus time
for 0.005 nats).
