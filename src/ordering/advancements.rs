//! Residual-core candidates and exact completion refinements.
use super::*;

pub(super) fn finish(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) {
    best.consider(scoring, separator::refine(pattern, &best.perm));
    let mut core = Candidate {
        perm: best.perm.clone(),
        cost: best.cost,
    };
    extra_cores(pattern, scoring, &mut core.perm, &mut core.cost);
    if core.cost.saturating_mul(100) < best.cost.saturating_mul(95) {
        std::thread::scope(|scope| {
            let primary = scope.spawn(|| refine_branch(pattern, scoring, best));
            refine_branch(pattern, scoring, &mut core);
            primary.join().unwrap();
        });
    } else {
        refine_branch(pattern, scoring, best);
    }
    if core.cost < best.cost {
        std::mem::swap(best, &mut core);
    }
    refine_core_regions(pattern, scoring, &mut best.perm, &mut best.cost);
    best.consider(
        scoring,
        recovery_regions::refine_legacy(pattern, &best.perm),
    );
    best.consider(
        scoring,
        recovery_regions::crossover_legacy(pattern, &best.perm, &core.perm),
    );
}

fn refine_branch(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) {
    complete(pattern, scoring, &mut best.perm, &mut best.cost);
    best.consider(scoring, separator::refine_large(pattern, &best.perm));
    best.consider(scoring, promotions::refine(pattern, &best.perm));
}

pub(super) fn extra_cores(
    pattern: &Pattern,
    sp: &ScoringPattern,
    best: &mut Vec<usize>,
    cost: &mut u64,
) {
    let n = pattern.n;
    if !(10_000..=100_000).contains(&n) || pattern.nnz() > 400_000 {
        return;
    }
    let mut ladder = ladder::CoreLadder::new(n, &pattern.col_ptr, &pattern.row_idx, 12);
    let mut previous_size = n;
    for threshold in [2, 3, 4, 5, 6, 12] {
        ladder.advance(threshold, 3_000_000);
        let core = ladder.export();
        let k = core.ids.len();
        if k == 0 {
            accept_if_cheaper(Some(core.prefix), n, sp, best, cost);
            break;
        }
        if k * 100 > previous_size * 95 {
            continue;
        }
        previous_size = k;
        if k > 12_000 || core.row_idx.len() > 600_000 {
            continue;
        }
        let cp: Vec<i32> = core.col_ptr.iter().map(|&v| v as i32).collect();
        let ri: Vec<i32> = core.row_idx.iter().map(|&v| v as i32).collect();
        let Some(graph) = feral_ordering_core::CscPattern::new(k, &cp, &ri) else {
            continue;
        };
        let candidates = core_candidates(&graph);
        let orders: Vec<Vec<usize>> = candidates
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|order| {
                let order: Vec<usize> = order.into_iter().map(|v| v as usize).collect();
                is_bijection(&order, k).then_some(order)
            })
            .collect();
        let residual = ScoringPattern {
            n: k,
            col_ptr: core.col_ptr.clone(),
            row_idx: core.row_idx.clone(),
        };
        // Every candidate has the same exact eliminated prefix. Rank on the
        // residual and replay only the winner on the original graph, rather
        // than rebuilding the full symbolic factor four times per threshold.
        let scores = parallel::map_indexed(&orders, 4, |_, order| {
            core.prefix_flops + flops_of(&residual, order)
        });
        let mut selected = None;
        let mut least = *cost;
        for (index, &value) in scores.iter().enumerate() {
            if value < least {
                least = value;
                selected = Some(index);
            }
        }
        if let Some(index) = selected {
            let mut full = core.prefix.clone();
            full.extend(orders[index].iter().map(|&v| core.ids[v]));
            accept_if_cheaper(Some(full), n, sp, best, cost);
        }
    }
}

fn core_candidates(
    graph: &feral_ordering_core::CscPattern<'_>,
) -> Vec<Result<Vec<i32>, feral_ordering_core::OrderingError>> {
    parallel::map_indexed(&[0, 1], 2, |_, &source| {
        if source == 0 {
            feral_amf::amf_order_opts(graph, &feral_amf::AmfOptions::default()).map(|(p, ..)| p)
        } else {
            let options = feral_metis::MetisOptions {
                niparts: 4,
                nd_to_amd_switch: 100,
                ..Default::default()
            };
            feral_metis::metis_order_full(graph, &options).map(|(p, ..)| p)
        }
    })
}

fn column_counts(sp: &ScoringPattern, order: &[usize]) -> Vec<u32> {
    let permuted = permute_pattern(sp, order);
    let tree = EliminationTree::from_pattern(&permuted);
    symbolic_counts(&permuted, &tree)
        .1
        .into_iter()
        .map(|c| c as u32)
        .collect()
}

fn accept_batch(
    candidates: Vec<Vec<usize>>,
    sp: &ScoringPattern,
    best: &mut Vec<usize>,
    cost: &mut u64,
) {
    for order in candidates {
        accept_if_cheaper(Some(order), sp.n, sp, best, cost);
    }
}

pub(super) fn complete(
    pattern: &Pattern,
    sp: &ScoringPattern,
    best: &mut Vec<usize>,
    cost: &mut u64,
) {
    let n = pattern.n;
    if n > 100_000 {
        return;
    }
    let limits = completion::CompletionLimits {
        max_n: 100_000,
        max_input_nnz: 1_500_000,
        max_lnnz: 1_500_000,
    };
    let mut remaining = 2_000_000usize;
    for round in 0..8 {
        let counts = column_counts(sp, best);
        let fill = counts.iter().map(|&c| c as usize).sum::<usize>();
        let charge = fill + 2 * (n + pattern.nnz());
        if fill > limits.max_lnnz || (round > 0 && charge > remaining) {
            break;
        }
        remaining = remaining.saturating_sub(charge);
        let before = *cost;
        if let Some(candidates) = completion::peo_candidates(
            n,
            &pattern.col_ptr,
            &pattern.row_idx,
            best,
            &counts,
            &limits,
        ) {
            accept_batch(candidates, sp, best, cost);
        }
        if *cost == before {
            break;
        }
    }
    if n > 30_000 {
        return;
    }
    let counts = column_counts(sp, best);
    let limits = completion::CompletionLimits {
        max_n: 30_000,
        max_input_nnz: 500_000,
        max_lnnz: 300_000,
    };
    let mut reward = 0;
    if let Some(candidates) = completion::minimalized_peo_candidates(
        n,
        &pattern.col_ptr,
        &pattern.row_idx,
        best,
        &counts,
        &limits,
        2_000_000,
        &mut reward,
    ) {
        accept_batch(candidates, sp, best, cost);
    }
    if n <= 1_200 {
        let counts = column_counts(sp, best);
        if let Some(candidates) =
            completion::flip_candidates(n, &pattern.col_ptr, &pattern.row_idx, best, &counts, n)
        {
            accept_batch(candidates, sp, best, cost);
        }
    }
}

/// Repeat separator refinement on an exact residual, retaining its fixed prefix.
pub(super) fn refine_core_regions(
    pattern: &Pattern,
    sp: &ScoringPattern,
    best: &mut Vec<usize>,
    cost: &mut u64,
) {
    let n = pattern.n;
    if !(10_000..=80_000).contains(&n) || pattern.nnz() > 500_000 {
        return;
    }
    let mut ladder = ladder::CoreLadder::new(n, &pattern.col_ptr, &pattern.row_idx, 3);
    ladder.advance(3, 3_000_000);
    let core = ladder.export();
    let k = core.ids.len();
    if !(1000..=12000).contains(&k) || k * 10 > n * 9 {
        return;
    }
    let mut inv = vec![usize::MAX; n];
    for (j, &v) in core.ids.iter().enumerate() {
        inv[v] = j;
    }
    let mut p: Vec<usize> = best
        .iter()
        .map(|&v| inv[v])
        .filter(|&v| v != usize::MAX)
        .collect();
    let local = Pattern {
        n: k,
        col_ptr: core.col_ptr,
        row_idx: core.row_idx,
    };
    let local_sp = ScoringPattern {
        n: k,
        col_ptr: local.col_ptr.clone(),
        row_idx: local.row_idx.clone(),
    };
    let mut local_cost = flops_of(&local_sp, &p);
    if (core.prefix_flops + local_cost).saturating_mul(100) > cost.saturating_mul(110) {
        return;
    }
    accept_if_cheaper(
        separator::refine_core(&local, &p),
        k,
        &local_sp,
        &mut p,
        &mut local_cost,
    );
    let mut full = core.prefix;
    full.extend(p.iter().map(|&v| core.ids[v]));
    accept_if_cheaper(Some(full), n, sp, best, cost);
}
