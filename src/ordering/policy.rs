//! Deterministic search limits. Changing these changes the policy.
use super::{rgreedy, Pattern};

/// Whole-graph AMF envelope; alpha 5 is the first alternative to AMD.
pub(super) const AMF_MAX_N: usize = 250_000;
pub(super) const AMF_MAX_NNZ: usize = 1_500_000;

pub(super) const MEDIUM_MAX_N: usize = 60_000;
pub(super) const MEDIUM_MAX_NNZ: usize = 400_000;
pub(super) const AMF_SWEEP_MAX_NNZ: usize = 1_500_000;
pub(super) const AMF_SWEEP_MAX_N: usize = 300_000;
/// Additional AMD alpha 1/16 candidates below this edge count.
pub(super) const SWEEP_EXTRA_MAX_NNZ: usize = 150_000;

pub(super) const ROBUST_MAX_N: usize = 150_000;
pub(super) const ROBUST_MAX_NNZ: usize = 130_000;

pub(super) const RCM_MAX_N: usize = 150_000;
pub(super) const RCM_MAX_NNZ: usize = 130_000;

pub(super) const ND_MAX_N: usize = 150_000;
pub(super) const ND_MAX_NNZ: usize = 130_000;

pub(super) const NDFM_MAX_N: usize = 150_000;
pub(super) const NDFM_MAX_NNZ: usize = 130_000;

pub(super) const MINFILL_MAX_N: usize = 3_000;
pub(super) const MINFILL_MAX_NNZ: usize = 12_000;

/// METIS envelopes bound both graph size and edge count. Their combined runtime
/// still needs measurement; individual gates do not certify a wall-clock limit.
pub(super) const METIS_MAX_N: usize = 130_000;
pub(super) const METIS_MAX_NNZ: usize = 320_000;

pub(super) const METIS_TUNED_MAX_N: usize = 21_000;
pub(super) const METIS_TUNED_MAX_NNZ: usize = 120_000;

/// Higher-trial METIS is limited to small graphs.
pub(super) const METIS_HITRIAL_MAX_N: usize = 8_000;
pub(super) const METIS_HITRIAL_MAX_NNZ: usize = 40_000;

pub(super) const METIS_VAR_MAX_N: usize = 30_000;
pub(super) const METIS_VAR_MAX_NNZ: usize = 60_000;

pub(super) const KAHIP_MULTI_MAX_N: usize = 12_000;
pub(super) const KAHIP_MULTI_MAX_NNZ: usize = 45_000;

pub(super) const KAHIP_MAX_N: usize = 6_000;
pub(super) const KAHIP_MAX_NNZ: usize = 50_000;

pub(super) const RELABEL_BUDGET: usize = 300_000;
pub(super) const RELABEL_MAX_RESTARTS: usize = 24;

pub(super) const RELABEL_AMF_MAX_NNZ: usize = 130_000;

pub(super) const DENSE_GREEDY_BUDGET: u64 = 1_000_000;
/// Charged (k + m) per core ordering pass. The quality-first allowance admits
/// the complementary AMF/METIS passes instead of spending everything on AMD.
pub(super) const CORE_PASS_BUDGET: u64 = 16_000_000;
/// Above this original matrix size, omit the second AMF variant. METIS is
/// admitted by residual size and nonzeros, rather than original dimension.
pub(super) const LARGE_CORE_N: usize = 150_000;

/// Adjacent-pair descent: swaps neighbouring pivots (a, b) where they are
/// adjacent in the elimination graph and deg(b) < deg(a), kept only on a
/// strict decrease. `_EXT_*` widens the gate to sparser matrices up to
/// `PAIR_DESCENT_EXT_MAX_N`, at a smaller ops budget.
pub(super) const PAIR_DESCENT_MIN_N: usize = 3;
pub(super) const PAIR_DESCENT_MAX_N: usize = 4_000;
pub(super) const PAIR_DESCENT_MAX_NNZ: usize = 60_000;
pub(super) const PAIR_DESCENT_SWEEPS: usize = 4;
pub(super) const PAIR_DESCENT_OPS_BUDGET: i64 = 128_000_000;
pub(super) const PAIR_DESCENT_EXT_MAX_N: usize = 12_000;
pub(super) const PAIR_DESCENT_EXT_OPS_BUDGET: i64 = 48_000_000;

/// Simplicial promotion: moves zero-deficiency pivots earlier across a local
/// lookahead window (Ost, Schulz, Strash 2020); zero fill added, so always
/// safe, kept only on a strict decrease.
pub(super) const SIMPLICIAL_PROMOTION_MIN_N: usize = 3;
pub(super) const SIMPLICIAL_PROMOTION_MAX_N: usize = 6_000;
pub(super) const SIMPLICIAL_PROMOTION_MAX_NNZ: usize = 100_000;
pub(super) const SIMPLICIAL_PROMOTION_MAX_DENSITY: usize = 24;
pub(super) const SIMPLICIAL_PROMOTION_OPS_BUDGET: i64 = 64_000_000;

/// Ranked-subtree refinement chain: bounded, disjoint-block exact local
/// search over the incumbent's elimination tree. `SUBTREE_MIN_N` is
/// structural (below it a tree has too few searchable subtrees);
/// `SUBTREE_MAX_N` bounds the one-time etree/column-count setup cost.
pub(super) const SUBTREE_MIN_N: usize = 24;
pub(super) const SUBTREE_MAX_N: usize = 250_000;
pub(super) const MID_MAX_S: usize = 128;
pub(super) const LARGE_MAX_S: usize = 384;
pub(super) const MID_BLOCKS: usize = 16;
pub(super) const LARGE_BLOCKS: usize = 16;
/// Effective work allowance for each additional medium/large subtree pass.
/// The kernel uses the supplied budget unchanged; policy owns this limit.
pub(super) const SUBTREE_PASS_WORK: i64 = 2_000_000;
pub(super) const SUBTREE_CFG: rgreedy::SubCfg = rgreedy::SubCfg {
    min_s: 32,
    max_s: 384,
    max_sub: 1_200,
    max_blocks: 32,
    budget: 1_000_000,
    streams: 1,
    rank_blocks: true,
    round: 0,
};

/// Base ranked-subtree settings; small graphs admit blocks down to size 8.
pub(super) fn subtree_cfg_for(n: usize, nnz: usize) -> rgreedy::SubCfg {
    let mut cfg = SUBTREE_CFG;
    if n < 64 {
        cfg.min_s = 8;
        cfg.max_s = 32;
        cfg.max_blocks = 8;
        cfg.budget = 1_000_000;
    } else if n <= 1_000 {
        cfg.min_s = 8;
        cfg.max_s = 256;
        cfg.max_blocks = 16;
        cfg.budget = if n == 1_000 { 1_000_000 } else { 2_000_000 };
    } else if n >= 10_000 {
        cfg.max_s = LARGE_MAX_S;
        cfg.max_blocks = LARGE_BLOCKS;
        if nnz <= n * 10 && nnz <= 150_000 {
            cfg.max_sub = 1_600;
        }
    } else {
        cfg.min_s = 32;
        cfg.max_s = MID_MAX_S;
        cfg.max_blocks = MID_BLOCKS;
    }
    cfg
}

/// Config for the deeper terminal ranked-subtree pass, run once the chain
/// above has settled: a wider window than any chain round, sized by whether
/// the incumbent is already strictly below the AMD anchor.
pub(super) fn terminal_deep_subtree_cfg(
    n: usize,
    nnz: usize,
    best_flops: u64,
    amd_flops: u64,
) -> rgreedy::SubCfg {
    let mut cfg = SUBTREE_CFG;
    cfg.min_s = 16;
    cfg.round = 5;
    let is_below = best_flops < amd_flops;
    if n < 10_000 {
        cfg.max_blocks = 4;
        cfg.max_s = 768;
        cfg.budget = if n >= 1_000 { SUBTREE_PASS_WORK } else { 4_000_000 };
    } else {
        cfg.max_blocks = 8;
        cfg.max_s = if is_below && nnz <= 50_000 {
            768
        } else {
            1_200
        };
        cfg.budget = 1_000_000;
        if nnz <= n * 10 && nnz <= 150_000 {
            cfg.max_sub = 1_600;
        }
    }
    cfg
}

/// Steps after the first accepted ranked-subtree pass. Rebuild from the base
/// configuration each time; some fields deliberately reset between steps.
pub(super) fn subtree_chain_cfg(
    n: usize,
    nnz: usize,
    cost: u64,
    amd_cost: u64,
    step: usize,
) -> Option<rgreedy::SubCfg> {
    let mut cfg = subtree_cfg_for(n, nnz);
    match step {
        1 | 2 => {
            cfg.round = 1;
            cfg.max_blocks = 32;
            cfg.min_s = 16;
            cfg.budget = if n >= 1_000 { SUBTREE_PASS_WORK } else { 8_000_000 };
            if step == 2 {
                cfg.max_s = 512;
            } else if cost < amd_cost && (1_000..10_000).contains(&n) {
                cfg.max_s = 256;
            }
        }
        3 => {
            cfg.round = 3;
            cfg.max_blocks = 32;
            cfg.min_s = 16;
            cfg.max_s = 768;
            cfg.budget = if n >= 1_000 { SUBTREE_PASS_WORK } else { 32_000_000 };
        }
        4 => {
            if n >= 100_000 && cost == amd_cost {
                return None;
            }
            cfg.round = 4;
            if (1_000..4_000).contains(&n) {
                cfg.max_blocks = 16;
            } else {
                cfg.max_blocks = 32;
            }
            cfg.budget = if n >= 1_000 { SUBTREE_PASS_WORK } else { 16_000_000 };
        }
        _ => return None,
    }
    Some(cfg)
}

pub(super) const WINDOW_MAX_N: usize = 4_000;

#[cfg(test)]
mod budget_tests {
    #[test]
    fn explicit_budgets_match_the_previous_kernel_limits() {
        for n in [24, 63, 64, 999, 1000, 1001, 3999, 4000, 5999, 6000, 9999,
                  10_000, 99_999, 100_000, 250_000] {
            for cost in [50, 100] {
                for step in 1..=4 {
                    let cfg = super::subtree_chain_cfg(n, 3 * n, cost, 100, step);
                    if step == 4 && n >= 100_000 && cost == 100 {
                        assert!(cfg.is_none());
                        continue;
                    }
                    let previous = match step {
                        1 | 2 => 8_000_000,
                        3 if (1000..6000).contains(&n) => 64_000_000,
                        3 => 32_000_000,
                        4 if (1000..4000).contains(&n) => 32_000_000,
                        _ => 16_000_000,
                    };
                    let effective = if n >= 1000 { (previous / 2).min(2_000_000) } else { previous };
                    assert_eq!(cfg.unwrap().budget, effective, "n={n}, step={step}");
                }
            }
        }
    }
}
pub(super) const WINDOW_MAX_NNZ: usize = 60_000;
pub(super) const WINDOW_K: usize = 12;
pub(super) const WINDOW_STRIDE: usize = 4;
pub(super) const WINDOW_BUDGET: i64 = 96_000_000;
pub(super) const INSERTION_MAX_N: usize = 1_000;
pub(super) const INSERTION_MAX_NNZ: usize = 30_000;
pub(super) const INSERTION_SWEEPS: usize = 2;
pub(super) const INSERTION_BUDGET: i64 = 64_000_000;

/// Shared eligibility for the initial local search and its final cleanup.
pub(super) struct LocalPolicy {
    pub pair_budget: Option<i64>,
    pub simplicial: bool,
}

impl LocalPolicy {
    pub fn for_pattern(pattern: &Pattern) -> Self {
        let n = pattern.n;
        let nnz = pattern.nnz();
        let max_degree = pattern
            .col_ptr
            .windows(2)
            .map(|c| c[1] - c[0])
            .max()
            .unwrap_or(0);
        let extended = n > PAIR_DESCENT_MAX_N
            && n <= PAIR_DESCENT_EXT_MAX_N
            && nnz <= 30_000
            && max_degree * 50 <= n;
        let pairs = n >= PAIR_DESCENT_MIN_N
            && nnz > 0
            && nnz <= PAIR_DESCENT_MAX_NNZ
            && (n <= PAIR_DESCENT_MAX_N || extended);
        Self {
            pair_budget: pairs.then_some(if extended {
                PAIR_DESCENT_EXT_OPS_BUDGET
            } else {
                PAIR_DESCENT_OPS_BUDGET
            }),
            simplicial: (SIMPLICIAL_PROMOTION_MIN_N..=SIMPLICIAL_PROMOTION_MAX_N).contains(&n)
                && nnz > 0
                && nnz <= SIMPLICIAL_PROMOTION_MAX_NNZ
                && nnz <= n.saturating_mul(SIMPLICIAL_PROMOTION_MAX_DENSITY),
        }
    }
}

/// Fixed streams in their original order. Later streams see earlier winners.
pub(super) fn greedy_streams(
    n: usize,
    nnz: usize,
    cost: u64,
    amd_cost: u64,
) -> &'static [(i64, u64)] {
    const SMALL: &[(i64, u64)] = &[
        (100_000_000, 0x9E37_79B9_7F4A_7C15),
        (50_000_000, 0xD1B5_4A32_D192_ED03),
        (50_000_000, 0x27BB_2EE6_87B0_B0FD),
        (50_000_000, 0x45A1_89C3_F208_7314),
        (100_000_000, 0xA076_1D64_78BD_642F),
        (50_000_000, 0xE703_7ED1_A0B4_28DB),
    ];
    const MEDIUM: &[(i64, u64)] = &[
        (100_000_000, 0xD1B5_4A32_D192_ED03),
        (50_000_000, 0xD1B5_4A32_D192_ED03),
        (50_000_000, 0x27BB_2EE6_87B0_B0FD),
    ];
    const MEDIUM_STRONG: &[(i64, u64)] = &[
        (100_000_000, 0xD1B5_4A32_D192_ED03),
        (100_000_000, 0x27BB_2EE6_87B0_B0FD),
        (100_000_000, 0xA076_1D64_78BD_642F),
        (50_000_000, 0x45A1_89C3_F208_7314),
        (50_000_000, 0xD1B5_4A32_D192_ED03),
    ];
    let strong =
        amd_cost > 0 && cost < amd_cost && cost.saturating_mul(5) < amd_cost.saturating_mul(4);
    if n <= 1_000 && nnz <= 30_000 {
        return &SMALL[..if strong { 6 } else { 5 }];
    }
    if !(1_001..=6_000).contains(&n) || !(nnz <= 30_000 || (strong && nnz <= 50_000)) {
        return &[];
    }
    if strong {
        MEDIUM_STRONG
    } else {
        &MEDIUM[..if cost < amd_cost && n <= 3_000 && nnz <= 18_000 {
            3
        } else {
            2
        }]
    }
}
