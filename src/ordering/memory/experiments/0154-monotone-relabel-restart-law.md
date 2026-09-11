# 0154 — Re-derive the relabelled-multi-start restart ladder as monotone laws

- **Date:** 2026-09-11
- **Score:** 0.792439 → **0.792573** (−1.69 relative bip, i.e. a small loss;
  all of it in `1k_10k`)
- **Status:** compliance change, measured loss on dev, submitted as a
  single-variable probe

## Hypothesis

`relabel_restarts_tuned` decides how many relabelled AMD restarts a pattern
gets. It was a ladder of five branches, and three of them selected a
**two-sided window** rather than a half-space:

1. `max_deg * 50 > n && (100_000..=150_000).contains(&nnz)` → cap at 4. The
   hub test is structural; the `nnz` band is not. Its own comment named the
   single dev pattern it was drawn around.
2. inside the sparse `gt_10k` floor, `n >= 40_000 && nnz <= 200_000` → floor 4
   instead of 8 — a rectangle carved out of a population that the surrounding
   branch already describes structurally.
3. `(500_000..=1_500_000).contains(&nnz)` on the sparse hub-free giants — a
   band where only the upper end is a cost bound.

A window fitted to dev fires on an unseen corpus at rows chosen at random with
respect to the property that motivated it. `RULES.md` asks for `n` / `nnz` /
degree gates, and a gate is a half-space; a band is a population.

## What changed

`src/ordering/mod.rs`, `relabel_restarts_tuned` only.

- The hub band becomes a **hub law applied at every size**: a hub graph
  (`max_deg * 50 > n`) gets its own, smaller work budget,
  `max(RELABEL_HUB_MIN, RELABEL_HUB_WORK / nnz)` with `RELABEL_HUB_WORK =
  400_000` and `RELABEL_HUB_MIN = 4`, as a **cap** on whatever the rest of the
  function returned. Monotone non-increasing in `nnz`.
- The `n >= 40_000 && nnz <= 200_000` rectangle is deleted; that branch now
  returns the lower of its two floors, `base_r.max(4)`, everywhere.
- The giants' `500_000 <=` lower bound is deleted, leaving
  `nnz <= RELABEL_GIANT_MAX_NNZ` as an ordinary cost bound.

The two remaining floors (`nnz <= 20_000`, and `nnz <= 150_000` on hub-free
graphs) are already half-spaces and are untouched. The restart law is otherwise
the same budget-over-`nnz` it always was.

## Result

| bucket | count | flop geomean | base |
|---|---|---|---|
| `lt_1k` | 147 | 0.887516 | 0.887516 |
| `1k_10k` | 108 | **0.840192** | 0.839747 |
| `gt_10k` | 45 | 0.685651 | 0.685651 |
| **weighted** | 300 | **0.792573** | 0.792439 |

Fill tiebreak 0.924518.

**`lt_1k` and `gt_10k` are bit-identical to the base.** The entire 1.69 bip is
`1k_10k`, and it is the hub law: 22 dev rows get a lower restart count, and the
ones that matter are `1k_10k` hub patterns in the 9k–18k `nnz` range whose
restart count falls from the 33–48 the old low-`nnz` branch handed them
(`batchs121208m` 40 → 26, `crudeoil_pooling_ct3` 42 → 28, `popdynm25` 35 → 23,
`nuclear25a` 33 → 22, `chimera_mgw-c16-2031-01` 33 → 22, `oil` 48 → 34).

## Why it lost on dev

Exactly the trade the windows bought: they were swept until the dev rows inside
them netted a gain, so deleting them gives that dev value back. What replaces
them is a statement about structure — *a hub graph's relabelled pass costs more
than `nnz` predicts, because dense absorption lets the quotient degrees grow
with the largest degree rather than the average one, so a hub graph gets a
smaller work budget* — which is testable on any corpus. "`nnz` is between
100 000 and 150 000" is not.

## The change cannot cost time, and this is checked rather than timed

`relabel_restarts_tuned` is a pure function of `(n, nnz, max_deg)` — its
`budget` and `cap` arguments are a pure function of `n` — so "does the new law
ever ask for more restarts than the ladder?" is decidable by enumeration rather
than by measurement. Modelling both over the dev corpus and over a grid of
6 415 admissible triples spanning every branch boundary of both functions, with
`max_deg <= nnz <= n * max_deg` enforced:

| | triples | total restarts | changed | **new > old** |
|---|---:|---:|---:|---:|
| admissible grid | 6 415 | 95 129 → 90 285 | 657 | **0** |
| dev corpus | 300 | 10 633 → 10 468 | 22 | **0** |

So the arm is a work *reduction* on every realisable pattern, not only on the
ones dev happens to contain. No timing run can add to that, and none is
reported here.

## Follow-ups

- `RELABEL_HUB_WORK = 400_000` is the largest constant for which the hub law
  still dominates the old band at its own boundary (`400_000 / 100_000 = 4`).
  A smaller one narrows further and costs more dev score; it is a free
  parameter of the law, not of a population.
- The two surviving floors are half-spaces, but their constants (`20_000`,
  `150_000`, `600_000`, `12`) are still swept-on-dev numbers. Re-deriving them
  as one continuous law is the next step and is a much larger change: any single
  hyperbola that stays below the flat mid-band floor cuts mid-`nnz` restarts by
  half or more.

## Links

- [0150](0150-stage1b-window-removal-isolated.md) — the same kind of removal at
  the stage-1b adoption site.
- [0011](0011-hub-gate-and-floors.md), [0018](0018-large-sparse-gt10k-restart-floor.md)
  — where the ladder's branches came from.
