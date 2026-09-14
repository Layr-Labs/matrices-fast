//! BRKGA-style memetic random-key search for the elimination order
//! (RAIRO-RO 2023 family, 10.1051/ro/2023081). Population of random-key
//! vectors; the DECODER is a greedy minimum-degree elimination whose
//! tie-break is the key — a lottery distinct from `relabel` (which permutes
//! the vertex numbering the library tie-breaks read; here the elimination
//! itself is key-driven). The decoder returns the order AND its exact
//! `Σ c_v²` (c_v = live degree at elimination), which ranks the population
//! for free. Portfolio winners enter as elite #0 (inverse permutation as
//! keys); every candidate the call site considers is scored exactly and
//! accepted only on a strict improvement, so the device is score-risk-free.
//!
//! Deterministic: LCG stream, per-decode op ledger (fill-pair checks); an
//! exhausted decode finishes with a degree-snapshot tail (still a bijection).

struct Lcg(u64);

impl Lcg {
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn below(&mut self, m: usize) -> usize {
        (self.next_f64() * m as f64) as usize % m.max(1)
    }
}

/// Greedy elimination ordered by `(live degree, key, id)`; returns
/// `(elimination order, Σ c_v²)`. `ops` counts fill-pair checks; once the
/// budget is spent the remaining vertices are appended in
/// `(degree snapshot, key)` order with no further fill accounting.
fn decode(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    keys: &[f64],
    ops: &mut u64,
    budget: u64,
) -> (Vec<usize>, u64) {
    let mut adj: Vec<Vec<u32>> = (0..n)
        .map(|v| {
            let mut list: Vec<u32> = row_idx[col_ptr[v]..col_ptr[v + 1]]
                .iter()
                .map(|&x| x as u32)
                .collect();
            list.sort_unstable();
            list.dedup();
            list
        })
        .collect();
    let mut alive = vec![true; n];
    let mut deg: Vec<usize> = adj.iter().map(|a| a.len()).collect();
    let mut order = Vec::with_capacity(n);
    let mut flops: u64 = 0;

    for _ in 0..n {
        // Pick the live vertex minimizing (degree, key, id).
        let mut best = usize::MAX;
        let mut best_key = (usize::MAX, f64::INFINITY, usize::MAX);
        for v in 0..n {
            if alive[v] && (deg[v], keys[v], v) < best_key {
                best_key = (deg[v], keys[v], v);
                best = v;
            }
        }
        alive[best] = false;
        order.push(best);
        let c = deg[best] as u64 + 1;
        flops = flops.saturating_add(c.saturating_mul(c));
        if *ops > budget {
            continue; // snapshot tail
        }
        let v = best as u32;
        let nbrs: Vec<u32> = adj[best]
            .iter()
            .copied()
            .filter(|&w| alive[w as usize])
            .collect();
        // Close the neighbourhood into a clique (exact elimination step).
        'pairs: for i in 0..nbrs.len() {
            for j in (i + 1)..nbrs.len() {
                *ops += 1;
                if *ops > budget {
                    break 'pairs;
                }
                let (a, b) = (nbrs[i], nbrs[j]);
                let (lo, hi) = if a < b { (a, b) } else { (b, a) };
                if adj[lo as usize].binary_search(&hi).is_err() {
                    // keep both adjacency lists sorted (insertion by splice)
                    let p = adj[lo as usize].partition_point(|&x| x < hi);
                    adj[lo as usize].insert(p, hi);
                    let q = adj[hi as usize].partition_point(|&x| x < lo);
                    adj[hi as usize].insert(q, lo);
                    deg[lo as usize] += 1;
                    deg[hi as usize] += 1;
                }
            }
        }
        for &w in nbrs.iter() {
            deg[w as usize] = deg[w as usize].saturating_sub(1);
        }
    }
    (order, flops)
}

/// One BRKGA run: `pop` key vectors over `gens` generations (elite copies,
/// fresh mutants, parametrized-uniform crossover otherwise), incumbent
/// injected as elite #0. Returns the population's best decode by its own
/// exact elimination objective. The caller scores the result exactly against
/// the incumbent.
pub(crate) fn search(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    incumbent: &[usize],
    pop: usize,
    gens: usize,
    per_decode_ops: u64,
    seed: u64,
) -> Vec<usize> {
    let mut rng = Lcg(seed | 1);
    let mut ledger: u64 = 0;
    let elite_n = (pop / 5).clamp(2, pop);
    let mutant_n = (pop / 10).max(1);

    let incumbent_keys: Vec<f64> = {
        let mut inv = vec![0f64; n];
        for (pos, &v) in incumbent.iter().enumerate() {
            inv[v] = pos as f64;
        }
        inv
    };
    let mut population: Vec<Vec<f64>> = Vec::with_capacity(pop);
    population.push(incumbent_keys);
    for _ in 1..pop {
        population.push((0..n).map(|_| rng.next_f64()).collect());
    }

    let mut best_order: Vec<usize> = incumbent.to_vec();
    let mut best_flops: u64 = u64::MAX;

    for gen in 0..=gens {
        let mut scored: Vec<(u64, usize)> = Vec::with_capacity(population.len());
        for (i, keys) in population.iter().enumerate() {
            let (order, f) = decode(n, col_ptr, row_idx, keys, &mut ledger, per_decode_ops);
            scored.push((f, i));
            if f < best_flops {
                best_flops = f;
                best_order = order;
            }
        }
        if gen == gens {
            break;
        }
        scored.sort_unstable();
        let mut next: Vec<Vec<f64>> = Vec::with_capacity(pop);
        for k in 0..elite_n.min(scored.len()) {
            next.push(population[scored[k].1].clone());
        }
        for _ in 0..mutant_n {
            if next.len() >= pop {
                break;
            }
            next.push((0..n).map(|_| rng.next_f64()).collect());
        }
        while next.len() < pop {
            let e = scored[rng.below(elite_n.min(scored.len()))].1;
            let rest = scored.len().saturating_sub(elite_n).max(1);
            let r = scored[elite_n + rng.below(rest)].1;
            let keys: Vec<f64> = (0..n)
                .map(|g| if rng.next_f64() < 0.7 { population[e][g] } else { population[r][g] })
                .collect();
            next.push(keys);
        }
        population = next;
    }
    best_order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pattern;

    #[test]
    fn search_returns_bijection_and_is_deterministic() {
        let n = 120usize;
        let edges: Vec<_> = (0..n)
            .flat_map(|v| {
                (v + 1..n)
                    .filter(move |&u| (v * 11 + u * 7) % 23 < 3)
                    .map(move |u| (v, u))
            })
            .collect();
        let p = Pattern::from_edges(n, &edges);
        let inc: Vec<usize> = (0..n).collect();
        let a = search(p.n, &p.col_ptr, &p.row_idx, &inc, 12, 3, 2_000_000, 0x1234_5678);
        let b = search(p.n, &p.col_ptr, &p.row_idx, &inc, 12, 3, 2_000_000, 0x1234_5678);
        assert_eq!(a, b);
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..n).collect::<Vec<_>>());
    }

    #[test]
    fn decode_matches_identity_on_empty_graph_and_survives_starvation() {
        let n = 6usize;
        let p = Pattern::from_edges(n, &[]);
        let keys: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let mut ops = 0u64;
        let (order, f) = decode(n, &p.col_ptr, &p.row_idx, &keys, &mut ops, 1_000);
        assert_eq!(order, (0..n).collect::<Vec<_>>());
        assert_eq!(f, n as u64); // all c_v = 1
        let (order2, _) = decode(n, &p.col_ptr, &p.row_idx, &keys, &mut ops, 0);
        let mut s = order2;
        s.sort_unstable();
        assert_eq!(s, (0..n).collect::<Vec<_>>());
    }
}
