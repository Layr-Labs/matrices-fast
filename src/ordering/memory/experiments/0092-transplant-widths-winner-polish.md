# 0092 — Transplant widths with winner-only polish and tiny lotteries

- **Date:** 2026-09-07
- **Score:** baseline 081af15 (d61c645, hidden 0.857182) dev 0.823737 → **0.823582** (−1.55 bips), fill 0.936733→0.936656
- **Status:** win (local full 300-matrix run, 69 tests pass, worst order() 0.697 s)

## Hypothesis

Intermediate transplant widths (1024, 64) catch block sizes the 4-ladder skips under the
same 250k ledger/gates (bound unchanged). Winner-only polish fixes pipeline order: pair
descent + simplicial ran before transplant, so a transplant winner has never seen them;
re-running only on strict transplant winners is nearly free and strictly monotonic. Tiny
lotteries stack via strict accept off the tail. Chained extra transplant ledgers and any
ledger raise are excluded: both failed hidden opaquely in this session despite local wins
(300k +2.91, sparse-large +1.51, chains included) — unconditional or winner-multiplied
exact scores undercharge fill (`n+nnz` ledger vs `Lnnz`-driven cost).

## What changed

`src/ordering/mod.rs` + `src/ordering/transplant_probe.rs` only:

- `transplant_pass` widths `[4096,512,128,32]`→`[4096,1024,512,128,64,32]` (production only).
- Winner-only polish: capture `pre_transplant`; if `best_flops < pre_transplant`, re-run
  `adjacent_pair_descent` (same gate/budget) and `simplicial_promotion` (same gate/budget),
  strict accept.
- Tiny MinFill 24→32 / 6→8; small search +1 stream (fixed seeds); small-core alphas +1.0/16.0;
  alternate rounds 8→12 (same 4M ledger).

No ledger raise (stays 250k), no transplant chains, no large-row tickets. All gates
structural, fixed seeds, strict `<`.

## Result

| | base 081af15 | candidate |
|---|---|---|
| weighted | 0.823737 | **0.823582** (−1.55) |
| lt_1k | 0.889664 | 0.889361 |
| 1k_10k | 0.856255 | 0.856043 |
| gt_10k | 0.749902 | 0.749902 |

Widths+tiny without polish: 0.823646 (+0.91); +polish →0.823582 (+1.55), so polish +0.64.
Reverted here: transplant chains (extra 250k ledgers on winners, suspect for dense hidden
winners), ledger-300k (unconditional, failed hidden), sparse-large (failed hidden),
seeds/donor-12, medium+1, CN6k.

69 tests pass; worst 0.697 s on this host.

## Follow-ups

- Condition minfill-core on disagreement (193/300 rows pay today).
- Never raise transplant ledger or multiply exact-score ledgers on winners without hidden
  re-validation; the `n+nnz` unit undercharges fill.

## Links

- Experiments: [0090](0090-transplant-verification-reservation-screen.md), [0084](0084-alternate-seed-chains-shipped.md)
