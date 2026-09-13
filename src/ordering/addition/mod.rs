use super::{flops_of, is_bijection, ScoringPattern};
use crate::Pattern;
use feral::ordering::elimination_tree::EliminationTree;
use feral::symbolic::column_counts_gnp;

mod decomposition;
mod joint;
mod parallel;
mod patch;
mod promotions;
mod quotient;
mod recovery_separator;

fn etree_prep(sp: &ScoringPattern, order: &[usize]) -> (Vec<usize>, Vec<u32>, Vec<i32>) {
    let permuted = super::permute_pattern(sp, order);
    let tree = EliminationTree::from_pattern(&permuted);
    let post = tree.postorder();
    let counts = column_counts_gnp(&permuted, &tree);
    let mut rank = vec![0; sp.n];
    for (i, &v) in post.iter().enumerate() {
        rank[v] = i;
    }
    (
        post.iter().map(|&v| order[v]).collect(),
        post.iter().map(|&v| counts[v] as u32).collect(),
        post.iter().map(|&v| tree.parent[v].map_or(-1, |p| rank[p] as i32)).collect(),
    )
}

pub(super) fn refine(pattern: &Pattern, baseline: Vec<usize>) -> Vec<usize> {
    let n = pattern.n;
    if !(64..=320_000).contains(&n) || pattern.nnz() > 1_500_000 {
        return baseline;
    }
    let sp = ScoringPattern {
        n,
        col_ptr: pattern.col_ptr.clone(),
        row_idx: pattern.row_idx.clone(),
    };
    let (_, counts, _) = etree_prep(&sp, &baseline);
    let entries: usize = counts.iter().map(|&c| c as usize).sum();
    let mut best_cost: u64 = counts.iter().map(|&c| (c as u64).pow(2)).sum();
    if entries + n + pattern.nnz() > 3_000_000 || best_cost < 32 * n as u64 {
        return baseline;
    }
    let mut best = baseline.clone();
    let mut donors = vec![baseline];
    for directed in [false, true] {
        let mut cfg = recovery_separator::RecoveryCfg::fast(false);
        cfg.roots = 6;
        cfg.caps = if directed { vec![512, 128] } else { vec![1024, 256] };
        cfg.skeleton_modes = if directed { vec![0, 9, 11] } else { vec![0, 5] };
        cfg.directed = directed;
        cfg.full_tasks = 4;
        cfg.modes = vec![0, 3, 6];
        cfg.greedy_work = 100_000;
        cfg.rotation_work = 100_000;
        cfg.rotation_depth = 32;
        cfg.patch_work = 500_000;
        cfg.max_interface = 1536;
        cfg.metis_tasks = 1;
        cfg.natives = 1;
        cfg.atoms = true;
        cfg.max_atoms = 256;
        if let Some(order) = recovery_separator::recovered(pattern, &best, &cfg) {
            if is_bijection(&order, n) {
                let cost = flops_of(&sp, &order);
                if cost < best_cost {
                    best_cost = cost;
                    best.clone_from(&order);
                }
                if !donors.contains(&order) {
                    donors.push(order);
                }
            }
        }
    }
    if donors.len() > 1 && n * donors.len() <= 150_000 {
        let inputs: Vec<_> = donors.iter().map(Vec::as_slice).collect();
        let mut pool = decomposition::WholePool::new(&sp);
        for order in [pool.combine(&inputs), decomposition::prefix_pool(&sp, &inputs)]
            .into_iter().flatten()
        {
            if is_bijection(&order, n) {
                let cost = flops_of(&sp, &order);
                if cost < best_cost {
                    best_cost = cost;
                    best = order;
                }
            }
        }
    }
    best
}
