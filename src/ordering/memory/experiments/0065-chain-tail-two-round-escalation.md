# 0065 — Chain-tail skip + two-round escalation + small third round

- **Date:** 2026-09-05
- **Base:** `784bfe5` (hidden **0.86837**, submission `c7d5fe7`)
- **Score:** 0.843358 → **0.843059** (−2.99 bip dev)
- **Status:** WIN (local); submitted after 0064 hidden timing failure

## Hypothesis

0064 (chain-tail + three full 8M escalation rounds + flops gates) scored −3.01 bip
dev but **failed** hidden Benchmark (~255 s fatal). Drop the unconditional third
8M round on all matrices; keep:

1. gt_10k chain round-5 skip at `work_spent >= 2e9`
2. Two below-anchor escalation rounds (8M×8×2) after pair descent, flops gate only
3. A **third** 8M round only on `n < 10_000 && nnz <= 30_000` (where escalation
   lives, without stacking on gt_10k hidden worst cases)
4. Flops-only acceptance on terminal deep pass and extra 16M ticket

## Result

| | Tip | Candidate |
|---|---|---|
| Dev score | 0.843358 | **0.843059** |
| Δ bip | — | **−2.99** |
| Buckets | 0.8903 / 0.8650 / 0.7919 | 0.8901 / **0.8642** / 0.7919 |
| Worst `order()` | 1.351 s | **1.356 s** |
| Harness ×2 | — | 0.8431 / 0.9435 fill |

Full three-round escalation on this box scored 0.843055 (−3.03 bip) but worst
1.458 s — too close to 0064's failed profile.

## Links

- Predecessor: [0064](0064-chain-tail-plus-flops-gated-escalation.md) (failed hidden)
- Timing: [0063](0063-chain-tail-skip-plus-escalation-flops-gate.md)
