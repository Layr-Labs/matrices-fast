# 0036 — Multi-Round Cascading Terminal Subtree Refinement with Sparsity-Gated Large Tier

- **Date:** 2026-09-03
- **Official Leaderboard Score:** 0.876094 (`hybridnoise`, `f04da6e`) → **0.875942** (`hybridnoise`, `1417f26`)
  (−0.000152 / −1.52 basis points; fill 0.957947 → **0.957904**)
- **Development Score:** 0.850464 → **0.850370** (−0.000094; fill **0.948420**)
- **Parent:** `f04da6e` (our promoted 1st-place lead `db5ad14`)
- **Status:** **PROMOTED TO #1 ON OFFICIAL LEADERBOARD** (`e4a98396-6949-4d49-af9c-814bf0ab52e0`)

## Hypothesis

1. **Cascading Funnel**: In elimination tree local search, when an ordering improves, internal parent pointers and column counts change. If we chain a tertiary pass that runs only when *both* round 1 and round 2 have succeeded ($f_3 < f_2 < f_1$), CPU cycles are invested exclusively where the graph has active convergence potential.
2. **Sparsity-Gated Large Tier**: In `gt_10k`, heavy QP/KKT instances ($nnz > 100,000$, such as `crudeoil_lee4_10`) cannot afford additional phases. However, ultra-sparse large matrices ($n \ge 10,000$ with $nnz \le 60,000$, like `emfl050_5_5`) take $< 0.35$s. Gating the chained pass by `(n < 10_000 && nnz <= 100_000) || (n >= 10_000 && nnz <= 60_000)` safely unlocks improvements in the 40%-weighted `gt_10k` bucket.
3. **Small Cluster Search (`min_s = 8`)**: As upper separators optimize, remaining fill bottlenecks often consist of tiny 8-to-15 variable dense cliques that can be evaluated in sub-millisecond bitset time.

## Design

Inside `order()`, when the primary terminal pass finds an improved bijection:

```rust
if (n < 10_000 && nnz <= 100_000) || (n >= 10_000 && nnz <= 60_000) {
    // Round 2 chained pass with unaliased round 6, min_s 8
    ...
    if improved2 > 0 && is_bijection(&candidate2, n) {
        let f2 = flops_of(&scoring_pat, &candidate2);
        if f2 < f {
            best_perm = candidate2;

            // Round 3 chained pass with unaliased round 7, min_s 8
            ...
            if improved3 > 0 && is_bijection(&candidate3, n) {
                let f3 = flops_of(&scoring_pat, &candidate3);
                if f3 < f2 {
                    best_perm = candidate3;
                }
            }
        }
    }
}
```

## Result

- Official evaluation score dropped to **0.875942** (broken through the 0.8760 milestone barrier!).
- Development score dropped to **0.850370** (fill tiebreak **0.948420**).
- `gt_10k` geomean dropped below 0.7970 to **0.796916**.
- Worst single-matrix execution time: **0.882 seconds** (> 1.118s of safety margin below 2.0s).
