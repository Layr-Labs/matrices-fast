# 0064 - Extra ranked-subtree round 5

- **Date:** 2026-09-05
- **Base:** 0063 time-margin package, dev score 0.839063
- **Score:** 0.839063 -> **0.838890** (-0.000173, 1.73 bips), fill 0.942817 -> **0.942755**
- **Status:** WIN locally; hidden validation pending

## Hypothesis
The completed shipped pipeline can still expose useful local minima on its final
elimination tree. A fifth diversified subtree search round may find them without
changing any earlier candidate gates or search basins.

## What changed
`src/ordering/mod.rs` adds one final strict-best ranked-subtree pass after the
existing reduce-then-AMF terminal phase. It matches the strongest measured probe
variant: `round=5`, four ranked blocks, `min_s=16`, `max_s=768`, `max_sub=1200`,
and an 8M operation budget, gated to `1,000 <= n <= 80,000` and `nnz <= 250,000`.

## Result
The full 300-matrix trusted run passes. The bucket flop geomeans are:

- `lt_1k`: 0.890341 -> 0.890341
- `1k_10k`: 0.868720 -> 0.868271
- `gt_10k`: 0.778362 -> 0.778266

The pass improves 34 matrices and worsens none. The fill tiebreak improves to
0.942755. The timing probe reports a worst local `order()` call of 0.947 s,
with `arki0013` as the slowest row. All 50 active tests and the synthetic probe
pass.

## Why it won
The extra round uses a distinct deterministic search seed schedule on the final
incumbent tree. Increasing the four-block budget finds additional strict local
improvements on the medium and sparse-large eligible rows; the AMD best-of floor
and strict admission prevent regressions. The probe ladder showed diminishing but
monotonic gains from 2M through 8M, with 8M the strongest tested setting.

## Follow-ups
- Validate the gain on an independent corpus or hidden submission before widening
  the gate or budget.
- Measure selective gates and lower budgets if hidden timing leaves less margin
  than the local 0.947 s worst case.

## Links
- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md)
- Prior timing constraints: [0063](0063-time-margin-by-structure.md)
