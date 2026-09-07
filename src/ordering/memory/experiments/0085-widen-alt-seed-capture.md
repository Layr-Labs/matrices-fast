# 0085 — Widen alternate-seed capture (extra-relabel + 8 seeds)

- **Date:** 2026-09-06
- **Score:** 0.826784 → **0.826723** (+0.74 bip); lt_1k 0.889730 tie, 1k_10k 0.863292 → 0.863089, gt_10k 0.752193 tie
- **Status:** wash for promotion (under 1 bip). Kept as stacked plumbing under 0086. Not submitted alone.

## Hypothesis

The 0084 alternate-seed PEO chain only retains orderings that flow through
`consider`. Extra-relabel tickets (the well-below-AMD lottery) update
`best_perm` directly, so those already-built permutations never enter
`runner_up`. Widening capture is plumbing, not extra ordering work. Raising
`PEO_ALT_SEEDS` 4 → 8 lets leftover ledger run extra seeds first-come; large
rows still skip when `n+nnz` exhausts the 4M allowance, so the 0084 tail
should hold.

## What changed

- `src/ordering/mod.rs`: extra-relabel AMF/AMD tickets now go through
  `consider` (seed capture + strict best-of). `PEO_ALT_SEEDS` 4 → 8. Same
  4M ledger and `n+nnz+Lnnz` cost law.

## Result

pending official local run

## Follow-ups

- Offer other direct `best_perm` writers that are portfolio orderings, not
  local-search neighbors of the incumbent.
