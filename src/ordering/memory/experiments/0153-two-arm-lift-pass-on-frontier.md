# 0153 — The deferred independent-set lift gets its own pipeline pass

- **Date:** 2026-09-11
- **Score:** 0.792439 → **0.792354** (−1.07 relative bip better; 4 rows better,
  **0 worse**)
- **Status:** win, submitted

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

Dev, 300 patterns, against the promoted frontier `ab30c0e`:

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

## Why it wins

Zero dev losers is structural, not tuned: the second pass only *adds* a
candidate and the winner is chosen on the exact final objective, so a row can
never end worse than it would have. The two rows that move materially are
exactly the shape the stage-1b comment predicts — a lift whose raw score trails
the portfolio incumbent but whose basin the downstream stages polish much
further.

## The ceiling, and what is behind the gate

The gate is a **cost** gate, not a correctness one. Ungated, the comparison
reaches **2.796 relative dev bip over 10 movers with zero dev losers**; the
shipped `n <= 600 && nnz <= 30_000` captures 1.07 of it. The remaining 1.724
bip needs a second pass that is affordable on larger rows — i.e. a deterministic
work bound on the pass, not a size window.

## Links

- [0150](0150-stage1b-window-removal-isolated.md) — the windows this replaces in
  principle, and the measurement that says removing them paid.
- [0151](0151-best-flops-writeback-isolated.md) — the invariant this pass
  depends on, since the pass now *returns* `best_flops` as its score.
