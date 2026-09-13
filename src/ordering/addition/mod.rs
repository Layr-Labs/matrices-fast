use super::{flops_of, is_bijection, ScoringPattern};
use crate::Pattern;

mod joint;
mod parallel;
mod patch;
mod promotions;
mod quotient;
mod recovery_separator;

fn etree_prep(sp: &ScoringPattern, order: &[usize]) -> (Vec<usize>, Vec<u32>, Vec<i32>) {
    let (counts, parent, post) = super::symbolic_flat::analyze(sp, order);
    let mut rank = vec![0; sp.n];
    for (i, &v) in post.iter().enumerate() {
        rank[v] = i;
    }
    (
        post.iter().map(|&v| order[v]).collect(),
        post.iter().map(|&v| counts[v] as u32).collect(),
        post.iter().map(|&v| {
            if parent[v] == usize::MAX { -1 } else { rank[parent[v]] as i32 }
        }).collect(),
    )
}

pub(super) fn refine(pattern: &Pattern, baseline: Vec<usize>) -> Vec<usize> {
    let n = pattern.n;
    if !(64..=320_000).contains(&n) || pattern.nnz() > 1_500_000 {
        return baseline;
    }
    let mut work = 64_000_000usize;
    // Reserve both global symbolic analyses and their array preparation.
    if recovery_separator::charge(&mut work, 16 * (n + pattern.nnz())).is_none() {
        return baseline;
    }
    let sp = ScoringPattern {
        n,
        col_ptr: pattern.col_ptr.clone(),
        row_idx: pattern.row_idx.clone(),
    };
    let prepared = etree_prep(&sp, &baseline);
    let entries: usize = prepared.1.iter().map(|&c| c as usize).sum();
    let cost: u64 = prepared.1.iter().map(|&c| (c as u64).pow(2)).sum();
    if entries + n + pattern.nnz() > 3_000_000 || cost < 32 * n as u64 {
        return baseline;
    }
    let Some(tree) = recovery_separator::Tree::from_prepared(pattern, prepared, &mut work) else {
        return baseline;
    };
    let mut proposals: Vec<Vec<recovery_separator::Proposal>> = (0..n).map(|_| Vec::new()).collect();
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
        cfg.patch_work = 300_000;
        cfg.max_interface = 1536;
        cfg.metis_tasks = 1;
        cfg.natives = 1;
        cfg.atoms = true;
        cfg.max_atoms = 64;
        let mut allowance = if directed { work } else { work / 2 };
        work -= allowance;
        recovery_separator::propose_regions(&tree, &cfg, &mut allowance, &mut proposals);
        work += allowance;
    }
    let Some(candidate) = recovery_separator::assemble(&tree, &proposals) else {
        return baseline;
    };
    if is_bijection(&candidate, n) && flops_of(&sp, &candidate) < cost {
        candidate
    } else {
        baseline
    }
}
