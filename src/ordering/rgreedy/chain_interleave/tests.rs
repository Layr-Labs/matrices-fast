use super::*;
use super::super::super::{flops_of, is_bijection, Pattern, ScoringPattern};

type Graph = Vec<Vec<bool>>;

fn graph_from_edges(n: usize, edges: &[(usize, usize)]) -> Graph {
    let mut graph = vec![vec![false; n]; n];
    for &(u, v) in edges {
        if u != v {
            graph[u][v] = true;
            graph[v][u] = true;
        }
    }
    graph
}

// Independent Boolean clique elimination: no snapshot/component formula or DP.
fn oracle_eliminate(graph: &mut Graph, pivot: usize) -> u64 {
    let neighbors: Vec<_> = graph[pivot]
        .iter()
        .enumerate()
        .filter_map(|(v, &edge)| edge.then_some(v))
        .collect();
    for &u in &neighbors {
        graph[u][pivot] = false;
        for &v in &neighbors {
            if u != v {
                graph[u][v] = true;
            }
        }
    }
    graph[pivot].fill(false);
    neighbors.len() as u64 + 1
}

fn oracle_replay(graph: &Graph, order: &[usize]) -> (u64, Graph) {
    let mut residual = graph.clone();
    let mut cost = 0;
    for &pivot in order {
        let width = oracle_eliminate(&mut residual, pivot);
        cost += width * width;
    }
    (cost, residual)
}

fn canonical(pat: &Pattern, order: &[usize]) -> u64 {
    flops_of(
        &ScoringPattern {
            n: pat.n,
            col_ptr: pat.col_ptr.clone(),
            row_idx: pat.row_idx.clone(),
        },
        order,
    )
}

// Enumerate the literal search family: unselected positions retain their order,
// while each selected position can occur anywhere and in either selected order.
fn interleavings(m: usize, selected: &[usize]) -> Vec<Vec<usize>> {
    fn visit(
        chain: &[usize],
        selected: &[usize],
        chain_at: usize,
        selected_used: u8,
        order: &mut Vec<usize>,
        out: &mut Vec<Vec<usize>>,
    ) {
        if order.len() == chain.len() + selected.len() {
            out.push(order.clone());
            return;
        }
        if chain_at < chain.len() {
            order.push(chain[chain_at]);
            visit(chain, selected, chain_at + 1, selected_used, order, out);
            order.pop();
        }
        for (slot, &pivot) in selected.iter().enumerate() {
            if selected_used & (1 << slot) == 0 {
                order.push(pivot);
                visit(
                    chain,
                    selected,
                    chain_at,
                    selected_used | (1 << slot),
                    order,
                    out,
                );
                order.pop();
            }
        }
    }
    let chain: Vec<_> = (0..m).filter(|v| !selected.contains(v)).collect();
    let mut out = Vec::new();
    visit(&chain, selected, 0, 0, &mut Vec::new(), &mut out);
    out
}

fn assert_chain(order: &[usize], m: usize, selected: &[usize]) {
    assert!(is_bijection(order, m));
    let expected: Vec<_> = (0..m).filter(|v| !selected.contains(v)).collect();
    let actual: Vec<_> = order
        .iter()
        .copied()
        .filter(|v| !selected.contains(v))
        .collect();
    assert_eq!(actual, expected, "changed the order of the fixed chain");
}

fn verify_case(
    n: usize,
    edges: &[(usize, usize)],
    prefix: &[usize],
    window: &[usize],
    selected: &[usize],
) {
    let pat = Pattern::from_edges(n, edges);
    let adj = Game::build_adj(n, &pat.col_ptr, &pat.row_idx).unwrap();
    let mut game = Game::new(n, &adj).unwrap();
    game.reset();
    let mut graph = graph_from_edges(n, edges);
    for &pivot in prefix {
        assert_eq!(game.deg[pivot] as u64 + 1, oracle_eliminate(&mut graph, pivot));
        game.eliminate(pivot);
    }
    let mut work = TripleWork { remaining: 100_000_000 };
    let snapshot = Snapshot::new(&game, window, &mut work).unwrap();

    // All residual subsets, including ones the chosen chain does not reach.
    assert!(window.len() <= 12);
    for mask in 0u32..(1u32 << window.len()) {
        let mut residual = graph.clone();
        for (local, &vertex) in window.iter().enumerate() {
            if mask & (1 << local) != 0 {
                oracle_eliminate(&mut residual, vertex);
            }
        }
        for (local, &vertex) in window.iter().enumerate() {
            if mask & (1 << local) == 0 {
                let width = residual[vertex].iter().filter(|&&edge| edge).count() as u64 + 1;
                assert_eq!(
                    snapshot.width(mask, local), width,
                    "n={n} prefix={prefix:?} window={window:?} mask={mask} pivot={local}"
                );
            }
        }
    }

    let (before, reference_residual) = oracle_replay(&graph, window);
    let mut optimum = u64::MAX;
    for local_order in interleavings(window.len(), selected) {
        let order: Vec<_> = local_order.iter().map(|&v| window[v]).collect();
        let (cost, residual) = oracle_replay(&graph, &order);
        optimum = optimum.min(cost);
        assert_eq!(residual, reference_residual, "window changed the suffix graph");
    }
    let solution = snapshot.solve(selected, &mut work).unwrap();
    let local_order = &solution.order[..window.len()];
    assert_chain(local_order, window.len(), selected);
    let chosen: Vec<_> = local_order.iter().map(|&v| window[v]).collect();
    let (chosen_cost, chosen_residual) = oracle_replay(&graph, &chosen);
    assert_eq!(solution.before, before);
    assert_eq!(solution.after, optimum);
    assert_eq!(chosen_cost, optimum);
    assert_eq!(chosen_residual, reference_residual);
    if optimum == before {
        assert_eq!(local_order, &(0..window.len()).collect::<Vec<_>>());
    }
    assert!(work.remaining >= 0);

    let suffix: Vec<_> = (0..n)
        .filter(|v| !prefix.contains(v) && !window.contains(v))
        .collect();
    let mut original = prefix.to_vec();
    original.extend_from_slice(window);
    original.extend_from_slice(&suffix);
    let mut reordered = prefix.to_vec();
    reordered.extend_from_slice(&chosen);
    reordered.extend_from_slice(&suffix);
    assert!(is_bijection(&reordered, n));
    assert_eq!(
        canonical(&pat, &original) - canonical(&pat, &reordered),
        before - optimum,
        "local gain disagrees with the independent production scorer"
    );
}

#[test]
fn interleave_dp_matches_literal_chain_oracle() {
    for n in 5..=10 {
        let mut edges = Vec::new();
        for u in 0..n {
            for v in u + 1..n {
                // A bounded deterministic family, independent of the production search.
                if (u * 17 + v * 11 + n * 3) % 7 < 3 {
                    edges.push((u, v));
                }
            }
        }
        let window: Vec<_> = (0..n - 1).rev().collect();
        for selected in [vec![1], vec![0, window.len() - 1], vec![0, 2, window.len() - 1]] {
            verify_case(n, &edges, &[], &window, &selected);
        }
    }
}

#[test]
fn interleave_handles_live_boundary_and_filled_prefix() {
    // A live shared boundary vertex cannot connect two eliminated components.
    verify_case(8, &[(0, 6), (2, 6), (1, 7), (4, 7), (6, 7)], &[], &[0, 1, 2, 3, 4, 5], &[0, 2, 4]);
    // Eliminating the prefix changes that: its neighbors now have actual fill edges.
    let edges = [
        (0, 2), (0, 4), (0, 7), (1, 3), (1, 5), (1, 8),
        (2, 6), (3, 6), (4, 9), (5, 9), (6, 7), (7, 8), (8, 9),
    ];
    verify_case(10, &edges, &[0, 1], &[7, 2, 8, 3, 4, 5], &[1, 3, 5]);
}

#[test]
fn interleave_handles_cross_word_labels() {
    let edges = [
        (1, 0), (1, 63), (1, 128), (2, 64), (2, 127), (2, 129),
        (0, 3), (63, 3), (64, 3), (127, 4), (128, 4), (129, 4),
        (3, 4), (0, 64), (63, 129), (127, 128),
    ];
    verify_case(130, &edges, &[1, 2], &[0, 63, 64, 127, 128, 129], &[0, 3, 5]);
}

#[test]
fn interleave_ties_retain_literal_incumbent() {
    for clique in [false, true] {
        let edges: Vec<_> = if clique {
            (0..7).flat_map(|u| (u + 1..7).map(move |v| (u, v))).collect()
        } else {
            Vec::new()
        };
        verify_case(7, &edges, &[], &[5, 2, 6, 0, 4, 1, 3], &[0, 3, 6]);
    }
}

#[test]
fn interleave_nonchordal_k6_10_witness() {
    let edges: Vec<_> = (0..6).flat_map(|u| (6..16).map(move |v| (u, v))).collect();
    let pat = Pattern::from_edges(16, &edges);
    let adj = Game::build_adj(16, &pat.col_ptr, &pat.row_idx).unwrap();
    let mut game = Game::new(16, &adj).unwrap();
    game.reset();
    let window: Vec<_> = (0..12).collect();
    let mut work = TripleWork { remaining: 100_000_000 };
    let snapshot = Snapshot::new(&game, &window, &mut work).unwrap();
    let solution = snapshot.solve(&[6, 8, 10], &mut work).unwrap();
    assert_chain(&solution.order[..12], 12, &[6, 8, 10]);
    let original: Vec<_> = (0..16).collect();
    let mut candidate = solution.order[..12].to_vec();
    candidate.extend(12..16);
    assert_eq!(canonical(&pat, &original), 1111);
    assert_eq!(canonical(&pat, &candidate), 966);
    assert_eq!(solution.before - solution.after, 145);
    verify_case(16, &edges, &[], &window, &[6, 8, 10]);
}

#[test]
fn interleave_snapshot_boundary_limit_and_full_u32_mask() {
    let n = 545;
    let window: Vec<_> = (0..32).collect();
    let mut edges: Vec<_> = (0..31).map(|v| (v, v + 1)).collect();
    edges.extend((32..544).map(|v| (0, v)));
    let pat = Pattern::from_edges(n, &edges);
    let adj = Game::build_adj(n, &pat.col_ptr, &pat.row_idx).unwrap();
    let mut game = Game::new(n, &adj).unwrap();
    game.reset();
    let mut work = TripleWork { remaining: 100_000_000 };
    let snapshot = Snapshot::new(&game, &window, &mut work).expect("512 boundary vertices fit");
    assert_eq!(snapshot.width(0, 31), 2);
    assert_eq!(snapshot.width(1u32 << 31, 30), 2);
    assert_eq!(snapshot.width(u32::MAX ^ 1, 0), 513);
    let solution = snapshot.solve(&[31], &mut work).unwrap();
    assert_chain(&solution.order, 32, &[31]);
    assert!(solution.after <= solution.before);
    assert!(work.remaining >= 0);

    edges.push((31, 544));
    let pat = Pattern::from_edges(n, &edges);
    let adj = Game::build_adj(n, &pat.col_ptr, &pat.row_idx).unwrap();
    let mut game = Game::new(n, &adj).unwrap();
    game.reset();
    let mut work = TripleWork { remaining: 100_000_000 };
    assert!(Snapshot::new(&game, &window, &mut work).is_none(), "513 boundary vertices must refuse");
    assert!(work.remaining >= 0);
}

#[test]
fn interleave_kernel_refuses_before_unfunded_work() {
    let pat = Pattern::from_edges(8, &[(0, 1), (1, 2), (2, 3), (3, 0), (0, 7)]);
    let adj = Game::build_adj(8, &pat.col_ptr, &pat.row_idx).unwrap();
    let mut game = Game::new(8, &adj).unwrap();
    game.reset();
    let mut empty = TripleWork { remaining: 0 };
    assert!(Snapshot::new(&game, &[0, 1, 2, 3, 4, 5], &mut empty).is_none());
    assert_eq!(empty.remaining, 0);
    let mut funded = TripleWork { remaining: 100_000_000 };
    let snapshot = Snapshot::new(&game, &[0, 1, 2, 3, 4, 5], &mut funded).unwrap();
    assert!(snapshot.solve(&[0, 2, 4], &mut empty).is_none());
    assert_eq!(empty.remaining, 0);
}


#[test]
fn interleave_selects_early_and_nonadjacent_high_degree_vertices() {
    // Position zero is isolated. The top degrees occur at local positions 1, 3, 6:
    // both the old minimum-position and adjacency predicates would exclude them.
    let edges = [
        (1, 8), (1, 9), (1, 10), (1, 11),
        (3, 8), (3, 9), (3, 10),
        (6, 8), (6, 9),
    ];
    let pat = Pattern::from_edges(12, &edges);
    let adj = Game::build_adj(12, &pat.col_ptr, &pat.row_idx).unwrap();
    let mut game = Game::new(12, &adj).unwrap();
    game.reset();
    let window = [0, 1, 2, 3, 4, 5, 6, 7];
    assert_eq!(game.deg[0], 0);
    assert_eq!(select_free_vertices(&game, &window), Some([1, 3, 6]));
}

#[test]
fn interleave_selection_ties_use_positions_and_keep_regular_tiles() {
    for empty in [false, true] {
        let edges: Vec<_> = if empty {
            Vec::new()
        } else {
            // An equal-degree, nonchordal cycle must still receive a free set.
            (0..8).map(|v| (v, (v + 1) % 8)).collect()
        };
        let pat = Pattern::from_edges(8, &edges);
        let adj = Game::build_adj(8, &pat.col_ptr, &pat.row_idx).unwrap();
        let mut game = Game::new(8, &adj).unwrap();
        game.reset();
        let window = [7, 6, 5, 4, 3, 2, 1, 0];
        assert!(window.iter().all(|&v| game.deg[v] == game.deg[window[0]]));
        let first = select_free_vertices(&game, &window);
        assert_eq!(first, Some([0, 1, 2]));
        assert_eq!(select_free_vertices(&game, &window), first);
    }
}

#[test]
fn interleave_selection_uses_current_filled_degrees() {
    let edges = [(0, 1), (0, 2), (0, 3), (0, 4), (5, 6), (5, 7)];
    let pat = Pattern::from_edges(9, &edges);
    let adj = Game::build_adj(9, &pat.col_ptr, &pat.row_idx).unwrap();
    let mut game = Game::new(9, &adj).unwrap();
    game.reset();
    let window = [5, 1, 2, 3, 4, 6, 7, 8];
    assert_eq!(select_free_vertices(&game, &window), Some([0, 1, 2]));
    game.eliminate(0);
    // Prefix elimination completes 1..4 into a clique. Their current degrees
    // now outrank vertex 5, even though their original degrees were smaller.
    assert_eq!(select_free_vertices(&game, &window), Some([1, 2, 3]));
}

#[test]
fn interleave_outer_is_deterministic_budgeted_and_strict() {
    let n = 16;
    // An induced four-cycle makes the graph nonchordal. Deferring the universal
    // hub avoids completing its fifteen neighbors into a clique.
    let mut edges: Vec<_> = (1..n).map(|v| (0, v)).collect();
    edges.extend([(1, 2), (2, 3), (3, 4), (4, 1)]);
    let pat = Pattern::from_edges(n, &edges);
    let seed: Vec<_> = (0..n).collect();
    for budget in [0, 1, 8_000_000] {
        let mut stats_a = Stats::default();
        let first = refine(n, &pat.col_ptr, &pat.row_idx, &seed, budget, &mut stats_a);
        let mut stats_b = Stats::default();
        let second = refine(n, &pat.col_ptr, &pat.row_idx, &seed, budget, &mut stats_b);
        assert!((0..=budget).contains(&stats_a.spent));
        assert!((0..=budget).contains(&stats_b.spent));
        assert_eq!(stats_a.spent, stats_b.spent);
        assert_eq!(
            first.as_ref().map(|x| (&x.order, x.before, x.after)),
            second.as_ref().map(|x| (&x.order, x.before, x.after))
        );
        if budget <= 1 {
            assert!(first.is_none());
        } else {
            let improvement = first.expect("bounded discovery should defer the universal hub");
            assert!(is_bijection(&improvement.order, n));
            let before = canonical(&pat, &seed);
            let after = canonical(&pat, &improvement.order);
            let graph = graph_from_edges(n, &edges);
            assert_eq!(before, oracle_replay(&graph, &seed).0);
            assert_eq!(after, oracle_replay(&graph, &improvement.order).0);
            assert_eq!(before, 1496);
            assert_eq!(after, 90);
            assert_eq!(improvement.before - improvement.after, before - after);
        }
    }
    for bad in [
        vec![0; n],
        (0..n - 1).collect(),
        (1..=n).collect(),
    ] {
        let mut stats = Stats::default();
        assert!(refine(n, &pat.col_ptr, &pat.row_idx, &bad, 8_000_000, &mut stats).is_none());
        assert!((0..=8_000_000).contains(&stats.spent));
    }
}

#[test]
fn interleave_exact_commit_ticket_and_sqrt_boundaries() {
    for value in [0u64, 1, 2, 3, 4, 15, 16, 17, 1_000_000, u64::MAX] {
        let root = floor_sqrt(value);
        assert!(root == 0 || root <= value / root);
        assert!(root + 1 > value / (root + 1));
    }
    let n = 16;
    // An induced four-cycle makes the graph nonchordal. Deferring the universal
    // hub avoids completing its fifteen neighbors into a clique.
    let mut edges: Vec<_> = (1..n).map(|v| (0, v)).collect();
    edges.extend([(1, 2), (2, 3), (3, 4), (4, 1)]);
    let pat = Pattern::from_edges(n, &edges);
    let seed: Vec<_> = (0..n).collect();
    let mut complete_stats = Stats::default();
    let complete = refine(n, &pat.col_ptr, &pat.row_idx, &seed, 8_000_000, &mut complete_stats).unwrap();
    // The gross ticket includes the deliberately over-reserved final replay.
    let ticket = complete_stats.spent + complete_stats.reserve_unused;
    let mut short_stats = Stats::default();
    assert!(refine(n, &pat.col_ptr, &pat.row_idx, &seed, ticket - 1, &mut short_stats).is_none());
    assert_eq!(short_stats.refusal, Refusal::Kernel);
    assert!(short_stats.spent <= ticket - 1);
    let mut exact_stats = Stats::default();
    let exact = refine(n, &pat.col_ptr, &pat.row_idx, &seed, ticket, &mut exact_stats).unwrap();
    assert_eq!(exact.order, complete.order);
    assert_eq!(exact.after, complete.after);
    assert_eq!(exact_stats.spent + exact_stats.reserve_unused, ticket);
}

#[test]
fn interleave_dp_charge_is_atomic_at_exact_boundary() {
    let pat = Pattern::from_edges(8, &[(0, 1), (1, 2), (2, 3), (3, 0), (0, 7)]);
    let adj = Game::build_adj(8, &pat.col_ptr, &pat.row_idx).unwrap();
    let mut game = Game::new(8, &adj).unwrap();
    game.reset();
    let mut setup = TripleWork { remaining: 100_000_000 };
    let snapshot = Snapshot::new(&game, &[0, 1, 2, 3, 4, 5], &mut setup).unwrap();
    let start = setup.remaining;
    let expected = snapshot.solve(&[0, 2, 4], &mut setup).unwrap();
    let ticket = start - setup.remaining;
    let mut short = TripleWork { remaining: ticket - 1 };
    assert!(snapshot.solve(&[0, 2, 4], &mut short).is_none());
    assert_eq!(short.remaining, ticket - 1);
    let mut exact = TripleWork { remaining: ticket };
    let result = snapshot.solve(&[0, 2, 4], &mut exact).unwrap();
    assert_eq!(exact.remaining, 0);
    assert_eq!(result.order, expected.order);
    assert_eq!(result.after, expected.after);
}

#[test]
fn five_then_interval_wrappers_rescore_each_fresh_input() {
    // This hand seed tests wrapper sequencing and fresh-score ownership; it is
    // not a claim that the full ordering portfolio returns a bad forest order.
    let n = 32;
    let edges: Vec<_> = (1..n).map(|v| (0, v)).collect();
    let pat = Pattern::from_edges(n, &edges);
    let seed: Vec<_> = (0..n).collect();
    let graph = graph_from_edges(n, &edges);
    let mut stats = Stats::default();
    let first = refine_five(n, &pat.col_ptr, &pat.row_idx, &seed, 8_000_000, &mut stats).unwrap();
    assert!(is_bijection(&first.order, n));
    assert_eq!(first.before, canonical(&pat, &seed));
    assert_eq!(first.after, canonical(&pat, &first.order));
    assert_eq!(first.after, oracle_replay(&graph, &first.order).0);
    assert_eq!((first.before, first.after), (11_440, 4_932));
    assert!(first.after < first.before);
    assert!((0..=8_000_000).contains(&stats.spent));

    // Reuse Stats deliberately: each independent call must reset accounting
    // and score its supplied input, not inherit the earlier stage's seed/F0.
    let second = refine(n, &pat.col_ptr, &pat.row_idx, &first.order, 8_000_000, &mut stats).unwrap();
    assert!(is_bijection(&second.order, n));
    assert_eq!(second.before, first.after);
    assert_eq!(second.before, canonical(&pat, &first.order));
    assert_eq!(second.after, canonical(&pat, &second.order));
    assert_eq!(second.after, oracle_replay(&graph, &second.order).0);
    assert_eq!(second.after, 125);
    assert!(second.after < second.before);
    assert!((0..=8_000_000).contains(&stats.spent));
    assert_eq!(seed, (0..n).collect::<Vec<_>>());
}
