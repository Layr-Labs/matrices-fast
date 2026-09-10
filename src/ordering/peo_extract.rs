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
    // Keep only the live entry for each vertex. Removing obsolete entries
    // eagerly preserves the old lazy stack's LIFO order among valid entries,
    // but bounds queue storage by n rather than the completion's edge count.
    let n = adj.len();
    const NONE: usize = usize::MAX;
    let mut head = vec![NONE; n + 1];
    let mut next = vec![NONE; n];
    let mut previous = vec![NONE; n];
    let mut weight = vec![0usize; n];
    for (i, &v) in incumbent.iter().enumerate() {
        if i == 0 { head[0] = v; }
        if i > 0 { previous[v] = incumbent[i - 1]; }
        if i + 1 < n { next[v] = incumbent[i + 1]; }
    }
    let mut maximum = 0;
    let mut visit = Vec::with_capacity(n);
    while visit.len() < n {
        while head[maximum] == NONE {
            if maximum == 0 { visit.reverse(); return visit; }
            maximum -= 1;
        }
        let v = head[maximum];
        head[maximum] = next[v];
        if next[v] != NONE { previous[next[v]] = NONE; }
        weight[v] = NONE;
        visit.push(v);
        for offset in 0..adj[v].len() {
            let index = if reverse_adj { adj[v].len() - 1 - offset } else { offset };
            let u = adj[v][index] as usize;
            let old = weight[u];
            if old == NONE { continue; }
            let before = previous[u];
            let after = next[u];
            if before == NONE { head[old] = after; } else { next[before] = after; }
            if after != NONE { previous[after] = before; }
            let new = old + 1;
            let first = head[new];
            previous[u] = NONE;
            next[u] = first;
            if first != NONE { previous[first] = u; }
            head[new] = u;
            weight[u] = new;
            maximum = maximum.max(new);
        }
    }
    visit.reverse();
    visit
}

#[cfg(test)]
fn mcs_peo_reference(adj: &[Vec<u32>], incumbent: &[usize], reverse_adj: bool) -> Vec<usize> {
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

    #[test]
    fn intrusive_mcs_matches_lazy_stack_with_both_tie_orders() {
        for n in [0, 1, 6, 17, 65, 257] {
            for sample in 0..12 {
                let mut adj = vec![Vec::new(); n];
                for v in 0..n {
                    for u in v + 1..n {
                        if (v * 31 + u * 17 + sample * 13) % 19 <= sample {
                            adj[v].push(u as u32);
                            adj[u].push(v as u32);
                        }
                    }
                }
                for row in &mut adj { if sample % 2 == 0 { row.reverse(); } }
                let mut seed: Vec<_> = (0..n).collect();
                if n > 0 { seed.rotate_left(sample % n); }
                if sample % 3 == 0 { seed.reverse(); }
                for reverse in [false, true] {
                    assert_eq!(mcs_peo(&adj, &seed, reverse), mcs_peo_reference(&adj, &seed, reverse));
                }
            }
        }
    }

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
