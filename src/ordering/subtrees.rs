//! Ranked-subtree refinement, followed by the deeper terminal passes.
use super::{policy::*, *};

pub(super) fn refine(
    pattern: &Pattern,
    scoring_pat: &ScoringPattern,
    best: &mut Candidate,
    amd_flops: u64,
) {
    let n = pattern.n;
    let nnz = pattern.nnz();
    if (SUBTREE_MIN_N..=SUBTREE_MAX_N).contains(&n) && nnz <= 1_500_000 {
        ranked_chain(pattern, scoring_pat, best, amd_flops);
    }
    if (SUBTREE_MIN_N..=80_000).contains(&n) && nnz <= 250_000 {
        terminal_chain(pattern, scoring_pat, best, amd_flops);
    }
    // One extra ranked-subtree ticket on below-anchor small/medium graphs.
    if best.cost < amd_flops && n < 10_000 && nnz <= 100_000 && n >= SUBTREE_MIN_N {
        let mut extra = SUBTREE_CFG;
        extra.min_s = 16;
        extra.max_s = 512;
        extra.max_blocks = 4;
        extra.budget = if n >= 1_000 {
            SUBTREE_PASS_WORK
        } else {
            4_000_000
        };
        extra.round = 8;
        apply(pattern, scoring_pat, best, extra);
    }
}

fn apply(
    pattern: &Pattern,
    scoring: &ScoringPattern,
    best: &mut Candidate,
    cfg: rgreedy::SubCfg,
) -> bool {
    let (mut candidate, counts, parent) = etree_prep(scoring, &best.perm);
    let improved = rgreedy::subtree_refine(
        pattern.n,
        &pattern.col_ptr,
        &pattern.row_idx,
        &mut candidate,
        &counts,
        &parent,
        cfg,
    );
    improved > 0 && best.consider(scoring, Some(candidate))
}

fn ranked_chain(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate, amd_cost: u64) {
    let n = pattern.n;
    let nnz = pattern.nnz();
    let (mut candidate, counts, parent) = etree_prep(scoring, &best.perm);
    let mut cfg = subtree_cfg_for(n, nnz);
    let mut improved = rgreedy::subtree_refine(
        n,
        &pattern.col_ptr,
        &pattern.row_idx,
        &mut candidate,
        &counts,
        &parent,
        cfg,
    );
    if improved == 0 && best.cost < amd_cost && n <= 80_000 && nnz <= 250_000 {
        cfg.round = 1;
        if n < 1_000 {
            cfg.streams = 2;
            cfg.budget = 1_000_000;
        } else {
            cfg.max_s = if n < 10_000 { 256 } else { 512 };
        }
        // Retry the same prepared candidate, as in the original schedule.
        improved = rgreedy::subtree_refine(
            n,
            &pattern.col_ptr,
            &pattern.row_idx,
            &mut candidate,
            &counts,
            &parent,
            cfg,
        );
    }
    if improved == 0 || !best.consider(scoring, Some(candidate)) {
        return;
    }
    for step in 1..=4 {
        let Some(cfg) = subtree_chain_cfg(n, nnz, best.cost, amd_cost, step) else {
            break;
        };
        if !apply(pattern, scoring, best, cfg) {
            break;
        }
    }
}

fn terminal_chain(
    pattern: &Pattern,
    scoring: &ScoringPattern,
    best: &mut Candidate,
    amd_cost: u64,
) {
    let n = pattern.n;
    let nnz = pattern.nnz();
    let before = best.cost;
    let cfg = terminal_deep_subtree_cfg(n, nnz, best.cost, amd_cost);
    if !apply(pattern, scoring, best, cfg) {
        return;
    }
    // Only the first terminal pass needs a 0.5% gain. Later passes need a
    // strict improvement and their own eligibility checks.
    let eligible = (n < 10_000 && nnz <= 100_000)
        || (n >= 10_000 && nnz <= 60_000)
        || (n >= 10_000 && nnz <= 100_000 && best.cost < amd_cost);
    if (before - best.cost) as f64 / (before as f64) < 0.005 || !eligible {
        return;
    }
    for round in [6, 7] {
        if round == 7
            && !((n < 10_000 && nnz <= 100_000)
                || (n >= 10_000 && nnz <= 80_000 && best.cost < amd_cost))
        {
            break;
        }
        let mut cfg = terminal_deep_subtree_cfg(n, nnz, best.cost, amd_cost);
        cfg.round = round;
        cfg.min_s = 8;
        cfg.max_s = if n >= 10_000 { 512 } else { 384 };
        cfg.max_blocks = if best.cost < amd_cost { 4 } else { 2 };
        cfg.budget = if n >= 1_000 {
            SUBTREE_PASS_WORK
        } else {
            4_000_000
        };
        if !apply(pattern, scoring, best, cfg) {
            break;
        }
    }
}
