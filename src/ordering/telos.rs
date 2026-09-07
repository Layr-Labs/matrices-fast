//! Bounded completion-gradient descent.
//!
//! For a fixed chordal completion the multiset of symbolic column counts is
//! invariant across its perfect elimination orders, and the objective is the
//! sum of their squares.  The current counts therefore provide a deterministic
//! ranking of vertices whose treatment matters most.  A round tries two kinds
//! of moves from one immutable snapshot: remove a ranked prefix, order the
//! induced remainder, and append the prefix; or simply splice that prefix to
//! the tail.  Every candidate is checked by the exact scorer and only a strict
//! improvement can leave the round.

use super::*;

// Bound both dimensions of the linear-work candidate family. A nonzero-only
// gate would admit a hidden pattern with many isolated vertices, multiplying
// every exact score and induced-subgraph allocation by an unbounded `n`.
const MAX_N: usize = 30_000;
const MAX_NNZ: usize = 400_000;
#[cfg(test)]
const TIER_A_MAX_NNZ: usize = 150_000;
#[cfg(test)]
const BUDGET_A: usize = 700_000;
#[cfg(test)]
const MIN_PASSES_A: usize = 3;
#[cfg(test)]
const MAX_PASSES_A: usize = 16;

pub(super) fn enabled(n: usize, nnz: usize) -> bool {
    (16..=MAX_N).contains(&n) && nnz > 0 && nnz < MAX_NNZ
}

#[derive(Clone, Copy)]
enum SubOrder {
    #[cfg(test)]
    Amd,
    NoDenseAmd,
    Amf5,
}

#[cfg(test)]
const PRIO_A: [(SubOrder, usize); 16] = [
    (SubOrder::Amf5, 1),
    (SubOrder::NoDenseAmd, 2),
    (SubOrder::Amf5, 4),
    (SubOrder::Amd, 1),
    (SubOrder::Amf5, 12),
    (SubOrder::NoDenseAmd, 6),
    (SubOrder::Amf5, 24),
    (SubOrder::Amd, 4),
    (SubOrder::NoDenseAmd, 12),
    (SubOrder::Amf5, 2),
    (SubOrder::Amd, 16),
    (SubOrder::NoDenseAmd, 8),
    (SubOrder::Amf5, 32),
    (SubOrder::Amd, 96),
    (SubOrder::Amf5, 6),
    (SubOrder::Amf5, 16),
];

const PRIO_B: [(SubOrder, usize); 2] = [(SubOrder::Amf5, 1), (SubOrder::NoDenseAmd, 6)];

#[cfg(test)]
const PEEL_A: [usize; 15] = [1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192];
const PEEL_B: [usize; 3] = [1, 2, 8];

fn strip(
    sp: &ScoringPattern,
    ranked: &[usize],
    k: usize,
    sub_order: SubOrder,
) -> Option<Vec<usize>> {
    let n = sp.n;
    if k == 0 || k >= n {
        return None;
    }

    let mut removed = vec![false; n];
    for &v in &ranked[..k] {
        removed[v] = true;
    }

    let mut new_id = vec![usize::MAX; n];
    let mut old_id = Vec::with_capacity(n - k);
    for v in 0..n {
        if !removed[v] {
            new_id[v] = old_id.len();
            old_id.push(v);
        }
    }

    let m = old_id.len();
    let mut col_ptr = Vec::with_capacity(m + 1);
    let mut row_idx = Vec::new();
    col_ptr.push(0i32);
    for &v in &old_id {
        for p in sp.col_ptr[v]..sp.col_ptr[v + 1] {
            let w = sp.row_idx[p];
            if !removed[w] {
                row_idx.push(new_id[w] as i32);
            }
        }
        col_ptr.push(row_idx.len() as i32);
    }

    let core = feral_ordering_core::CscPattern::new(m, &col_ptr, &row_idx)?;
    let local = match sub_order {
        #[cfg(test)]
        SubOrder::Amd => feral_amd::amd_order(&core).ok()?,
        SubOrder::NoDenseAmd => {
            let options = feral_amd::AmdOptions {
                aggressive: false,
                dense_alpha: -1.0,
            };
            feral_amd::amd_order_opts(&core, &options).ok()?.0
        }
        SubOrder::Amf5 => {
            let options = feral_amf::AmfOptions {
                dense_alpha: 5.0,
                ..Default::default()
            };
            feral_amf::amf_order_opts(&core, &options).ok()?.0
        }
    };

    let mut out: Vec<usize> = local.into_iter().map(|v| old_id[v as usize]).collect();
    let mut tail = ranked[..k].to_vec();
    tail.sort_unstable_by_key(|&v| (sp.col_ptr[v + 1] - sp.col_ptr[v], v));
    out.extend(tail);
    Some(out)
}

fn descend(
    sp: &ScoringPattern,
    nnz: usize,
    start: &[usize],
    start_flops: u64,
    max_rounds: usize,
    priorities: &[(SubOrder, usize)],
    passes: usize,
    peels: &[usize],
) -> (Vec<usize>, u64) {
    let n = sp.n;
    let mut current = start.to_vec();
    let mut current_flops = start_flops;

    for _ in 0..max_rounds {
        let pp = permute_pattern(sp, &current);
        let etree = EliminationTree::from_pattern(&pp);
        let counts = column_counts_gnp(&pp, &etree);
        let mut count_by_vertex = vec![0usize; n];
        for (pos, &v) in current.iter().enumerate() {
            count_by_vertex[v] = counts[pos];
        }
        let mut ranked: Vec<usize> = (0..n).collect();
        ranked
            .sort_unstable_by(|&a, &b| count_by_vertex[b].cmp(&count_by_vertex[a]).then(a.cmp(&b)));

        let base = current.clone();
        let ranked_ref = &ranked;
        let base_ref = &base;
        let mut tasks: Vec<parallel::PermFn<'_>> = Vec::new();
        for &(sub_order, k) in priorities.iter().take(passes) {
            if k < n {
                tasks.push(Box::new(move || strip(sp, ranked_ref, k, sub_order)));
            }
        }
        for &k in peels {
            if k >= n {
                continue;
            }
            tasks.push(Box::new(move || {
                let mut deferred = vec![false; n];
                for &v in &ranked_ref[..k] {
                    deferred[v] = true;
                }
                let mut candidate = Vec::with_capacity(n);
                candidate.extend(base_ref.iter().copied().filter(|&v| !deferred[v]));
                let mut tail = ranked_ref[..k].to_vec();
                tail.sort_unstable_by_key(|&v| (sp.col_ptr[v + 1] - sp.col_ptr[v], v));
                candidate.extend(tail);
                Some(candidate)
            }));
        }

        let mut results = parallel::run_perms(&tasks, sp, n, nnz, current_flops);
        if !parallel::accept(&mut results, &mut current_flops, &mut current) {
            break;
        }
    }
    (current, current_flops)
}

#[cfg(test)]
fn main_schedule(nnz: usize) -> (usize, &'static [(SubOrder, usize)], usize, &'static [usize]) {
    if nnz < TIER_A_MAX_NNZ {
        (
            2,
            &PRIO_A,
            (BUDGET_A / nnz.max(1)).clamp(MIN_PASSES_A, MAX_PASSES_A),
            &PEEL_A,
        )
    } else {
        (1, &PRIO_B, PRIO_B.len(), &PEEL_B)
    }
}

#[cfg(test)]
fn extra_seed_gate(n: usize, nnz: usize, max_degree: usize) -> bool {
    nnz <= 20_000
        || ((20_000 < nnz && nnz <= 55_000) && max_degree.saturating_mul(50) <= n)
}

/// One small round on the current leader: two induced suborders and three
/// simple peels. The same finite schedule applies throughout the structural
/// gate, with no extra-seed descent and no earned continuation.
pub(super) fn refine(
    sp: &ScoringPattern,
    nnz: usize,
    leader: &[usize],
    leader_flops: u64,
) -> (Vec<usize>, u64) {
    if !enabled(sp.n, nnz) {
        return (leader.to_vec(), leader_flops);
    }
    descend(sp, nnz, leader, leader_flops, 1, &PRIO_B, PRIO_B.len(), &PEEL_B)
}

#[cfg(test)]
pub(super) fn refine_for_probe(
    sp: &ScoringPattern,
    nnz: usize,
    max_degree: usize,
    leader: &[usize],
    leader_flops: u64,
    runner_up: &[(u64, Vec<usize>)],
) -> (Vec<usize>, u64) {
    if super::probe::terminal_phase_enabled(8) {
        refine_legacy(sp, nnz, max_degree, leader, leader_flops, runner_up)
    } else {
        refine(sp, nnz, leader, leader_flops)
    }
}

/// Test-only reference for matched whole-pipeline cost/score ablation.
#[cfg(test)]
fn refine_legacy(
    sp: &ScoringPattern,
    nnz: usize,
    max_degree: usize,
    leader: &[usize],
    leader_flops: u64,
    runner_up: &[(u64, Vec<usize>)],
) -> (Vec<usize>, u64) {
    if !enabled(sp.n, nnz) {
        return (leader.to_vec(), leader_flops);
    }

    let (rounds, priorities, passes, peels) = main_schedule(nnz);
    let (mut best, mut best_flops) = descend(
        sp,
        nnz,
        leader,
        leader_flops,
        rounds,
        priorities,
        passes,
        peels,
    );

    if extra_seed_gate(sp.n, nnz, max_degree) {
        for (seed_flops, seed) in runner_up
            .iter()
            .filter(|(_, seed)| seed.as_slice() != leader)
            .take(2)
        {
            let (candidate, candidate_flops) = descend(
                sp,
                nnz,
                seed,
                *seed_flops,
                1,
                &PRIO_B,
                PRIO_B.len(),
                &PEEL_B,
            );
            if candidate_flops < best_flops {
                best = candidate;
                best_flops = candidate_flops;
            }
        }
    }
    (best, best_flops)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_gate_bounds_both_input_dimensions() {
        assert!(!enabled(15, 20));
        assert!(enabled(16, 20));
        assert!(!enabled(MAX_N + 1, 20));
        assert!(enabled(MAX_N, MAX_NNZ - 1));
        assert!(!enabled(16, MAX_NNZ));
        assert!(!enabled(16, 0));
    }

    #[test]
    fn strip_returns_a_bijection() {
        let p = Pattern::from_edges(8, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7)]);
        let sp = ScoringPattern {
            n: p.n,
            col_ptr: p.col_ptr.clone(),
            row_idx: p.row_idx.clone(),
        };
        let ranked: Vec<usize> = (0..8).rev().collect();
        let out = strip(&sp, &ranked, 2, SubOrder::Amd).unwrap();
        assert!(is_bijection(&out, 8));
    }
}
