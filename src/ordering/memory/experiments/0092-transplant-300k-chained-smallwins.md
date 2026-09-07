# 0092 — Transplant 300k with chained passes plus small ledger-bounded wins

- **Date:** 2026-09-07
- **Score:** baseline 081af15 (d61c645, hidden 0.857182) dev 0.823737 → **0.823446** (−2.91 bips), fill 0.936733→0.936627
- **Status:** win (local full 300-matrix run, 69 tests pass, worst order() 0.714 s)

## Hypothesis

The promoted 250k transplant (100k→250k, gate 4*→3*) moved `gt_10k` for the first time.
A further +20% ledger (250k→300k, same 3* gate) should let more donor draws complete and
admit a few medium-large-sparse rows (unit 83k–100k) whose cost previously exceeded the
ledger, while still refusing the densest/largest rows that stress the 2 s cap. Chained
second/third transplant passes spend only where the previous pass strictly won (different
elimination tree → different blocks), so they are substitutive, not unconditional.
Tiny-graph extras (MinFill restarts, small exact-search streams, small-core alphas,
alternate-chain rounds) are ledger- or gate-bounded to rows far from the tail.

## What changed

`src/ordering/mod.rs` + `src/ordering/transplant_probe.rs` only:

- `TRANSPLANT_LEDGER` 250000→**300000** (same `3*unit` admission gate, same below-AMD gate,
  same widths `[4096,512,128,32]`, same reservation policy).
- Chained transplant: if first `refine_with_donors` strictly improves, run again on the new
  incumbent (same donors/ledger); if second wins, run a third time. Only winners pay extra.
- Tiny MinFill restarts 24→32 (`n<=1000 && nnz<=5000`) and 6→8 (`n<2000 && nnz<10000`).
- Small exact-search streams +1 each for `n<=1000` (well_below 6→7, else 5→6, new fixed seeds
  `0xC0FF...`/`0xDEAD...`).
- Small-core relabel alphas `[2.5,10.0,0.5,5.0]`→`+1.0,16.0` (K=3, n 1000..10000, nnz≤50k,
  cn 8..4000, core_nnz≤30k, terminal candidate, strict accept).
- Alternate-seed rounds 8→12 under the unchanged shared 4M `PEO_ALT_LEDGER` (breaks on ledger,
  so worst-case time unchanged).

All gates structural (`n,nnz,cn,core_nnz,amd_flops,incumbent_flops`), fixed seeds, no
clock/env/fs, every acceptance strict `<` on exact `Σc²`.

## Result

Full 300-matrix trusted run:

| | 081af15 baseline | candidate |
|---|---|---|
| weighted flop geomean | 0.823737 | **0.823446** (−2.91 bips) |
| fill | 0.936733 | 0.936627 |
| lt_1k (147) | 0.889664 | 0.889596 |
| 1k_10k (108) | 0.856255 | 0.856174 |
| gt_10k (45) | 0.749902 | **0.749287** |

Ablation on this base: minfill32-8 alone +0.05; +smallcore6alpha +0.00; +alt-rounds12 +0.00;
+chained-2nd +0.12 (→0.82372); +smallstreams +0.20 (→0.82370); +3rd pass +0.02 (→0.823698);
+ledger300k →**0.823446** (+2.5 bips of the total). Seeds 8→12 alone −0.11 (runner_up
reshuffle), donor-12 −0.20, medium-wellbelow+1 −0.03, CN6k +0.00 — all reverted.

69 tests pass. `probe_timing_and_score` worst **0.714 s** on this host.

## Why it won

250k→300k is the measured knee past the promoted package: more donor draws complete before
reservation truncates, and rows with unit 83k–100k newly afford entry. The win concentrates
in `1k_10k`/`gt_10k` (transplant territory), while tiny-graph extras move `lt_1k`/`1k_10k`
by small exact-search lottery draws. Chained passes work because the improved incumbent has
a different postorder tree, exposing new block boundaries to the same donors — strictly
monotonic and nearly free (few winners).

## Follow-ups

- Do not raise transplant further without an isolated worst-case re-measure; 500k+ approaches
  the historical timeout class (per d61c645 note).
- Condition minfill-core on portfolio disagreement (0091 exposure: 193/300 rows pay for
  6 winners).

## Links

- Experiments: [0090](0090-transplant-verification-reservation-screen.md), [0091](0091-residual-core-exact-minimum-fill.md), [0084](0084-alternate-seed-chains-shipped.md)
