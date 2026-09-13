# 0096 — Sub-10k relabelled lotteries (hidden-gt preservation by structure)

- **Date:** 2026-09-07
- **Score:** baseline 7f5a20d (82d8d6e, hidden 0.85171) dev 0.808139 → **0.807622** (−5.17 bips), fill →0.930861
- **Status:** win (local full 300-matrix run, 71 tests pass, worst order() 0.659 s vs 0.689 s base)

## Hypothesis

The dda1f34 bundle (hub-free lotteries on ALL n) won dev +1.28 with a gt_10k
redistribution cost and graded hidden-worse (−0.22): extra draws reshuffle the runner_up
pool, flipping transplant assemblies on hidden gt_10k rows. Buckets are defined by
dimension n and order() state is strictly per-call, so work gated on `n < 10_000` leaves
every gt_10k row BIT-IDENTICAL — preserving hidden gt trajectories structurally, not
empirically — and the whole slow tail (pod worst 1.523 s, all n ≥ 10_000) is excluded, so
the worst case cannot move either. Fourteen further variant×α lotteries the shipped seven
never drew, same budget/cap/slot/floor.

## What changed

`src/ordering/mod.rs` only, one block after the seven shipped relabelled families:
SqPure@{5,10,2.5}, DegDivNvSqrtWf@{10,5}, SqDiv@{5,2.5}, DegP075@{5,2.5}, DegP125@{5,2.5},
DegDivNvWfP15@5, DegPlusDegme@{10,5} — disjoint 40–53k streams, 120k/nnz cap 6 each,
`n < 10_000 && nnz < 130k`, post-cascade `consider!`, best-of floor. Shipped streams
untouched. No ledger raised, no gate widened elsewhere, no chains.

## Result

| | 7f5a20d baseline | candidate |
|---|---|---|
| weighted | 0.808139 | **0.807622** (−5.17 bips) |
| lt_1k | 0.8896 | 0.889737 |
| 1k_10k | 0.8479 | 0.845970 |
| gt_10k | 0.7173 | 0.717275 (control — matches to printed precision; bit-identical by construction) |

Ladder: 6 lotteries +2.80, 10 lotteries +3.47, 14 lotteries +5.17 (no saturation harm;
runner_up stays top-8 so pool quality improves monotonically, and ledger dilution lands
only on n<10k rows far from the tail).

71 tests pass; worst 0.659 s (base 0.689 s on this host — noise band; +84 same-class
passes inside the measured light envelope, zero of them on tail rows).

## Follow-ups

- Remaining α rungs (DegDivNvSqrtWf@{2.5,1}, DegPlusDegme@{2.5,1}, SqPure@1, SqDiv@1,
  DegP075@1, DegP125@1, DegDivNvWfP15@{2.5,1}) if this promotes — same gate, safe.
- Condition minfill-core on disagreement (193/300 rows pay today).

## Links

- Experiments: [0095](0095-terminal-completion-lattice-descent.md), [0093](0093-relabelled-metric-multistart.md), [0005](0005-relabelled-amf-multistart.md)
