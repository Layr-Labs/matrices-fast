# 0076 — Sixth AMF lottery objective (α = 0.5) in relabel sweeps

- **Date:** 2026-09-06
- **Score:** 0.827195 → 0.827213 (geomean vs AMD; lower is better)
- **Status:** NEGATIVE locally; reverted, do not retry α = 0.5.

## Hypothesis

Open-questions lead: the relabelled-AMF lottery ships α ∈ {5.0, 2.0, −1.0,
1.0, 16.0}; a new α is a new objective, hence a distinct lottery at zero
marginal pass cost (same restart count, reshuffled assignment). α = 0.5 was
the cheapest untested value.

## What changed

`src/ordering/mod.rs`, two arrays (main relabel sweep + extra-relabel
tickets): `[5.0, 2.0, -1.0, 1.0, 16.0]` → `[…, 0.5]` with `% 5` → `% 6`.
Same pass count, same gates, same seeds; best-of floor preserved.

## Result

Full trusted 300-matrix run: 0.827195 → **0.827213** (+0.18 bip, noise).

| bucket | before | after |
|---|---:|---:|
| `lt_1k` | 0.8897 | 0.8896 |
| `1k_10k` | 0.8634 | 0.8636 |
| `gt_10k` | 0.7531 | 0.7531 |

No bucket moves beyond rounding. Reverted.

## Why it lost

A sixth objective reshuffles (seed → α) assignment without adding passes, so
it can only win if α = 0.5 exposes minima the other five never reach. It does
not on this corpus — consistent with 0004's lottery doctrine (objectives
differ, but most draws are interchangeable; only restarts reliably pay).

## Follow-ups

- Do not retry α = 0.5 in either relabel array.
- α = 2.5 remains the only untested value from the open question; same
  zero-marginal-cost shape, but expect the same null — try only bundled with
  a second change that adds passes where the budget allows.
- SUBTREE_MIN_N is already 24 in-tree; the open question ("push to 16/32")
  is stale — do not treat it as a lead.

## Links

- Research queue: [open-questions](../open-questions.md)
