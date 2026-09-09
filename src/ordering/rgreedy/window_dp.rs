//! Exact window search factored by connected components of the live graph.

use super::window_signatures::{ChargeModel, SignatureEngine};
use super::{Game, TripleWork};

const MAX_WIDTH: usize = 14;
const MAX_DIMENSION: usize = super::MAX_N;

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

fn solve_component(
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
    let mut path = vec![u64::MAX; states];
    best[0] = 0;
    path[0] = 0;
    for mask in 0..states - 1 {
        for pivot in 0..k {
            let bit = 1usize << pivot;
            if mask & bit != 0 {
                continue;
            }
            let next = mask | bit;
            let width = widths[components[next * k + pivot] as usize];
            let cost = best[mask] + width * width;
            let code = (path[mask] << 4) | pivot as u64;
            if cost < best[next] || (cost == best[next] && code < path[next]) {
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
    let order = if best[states - 1] < incumbent {
        (0..k)
            .map(|i| vertices[((path[states - 1] >> (4 * (k - i - 1))) & 15) as usize])
            .collect()
    } else {
        vertices.to_vec()
    };
    Some((order, best[states - 1], incumbent))
}

fn refine_window(
    game: &Game<'_>,
    window: &mut [usize],
    work: &mut TripleWork,
    engine: &mut Option<SignatureEngine>,
    charge_model: ChargeModel,
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
        #[cfg(test)]
        terminal_state("component", game, &vertices);
        let solution = if vertices.len() >= 5 && signature_cost < union_cost {
            let engine = engine.get_or_insert_with(|| SignatureEngine::new(game.n));
            engine.set_charge_model(charge_model);
            engine.solve_component(game, &vertices, work)
        } else {
            solve_component(game, &vertices, work)
        };
        #[cfg(test)]
        terminal_component(vertices.len() >= 5 && signature_cost < union_cost, &vertices, &solution);
        let Some((order, best, incumbent)) = solution else {
            return if changed { Some(true) } else { None };
        };
        if best < incumbent {
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
    )
}

/// Owns only the immutable input graph, never a working Game or signature memo.
/// Binding the input borrows here prevents reuse for a different graph.
pub(crate) struct TerminalAdjacency<'a> {
    n: usize,
    col_ptr: &'a [usize],
    row_idx: &'a [usize],
    pristine: Option<Vec<u64>>,
}

impl<'a> TerminalAdjacency<'a> {
    pub(crate) fn new(n: usize, col_ptr: &'a [usize], row_idx: &'a [usize]) -> Self {
        Self { n, col_ptr, row_idx, pristine: None }
    }

    pub(crate) fn parity(&mut self, seed: &[usize], width: usize, sweeps: usize,
                         budget: i64) -> Option<Vec<usize>> {
        subset_window_descent_owned(self.n, self.col_ptr, self.row_idx, seed,
            width, sweeps, (width / 2).max(1), budget, ChargeModel::UnionParity,
            &mut self.pristine, false)
    }

    // Consuming the owner makes the credited call terminal in its chain.
    pub(crate) fn finish(mut self, seed: &[usize], width: usize, sweeps: usize,
                         offset_step: usize, budget: i64) -> Option<Vec<usize>> {
        subset_window_descent_owned(self.n, self.col_ptr, self.row_idx, seed,
            width, sweeps, offset_step, budget, ChargeModel::SignatureTrue,
            &mut self.pristine, true)
    }
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
) -> Option<Vec<usize>> {
    subset_window_descent_owned(n, col_ptr, row_idx, seed, width, sweeps,
        offset_step, budget, charge_model, &mut None, false)
}

fn subset_window_descent_owned(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    offset_step: usize,
    budget: i64,
    charge_model: ChargeModel,
    owner: &mut Option<Vec<u64>>,
    final_credit: bool,
) -> Option<Vec<usize>> {
    if n < 2
        || n > MAX_DIMENSION
        || !(2..=MAX_WIDTH).contains(&width)
        || offset_step >= width
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
    // The first three calls deliberately retain their full original charges.
    // Only an actual final hit avoids (and credits) pristine zero-initialization.
    let credit = if final_credit && owner.is_some() {
        n.checked_mul(words)?
    } else {
        0
    };
    if !work.charge(setup.checked_sub(credit)?) {
        return None;
    }
    if owner.is_none() {
        *owner = Some(Game::build_adj(n, col_ptr, row_idx)?);
    }
    let mut game = Game::new(n, owner.as_deref()?)?;
    let mut engine = None;
    let mut current = seed.to_vec();
    let mut changed = false;
    for sweep in 0..sweeps {
        #[cfg(test)]
        terminal_state("reset", &game, &current);
        if !work.charge(2 * n * words + 8 * n) {
            return changed.then_some(current);
        }
        game.reset();
        let offset = (sweep * offset_step) % width;
        for &v in current.iter().take(offset.min(n)) {
            if !work.eliminate(&mut game, v) {
                return changed.then_some(current);
            }
        }
        let mut start = offset;
        while start + 1 < n {
            let end = (start + width).min(n);
            #[cfg(test)]
            terminal_state("window", &game, &current);
            match refine_window(
                &game,
                &mut current[start..end],
                &mut work,
                &mut engine,
                charge_model,
            ) {
                Some(improved) => changed |= improved,
                None => return changed.then_some(current),
            }
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

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TerminalEvent {
    Charge { cost: usize, remaining: i64, admitted: bool },
    State { label: &'static str, vertices: Vec<usize>, adj: Vec<(usize, u64)>, deg: Vec<u32>,
            live: Vec<u32>, pos: Vec<u32>, buckets: Vec<i32>, next: Vec<i32>,
            prev: Vec<i32>, mind: usize, nlive: usize, ops: i64 },
    Component { signature: bool, vertices: Vec<usize>, solution: Option<(Vec<usize>, u64, u64)> },
}
#[cfg(test)]
thread_local! {
    static TERMINAL_TRACE: std::cell::RefCell<Option<Vec<TerminalEvent>>> = const { std::cell::RefCell::new(None) };
}
#[cfg(test)]
pub(super) fn terminal_charge(cost: usize, remaining: i64) {
    TERMINAL_TRACE.with(|cell| {
        if let Some(trace) = cell.borrow_mut().as_mut() {
            trace.push(TerminalEvent::Charge { cost, remaining,
                admitted: i64::try_from(cost).is_ok_and(|c| c <= remaining) });
        }
    });
}
#[cfg(test)]
pub(super) fn terminal_state(label: &'static str, game: &Game<'_>, vertices: &[usize]) {
    TERMINAL_TRACE.with(|cell| {
        if let Some(trace) = cell.borrow_mut().as_mut() {
            trace.push(TerminalEvent::State { label, vertices: vertices.to_vec(),
                adj: game.adj.iter().copied().enumerate().filter(|(_, word)| *word != 0).collect(), deg: game.deg.clone(), live: game.livelist.clone(),
                pos: game.pos.clone(), buckets: game.bhead.clone(), next: game.bnext.clone(),
                prev: game.bprev.clone(), mind: game.mind, nlive: game.nlive, ops: game.ops });
        }
    });
}
#[cfg(test)]
fn terminal_component(signature: bool, vertices: &[usize], solution: &Option<(Vec<usize>, u64, u64)>) {
    TERMINAL_TRACE.with(|cell| {
        if let Some(trace) = cell.borrow_mut().as_mut() {
            trace.push(TerminalEvent::Component { signature, vertices: vertices.to_vec(), solution: solution.clone() });
        }
    });
}

#[cfg(test)]
// Mechanically extracted from the pinned source; only test observation points added.
fn original_pinned_config(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    offset_step: usize,
    budget: i64,
    charge_model: ChargeModel,
) -> Option<Vec<usize>> {
    if n < 2
        || n > MAX_DIMENSION
        || !(2..=MAX_WIDTH).contains(&width)
        || offset_step >= width
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
        #[cfg(test)]
        terminal_state("reset", &game, &current);
        if !work.charge(2 * n * words + 8 * n) {
            return changed.then_some(current);
        }
        game.reset();
        let offset = (sweep * offset_step) % width;
        for &v in current.iter().take(offset.min(n)) {
            if !work.eliminate(&mut game, v) {
                return changed.then_some(current);
            }
        }
        let mut start = offset;
        while start + 1 < n {
            let end = (start + width).min(n);
            #[cfg(test)]
            terminal_state("window", &game, &current);
            match refine_window(
                &game,
                &mut current[start..end],
                &mut work,
                &mut engine,
                charge_model,
            ) {
                Some(improved) => changed |= improved,
                None => return changed.then_some(current),
            }
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
    #[cfg(test)]
    {
        report.completed = true;
    }
    changed.then_some(current)
}

#[cfg(test)]
mod implementation28_terminal_tests {
    use super::*;
    use crate::Pattern;
    fn diagnostic(message: String) {
        use std::io::Write;
        writeln!(std::io::stdout(), "{message}").unwrap();
    }

    fn traced(f: impl FnOnce() -> Option<Vec<usize>>) -> (Option<Vec<usize>>, Vec<TerminalEvent>) {
        TERMINAL_TRACE.with(|c| *c.borrow_mut() = Some(Vec::new()));
        let result = f();
        let events = TERMINAL_TRACE.with(|c| c.borrow_mut().take().unwrap());
        (result, events)
    }
    fn original(p: &Pattern, seed: &[usize], width: usize, sweeps: usize, step: usize,
                budget: i64, model: ChargeModel) -> Option<Vec<usize>> {
        original_pinned_config(p.n, &p.col_ptr, &p.row_idx, seed, width, sweeps, step, budget, model)
    }
    fn exact(p: &Pattern, seed: &[usize], result: &Option<Vec<usize>>, trace: &[TerminalEvent]) -> (usize, usize, u64) {
        let out = result.as_deref().unwrap_or(seed);
        let mut sorted = out.to_vec(); sorted.sort_unstable();
        assert_eq!(sorted, (0..p.n).collect::<Vec<_>>());
        let before = ssi_scoring::score(p, seed).flops;
        let after = ssi_scoring::score(p, out).flops;
        let pristine = Game::build_adj(p.n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(p.n, &pristine).unwrap();
        assert_eq!(before, game.replay_flops(seed));
        assert_eq!(after, game.replay_flops(out));
        let mut completed = 0; let mut refused = 0; let mut gain = 0;
        for event in trace {
            if let TerminalEvent::Component { solution, .. } = event {
                if let Some((_, best, incumbent)) = solution {
                    completed += 1;
                    assert!(best <= incumbent);
                    gain += incumbent - best;
                } else { refused += 1; }
            }
        }
        assert!(after <= before);
        assert_eq!(before - after, gain, "sum of accepted component gains equals independent exact total delta");
        (completed, refused, gain)
    }
    fn prefix(base: &[TerminalEvent], hit: &[TerminalEvent], credit: i64) {
        let mut charges = 0;
        for (i, b) in base.iter().enumerate() {
            let mut h = hit.get(i).expect("lost funded baseline operation").clone();
            if let TerminalEvent::Charge { cost, remaining, .. } = &mut h {
                if charges == 1 { *cost += credit as usize; }
                if charges > 1 { *remaining -= credit; }
                charges += 1;
            }
            if let TerminalEvent::Charge { admitted: false, cost, remaining } = b {
                if let TerminalEvent::Charge { cost: hc, remaining: hr, .. } = h {
                    assert_eq!((*cost, *remaining), (hc, hr));
                } else { panic!("different refusal operation"); }
                return;
            }
            assert_eq!(b, &h, "trajectory divergence at event {i}");
        }
        assert_eq!(base.len(), hit.len(), "completed baseline must retain its schedule");
    }
    fn setup(p: &Pattern) -> i64 {
        let n = p.n; let w = n.div_ceil(64);
        (n + 1 + p.row_idx.len() + 2*n + 3*n*w + 2*p.row_idx.len() + 16*n + w) as i64
    }
    fn fixture(n: usize, kind: usize) -> Pattern {
        let mut edges = Vec::new();
        for v in 0..n {
            if v % 12 == 0 || (kind == 0 && v % 12 == 6) {
                for u in v+1..(v+6).min(n) { edges.push((v,u)); }
            }
            if kind == 1 && v+12 < n { edges.push((v,v+12)); }
            if kind == 2 && v+1 < n { edges.push((v,v+1)); }
        }
        Pattern::from_edges(n, &edges)
    }
    #[test]
    fn implementation28_lazy_validation_hit_miss() {
        let p = fixture(65, 1); let seed: Vec<_> = (0..p.n).collect();
        let mut owner = TerminalAdjacency::new(p.n, &p.col_ptr, &p.row_idx);
        for budget in [0, 1, setup(&p)-1] {
            assert!(owner.parity(&seed, 8, 2, budget).is_none());
            assert!(owner.pristine.is_none());
        }
        let mut bad = seed.clone(); bad[1] = bad[0];
        assert!(owner.parity(&bad, 8, 2, i64::MAX).is_none());
        assert!(owner.pristine.is_none());
        assert!(owner.parity(&seed, 8, 2, setup(&p)).is_none());
        let ptr = owner.pristine.as_ref().unwrap().as_ptr();
        for _ in 0..2 {
            let _ = owner.parity(&seed, 12, 2, setup(&p));
            assert_eq!(ptr, owner.pristine.as_ref().unwrap().as_ptr());
        }
        assert!(owner.parity(&bad, 8, 2, i64::MAX).is_none());
        for budget in [1, setup(&p)-1, setup(&p), 100_000] {
            let baseline = traced(|| original(&p, &seed, 12, 4, 5, budget, ChargeModel::SignatureTrue));
            let miss = traced(|| TerminalAdjacency::new(p.n, &p.col_ptr, &p.row_idx).finish(&seed, 12, 4, 5, budget));
            assert_eq!(baseline, miss, "miss must pay full setup");
        }
        for (n, cp, ri, s) in [
            (4, vec![0], vec![], vec![0,1,2,3]),
            (4, vec![0,0,2,1,2], vec![0,1], vec![0,1,2,3]),
            (4, vec![0,0,0,0,1], vec![4], vec![0,1,2,3]),
            (MAX_DIMENSION+1, vec![], vec![], vec![]),
        ] {
            let mut invalid = TerminalAdjacency::new(n, &cp, &ri);
            assert!(invalid.parity(&s, 8, 2, i64::MAX).is_none());
            assert!(invalid.pristine.is_none());
        }
    }
    #[test]
    fn implementation28_paired_terminal_trajectories() {
        let mut extra = 0; let mut extra_gain = 0; let mut completions = [0;2]; let mut refusals = [0;2];
        let mut branches = [false;2]; let mut partial = false;
        for n in [24, 65, 257] {
            for kind in 0..3 {
                let p = fixture(n, kind);
                for allowance in [0, 1, setup(&p)-1, setup(&p)+3000, 100_000, 1_000_000, 64_000_000] {
                    let mut owner = TerminalAdjacency::new(n, &p.col_ptr, &p.row_idx);
                    let mut seed: Vec<_> = (0..n).collect();
                    for (width,budget) in [(8,16_000_000), (12,32_000_000), (10,24_000_000)] {
                        let b = traced(|| original(&p, &seed, width, 2, width/2, budget, ChargeModel::UnionParity));
                        let h = traced(|| owner.parity(&seed, width, 2, budget));
                        assert_eq!(b,h, "first three charges, states, solves, and outputs");
                        exact(&p, &seed, &h.0, &h.1);
                        if let Some(next) = h.0 {
                            if ssi_scoring::score(&p,&next).flops < ssi_scoring::score(&p,&seed).flops { seed=next; }
                        }
                    }
                    let b = traced(|| original(&p, &seed, 12, 4, 5, allowance, ChargeModel::SignatureTrue));
                    let h = traced(|| owner.finish(&seed, 12, 4, 5, allowance));
                    let credit = (n*n.div_ceil(64)) as i64;
                    prefix(&b.1, &h.1, credit);
                    // Once validation/setup is admitted, exactly the credit explains
                    // the entire extended run, not only its output or a prefix hash.
                    if allowance >= setup(&p) {
                        let extended = traced(|| original(&p, &seed, 12, 4, 5, allowance+credit, ChargeModel::SignatureTrue));
                        assert_eq!(extended.0,h.0);
                        let mut normalized = extended.1.clone(); let mut charges=0;
                        for e in &mut normalized {
                            if let TerminalEvent::Charge { cost, remaining, .. } = e {
                                if charges < 2 { *remaining-=credit; }
                                if charges == 1 { *cost-=credit as usize; }
                                charges+=1;
                            }
                        }
                        assert_eq!(normalized,h.1);
                    }
                    let bc = exact(&p,&seed,&b.0,&b.1); let hc = exact(&p,&seed,&h.0,&h.1);
                    completions[0]+=bc.0; completions[1]+=hc.0;
                    refusals[0]+=bc.1; refusals[1]+=hc.1;
                    extra += hc.0.saturating_sub(bc.0);
                    extra_gain += hc.2.saturating_sub(bc.2);
                    assert!(hc.2 >= bc.2, "must preserve partial-window gains");
                    for e in &h.1 {
                        if let TerminalEvent::Component { signature, solution: Some(_), .. } = e { branches[usize::from(*signature)]=true; }
                    }
                    partial |= bc.1 > 0 && bc.2 > 0;
                }
            }
        }
        // Do not require a synthetic gain: counters describe actual executed work.
        diagnostic(format!("implementation28 synthetic terminal completions={completions:?} refusals={refusals:?} extra={extra} extra_exact_gain={extra_gain} branches={branches:?} partial={partial}; public components unmeasured"));
        assert!(branches[0]); // Signature arm is required separately below at cross-word scale.
    }
    #[test]
    fn implementation28_signature_refusal_trajectory() {
        let n: usize = 1729;
        let edges: Vec<_> = [0, 6].into_iter().flat_map(|v| (v+1..v+6).map(move |u| (v,u))).collect();
        let p=Pattern::from_edges(n,&edges); let seed: Vec<_>=(0..n).collect();
        let w=n.div_ceil(64);
        let (union,signature)=SignatureEngine::charge_costs(6,w,10);
        assert!(signature<union);
        let mut completed=[0usize;2]; let mut refused=[0usize;2]; let mut gains=[0u64;2];
        for extra in [signature as i64-1, signature as i64+1] {
            let budget=setup(&p)+(2*n*w+8*n+8*12*12+8*12) as i64+extra;
            let mut owner=TerminalAdjacency::new(n,&p.col_ptr,&p.row_idx);
            // The same full setup admission is used to warm the chain; these
            // deliberately truncated calls leave the seed unchanged.
            for width in [8,12,10] {
                let b=traced(|| original(&p,&seed,width,2,width/2,setup(&p),ChargeModel::UnionParity));
                let h=traced(|| owner.parity(&seed,width,2,setup(&p)));
                assert_eq!(b,h);
            }
            let b=traced(|| original(&p,&seed,12,4,5,budget,ChargeModel::SignatureTrue));
            let h=traced(|| owner.finish(&seed,12,4,5,budget));
            prefix(&b.1,&h.1,(n*w) as i64);
            let bc=exact(&p,&seed,&b.0,&b.1); let hc=exact(&p,&seed,&h.0,&h.1);
            assert!(hc.2>=bc.2);
            for (i,t) in [&b.1,&h.1].into_iter().enumerate() {
                for e in t {
                    if let TerminalEvent::Component {signature:true,solution,..}=e {
                        if let Some((_,best,incumbent))=solution { completed[i]+=1; gains[i]+=incumbent-best; }
                        else { refused[i]+=1; }
                    }
                }
            }
        }
        diagnostic(format!("implementation28 signature synthetic completed={completed:?} refused={refused:?} exact_gains={gains:?}; not public-score evidence"));
        assert!(completed[1]>0 && refused[0]>0);
        assert!(gains[1]>=gains[0]);
    }
    #[test]
    fn implementation28_refusal_boundaries_and_partial_gains() {
        let mut extra=0; let mut improving=0; let mut partial=0; let mut counts=[0usize;4];
        // Two star components in the same window: the first gain must survive a
        // refusal in the second. Cross-word dimension selects both solver arms.
        for n in [24,257] {
            let p=fixture(n,0); let seed: Vec<_>=(0..n).collect();
            let full=traced(|| original(&p,&seed,12,4,5,1_000_000,ChargeModel::SignatureTrue));
            let mut spent=0i64; let mut boundaries=Vec::new();
            for event in &full.1 {
                if let TerminalEvent::Charge {cost,admitted:true,..}=event {
                    spent+=*cost as i64;
                    if spent>=setup(&p) { boundaries.push(spent-1); }
                }
            }
            for budget in boundaries.into_iter().take(120) {
                let mut owner=TerminalAdjacency::new(n,&p.col_ptr,&p.row_idx);
                let _=owner.parity(&seed,8,2,setup(&p));
                let b=traced(|| original(&p,&seed,12,4,5,budget,ChargeModel::SignatureTrue));
                let h=traced(|| owner.finish(&seed,12,4,5,budget));
                prefix(&b.1,&h.1,(n*n.div_ceil(64)) as i64);
                let bc=exact(&p,&seed,&b.0,&b.1); let hc=exact(&p,&seed,&h.0,&h.1);
                assert!(hc.2>=bc.2);
                counts[0]+=bc.0; counts[1]+=hc.0; counts[2]+=bc.1; counts[3]+=hc.1;
                extra+=hc.0.saturating_sub(bc.0); improving+=usize::from(hc.2>bc.2);
                partial+=usize::from(bc.1>0 && bc.2>0);
                let mut repeat=TerminalAdjacency::new(n,&p.col_ptr,&p.row_idx);
                let _=repeat.parity(&seed,8,2,setup(&p));
                assert_eq!(h,traced(|| repeat.finish(&seed,12,4,5,budget)));
            }
        }
        diagnostic(format!("implementation28 synthetic boundary completed_base/hit refused_base/hit={counts:?} extra_components={extra} improving_runs={improving} retained_partial_runs={partial}; not public-score evidence"));
        assert!(partial>0, "exercise partial-window return rather than only plumbing");
    }
}
