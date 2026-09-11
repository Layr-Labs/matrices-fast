//! Exact-scored local search and final window cleanup.
use super::{policy::*, *};

pub(super) fn refine(
    pattern: &Pattern,
    scoring_pat: &ScoringPattern,
    best: &mut Candidate,
    amd_flops: u64,
) {
    let n = pattern.n;
    let nnz = pattern.nnz();
    let policy = LocalPolicy::for_pattern(pattern);
    if let Some(budget) = policy.pair_budget {
        best.consider(
            scoring_pat,
            rgreedy::adjacent_pair_descent(
                n,
                &pattern.col_ptr,
                &pattern.row_idx,
                &best.perm,
                PAIR_DESCENT_SWEEPS,
                budget,
            ),
        );
    }

    if policy.simplicial {
        best.consider(
            scoring_pat,
            rgreedy::simplicial_promotion(
                n,
                &pattern.col_ptr,
                &pattern.row_idx,
                &best.perm,
                SIMPLICIAL_PROMOTION_OPS_BUDGET,
            ),
        );
    }

    // Choose streams once, before any stream changes the incumbent.
    let streams = greedy_streams(n, nnz, best.cost, amd_flops);
    for &(budget, seed) in streams {
        best.consider(
            scoring_pat,
            rgreedy::search(
                n,
                &pattern.col_ptr,
                &pattern.row_idx,
                &best.perm,
                best.cost,
                budget,
                seed,
            )
            .map(|(p, _)| p),
        );
    }

    if n > 1_000 && !streams.is_empty() {
        if let Some(budget) = policy.pair_budget {
            best.consider(
                scoring_pat,
                rgreedy::adjacent_pair_descent(
                    n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &best.perm,
                    PAIR_DESCENT_SWEEPS,
                    budget,
                ),
            );
        }
    }
}

pub(super) fn cleanup(pattern: &Pattern, scoring_pat: &ScoringPattern, best: &mut Candidate) {
    let n = pattern.n;
    let nnz = pattern.nnz();
    let policy = LocalPolicy::for_pattern(pattern);
    if policy.simplicial {
        best.consider(
            scoring_pat,
            rgreedy::simplicial_promotion(
                n,
                &pattern.col_ptr,
                &pattern.row_idx,
                &best.perm,
                SIMPLICIAL_PROMOTION_OPS_BUDGET,
            ),
        );
    }
    if let Some(budget) = policy.pair_budget {
        for _ in 0..2 {
            let before = best.cost;
            if n >= 5 {
                best.consider(
                    scoring_pat,
                    rgreedy::adjacent_five_descent(
                        n,
                        &pattern.col_ptr,
                        &pattern.row_idx,
                        &best.perm,
                        budget,
                    ),
                );
            }
            best.consider(
                scoring_pat,
                rgreedy::adjacent_four_descent(
                    n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &best.perm,
                    budget,
                ),
            );
            if best.cost == before {
                break;
            }
        }
    }

    if n >= 3 && n <= rgreedy::MAX_N {
        if n <= WINDOW_MAX_N && nnz <= WINDOW_MAX_NNZ {
            best.consider(
                scoring_pat,
                rgreedy::window_descent(
                    n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &best.perm,
                    WINDOW_K,
                    WINDOW_STRIDE,
                    WINDOW_BUDGET,
                ),
            );
        }
        if n <= INSERTION_MAX_N && nnz <= INSERTION_MAX_NNZ {
            best.consider(
                scoring_pat,
                rgreedy::insertion_descent(
                    n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &best.perm,
                    INSERTION_SWEEPS,
                    INSERTION_BUDGET,
                ),
            );
        }
    }
}
