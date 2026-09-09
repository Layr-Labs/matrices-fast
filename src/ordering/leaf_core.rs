//! Certified pendant-forest prefix and induced residual 2-core.
//!
//! Repeatedly eliminating a vertex of live degree at most one introduces no
//! fill. More strongly, moving such a vertex to the front cannot increase the
//! FLOP objective of any ordering. This also holds for any explicitly verified
//! simplicial vertex. A non-neighbor commutes with it. If a simplicial vertex
//! `v` of degree `a` is swapped ahead of a neighboring predecessor `w` of
//! degree `d`, then `N[v]` is a subset of `N[w]`, so `d >= a`; the two costs
//! change from `(d + 1)^2 + d^2` to `(a + 1)^2 + d^2`. Applying that exchange
//! repeatedly proves that, for every full ordering `p`,
//!
//! ```text
//!   FLOPs(prefix ++ restrict(p, core)) <= FLOPs(p).
//! ```
//!
//! Since the prefix adds no fill, the residual pattern is induced (the 2-core
//! for degree-one peeling), and every lifted core ordering has the exact split
//!
//! ```text
//!   FLOPs(prefix ++ core_order) =
//!       prefix_flops + FLOPs(induced_core, core_order).
//! ```
//!
//! The production admission constants deliberately describe only the bounded
//! experiment. They are structural and deterministic; callers should still
//! retain their incumbent and accept a refined lift only on a strict trusted
//! full-pattern score improvement.

use crate::Pattern as ScoringPattern;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// Fixed work allowance for the caller's existing core-window refinement.
pub(crate) const CORE_WINDOW_BUDGET: i64 = 32_000_000;
pub(crate) const MAX_CORE_N: usize = 12_000;
/// Directed off-diagonal nonzeros, matching `Pattern::nnz()`.
pub(crate) const MAX_CORE_NNZ: usize = 200_000;

const MIN_REMOVAL_NUMERATOR: usize = 1;
const MIN_REMOVAL_DENOMINATOR: usize = 5;
const NONE: usize = usize::MAX;
const PAIR_CHECK_BUDGET: u64 = 500_000;

#[cfg(test)]
thread_local! {
    static TEST_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

#[cfg(test)]
pub(crate) fn enabled() -> bool {
    TEST_ENABLED.with(|cell| cell.get())
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) const fn enabled() -> bool {
    true
}

pub(crate) fn refine(pattern: &crate::Pattern, incumbent: &[usize]) -> Option<(Vec<usize>, u64)> {
    let core = reduce_simplicial(pattern, 2, PAIR_CHECK_BUDGET)?;
    let mut core_seed = core.restrict(incumbent)?;
    let n = core.core_n();
    let nnz = core.core_nnz();
    let scoring = super::ScoringPattern {
        n,
        col_ptr: core.core_col_ptr.clone(),
        row_idx: core.core_row_idx.clone(),
    };
    let mut workspace = super::scoring_ws::ScoreWorkspace::new(n, nnz);
    let mut best_flops = workspace.flops(&scoring, &core_seed);

    // Every component of a simple 2-regular graph is a cycle; all elimination
    // orders have the same column counts up to interleaving components.
    if !core.core_col_ptr.windows(2).all(|p| p[1] - p[0] == 2) {
        let cp = &core.core_col_ptr;
        let ri = &core.core_row_idx;
        let seed = &core_seed;
        let tasks: Vec<super::parallel::PermFn<'_>> = [(8, 32_000_000), (10, 48_000_000)]
            .into_iter()
            .map(|(width, budget)| {
                let task: super::parallel::PermFn<'_> = Box::new(move || {
                    super::rgreedy::subset_window_descent_step(n, cp, ri, seed, width, 4, 3, budget)
                });
                task
            })
            .collect();
        let mut results = super::parallel::run_perms(&tasks, &scoring, n, nnz, best_flops);
        drop(tasks);
        super::parallel::accept(&mut results, &mut best_flops, &mut core_seed);
    }
    let predicted = core.prefix_flops.checked_add(best_flops)?;
    Some((core.splice(&core_seed)?, predicted))
}

/// A certified no-fill prefix and the untouched induced residual graph.
pub(crate) struct LeafCore {
    original_n: usize,
    /// Original vertex IDs in deterministic leaf-peeling order.
    pub(crate) prefix: Vec<usize>,
    /// Core-local ID to original ID, ascending by original ID.
    pub(crate) core_ids: Vec<usize>,
    pub(crate) core_col_ptr: Vec<usize>,
    pub(crate) core_row_idx: Vec<usize>,
    /// Sum of prefix column-count squares; leaf-only terms are one or four.
    pub(crate) prefix_flops: u64,
}

impl LeafCore {
    #[inline]
    pub(crate) fn core_n(&self) -> usize {
        self.core_ids.len()
    }

    #[inline]
    pub(crate) fn core_nnz(&self) -> usize {
        self.core_row_idx.len()
    }

    /// Restrict a full ordering to the core while preserving relative order.
    ///
    /// The result uses core-local IDs, ready for core scoring or refinement.
    /// Malformed full permutations fail closed.
    pub(crate) fn restrict(&self, full_perm: &[usize]) -> Option<Vec<usize>> {
        if full_perm.len() != self.original_n {
            return None;
        }
        let mut local_of = vec![NONE; self.original_n];
        for (local, &original) in self.core_ids.iter().enumerate() {
            local_of[original] = local;
        }
        let mut seen = vec![false; self.original_n];
        let mut core_perm = Vec::with_capacity(self.core_n());
        for &v in full_perm {
            if v >= self.original_n || seen[v] {
                return None;
            }
            seen[v] = true;
            let local = local_of[v];
            if local != NONE {
                core_perm.push(local);
            }
        }
        (core_perm.len() == self.core_n()).then_some(core_perm)
    }

    /// Expand a core-local ordering after the certified prefix.
    ///
    /// Malformed core permutations fail closed.
    pub(crate) fn splice(&self, core_perm: &[usize]) -> Option<Vec<usize>> {
        if core_perm.len() != self.core_n() {
            return None;
        }
        let mut seen = vec![false; self.core_n()];
        let mut full = Vec::with_capacity(self.original_n);
        full.extend_from_slice(&self.prefix);
        for &local in core_perm {
            if local >= self.core_n() || seen[local] {
                return None;
            }
            seen[local] = true;
            full.push(self.core_ids[local]);
        }
        (full.len() == self.original_n).then_some(full)
    }

    /// Certified non-worsening transformation of an arbitrary incumbent.
    pub(crate) fn lift_restricted(&self, full_perm: &[usize]) -> Option<Vec<usize>> {
        let core_perm = self.restrict(full_perm)?;
        self.splice(&core_perm)
    }
}

/// Peel pendant forests and return the induced 2-core when the fixed
/// experimental admission gates are met.
pub(crate) fn reduce(sp: &ScoringPattern) -> Option<LeafCore> {
    reduce_with_limits(
        sp,
        MIN_REMOVAL_NUMERATOR,
        MIN_REMOVAL_DENOMINATOR,
        MAX_CORE_N,
        MAX_CORE_NNZ,
    )
}

/// Experimental extension of [`reduce`] that also removes low-degree
/// simplicial vertices. `max_degree` must be two or three. Every live-neighbor
/// pair is checked in the original sorted CSC before a pivot is accepted, so
/// the prefix remains a certified no-fill prefix.
///
/// A rejected vertex is retried only after one of its neighbors is removed.
/// `max_pair_checks` bounds all binary-search adjacency tests, including failed
/// simpliciality tests; exhaustion declines the complete candidate.
pub(crate) fn reduce_simplicial(
    sp: &ScoringPattern,
    max_degree: usize,
    max_pair_checks: u64,
) -> Option<LeafCore> {
    reduce_simplicial_with_limits(
        sp,
        max_degree,
        max_pair_checks,
        MIN_REMOVAL_NUMERATOR,
        MIN_REMOVAL_DENOMINATOR,
        MAX_CORE_N,
        MAX_CORE_NNZ,
    )
}

fn reduce_with_limits(
    sp: &ScoringPattern,
    min_removed_numerator: usize,
    min_removed_denominator: usize,
    max_core_n: usize,
    max_core_nnz: usize,
) -> Option<LeafCore> {
    let n = sp.n;
    if n == 0
        || min_removed_denominator == 0
        || sp.col_ptr.len() != n.checked_add(1)?
        || sp.col_ptr.first().copied() != Some(0)
        || sp.col_ptr.last().copied() != Some(sp.row_idx.len())
    {
        return None;
    }

    let mut degree = Vec::with_capacity(n);
    let mut queue = VecDeque::new();
    for v in 0..n {
        let start = sp.col_ptr[v];
        let end = sp.col_ptr[v + 1];
        if start > end || end > sp.row_idx.len() {
            return None;
        }
        let d = end - start;
        degree.push(d);
        if d <= 1 {
            queue.push_back(v);
        }
    }
    if queue.is_empty() {
        return None;
    }

    let mut removed = vec![false; n];
    let mut prefix = Vec::new();
    let mut prefix_flops = 0u64;
    while let Some(v) = queue.pop_front() {
        if removed[v] || degree[v] > 1 {
            continue;
        }
        prefix_flops = prefix_flops.checked_add(if degree[v] == 0 { 1 } else { 4 })?;
        removed[v] = true;
        prefix.push(v);
        for &u in &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]] {
            if u >= n || u == v {
                return None;
            }
            if !removed[u] {
                degree[u] = degree[u].checked_sub(1)?;
                if degree[u] == 1 {
                    queue.push_back(u);
                }
            }
        }
    }

    finish_reduction(
        sp,
        degree,
        removed,
        prefix,
        prefix_flops,
        min_removed_numerator,
        min_removed_denominator,
        max_core_n,
        max_core_nnz,
    )
}

#[allow(clippy::too_many_arguments)]
fn reduce_simplicial_with_limits(
    sp: &ScoringPattern,
    max_degree: usize,
    max_pair_checks: u64,
    min_removed_numerator: usize,
    min_removed_denominator: usize,
    max_core_n: usize,
    max_core_nnz: usize,
) -> Option<LeafCore> {
    let n = sp.n;
    if n == 0
        || !(2..=3).contains(&max_degree)
        || min_removed_denominator == 0
        || sp.col_ptr.len() != n.checked_add(1)?
        || sp.col_ptr.first().copied() != Some(0)
        || sp.col_ptr.last().copied() != Some(sp.row_idx.len())
    {
        return None;
    }

    let mut degree = Vec::with_capacity(n);
    let mut queue: BinaryHeap<Reverse<(usize, usize)>> = BinaryHeap::new();
    for v in 0..n {
        let start = sp.col_ptr[v];
        let end = sp.col_ptr[v + 1];
        if start > end || end > sp.row_idx.len() {
            return None;
        }
        let d = end - start;
        degree.push(d);
        if d <= max_degree {
            queue.push(Reverse((d, v)));
        }
    }
    if queue.is_empty() {
        return None;
    }

    let mut removed = vec![false; n];
    let mut prefix = Vec::new();
    let mut prefix_flops = 0u64;
    let mut pair_checks = 0u64;
    let mut live_neighbors = [NONE; 3];
    while let Some(Reverse((queued_degree, v))) = queue.pop() {
        if removed[v] || degree[v] != queued_degree || degree[v] > max_degree {
            continue;
        }
        let mut live_count = 0usize;
        for &u in &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]] {
            if u >= n || u == v {
                return None;
            }
            if !removed[u] {
                if live_count == live_neighbors.len() {
                    return None;
                }
                live_neighbors[live_count] = u;
                live_count += 1;
            }
        }
        if live_count != degree[v] {
            return None;
        }

        let mut simplicial = true;
        'pairs: for i in 0..live_count {
            let u = live_neighbors[i];
            let row = &sp.row_idx[sp.col_ptr[u]..sp.col_ptr[u + 1]];
            for &w in &live_neighbors[(i + 1)..live_count] {
                pair_checks = pair_checks.checked_add(1)?;
                if pair_checks > max_pair_checks {
                    return None;
                }
                if row.binary_search(&w).is_err() {
                    simplicial = false;
                    break 'pairs;
                }
            }
        }
        if !simplicial {
            continue;
        }

        let count = degree[v] as u64 + 1;
        prefix_flops = prefix_flops.checked_add(count.checked_mul(count)?)?;
        removed[v] = true;
        prefix.push(v);
        for &u in &live_neighbors[..live_count] {
            degree[u] = degree[u].checked_sub(1)?;
            if degree[u] <= max_degree {
                // A failed clique test can become true only when a live
                // neighbor disappears. Queueing here is therefore sufficient.
                queue.push(Reverse((degree[u], u)));
            }
        }
    }

    finish_reduction(
        sp,
        degree,
        removed,
        prefix,
        prefix_flops,
        min_removed_numerator,
        min_removed_denominator,
        max_core_n,
        max_core_nnz,
    )
}

#[allow(clippy::too_many_arguments)]
fn finish_reduction(
    sp: &ScoringPattern,
    degree: Vec<usize>,
    removed: Vec<bool>,
    prefix: Vec<usize>,
    prefix_flops: u64,
    min_removed_numerator: usize,
    min_removed_denominator: usize,
    max_core_n: usize,
    max_core_nnz: usize,
) -> Option<LeafCore> {
    let n = sp.n;
    let core_n = n.checked_sub(prefix.len())?;
    if prefix.is_empty() || core_n == 0 || core_n > max_core_n {
        return None;
    }
    if prefix.len().checked_mul(min_removed_denominator)? < n.checked_mul(min_removed_numerator)? {
        return None;
    }

    // Degrees have been decremented once for every removed neighbor, so their
    // sum on survivors is the induced core's directed nnz. This checks the
    // expensive-core gate before allocating or copying its CSC.
    let mut core_nnz = 0usize;
    for v in 0..n {
        if !removed[v] {
            core_nnz = core_nnz.checked_add(degree[v])?;
            if core_nnz > max_core_nnz {
                return None;
            }
        }
    }

    let core_ids: Vec<usize> = (0..n).filter(|&v| !removed[v]).collect();
    let mut local_of = vec![NONE; n];
    for (local, &original) in core_ids.iter().enumerate() {
        local_of[original] = local;
    }
    let mut core_col_ptr = Vec::with_capacity(core_n + 1);
    let mut core_row_idx = Vec::with_capacity(core_nnz);
    core_col_ptr.push(0);
    for &v in &core_ids {
        for &u in &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]] {
            if u >= n || u == v {
                return None;
            }
            if !removed[u] {
                let local = local_of[u];
                if local == NONE {
                    return None;
                }
                core_row_idx.push(local);
            }
        }
        core_col_ptr.push(core_row_idx.len());
    }
    if core_row_idx.len() != core_nnz {
        return None;
    }

    Some(LeafCore {
        original_n: n,
        prefix,
        core_ids,
        core_col_ptr,
        core_row_idx,
        prefix_flops,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn probe_specialized_order_pair() {
        struct Restore(bool);
        impl Drop for Restore {
            fn drop(&mut self) {
                TEST_ENABLED.with(|cell| cell.set(self.0));
            }
        }
        let _restore = Restore(TEST_ENABLED.with(|cell| cell.get()));
        for (index, (name, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            let run = |enabled| {
                TEST_ENABLED.with(|cell| cell.set(enabled));
                let start = std::time::Instant::now();
                let permutation = crate::ordering::order(&pattern);
                (permutation, start.elapsed().as_secs_f64())
            };
            let (baseline, candidate) = if index % 2 == 0 {
                (run(false), run(true))
            } else {
                let candidate = run(true);
                (run(false), candidate)
            };
            let before = ssi_scoring::score(&pattern, &baseline.0).flops;
            let after = ssi_scoring::score(&pattern, &candidate.0).flops;
            assert!(after <= before, "{name}: final specialization regressed");
            println!(
                "ROUND3_AB\t{name}\t{}\t{}\t{before}\t{after}\t{:.6}\t{:.6}",
                pattern.n,
                pattern.nnz(),
                baseline.1,
                candidate.1,
            );
        }
    }

    #[test]
    fn refinement_lifts_exact_scores_and_specializes_cycle_cores() {
        let mut edges: Vec<_> = (0..8).map(|v| (v, (v + 1) % 8)).collect();
        edges.extend((0..8).map(|v| (v, v + 8)));
        let p = pattern(16, &edges);
        let seed: Vec<_> = (0..16).collect();
        let (candidate, predicted) = refine(&p, &seed).unwrap();
        assert_eq!(predicted, 8 * 4 + 9 * 8 - 13);
        assert_eq!(ssi_scoring::score(&p, &candidate).flops, predicted);
        assert!(predicted <= ssi_scoring::score(&p, &seed).flops);
        assert_eq!(refine(&p, &seed), Some((candidate, predicted)));
    }

    #[test]
    fn independent_core_solvers_preserve_parallel_replay() {
        struct Restore(bool);
        impl Drop for Restore {
            fn drop(&mut self) {
                crate::ordering::parallel::FORCE_SEQUENTIAL.with(|cell| cell.set(self.0));
            }
        }
        let cn = 2_000;
        let mut edges: Vec<_> = (0..cn)
            .flat_map(|v| [1, 7, 31].map(move |step| (v, (v + step) % cn)))
            .collect();
        edges.extend((0..1_000).map(|v| (v, cn + v)));
        let p = pattern(3_000, &edges);
        let seed: Vec<_> = (0..p.n).collect();
        let previous = crate::ordering::parallel::FORCE_SEQUENTIAL.with(|cell| cell.replace(false));
        let _restore = Restore(previous);
        let parallel = refine(&p, &seed).unwrap();
        crate::ordering::parallel::FORCE_SEQUENTIAL.with(|cell| cell.set(true));
        let sequential = refine(&p, &seed).unwrap();
        assert_eq!(parallel, sequential);
        assert_eq!(ssi_scoring::score(&p, &parallel.0).flops, parallel.1);
        assert!(parallel.1 <= ssi_scoring::score(&p, &seed).flops);
    }

    #[test]
    #[ignore]
    fn probe_core_specializations() {
        for (name, pattern) in crate::corpus::corpus() {
            let incumbent = crate::ordering::order(&pattern);
            let baseline = ssi_scoring::score(&pattern, &incumbent).flops;
            for cap in 1..=3 {
                let start = std::time::Instant::now();
                let reduced = if cap == 1 {
                    reduce(&pattern)
                } else {
                    reduce_simplicial(&pattern, cap, 500_000)
                };
                let Some(core) = reduced else {
                    println!(
                        "CORE_SPECIAL\t{name}\t{}\t{}\t{cap}\t0\t0\t{baseline}\t{baseline}\t{:.6}",
                        pattern.n,
                        pattern.nnz(),
                        start.elapsed().as_secs_f64(),
                    );
                    continue;
                };
                let core_seed = core.restrict(&incumbent).unwrap();
                let candidate_core = crate::ordering::rgreedy::subset_window_descent_step(
                    core.core_n(),
                    &core.core_col_ptr,
                    &core.core_row_idx,
                    &core_seed,
                    12,
                    4,
                    5,
                    CORE_WINDOW_BUDGET,
                )
                .unwrap_or(core_seed);
                let candidate = core.splice(&candidate_core).unwrap();
                let seconds = start.elapsed().as_secs_f64();
                let after = ssi_scoring::score(&pattern, &candidate).flops;
                assert!(after <= baseline, "{name}: certified core cap {cap}");
                println!(
                    "CORE_SPECIAL\t{name}\t{}\t{}\t{cap}\t{}\t{}\t{baseline}\t{after}\t{seconds:.6}",
                    pattern.n, pattern.nnz(), core.core_n(), core.core_nnz(),
                );
            }
        }
    }

    #[test]
    #[ignore]
    fn probe_core_solver_plans() {
        for (name, pattern) in crate::corpus::corpus() {
            let incumbent = crate::ordering::order(&pattern);
            let baseline = ssi_scoring::score(&pattern, &incumbent).flops;
            let prepare = std::time::Instant::now();
            let Some(core) = reduce_simplicial(&pattern, 2, 500_000) else {
                continue;
            };
            let seed = core.restrict(&incumbent).unwrap();
            let core_pattern = crate::Pattern {
                n: core.core_n(),
                col_ptr: core.core_col_ptr.clone(),
                row_idx: core.core_row_idx.clone(),
            };
            let seed_flops = ssi_scoring::score(&core_pattern, &seed).flops;
            assert!(core.prefix_flops + seed_flops <= baseline);
            let prepare_seconds = prepare.elapsed().as_secs_f64();
            let cycles = core.core_col_ptr.windows(2).all(|p| p[1] - p[0] == 2);
            for plan in 0..7 {
                let start = std::time::Instant::now();
                let mut candidate = seed.clone();
                if !cycles {
                    if plan <= 3 {
                        let (width, step, budget) = [
                            (8, 3, 32_000_000),
                            (10, 3, 48_000_000),
                            (12, 5, 64_000_000),
                            (14, 5, 64_000_000),
                        ][plan];
                        if let Some(next) = crate::ordering::rgreedy::subset_window_descent_step(
                            core.core_n(),
                            &core.core_col_ptr,
                            &core.core_row_idx,
                            &candidate,
                            width,
                            4,
                            step,
                            budget,
                        ) {
                            candidate = next;
                        }
                    } else if plan == 4 {
                        for (width, step, sweeps, budget) in [
                            (8, 3, 2, 16_000_000),
                            (12, 5, 4, 32_000_000),
                            (10, 3, 2, 24_000_000),
                        ] {
                            if let Some(next) = crate::ordering::rgreedy::subset_window_descent_step(
                                core.core_n(),
                                &core.core_col_ptr,
                                &core.core_row_idx,
                                &candidate,
                                width,
                                sweeps,
                                step,
                                budget,
                            ) {
                                candidate = next;
                            }
                        }
                    } else {
                        let result = if plan == 5 {
                            crate::ordering::rgreedy::search(
                                core.core_n(),
                                &core.core_col_ptr,
                                &core.core_row_idx,
                                &candidate,
                                seed_flops,
                                32_000_000,
                                0x5e71_c0de,
                            )
                        } else {
                            crate::ordering::rgreedy::search_par(
                                core.core_n(),
                                &core.core_col_ptr,
                                &core.core_row_idx,
                                &candidate,
                                seed_flops,
                                32_000_000,
                                0,
                            )
                        };
                        if let Some((next, _)) = result {
                            candidate = next;
                        }
                    }
                }
                let lifted = core.splice(&candidate).unwrap();
                let seconds = prepare_seconds + start.elapsed().as_secs_f64();
                let after = ssi_scoring::score(&pattern, &lifted).flops;
                assert!(after <= baseline, "{name}: core solver {plan}");
                println!(
                    "CORE_PLAN\t{name}\t{}\t{}\t{plan}\t{}\t{}\t{baseline}\t{after}\t{seconds:.6}",
                    pattern.n,
                    pattern.nnz(),
                    core.core_n(),
                    core.core_nnz(),
                );
            }
        }
    }

    fn pattern(n: usize, edges: &[(usize, usize)]) -> ScoringPattern {
        let mut rows = vec![Vec::new(); n];
        for &(u, v) in edges {
            assert!(u < n && v < n && u != v);
            rows[u].push(v);
            rows[v].push(u);
        }
        let mut col_ptr = Vec::with_capacity(n + 1);
        let mut row_idx = Vec::new();
        col_ptr.push(0);
        for row in &mut rows {
            row.sort_unstable();
            row.dedup();
            row_idx.extend_from_slice(row);
            col_ptr.push(row_idx.len());
        }
        ScoringPattern {
            n,
            col_ptr,
            row_idx,
        }
    }

    fn unbounded(sp: &ScoringPattern) -> Option<LeafCore> {
        reduce_with_limits(sp, 0, 1, usize::MAX, usize::MAX)
    }

    fn simplicial_unbounded(
        sp: &ScoringPattern,
        max_degree: usize,
        max_pair_checks: u64,
    ) -> Option<LeafCore> {
        reduce_simplicial_with_limits(
            sp,
            max_degree,
            max_pair_checks,
            0,
            1,
            usize::MAX,
            usize::MAX,
        )
    }

    fn literal_flops(sp: &ScoringPattern, perm: &[usize]) -> u64 {
        assert_eq!(perm.len(), sp.n);
        let mut rows: Vec<Vec<bool>> = vec![vec![false; sp.n]; sp.n];
        for v in 0..sp.n {
            for &u in &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]] {
                rows[v][u] = true;
            }
        }
        let mut alive = vec![true; sp.n];
        let mut total = 0u64;
        for &v in perm {
            assert!(v < sp.n && alive[v]);
            let neighbors: Vec<usize> = (0..sp.n).filter(|&u| alive[u] && rows[v][u]).collect();
            let count = neighbors.len() as u64 + 1;
            total += count * count;
            for i in 0..neighbors.len() {
                for j in (i + 1)..neighbors.len() {
                    let (u, w) = (neighbors[i], neighbors[j]);
                    rows[u][w] = true;
                    rows[w][u] = true;
                }
            }
            alive[v] = false;
        }
        total
    }

    fn next_permutation(p: &mut [usize]) -> bool {
        let Some(i) = (0..p.len().saturating_sub(1))
            .rev()
            .find(|&i| p[i] < p[i + 1])
        else {
            return false;
        };
        let j = (i + 1..p.len()).rev().find(|&j| p[i] < p[j]).unwrap();
        p.swap(i, j);
        p[i + 1..].reverse();
        true
    }

    fn graph_from_mask(n: usize, mask: u64) -> ScoringPattern {
        let mut edges = Vec::new();
        let mut bit = 0;
        for u in 0..n {
            for v in (u + 1)..n {
                if mask & (1 << bit) != 0 {
                    edges.push((u, v));
                }
                bit += 1;
            }
        }
        pattern(n, &edges)
    }

    #[test]
    fn exhaustive_restricted_incumbent_never_worsens_and_score_splits() {
        for n in 1..=5 {
            let edge_count = n * (n - 1) / 2;
            for mask in 0..(1u64 << edge_count) {
                let sp = graph_from_mask(n, mask);
                let Some(lift) = unbounded(&sp) else {
                    continue;
                };
                let core_sp = ScoringPattern {
                    n: lift.core_n(),
                    col_ptr: lift.core_col_ptr.clone(),
                    row_idx: lift.core_row_idx.clone(),
                };
                let mut incumbent: Vec<usize> = (0..n).collect();
                loop {
                    let core_perm = lift.restrict(&incumbent).unwrap();
                    let lifted = lift.splice(&core_perm).unwrap();
                    assert!(
                        literal_flops(&sp, &lifted) <= literal_flops(&sp, &incumbent),
                        "n={n} graph={mask:#x} incumbent={incumbent:?}"
                    );
                    assert_eq!(
                        literal_flops(&sp, &lifted),
                        lift.prefix_flops + literal_flops(&core_sp, &core_perm),
                        "n={n} graph={mask:#x} core={core_perm:?}"
                    );
                    assert_eq!(lift.lift_restricted(&incumbent), Some(lifted));
                    if !next_permutation(&mut incumbent) {
                        break;
                    }
                }
            }
        }
    }

    #[test]
    fn exhaustive_low_degree_simplicial_restriction_never_worsens() {
        for n in 1..=5 {
            let edge_count = n * (n - 1) / 2;
            for mask in 0..(1u64 << edge_count) {
                let sp = graph_from_mask(n, mask);
                let Some(lift) = simplicial_unbounded(&sp, 3, u64::MAX) else {
                    continue;
                };
                let core_sp = ScoringPattern {
                    n: lift.core_n(),
                    col_ptr: lift.core_col_ptr.clone(),
                    row_idx: lift.core_row_idx.clone(),
                };
                let mut incumbent: Vec<usize> = (0..n).collect();
                loop {
                    let core_perm = lift.restrict(&incumbent).unwrap();
                    let lifted = lift.splice(&core_perm).unwrap();
                    assert!(
                        literal_flops(&sp, &lifted) <= literal_flops(&sp, &incumbent),
                        "n={n} graph={mask:#x} incumbent={incumbent:?}"
                    );
                    assert_eq!(
                        literal_flops(&sp, &lifted),
                        lift.prefix_flops + literal_flops(&core_sp, &core_perm),
                        "n={n} graph={mask:#x} core={core_perm:?}"
                    );
                    if !next_permutation(&mut incumbent) {
                        break;
                    }
                }
            }
        }
    }

    #[test]
    fn exhaustive_forest_attachments_leave_the_triangle_core() {
        // Vertices 3..=6 independently choose no parent or one lower-numbered
        // parent. Every choice is a forest attached to, or disconnected from,
        // the fixed triangle. This covers branching, chains and tree components.
        for p3 in 0..=3 {
            for p4 in 0..=4 {
                for p5 in 0..=5 {
                    for p6 in 0..=6 {
                        let choices = [p3, p4, p5, p6];
                        let mut edges = vec![(0, 1), (1, 2), (2, 0)];
                        for (offset, &choice) in choices.iter().enumerate() {
                            let v = offset + 3;
                            if choice < v {
                                edges.push((v, choice));
                            }
                        }
                        let sp = pattern(7, &edges);
                        let lift = unbounded(&sp).expect("forest extras must peel");
                        assert_eq!(lift.core_ids, vec![0, 1, 2]);
                        assert_eq!(lift.core_col_ptr, vec![0, 2, 4, 6]);
                        assert_eq!(lift.core_row_idx, vec![1, 2, 0, 2, 0, 1]);
                        for core_perm in [
                            vec![0, 1, 2],
                            vec![0, 2, 1],
                            vec![1, 0, 2],
                            vec![1, 2, 0],
                            vec![2, 0, 1],
                            vec![2, 1, 0],
                        ] {
                            let full = lift.splice(&core_perm).unwrap();
                            let core_sp = pattern(3, &[(0, 1), (1, 2), (2, 0)]);
                            assert_eq!(
                                literal_flops(&sp, &full),
                                lift.prefix_flops + literal_flops(&core_sp, &core_perm)
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn bounded_simplicial_peel_removes_chordal_attachment_not_cycle_core() {
        // A triangle shares vertex 0 with a chordless four-cycle. Vertices 4
        // and 5 are degree-two simplicial; cycle vertices are not simplicial.
        let sp = pattern(6, &[(0, 1), (1, 2), (2, 3), (3, 0), (0, 4), (4, 5), (5, 0)]);
        assert!(unbounded(&sp).is_none(), "there are no pendant vertices");
        assert!(
            simplicial_unbounded(&sp, 2, 3).is_none(),
            "failed clique tests consume the explicit pair budget"
        );
        let lift = simplicial_unbounded(&sp, 2, 32).expect("triangle attachment peels");
        assert_eq!(lift.prefix, vec![4, 5]);
        assert_eq!(lift.prefix_flops, 13);
        assert_eq!(lift.core_ids, vec![0, 1, 2, 3]);
        assert_eq!(lift.core_col_ptr, vec![0, 2, 4, 6, 8]);
        assert_eq!(lift.core_row_idx, vec![1, 3, 0, 2, 1, 3, 0, 2]);

        let incumbent = vec![0, 4, 1, 5, 2, 3];
        let restricted = lift.restrict(&incumbent).unwrap();
        assert_eq!(restricted, vec![0, 1, 2, 3]);
        let lifted = lift.splice(&restricted).unwrap();
        assert!(literal_flops(&sp, &lifted) <= literal_flops(&sp, &incumbent));
        let core_sp = pattern(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]);
        assert_eq!(
            literal_flops(&sp, &lifted),
            lift.prefix_flops + literal_flops(&core_sp, &restricted)
        );
    }

    #[test]
    fn degree_three_simplicial_check_is_explicit_and_deterministic() {
        // K4 shares vertex 0 with a chordless four-cycle. Its three nonboundary
        // vertices need all three pair checks at live degree three.
        let sp = pattern(
            7,
            &[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (0, 4),
                (0, 5),
                (0, 6),
                (4, 5),
                (4, 6),
                (5, 6),
            ],
        );
        let lift = simplicial_unbounded(&sp, 3, 64).expect("K4 attachment peels");
        assert_eq!(lift.prefix, vec![4, 5, 6]);
        assert_eq!(lift.prefix_flops, 16 + 9 + 4);
        assert_eq!(lift.core_ids, vec![0, 1, 2, 3]);
        let repeat = simplicial_unbounded(&sp, 3, 64).unwrap();
        assert_eq!(lift.prefix, repeat.prefix);
        assert_eq!(lift.core_col_ptr, repeat.core_col_ptr);
        assert_eq!(lift.core_row_idx, repeat.core_row_idx);
        assert!(simplicial_unbounded(&sp, 1, u64::MAX).is_none());
        assert!(simplicial_unbounded(&sp, 4, u64::MAX).is_none());
    }

    #[test]
    fn production_gates_and_helpers_fail_closed() {
        assert_eq!(CORE_WINDOW_BUDGET, 32_000_000);
        assert_eq!(MAX_CORE_N, 12_000);
        assert_eq!(MAX_CORE_NNZ, 200_000);

        let mut edges = vec![(0, 1), (1, 2), (2, 0)];
        edges.extend([(0, 3), (3, 4), (1, 5), (5, 6), (6, 7)]);
        let sp = pattern(8, &edges);
        let a = reduce(&sp).expect("five of eight vertices peel");
        let b = reduce(&sp).expect("deterministic repeat");
        assert_eq!(a.prefix, b.prefix);
        assert_eq!(a.core_ids, b.core_ids);
        assert_eq!(a.core_col_ptr, b.core_col_ptr);
        assert_eq!(a.core_row_idx, b.core_row_idx);
        assert_eq!(a.prefix_flops, b.prefix_flops);
        assert_eq!(a.core_ids, vec![0, 1, 2]);

        assert!(a.restrict(&[0, 1]).is_none());
        assert!(a.restrict(&[0, 1, 2, 3, 4, 5, 6, 6]).is_none());
        assert!(a.splice(&[0, 1]).is_none());
        assert!(a.splice(&[0, 1, 1]).is_none());

        let too_little_removed = pattern(
            8,
            &[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 5),
                (5, 6),
                (6, 0),
                (0, 7),
            ],
        );
        assert!(reduce(&too_little_removed).is_none());
    }
}
