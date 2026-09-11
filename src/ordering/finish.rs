//! Separator, completion and region refinements of the legacy incumbent.
use super::*;

/// Refine the incumbent and an alternate core branch, then combine their regions.
pub(super) fn refine(
    pattern: &Pattern,
    scoring_pat: &ScoringPattern,
    best: &mut Candidate,
) {
    #[cfg(test)]
    let mut mark = stage_timer("FINISH_STAGE");
    best.consider(scoring_pat, separator::refine(pattern, &best.perm));
    #[cfg(test)] mark("separator");
    let mut core = Candidate { perm: best.perm.clone(), cost: best.cost };
    advancements::extra_cores(
        pattern,
        scoring_pat,
        &mut core.perm,
        &mut core.cost,
    );
    #[cfg(test)] mark("extra_cores");
    let alternate = core.cost.saturating_mul(100) < best.cost.saturating_mul(95);
    if alternate {
        // Both branches start from fixed orders; their searches are independent.
        std::thread::scope(|scope| {
            let primary = scope.spawn(|| refine_branch(pattern, scoring_pat, best));
            refine_branch(pattern, scoring_pat, &mut core);
            primary.join().unwrap();
        });
    } else {
        refine_branch(pattern, scoring_pat, best);
    }
    if core.cost < best.cost {
        std::mem::swap(best, &mut core);
    }
    #[cfg(test)] mark("branches");
    advancements::refine_core_regions(pattern, scoring_pat, &mut best.perm, &mut best.cost);
    #[cfg(test)] mark("core_regions");
    best.consider(scoring_pat, recovery_regions::refine_legacy(pattern, &best.perm));
    #[cfg(test)] mark("regions");
    best.consider(scoring_pat, recovery_regions::crossover_legacy(pattern, &best.perm, &core.perm));
    #[cfg(test)] mark("crossover");
}

fn refine_branch(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) {
    advancements::complete(pattern, scoring, &mut best.perm, &mut best.cost);
    best.consider(scoring, separator::refine_large(pattern, &best.perm));
    best.consider(scoring, promotions::refine(pattern, &best.perm));
}
