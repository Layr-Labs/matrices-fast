# 0184 — Timing resolution of the probe, and the first in-run draw price

Committed variant of this experiment: the sparse-band budget change it justifies
(0185). Evidence: `0184-probe-repro-a.log`, `0184-probe-repro-b.log`,
`0185-probe-density-shaped.log`.

## Why: the probe's own wall clock was never calibrated

Every per-row price this session shipped (the "one 2e8 draw costs 0.056 s mean /
0.187 s max" family, and every "worst `order()`" comparison between two builds)
was obtained by **differencing two runs**. Nothing had measured the noise floor
of that estimator, so a run of the same binary was taken this session, twice,
back to back (`0184-probe-repro-a`, `-b`, load 0.8 → 3.3 on this host).

| statistic | run A | run B |
|---|---|---|
| corpus `order()` total | 125.3 s | 127.8 s |
| printed `WORST order()` | 1.196 s | 1.118 s |
| `SCORE` / flops on all 300 rows | 0.792215 / identical | 0.792215 / identical |

Same binary, same code, same inputs: the score and **all 300 flop counts are
byte-identical** (`COUNTS` lines), while per-row *times* move by median
|Δ| = 0.0098 s, p90 0.040 s, max 0.156 s, and the headline "worst row" moves by
0.078 s (7 %). Both runs agree to 1.4e-6 on the score, so the *value* side of any
A/B is exact; only the *time* side needs care.

Cross-session this is far worse. `acopf_case9241pegase_qcqp` (n = 313 068,
nnz = 1 292 408) is outside every gate this session shipped, so its code path is
identical in all 21 probe logs with a TSV table:

    0.4846 (0151 baseline) 0.6929 0.7509 1.0260 1.1322 1.1486 1.1603 1.1630
    1.196 (0184-a) …

a 2.5× spread on an untouched row. Consequence: the shipped window's safety
argument ("heaviest in-window row 0.967 s vs the frontier's own worst 1.132 s")
and the report "one draw adds ≤ 0.237 s to any row" are **inside the noise
band**; neither was ever established at this host's resolution. The 21 archived
tables make the noise quantifiable: on rows that received **no** draw in any run,
cross-run spread is > 0.1 s on 267/300 rows and > 0.2 s on 209/300 (median
0.242 s, worst 0.709 s). That is host load, not the candidate.

## The instrument that fixes it

The probe prints two marks for the same row inside the same process: the
`LADGATE` line's third field is `t_pipeline.elapsed()` at the gate (before the
draw) and the TSV row's first field is the `order()` total. Their difference is
the draw's own price, measured **in-run** — no cross-run subtraction:

* 238 in-gate rows, two runs of the same binary: price mean 0.0537 / 0.0546 s,
  max 0.0957 / 0.1033 s;
* per-row repeatability of that price across the two runs: median |Δ| **0.0010 s**,
  p90 0.0051 s, max 0.015 s — 10× tighter than the cross-run estimator it
  replaces (median 0.0098, max 0.156).

Re-priced this way, the whole session's cost model changes shape:

| config | rows | price mean | p90 | max | corpus total |
|---|---|---|---|---|---|
| one 2e8 draw (0184-a) | 238 | 0.0537 | 0.0733 | 0.0957 | 12.8 s |
| four 1e8 draws (0181) | 240 | 0.1095 | 0.1468 | 0.2010 | 26.3 s |
| one 5e8 draw (0180) | 240 | 0.1396 | 0.1956 | 0.2625 | 33.5 s |
| 2×2e8 tiered (0176) | 239 | 0.1083 | 0.1583 | 0.2393 | 25.9 s |

so the earlier per-row add estimates were inflated by the run-level scale factor
(0180's own max was reported as 0.237 s; in-run it is 0.2625 only because the
budget is 2.5×). The price is **linear in the budget** (20 m: 0.0068; 50 m:
0.0149; 100 m: 0.0275; 200 m: 0.0537) — 0.00027 s per 1e6 ops — and
**density-dependent**, not size-dependent: `corr(price, nnz/n) = -0.445`,
`corr(price, nnz) = -0.398`, `corr(price, n) = -0.123`.

| band (`nnz/n`) | rows | price @200 m | price @500 m | ns per op |
|---|---|---|---|---|
| sparse `< 3` | 66 | 0.0629 | 0.1621 | 0.315 |
| mid `3…12` | 152 | 0.0559 | 0.1384 | 0.275 |
| dense `≥ 12` | 20 | 0.0322 | 0.0780 | 0.150 |

## What it says about the ladder

* The draw's price is set by the **budget**, and only weakly by the row; gating
  on `n` or `nnz` cannot bound it, but the sparse band is where the dearest
  per-row draws sit (max 0.096-0.111 s at 2e8) and where the corpus's own worst
  add lives.
* The sparse band's **value saturates early**: 3 of the 3 sparse movers
  (`rsyn0820m04m`, `rsyn0830m04m`, `rsyn0840m02m`) are already produced by the
  5e7 rung, at 0.0166 s mean / 0.0375 s max per row instead of 0.0629 / 0.0957.
  The mid band's 10 movers need the full 2e8; the dense band is the cheapest per
  op and its 2 movers are already there at 2e8.
* Left over: the price law is predictable *before* the draw runs (nnz/n), so a
  per-row budget that targets a fixed price is implementable as a pure predicate;
  0185 ships the coarse version of it.
