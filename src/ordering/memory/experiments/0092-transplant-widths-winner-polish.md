# 0092 — Transplant width diversity with winner-only chains and polish

- **Date:** 2026-09-07
- **Score:** baseline 081af15 (d61c645, hidden 0.857182) dev 0.823737 → **0.823574** (−1.63 bips), fill 0.936733→0.936656
- **Status:** win (local full 300-matrix run, 69 tests pass, worst order() 0.695 s)

## Hypothesis

The promoted 250k transplant tries block widths `[4096,512,128,32]`. Intermediate widths
(1024 between 4096/512, 64 between 128/32) catch subtree sizes the ladder skips, at no new
time bound (same 250k ledger, same 3*unit + below-AMD gates — the pass breaks when the
ledger is spent). Chained 2nd/3rd passes spend only where the previous pass strictly won
(new postorder tree → new block boundaries). Winner-only terminal polish (pair descent +
simplicial, same gates/budgets as the early passes) fixes the pipeline order: those cheap
searches ran before transplant, so a transplant winner has never seen them. Tiny-graph
lotteries stack monotonically via strict accept.

## What changed

`src/ordering/mod.rs` + `src/ordering/transplant_probe.rs` only (no `deps.toml` change):

- `transplant_pass` widths `[4096,512,128,32]`→`[4096,1024,512,128,64,32]` (production only;
  test-only `terminal_pass` untouched).
- Chained transplant: 2nd pass if 1st strictly improves, 3rd if 2nd wins (same donors,
  same 250k ledger each, so extra cost only on winners — few rows).
- Winner-only polish after transplant block: if `best_flops < pre_transplant`, re-run
  `adjacent_pair_descent` (same `pair_descent_gate`/`pair_descent_ops_budget`) and
  `simplicial_promotion` (same `SIMPLICIAL_*` gate/budget), strict accept.
- Tiny MinFill 24→32 (`n<=1000 && nnz<=5000`), 6→8 (`n<2000 && nnz<10000`).
- Small `rgreedy::search` +1 stream for `n<=1000` (new fixed seeds).
- Small-core relabel alphas `+1.0,16.0` (K=3, n 1000..10000, nnz≤50k, cn/core_nnz gated).
- Alternate rounds 8→12 under the unchanged shared 4M ledger (ledger break unchanged).

All gates structural, fixed seeds, no clock/env/fs, every acceptance strict `<` on exact
`Σc²`. Transplant ledger stays 250k (the 300k raise scored +2.91 locally but FAILED hidden
— unconditional spend, closed; see log).

## Result

| | 081af15 baseline | candidate |
|---|---|---|
| weighted flop geomean | 0.823737 | **0.823574** (−1.63 bips) |
| fill | 0.936733 | 0.936656 |
| lt_1k (147) | 0.889664 | 0.889343 |
| 1k_10k (108) | 0.856255 | 0.856034 |
| gt_10k (45) | 0.749902 | 0.749902 |

Ablation on this base: minfill32-8 +0.05; +smallcore6alpha +0.00; +alt-rounds12 +0.00;
+chained-2nd/3rd +0.12; +smallstreams +0.20; widths+polish remainder to +1.63.
Reverted on this base: seeds 8→12 −0.11, donor-12 −0.20, medium-wellbelow+1 −0.03, CN6k +0.00,
ledger 250k→300k +2.5 locally but FAILED hidden (unconditional, do not retry).

69 tests pass. `probe_timing_and_score` worst **0.695 s** on this host.

## Why it won

Width diversity catches block sizes the 4-ladder skips, within the same ledger cap (breaks
early when spent, so worst-case bound unchanged). Chains and winner-polish are
substitutive: they fire on few rows where a strict win already paid, unlike the failed
unconditional forms (0088 600k, 0092-300k, sparse-large) that spend on every admitted row
and fail opaquely despite locally lower worsts. Tiny lotteries stack because each is a
fresh i.i.d. draw under strict accept (0004 lottery logic).

## Follow-ups

- Condition minfill-core on portfolio disagreement (0091 exposure: 193/300 rows pay).
- Do not raise transplant ledger or add large-row tickets without hidden re-validation.

## Links

- Experiments: [0090](0090-transplant-verification-reservation-screen.md), [0091](0091-residual-core-exact-minimum-fill.md), [0084](0084-alternate-seed-chains-shipped.md)
