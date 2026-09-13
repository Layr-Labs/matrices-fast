# 0219 — the stage-1b seed choice is a lottery: every cheap arbitration of it is priced and dead

- **Date:** 2026-09-12 (iter46)
- **Score:** production **0.791635** (reproduced exactly) → arbitration arms **0.792805**
  (+1.17e-3 = 14.8 dev bips worse); the per-row oracle between the two pure arms is **0.791387**
  (2.48e-4 better than production) and is shown to be unreachable at bounded cost.
- **Status:** measured, **nothing shipped, nothing submitted** — the only large dev lever measured
  this iteration is a large *negative*. Tree carries one additional test-only seam (`SSI_INDEP_ARB`,
  inert in production) and two new probe lines (`ARB`, `LIFT`).

## Hypothesis

The frontier's own hidden gain on this board lives in the stage-1b *adoption trajectory* (the
promoted step's whole 3.2e-4 came from removing force-adoption windows). The iter45 audit left the
largest prize on the board quantified but unclaimed: the per-row min over the two pure arms
(force-everywhere vs never-force) is 2.48e-4 below production. Lead 22 proposed capturing it by
"polishing the deferred lift with the same cascade and comparing like with like at 4b". The cheap
form of that is a *timing* change, not more work: compare the two seeds **after the cheap
pre-cascade polish** (2.descent + 3.search) and hand the cascade to the winner. Zero added work,
and monotone-safe (the pre-cascade incumbent is never better than the post-cascade one, so no
adoption 4b makes today is lost).

## What changed (test-only, shipped path untouched)

- `SSI_INDEP_ARB` (per-mille band, unset = production) + the arbitration site at the top of the
  subtree chain (`mod.rs`), consuming the held lift.
- `force_audit::note_arb` / `take_arb` and the probe's `ARB` line: the arbitration's own margin
  `lift / incumbent` per row.
- `indep_first::last_lift` + the probe's `LIFT` line: the *shape* of the lift `run` returns
  (`core_n`, `core_nnz`, `|X|`) — lead 23's observable, now measurable per row.

## Result

| arm | score | worst `order()` |
|---|---|---|
| P production (`FORCE=1`) | 0.791635 | 1.250 s |
| A1 gate + arbitration | **0.792805** | 1.288 s |
| A2 arbitration, no gate | **0.792805** | 1.319 s (row-for-row identical to A1) |

- A1/A2 differ from production on 10 rows: 8 gains (waterund14 .9765, methanol200 .9911,
  crudeoil_lee4_10 .9925, lee4_09 .9994, torsion50 .9983, glider400 .9998, graphpart 1.0000,
  wastewater05m1 1.0039 — the last is itself a loss) totalling −1.43e-4, against **two losses**:
  `mpbp_35` 1.2199 (+1.21e-3) and `chimera_lga-01` 1.0446 (+1.02e-4).
- All four cheap predictors fail to separate the 13 force-wins from the 6 defer-wins: the raw lead
  (`lift/raw portfolio`, 0.9122–0.9994 in both classes), the arbitration's own margin
  (`lift/cheap incumbent`), the portfolio headroom (`portfolio/AMD`), and the residual-core shape
  (`core_n/n` 0.52–0.84, core fill 4.67–19.99 in both classes).
- `mpbp_35` carries the mechanism: its early lead is the **largest of all 28 held rows** (m = 0.9221)
  and it is the worst flip. The cascade's own objective ranks the lift *better* (L\* 0.4013 vs
  P\* 0.4227) while the end result is 21.7 % worse (0.3918 vs 0.3212): the post-4b stages invert the
  ranking, so *any* criterion evaluated before them can pick wrongly, and the only correct ranking
  needs a second full trajectory on the other seed (~2× on the 39 site rows — cap-fatal).

## Consequences

1. Lead 22's cheap surrogate is **measured dead** (+1.17e-3); its expensive form (cascade *and*
   the late chain on both seeds) is the only thing that could rank the seeds correctly, and it is
   not affordable under the 2 s cap. Do not spend another iteration on seed-selection rules that
   are decided before the end of the pipeline.
2. Lead 23 (core-shaped gate) is **dead on dev**: the core shape does not separate the two classes
   on any column measured.
3. The `n >= 20 000` gate remains the best measured rule on this corpus (0.791635), and the dev
   oracle (0.791387) is *not* a design target — it is an upper bound no bounded device can reach.
4. The 20 site rows both arms leave bit-identical are the reason sweeping gates looks cheap: only
   19 of 39 site rows move at all, and 6 of those 19 are rows every predictor puts on the wrong
   side.
