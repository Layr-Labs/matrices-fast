//! Completion-refinement: reconstruct the incumbent's chordal completion H
//! from its exact column counts, optionally delete redundant fill edges of
//! H, and re-extract perfect elimination orders of H by maximum cardinality
//! search (MCS). Eliminating the original graph along any perfect
//! elimination order of H produces a completion contained in H, so every
//! candidate this module returns scores no worse than the incumbent, and
//! strictly better exactly when H (as built from the incumbent) was not
//! already an inclusion-minimal triangulation.
//!
//! All graph work uses standard-library containers only. Nothing here reads
//! a clock or a random source; traversal is deterministic.

use std::collections::VecDeque;

/// Structural limits the reconstruction will trust: input graph size and
/// density, and the size of the reconstructed completion's factor.
pub(super) struct CompletionLimits {
    pub max_n: usize,
    pub max_input_nnz: usize,
    pub max_lnnz: usize,
}

/// An undirected fill edge of the reconstructed completion, always stored
/// with the smaller endpoint first so a sorted `Vec<FillEdge>` can be
/// binary-searched by endpoint pair.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct FillEdge {
    lo: u32,
    hi: u32,
}

impl FillEdge {
    fn new(a: u32, b: u32) -> Self {
        if a < b {
            FillEdge { lo: a, hi: b }
        } else {
            FillEdge { lo: b, hi: a }
        }
    }
}

/// The incumbent's chordal completion: adjacency in the order edges were
/// discovered while replaying the elimination, and the fill edges (present
/// in the completion, absent from the original graph) in ascending order.
struct Completion {
    adj: Vec<Vec<u32>>,
    fill: Vec<FillEdge>,
}

/// Perfect elimination orders of the incumbent's chordal completion, at most
/// eight, each a bijection of `0..n`. Returns `None` when a structural limit
/// is exceeded or the reconstruction cannot be trusted.
pub(super) fn peo_candidates(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    perm: &[usize],
    counts: &[u32],
    limits: &CompletionLimits,
) -> Option<Vec<Vec<usize>>> {
    let adj = reconstruct_graph(n, col_ptr, row_idx, perm, counts, limits)?;
    let sorted = sorted_rows(&adj);
    eight_variants(&adj, &sorted, perm)
}

/// Reverse-neighbor MCS changes the elimination tree without leaving the
/// incumbent completion. Generate this one order without sorting another
/// adjacency copy or constructing the seven unused variants.
#[cfg(test)]
pub(super) fn alternate_peo(
    n: usize, col_ptr: &[usize], row_idx: &[usize], perm: &[usize],
    counts: &[u32], limits: &CompletionLimits,
) -> Option<Vec<usize>> {
    let adj = reconstruct_graph(n, col_ptr, row_idx, perm, counts, limits)?;
    let order = mcs_peo(&adj, perm, true);
    is_permutation(&order, n).then_some(order)
}

/// The same, after first deleting redundant fill edges from the completion
/// under a deterministic work budget. `reward` receives the exact potential
/// drop `sum (3 + 2|common|)` over the deleted edges, a lower bound on the
/// score improvement available from the returned candidates.
pub(super) fn minimalized_peo_candidates(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    perm: &[usize],
    counts: &[u32],
    limits: &CompletionLimits,
    budget: u64,
    reward: &mut u64,
) -> Option<Vec<Vec<usize>>> {
    *reward = 0;
    let completion = reconstruct(n, col_ptr, row_idx, perm, counts, limits)?;
    let mut sorted = sorted_rows(&completion.adj);
    if completion.fill.is_empty() {
        return eight_variants(&completion.adj, &sorted, perm);
    }
    let scan_order = cheapest_scan_order(&completion.fill, &sorted);
    let deleted = minimalize(&mut sorted, &completion.fill, &scan_order, budget, reward);
    let raw_pruned = prune_raw(&completion.adj, &completion.fill, &deleted);
    eight_variants(&raw_pruned, &sorted, perm)
}

/// Replay the elimination game along `perm`, folding each column's reach
/// into its parent (the minimum later index in that reach) instead of
/// storing a full elimination tree, and check the result against the exact
/// column counts the caller supplied. Any mismatch, and any input shape
/// that cannot be trusted, returns `None` rather than panicking.
fn reconstruct_graph(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    perm: &[usize],
    counts: &[u32],
    limits: &CompletionLimits,
) -> Option<Vec<Vec<u32>>> {
    if n == 0 || n > limits.max_n {
        return None;
    }
    if col_ptr.len() != n + 1 || row_idx.len() > limits.max_input_nnz {
        return None;
    }
    if col_ptr[0] != 0 || col_ptr[n] != row_idx.len() {
        return None;
    }
    if col_ptr.windows(2).any(|w| w[0] > w[1]) {
        return None;
    }
    if row_idx.iter().any(|&v| v >= n) {
        return None;
    }
    if counts.len() != n || !is_permutation(perm, n) {
        return None;
    }

    let lnnz = counts
        .iter()
        .try_fold(0usize, |sum, &c| sum.checked_add(c as usize))?;
    if lnnz > limits.max_lnnz {
        return None;
    }

    let mut inverse = vec![usize::MAX; n];
    for (rank, &vertex) in perm.iter().enumerate() {
        inverse[vertex] = rank;
    }

    // pending[j] collects, before column j is processed, every later index
    // whose own reach folded j in as its minimum (its elimination-tree
    // parent). This reproduces the elimination tree without ever building
    // one explicitly.
    let mut pending: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut mark = vec![usize::MAX; n];

    for rank in 0..n {
        let vertex = perm[rank];
        let mut reach = Vec::<u32>::new();
        for &neighbor in &row_idx[col_ptr[vertex]..col_ptr[vertex + 1]] {
            let neighbor_rank = inverse[neighbor];
            if neighbor_rank > rank && mark[neighbor_rank] != rank {
                mark[neighbor_rank] = rank;
                reach.push(neighbor_rank as u32);
            }
        }
        for &folded in &pending[rank] {
            if folded > rank && mark[folded] != rank {
                mark[folded] = rank;
                reach.push(folded as u32);
            }
        }
        pending[rank] = Vec::new();

        if reach.len().checked_add(1)? != counts[rank] as usize {
            return None;
        }

        let mut parent_rank = n;
        for &r in &reach {
            parent_rank = parent_rank.min(r as usize);
        }
        for &r in &reach {
            let other = perm[r as usize];
            adj[vertex].push(other as u32);
            adj[other].push(vertex as u32);
            if r as usize != parent_rank {
                pending[parent_rank].push(r as usize);
            }
        }
    }

    Some(adj)
}

fn reconstruct(
    n: usize, col_ptr: &[usize], row_idx: &[usize], perm: &[usize],
    counts: &[u32], limits: &CompletionLimits,
) -> Option<Completion> {
    let adj = reconstruct_graph(n, col_ptr, row_idx, perm, counts, limits)?;
    let mut fill = Vec::new();
    for v in 0..n {
        let mut row = adj[v].clone();
        row.sort_unstable();
        let original = &row_idx[col_ptr[v]..col_ptr[v + 1]];
        for &w in &row {
            if (w as usize) > v && original.binary_search(&(w as usize)).is_err() {
                fill.push(FillEdge::new(v as u32, w));
            }
        }
    }
    Some(Completion { adj, fill })
}

fn sorted_rows(adj: &[Vec<u32>]) -> Vec<Vec<u32>> {
    adj.iter()
        .map(|row| {
            let mut sorted = row.clone();
            sorted.sort_unstable();
            sorted
        })
        .collect()
}

/// Fill-edge ids in the order the watcher should first test them: cheapest
/// endpoint degree sum first, ties broken by the edge's own id so the order
/// depends only on the completion, never on iteration or hashing.
fn cheapest_scan_order(fill: &[FillEdge], adj: &[Vec<u32>]) -> Vec<u32> {
    let mut ids: Vec<u32> = (0..fill.len() as u32).collect();
    ids.sort_unstable_by_key(|&id| {
        let edge = fill[id as usize];
        (
            adj[edge.lo as usize].len() + adj[edge.hi as usize].len(),
            id,
        )
    });
    ids
}

const NO_NODE: u32 = u32::MAX;

/// Fixed-size intrusive reverse index from a watched fill edge to the
/// candidates whose current four-cycle witness depends on that edge.
/// Candidate `e` owns nodes `4*e .. 4*e+4`; a node is either detached or
/// lives in exactly one doubly-linked list rooted at `head[support]`, so all
/// four of a candidate's watches can be replaced in O(1) apiece.
struct WatchLists {
    head: Vec<u32>,
    prev: Vec<u32>,
    next: Vec<u32>,
    support: Vec<u32>,
}

impl WatchLists {
    fn new(fill_len: usize) -> Self {
        let nodes = fill_len * 4;
        WatchLists {
            head: vec![NO_NODE; fill_len],
            prev: vec![NO_NODE; nodes],
            next: vec![NO_NODE; nodes],
            support: vec![NO_NODE; nodes],
        }
    }

    fn detach(&mut self, node: usize) {
        let support = self.support[node];
        if support == NO_NODE {
            return;
        }
        let prev = self.prev[node];
        let next = self.next[node];
        if prev == NO_NODE {
            self.head[support as usize] = next;
        } else {
            self.next[prev as usize] = next;
        }
        if next != NO_NODE {
            self.prev[next as usize] = prev;
        }
        self.prev[node] = NO_NODE;
        self.next[node] = NO_NODE;
        self.support[node] = NO_NODE;
    }

    fn detach_candidate(&mut self, candidate: usize) {
        let base = candidate * 4;
        for node in base..base + 4 {
            self.detach(node);
        }
    }

    fn attach(&mut self, candidate: usize, slot: usize, support: usize) {
        let node = candidate * 4 + slot;
        self.detach(node);
        let old_head = self.head[support];
        self.support[node] = support as u32;
        self.prev[node] = NO_NODE;
        self.next[node] = old_head;
        if old_head != NO_NODE {
            self.prev[old_head as usize] = node as u32;
        }
        self.head[support] = node as u32;
    }

    /// Detach every watcher of a just-deleted support edge and enqueue the
    /// corresponding live candidate once.
    fn wake(
        &mut self,
        support: usize,
        alive: &[bool],
        queued: &mut [bool],
        queue: &mut VecDeque<u32>,
    ) {
        while self.head[support] != NO_NODE {
            let node = self.head[support] as usize;
            let candidate = node / 4;
            self.detach(node);
            if alive[candidate] && !queued[candidate] {
                queued[candidate] = true;
                queue.push_back(candidate as u32);
            }
        }
    }
}

/// Fill-edge id for an unordered pair, if that pair is itself a fill edge.
/// `fill` is sorted ascending, so this is a plain binary search.
fn fill_edge_id(fill: &[FillEdge], a: u32, b: u32) -> Option<usize> {
    fill.binary_search(&FillEdge::new(a, b)).ok()
}

const MAX_COMMON: usize = 1024;

/// Delete every fill edge of `adj` whose common neighbourhood is a clique,
/// working a FIFO queue seeded by `scan_order` and re-testing a candidate
/// only when a witness that blocked it is itself deleted. `adj` must be
/// sorted per row on entry and remains sorted per row throughout. Returns,
/// per fill-edge id, whether that edge was deleted.
///
/// Every scan is charged against `budget` before it runs: `deg(u)+deg(v)`
/// for the common-neighbourhood merge, then `deg(x)+|common|` for each
/// common neighbour `x` checked against the rest of that neighbourhood.
/// Running out mid-test stops the whole pass, leaving `adj` a valid (if not
/// fully minimalized) chordal completion.
fn minimalize(
    adj: &mut [Vec<u32>],
    fill: &[FillEdge],
    scan_order: &[u32],
    budget: u64,
    reward: &mut u64,
) -> Vec<bool> {
    let count = fill.len();
    let mut deleted = vec![false; count];
    if count == 0 {
        return deleted;
    }
    let mut ops = budget.min(i64::MAX as u64) as i64;
    let mut alive = vec![true; count];
    let mut queued = vec![false; count];
    let mut queue: VecDeque<u32> = VecDeque::with_capacity(count);
    for &edge in scan_order {
        let idx = edge as usize;
        if idx < count && !queued[idx] {
            queued[idx] = true;
            queue.push_back(edge);
        }
    }

    let mut watches = WatchLists::new(count);
    let mut common = Vec::<u32>::new();

    'edges: while let Some(edge_id) = queue.pop_front() {
        let edge = edge_id as usize;
        queued[edge] = false;
        if !alive[edge] {
            continue;
        }
        watches.detach_candidate(edge);
        let FillEdge { lo, hi } = fill[edge];
        let (u, v) = (lo as usize, hi as usize);

        ops -= (adj[u].len() + adj[v].len()) as i64;
        if ops < 0 {
            break;
        }

        common.clear();
        {
            let (mut i, mut j) = (0usize, 0usize);
            while i < adj[u].len() && j < adj[v].len() {
                if adj[u][i] < adj[v][j] {
                    i += 1;
                } else if adj[u][i] > adj[v][j] {
                    j += 1;
                } else {
                    common.push(adj[u][i]);
                    i += 1;
                    j += 1;
                }
            }
        }
        if common.len() > MAX_COMMON {
            continue;
        }

        let mut witness: Option<(u32, u32)> = None;
        for &x in &common {
            let row_len = adj[x as usize].len();
            ops -= (row_len + common.len()) as i64;
            if ops < 0 {
                break 'edges;
            }
            let row = &adj[x as usize];
            let mut k = 0usize;
            for &y in &common {
                if y == x {
                    continue;
                }
                while k < row.len() && row[k] < y {
                    k += 1;
                }
                if k == row.len() || row[k] != y {
                    witness = Some((x, y));
                    break;
                }
            }
            if witness.is_some() {
                break;
            }
        }

        if let Some((x, y)) = witness {
            let mut slot = 0usize;
            for &(a, b) in &[(lo, x), (hi, x), (lo, y), (hi, y)] {
                if let Some(support) = fill_edge_id(fill, a, b) {
                    if alive[support] {
                        watches.attach(edge, slot, support);
                        slot += 1;
                    }
                }
            }
            continue;
        }

        // Common neighbourhood is a clique: the edge can be deleted while
        // preserving chordality. Fail closed (skip, keep the edge) if the
        // adjacency somehow disagrees with the fill bookkeeping.
        let (Ok(pu), Ok(pv)) = (adj[u].binary_search(&hi), adj[v].binary_search(&lo)) else {
            continue;
        };
        adj[u].remove(pu);
        adj[v].remove(pv);
        alive[edge] = false;
        deleted[edge] = true;
        *reward = reward.saturating_add(3 + 2 * common.len() as u64);
        watches.wake(edge, &alive, &mut queued, &mut queue);
    }
    deleted
}

/// Apply the same edge deletions to the raw-encounter-order adjacency,
/// preserving each surviving neighbour's original relative position.
fn prune_raw(adj_raw: &[Vec<u32>], fill: &[FillEdge], deleted: &[bool]) -> Vec<Vec<u32>> {
    let n = adj_raw.len();
    let mut removed: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (id, edge) in fill.iter().enumerate() {
        if deleted[id] {
            removed[edge.lo as usize].push(edge.hi);
            removed[edge.hi as usize].push(edge.lo);
        }
    }
    let mut pruned = Vec::with_capacity(n);
    for v in 0..n {
        let mut skip = removed[v].clone();
        skip.sort_unstable();
        let row: Vec<u32> = adj_raw[v]
            .iter()
            .copied()
            .filter(|w| skip.binary_search(w).is_err())
            .collect();
        pruned.push(row);
    }
    pruned
}

/// Maximum cardinality search over `adj`, seeded so the first vertex
/// visited is `seed[0]`; ties among equal-weight vertices resolve toward
/// whichever was pushed onto its bucket last. Later neighbours of each
/// visited vertex are pushed either in `adj` row order or its reverse.
/// Returns the elimination order (a perfect elimination order of `adj` when
/// `adj` is chordal).
fn mcs_peo(adj: &[Vec<u32>], seed: &[usize], reverse_scan: bool) -> Vec<usize> {
    let n = adj.len();
    let mut weight = vec![0usize; n];
    let mut visited = vec![false; n];
    let mut buckets: Vec<Vec<u32>> = vec![seed.iter().rev().map(|&v| v as u32).collect()];
    let mut max_weight = 0usize;
    let mut visit = Vec::with_capacity(n);
    while visit.len() < n {
        let Some(v) = buckets[max_weight].pop() else {
            if max_weight == 0 {
                break;
            }
            max_weight -= 1;
            continue;
        };
        let v = v as usize;
        if visited[v] || weight[v] != max_weight {
            continue;
        }
        visited[v] = true;
        visit.push(v);
        for offset in 0..adj[v].len() {
            let index = if reverse_scan {
                adj[v].len() - 1 - offset
            } else {
                offset
            };
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
    visit.reverse();
    visit
}

/// Eight deterministic tie variants: `{basis_a, basis_b}` (typically raw
/// encounter order and sorted adjacency) crossed with `{seeded at perm's
/// first vertex, seeded at perm's last vertex}` crossed with `{forward,
/// reverse}` adjacency scan order. `None` if any variant fails to come back
/// a bijection of `0..perm.len()`.
fn eight_variants(
    basis_a: &[Vec<u32>],
    basis_b: &[Vec<u32>],
    perm: &[usize],
) -> Option<Vec<Vec<usize>>> {
    let n = perm.len();
    let forward_seed = perm.to_vec();
    let backward_seed: Vec<usize> = perm.iter().rev().copied().collect();
    let mut candidates = Vec::with_capacity(8);
    for basis in [basis_a, basis_b] {
        for seed in [&forward_seed, &backward_seed] {
            for reverse_scan in [false, true] {
                let candidate = mcs_peo(basis, seed, reverse_scan);
                if !is_permutation(&candidate, n) {
                    return None;
                }
                candidates.push(candidate);
            }
        }
    }
    Some(candidates)
}

fn is_permutation(order: &[usize], n: usize) -> bool {
    if order.len() != n {
        return false;
    }
    let mut seen = vec![false; n];
    for &v in order {
        if v >= n || seen[v] {
            return false;
        }
        seen[v] = true;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_LIMITS: CompletionLimits = CompletionLimits {
        max_n: 200,
        max_input_nnz: 20_000,
        max_lnnz: 200_000,
    };

    #[test]
    fn single_alternate_matches_portfolio_and_does_not_increase_literal_cost() {
        let mut seed = 612789;
        for n in [6, 12, 25, 40] {
            for density in [1, 3, 5] {
                let a = random_graph(n, density, 7, &mut seed);
                let (cp, ri) = pattern_of(&a);
                let p = random_permutation(n, &mut seed);
                let (_, counts) = literal_replay(&a, &p);
                let q = alternate_peo(n, &cp, &ri, &p, &counts, &TEST_LIMITS).unwrap();
                assert_eq!(q, peo_candidates(n, &cp, &ri, &p, &counts, &TEST_LIMITS).unwrap()[1]);
                let (_, actual) = literal_replay(&a, &q);
                assert!(actual.iter().map(|&c| (c as u64).pow(2)).sum::<u64>()
                    <= counts.iter().map(|&c| (c as u64).pow(2)).sum::<u64>());
            }
        }
    }

    #[test]
    fn flip_proposals_preserve_boundary_partition_and_literal_scores() {
        let mut seed = 77199;
        for n in [12, 25, 40] {
            let a = random_graph(n, 1, 5, &mut seed);
            let (cp, ri) = pattern_of(&a);
            let p = random_permutation(n, &mut seed);
            let (_, counts) = literal_replay(&a, &p);
            for free in [n / 2, n] {
                for q in flip_candidates(n, &cp, &ri, &p, &counts, free).unwrap() {
                    assert!(is_permutation(&q, n));
                    assert!(q[..free].iter().all(|&v| v < free));
                    let (_, c) = literal_replay(&a, &q);
                    let graph = super::super::ScoringPattern {
                        n,
                        col_ptr: cp.clone(),
                        row_idx: ri.clone(),
                    };
                    assert_eq!(
                        super::super::flops_of(&graph, &q),
                        c.iter().map(|&v| (v as u64).pow(2)).sum::<u64>()
                    );
                }
            }
        }
    }
    fn next_rand(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    fn random_graph(n: usize, edge_num: u64, edge_den: u64, state: &mut u64) -> Vec<Vec<bool>> {
        let mut adj = vec![vec![false; n]; n];
        for u in 0..n {
            for v in u + 1..n {
                if next_rand(state) % edge_den < edge_num {
                    adj[u][v] = true;
                    adj[v][u] = true;
                }
            }
        }
        adj
    }

    fn random_permutation(n: usize, state: &mut u64) -> Vec<usize> {
        let mut order: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = (next_rand(state) as usize) % (i + 1);
            order.swap(i, j);
        }
        order
    }

    fn pattern_of(adj: &[Vec<bool>]) -> (Vec<usize>, Vec<usize>) {
        let n = adj.len();
        let mut col_ptr = vec![0usize];
        let mut row_idx = Vec::new();
        for v in 0..n {
            for u in 0..n {
                if adj[v][u] {
                    row_idx.push(u);
                }
            }
            col_ptr.push(row_idx.len());
        }
        (col_ptr, row_idx)
    }

    /// Literal elimination-game replay: no elimination tree, no folding,
    /// just the textbook "connect all surviving neighbours, then remove the
    /// eliminated vertex" loop. Independent of `reconstruct` for
    /// cross-checking.
    fn literal_replay(adj: &[Vec<bool>], order: &[usize]) -> (Vec<Vec<bool>>, Vec<u32>) {
        let n = adj.len();
        let mut filled = adj.to_vec();
        let mut live = vec![true; n];
        let mut counts = vec![0u32; n];
        for (k, &v) in order.iter().enumerate() {
            let neighbors: Vec<usize> = (0..n).filter(|&u| live[u] && filled[v][u]).collect();
            counts[k] = (neighbors.len() + 1) as u32;
            for &a in &neighbors {
                for &b in &neighbors {
                    if a != b {
                        filled[a][b] = true;
                    }
                }
            }
            live[v] = false;
        }
        (filled, counts)
    }

    fn literal_score(adj: &[Vec<bool>], order: &[usize]) -> u64 {
        let (_, counts) = literal_replay(adj, order);
        counts.iter().map(|&c| (c as u64) * (c as u64)).sum()
    }

    fn is_permutation_literal(order: &[usize], n: usize) -> bool {
        if order.len() != n {
            return false;
        }
        let mut seen = vec![false; n];
        for &v in order {
            if v >= n || seen[v] {
                return false;
            }
            seen[v] = true;
        }
        true
    }

    /// A literal perfect-elimination-order check: for every vertex, its
    /// later neighbours in `order` must form a clique in `adj`.
    fn is_peo_literal(adj: &[Vec<bool>], order: &[usize]) -> bool {
        let n = adj.len();
        if !is_permutation_literal(order, n) {
            return false;
        }
        for (k, &v) in order.iter().enumerate() {
            let later: Vec<usize> = order[k + 1..]
                .iter()
                .copied()
                .filter(|&u| adj[v][u])
                .collect();
            for &a in &later {
                for &b in &later {
                    if a != b && !adj[a][b] {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// A literal chordality check via a from-scratch maximum cardinality
    /// search (independent of `mcs_peo`) followed by the same PEO property
    /// check used above.
    fn is_chordal_literal(adj: &[Vec<bool>]) -> bool {
        let n = adj.len();
        let mut weight = vec![0usize; n];
        let mut used = vec![false; n];
        let mut order = Vec::with_capacity(n);
        for _ in 0..n {
            let Some(v) = (0..n).filter(|&v| !used[v]).max_by_key(|&v| (weight[v], v)) else {
                return false;
            };
            used[v] = true;
            order.push(v);
            for u in 0..n {
                if !used[u] && adj[v][u] {
                    weight[u] += 1;
                }
            }
        }
        order.reverse();
        is_peo_literal(adj, &order)
    }

    fn adj_to_bool(n: usize, adj: &[Vec<u32>]) -> Vec<Vec<bool>> {
        let mut out = vec![vec![false; n]; n];
        for v in 0..n {
            for &u in &adj[v] {
                out[v][u as usize] = true;
            }
        }
        out
    }

    fn fill_of(g: &[Vec<bool>], h: &[Vec<bool>]) -> Vec<(usize, usize)> {
        let n = g.len();
        let mut out = Vec::new();
        for u in 0..n {
            for v in u + 1..n {
                if h[u][v] && !g[u][v] {
                    out.push((u, v));
                }
            }
        }
        out
    }

    #[test]
    fn reconstruction_matches_literal_replay_and_rejects_bad_counts() {
        let mut state = 0x9E3779B97F4A7C15u64;
        for case in 0..2_000usize {
            let n = 2 + (case % 10);
            let graph = random_graph(n, 2, 5, &mut state);
            let order = random_permutation(n, &mut state);
            let (col_ptr, row_idx) = pattern_of(&graph);
            let (filled, counts) = literal_replay(&graph, &order);

            let completion = reconstruct(n, &col_ptr, &row_idx, &order, &counts, &TEST_LIMITS)
                .expect("reconstruction must succeed with exact counts");
            let reconstructed_bool = adj_to_bool(n, &completion.adj);
            assert_eq!(
                reconstructed_bool, filled,
                "H must equal the literal replay completion"
            );

            // H contains G.
            for u in 0..n {
                for v in 0..n {
                    if graph[u][v] {
                        assert!(
                            reconstructed_bool[u][v],
                            "H must contain every original edge"
                        );
                    }
                }
            }

            let mut expected_fill = fill_of(&graph, &filled);
            expected_fill.sort_unstable();
            let mut actual_fill: Vec<(usize, usize)> = completion
                .fill
                .iter()
                .map(|e| (e.lo as usize, e.hi as usize))
                .collect();
            actual_fill.sort_unstable();
            assert_eq!(
                actual_fill, expected_fill,
                "fill set must match the literal replay's new edges"
            );

            // Corrupting one column count must fail closed, not panic.
            let mut bad_counts = counts.clone();
            bad_counts[0] = bad_counts[0].wrapping_add(1);
            assert!(
                reconstruct(n, &col_ptr, &row_idx, &order, &bad_counts, &TEST_LIMITS).is_none()
            );
        }
    }

    #[test]
    fn mcs_variants_are_bijections_and_perfect_elimination_orders() {
        let mut state = 0x1234_5678_9ABC_DEF1u64;
        for case in 0..1_000usize {
            let n = 2 + (case % 12);
            let graph = random_graph(n, 1, 3, &mut state);
            let order = random_permutation(n, &mut state);
            let (col_ptr, row_idx) = pattern_of(&graph);
            let (filled, counts) = literal_replay(&graph, &order);

            let candidates = peo_candidates(n, &col_ptr, &row_idx, &order, &counts, &TEST_LIMITS)
                .expect("well-formed input must produce candidates");
            assert_eq!(candidates.len(), 8);
            for candidate in &candidates {
                assert!(is_permutation_literal(candidate, n));
                assert!(
                    is_peo_literal(&filled, candidate),
                    "candidate must be a PEO of H"
                );
            }
        }
    }

    #[test]
    fn candidates_never_cost_more_and_sometimes_cost_less() {
        let mut state = 0xC0FF_EE00_1234_5678u64;
        let mut strict_improvements = 0usize;
        for case in 0..3_000usize {
            let n = 3 + (case % 10);
            let graph = random_graph(n, 2, 5, &mut state);
            let order = random_permutation(n, &mut state);
            let (col_ptr, row_idx) = pattern_of(&graph);
            let (_, counts) = literal_replay(&graph, &order);
            let baseline = literal_score(&graph, &order);

            let Some(candidates) =
                peo_candidates(n, &col_ptr, &row_idx, &order, &counts, &TEST_LIMITS)
            else {
                continue;
            };
            for candidate in &candidates {
                let score = literal_score(&graph, candidate);
                assert!(score <= baseline, "re-extracted order must never cost more");
                if score < baseline {
                    strict_improvements += 1;
                }
            }
        }
        assert!(
            strict_improvements > 0,
            "some random case must show a strict improvement"
        );
    }

    #[test]
    fn deletion_criterion_and_exact_potential_drop() {
        fn common_neighbors(adj: &[Vec<bool>], u: usize, v: usize) -> Vec<usize> {
            (0..adj.len())
                .filter(|&x| x != u && x != v && adj[u][x] && adj[v][x])
                .collect()
        }
        fn is_clique(adj: &[Vec<bool>], verts: &[usize]) -> bool {
            for &a in verts {
                for &b in verts {
                    if a != b && !adj[a][b] {
                        return false;
                    }
                }
            }
            true
        }
        fn potential(adj: &[Vec<bool>]) -> u64 {
            let n = adj.len();
            let mut edges = 0u64;
            let mut triangles = 0u64;
            for u in 0..n {
                for v in u + 1..n {
                    if adj[u][v] {
                        edges += 1;
                        for w in v + 1..n {
                            if adj[u][w] && adj[v][w] {
                                triangles += 1;
                            }
                        }
                    }
                }
            }
            n as u64 + 3 * edges + 2 * triangles
        }

        let mut state = 0xABCD_EF01_2345_6789u64;
        let mut checked = 0usize;
        for case in 0..2_000usize {
            let n = 3 + (case % 9);
            let graph = random_graph(n, 2, 5, &mut state);
            let order = random_permutation(n, &mut state);
            let (h, _) = literal_replay(&graph, &order);
            assert!(is_chordal_literal(&h));

            for u in 0..n {
                for v in u + 1..n {
                    if !h[u][v] || graph[u][v] {
                        continue; // only fill edges of H are ever deletion candidates
                    }
                    checked += 1;
                    let common = common_neighbors(&h, u, v);
                    let clique = is_clique(&h, &common);

                    let mut deleted = h.clone();
                    deleted[u][v] = false;
                    deleted[v][u] = false;
                    let still_chordal = is_chordal_literal(&deleted);
                    assert_eq!(
                        clique, still_chordal,
                        "deletable iff common neighbourhood is a clique"
                    );

                    if clique {
                        let before = potential(&h);
                        let after = potential(&deleted);
                        assert_eq!(before - after, 3 + 2 * common.len() as u64);
                    }
                }
            }
        }
        assert!(
            checked > 0,
            "the random corpus must exercise at least one fill edge"
        );
    }

    #[test]
    fn identical_inputs_produce_byte_identical_output() {
        let mut state = 0x0BAD_F00D_DEAD_BEEFu64;
        for case in 0..200usize {
            let n = 3 + (case % 8);
            let graph = random_graph(n, 2, 5, &mut state);
            let order = random_permutation(n, &mut state);
            let (col_ptr, row_idx) = pattern_of(&graph);
            let (_, counts) = literal_replay(&graph, &order);

            let a = peo_candidates(n, &col_ptr, &row_idx, &order, &counts, &TEST_LIMITS);
            let b = peo_candidates(n, &col_ptr, &row_idx, &order, &counts, &TEST_LIMITS);
            assert_eq!(a, b);

            let mut reward_a = 0u64;
            let mut reward_b = 0u64;
            let ma = minimalized_peo_candidates(
                n,
                &col_ptr,
                &row_idx,
                &order,
                &counts,
                &TEST_LIMITS,
                10_000,
                &mut reward_a,
            );
            let mb = minimalized_peo_candidates(
                n,
                &col_ptr,
                &row_idx,
                &order,
                &counts,
                &TEST_LIMITS,
                10_000,
                &mut reward_b,
            );
            assert_eq!(ma, mb);
            assert_eq!(reward_a, reward_b);
        }
    }

    #[test]
    fn budget_exhaustion_stays_valid_and_zero_budget_matches_unminimalized() {
        let mut state = 0x5EED_5EED_5EED_5EEDu64;
        for case in 0..500usize {
            let n = 3 + (case % 10);
            let graph = random_graph(n, 2, 5, &mut state);
            let order = random_permutation(n, &mut state);
            let (col_ptr, row_idx) = pattern_of(&graph);
            let (_, counts) = literal_replay(&graph, &order);

            let plain = peo_candidates(n, &col_ptr, &row_idx, &order, &counts, &TEST_LIMITS);

            for &budget in &[0u64, 1, 2, 5] {
                let mut reward = 0u64;
                let result = minimalized_peo_candidates(
                    n,
                    &col_ptr,
                    &row_idx,
                    &order,
                    &counts,
                    &TEST_LIMITS,
                    budget,
                    &mut reward,
                );
                if let Some(candidates) = &result {
                    for candidate in candidates {
                        assert!(
                            is_permutation_literal(candidate, n),
                            "must never return a non-bijection"
                        );
                    }
                }
                if budget == 0 {
                    assert_eq!(
                        result, plain,
                        "zero budget must delete nothing and match the unminimalized path"
                    );
                    assert_eq!(reward, 0);
                }
            }
        }
    }
}

/// Completion escape by saturating a fill edge's common neighborhood and
/// removing that edge. Returned orders are proposals, scored on the input.
pub(super) fn flip_candidates(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    perm: &[usize],
    counts: &[u32],
    free: usize,
) -> Option<Vec<Vec<usize>>> {
    if n > 1200 || free > n {
        return None;
    }
    let limits = CompletionLimits {
        max_n: 1200,
        max_input_nnz: 100_000,
        max_lnnz: 100_000,
    };
    let h = reconstruct(n, cp, ri, perm, counts, &limits)?;
    let sorted = sorted_rows(&h.adj);
    let mut budget = 1_000_000usize;
    let mut choices = Vec::new();
    for edge in &h.fill {
        let (u, v) = (edge.lo as usize, edge.hi as usize);
        if u >= free || v >= free {
            continue;
        }
        let charge = sorted[u].len() + sorted[v].len();
        if charge > budget {
            break;
        }
        budget -= charge;
        let mut common = Vec::new();
        let (mut a, mut b) = (0, 0);
        while a < sorted[u].len() && b < sorted[v].len() {
            if sorted[u][a] < sorted[v][b] {
                a += 1;
            } else if sorted[u][a] > sorted[v][b] {
                b += 1;
            } else {
                common.push(sorted[u][a]);
                a += 1;
                b += 1;
            }
        }
        if common.len() > 64 {
            continue;
        }
        let charge = common.len() * common.len();
        if charge > budget {
            break;
        }
        budget -= charge;
        let mut missing = 0usize;
        for i in 0..common.len() {
            for j in i + 1..common.len() {
                if sorted[common[i] as usize]
                    .binary_search(&common[j])
                    .is_err()
                {
                    missing += 1;
                }
            }
        }
        if missing > 0 && missing <= 64 {
            choices.push((missing, common.len() + 1, u, v, common));
        }
    }
    choices.sort_by(|a, b| {
        (a.0 * b.1)
            .cmp(&(b.0 * a.1))
            .then(a.2.cmp(&b.2))
            .then(a.3.cmp(&b.3))
    });
    choices.truncate(8);
    let mut out = Vec::new();
    for (_, _, u, v, common) in choices {
        let charge: usize = sorted.iter().map(Vec::len).sum();
        if charge > budget {
            break;
        }
        budget -= charge;
        let mut q = sorted.clone();
        for &x in &common {
            for &y in &common {
                if x != y {
                    if let Err(pos) = q[x as usize].binary_search(&y) {
                        q[x as usize].insert(pos, y);
                    }
                }
            }
        }
        let a = q[u].binary_search(&(v as u32)).ok()?;
        q[u].remove(a);
        let b = q[v].binary_search(&(u as u32)).ok()?;
        q[v].remove(b);
        // MCS yields a candidate even if the modified completion is not chordal;
        // no containment/optimality claim is used for accepting its input cost.
        let p = mcs_peo(&q, perm, false);
        let mut candidate: Vec<_> = p.iter().copied().filter(|&v| v < free).collect();
        candidate.extend(p.into_iter().filter(|&v| v >= free));
        if is_permutation(&candidate, n) {
            out.push(candidate);
        }
    }
    Some(out)
}
