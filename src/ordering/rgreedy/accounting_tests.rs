use super::*;

#[test]
fn cleanup_replay_precharges_every_elimination_operation() {
    for n in [65usize, 130, CLIQUE_PRUNING_MIN_N] {
        let w = n.div_ceil(64);
        let mut adj = vec![0u64; n * w];
        for u in 0..n {
            for stride in [1, 7, 63] {
                let v = (u + stride) % n;
                adj[u * w + v / 64] |= 1 << (v % 64);
                adj[v * w + u / 64] |= 1 << (u % 64);
            }
        }
        for pruning in [false, true] {
            let mut game = Game::new(n, &adj).unwrap();
            game.clique_pruning = pruning;
            game.reset();
            for v in 0..n.min(96) {
                let cost = game.elimination_ops(v) as i64;
                let before = game.ops;
                let mut short = TripleWork {
                    remaining: cost - 1,
                };
                assert!(!short.eliminate(&mut game, v));
                assert_eq!(short.remaining, cost - 1);
                assert_eq!(game.ops, before);
                assert_eq!(game.nlive, n - v);

                let mut exact = TripleWork { remaining: cost };
                assert!(exact.eliminate(&mut game, v));
                assert_eq!(exact.remaining, 0);
                assert_eq!(game.ops - before, cost, "n={n} pruning={pruning} v={v}");
            }
        }
    }
}

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
    neighbors.len() as u64 + 1
}

#[test]
fn multiword_edge_floor_matches_independent_boolean_elimination() {
    for n in [65usize, 130] {
        for seed in [0xa3bf_u64, 0x7717] {
            let w = n.div_ceil(64);
            let mut rng = seed;
            let mut oracle = vec![vec![false; n]; n];
            let mut adj = vec![0u64; n * w];
            for u in 0..n {
                for v in u + 1..n {
                    if xs64(&mut rng).is_multiple_of(13) {
                        oracle[u][v] = true;
                        oracle[v][u] = true;
                        adj[u * w + v / 64] |= 1 << (v % 64);
                        adj[v * w + u / 64] |= 1 << (u % 64);
                    }
                }
            }
            let mut order: Vec<_> = (0..n).collect();
            for i in 1..n {
                order.swap(i, below(&mut rng, (i + 1) as u32) as usize);
            }
            let mut reference = oracle.clone();
            let total: u64 = order
                .iter()
                .map(|&v| oracle_eliminate(&mut reference, v).pow(2))
                .sum();
            let mut game = Game::new(n, &adj).unwrap();
            // The ordinary size policy disables this path on small games.
            // Explicitly exercise the combined bound across word boundaries.
            game.clique_pruning = true;
            game.reset();
            let mut prefix = 0;
            for &v in &order {
                let expected = oracle_eliminate(&mut oracle, v);
                assert_eq!(game.eliminate(v), expected);
                prefix += expected * expected;
                let edges: usize = oracle
                    .iter()
                    .enumerate()
                    .map(|(u, row)| row[u + 1..].iter().filter(|&&edge| edge).count())
                    .sum();
                assert_eq!(game.edges, edges, "n={n} seed={seed} pivot={v}");
                for (u, row) in oracle.iter().enumerate() {
                    for (v, &edge) in row.iter().enumerate() {
                        assert_eq!(game.adj[u * w + v / 64] & (1 << (v % 64)) != 0, edge);
                    }
                }
                assert!(!game.prune_clique_floor(prefix, total + 1, i64::MAX));
            }
            assert_eq!(prefix, total);
        }
    }
}
