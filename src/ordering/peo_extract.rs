//! Two linear-work PEO tie variations of one incumbent chordal completion.
//!
//! This is a sibling of the terminal fill-edge watcher: it never erases edges
//! or changes the watcher's candidate, tie order, or work allowance.

pub(super) const MAX_N: usize = 30_000;
pub(super) const MAX_INPUT_NNZ: usize = 180_000;
pub(super) const MAX_LNNZ: usize = 300_000;

pub(super) fn candidates(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    parent: &[Option<usize>],
    counts: &[usize],
    incumbent: &[usize],
) -> Option<[Vec<usize>; 2]> {
    candidates_bounded(n, cp, ri, parent, counts, incumbent, MAX_N, MAX_INPUT_NNZ, MAX_LNNZ)
}

/// The same two MCS extractions under caller-supplied structural limits (dimension,
/// input nonzeros, factor nonzeros). Limits are structure, never identity.
pub(super) fn candidates_bounded(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    parent: &[Option<usize>],
    counts: &[usize],
    incumbent: &[usize],
    max_n: usize,
    max_nnz: usize,
    max_lnnz: usize,
) -> Option<[Vec<usize>; 2]> {
    let adj = reconstruct(n, cp, ri, parent, counts, incumbent, max_n, max_nnz, max_lnnz)?;
    let forward = mcs_peo(&adj, incumbent, false);
    let reverse = mcs_peo(&adj, incumbent, true);
    if !super::is_bijection(&forward, n) || !super::is_bijection(&reverse, n) {
        return None;
    }
    Some([forward, reverse])
}

/// MCS extractions using several structural quantities as static tie ranks.
/// This is kept separate from [`candidates_bounded`] so callers can spend it
/// at a strict terminal best-of boundary. The first four candidates retain
/// the historical id/completed-degree order byte-for-byte; original degree,
/// completion fill surplus, and incumbent column count add genuinely
/// different deterministic views of the same completion.
pub(super) fn ranked_candidates_bounded(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    parent: &[Option<usize>],
    counts: &[usize],
    incumbent: &[usize],
    max_n: usize,
    max_nnz: usize,
    max_lnnz: usize,
) -> Option<Vec<Vec<usize>>> {
    let adj = reconstruct(n, cp, ri, parent, counts, incumbent, max_n, max_nnz, max_lnnz)?;
    // Reuse one adjacency allocation for every static rank. Symmetry lets
    // a global rank-ordered traversal emit each row in exactly sorted order
    // in O(n + edges), instead of sorting every neighbor list separately.
    let mut ranked_adj: Vec<Vec<u32>> = adj.iter().map(|row| Vec::with_capacity(row.len())).collect();
    let mut out = Vec::with_capacity(10);
    let id: Vec<usize> = (0..n).collect();
    order_neighbors_by_rank(&adj, &id, &mut ranked_adj);
    out.push(mcs_peo(&ranked_adj, &id, false));
    out.push(mcs_peo(&ranked_adj, &id, true));

    let completed_degree: Vec<usize> = adj.iter().map(Vec::len).collect();
    let mut degree_order: Vec<usize> = (0..n).collect();
    degree_order.sort_unstable_by_key(|&v| (completed_degree[v], v));
    order_neighbors_by_rank(&adj, &degree_order, &mut ranked_adj);
    out.push(mcs_peo(&ranked_adj, &degree_order, false));
    out.push(mcs_peo(&ranked_adj, &degree_order, true));
    let mut seen_orders = vec![id, degree_order];

    let mut original_degree = vec![0usize; n];
    let mut column_count = vec![0usize; n];
    for (position, &vertex) in incumbent.iter().enumerate() {
        original_degree[vertex] = cp[position + 1] - cp[position];
        column_count[vertex] = counts[position];
    }
    let fill_surplus: Vec<usize> = completed_degree
        .iter()
        .zip(&original_degree)
        .map(|(&filled, &original)| filled.saturating_sub(original))
        .collect();

    for rank in [&original_degree, &fill_surplus, &column_count] {
        let mut initial: Vec<usize> = (0..n).collect();
        initial.sort_unstable_by_key(|&v| (rank[v], v));
        // Equal total rank orders imply identical row orders and MCS output.
        // Compare the ranks themselves; no probabilistic content hash.
        if seen_orders.iter().any(|previous| *previous == initial) {
            continue;
        }
        order_neighbors_by_rank(&adj, &initial, &mut ranked_adj);
        out.push(mcs_peo(&ranked_adj, &initial, false));
        out.push(mcs_peo(&ranked_adj, &initial, true));
        seen_orders.push(initial);
    }

    if out.iter().any(|candidate| !super::is_bijection(candidate, n)) {
        return None;
    }
    Some(out)
}

fn order_neighbors_by_rank(adj: &[Vec<u32>], rank_order: &[usize], out: &mut [Vec<u32>]) {
    for row in out.iter_mut() {
        row.clear();
    }
    // The graph is undirected. Visiting u in ascending total rank and
    // emitting u into every neighbor's row is therefore a stable transpose
    // whose rows equal comparison-sorting adj[v] by that same total rank.
    for &u in rank_order {
        for &v in &adj[u] {
            out[v as usize].push(u as u32);
        }
    }
}

/// Per-completion MCS scratch: weights, marks and bucket capacities survive
/// the rank/direction trials. Every logical entry is reset between trials;
/// only allocation capacity is reused, and no state survives an order() call.
#[cfg(test)]
struct McsWorkspace {
    weight: Vec<usize>,
    visited: Vec<bool>,
    buckets: Vec<Vec<u32>>,
}

#[cfg(test)]
impl McsWorkspace {
    fn new(n: usize) -> Self {
        Self {
            weight: vec![0; n],
            visited: vec![false; n],
            buckets: vec![Vec::with_capacity(n)],
        }
    }

    fn extract(&mut self, adj: &[Vec<u32>], initial: &[usize], reverse: bool) -> Vec<usize> {
        let n = adj.len();
        debug_assert_eq!(self.weight.len(), n);
        self.weight.fill(0);
        self.visited.fill(false);
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        self.buckets[0].extend(initial.iter().rev().map(|&v| v as u32));
        let mut max_weight = 0;
        let mut visit = Vec::with_capacity(n);
        while visit.len() < n {
            let Some(v) = self.buckets[max_weight].pop() else {
                if max_weight == 0 { break; }
                max_weight -= 1;
                continue;
            };
            let v = v as usize;
            if self.visited[v] || self.weight[v] != max_weight { continue; }
            self.visited[v] = true;
            visit.push(v);
            for offset in 0..adj[v].len() {
                let index = if reverse { adj[v].len() - 1 - offset } else { offset };
                let u = adj[v][index] as usize;
                if !self.visited[u] {
                    self.weight[u] += 1;
                    let new_weight = self.weight[u];
                    if new_weight >= self.buckets.len() {
                        self.buckets.resize_with(new_weight + 1, Vec::new);
                    }
                    self.buckets[new_weight].push(u as u32);
                    max_weight = max_weight.max(new_weight);
                }
            }
        }
        visit.reverse();
        visit
    }
}

fn reconstruct(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    parent: &[Option<usize>],
    counts: &[usize],
    incumbent: &[usize],
    max_n: usize,
    max_nnz: usize,
    max_lnnz: usize,
) -> Option<Vec<Vec<u32>>> {
    if n == 0 || n > max_n || ri.len() > max_nnz
        || cp.len() != n + 1 || parent.len() != n || counts.len() != n
        || cp.first().copied() != Some(0) || cp.last().copied() != Some(ri.len())
        || !super::is_bijection(incumbent, n)
    {
        return None;
    }
    if cp.windows(2).any(|w| w[0] > w[1]) || ri.iter().any(|&v| v >= n) {
        return None;
    }
    let lnnz = counts.iter().enumerate().try_fold(0usize, |sum, (j, &c)| {
        if c == 0 || c > n - j { None } else { sum.checked_add(c) }
    })?;
    if lnnz > max_lnnz { return None; }

    let mut children = vec![Vec::<usize>::new(); n];
    for (j, &p) in parent.iter().enumerate() {
        if let Some(p) = p {
            if p <= j || p >= n { return None; }
            children[p].push(j);
        }
    }
    let mut columns = vec![Vec::<u32>::new(); n];
    let mut adj = vec![Vec::<u32>::new(); n];
    let mut mark = vec![usize::MAX; n];
    let mut total = 0usize;
    for j in 0..n {
        let mut reach = Vec::<u32>::with_capacity(counts[j] - 1);
        for &i in &ri[cp[j]..cp[j + 1]] {
            if i > j && mark[i] != j {
                mark[i] = j;
                reach.push(i as u32);
            }
        }
        for &child in &children[j] {
            for &i in &columns[child] {
                let i = i as usize;
                if i > j && mark[i] != j {
                    mark[i] = j;
                    reach.push(i as u32);
                }
            }
            columns[child] = Vec::new();
        }
        // Never extract from an unchecked reconstruction. Each column is
        // scanned at most once again, at its unique elimination-tree parent.
        if reach.len().checked_add(1)? != counts[j] { return None; }
        total = total.checked_add(reach.len())?;
        if total > max_lnnz.saturating_sub(n) { return None; }
        let v = incumbent[j];
        for &i in &reach {
            let w = incumbent[i as usize];
            adj[v].push(w as u32);
            adj[w].push(v as u32);
        }
        columns[j] = reach;
    }
    if total.checked_add(n)? != lnnz { return None; }
    Some(adj)
}

fn mcs_peo(adj: &[Vec<u32>], incumbent: &[usize], reverse_adj: bool) -> Vec<usize> {
    let n = adj.len();
    let mut weight = vec![0usize; n];
    let mut visited = vec![false; n];
    // The first visited zero-weight vertex follows the incumbent, rather than
    // a label ordering. Updated buckets are deterministic LIFO stacks.
    let mut buckets: Vec<Vec<u32>> = vec![incumbent.iter().rev().map(|&v| v as u32).collect()];
    let mut max_weight = 0usize;
    let mut visit = Vec::with_capacity(n);
    while visit.len() < n {
        let Some(v) = buckets[max_weight].pop() else {
            if max_weight == 0 { break; }
            max_weight -= 1;
            continue;
        };
        let v = v as usize;
        if visited[v] || weight[v] != max_weight { continue; }
        visited[v] = true;
        visit.push(v);
        for offset in 0..adj[v].len() {
            let index = if reverse_adj { adj[v].len() - 1 - offset } else { offset };
            let u = adj[v][index] as usize;
            if !visited[u] {
                weight[u] += 1;
                let new_weight = weight[u];
                if new_weight >= buckets.len() {
                    buckets.resize_with(new_weight + 1, Vec::new);
                }
                buckets[new_weight].push(u as u32);
                max_weight = max_weight.max(new_weight);
            }
        }
    }
    // MCS visits a reverse PEO. Every edge causes exactly one bucket insertion;
    // stale entries do not exceed that count, so work and storage are O(n+|E|).
    visit.reverse();
    visit
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{column_counts_gnp, flops_of, is_bijection,
        permute_pattern, EliminationTree, ScoringPattern};

    fn pattern(adj: &[Vec<bool>]) -> ScoringPattern {
        let mut cp = vec![0];
        let mut ri = Vec::new();
        for row in adj {
            ri.extend(row.iter().enumerate().filter_map(|(v, &edge)| edge.then_some(v)));
            cp.push(ri.len());
        }
        ScoringPattern { n: adj.len(), col_ptr: cp, row_idx: ri }
    }

    fn complete(adj: &[Vec<bool>], order: &[usize]) -> Vec<Vec<bool>> {
        let mut filled = adj.to_vec();
        let mut live = vec![true; adj.len()];
        for &v in order {
            let neighbors: Vec<_> = (0..adj.len()).filter(|&u| live[u] && filled[v][u]).collect();
            for &u in &neighbors {
                for &w in &neighbors {
                    if u != w { filled[u][w] = true; }
                }
            }
            live[v] = false;
        }
        filled
    }

    fn is_peo(adj: &[Vec<bool>], order: &[usize]) -> bool {
        if !is_bijection(order, adj.len()) { return false; }
        for (k, &v) in order.iter().enumerate() {
            let neighbors: Vec<_> = order[k + 1..].iter().copied().filter(|&u| adj[v][u]).collect();
            for &u in &neighbors {
                for &w in &neighbors {
                    if u != w && !adj[u][w] { return false; }
                }
            }
        }
        true
    }

    fn next_permutation(p: &mut [usize]) -> bool {
        let Some(i) = (0..p.len().saturating_sub(1)).rev().find(|&i| p[i] < p[i + 1]) else { return false; };
        let j = (i + 1..p.len()).rev().find(|&j| p[j] > p[i]).unwrap();
        p.swap(i, j);
        p[i + 1..].reverse();
        true
    }

    #[test]
    fn rank_transpose_matches_comparison_sorted_neighbors() {
        for n in 1..=5 {
            let edges: Vec<_> = (0..n).flat_map(|u| (u + 1..n).map(move |v| (u, v))).collect();
            for mask in 0..(1usize << edges.len()) {
                let mut adj = vec![Vec::new(); n];
                for (bit, &(u, v)) in edges.iter().enumerate() {
                    if mask & (1 << bit) != 0 {
                        adj[u].push(v as u32);
                        adj[v].push(u as u32);
                    }
                }
                let mut out: Vec<Vec<u32>> = adj.iter().map(|row| Vec::with_capacity(row.len())).collect();
                let mut mcs = McsWorkspace::new(n);
                for mode in 0..3 {
                    let mut rank: Vec<_> = (0..n).collect();
                    rank.sort_unstable_by_key(|&v| match mode {
                        0 => (0, v),
                        1 => (adj[v].len(), v),
                        _ => (v % 2, n - v),
                    });
                    let mut inverse = vec![0; n];
                    for (i, &v) in rank.iter().enumerate() { inverse[v] = i; }
                    let mut expected = adj.clone();
                    for row in &mut expected { row.sort_unstable_by_key(|&v| inverse[v as usize]); }
                    order_neighbors_by_rank(&adj, &rank, &mut out);
                    assert_eq!(out, expected, "n={n} mask={mask} mode={mode}");
                    for reverse in [false, true] {
                        assert_eq!(mcs_peo(&out, &rank, reverse), mcs_peo(&expected, &rank, reverse));
                        assert_eq!(mcs.extract(&out, &rank, reverse), mcs_peo(&expected, &rank, reverse));
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "matched real-completion neighbor preparation benchmark"]
    fn probe_rank_sort_vs_transpose() {
        use std::hint::black_box;
        use std::time::Instant;
        for (name, p) in crate::corpus::corpus() {
            if !["mpbp_48", "crudeoil_lee4_10", "nuclear10a"].contains(&name.as_str()) { continue; }
            let sp = super::super::ScoringPattern { n: p.n, col_ptr: p.col_ptr.clone(), row_idx: p.row_idx.clone() };
            let perm = super::super::order(&p);
            let pp = permute_pattern(&sp, &perm);
            let et = EliminationTree::from_pattern(&pp);
            let counts = column_counts_gnp(&pp, &et);
            let adj = reconstruct(p.n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &perm, MAX_N, MAX_INPUT_NNZ, 1_000_000).unwrap();
            let degree: Vec<_> = adj.iter().map(Vec::len).collect();
            let original: Vec<_> = (0..p.n).map(|v| p.col(v).len()).collect();
            let surplus: Vec<_> = degree.iter().zip(&original).map(|(&a, &b)| a.saturating_sub(b)).collect();
            let mut count_by_vertex = vec![0; p.n];
            for (i, &v) in perm.iter().enumerate() { count_by_vertex[v] = counts[i]; }
            let ranks = vec![(0..p.n).collect::<Vec<_>>(), degree, original, surplus, count_by_vertex];
            let orders: Vec<Vec<usize>> = ranks.iter().map(|r| {
                let mut order: Vec<_> = (0..p.n).collect();
                order.sort_unstable_by_key(|&v| (r[v], v));
                order
            }).collect();
            let mut sorted = adj.clone();
            let mut transposed: Vec<Vec<u32>> = adj.iter().map(|row| Vec::with_capacity(row.len())).collect();
            let mut old_samples = Vec::new();
            let mut new_samples = Vec::new();
            for trial in 0..6 {
                for arm in 0..2 {
                    let old = (trial + arm) % 2 == 0;
                    let start = Instant::now();
                    for _ in 0..4 {
                        for (rank, order) in ranks.iter().zip(&orders) {
                            if old {
                                for row in &mut sorted { row.sort_unstable_by_key(|&u| (rank[u as usize], u)); }
                                black_box(&sorted);
                            } else {
                                order_neighbors_by_rank(&adj, order, &mut transposed);
                                black_box(&transposed);
                            }
                        }
                    }
                    let secs = start.elapsed().as_secs_f64();
                    if old { old_samples.push(secs); } else { new_samples.push(secs); }
                }
                assert_eq!(sorted, transposed);
            }
            old_samples.sort_by(f64::total_cmp);
            new_samples.sort_by(f64::total_cmp);
            let old = (old_samples[2] + old_samples[3]) / 2.0;
            let new = (new_samples[2] + new_samples[3]) / 2.0;
            println!("RANK_PREP\t{name}\told_ms={:.3}\ttranspose_ms={:.3}\tspeedup={:.3}", old * 1000.0, new * 1000.0, old / new);
        }
    }

    #[test]
    #[ignore = "matched real-completion MCS allocation benchmark"]
    fn probe_mcs_workspace() {
        use std::hint::black_box;
        use std::time::Instant;
        for (name, p) in crate::corpus::corpus() {
            if !["mpbp_48", "crudeoil_lee4_10", "nuclear10a", "sporttournament48"]
                .contains(&name.as_str()) { continue; }
            let sp = ScoringPattern { n: p.n, col_ptr: p.col_ptr.clone(), row_idx: p.row_idx.clone() };
            let perm = super::super::order(&p);
            let pp = permute_pattern(&sp, &perm);
            let et = EliminationTree::from_pattern(&pp);
            let counts = column_counts_gnp(&pp, &et);
            let adj = reconstruct(p.n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts,
                &perm, MAX_N, MAX_INPUT_NNZ, 1_000_000).unwrap();
            let degree: Vec<_> = adj.iter().map(Vec::len).collect();
            let original: Vec<_> = (0..p.n).map(|v| p.col(v).len()).collect();
            let surplus: Vec<_> = degree.iter().zip(&original)
                .map(|(&a, &b)| a.saturating_sub(b)).collect();
            let mut count_by_vertex = vec![0; p.n];
            for (i, &v) in perm.iter().enumerate() { count_by_vertex[v] = counts[i]; }
            let ranks = vec![(0..p.n).collect::<Vec<_>>(), degree, original, surplus, count_by_vertex];
            let inputs: Vec<_> = ranks.iter().map(|rank| {
                let mut initial: Vec<_> = (0..p.n).collect();
                initial.sort_unstable_by_key(|&v| (rank[v], v));
                let mut rows: Vec<Vec<u32>> = adj.iter().map(|r| Vec::with_capacity(r.len())).collect();
                order_neighbors_by_rank(&adj, &initial, &mut rows);
                (initial, rows)
            }).collect();
            let mut check = McsWorkspace::new(p.n);
            for (initial, rows) in &inputs {
                for reverse in [false, true] {
                    assert_eq!(mcs_peo(rows, initial, reverse), check.extract(rows, initial, reverse));
                }
            }
            let mut samples = [Vec::new(), Vec::new()];
            for trial in 0..8 {
                for position in 0..2 {
                    let arm = (trial + position) % 2;
                    let start = Instant::now();
                    for _ in 0..4 {
                        // Construct fresh per-completion scratch inside the
                        // measured new arm, then reuse it only for ten trials.
                        let mut workspace = (arm == 1).then(|| McsWorkspace::new(p.n));
                        let mut output = Vec::with_capacity(10);
                        for (initial, rows) in &inputs {
                            for reverse in [false, true] {
                                output.push(match &mut workspace {
                                    None => mcs_peo(rows, initial, reverse),
                                    Some(w) => w.extract(rows, initial, reverse),
                                });
                            }
                        }
                        black_box(output);
                    }
                    samples[arm].push(start.elapsed().as_secs_f64());
                }
            }
            for sample in &mut samples { sample.sort_by(f64::total_cmp); }
            let old = (samples[0][3] + samples[0][4]) / 2.0;
            let new = (samples[1][3] + samples[1][4]) / 2.0;
            println!("MCS_SCRATCH\t{name}\tfresh_ms={:.3}\treused_ms={:.3}\tspeedup={:.3}",
                old * 1000.0, new * 1000.0, old / new);
        }
    }

    #[test]
    fn peo_extraction_exhaustive_graphs_and_orders() {
        for n in 1..=5 {
            let edges: Vec<_> = (0..n).flat_map(|u| (u + 1..n).map(move |v| (u, v))).collect();
            for mask in 0..(1usize << edges.len()) {
                let mut graph = vec![vec![false; n]; n];
                for (bit, &(u, v)) in edges.iter().enumerate() {
                    if mask & (1 << bit) != 0 { graph[u][v] = true; graph[v][u] = true; }
                }
                let pat = pattern(&graph);
                let mut incumbent: Vec<_> = (0..n).collect();
                loop {
                    let pp = permute_pattern(&pat, &incumbent);
                    let et = EliminationTree::from_pattern(&pp);
                    let counts = column_counts_gnp(&pp, &et);
                    let filled = complete(&graph, &incumbent);
                    let reconstructed = reconstruct(n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &incumbent, MAX_N, MAX_INPUT_NNZ, MAX_LNNZ).unwrap();
                    for v in 0..n {
                        let mut row = vec![false; n];
                        for &u in &reconstructed[v] { assert!(!row[u as usize]); row[u as usize] = true; }
                        assert_eq!(row, filled[v]);
                    }
                    let baseline = flops_of(&pat, &incumbent);
                    let mut best = baseline;
                    for reverse in [false, true] {
                        let candidate = mcs_peo(&reconstructed, &incumbent, reverse);
                        assert!(is_peo(&filled, &candidate));
                        // Independent symbolic scorer, not MCS weights/counts.
                        let f = flops_of(&pat, &candidate);
                        best = best.min(f);
                        assert!(f <= baseline, "n={n}, mask={mask}, order={incumbent:?}");
                    }
                    for candidate in ranked_candidates_bounded(
                        n,
                        &pp.col_ptr,
                        &pp.row_idx,
                        &et.parent,
                        &counts,
                        &incumbent,
                        MAX_N,
                        MAX_INPUT_NNZ,
                        MAX_LNNZ,
                    )
                    .unwrap()
                    {
                        assert!(is_peo(&filled, &candidate));
                        assert!(
                            flops_of(&pat, &candidate) <= baseline,
                            "ranked n={n}, mask={mask}, order={incumbent:?}"
                        );
                    }
                    assert!(best <= baseline);
                    if !next_permutation(&mut incumbent) { break; }
                }
            }
        }
    }

    #[test]
    fn peo_extraction_strict_path_witness_and_bad_inputs() {
        let graph = vec![vec![false, true, false], vec![true, false, true], vec![false, true, false]];
        let pat = pattern(&graph);
        let incumbent = vec![1, 0, 2];
        let pp = permute_pattern(&pat, &incumbent);
        let et = EliminationTree::from_pattern(&pp);
        let counts = column_counts_gnp(&pp, &et);
        let orders = candidates(3, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &incumbent).unwrap();
        assert_eq!(flops_of(&pat, &incumbent), 14);
        assert_eq!(orders.iter().map(|p| flops_of(&pat, p)).min(), Some(9));
        assert!(candidates(3, &pp.col_ptr, &pp.row_idx, &et.parent, &[1, 1, 1], &incumbent).is_none());
        assert!(candidates(3, &[0, 4, 2, 4], &pp.row_idx, &et.parent, &counts, &incumbent).is_none());
        assert!(candidates(3, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &[1, 1, 2]).is_none());
        assert!(candidates(3, &pp.col_ptr, &pp.row_idx, &[], &counts, &incumbent).is_none());
        assert!(candidates(MAX_N + 1, &[], &[], &[], &[], &[]).is_none());
    }
}
