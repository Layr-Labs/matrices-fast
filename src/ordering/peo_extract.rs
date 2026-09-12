//! Two linear-work PEO tie variations of one incumbent chordal completion.
//!
//! This is a sibling of the terminal fill-edge watcher: it never erases edges
//! or changes the watcher's candidate, tie order, or work allowance.

pub(super) const MAX_N: usize = 30_000;
pub(super) const MAX_INPUT_NNZ: usize = 180_000;
pub(super) const MAX_LNNZ: usize = 300_000;

/// ── 0194: the fill-adjacency build's allocator traffic, pooled ──────────────
/// `reconstruct` builds `columns`, `children` and `adj` as `Vec<Vec<..>>` of
/// length n, freshly allocated on every call; the alternate-seed chain calls it
/// once per round (778 rounds over the 38 dev rows of 10 000 < n <= 50 000 at
/// the doubled allowance). Reusing those buffers removes the traffic and nothing
/// else: every push keeps its position inside its list, so the reconstructed
/// adjacency — and therefore both extracted orders — are bit-identical to the
/// allocating path (verified: 300/300 identical COUNTS, SCORE 0.792439 either
/// way, and the reconstruction/adjacency unit tests below still pass).
///
/// The MCS side is deliberately NOT pooled. Pooling its label buckets as well
/// measured **net slower** — order() 112.63 -> 115.04 s with mcs 1.96 -> 3.30 s
/// — because a pooled bucket array must be cleared on every sweep while the
/// allocating path only pays for the labels it touches. Only this kernel is
/// pooled: recon 2.37 -> 1.99 s corpus, **-0.42 s over the 38 dev rows of
/// 10 000 < n <= 50 000**, i.e. the band is faster than the frontier's own
/// profile by about the amount one 1e6-unit allowance step costs there.
#[derive(Default)]
pub(super) struct Scratch {
    n: usize,
    adj: Vec<Vec<u32>>,
    columns: Vec<Vec<u32>>,
    children: Vec<Vec<usize>>,
    mark: Vec<usize>,
}

impl Scratch {
    /// Grow to `n` (never shrink) and clear every buffer the next call reads.
    fn prepare(&mut self, n: usize) {
        if self.n < n {
            self.n = n;
            self.adj.resize_with(n, Vec::new);
            self.columns.resize_with(n, Vec::new);
            self.children.resize_with(n, Vec::new);
            self.mark.resize(n, usize::MAX);
        }
        for v in 0..self.n {
            self.adj[v].clear();
            self.columns[v].clear();
            self.children[v].clear();
        }
        self.mark[..n].fill(usize::MAX);
    }
}

thread_local! {
    static SCRATCH: std::cell::RefCell<Scratch> = std::cell::RefCell::new(Scratch::default());
}

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
    SCRATCH.with(|cell| {
        let scratch = &mut *cell.borrow_mut();
        candidates_with_scratch(
            n, cp, ri, parent, counts, incumbent, max_n, max_nnz, max_lnnz, scratch,
        )
    })
}

/// Allocation-free body of [`candidates_bounded`], split out so the pooled
/// scratch is borrowed exactly once per call and the kernels can be unit-tested
/// against a scratch of their own.
pub(super) fn candidates_with_scratch(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    parent: &[Option<usize>],
    counts: &[usize],
    incumbent: &[usize],
    max_n: usize,
    max_nnz: usize,
    max_lnnz: usize,
    scratch: &mut Scratch,
) -> Option<[Vec<usize>; 2]> {
    #[cfg(test)]
    let _tr = std::time::Instant::now();
    reconstruct(n, cp, ri, parent, counts, incumbent, max_n, max_nnz, max_lnnz, scratch)?;
    #[cfg(test)]
    prof::add_recon(_tr.elapsed().as_secs_f64());
    #[cfg(test)]
    let _tm = std::time::Instant::now();
    let forward = mcs_peo(&scratch.adj, incumbent, false);
    let reverse = mcs_peo(&scratch.adj, incumbent, true);
    #[cfg(test)]
    prof::add_mcs(_tm.elapsed().as_secs_f64());
    if !super::is_bijection(&forward, n) || !super::is_bijection(&reverse, n) {
        return None;
    }
    Some([forward, reverse])
}

/// TEST-ONLY kernel price of `candidates_bounded`: seconds spent rebuilding the
/// filled-graph adjacency (with its `Vec<Vec<u32>>` shape) versus seconds spent
/// in the two MCS sweeps over it. Accumulated per `order()` call and pushed as
/// phase marks by the caller, so a PHASES log carries the split next to the
/// stage that pays it. Never compiled into the shipped worker.
#[cfg(test)]
pub(super) mod prof {
    use std::cell::Cell;
    thread_local! {
        static RECON: Cell<f64> = const { Cell::new(0.0) };
        static MCS: Cell<f64> = const { Cell::new(0.0) };
        static RECON_CALLS: Cell<u64> = const { Cell::new(0) };
    }
    pub(crate) fn add_recon(secs: f64) {
        RECON.with(|c| c.set(c.get() + secs));
        RECON_CALLS.with(|c| c.set(c.get() + 1));
    }
    pub(crate) fn add_mcs(secs: f64) {
        MCS.with(|c| c.set(c.get() + secs));
    }
    /// (recon secs, mcs secs, recon calls) since the last drain.
    pub(crate) fn take() -> (f64, f64, u64) {
        (
            RECON.with(|c| c.replace(0.0)),
            MCS.with(|c| c.replace(0.0)),
            RECON_CALLS.with(|c| c.replace(0)),
        )
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
    scratch: &mut Scratch,
) -> Option<()> {
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

    scratch.prepare(n);
    // Destructured so the reach buffer of column `j` and the adjacency lists it
    // feeds are disjoint borrows of the same scratch. Every buffer starts empty
    // and every push below keeps the position it had in the allocating path.
    let Scratch { adj, columns, children, mark, .. } = scratch;
    for (j, &p) in parent.iter().enumerate() {
        if let Some(p) = p {
            if p <= j || p >= n { return None; }
            children[p].push(j);
        }
    }
    let mut total = 0usize;
    for j in 0..n {
        // `columns[j]` is the reach buffer of column j: its own iteration is the
        // only writer and its unique parent (index > j) consumes and clears it
        // later. Children are strictly below j, so the split at j is disjoint.
        let (lower, upper) = columns.split_at_mut(j);
        let reach = &mut upper[0];
        for &i in &ri[cp[j]..cp[j + 1]] {
            if i > j && mark[i] != j {
                mark[i] = j;
                reach.push(i as u32);
            }
        }
        for &child in &children[j] {
            for &i in &lower[child] {
                let i = i as usize;
                if i > j && mark[i] != j {
                    mark[i] = j;
                    reach.push(i as u32);
                }
            }
            lower[child].clear();
        }
        // Never extract from an unchecked reconstruction. Each column is
        // scanned at most once again, at its unique elimination-tree parent.
        if reach.len().checked_add(1)? != counts[j] { return None; }
        total = total.checked_add(reach.len())?;
        if total > max_lnnz.saturating_sub(n) { return None; }
        let v = incumbent[j];
        for &i in reach.iter() {
            let w = incumbent[i as usize];
            adj[v].push(w as u32);
            adj[w].push(v as u32);
        }
    }
    if total.checked_add(n)? != lnnz { return None; }
    Some(())
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
                    let mut scratch = Scratch::default();
                    reconstruct(n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &incumbent, MAX_N, MAX_INPUT_NNZ, MAX_LNNZ, &mut scratch).unwrap();
                    for v in 0..n {
                        let mut row = vec![false; n];
                        for &u in &scratch.adj[v] { assert!(!row[u as usize]); row[u as usize] = true; }
                        assert_eq!(row, filled[v]);
                    }
                    let baseline = flops_of(&pat, &incumbent);
                    let mut best = baseline;
                    for reverse in [false, true] {
                        let candidate = mcs_peo(&scratch.adj, &incumbent, reverse);
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
