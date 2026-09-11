# 0156 — Deepen `2.descent`, the second-highest-rate stage

- **Date:** 2026-09-11
- **Score:** 0.792573 → **0.792612** (−0.49 relative bip, worse)
- **Status:** loss, reverted

## Hypothesis

The terminal adjacent-pair descent is a strictly monotone local search on the
exact objective: it swaps adjacent pairs in `best_perm` and accepts only a
strict flop decrease. Per the stage census it is the second-highest-value stage
per second in the pipeline and it costs **0.56 s over the whole 300-pattern
corpus** — 72 rows fire, worst single row 0.032 s. Its depth knobs are explicit
(`PAIR_DESCENT_SWEEPS`, two ops budgets), and doubling them is affordable
several times over against a 2 s per-matrix cap. If a high value-per-second
stage has any marginal return left, this is the cheapest place in the tree to
collect it.

## What changed

`src/ordering/mod.rs`, three constants:

- `PAIR_DESCENT_SWEEPS` 4 → 8
- `PAIR_DESCENT_OPS_BUDGET` 128 M → 256 M
- `PAIR_DESCENT_EXT_OPS_BUDGET` 48 M → 96 M

No gate is touched, so the admitted population is identical and only the depth
changes.

## Result

| bucket | base | deepened |
|---|---|---|
| `lt_1k` | 0.887516 | **0.887644** |
| `1k_10k` | 0.840192 | **0.840224** |
| `gt_10k` | 0.685651 | **0.685628** |
| **weighted** | 0.792573 | **0.792612** |

`lt_1k`, where almost all of the descent's work is, gets **1.44 relative bip
worse**. `gt_10k` improves by 0.34. Net **−0.49 bip**.

## Why it lost

**A deeper descent produces a strictly better stage-2 incumbent and a worse
final ordering.** The descent's acceptance test is exact and monotone, so the
permutation leaving stage 2 cannot be worse than before. What follows it is
thirteen further stages whose gain — and whose cost — depend on which basin they
are handed. A better local optimum at stage 2 is a *deeper* local optimum, and
the downstream searches have less room to move out of it.

This is the same mechanism as the stage-13 result
([0152](0152-stage13-retirement-isolated.md)), read in the opposite direction:
there, deleting a low-yield stage made four rows worse; here, improving a
high-yield stage makes the corpus worse. Together they say something sharper
than "stages interact":

> **Improving an intermediate incumbent is not weakly good in this pipeline.**
> The only reliable lever on the final objective is a comparison made *on the
> final objective* — run both candidates to the end and keep the winner.

That is what [0153](0153-two-arm-lift-pass-on-frontier.md) does, and it is why
that comparison wins on dev with zero losers by construction while every
stage-local deepening measured here loses.

## Follow-ups

- Do not re-try a depth knob anywhere in stages 1–13 without a whole-pipeline
  A/B; a stage-local before/after reading has the wrong sign too often to be
  worth taking.
- The corollary is a *cheap* direction: where two candidates already exist,
  scoring both at the end is worth more than polishing either in the middle.

## Links

- [0155](0155-transplant-tie-window-and-portfolio-depth.md) — the same result for
  `1.portfolio`, the highest-rate stage.
- [0152](0152-stage13-retirement-isolated.md), [0153](0153-two-arm-lift-pass-on-frontier.md)
