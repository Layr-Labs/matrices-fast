//! Event-driven completion minimalization.
//!
//! The existing MINL pass revisits every surviving fill edge for a fixed
//! number of rounds. This sibling pass records *why* a fill edge could not
//! be deleted. For a chordal graph `H`, a fill edge `uv` is not deletable
//! exactly when two vertices `x,y` in `N_H(u) ∩ N_H(v)` are nonadjacent.
//! The four-cycle `u-x-v-y-u` then has `uv` as its unique chord. That witness
//! remains valid until one of its four supporting fill edges is deleted, so
//! only that event needs to put `uv` back on the work queue.
//!
//! Four intrusive watcher nodes per fill edge keep the auxiliary memory
//! linear in the number of fill edges. All limits fail closed: stopping
//! early leaves a chordal supergraph of the original pattern, and the caller
//! still validates/scores the resulting PEO with the exact oracle.

use std::collections::VecDeque;

const NONE: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeleteResult {
    Deleted,
    Witness(u32, u32),
    TooWide,
    BudgetExhausted,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct WatcherResult {
    pub(super) removed: usize,
    pub(super) tests: usize,
    pub(super) exhausted: bool,
    pub(super) too_wide: usize,
}

/// Fixed-size intrusive reverse index from a watched fill edge to the
/// candidates whose current four-cycle witness depends on that edge.
///
/// Candidate `e` owns nodes `4*e .. 4*e+4`. A node is either detached or is
/// in exactly one doubly-linked list rooted at `head[support_edge]`, allowing
/// all four old watches to be replaced in O(1) apiece when `e` is retested.
struct WatchLists {
    head: Vec<u32>,
    prev: Vec<u32>,
    next: Vec<u32>,
    support: Vec<u32>,
}

impl WatchLists {
    fn new(fill_len: usize) -> Option<Self> {
        let nodes = fill_len.checked_mul(4)?;
        if nodes > (u32::MAX as usize) {
            return None;
        }
        Some(Self {
            head: vec![NONE; fill_len],
            prev: vec![NONE; nodes],
            next: vec![NONE; nodes],
            support: vec![NONE; nodes],
        })
    }

    fn detach(&mut self, node: usize) {
        let support = self.support[node];
        if support == NONE {
            return;
        }
        let prev = self.prev[node];
        let next = self.next[node];
        if prev == NONE {
            self.head[support as usize] = next;
        } else {
            self.next[prev as usize] = next;
        }
        if next != NONE {
            self.prev[next as usize] = prev;
        }
        self.prev[node] = NONE;
        self.next[node] = NONE;
        self.support[node] = NONE;
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
        self.prev[node] = NONE;
        self.next[node] = old_head;
        if old_head != NONE {
            self.prev[old_head as usize] = node as u32;
        }
        self.head[support] = node as u32;
    }

    /// Detach every watcher of a just-deleted support edge and enqueue the
    /// corresponding live candidate once. Its other watch nodes stay linked
    /// until the candidate is popped, when `detach_candidate` removes them.
    fn wake(
        &mut self,
        support: usize,
        alive: &[bool],
        queued: &mut [bool],
        queue: &mut VecDeque<u32>,
    ) {
        while self.head[support] != NONE {
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

/// Return the fill-edge id for an undirected pair, if the pair was fill in
/// the starting completion. `fill` is produced by the caller in lexicographic
/// `(u,v)` order, so a binary search is deterministic and allocation-free.
#[inline]
fn fill_edge_id(fill: &[(u32, u32)], a: u32, b: u32) -> Option<usize> {
    let edge = if a < b { (a, b) } else { (b, a) };
    fill.binary_search(&edge).ok()
}

/// Check whether the sorted common-neighbor set is a clique. On failure,
/// return an explicit nonedge `(x,y)` from that set: the four present edges
/// `ux,xv,uy,yv` are then precisely the supports watched by the caller.
fn common_clique_or_witness(
    adj: &[Vec<u32>],
    common: &[u32],
    ops: &mut i64,
) -> Result<(), Option<(u32, u32)>> {
    if common.len() <= 1 {
        return Ok(());
    }
    for &a in common {
        let row = &adj[a as usize];
        *ops -= (row.len() + common.len()) as i64;
        if *ops < 0 {
            return Err(None);
        }
        let mut i = 0usize;
        for &b in common {
            if b == a {
                continue;
            }
            while i < row.len() && row[i] < b {
                i += 1;
            }
            if i == row.len() || row[i] != b {
                return Err(Some((a, b)));
            }
        }
    }
    Ok(())
}

/// Attempt one exact chordality-preserving deletion and, on failure, expose
/// a four-cycle witness instead of returning only `false` as ordinary MINL
/// does. Adjacency rows remain sorted after a successful removal.
fn try_delete_with_witness(
    adj: &mut [Vec<u32>],
    u: u32,
    v: u32,
    ops: &mut i64,
    max_common: usize,
    common: &mut Vec<u32>,
) -> DeleteResult {
    let (uu, vv) = (u as usize, v as usize);
    let (au, av) = (&adj[uu], &adj[vv]);
    *ops -= (au.len() + av.len()) as i64;
    if *ops < 0 {
        return DeleteResult::BudgetExhausted;
    }

    common.clear();
    let (mut i, mut j) = (0usize, 0usize);
    while i < au.len() && j < av.len() {
        if au[i] < av[j] {
            i += 1;
        } else if au[i] > av[j] {
            j += 1;
        } else {
            common.push(au[i]);
            i += 1;
            j += 1;
        }
    }
    if common.len() > max_common {
        return DeleteResult::TooWide;
    }
    match common_clique_or_witness(adj, common, ops) {
        Err(Some((x, y))) => return DeleteResult::Witness(x, y),
        Err(None) => return DeleteResult::BudgetExhausted,
        Ok(()) => {}
    }

    // Only callers with `alive[e]` reach here, but fail closed if adjacency
    // and fill bookkeeping ever disagree instead of risking a one-sided row.
    let (Ok(pu), Ok(pv)) = (adj[uu].binary_search(&v), adj[vv].binary_search(&u)) else {
        return DeleteResult::TooWide;
    };
    adj[uu].remove(pu);
    adj[vv].remove(pv);
    DeleteResult::Deleted
}

/// Event-driven sibling of the existing round-based MINL scan.
///
/// `scan_order` must contain fill-edge ids (normally the existing
/// cheapest-degree-sum order). The routine deletes only fill edges for which
/// the local chordality test succeeds, and returns early/fail-closed on any
/// hard cap. If it drains the queue without a cap, every surviving fill edge
/// has a live unique-chord four-cycle witness, hence the resulting chordal
/// completion is inclusion-minimal.
pub(super) fn watcher_minimalize(
    adj: &mut [Vec<u32>],
    fill: &[(u32, u32)],
    scan_order: &[u32],
    mut ops: i64,
    max_common: usize,
) -> WatcherResult {
    let m = fill.len();
    if m == 0 || m > (u32::MAX as usize) {
        return WatcherResult::default();
    }
    let Some(mut watches) = WatchLists::new(m) else {
        return WatcherResult {
            exhausted: true,
            ..WatcherResult::default()
        };
    };
    let mut alive = vec![true; m];
    let mut queued = vec![false; m];
    let mut queue = VecDeque::with_capacity(m);
    for &edge in scan_order {
        let edge = edge as usize;
        if edge < m && !queued[edge] {
            queued[edge] = true;
            queue.push_back(edge as u32);
        }
    }

    let mut result = WatcherResult::default();
    let mut common = Vec::<u32>::new();
    while let Some(edge_u32) = queue.pop_front() {
        let edge = edge_u32 as usize;
        queued[edge] = false;
        if !alive[edge] {
            continue;
        }
        watches.detach_candidate(edge);
        result.tests += 1;
        let (u, v) = fill[edge];
        match try_delete_with_witness(adj, u, v, &mut ops, max_common, &mut common) {
            DeleteResult::Deleted => {
                alive[edge] = false;
                result.removed += 1;
                watches.wake(edge, &alive, &mut queued, &mut queue);
            }
            DeleteResult::Witness(x, y) => {
                let supports = [(u, x), (v, x), (u, y), (v, y)];
                let mut slot = 0usize;
                for &(a, b) in &supports {
                    let Some(support) = fill_edge_id(fill, a, b) else {
                        continue; // original edge: this support is permanent
                    };
                    if alive[support] {
                        watches.attach(edge, slot, support);
                        slot += 1;
                    }
                }
            }
            DeleteResult::TooWide => result.too_wide += 1,
            DeleteResult::BudgetExhausted => {
                result.exhausted = true;
                break;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adjacency(n: usize, edges: &[(u32, u32)]) -> Vec<Vec<u32>> {
        let mut adj = vec![Vec::new(); n];
        for &(u, v) in edges {
            if u == v {
                continue;
            }
            adj[u as usize].push(v);
            adj[v as usize].push(u);
        }
        for row in &mut adj {
            row.sort_unstable();
            row.dedup();
        }
        adj
    }

    fn has_edge(adj: &[Vec<u32>], u: usize, v: usize) -> bool {
        adj[u].binary_search(&(v as u32)).is_ok()
    }

    fn witness(adj: &[Vec<u32>], u: usize, v: usize) -> Option<(usize, usize)> {
        let mut common = Vec::new();
        for &x in &adj[u] {
            if adj[v].binary_search(&x).is_ok() {
                common.push(x as usize);
            }
        }
        for i in 0..common.len() {
            for j in i + 1..common.len() {
                if !has_edge(adj, common[i], common[j]) {
                    return Some((common[i], common[j]));
                }
            }
        }
        None
    }

    fn is_chordal(adj: &[Vec<u32>]) -> bool {
        let n = adj.len();
        let mut weight = vec![0usize; n];
        let mut used = vec![false; n];
        let mut visit = Vec::with_capacity(n);
        for _ in 0..n {
            let Some(v) = (0..n).filter(|&v| !used[v]).max_by_key(|&v| (weight[v], v)) else {
                return false;
            };
            used[v] = true;
            visit.push(v);
            for &u in &adj[v] {
                if !used[u as usize] {
                    weight[u as usize] += 1;
                }
            }
        }
        visit.reverse();
        let mut rank = vec![0usize; n];
        for (i, &v) in visit.iter().enumerate() {
            rank[v] = i;
        }
        for v in 0..n {
            let higher: Vec<usize> = adj[v]
                .iter()
                .map(|&u| u as usize)
                .filter(|&u| rank[u] > rank[v])
                .collect();
            for i in 0..higher.len() {
                for j in i + 1..higher.len() {
                    if !has_edge(adj, higher[i], higher[j]) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn fill_from_order(
        original: &[Vec<bool>],
        order: &[usize],
    ) -> (Vec<Vec<u32>>, Vec<(u32, u32)>) {
        let n = original.len();
        let mut h = original.to_vec();
        let mut live = vec![true; n];
        let mut fill = Vec::new();
        for &v in order {
            let neighbors: Vec<usize> = (0..n).filter(|&u| live[u] && h[v][u]).collect();
            for i in 0..neighbors.len() {
                for j in i + 1..neighbors.len() {
                    let (a, b) = (neighbors[i], neighbors[j]);
                    if !h[a][b] {
                        h[a][b] = true;
                        h[b][a] = true;
                        fill.push((a.min(b) as u32, a.max(b) as u32));
                    }
                }
            }
            live[v] = false;
        }
        fill.sort_unstable();
        fill.dedup();
        let mut edges = Vec::new();
        for u in 0..n {
            for v in u + 1..n {
                if h[u][v] {
                    edges.push((u as u32, v as u32));
                }
            }
        }
        (adjacency(n, &edges), fill)
    }

    fn cheapest_scan(adj: &[Vec<u32>], fill: &[(u32, u32)]) -> Vec<u32> {
        let mut ids: Vec<u32> = (0..fill.len() as u32).collect();
        ids.sort_unstable_by_key(|&i| {
            let (u, v) = fill[i as usize];
            (adj[u as usize].len() + adj[v as usize].len(), i)
        });
        ids
    }

    fn assert_minimal_completion(original: &[Vec<bool>], adj: &[Vec<u32>]) {
        let n = original.len();
        assert!(is_chordal(adj), "resulting completion is not chordal");
        for u in 0..n {
            for v in u + 1..n {
                if original[u][v] {
                    assert!(has_edge(adj, u, v), "original edge {u}-{v} was removed");
                } else if has_edge(adj, u, v) {
                    assert!(
                        witness(adj, u, v).is_some(),
                        "surviving fill edge {u}-{v} has no unique-chord witness"
                    );
                }
            }
        }
    }

    #[test]
    fn watcher_revisits_only_after_witness_support_deletion() {
        // Eliminating G in [0,3,1,4,2] creates fill edges 1-4 and 2-4.
        // Edge 1-4 is initially blocked by common nonneighbors 2 and 3; 2-4
        // is one of that witness's supports. Deleting 2-4 wakes 1-4, so the
        // queue removes both without a second global scan.
        let original_edges = [(0, 2), (1, 2), (1, 3), (3, 4)];
        let mut original = vec![vec![false; 5]; 5];
        for &(u, v) in &original_edges {
            original[u][v] = true;
            original[v][u] = true;
        }
        let (mut adj, fill) = fill_from_order(&original, &[0, 3, 1, 4, 2]);
        assert_eq!(fill, vec![(1, 4), (2, 4)]);
        let scan = cheapest_scan(&adj, &fill);
        assert_eq!(scan, vec![0, 1]);
        let result = watcher_minimalize(&mut adj, &fill, &scan, 1_000_000, usize::MAX);
        assert_eq!(result.removed, 2);
        assert_eq!(result.tests, 3); // blocked, support deleted, then wake/retest
        assert!(!result.exhausted);
        assert_minimal_completion(&original, &adj);
    }

    #[test]
    fn watcher_keeps_a_fill_edge_with_permanent_original_witness() {
        // A four-cycle plus one diagonal is already a minimal triangulation;
        // all four witness supports belong to the original graph.
        let original_edges = [(0, 1), (1, 2), (2, 3), (0, 3)];
        let filled_edges = [(0, 1), (1, 2), (2, 3), (0, 3), (0, 2)];
        let mut adj = adjacency(4, &filled_edges);
        let fill = [(0, 2)];
        let result = watcher_minimalize(&mut adj, &fill, &[0], 1_000_000, usize::MAX);
        assert_eq!(result.removed, 0);
        assert_eq!(result.tests, 1);
        assert!(has_edge(&adj, 0, 2));

        let mut original = vec![vec![false; 4]; 4];
        for &(u, v) in &original_edges {
            original[u][v] = true;
            original[v][u] = true;
        }
        assert_minimal_completion(&original, &adj);
    }

    fn permutations(n: usize) -> Vec<Vec<usize>> {
        fn rec(pos: usize, p: &mut [usize], out: &mut Vec<Vec<usize>>) {
            if pos == p.len() {
                out.push(p.to_vec());
                return;
            }
            for i in pos..p.len() {
                p.swap(pos, i);
                rec(pos + 1, p, out);
                p.swap(pos, i);
            }
        }
        let mut p: Vec<usize> = (0..n).collect();
        let mut out = Vec::new();
        rec(0, &mut p, &mut out);
        out
    }

    fn validate_case(original: &[Vec<bool>], order: &[usize]) {
        let (mut adj, fill) = fill_from_order(original, order);
        let scan = cheapest_scan(&adj, &fill);
        let result = watcher_minimalize(&mut adj, &fill, &scan, i64::MAX / 4, usize::MAX);
        assert!(!result.exhausted);
        assert_eq!(result.too_wide, 0);
        assert_minimal_completion(original, &adj);
    }

    #[test]
    fn watcher_is_minimal_on_exhaustive_small_and_deterministic_random_cases() {
        // Exhaust every labelled graph and every elimination order through
        // n=5 (124,470 graph/order pairs in total).
        for n in 0..=5usize {
            let pairs: Vec<(usize, usize)> = (0..n)
                .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
                .collect();
            let perms = permutations(n);
            for mask in 0usize..(1usize << pairs.len()) {
                let mut original = vec![vec![false; n]; n];
                for (bit, &(u, v)) in pairs.iter().enumerate() {
                    if ((mask >> bit) & 1) != 0 {
                        original[u][v] = true;
                        original[v][u] = true;
                    }
                }
                for order in &perms {
                    validate_case(&original, order);
                }
            }
        }

        // Then cover larger shapes with a fixed-seed generator. This is not
        // probabilistic test behavior: the exact same 20,000 cases run every
        // time, preserving the challenge's determinism contract.
        let mut state = 0xD1B5_4A32_D192_ED03u64;
        for case in 0..20_000usize {
            let n = 6 + (case % 4);
            let mut original = vec![vec![false; n]; n];
            for u in 0..n {
                for v in u + 1..n {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    if state % 5 < 2 {
                        original[u][v] = true;
                        original[v][u] = true;
                    }
                }
            }
            let mut order: Vec<usize> = (0..n).collect();
            for i in (1..n).rev() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                order.swap(i, (state as usize) % (i + 1));
            }
            validate_case(&original, &order);
        }
    }
}
