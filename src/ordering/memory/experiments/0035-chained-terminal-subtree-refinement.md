# 0035 — Chained Terminal Subtree Refinement with Unaliased Diversification

- **Date:** 2026-09-03
- **Official Leaderboard Score:** 0.876273 (`GordoAR`, `971649b`) → **0.876094** (`hybridnoise`, `f04da6e`)
  (−0.000179 / −1.79 basis points; fill 0.958045 → **0.957947**)
- **Development Score:** 0.850594 → **0.850464** (−0.000130; fill **0.948439**)
- **Parent:** `971649b` (GordoAR's promoted lead `586b684`)
- **Status:** **PROMOTED TO #1 ON OFFICIAL LEADERBOARD** (`db5ad145-d165-4e89-a5ea-e86630043035`)

## Hypothesis

A major challenge with adding terminal subtree search phases is the remote hidden test set time limit (2.0s per matrix). As discovered in Experiment 0025, blanket additions across all matrices cause SIGKILL timeouts on dense hidden graphs.

However, once an initial terminal pass successfully lowers the factorization flops, the underlying elimination tree $T(A)$ undergoes topological transformation. New subtrees with lower elimination fill emerge. If we chain an additional refinement pass **strictly conditioned on the first pass having found an improvement**, and strictly restrict it to medium sparse graphs ($n < 10,000$ and $nnz \le 100,000$), we can extract these secondary gains with zero overhead on the vast majority of non-improving or heavy matrices.

## Design

Inside `order()`, when the primary terminal pass finds an improved bijection:

```rust
if improved > 0 && is_bijection(&candidate, n) {
    let f = flops_of(&scoring_pat, &candidate);
    if f < incumbent_flops {
        best_perm = candidate;

        // Chained terminal pass 2: runs ONLY on medium sparse matrices (n < 10_000 && nnz <= 100_000)
        // that strictly improved in the first terminal pass. Uses unaliased
        // round = 6 and a small 4-block budget on the newly uncovered elimination tree.
        if n < 10_000 && nnz <= 100_000 {
            let permuted2 = permute_pattern(&scoring_pat, &best_perm);
            let etree2 = EliminationTree::from_pattern(&permuted2);
            let post2 = etree2.postorder();
            let mut candidate2: Vec<usize> = post2.iter().map(|&j| best_perm[j]).collect();
            let post_pattern2 = permute_pattern(&scoring_pat, &candidate2);
            let post_etree2 = EliminationTree::from_pattern(&post_pattern2);
            let counts2: Vec<u32> = column_counts_gnp(&post_pattern2, &post_etree2)
                .into_iter()
                .map(|c| c as u32)
                .collect();
            let parent2: Vec<i32> = post_etree2
                .parent
                .iter()
                .map(|p| p.map_or(-1, |j| j as i32))
                .collect();
            let mut cfg2 = terminal_deep_subtree_cfg(n);
            cfg2.round = 6;
            cfg2.max_blocks = 4;
            cfg2.budget = 4_000_000;
            let improved2 = rgreedy::subtree_refine(
                n,
                &pattern.col_ptr,
                &pattern.row_idx,
                &mut candidate2,
                &counts2,
                &parent2,
                cfg2,
            );
            if improved2 > 0 && is_bijection(&candidate2, n) {
                let f2 = flops_of(&scoring_pat, &candidate2);
                if f2 < f {
                    best_perm = candidate2;
                }
            }
        }
    }
}
```

Key principles:
1. **Zero Added Operations on Inactive Matrices**: Non-improving matrices never enter the branch.
2. **Dense/Large Guard**: Large matrices ($n \ge 10,000$) and dense matrices ($nnz > 100,000$) like `crudeoil_lee4_10` never execute this code.
3. **Unaliased Stream (`round = 6`)**: Employs an independent PRNG trajectory to avoid repeating identical search walks.
4. **Strict Monotonicity**: Kept if and only if $f_2 < f$.

## Result

- Official hidden test set score dropped to **0.876094** (promotion from GordoAR's 0.876273).
- Development corpus score dropped from 0.850594 to **0.850464** (fill tiebreak **0.948439**).
- Worst single-matrix execution time: **0.876 seconds**, maintaining > 1.124s of safety margin below the 2.0s limit.
