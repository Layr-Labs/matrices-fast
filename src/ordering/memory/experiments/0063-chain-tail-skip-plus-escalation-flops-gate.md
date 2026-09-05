# 0063 — Chain-tail skip on gt_10k + below-anchor escalation with flops gate

- **Date:** 2026-09-05
- **Score:** 0.843658 → **0.843339** (−3.19 bip on dev; 1 bip = 1e-4)
- **Status:** win (local); submitted after fixing hidden-cap failure mode

## Hypothesis

Two independent fixes compose:

1. **Substitution (timing):** Skip subtree-chain round 5 on `n >= 10_000` once a
   requested-work ledger exceeds 2.0e9. Round 4 stays — it carries medium-band
   gains. Round 5's 16–32M×32-block tail is what stacked on expensive hidden
   matrices in failed additive-escalation submissions (0060–0062).

2. **Below-anchor terminal escalation (score):** Three unconditional postordered
   subtree rounds (8M×8×3) after all cleanup, gated on `best_flops < amd_flops`
   and `nnz <= 50_000`. Accept improvements via `flops_of` only — **not** via
   `subtree_refine`'s `improved` return counter, which can be zero even when the
   incumbent permutation strictly improves.

## What changed

`src/ordering/mod.rs` only:

- `CHAIN_TAIL_SPENT_CEILING`, `sub_cfg_work()`, `work_spent` ledger through exact
  search and subtree-chain rounds 1–5.
- Round-5 guard: `skip_round5 = work_spent >= ceiling && n >= 10_000`.
- Terminal escalation block at end of `order()` (after pair descent): three
  rounds `(384,8,8M) → (768,8,8M) → (384,8,8M)`.

## Result

| metric | base `c3fae01` | candidate |
|---|---|---|
| dev score | 0.843658 | **0.843339** |
| Δ bip | — | **−3.19** |
| worse / better | — | **0 / 42** |
| buckets lt_1k / 1k_10k / gt_10k | 0.8903 / 0.8659 / 0.7919 | 0.8901 / 0.8651 / 0.7919 |
| worst `order()` | 1.364 s | **1.357 s** |
| harness ×2 | — | 0.8433 / 0.9436 fill, byte-identical |

Largest movers: `graphpart_clique-70`, `arki0002`, `maxcsp-langford-3-11`,
`netmod_kar1`, `rsyn0810m04m`.

## Why it won / lost

**The `improved > 0` guard was silently killing escalation.** On `c3fae01`,
`subtree_refine` can return 0 while still leaving a strictly lower-flops
permutation in `candidate`. The prior 0062 draft used `improved > 0` and measured
0.843658 (identical to base) in this session until the guard was removed; with
flops-only acceptance the score dropped to 0.843339 as originally intended.

**Global round-5 skip regressed 23 medium matrices** (chimera, crudeoil_li05).
Restricting the skip to `n >= 10_000` preserves those movers while still cutting
tail work on gt_10k paths that approach the hidden 2 s cap.

**Additive escalation without chain-tail discipline failed hidden** (0062
submission `b784b8b6`, Benchmark step ~255 s). The chain-tail skip is the
substitution that keeps total work bounded on the slowest instances while
retaining the score-positive escalation on sparse below-anchor matrices.

## Follow-ups

- If hidden still fails timing: lower escalation to 2×8M rounds or raise
  `CHAIN_TAIL_SPENT_CEILING` only on matrices with measured local time > 1.2 s.
- Bootstrap the −3.19 bip delta — gain is concentrated in ~6 matrices.

## Links

- Techniques: [nested-dissection](techniques/nested-dissection.md) (negative on ties)
- Prior: [0060](0060-conditional-search-escalation-below-anchor.md)
