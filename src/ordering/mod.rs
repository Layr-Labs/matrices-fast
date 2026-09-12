//! Deterministic seed, residual and separator search with exact symbolic acceptance.
use crate::Pattern;

mod advancements;
mod completion;
mod decomposition;
mod joint;
mod ladder;
mod linegraph;
mod parallel;
mod patch;
mod promotions;
mod quality;
mod recovery_regions;
mod recovery_separator;
mod rgreedy;
mod separator;

mod core_search;
mod graph_orderers;
mod independent;
mod local_search;
mod policy;
mod score;
mod seeds;
mod subtrees;

use feral::ordering::elimination_tree::EliminationTree;
use feral::sparse::csc::CscPattern as ScoringPattern;

use score::*;

struct Candidate {
    perm: Vec<usize>,
    cost: u64,
}

impl Candidate {
    fn consider(&mut self, scoring: &ScoringPattern, candidate: Option<Vec<usize>>) -> bool {
        let before = self.cost;
        accept_if_cheaper(
            candidate,
            scoring.n,
            scoring,
            &mut self.perm,
            &mut self.cost,
        );
        self.cost < before
    }
}

pub fn order(pattern: &Pattern) -> Vec<usize> {
    let n = pattern.n;
    if n == 0 {
        return Vec::new();
    }
    let col_ptr: Vec<i32> = pattern
        .col_ptr
        .iter()
        .map(|&v| i32::try_from(v).expect("column pointer exceeds i32"))
        .collect();
    let row_idx: Vec<i32> = pattern
        .row_idx
        .iter()
        .map(|&v| i32::try_from(v).expect("row index exceeds i32"))
        .collect();
    let core =
        feral_ordering_core::CscPattern::new(n, &col_ptr, &row_idx).expect("invalid pattern");
    let scoring = ScoringPattern {
        n,
        col_ptr: pattern.col_ptr.clone(),
        row_idx: pattern.row_idx.clone(),
    };
    let perm = feral_amd::amd_order(&core)
        .expect("AMD ordering failed")
        .into_iter()
        .map(|v| v as usize)
        .collect::<Vec<_>>();
    let (amd_cost, factor_entries) = cost_and_entries(&scoring, &perm);
    if factor_entries == n + pattern.nnz() / 2 {
        return perm;
    }
    let mut best = Candidate {
        perm,
        cost: amd_cost,
    };

    seeds::refine(pattern, &core, &scoring, &mut best);
    let quality_enabled = quality::admits(pattern, amd_cost);
    let donors = if quality_enabled {
        vec![best.perm.clone()]
    } else {
        Vec::new()
    };
    let residual = core_search::refine(pattern, &scoring, &mut best);
    let quality_seeds = quality_enabled.then_some(quality::Seeds {
        core: residual,
        donors,
    });
    local_search::refine(pattern, &scoring, &mut best, amd_cost);
    subtrees::refine(pattern, &scoring, &mut best, amd_cost);
    local_search::cleanup(pattern, &scoring, &mut best);
    if let Some(seeds) = quality_seeds {
        if n < 10_000 {
            advancements::finish(pattern, &scoring, &mut best);
            quality::refine_small(pattern, &scoring, &mut best);
        }
        if n < 10_000 || best.cost < amd_cost {
            quality::bounded(pattern, &scoring, &mut best, seeds);
        }
    } else {
        advancements::finish(pattern, &scoring, &mut best);
    }
    quality::refine_small(pattern, &scoring, &mut best);
    independent::refine(pattern, &scoring, &mut best);
    best.perm
}
