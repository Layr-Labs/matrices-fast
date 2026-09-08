//! Exact window search factored by connected components of the live graph.

use super::{Game, TripleWork};

const MAX_WIDTH: usize = 12;
const MAX_DIMENSION: usize = super::MAX_N;

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
        return None;
    }
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

fn refine_window(game: &Game<'_>, window: &mut [usize], work: &mut TripleWork) -> Option<bool> {
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
        let Some((order, best, incumbent)) = solve_component(game, &vertices, work) else {
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
    if n < 2
        || n > MAX_DIMENSION
        || !(2..=MAX_WIDTH).contains(&width)
        || sweeps == 0
        || budget <= 0
        || seed.len() != n
        || col_ptr.len() != n + 1
    {
        return None;
    }
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
    let mut current = seed.to_vec();
    let mut changed = false;
    for sweep in 0..sweeps {
        if !work.charge(2 * n * words + 8 * n) {
            return changed.then_some(current);
        }
        game.reset();
        let offset = (sweep * (width / 2).max(1)) % width;
        for &v in current.iter().take(offset.min(n)) {
            if !work.eliminate(&mut game, v) {
                return changed.then_some(current);
            }
        }
        let mut start = offset;
        while start + 1 < n {
            let end = (start + width).min(n);
            match refine_window(&game, &mut current[start..end], &mut work) {
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
    changed.then_some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pattern;

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
        assert!(
            subset_window_descent(4, &p.col_ptr, &p.row_idx, &[0, 1, 2, 3], 13, 1, 1_000_000)
                .is_none()
        );
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
