# 0056 — H3: SqDiv/SqPure gate to n<10000, density>=2

**Date:** 2026-09-04
**Score:** `0.845281` → **`0.845187`** (−0.000094, −1.11 basis points); fill `0.944707` unchanged
**Status:** FAILED hidden validation (`21e1d96c`, commit `2f1b84a`). Same class as R5-32M: GHA Benchmark exit 1, almost certainly the 2.0s hidden `order()` cap. Local 300-matrix run was clean (worst well under 2s) but the extra density-2 / n<10k matrices on the hidden corpus did not stay inside the cap. **Do not resubmit this gate.** Reverted to promoted H2 (`n<5000 && nnz>=3n`).
**Peak latency:** full corpus finished under the local 2.0s cap (wall ~229 s, no FAIL/timeout rows)

## Hypothesis

H2 (`4245c79`) extended SqDiv/SqPure from `nnz >= 10n` to also `n<5000 && nnz >= 3n`. That transferred to hidden (`0.871239 → 0.871032`). The remaining 1k_10k headroom is matrices with n in [5000, 10000) or density in [2, 3) that still never see those four quotient-graph passes. Same four calls, wider gate, large-n cap cases untouched.

## Change

```rust
if nnz <= 300_000 && (nnz >= 10 * n || (n < 10_000 && nnz >= 2 * n)) {
```

was `n < 5_000 && nnz >= 3 * n`. No new variant, alpha, pass, block, seed, or budget.

## Ablations on this box (all reverted except the shipped gate)

| trial | local score | vs 0.845281 | notes |
|---|---:|---:|---|
| relabel-AMF extra α 0.5/2.5, n<1000 nnz<=20k | lt_1k 0.8930 | noise | too small |
| DegSqrt/P075/P125 × {1,10} **early** (pre-search) | 0.8454 | **regress** 1k_10k 0.8689 | steals incumbent, subtree path worse |
| same Deg* **terminal** (post-search) | 0.845281 | 0 | refined incumbent already better |
| n<8000 && density>=3 | 0.845281 | 0 | H2 band already had the 3× winners |
| terminal 8-seed relabel RCM/Sloan, n<1000 | lt_1k 0.8931 | 0 | family still zero unique wins |
| n<5000 && density>=2 | 0.845255 | −0.31 bips | 1k_10k 0.868302 |
| **n<10000 && density>=2 (this)** | **0.845187** | **−1.11 bips** | 1k_10k 0.868076 |

## Buckets

| bucket | H2 (4245c79) | H3 | change |
|---|---:|---:|---:|
| lt_1k (147) | 0.893120 | 0.893120 | 0 |
| 1k_10k (108) | 0.868390 | 0.868076 | −0.000314 |
| gt_10k (45) | 0.792071 | 0.792071 | 0 |
| weighted | 0.845281 | **0.845187** | **−0.000094** |

gt_10k unchanged is the timing thesis: crudeoil_lee4_10 (n=17809) and arki0013 (n=44909) never enter this gate.

## Why not the other recent attempts

R5 32M in `1k..6k` (62253c5) and gdonninelli's R5 ceiling raises failed hidden `order() exceeded the 2.0s per-matrix cap`. jtaroreh's SUBTREE_MIN_N=24 + ACOPF skip + max_sub=1600 scored 0.844821 local (−4.6 bips) but only 0.870978 hidden (−0.62 bips) — search-path overfitting. This change copies H2's mechanism, not their search budgets.
