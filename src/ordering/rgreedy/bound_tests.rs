use super::*;

fn next_permutation(p: &mut [usize]) -> bool {
    let Some(i) = (0..p.len().saturating_sub(1))
        .rev()
        .find(|&i| p[i] < p[i + 1])
    else {
        return false;
    };
    let j = (i + 1..p.len()).rev().find(|&j| p[j] > p[i]).unwrap();
    p.swap(i, j);
    p[i + 1..].reverse();
    true
}

#[test]
fn clique_floor_never_prunes_a_feasible_completion_exhaustively() {
    // Every simple labelled graph through five vertices, every choice of
    // boundary size, every order of the eliminable vertices, every prefix.
    for n in 1usize..=5 {
        let pairs: Vec<_> = (0..n)
            .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
            .collect();
        for mask in 0..1usize << pairs.len() {
            let mut adj = vec![0u64; n];
            for (bit, &(u, v)) in pairs.iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    adj[u] |= 1 << v;
                    adj[v] |= 1 << u;
                }
            }
            for nelim in 1..=n {
                let mut order: Vec<_> = (0..nelim).collect();
                loop {
                    let mut game = Game::new_partial(n, &adj, nelim).unwrap();
                    let total = game.replay_flops(&order);
                    game.reset();
                    let mut prefix = 0;
                    for &v in &order {
                        let c = game.eliminate(v);
                        prefix += c * c;
                        assert!(
                            !game.prune_clique_floor(prefix, total + 1, i64::MAX),
                            "n={n} mask={mask} nelim={nelim} order={order:?} pivot={v}"
                        );
                    }
                    if !next_permutation(&mut order) {
                        break;
                    }
                }
            }
        }
    }
}

#[test]
fn clique_floor_accounts_for_permanently_live_boundary() {
    let adj = vec![0b11110, 0b11101, 0b11011, 0b10111, 0b01111];
    let mut game = Game::new_partial(5, &adj, 2).unwrap();
    game.reset();
    let c = game.eliminate(0);
    assert_eq!(c, 5);
    // One remaining pivot stays adjacent to all three boundary vertices.
    assert!(game.prune_clique_floor(25, 41, i64::MAX));
    assert!(!game.prune_clique_floor(25, 42, i64::MAX));
}

#[test]
fn clique_floor_refuses_unaffordable_boundary_scan() {
    let adj = vec![0b11110, 0b11101, 0b11011, 0b10111, 0b01111];
    let mut game = Game::new_partial(5, &adj, 2).unwrap();
    game.reset();
    game.eliminate(0);
    let before = game.ops;
    assert!(game.prune_clique_floor(25, 100, before));
    assert_eq!(game.ops, before);
}

#[test]
fn root_policy_preserves_small_games_and_enables_large_games() {
    for n in [CLIQUE_PRUNING_MIN_N - 1, CLIQUE_PRUNING_MIN_N] {
        let adj = vec![0; n * n.div_ceil(64)];
        let mut game = Game::new(n, &adj).unwrap();
        game.reset();
        game.eliminate(0);
        assert_eq!(
            game.prune_after_elimination(1, n as u64, i64::MAX),
            n >= CLIQUE_PRUNING_MIN_N
        );
    }
}

#[test]
fn clique_floor_handles_multiword_partial_games() {
    let mut rng = 0x31d2_799a_u64;
    for n in [65usize, 130] {
        let w = n.div_ceil(64);
        for nelim in [n / 3, n / 2, n] {
            for _ in 0..12 {
                let mut adj = vec![0u64; n * w];
                for u in 0..n {
                    for v in u + 1..n {
                        if xs64(&mut rng).is_multiple_of(13) {
                            adj[u * w + v / 64] |= 1 << (v % 64);
                            adj[v * w + u / 64] |= 1 << (u % 64);
                        }
                    }
                }
                let mut order: Vec<_> = (0..nelim).collect();
                for i in 1..nelim {
                    order.swap(i, below(&mut rng, (i + 1) as u32) as usize);
                }
                let mut game = Game::new_partial(n, &adj, nelim).unwrap();
                let total = game.replay_flops(&order);
                game.reset();
                let mut prefix = 0;
                for &v in &order {
                    let c = game.eliminate(v);
                    prefix += c * c;
                    assert!(!game.prune_clique_floor(prefix, total + 1, i64::MAX));
                }
            }
        }
    }
}
