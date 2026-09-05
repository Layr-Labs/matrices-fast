# 0003 — Relabelled-AMD multi-start, on a per-matrix time budget

**Date:** 2026-07-26
**Score:** 0.883906 → **0.876925** (fill 0.965241 → 0.962248), 300-pattern dev corpus
**Status:** WIN, committed

The largest single gain measured on this problem so far. For scale,
[0002](0002-measured-gates-metis-kahip.md)'s entire 12-variant partitioner sweep
bought 0.0042; this buys 0.0070.

## The idea

[0002](0002-measured-gates-metis-kahip.md) left the portfolio near the ceiling of
partitioner-parameter tuning, and left a specific reason why: **122 of 300
matrices were tied at exactly 1.000**. On those, AMD beat every separator-,
profile- and bandwidth-based candidate in the portfolio. No amount of METIS or
KaHIP tuning addresses that set, because the whole separator family already loses
there.

But AMD's output is decided by its tie-breaking, and its tie-breaking reads the
vertex **numbering**. So running feral's own AMD on a relabelled copy of the
pattern, `B = Q A Qᵀ`, and composing the result back through `Q`, gives a
genuinely different minimum-degree ordering for the cost of one AMD pass — a
randomized-restart minimum degree with no MD implementation to write. A
*different AMD* is the one family never tried on the set where AMD itself wins.

## Step 1 — does the family work? (`probe_relabel_amd`)

Flat restart counts on everything under n<40k / nnz<200k:

| restarts | score |
|---|---|
| current | 0.883906 |
| 4 | 0.878255 |
| 8 | 0.877292 |
| 16 | 0.875431 |
| 24 | 0.874024 |

**41 of 300 matrices improved**, against 7 of 260 for the whole partitioner
sweep. The wins are large and land on former ties:

| matrix | n / nnz | before → after |
|---|---|---|
| `chp_shorttermplan2d` | 16364 / 52108 | 0.7638 → **0.5355** |
| `crudeoil_lee4_10` | 17809 / 120632 | 0.9313 → **0.7042** |
| `crudeoil_lee4_09` | 15904 / 101792 | **1.0000** → 0.8257 |
| `chimera_lga-01` | 1120 / 6400 | **1.0000** → 0.9112 |
| `mpbp_21` | 11716 / 37660 | **1.0000** → 0.9195 |

## Step 2 — a flat count is unshippable

Restart cost stacks on top of each matrix's existing `order()` time, and 24
restarts costs 1.444 s on `nuclear10a`, 1.176 s on `ringpack_30_2`. That is an
instant cap breach. But cost is roughly `k · nnz` per restart, and `k` is
*nearly* stable — which suggests a budget rather than a count.

Checked that assumption rather than assuming it:

| matrix | nnz | k = s/(restart·nnz) |
|---|---|---|
| `methanol400` | 151728 | 1.33e-7 |
| `crudeoil_lee4_09` | 101792 | 2.20e-7 |
| `nuclear10a` | 163816 | 3.67e-7 |
| `ringpack_30_2` | 121458 | 4.03e-7 |
| `sfacloc2_3_80` | 20600 | **9.36e-7** |

A **7× spread** — cost depends on structure, not nnz alone. So the budget must be
sized on the worst `k`, not the mean. Rounding it up to `k = 1e-6` makes the
budget constant read directly as microseconds of restart time.

## Step 3 — score the budget policy (`probe_relabel_budget`)

`restarts = budget / nnz`, clamped to `cap`. `worst_s` is measured `order()` time
**plus** measured restart time, per matrix:

| budget | cap | score | worst combined | improved |
|---|---|---|---|---|
| 150000 | 24 | 0.879253 | 0.925 s | 36 |
| **300000** | **24** | **0.876925** | **0.978 s** | **40** |
| 300000 | 48 | 0.876879 | 0.978 s | 41 |
| 450000 | 24 | 0.876757 | 1.027 s | 40 |
| 600000 | 48 | 0.876130 | 1.079 s | 42 |
| 900000 | 96 | 0.875194 | 1.183 s | 44 |

**300000 / cap 24 is the knee** — past it each further 0.05 s of worst case buys
under 0.0002 of score. The cap barely matters (24 vs 48 vs 96 differ by <0.0001)
because the wins land in the first handful of restarts.

The budget is **its own gate**: `nnz > budget` yields zero restarts, so unlike
every other candidate in `order()` this one needs no `(n, nnz)` cutoff.

## Result

Probe predicted 0.876925; the harness measured 0.876925, bucket for bucket
(lt_1k 0.9073 · 1k_10k 0.9069 · gt_10k 0.8317). The what-if scoring remains
exact. Worst shipped `order()` measured at **0.896 s** (`arki0016`), below the
1.019 s carried by the revision that already passed the grader.

Note the budget costs some of the headline wins: `chp_shorttermplan2d` gets 5
restarts rather than 24, so it lands at 0.7280, not 0.5355. That is the trade the
sweep priced.

## Correction to earlier pages — timing numbers are ±1.6×

Two runs of the same probe on the same code, hours apart, disagreed badly:

| | run A | run B |
|---|---|---|
| worst overall | 1.019 s (`crudeoil_lee4_10`) | 0.803 s (`arki0016`) |
| `crudeoil_lee4_10` | 1.019 s | 0.646 s |

**The local worst case is known to about one significant figure.**
[0002](0002-measured-gates-metis-kahip.md) and the module header both stated
1.019 s as though it were precise, and the header additionally claimed the grader
is "~3-5× slower than local, so worst-case LOCAL time must stay well under
~0.35 s". That claim cannot be right: the revision carrying a 1.019 s local worst
**passed** the grader, which is impossible at 3-5× against a 2 s SIGKILL. We have
no calibration of grader speed.

The defensible rule is comparative, not absolute: **keep the worst local
`order()` at or below that of a revision already known to have passed.** Both the
module header and this page now state it that way.

## What to try next

See [open-questions](../open-questions.md). The obvious follow-on: restarts are
currently uniform-random relabelings, which is the dumbest possible `Q`. A
*structured* `Q` (e.g. seeding from RCM, or from a partitioner's block order)
might beat random at the same cost, and the budget machinery is already in place
to price it.
