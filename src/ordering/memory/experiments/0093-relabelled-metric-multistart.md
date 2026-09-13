# 0093 — Relabelled quotient-metric multi-start (the lottery 0092 never built)

- **Date:** 2026-09-07
- **Score:** baseline 017a036 (b229fd6, hidden 0.85328) dev 0.813331 → **0.812247** (−10.84 bips), fill 0.933055→0.932372
- **Status:** win (local full 300-matrix run, 71 tests pass, worst order() 0.484 s, unchanged)

## Hypothesis

0092 relabels only AMD/AMF, but the relabel trick is general (0005): any walk that reads
the vertex numbering becomes a randomized-restart algorithm under `relabel`. The ported
`order_variant` walks are the same quotient-graph class over the same primitives
(hash-bucket insertion order), so `METRIC(Q A Qᵀ)` composed back is a new
minimum-METRIC lottery family the winner never built. One variant (DegSqrt α1) first for
clean attribution, then the two most distinct shipped objectives (DegP075, SqDiv at α10).

## What changed

`src/ordering/mod.rs` only, one block after the light-tier metric family (hence after the
partitioner cascade, so cascade gates still see crown values):

- Three variant lotteries (DegSqrt α1; DegP075 α10; SqDiv α10), each
  `RELABEL_METRIC_BUDGET/nnz` passes (120k, cap 6), gate `nnz < 130k` (the measured-safe
  light envelope), fixed seeds 30000/31000/32000 + r, `consider!` queued (parallel-safe,
  byte-identical replay), best-of floor.

## Result

| | 017a036 baseline | candidate |
|---|---|---|
| weighted | 0.813331 | **0.812247** (−10.84 bips) |
| lt_1k | 0.889572 | 0.889572 |
| 1k_10k | 0.856058 | 0.854293 |
| gt_10k | 0.724105 | 0.722718 |

DegSqrt alone: +0.20 bips; +DegP075/SqDiv → +10.84 (heavy-tailed: new lotteries move
1k_10k/gt_10k rows nothing else reaches). Rejected before this: light α5 for all ten
variants (+0.00, reverted — same draws, not new lotteries); dense-130k–200k admission
(+0.00, reverted).

71 tests pass; worst 0.484 s vs 0.483 s pre-change on this host (10–18 extra same-class
passes, per-pass cost already measured safe; winner's pod worst 1.208 s leaves margin).

## Why it won

New lotteries, not more draws: α5 repeats the same distribution (nothing new to win),
while relabelled metrics draw from three distributions absent from the portfolio. Same
structural pattern as 0092 itself (introduction with measured envelope + cascade-safe
queue position), which is why it transfers by construction rather than by dev luck —
though only grading decides.

## Follow-ups

- More variant lotteries (DegP125, DegDivNvWfP15 at shipped α) if this promotes; each is
  ~6 same-class passes.
- Relabelled `metric_sweep` specs on the heavy tier (none relabelled today).

## Links

- Experiments: [0092](0092-ported-heavy-tier-metrics-parallel-portfolio.md), [0005](0005-relabelled-amf-multistart.md)
