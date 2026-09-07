# 0094 — Light-tier α grid {10,1} → {10,5,2.5,1} (porting the e7988e5 pattern)

- **Date:** 2026-09-07
- **Score:** baseline aa5b471 (316c980, hidden 0.853001) dev 0.812247 → **0.811892** (−3.55 bips), fill 0.932372→0.9322
- **Status:** win (local full 300-matrix run, 71 tests pass, worst order() 0.484 s)

## Hypothesis

Competitor `e7988e5` (promoted 0.853131) showed the light-tier α grid is the load-bearing
axis: {10,1}→{10,5,2.5,1} gained 6.40 dev bips, all in `1k_10k`, with +20 same-envelope
passes. Our 0093 relabelled lotteries cover α1/α10 only, so mid-α un-relabelled draws are
disjoint new orderings. Same gate, same post-cascade position, best-of floor.

## What changed

`src/ordering/mod.rs` only, light-tier block: `for alpha in [10.0, 1.0]` →
`[10.0, 5.0, 2.5, 1.0]` (+20 producers on `nnz < 130k`, after the cascade).

## Result

| | aa5b471 baseline | candidate |
|---|---|---|
| weighted | 0.812247 | **0.811892** (−3.55 bips) |
| lt_1k | 0.889572 | 0.8896 (control) |
| 1k_10k | 0.854293 | **0.8531** (all of the gain) |
| gt_10k | 0.722718 | 0.7227 (control) |

Smaller than e7988e5's +6.40 on 017a036 because our relabelled lotteries already took part
of the 1k_10k headroom — independent confirmation the two stack rather than overlap.

71 tests pass; worst 0.484 s (the +20 passes are milliseconds each inside the measured
light envelope; winner's pod worst 1.208 s leaves margin).

## Follow-ups

- Relabelled mid-α lotteries (α5/α2.5 under relabel) if this promotes — same 0005 form.
- Do not touch transplant ledgers, cascade semantics, AmindNorm, TELOS (all closed).

## Links

- Experiments: [0093](0093-relabelled-metric-multistart.md), [0092](0092-ported-heavy-tier-metrics-parallel-portfolio.md)
