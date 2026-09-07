# 0096 — Hub-free relabelled lotteries (SqPure@5, DegDivNvSqrtWf@10, SqDiv@5) plus chained MINL

- **Date:** 2026-09-07
- **Score:** baseline 7f5a20d (82d8d6e, hidden 0.85171) dev 0.808139 → **0.808011** (−1.28 bips), fill →0.931233
- **Status:** win (local full 300-matrix run, 71 tests pass, worst order() 0.620 s vs 0.689 s base)

## Hypothesis

Two leftovers, both shaped by measured failures: (1) the 14-family escalation died
mid-corpus and the only measured per-pass cliff in this walk class sits on hub rows, so
three new relabelled lotteries — SqPure@5 (the heavy block's measured cm_sqpure@5 winner,
relabelled), DegDivNvSqrtWf@10 (plain-shipped variant, relabelled), SqDiv@5 (mid-α for the
most general variant) — run hub-free gated (`max_deg*50<=n`; hubs keep the seven shipped
lotteries), same 120k/nnz cap-6 budget, disjoint 40k streams, post-cascade slot; (2) the
40M-op MINL budget can break a scan early, so a fresh budget continues the descent only
after a strict win (≤3 links). Rejected: AMF-on-`M` realization (+0.00).

## What changed

`src/ordering/mod.rs` only: hub-free 3-lottery block after the seven shipped relabelled
families; MINL call site wrapped in win-conditioned ≤3 chain. No ledger raised, no gate
widened, no large-row tickets.

## Result

| | 7f5a20d baseline | candidate |
|---|---|---|
| weighted | 0.808139 | **0.808011** (−1.28 bips) |
| lt_1k | 0.8896 | 0.8896 (control) |
| 1k_10k | 0.8479 | 0.8470 |
| gt_10k | 0.7173 | 0.7176 (slight redistribution, −0.17 bips of cost) |

71 tests pass; worst 0.620 s (improved on noise; +18 same-class passes inside the measured
light envelope).

## Follow-ups

- Per-lottery subset attribution if rejected for margin (SqPure@5 vs DegDivNvSqrtWf@10 vs
  SqDiv@5) rather than more families.
- Condition minfill-core on disagreement (193/300 rows pay today).

## Links

- Experiments: [0095](0095-terminal-completion-lattice-descent.md), [0093](0093-relabelled-metric-multistart.md), [0005](0005-relabelled-amf-multistart.md)
