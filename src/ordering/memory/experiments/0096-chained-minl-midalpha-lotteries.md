# 0096 — Chained MINL descent plus relabelled mid-α metric lotteries

- **Date:** 2026-09-07
- **Score:** baseline d74ab4b (09bd1f3, hidden 0.852519) dev 0.808268 → **0.807908** (−3.60 bips), fill 0.9311→0.9310
- **Status:** win (local full 300-matrix run, 71 tests pass, worst order() 0.637 s vs 0.622 s base)

## Hypothesis

Two untouched edges of the newest stage and family, both substitutive/new-lottery shaped:
(1) the 40M-op MINL budget can break a cheapest-first scan early, leaving the completion
non-minimal — a fresh budget continues the same descent, but only after a strict win, so
non-winners pay exactly the single-descent cost; (2) the 0093 relabelled lotteries draw at
α1/α10 while the plain grid just showed mid-α to be load-bearing — mid-α relabelled draws
are new lotteries at the same per-ticket cost. Rejected first: AMF realization of `M`
(+0.00 — MCS+AMD already cover it).

## What changed

`src/ordering/mod.rs` + `src/ordering/minl.rs` untouched (no — `mod.rs` only; `minl.rs`
reverted):

- MINL call site: up to 3 descents, each after the previous strictly won (fresh 40M budget
  continues a budget-broken scan; a minimal `M` returns `None` after one fruitless scan).
- Relabelled block: each of the three variant lotteries drawn at shipped α and at mid
  α5.0 on a disjoint fixed stream (shipped 30–32k streams bit-preserved), same 120k/nnz
  cap-6 budget, same `nnz<130k` post-cascade slot.

## Result

| | d74ab4b baseline | candidate |
|---|---|---|
| weighted | 0.808268 | **0.807908** (−3.60 bips) |
| lt_1k | 0.8896 | 0.8896 (control) |
| 1k_10k | 0.8472 | 0.8470 |
| gt_10k | 0.7181 | 0.7174 |

Chained descent alone: +0.11 (nearly everything is already minimal — kept, nearly free).
+ mid-α relabelled: remainder. AMF-on-`M` control +0.00, reverted.

71 tests pass; worst 0.637 s (+15 ms vs base on this host; winner's pod worst 1.523 s
leaves margin; no ledger raised, no gate widened).

## Follow-ups

- Relabelled α2.5 lotteries if this promotes (same form, third α rung).
- Condition minfill-core on disagreement (193/300 rows pay today).

## Links

- Experiments: [0095](0095-terminal-completion-lattice-descent.md), [0093](0093-relabelled-metric-multistart.md), [0005](0005-relabelled-amf-multistart.md)
