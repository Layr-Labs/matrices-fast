//! Deterministic elimination ordering with full symbolic candidate checks.
//!
//! The scheduler composes seed generation, exact residual search, local moves
//! and separator/region refinement. Kernels do not choose the outer schedule.
//! All work limits and random seeds are deterministic; strict acceptance keeps
//! the first order on equal costs. See policy.rs for the fixed search limits.
use crate::Pattern;

mod advancements;
mod bags;
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
#[cfg(test)]
mod regions;
mod rgreedy;
mod separator;

mod core_search;
mod finish;
mod graph_orderers;
mod independent;
mod local_search;
mod policy;
mod score;
mod seeds;
mod subtrees;

use feral::ordering::elimination_tree::EliminationTree;
use feral::sparse::csc::CscPattern as ScoringPattern;
#[cfg(test)]
use feral::symbolic::column_counts_gnp;

use score::*;

#[cfg(test)]
use graph_orderers::*;
#[cfg(test)]
use policy::*;
#[cfg(test)]
use seeds::{perturb, relabel, relabel_restarts};

/// An incumbent and its full symbolic cost, updated together.
struct Candidate {
    perm: Vec<usize>,
    cost: u64,
}

impl Candidate {
    /// Strict acceptance preserves the first permutation when costs tie.
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

/// Produce a permutation from the graph alone. Search-policy changes must be
/// checked separately from refactors against the real corpus and time limit.
pub fn order(pattern: &Pattern) -> Vec<usize> {
    #[cfg(test)]
    let mut mark = stage_timer("ORDER_STAGE");
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
    #[cfg(test)] mark("amd");

    seeds::refine(pattern, &core, &scoring, &mut best);
    #[cfg(test)] mark("seeds");
    let quality_enabled = quality::admits(pattern, amd_cost);
    let donors = if quality_enabled { vec![best.perm.clone()] } else { Vec::new() };
    let residual = core_search::refine(pattern, &scoring, &mut best);
    let quality_seeds = quality_enabled.then_some(quality::Seeds { core: residual, donors });
    #[cfg(test)] mark("cores");
    local_search::refine(pattern, &scoring, &mut best, amd_cost);
    #[cfg(test)] mark("local");
    subtrees::refine(pattern, &scoring, &mut best, amd_cost);
    #[cfg(test)] mark("subtrees");
    local_search::cleanup(pattern, &scoring, &mut best);
    #[cfg(test)] mark("cleanup");
    if let Some(seeds) = quality_seeds {
        if n < 10_000 {
            finish::refine(pattern, &scoring, &mut best);
            quality::refine_small(pattern, &scoring, &mut best);
        }
        if n < 10_000 || best.cost < amd_cost {
            quality::bounded(pattern, &scoring, &mut best, seeds);
        }
    } else {
        finish::refine(pattern, &scoring, &mut best);
    }
    #[cfg(test)] mark("finish");
    quality::refine_small(pattern, &scoring, &mut best);
    #[cfg(test)] mark("small");
    independent::refine(pattern, &scoring, &mut best);
    #[cfg(test)] mark("independent");
    best.perm
}

#[cfg(test)]
fn stage_timer(prefix: &'static str) -> impl FnMut(&str) {
    let mut start = std::time::Instant::now();
    move |stage| {
        eprintln!("{prefix} {stage} {:.6}", start.elapsed().as_secs_f64());
        start = std::time::Instant::now();
    }
}

#[cfg(test)]
mod tests;
