//! Reconstruct the completion, delete redundant fill and extract alternative PEOs.

use std::collections::VecDeque;

pub(super) struct CompletionLimits {
    pub max_n: usize,
    pub max_input_nnz: usize,
    pub max_lnnz: usize,
}

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

/// Adjacency retains discovery order; fill edges are sorted by endpoints.
struct Completion {
    adj: Vec<Vec<u32>>,
    fill: Vec<FillEdge>,
}

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
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    perm: &[usize],
    counts: &[u32],
    limits: &CompletionLimits,
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
