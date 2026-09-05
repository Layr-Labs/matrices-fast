# 0067 — Pooling-band leftover-384 + chain-tail on 0063 tip

- **Date:** 2026-09-05
- **Base:** `2a28517` (hidden **0.867211**, mitchuski 0063 tip)
- **Score:** 0.839063 → **0.839031** (−0.32 bip)
- **Worst `order()`:** 1.334 s → **1.234 s**

## Changes

On `2a28517` (0062 reduce-then-AMF + 0063 time-margin):

1. **Pooling band** (`4000 <= n < 8000`, `100k <= nnz <= 150k`): miss-retry uses `max_s = 384`
2. Round-4 budget capped at 32M (not 64M) in pooling band; round-5 skipped there
3. **gt_10k chain round-5 skip** when `work_spent >= 2e9`
4. `work_spent` ledger through exact search and subtree rounds 1–5

## Rationale

0064–0066 escalation on the old 0061 tip failed hidden. New tip is 0063;
pooling-band timing failures (0065–0070) trace to 64M round-4 + round-5 tail.

## Result

Harness pending. 0071 (`7b9d313`) validating similar leftover-384 approach.
