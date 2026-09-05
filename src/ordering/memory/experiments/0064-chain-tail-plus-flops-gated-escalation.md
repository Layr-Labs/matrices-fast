# 0064 — Chain-tail skip + flops-gated terminal escalation on 0061 tip

- **Date:** 2026-09-05
- **Base:** `784bfe5` (hidden **0.86837**, submission `c7d5fe7`)
- **Score:** 0.843358 → **0.843057** (−3.01 bip dev)
- **Status:** WIN (local); submitted

## Hypothesis

The promoted 0061 tip already spends margin-scaled leftover search. Two independent
additions from cycle 0063 still had headroom:

1. **gt_10k chain round-5 skip** when `work_spent >= 2e9` — timing substitution
   that let hidden Benchmark complete (0063 `8e2c214` scored 0.869356 vs failed
   `b784b8b6`).
2. **Three-round below-anchor terminal escalation** after pair descent, accepting
   on `flops_of` only. The `improved > 0` guard silently no-ops the stage.

Also fix the 0061 extra 16M pass to use the same flops gate.

## Result

| | Tip | Candidate |
|---|---|---|
| Dev score | 0.843358 | **0.843057** |
| Δ bip | — | **−3.01** |
| worse / better | — | **0 / 40** |
| Buckets | 0.8903 / 0.8650 / 0.7919 | 0.8901 / **0.8642** / 0.7919 |
| Worst `order()` | 1.352 s | 1.360 s |
| Harness ×2 | — | 0.8431 / 0.9435 fill |

Largest movers: `graphpart_clique-70`, `arki0002`, `crudeoil_pooling_ct3`,
`netmod_kar1`.

## Links

- Predecessor: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
- Timing: [0063-chain-tail-skip-plus-escalation-flops-gate.md](0063-chain-tail-skip-plus-escalation-flops-gate.md)
