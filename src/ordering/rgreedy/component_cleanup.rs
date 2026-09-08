//! Reorder small connected components interleaved within a fixed live span.
//!
//! Different components of the CURRENT induced span cannot change one another's
//! pivot rows while all outside vertices stay live. Their eliminations commute,
//! even with shared outside neighbors; each internal order leaves the same
//! residual after the whole span. Small exact kernels can therefore be scattered
//! back into their original positions without touching intervening components.

use super::*;

const SPAN: usize = 32;

#[cfg(test)]
thread_local! {
    static CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

struct SpanProgress {
    changed: bool,
    complete: bool,
}

fn improve_span(game: &Game<'_>, span: &mut [usize], work: &mut TripleWork) -> SpanProgress {
    let h = span.len();
    let mut progress = SpanProgress {
        changed: false,
        complete: false,
    };
    // Precharge adjacency probes, bitset flood-fill, and component enumeration.
    if !work.charge(8 * h * h + 16 * h) {
        return progress;
    }
    let mut inside = [0u32; SPAN];
    for i in 0..h {
        for j in i + 1..h {
            if game.adj[span[i] * game.w + span[j] / 64] & (1 << (span[j] % 64)) != 0 {
                inside[i] |= 1 << j;
                inside[j] |= 1 << i;
            }
        }
    }
    let mut unseen = if h == SPAN { u32::MAX } else { (1u32 << h) - 1 };
    while unseen != 0 {
        let mut frontier = 1 << unseen.trailing_zeros();
        let mut component = 0u32;
        unseen &= !frontier;
        while frontier != 0 {
            let bit = frontier & frontier.wrapping_neg();
            frontier &= !bit;
            component |= bit;
            let next = inside[bit.trailing_zeros() as usize] & unseen;
            unseen &= !next;
            frontier |= next;
        }
        let size = component.count_ones() as usize;
        if !(2..=5).contains(&size) {
            continue;
        }
        // Reserve gathering, tie checks, and an atomic scatter in advance.
        if !work.charge(16 * size) {
            return progress;
        }
        let mut positions = [0usize; 5];
        let mut vertices = [0usize; 5];
        for i in 0..size {
            let position = component.trailing_zeros() as usize;
            component &= component - 1;
            positions[i] = position;
            vertices[i] = span[position];
        }
        let mut order = [0, 1, 2, 3, 4];
        let changed = match size {
            2 => {
                // A two-vertex component is an edge. The second width is
                // order-independent, so the smaller current degree goes first.
                if game.deg[vertices[1]] < game.deg[vertices[0]] {
                    order.swap(0, 1);
                    true
                } else {
                    false
                }
            }
            3 => {
                if !work.charge(20 * game.w + 192) {
                    return progress;
                }
                let costs = triple_costs(game, [vertices[0], vertices[1], vertices[2]]);
                let mut best = 0;
                for choice in 1..TRIPLE_ORDERS.len() {
                    if costs[choice] < costs[best] {
                        best = choice;
                    }
                }
                order[..3].copy_from_slice(&TRIPLE_ORDERS[best]);
                costs[best] < costs[0]
            }
            4 => {
                if !work.charge(four_window_work(game.w)) {
                    return progress;
                }
                let kernel =
                    FourWindow::new(game, [vertices[0], vertices[1], vertices[2], vertices[3]]);
                let (best_order, best, incumbent) = kernel.solve();
                order[..4].copy_from_slice(&best_order);
                best < incumbent
            }
            5 => {
                let Some(kernel) = FiveWindow::new(game, vertices, work) else {
                    return progress;
                };
                let (best_order, best, incumbent) = kernel.solve();
                order = best_order;
                best < incumbent
            }
            _ => unreachable!(),
        };
        if changed {
            for i in 0..size {
                span[positions[i]] = vertices[order[i]];
            }
            progress.changed = true;
        }
    }
    progress.complete = true;
    progress
}

/// Complementary span/K4 cleanup sharing one existing ticket and final score.
/// The span's scratch drops before K4 starts. Its unused credits, not a fresh
/// allowance, fund the inherited four-offset cycle.
pub(crate) fn descent(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
    budget: i64,
) -> Option<Vec<usize>> {
    let mut work = TripleWork { remaining: budget };
    let proposal = span_pass(n, cp, ri, seed, &mut work);
    let four = adjacent_four_descent(
        n,
        cp,
        ri,
        proposal.as_deref().unwrap_or(seed),
        work.remaining,
    );
    four.or(proposal)
}

fn span_pass(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
    work: &mut TripleWork,
) -> Option<Vec<usize>> {
    if n < 2 || n > MAX_N || work.remaining <= 0 || seed.len() != n || cp.len() != n + 1 {
        return None;
    }
    if !work.charge((n + 1).saturating_add(ri.len()).saturating_add(2 * n))
        || cp.first().copied() != Some(0)
        || cp.last().copied() != Some(ri.len())
        || cp.windows(2).any(|p| p[0] > p[1] || p[1] > ri.len())
        || ri.iter().any(|&v| v >= n)
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
    let build = n
        .saturating_mul(words)
        .saturating_add(2usize.saturating_mul(ri.len()))
        .saturating_add(n);
    if !work.charge(build) {
        return None;
    }
    let adj = Game::build_adj(n, cp, ri)?;
    let setup = 2usize
        .saturating_mul(n)
        .saturating_mul(words)
        .saturating_add(13usize.saturating_mul(n))
        .saturating_add(words);
    if !work.charge(setup) {
        return None;
    }
    let mut game = Game::new(n, &adj)?;
    let mut current = seed.to_vec();
    if !work.charge(game.reset_ops()) {
        return None;
    }
    game.reset();
    #[cfg(test)]
    CALLS.with(|count| count.set(count.get() + 1));
    let mut changed = false;
    for start in (0..n).step_by(SPAN) {
        let end = (start + SPAN).min(n);
        let progress = improve_span(&game, &mut current[start..end], work);
        changed |= progress.changed;
        if !progress.complete {
            return changed.then_some(current);
        }
        if end < n {
            for &v in &current[start..end] {
                if !work.eliminate(&mut game, v) {
                    return changed.then_some(current);
                }
            }
        }
    }
    changed.then_some(current)
}

#[cfg(test)]
mod tests {
    use super::super::super::{flops_of, is_bijection, order, Pattern, ScoringPattern};
    use super::*;

    fn oracle_eliminate(adj: &mut [Vec<bool>], v: usize) -> u64 {
        let neighbors: Vec<_> = adj[v]
            .iter()
            .enumerate()
            .filter_map(|(u, &edge)| edge.then_some(u))
            .collect();
        for &u in &neighbors {
            for &w in &neighbors {
                if u != w {
                    adj[u][w] = true;
                }
            }
            adj[u][v] = false;
        }
        adj[v].fill(false);
        (neighbors.len() as u64 + 1).pow(2)
    }

    #[test]
    fn scattered_components_match_boolean_replay_after_filled_prefixes() {
        let n = 130;
        let groups = [
            vec![10, 70],
            vec![11, 71, 12],
            vec![72, 13, 73, 14],
            vec![74, 15, 75, 16, 76],
        ];
        for seed in 1..=24 {
            let mut rng = seed;
            let mut original = vec![vec![false; n]; n];
            let mut edges = Vec::new();
            for (i, group) in groups.iter().enumerate() {
                edges.push((i, group[0]));
                edges.push((i, *group.last().unwrap()));
                edges.push((i, 100 + i));
                for pair in group.windows(2) {
                    edges.push((pair[0], pair[1]));
                }
                for &u in group {
                    edges.push((u, 129)); // Shared live boundary, not a span edge.
                    for v in 100..129 {
                        if xs64(&mut rng).is_multiple_of(3) {
                            edges.push((u, v));
                        }
                    }
                }
            }
            for &(u, v) in &edges {
                original[u][v] = true;
                original[v][u] = true;
            }
            let pat = Pattern::from_edges(n, &edges);
            let adj = Game::build_adj(n, &pat.col_ptr, &pat.row_idx).unwrap();
            let mut game = Game::new(n, &adj).unwrap();
            game.clique_pruning = true;
            game.reset();
            for v in 0..4 {
                game.eliminate(v);
                oracle_eliminate(&mut original, v);
            }
            let mut span = Vec::new();
            for i in 0..5 {
                for group in &groups {
                    if let Some(&v) = group.get(i) {
                        span.push(v);
                    }
                }
            }
            let old_span = span.clone();
            let progress = improve_span(
                &game,
                &mut span,
                &mut TripleWork {
                    remaining: i64::MAX,
                },
            );
            assert!(progress.complete);
            let mut old = original.clone();
            let mut new = original;
            let before: u64 = old_span
                .iter()
                .map(|&v| oracle_eliminate(&mut old, v))
                .sum();
            let after: u64 = span.iter().map(|&v| oracle_eliminate(&mut new, v)).sum();
            assert!(after <= before);
            assert_eq!(progress.changed, after < before);
            assert_eq!(old, new, "residual differs for seed {seed}");
        }
    }

    #[test]
    fn commuting_interveners_expose_a_seven_flop_kernel_gain() {
        let pat = Pattern::from_edges(40, &[(0, 7), (0, 33), (0, 35), (7, 34)]);
        let seed: Vec<_> = (0..40).collect();
        let scoring = ScoringPattern {
            n: 40,
            col_ptr: pat.col_ptr.clone(),
            row_idx: pat.row_idx.clone(),
        };
        let before = flops_of(&scoring, &seed);
        assert!(adjacent_four_descent(40, &pat.col_ptr, &pat.row_idx, &seed, 1_000_000).is_none());
        assert!(adjacent_five_descent(40, &pat.col_ptr, &pat.row_idx, &seed, 1_000_000).is_none());
        let candidate = descent(40, &pat.col_ptr, &pat.row_idx, &seed, 1_000_000).unwrap();
        assert_eq!(before - flops_of(&scoring, &candidate), 7);
        // Every integer cap across setup, census, first scatter and replay.
        for cap in 0..20_000 {
            if let Some(candidate) = descent(40, &pat.col_ptr, &pat.row_idx, &seed, cap) {
                assert!(is_bijection(&candidate, 40));
                assert_eq!(before - flops_of(&scoring, &candidate), 7);
            }
        }
    }

    #[test]
    fn complementary_stages_forward_only_remaining_credits() {
        let n = 81;
        let mut edges = Vec::new();
        for u in 0..n {
            if u % 9 < 8 {
                edges.push((u, u + 1));
            }
            if u + 9 < n {
                edges.push((u, u + 9));
            }
        }
        let pattern = Pattern::from_edges(n, &edges);
        let seed: Vec<_> = (0..n).rev().collect();
        for budget in [0, 1, 1_000, 10_000, 20_000, 50_000, 100_000] {
            let mut work = TripleWork { remaining: budget };
            let span = span_pass(n, &pattern.col_ptr, &pattern.row_idx, &seed, &mut work);
            assert!((0..=budget).contains(&work.remaining));
            let four = adjacent_four_descent(
                n,
                &pattern.col_ptr,
                &pattern.row_idx,
                span.as_deref().unwrap_or(&seed),
                work.remaining,
            );
            assert_eq!(
                descent(n, &pattern.col_ptr, &pattern.row_idx, &seed, budget),
                four.or(span)
            );
        }
    }

    #[test]
    fn nonforest_pipeline_reaches_component_cleanup() {
        // A connected nonchordal mesh avoids the forest/fill-free fast paths.
        let side = 9;
        let n = side * side;
        let mut edges = Vec::new();
        for u in 0..n {
            if u % side + 1 < side {
                edges.push((u, u + 1));
            }
            if u + side < n {
                edges.push((u, u + side));
            }
        }
        let pattern = Pattern::from_edges(n, &edges);
        CALLS.with(|count| count.set(0));
        let candidate = order(&pattern);
        assert!(is_bijection(&candidate, n));
        assert!(
            CALLS.with(|count| count.get()) > 0,
            "fixture bypassed replacement pass"
        );
        assert_eq!(order(&pattern), candidate);
    }
}
