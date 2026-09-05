# 0068 — Core relabel multistart + 0067 pooling-band chain-tail

- **Date:** 2026-09-05
- **Base:** `2a28517` (hidden **0.867211**, 0063 tip)
- **Score:** 0.839063 → **0.838481** (−5.82 bip)
- **Worst `order()`:** 1.334 s → **1.479 s**
- **Status:** submitted

## Changes

1. **0067 timing** (re-applied): pooling-band leftover-384, 32M round-4, skip round-5,
   gt_10k chain-tail at `work_spent >= 2e9`
2. **0062 follow-up:** relabelled AMD + AMF multistart on the residual core in the
   terminal reduce-then-AMF block (`REDUCE_CORE_RELABEL_BUDGET=150k`, cap 6,
   `cn < 12k`, `core_nnz <= 350k`, AMF capped at 4 passes)

## Rationale

0067 was hidden-neutral (+0.000001). Score moves need 0062 basin changes; core relabel
tickets are ~3× cheaper than full-matrix restarts and reach orderings the fixed AMF grid misses.
