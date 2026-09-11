# 0153 — The deferred independent-set lift gets its own pipeline pass

- **Date:** 2026-09-11
- **Score:** 0.792439 → **0.792354** (−1.07 relative bip; 4 rows better,
  **0 worse**) under an `n <= 600 && nnz <= 30_000` cost gate
- **Status:** dev win, **not adopted** — the gate bounds size, not cost, and
  leaves no margin against the 2 s cap

## Hypothesis

At stage 1b the independent-set lift either wins the 10 % immediate-adoption
margin or is *deferred* and compared against the incumbent after stage 4. That
comparison is unfair in a way the code's own comment admits: the incumbent has
had the full stage-4 chain and the lift has had nothing, so a lift that would
end ahead loses. Polishing the lift with one round of the stage-4 chain to make
the comparison "fair" was measured and **regressed the corpus by 21 bips** — one
round is not a proxy for what the incumbent has received.

The fair comparison is therefore the expensive one: **run the whole pipeline on
both candidates and pick the winner on the final score.**

## What changed

`leader_order` is split so that a pass can be run with an injected stage-1b
lift. When a deferred lift exists and the pattern is small enough to afford a
second pass, both arms run to completion and the better final score wins. The
cost gate is `n <= 600 && nnz <= 30_000`.

Support: `parallel.rs` grows an `indep_trace` channel (test-only) recording which
adoption branch each row took, and `probe.rs` grows the three `probe_indep_*`
instruments that call the pass directly with and without the lift.

## Result

Dev, 300 patterns:

| row | n | nnz | Δ ln(flops) |
|---|---:|---:|---:|
| `waterund14` | 333 | 2204 | **−2.38 %** |
| `pooling_digabel19` | 514 | 5340 | **−2.09 %** |
| `graphpart_3g-0244-0244` | 128 | 672 | −0.18 % |
| `wastewater05m1` | 98 | 536 | −0.04 % |

**4 movers, 0 worse.** Half the total gain survives dropping the largest mover.
Corpus `order()` time rises 145.95 s → 148.48 s (+1.7 %, of which ≈0.4 s is the
test-only `STALE_BEST_FLOPS` guard this tree also carries), worst row
1.315 → 1.309 s.

## Why it won / lost

The score half wins, and wins structurally rather than by tuning: the second
pass only *adds* a candidate and the winner is chosen on the exact final
objective, so a row can never end worse than it would have. The two rows that
move materially are exactly the shape the stage-1b comment predicts — a lift
whose raw score trails the portfolio incumbent but whose basin the downstream
stages polish much further.

**The cost half loses, and that is why the tree is not adopted.** Measured per
row, one pass against two, the second pass **doubles** every row it fires on —
**1.89×–2.07×, no exceptions** (`pooling_digabel19` 0.533 → 1.074 s,
`wastewater05m1` 0.353 → 0.731 s). A size gate cannot bound that:

- **Pipeline cost on small rows is nearly flat in `n`.** A 98-vertex,
  536-nonzero pattern costs 0.353 s against a 514-vertex one's 0.533 s, because
  the small-row budget is set by stage constants, not by the pattern. Widening
  or narrowing `n` moves the admitted population without moving the cost.
- **Density is inversely related to cost on small rows.** The densest in-gate
  dev rows are the fastest (`qspp_0_10_0_1_10_1`, n = 280, 105.8 nonzeros per
  row, 0.232 s); the slowest are sparse (`chimera_mgw-c8-439-onc8-001`, 6.9 per
  row, 0.577 s). A gate built to exclude "small but dense" rows is aimed at the
  wrong class.
- **The arithmetic that should have preceded the gate.** Dev's slowest in-gate
  row is 0.577 s and the multiplier is 2×, so the gated tree's own worst
  in-gate row is ~1.15 s of a 2 s budget on this box — before any slower box or
  any unseen row is considered. Ungated, the worst dev row reaches **2.474 s**,
  over the cap locally.
- **A deterministic meter counting exact scorings does not track time.**
  Seconds per million work units span 0.14 to 10.46 across the 23 dev rows with
  a deferred lift, because the meter misses `3.search` and `9.reduce`, which is
  where a small row's cost lives.

## Follow-ups

- The mechanism is right and none of it is shippable until the second pass is
  admitted on a **deterministic work count that actually bounds time**, not on
  a size window. That means counters inside the dominating kernels
  (`rgreedy::*`, `minl::*`, `core_lift::*`), screened against seconds before
  any gate is built on them — the `(n + nnz)`-per-scoring meter above is the
  screen that already failed.
- Aborting the second pass is free whenever the budget is exceeded, because
  pass 1 already holds a complete answer.

## Links

- [0150](0150-stage1b-window-removal-isolated.md) — the windows this replaces in
  principle.
- [0151](0151-best-flops-writeback-isolated.md) — the invariant this pass
  depends on, since the pass now *returns* `best_flops` as its score.
