//! Exact window search factored by connected components of the live graph.

use super::window_signatures::{ChargeModel, SignatureEngine};
use super::{Game, TripleWork};

const MAX_WIDTH: usize = 14;
const MAX_DIMENSION: usize = super::MAX_N;

/// Existing passes preserve neutral order; the alternate final pass explores
/// largest local-index optimal orders before later overlapping windows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum WindowPolicy {
    Strict,
    NeutralLargest,
    NeutralRollingLargest,
}

impl WindowPolicy {
    fn permits_neutral(self) -> bool {
        matches!(self, Self::NeutralLargest | Self::NeutralRollingLargest)
    }
}

#[cfg(test)]
#[derive(Clone, Default)]
struct WorkStats {
    calls: [usize; 17],
    completed: [usize; 17],
    components: [usize; 17],
    refused_components: usize,
}

#[cfg(test)]
thread_local! {
    static WORK_STATS: std::cell::RefCell<WorkStats> = std::cell::RefCell::new(WorkStats::default());
}

#[cfg(test)]
struct WorkReport {
    width: usize,
    completed: bool,
}

#[cfg(test)]
impl Drop for WorkReport {
    fn drop(&mut self) {
        WORK_STATS.with(|cell| {
            let mut stats = cell.borrow_mut();
            stats.calls[self.width] += 1;
            stats.completed[self.width] += usize::from(self.completed);
        });
    }
}

fn solve_component_policy<const NEUTRAL: bool>(
    game: &Game<'_>,
    vertices: &[usize],
    work: &mut TripleWork,
) -> Option<(Vec<usize>, u64, u64)> {
    let k = vertices.len();
    let states = 1usize << k;
    let cost = states.saturating_mul(
        16usize
            .saturating_mul(k)
            .saturating_add(6usize.saturating_mul(game.w))
            .saturating_add(24),
    );
    if !work.charge(cost) {
        #[cfg(test)]
        WORK_STATS.with(|cell| cell.borrow_mut().refused_components += 1);
        return None;
    }
    #[cfg(test)]
    WORK_STATS.with(|cell| cell.borrow_mut().components[k] += 1);
    let mut inside = vec![0u16; k];
    for (i, &v) in vertices.iter().enumerate() {
        for (j, &u) in vertices.iter().enumerate() {
            if game.adj[v * game.w + u / 64] & (1u64 << (u % 64)) != 0 {
                inside[i] |= 1 << j;
            }
        }
    }
    let mut components = vec![0u16; states * k];
    let mut unions = vec![0u64; states * game.w];
    let mut widths = vec![0u64; states];
    for mask in 1..states {
        let v = mask.trailing_zeros() as usize;
        let bit = 1usize << v;
        let rest = mask ^ bit;
        let mut merged = bit as u16;
        let mut neighbors = inside[v] & rest as u16;
        while neighbors != 0 {
            let u = neighbors.trailing_zeros() as usize;
            neighbors &= neighbors - 1;
            merged |= components[rest * k + u];
        }
        for u in 0..k {
            components[mask * k + u] = if merged & (1 << u) != 0 {
                merged
            } else {
                components[rest * k + u]
            };
        }
        let vertex = vertices[v];
        for word in 0..game.w {
            unions[mask * game.w + word] =
                unions[rest * game.w + word] | game.adj[vertex * game.w + word];
        }
        unions[mask * game.w + vertex / 64] |= 1u64 << (vertex % 64);
        if merged as usize == mask {
            let boundary = unions[mask * game.w..(mask + 1) * game.w]
                .iter()
                .map(|word| word.count_ones() as u64)
                .sum::<u64>();
            widths[mask] = boundary - mask.count_ones() as u64 + 1;
        }
    }

    // A pivot sees the boundary of its eliminated-prefix component. Other
    // eliminated components cannot touch it, so no fill graph replay is needed.
    let mut best = vec![u64::MAX; states];
    let mut path = vec![if NEUTRAL { 0 } else { u64::MAX }; states];
    best[0] = 0;
    path[0] = 0;
    for mask in 0..states - 1 {
        if NEUTRAL && best[mask] == u64::MAX {
            continue;
        }
        for pivot in 0..k {
            let bit = 1usize << pivot;
            if mask & bit != 0 {
                continue;
            }
            let next = mask | bit;
            let width = widths[components[next * k + pivot] as usize];
            let cost = best[mask] + width * width;
            let code = (path[mask] << 4) | pivot as u64;
            let preferred_tie = if NEUTRAL { code > path[next] } else { code < path[next] };
            if cost < best[next] || (cost == best[next] && preferred_tie) {
                best[next] = cost;
                path[next] = code;
            }
        }
    }
    let incumbent = (0..k)
        .map(|pivot| {
            let mask = (1usize << (pivot + 1)) - 1;
            let width = widths[components[mask * k + pivot] as usize];
            width * width
        })
        .sum();
    let order = if best[states - 1] < incumbent
        || (NEUTRAL && best[states - 1] == incumbent)
    {
        (0..k)
            .map(|i| vertices[((path[states - 1] >> (4 * (k - i - 1))) & 15) as usize])
            .collect()
    } else {
        vertices.to_vec()
    };
    Some((order, best[states - 1], incumbent))
}

fn solve_component(
    game: &Game<'_>,
    vertices: &[usize],
    work: &mut TripleWork,
) -> Option<(Vec<usize>, u64, u64)> {
    solve_component_policy::<false>(game, vertices, work)
}

fn refine_window(
    game: &Game<'_>,
    window: &mut [usize],
    work: &mut TripleWork,
    engine: &mut Option<SignatureEngine>,
    charge_model: ChargeModel,
    policy: WindowPolicy,
) -> Option<bool> {
    let k = window.len();
    if !work.charge(8 * k * k + 8 * k) {
        return None;
    }
    let mut unseen = (1u16 << k) - 1;
    let mut changed = false;
    while unseen != 0 {
        let mut component = 1u16 << unseen.trailing_zeros();
        let mut frontier = component;
        while frontier != 0 {
            let i = frontier.trailing_zeros() as usize;
            frontier &= frontier - 1;
            let v = window[i];
            for (j, &u) in window.iter().enumerate() {
                let bit = 1u16 << j;
                if unseen & bit != 0
                    && component & bit == 0
                    && game.adj[v * game.w + u / 64] & (1u64 << (u % 64)) != 0
                {
                    component |= bit;
                    frontier |= bit;
                }
            }
        }
        unseen &= !component;
        if component.count_ones() < 2 {
            continue;
        }
        let positions: Vec<usize> = (0..k).filter(|&i| component & (1 << i) != 0).collect();
        let vertices: Vec<usize> = positions.iter().map(|&i| window[i]).collect();
        let incident = vertices.iter().map(|&v| game.deg[v] as usize).sum();
        let (union_cost, signature_cost) =
            SignatureEngine::charge_costs(vertices.len(), game.w, incident);
        // Neutral results never enter the strict signature memo. Its keys and
        // clique certificates therefore retain their existing exact semantics.
        let solution = if policy.permits_neutral() {
            solve_component_policy::<true>(game, &vertices, work)
        } else if vertices.len() >= 5 && signature_cost < union_cost {
            let engine = engine.get_or_insert_with(|| SignatureEngine::new(game.n));
            engine.set_charge_model(charge_model);
            engine.solve_component(game, &vertices, work)
        } else {
            solve_component(game, &vertices, work)
        };
        let Some((order, best, incumbent)) = solution else {
            return if changed { Some(true) } else { None };
        };
        if best < incumbent
            || (policy.permits_neutral()
                && best == incumbent
                && order != vertices)
        {
            // Preserve component interleaving: only permute the positions
            // belonging to this connected component of the live window.
            for (&position, &v) in positions.iter().zip(&order) {
                window[position] = v;
            }
            changed = true;
        }
    }
    Some(changed)
}

/// Every window leaves the same suffix graph. Keep completed strict gains even
/// when the precharged work allowance cannot fund the rest of a sweep.
pub(crate) fn subset_window_descent(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    budget: i64,
) -> Option<Vec<usize>> {
    subset_window_descent_config(
        n,
        col_ptr,
        row_idx,
        seed,
        width,
        sweeps,
        (width / 2).max(1),
        budget,
        ChargeModel::UnionParity,
        WindowPolicy::Strict,
    )
}

pub(crate) fn subset_window_descent_step(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    offset_step: usize,
    budget: i64,
) -> Option<Vec<usize>> {
    subset_window_descent_config(
        n,
        col_ptr,
        row_idx,
        seed,
        width,
        sweeps,
        offset_step,
        budget,
        ChargeModel::SignatureTrue,
        WindowPolicy::Strict,
    )
}

/// Bounded neutral moves are internal to this alternate pass. The caller
/// must retain its seed unless the returned whole ordering strictly improves.
pub(crate) fn subset_window_descent_neutral(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    offset_step: usize,
    budget: i64,
) -> Option<Vec<usize>> {
    subset_window_descent_config(
        n, col_ptr, row_idx, seed, width, sweeps, offset_step, budget,
        ChargeModel::UnionParity, WindowPolicy::NeutralLargest,
    )
}

/// Neutral windows overlap immediately. Only the outgoing prefix is
/// eliminated before the next solve, preserving its exact live boundary.
pub(crate) fn subset_window_descent_neutral_rolling(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    advance: usize,
    budget: i64,
) -> Option<Vec<usize>> {
    subset_window_descent_config(
        n, col_ptr, row_idx, seed, width, sweeps, advance, budget,
        ChargeModel::UnionParity, WindowPolicy::NeutralRollingLargest,
    )
}

fn subset_window_descent_config(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    offset_step: usize,
    budget: i64,
    charge_model: ChargeModel,
    policy: WindowPolicy,
) -> Option<Vec<usize>> {
    if n < 2
        || n > MAX_DIMENSION
        || !(2..=MAX_WIDTH).contains(&width)
        || offset_step >= width
        || (policy == WindowPolicy::NeutralRollingLargest && offset_step == 0)
        || sweeps == 0
        || budget <= 0
        || seed.len() != n
        || col_ptr.len() != n + 1
    {
        return None;
    }
    #[cfg(test)]
    let mut report = WorkReport {
        width,
        completed: false,
    };
    let mut work = TripleWork { remaining: budget };
    if !work.charge(n + 1 + row_idx.len() + 2 * n)
        || col_ptr.first().copied() != Some(0)
        || col_ptr.last().copied() != Some(row_idx.len())
        || col_ptr.windows(2).any(|p| p[0] > p[1])
        || row_idx.iter().any(|&v| v >= n)
    {
        return None;
    }
    let mut seen = vec![false; n];
    for &v in seed {
        if v >= n || seen[v] {
            return None;
        }
        seen[v] = true;
    }
    let words = n.div_ceil(64);
    let setup = 3 * n * words + 2 * row_idx.len() + 16 * n + words;
    if !work.charge(setup) {
        return None;
    }
    let pristine = Game::build_adj(n, col_ptr, row_idx)?;
    let mut game = Game::new(n, &pristine)?;
    let mut engine = None;
    let mut current = seed.to_vec();
    let mut changed = false;
    for sweep in 0..sweeps {
        if !work.charge(2 * n * words + 8 * n) {
            return changed.then_some(current);
        }
        if sweep == 0 {
            // Game::new just copied pristine adjacency. Keep the same logical
            // charge and initialize metadata without copying it a second time.
            game.reset_fresh();
        } else {
            game.reset();
        }
        let offset = (sweep * offset_step) % width;
        for &v in current.iter().take(offset.min(n)) {
            if !work.eliminate(&mut game, v) {
                return changed.then_some(current);
            }
        }
        let mut start = offset;
        while start + 1 < n {
            let end = (start + width).min(n);
            match refine_window(
                &game,
                &mut current[start..end],
                &mut work,
                &mut engine,
                charge_model,
                policy,
            ) {
                Some(improved) => changed |= improved,
                None => return changed.then_some(current),
            }
            if policy == WindowPolicy::NeutralRollingLargest {
                // The final suffix has already been optimized; do not revisit
                // progressively shorter copies of it or replay it pointlessly.
                if end == n {
                    break;
                }
                let next = start + offset_step;
                for &v in &current[start..next] {
                    if !work.eliminate(&mut game, v) {
                        return changed.then_some(current);
                    }
                }
                start = next;
            } else {
                if end < n {
                    for &v in &current[start..end] {
                        if !work.eliminate(&mut game, v) {
                            return changed.then_some(current);
                        }
                    }
                }
                start = end;
            }
        }
    }
    #[cfg(test)]
    {
        report.completed = true;
    }
    changed.then_some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pattern;

    #[test]
    #[ignore]
    fn probe_window_work() {
        for (name, pattern) in crate::corpus::corpus() {
            WORK_STATS.with(|cell| *cell.borrow_mut() = WorkStats::default());
            let _ = super::super::window_signatures::take_probe_stats();
            let permutation = crate::ordering::order(&pattern);
            let flops = ssi_scoring::score(&pattern, &permutation).flops;
            let stats = WORK_STATS.with(|cell| cell.borrow().clone());
            let memo = super::super::window_signatures::take_probe_stats();
            println!(
                "SIGNATURE_MEMO\t{name}\t{}\t{}\t{}\t{}",
                memo.probes, memo.hits, memo.inserted, memo.trivial,
            );
            println!(
                "WINDOW_WORK\t{name}\t{}\t{}\t{flops}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}",
                pattern.n,
                pattern.nnz(),
                stats.calls[8],
                stats.completed[8],
                stats.calls[10],
                stats.completed[10],
                stats.calls[12],
                stats.completed[12],
                stats.refused_components,
                stats.components,
            );
        }
    }

    #[test]
    #[ignore]
    fn probe_next_windows() {
        for (name, pattern) in crate::corpus::corpus() {
            if pattern.n < 6 || pattern.n > MAX_DIMENSION || pattern.nnz() > 200_000 {
                continue;
            }

            let incumbent = crate::ordering::order(&pattern);
            let before = ssi_scoring::score(&pattern, &incumbent).flops;
            for (width, sweeps, budget) in [
                (12, 4, 64_000_000),
                (14, 2, 48_000_000),
                (14, 4, 96_000_000),
            ] {
                WORK_STATS.with(|cell| *cell.borrow_mut() = WorkStats::default());
                let start = std::time::Instant::now();
                let candidate = subset_window_descent(
                    pattern.n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &incumbent,
                    width,
                    sweeps,
                    budget,
                );
                let seconds = start.elapsed().as_secs_f64();
                let after = candidate
                    .as_ref()
                    .map_or(before, |p| ssi_scoring::score(&pattern, p).flops);
                assert!(after <= before, "{name}: additional width {width}");
                let completed = WORK_STATS.with(|cell| cell.borrow().completed[width]);
                println!(
                    "NEXT_WINDOW\t{name}\t{}\t{}\t{before}\t{after}\t{seconds:.6}\t{width}\t{sweeps}\t{budget}\t{completed}",
                    pattern.n, pattern.nnz(),
                );
            }
        }
    }

    #[test]
    #[ignore]
    fn probe_window_offsets() {
        for (name, pattern) in crate::corpus::corpus() {
            if pattern.n < 6 || pattern.n > MAX_DIMENSION || pattern.nnz() > 200_000 {
                continue;
            }
            let incumbent = crate::ordering::order(&pattern);
            let before = ssi_scoring::score(&pattern, &incumbent).flops;
            for (width, step, budget) in [
                (12, 1, 64_000_000),
                (12, 5, 64_000_000),
                (14, 1, 96_000_000),
                (14, 5, 96_000_000),
            ] {
                let start = std::time::Instant::now();
                let candidate = subset_window_descent_step(
                    pattern.n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &incumbent,
                    width,
                    4,
                    step,
                    budget,
                );
                let seconds = start.elapsed().as_secs_f64();
                let after = candidate
                    .as_ref()
                    .map_or(before, |p| ssi_scoring::score(&pattern, p).flops);
                assert!(after <= before, "{name}: offset {width}/{step}");
                println!(
                    "OFFSET_WINDOW\t{name}\t{}\t{}\t{before}\t{after}\t{seconds:.6}\t{width}\t{step}",
                    pattern.n, pattern.nnz(),
                );
            }
        }
    }

    fn permutations(values: &mut [usize], i: usize, visit: &mut impl FnMut(&[usize])) {
        if i == values.len() {
            visit(values);
        } else {
            for j in i..values.len() {
                values.swap(i, j);
                permutations(values, i + 1, visit);
                values.swap(i, j);
            }
        }
    }


    fn verify_neutral_window(p: &Pattern, prefix: &[usize], window: &[usize]) {
        let pristine = Game::build_adj(p.n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(p.n, &pristine).unwrap();
        game.reset();
        for &v in prefix { game.eliminate(v); }
        let mut work = TripleWork { remaining: i64::MAX };
        let (order, best, incumbent) =
            solve_component_policy::<true>(&game, window, &mut work).unwrap();
        let mut expected = u64::MAX;
        let mut expected_code = 0u64;
        let mut expected_order = Vec::new();
        let mut original = 0;
        permutations(&mut window.to_vec(), 0, &mut |candidate| {
            game.reset();
            for &v in prefix { game.eliminate(v); }
            let cost: u64 = candidate.iter().map(|&v| game.eliminate(v).pow(2)).sum();
            let code = candidate.iter().fold(0u64, |code, v| {
                (code << 4) | window.iter().position(|w| w == v).unwrap() as u64
            });
            if candidate == window { original = cost; }
            if cost < expected || (cost == expected && code > expected_code) {
                expected = cost;
                expected_code = code;
                expected_order = candidate.to_vec();
            }
        });
        assert_eq!((best, incumbent), (expected, original));
        assert_eq!(order, expected_order);
        game.reset();
        for &v in prefix { game.eliminate(v); }
        for &v in window { game.eliminate(v); }
        let suffix = game.adj.clone();
        game.reset();
        for &v in prefix { game.eliminate(v); }
        for &v in &order { game.eliminate(v); }
        assert_eq!(game.adj, suffix);
    }

    #[test]
    fn neutral_window_matches_exhaustive_cost_and_largest_local_tie() {
        let edges: Vec<_> = (0..4)
            .flat_map(|a| (a + 1..4).map(move |b| (a, b))).collect();
        for mask in 0usize..1 << edges.len() {
            let chosen: Vec<_> = edges.iter().enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0).map(|(_, &e)| e).collect();
            verify_neutral_window(&Pattern::from_edges(4, &chosen), &[], &[2, 0, 3, 1]);
        }
        for n in [9usize, 70] {
            for sample in 0..4 {
                let edges: Vec<_> = (0..n).flat_map(|a| {
                    (a + 1..n)
                        .filter(move |&b| (a * 31 + b * 17 + sample * 13) % 11 < 3)
                        .map(move |b| (a, b))
                }).collect();
                verify_neutral_window(
                    &Pattern::from_edges(n, &edges), &[0, 1], &[6, 2, 5, 3, 7, 4],
                );
            }
        }
    }

    #[test]
    fn neutral_window_handles_full_width_codes_and_component_interleaving() {
        let n = 17;
        let edges: Vec<_> = (0..n)
            .flat_map(|a| (a + 1..n).map(move |b| (a, b))).collect();
        let p = Pattern::from_edges(n, &edges);
        let pristine = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(n, &pristine).unwrap();
        game.reset();
        game.eliminate(0);
        let vertices: Vec<_> = (1..15).collect();
        let mut work = TripleWork { remaining: i64::MAX };
        let (order, best, incumbent) =
            solve_component_policy::<true>(&game, &vertices, &mut work).unwrap();
        assert_eq!(best, incumbent);
        assert_eq!(order, vertices.iter().copied().rev().collect::<Vec<_>>());

        let p = Pattern::from_edges(4, &[(0, 2), (1, 3)]);
        let pristine = Game::build_adj(4, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(4, &pristine).unwrap();
        game.reset();
        let mut window = vec![0, 1, 2, 3];
        let mut engine = None;
        let mut work = TripleWork { remaining: 1_000_000 };
        assert_eq!(refine_window(
            &game, &mut window, &mut work, &mut engine, ChargeModel::UnionParity,
            WindowPolicy::NeutralLargest,
        ), Some(true));
        assert_eq!(window, vec![2, 3, 0, 1]);
        assert!(engine.is_none(), "neutral orders must not enter the strict memo");
    }

    #[test]
    fn neutral_ties_unlock_a_strict_gain_in_a_later_overlapping_window() {
        let p = Pattern::from_edges(6, &[
            (0, 1), (0, 2), (0, 3), (1, 4), (1, 5), (2, 4), (2, 5),
        ]);
        let seed = vec![0, 1, 2, 3, 4, 5];
        let pristine = Game::build_adj(6, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(6, &pristine).unwrap();
        assert_eq!(game.replay_flops(&seed), 71);
        // Both alternating pair offsets are locally tied in the seed.
        assert!(subset_window_descent(
            6, &p.col_ptr, &p.row_idx, &seed, 2, 3, 1_000_000,
        ).is_none());
        // One neutral sweep reverses the three tied pairs without changing cost.
        let plateau = subset_window_descent_neutral(
            6, &p.col_ptr, &p.row_idx, &seed, 2, 1, 1, 1_000_000,
        ).unwrap();
        assert_eq!(plateau, vec![1, 0, 3, 2, 5, 4]);
        assert_eq!(game.replay_flops(&plateau), 71);
        // The shifted window now exposes degree-one vertex 3 before vertex 0.
        let improved = subset_window_descent(
            6, &p.col_ptr, &p.row_idx, &plateau, 2, 2, 1_000_000,
        ).unwrap();
        assert_eq!(game.replay_flops(&improved), 50);
    }

    #[test]
    fn neutral_descent_is_deterministic_and_nonworsening_with_partial_budgets() {
        for n in [8usize, 17, 65] {
            let edges: Vec<_> = (0..n).flat_map(|a| {
                (a + 1..n).filter(move |&b| (a * 19 + b * 7) % 13 < 3)
                    .map(move |b| (a, b))
            }).collect();
            let p = Pattern::from_edges(n, &edges);
            let seed: Vec<_> = (0..n).rev().collect();
            let pristine = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
            let mut game = Game::new(n, &pristine).unwrap();
            let before = game.replay_flops(&seed);
            for budget in [0, 1, 1_000, 10_000, 100_000, 1_000_000] {
                let first = subset_window_descent_neutral(
                    n, &p.col_ptr, &p.row_idx, &seed, 8, 4, 3, budget,
                );
                assert_eq!(first, subset_window_descent_neutral(
                    n, &p.col_ptr, &p.row_idx, &seed, 8, 4, 3, budget,
                ));
                if let Some(order) = first {
                    let mut sorted = order.clone();
                    sorted.sort_unstable();
                    assert_eq!(sorted, (0..n).collect::<Vec<_>>());
                    assert!(game.replay_flops(&order) <= before);
                }
            }
        }
    }


    #[test]
    fn rolling_neutral_reaches_an_overlap_gain_in_its_first_sweep() {
        let p = Pattern::from_edges(6, &[
            (0, 1), (0, 2), (0, 3), (1, 4), (1, 5), (2, 4), (2, 5),
        ]);
        let seed = vec![0, 1, 2, 3, 4, 5];
        let pristine = Game::build_adj(6, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(6, &pristine).unwrap();
        let disjoint = subset_window_descent_neutral(
            6, &p.col_ptr, &p.row_idx, &seed, 2, 1, 1, 1_000_000,
        ).unwrap();
        assert_eq!(game.replay_flops(&disjoint), 71);
        let rolling = subset_window_descent_neutral_rolling(
            6, &p.col_ptr, &p.row_idx, &seed, 2, 1, 1, 1_000_000,
        ).unwrap();
        assert_eq!(game.replay_flops(&rolling), 50);
    }

    #[test]
    fn rolling_neutral_stops_after_the_first_window_reaching_the_tail() {
        let n = 10;
        let edges: Vec<_> = (0..n)
            .flat_map(|a| (a + 1..n).map(move |b| (a, b))).collect();
        let p = Pattern::from_edges(n, &edges);
        let seed: Vec<_> = (0..n).collect();
        let order = subset_window_descent_neutral_rolling(
            n, &p.col_ptr, &p.row_idx, &seed, 8, 1, 3, 1_000_000,
        ).unwrap();
        // The two tied cliques occupy 0..8 and 3..10. A third shrinking tail
        // window would reverse part of this suffix again and fail this check.
        assert_eq!(order, vec![7, 6, 5, 9, 8, 0, 1, 2, 3, 4]);
        assert!(subset_window_descent_neutral_rolling(
            n, &p.col_ptr, &p.row_idx, &seed, 8, 1, 0, 1_000_000,
        ).is_none());
    }

    #[test]
    fn rolling_neutral_is_deterministic_and_nonworsening_with_partial_budgets() {
        for n in [8usize, 17, 65] {
            let edges: Vec<_> = (0..n).flat_map(|a| {
                (a + 1..n).filter(move |&b| (a * 19 + b * 7) % 13 < 3)
                    .map(move |b| (a, b))
            }).collect();
            let p = Pattern::from_edges(n, &edges);
            let seed: Vec<_> = (0..n).rev().collect();
            let pristine = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
            let mut game = Game::new(n, &pristine).unwrap();
            let before = game.replay_flops(&seed);
            for budget in [0, 1, 1_000, 10_000, 100_000, 1_000_000] {
                let first = subset_window_descent_neutral_rolling(
                    n, &p.col_ptr, &p.row_idx, &seed, 8, 3, 3, budget,
                );
                assert_eq!(first, subset_window_descent_neutral_rolling(
                    n, &p.col_ptr, &p.row_idx, &seed, 8, 3, 3, budget,
                ));
                if let Some(order) = first {
                    let mut sorted = order.clone();
                    sorted.sort_unstable();
                    assert_eq!(sorted, (0..n).collect::<Vec<_>>());
                    assert!(game.replay_flops(&order) <= before);
                }
            }
        }
    }

    fn verify_window(p: &Pattern, prefix: &[usize], window: &[usize]) {
        let pristine = Game::build_adj(p.n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(p.n, &pristine).unwrap();
        game.reset();
        for &v in prefix {
            game.eliminate(v);
        }
        let mut work = TripleWork {
            remaining: i64::MAX,
        };
        let (order, best, incumbent) = solve_component(&game, window, &mut work).unwrap();
        let mut expected = u64::MAX;
        let mut original = 0;
        permutations(&mut window.to_vec(), 0, &mut |candidate| {
            game.reset();
            for &v in prefix {
                game.eliminate(v);
            }
            let cost = candidate.iter().map(|&v| game.eliminate(v).pow(2)).sum();
            if candidate == window {
                original = cost;
            }
            expected = expected.min(cost);
        });
        assert_eq!(best, expected);
        assert_eq!(incumbent, original);
        game.reset();
        for &v in prefix {
            game.eliminate(v);
        }
        assert_eq!(
            order.iter().map(|&v| game.eliminate(v).pow(2)).sum::<u64>(),
            best
        );
        if best == incumbent {
            assert_eq!(order, window);
        }
    }

    #[test]
    fn window_dp_exhaustive_four_vertex_graphs() {
        let edges: Vec<_> = (0..4)
            .flat_map(|v| (v + 1..4).map(move |u| (v, u)))
            .collect();
        for mask in 0usize..1 << edges.len() {
            let selected: Vec<_> = edges
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, &e)| e)
                .collect();
            verify_window(&Pattern::from_edges(4, &selected), &[], &[0, 1, 2, 3]);
        }
    }

    #[test]
    fn window_dp_fourteen_pivots_preserves_ties_and_boundary_cost() {
        let n = 17;
        let edges: Vec<_> = (0..n)
            .flat_map(|v| (v + 1..n).map(move |u| (v, u)))
            .collect();
        let p = Pattern::from_edges(n, &edges);
        let pristine = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(n, &pristine).unwrap();
        game.reset();
        game.eliminate(0);
        let vertices: Vec<_> = (1..15).collect();
        let mut work = TripleWork {
            remaining: 10_000_000,
        };
        let (order, best, incumbent) = solve_component(&game, &vertices, &mut work).unwrap();
        let expected: u64 = (3u64..=16).map(|c| c * c).sum();
        assert_eq!((best, incumbent), (expected, expected));
        assert_eq!(order, vertices);
    }

    #[test]
    fn window_dp_matches_exhaustive_with_prefix_and_external_boundary() {
        for n in [9, 70] {
            for sample in 0..8 {
                let edges: Vec<_> = (0..n)
                    .flat_map(|v| {
                        (v + 1..n)
                            .filter(move |&u| (v * 31 + u * 17 + sample * 13) % 11 < 3)
                            .map(move |u| (v, u))
                    })
                    .collect();
                verify_window(
                    &Pattern::from_edges(n, &edges),
                    &[0, 1],
                    &[2, 3, 4, 5, 6, 7],
                );
            }
        }
    }

    #[test]
    fn window_dp_descent_is_deterministic_and_never_worsens() {
        for n in [8, 17, 65] {
            let edges: Vec<_> = (0..n)
                .flat_map(|v| {
                    (v + 1..n)
                        .filter(move |&u| (v * 19 + u * 7) % 13 < 3)
                        .map(move |u| (v, u))
                })
                .collect();
            let p = Pattern::from_edges(n, &edges);
            let seed: Vec<_> = (0..n).rev().collect();
            let pristine = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
            let mut game = Game::new(n, &pristine).unwrap();
            let before = game.replay_flops(&seed);
            for budget in [0, 1, 1_000, 10_000, 100_000, 10_000_000] {
                let first = subset_window_descent(n, &p.col_ptr, &p.row_idx, &seed, 8, 3, budget);
                assert_eq!(
                    first,
                    subset_window_descent(n, &p.col_ptr, &p.row_idx, &seed, 8, 3, budget)
                );
                if let Some(order) = first {
                    let mut sorted = order.clone();
                    sorted.sort_unstable();
                    assert_eq!(sorted, (0..n).collect::<Vec<_>>());
                    assert!(game.replay_flops(&order) < before);
                }
            }
        }
    }

    #[test]
    fn window_dp_rejects_malformed_inputs() {
        let p = Pattern::from_edges(4, &[(0, 1)]);
        for seed in [&[0, 1, 2][..], &[0, 1, 1, 3][..], &[0, 1, 2, 4][..]] {
            assert!(
                subset_window_descent(4, &p.col_ptr, &p.row_idx, seed, 8, 1, 1_000_000).is_none()
            );
        }
        assert!(subset_window_descent(
            4,
            &[0, 0, 2, 1, 2],
            &p.row_idx,
            &[0, 1, 2, 3],
            8,
            1,
            1_000_000
        )
        .is_none());
        assert!(subset_window_descent(
            4,
            &p.col_ptr,
            &p.row_idx,
            &[0, 1, 2, 3],
            MAX_WIDTH + 1,
            1,
            1_000_000
        )
        .is_none());
    }

    #[test]
    #[ignore]
    fn probe_window_dp_candidates() {
        let configs = [(8, 2, 16_000_000), (10, 2, 24_000_000), (12, 2, 32_000_000)];
        for (name, pattern) in crate::corpus::corpus() {
            if pattern.n > MAX_DIMENSION || pattern.nnz() > 200_000 {
                continue;
            }
            let incumbent = crate::ordering::order(&pattern);
            let before = ssi_scoring::score(&pattern, &incumbent).flops;
            for (width, sweeps, budget) in configs {
                let start = std::time::Instant::now();
                let candidate = subset_window_descent(
                    pattern.n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &incumbent,
                    width,
                    sweeps,
                    budget,
                );
                let elapsed = start.elapsed().as_secs_f64();
                let after = candidate.as_ref().map_or(before, |p| {
                    let mut sorted = p.clone();
                    sorted.sort_unstable();
                    assert_eq!(sorted, (0..pattern.n).collect::<Vec<_>>());
                    ssi_scoring::score(&pattern, p).flops
                });
                assert!(after <= before, "{name}: width {width}");
                println!(
                    "WINDOW_DP\t{name}\t{}\t{}\t{before}\t{after}\t{elapsed:.6}\t{width}",
                    pattern.n,
                    pattern.nnz()
                );
            }
            let start = std::time::Instant::now();
            let mut candidate = incumbent;
            for (width, sweeps, budget) in [configs[0], configs[2], configs[1]] {
                if let Some(next) = subset_window_descent(
                    pattern.n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &candidate,
                    width,
                    sweeps,
                    budget,
                ) {
                    candidate = next;
                }
            }
            let elapsed = start.elapsed().as_secs_f64();
            let after = ssi_scoring::score(&pattern, &candidate).flops;
            assert!(after <= before, "{name}: chained windows");
            println!(
                "WINDOW_DP\t{name}\t{}\t{}\t{before}\t{after}\t{elapsed:.6}\t0",
                pattern.n,
                pattern.nnz()
            );
        }
    }
}
