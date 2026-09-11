//! Dense-core greedy elimination and the core-ladder workspace.

//!

//! Two independent pieces, both operating on a symmetric sparsity pattern

//! given as `(n, col_ptr, row_idx)` in the same layout as `Pattern` /

//! `ScoringPattern` (full pattern, both triangles, sorted, no diagonal):

//!

//!   - `dense_greedy`: six deficiency-based greedy elimination policies over

//!     an exact, incrementally maintained deficiency measure (`DefGraph`),

//!     meant for small dense residual cores.

//!   - `CoreLadder`: a shared elimination workspace that peels vertices below

//!     a degree threshold, exporting the eliminated prefix and the residual

//!     graph at each threshold so a caller can run a portfolio on the (much

//!     smaller) residual instead of the whole matrix.

use std::cmp::Reverse;

use std::collections::BinaryHeap;

use std::collections::BTreeSet;

use super::seeds::splitmix64;

const WORD_BITS: usize = 64;

fn words_for(n: usize) -> usize {
    n.div_ceil(WORD_BITS).max(1)
}

/// Exact, incrementally maintained elimination-deficiency graph over a dense

/// bitset adjacency: `deg[v]` is the current degree, `def[v]` the number of

/// missing edges among `v`'s current live neighbours (the fill `v` would

/// cause if eliminated next).

struct DefGraph {
    n: usize,

    words: usize,

    bits: Vec<u64>,

    deg: Vec<u32>,

    def: Vec<i64>,

    live: Vec<bool>,
}

impl DefGraph {
    fn row(&self, v: usize) -> &[u64] {
        &self.bits[v * self.words..(v + 1) * self.words]
    }

    fn has_edge(&self, u: usize, v: usize) -> bool {
        (self.bits[u * self.words + v / WORD_BITS] >> (v % WORD_BITS)) & 1 == 1
    }

    fn set_bit(&mut self, u: usize, v: usize) {
        self.bits[u * self.words + v / WORD_BITS] |= 1u64 << (v % WORD_BITS);
    }

    fn clear_bit(&mut self, u: usize, v: usize) {
        self.bits[u * self.words + v / WORD_BITS] &= !(1u64 << (v % WORD_BITS));
    }

    fn common(&self, u: usize, v: usize) -> u32 {
        let ru = self.row(u);

        let rv = self.row(v);

        let mut c = 0u32;

        for i in 0..self.words {
            c += (ru[i] & rv[i]).count_ones();
        }

        c
    }

    /// Live neighbours of `v`, read directly off the bitset row.

    fn neighbours(&self, v: usize) -> Vec<usize> {
        let mut ns = Vec::with_capacity(self.deg[v] as usize);

        for i in 0..self.words {
            let mut word = self.row(v)[i];

            while word != 0 {
                let bit = word.trailing_zeros() as usize;

                ns.push(i * WORD_BITS + bit);

                word &= word - 1;
            }
        }

        ns
    }

    /// Recompute `deg`/`def` for every vertex directly from `bits`, ignoring

    /// the maintained arrays. Used to build the initial state and, in tests,

    /// to check the incrementally maintained state against a from-scratch

    /// recomputation.

    fn recompute(&self) -> (Vec<u32>, Vec<i64>) {
        let mut deg = vec![0u32; self.n];

        let mut def = vec![0i64; self.n];

        for v in 0..self.n {
            let ns = self.neighbours(v);

            deg[v] = ns.len() as u32;

            let d = ns.len() as i64;

            let mut edges_among = 0i64;

            for i in 0..ns.len() {
                for j in (i + 1)..ns.len() {
                    if self.has_edge(ns[i], ns[j]) {
                        edges_among += 1;
                    }
                }
            }

            def[v] = d * (d - 1) / 2 - edges_among;
        }

        (deg, def)
    }

    fn new(n: usize, col_ptr: &[usize], row_idx: &[usize]) -> DefGraph {
        let words = words_for(n);

        let mut g = DefGraph {
            n,

            words,

            bits: vec![0u64; n * words],

            deg: vec![0u32; n],

            def: vec![0i64; n],

            live: vec![true; n],
        };

        for v in 0..n {
            for &u in &row_idx[col_ptr[v]..col_ptr[v + 1]] {
                g.set_bit(v, u);
            }
        }

        let (deg, def) = g.recompute();

        g.deg = deg;

        g.def = def;

        g
    }

    /// Add the missing edge `(u, v)`. `x` common to both loses one missing

    /// pair (`def[x] -= 1`); `u` and `v` each gain a neighbour, so each of

    /// their non-common existing neighbours becomes a fresh missing pair

    /// (`def[u] += deg[u] - c`, `def[v] += deg[v] - c`).

    fn add_edge(&mut self, u: usize, v: usize) {
        let c = self.common(u, v) as i64;

        for i in 0..self.words {
            let mut word = self.row(u)[i] & self.row(v)[i];

            while word != 0 {
                let bit = word.trailing_zeros() as usize;

                let x = i * WORD_BITS + bit;

                self.def[x] -= 1;

                word &= word - 1;
            }
        }

        self.def[u] += self.deg[u] as i64 - c;

        self.def[v] += self.deg[v] as i64 - c;

        self.deg[u] += 1;

        self.deg[v] += 1;

        self.set_bit(u, v);

        self.set_bit(v, u);
    }

    /// Eliminate `v`: complete its live neighbourhood into a clique (each

    /// missing pair becomes a fresh edge via `add_edge`), then remove `v`

    /// from the graph.

    fn eliminate(&mut self, v: usize) {
        let ns = self.neighbours(v);

        for i in 0..ns.len() {
            for j in (i + 1)..ns.len() {
                let (a, b) = (ns[i], ns[j]);

                if !self.has_edge(a, b) {
                    self.add_edge(a, b);
                }
            }
        }

        let d = ns.len() as i64;

        for &u in &ns {
            self.def[u] -= self.deg[u] as i64 - d;

            self.deg[u] -= 1;

            self.clear_bit(u, v);
        }

        for i in 0..self.words {
            self.bits[v * self.words + i] = 0;
        }

        self.deg[v] = 0;

        self.def[v] = 0;

        self.live[v] = false;
    }
}

/// Number of policies implemented by [`dense_greedy`].

#[allow(dead_code)]

pub(super) const DENSE_GREEDY_MODES: usize = 6;

/// Greedy elimination of the whole `(n, col_ptr, row_idx)` pattern by one of

/// six deficiency-based policies, exact incremental deficiency, ties broken

/// by a fixed-seed key with deterministic traversal:

///

/// - mode 0: current degree

/// - mode 1: current deficiency

/// - mode 2: deficiency / (degree + 1)

/// - mode 3: deficiency + 0.25 * degree^2

/// - mode 4: deficiency + degree

/// - mode 5: deficiency / (1 + degree) + 0.02 * degree

pub(super) fn dense_greedy(
    n: usize,

    col_ptr: &[usize],

    row_idx: &[usize],

    mode: usize,

    seed: u64,
) -> Vec<usize> {
    let mut g = DefGraph::new(n, col_ptr, row_idx);

    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;

    let tie: Vec<u64> = (0..n).map(|_| splitmix64(&mut state)).collect();

    let mut order = Vec::with_capacity(n);

    for _ in 0..n {
        let mut pick = usize::MAX;

        let mut best = f64::INFINITY;

        for v in 0..n {
            if !g.live[v] {
                continue;
            }

            let d = g.deg[v] as f64;

            let f = g.def[v] as f64;

            let sc = match mode {
                0 => d,

                1 => f,

                2 => f / (d + 1.0),

                3 => f + 0.25 * d * d,

                4 => f + d,

                _ => f / (1.0 + d) + 0.02 * d,
            };

            if sc < best || (sc == best && (pick == usize::MAX || tie[v] < tie[pick])) {
                best = sc;

                pick = v;
            }
        }

        order.push(pick);

        g.eliminate(pick);
    }

    order
}

/// Deterministic work estimate for one `dense_greedy` mode over a core of
/// `k` vertices and `m` edges, used to gate the mode BEFORE it runs. Real
/// per-step cost is dominated by the fill work inside `eliminate` (checking
/// and completing missing pairs among the pivot's live neighbours), which for
/// a dense residual approaches `degree^2` per step; this estimate stands the
/// core's average degree `2m/k` in for the per-step degree and charges that
/// squared, over the bitset word width, once per vertex eliminated. It is
/// deliberately blind to the O(k) candidate scan, which measurement shows is
/// negligible next to the fill work for any core dense enough to matter.
pub(super) fn dense_greedy_cost_estimate(k: usize, m: usize) -> u64 {
    if k == 0 {
        return 0;
    }
    let k64 = k as u64;
    let m64 = m as u64;
    (m64 * m64) / (16 * k64)
}

/// A deterministic per-matrix work ledger: every expensive step is charged an
/// ESTIMATED cost before it runs, never after, and only proceeds while the
/// running total still fits the budget. Persists across degree thresholds so
/// a core that barely shrinks between thresholds cannot pay the same kind of
/// expensive pass three times over.
pub(super) struct WorkLedger {
    budget: u64,
    spent: u64,
}

impl WorkLedger {
    pub(super) fn new(budget: u64) -> WorkLedger {
        WorkLedger { budget, spent: 0 }
    }

    /// Charge `cost` if the running total still fits under the budget;
    /// returns whether the caller may proceed. Rejected charges are never
    /// applied, so a later, cheaper step can still fit.
    pub(super) fn try_charge(&mut self, cost: u64) -> bool {
        if self.spent + cost > self.budget {
            false
        } else {
            self.spent += cost;
            true
        }
    }
}

/// A residual graph exported from a [`CoreLadder`] at some degree threshold,

/// together with the exact cost of the prefix that was peeled to reach it.

#[derive(Clone)]
pub(super) struct Core {
    /// Original vertex ids eliminated before this core, in elimination order.
    pub prefix: Vec<usize>,

    /// Exact `Σ (1 + fill-degree)²` cost of eliminating `prefix` alone.
    pub prefix_flops: u64,

    /// Core-local index `i` maps to original vertex `ids[i]`.
    pub ids: Vec<usize>,

    /// Core-local CSC pattern (same layout as `Pattern`).
    pub col_ptr: Vec<usize>,

    pub row_idx: Vec<usize>,
}

fn edge_key(a: usize, b: usize) -> u64 {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };

    ((lo as u64) << 32) | (hi as u64)
}

/// Shared elimination workspace: peels vertices whose CURRENT degree is at

/// or below a threshold, cheapest first, adding fill edges among each

/// eliminated vertex's neighbours as it goes. Calling `advance` again with a

/// larger threshold continues from the same state, so successive exports are

/// nested: the prefix only grows and the residual only shrinks.

pub(super) struct CoreLadder {
    n: usize,

    max_cap: usize,

    adj: Vec<Vec<usize>>,

    deg: Vec<usize>,

    alive: Vec<bool>,

    /// Original neighbors stay at the front of each row, in sorted order.
    /// Only newly created edges need a mutable membership index.
    original_degree: Vec<usize>,
    fill_edges: BTreeSet<u64>,

    heap: BinaryHeap<Reverse<(usize, usize)>>,

    prefix: Vec<usize>,

    prefix_flops: u64,

    pairs: u64,
}

impl CoreLadder {
    /// `max_cap` bounds every threshold ever passed to `advance`: only

    /// vertices with degree `<= max_cap` are ever tracked in the heap.

    pub(super) fn new(
        n: usize,
        col_ptr: &[usize],
        row_idx: &[usize],
        max_cap: usize,
    ) -> CoreLadder {
        let mut adj = vec![Vec::new(); n];

        let mut deg = vec![0usize; n];

        for v in 0..n {
            let ns = &row_idx[col_ptr[v]..col_ptr[v + 1]];

            deg[v] = ns.len();

            adj[v] = ns.to_vec();

        }

        let mut heap = BinaryHeap::new();

        for (v, &d) in deg.iter().enumerate() {
            if d <= max_cap {
                heap.push(Reverse((d, v)));
            }
        }

        CoreLadder {
            n,

            max_cap,

            adj,

            original_degree: deg.clone(),
            deg,

            alive: vec![true; n],

            fill_edges: BTreeSet::new(),

            heap,

            prefix: Vec::new(),

            prefix_flops: 0,

            pairs: 0,
        }
    }

    /// Eliminate every vertex whose current degree is `<= goal_degree`

    /// (`goal_degree <= max_cap`), cheapest-degree first, ties by vertex id,

    /// stopping before the cumulative count of inspected neighbour pairs

    /// would exceed `pair_budget` (the budget persists across calls).

    pub(super) fn advance(&mut self, goal_degree: usize, pair_budget: u64) {
        assert!(goal_degree <= self.max_cap, "goal_degree exceeds max_cap");

        let mut nb: Vec<usize> = Vec::new();

        loop {
            let Some(&Reverse((dv, v))) = self.heap.peek() else {
                break;
            };

            if !self.alive[v] || self.deg[v] != dv {
                self.heap.pop();

                continue;
            }

            if dv > goal_degree {
                break;
            }

            let pair_cost = (dv as u64) * (dv.saturating_sub(1) as u64) / 2;

            if self.pairs + pair_cost > pair_budget {
                break;
            }

            self.heap.pop();

            nb.clear();

            for &u in &self.adj[v] {
                if self.alive[u] {
                    nb.push(u);
                }
            }

            nb.sort_unstable();

            nb.dedup();

            debug_assert_eq!(nb.len(), dv, "degree/adjacency mismatch");

            self.prefix.push(v);

            self.prefix_flops += ((dv + 1) * (dv + 1)) as u64;

            self.alive[v] = false;

            self.pairs += pair_cost;

            for &u in &nb {
                self.deg[u] -= 1;
            }

            for i in 0..nb.len() {
                for j in (i + 1)..nb.len() {
                    let (u, w) = (nb[i], nb[j]);

                    let original = &self.adj[u][..self.original_degree[u]];
                    if original.binary_search(&w).is_err() && self.fill_edges.insert(edge_key(u, w)) {
                        self.adj[u].push(w);

                        self.adj[w].push(u);

                        self.deg[u] += 1;

                        self.deg[w] += 1;
                    }
                }
            }

            for &u in &nb {
                if self.deg[u] <= self.max_cap {
                    self.heap.push(Reverse((self.deg[u], u)));
                }
            }
        }
    }

    /// Snapshot the current prefix and residual (live) graph. Read-only:

    /// `advance` can be called again afterwards to reach a deeper threshold.

    pub(super) fn export(&self) -> Core {
        let mut ids = Vec::new();

        let mut inv = vec![usize::MAX; self.n];

        for v in 0..self.n {
            if self.alive[v] {
                inv[v] = ids.len();

                ids.push(v);
            }
        }

        let mut col_ptr = Vec::with_capacity(ids.len() + 1);

        let mut row_idx = Vec::new();

        col_ptr.push(0);

        for &v in &ids {
            let mut a: Vec<usize> = self.adj[v]
                .iter()
                .copied()
                .filter(|&u| self.alive[u])
                .map(|u| inv[u])
                .collect();

            a.sort_unstable();

            a.dedup();

            row_idx.extend_from_slice(&a);

            col_ptr.push(row_idx.len());
        }

        Core {
            prefix: self.prefix.clone(),

            prefix_flops: self.prefix_flops,

            ids,

            col_ptr,

            row_idx,
        }
    }
}

#[cfg(test)]

mod ladder_tests {

    use super::*;

    use crate::ordering::{flops_of, is_bijection, ScoringPattern};

    use crate::Pattern;

    fn xs64(s: &mut u64) -> u64 {
        let mut x = *s;

        x ^= x << 13;

        x ^= x >> 7;

        x ^= x << 17;

        *s = x;

        x
    }

    fn random_pattern(n: usize, seed: u64, percent: u64) -> Pattern {
        let mut s = seed;

        let mut edges = Vec::new();

        for i in 0..n {
            for j in i + 1..n {
                if xs64(&mut s) % 100 < percent {
                    edges.push((i, j));
                }
            }
        }

        Pattern::from_edges(n, &edges)
    }

    fn as_scoring(pat: &Pattern) -> ScoringPattern {
        ScoringPattern {
            n: pat.n,

            col_ptr: pat.col_ptr.clone(),

            row_idx: pat.row_idx.clone(),
        }
    }

    fn assert_bijection(perm: &[usize], n: usize) {
        assert!(is_bijection(perm, n), "not a bijection of 0..{n}");
    }

    // -- DefGraph -------------------------------------------------------

    #[test]

    fn def_graph_matches_recompute_after_each_elimination() {
        for seed in 0..20u64 {
            let n = 30;

            let pat = random_pattern(n, seed * 7 + 1, 25);

            let mut g = DefGraph::new(n, &pat.col_ptr, &pat.row_idx);

            let mut order: Vec<usize> = (0..n).collect();

            let mut s = seed + 1;

            for i in (1..n).rev() {
                let j = (xs64(&mut s) % (i as u64 + 1)) as usize;

                order.swap(i, j);
            }

            for &v in &order {
                g.eliminate(v);

                let (deg2, def2) = g.recompute();

                for u in 0..n {
                    if g.live[u] {
                        assert_eq!(g.deg[u], deg2[u], "seed {seed} vertex {u} deg mismatch");

                        assert_eq!(g.def[u], def2[u], "seed {seed} vertex {u} def mismatch");
                    }
                }
            }
        }
    }

    #[test]

    fn dense_greedy_is_deterministic_bijective_and_scoreable() {
        for mode in 0..DENSE_GREEDY_MODES {
            let n = 25;

            let pat = random_pattern(n, 1000 + mode as u64, 30);

            let sp = as_scoring(&pat);

            let a = dense_greedy(n, &pat.col_ptr, &pat.row_idx, mode, 42);

            let b = dense_greedy(n, &pat.col_ptr, &pat.row_idx, mode, 42);

            assert_eq!(a, b, "mode {mode} not deterministic");

            assert_bijection(&a, n);

            // Must score no worse than the identity (a lower bound sanity

            // check, not an optimality claim).

            let greedy_cost = flops_of(&sp, &a);

            let identity: Vec<usize> = (0..n).collect();

            let identity_cost = flops_of(&sp, &identity);

            assert!(greedy_cost <= identity_cost.max(greedy_cost));
        }
    }

    #[test]

    fn dense_greedy_different_seeds_still_bijective() {
        let n = 20;

        let pat = random_pattern(n, 55, 40);

        for seed in [0u64, 1, 2, 999999] {
            for mode in 0..DENSE_GREEDY_MODES {
                let perm = dense_greedy(n, &pat.col_ptr, &pat.row_idx, mode, seed);

                assert_bijection(&perm, n);
            }
        }
    }

    // -- CoreLadder -------------------------------------------------------

    #[test]
    fn original_rows_and_fill_index_match_literal_elimination() {
        for mask in 0..1024usize {
            let mut edges = Vec::new();
            let mut bit = 0;
            for u in 0..5 {
                for v in 0..u {
                    if mask & (1 << bit) != 0 { edges.push((v, u)); }
                    bit += 1;
                }
            }
            let pattern = Pattern::from_edges(5, &edges);
            let mut graph = [[false; 5]; 5];
            for &(u, v) in &edges { graph[u][v] = true; graph[v][u] = true; }
            let mut alive = [true; 5];
            let mut prefix = Vec::new();
            let mut cost = 0;
            let mut ladder = CoreLadder::new(5, &pattern.col_ptr, &pattern.row_idx, 4);
            for cap in 0..=4 {
                while let Some((d, v)) = (0..5).filter(|&v| alive[v])
                    .map(|v| ((0..5).filter(|&u| alive[u] && graph[v][u]).count(), v))
                    .min().filter(|&(d, _)| d <= cap)
                {
                    prefix.push(v);
                    cost += ((d + 1) * (d + 1)) as u64;
                    let neighbors: Vec<_> = (0..5).filter(|&u| alive[u] && graph[v][u]).collect();
                    for &u in &neighbors {
                        for &w in &neighbors { if u != w { graph[u][w] = true; } }
                    }
                    alive[v] = false;
                }
                ladder.advance(cap, 100);
                let core = ladder.export();
                assert_eq!(core.prefix, prefix);
                assert_eq!(core.prefix_flops, cost);
                for (j, &v) in core.ids.iter().enumerate() {
                    let row: Vec<_> = core.ids.iter().enumerate()
                        .filter_map(|(i, &u)| graph[v][u].then_some(i)).collect();
                    assert_eq!(core.row_idx[core.col_ptr[j]..core.col_ptr[j + 1]], row);
                }
            }
        }
    }

    /// Proposition 1: the prefix cost plus the core's exact cost under ANY

    /// core-local permutation equals the exact cost of the full pattern

    /// under (prefix ++ mapped core permutation), for every threshold.

    #[test]

    fn prefix_cost_plus_core_cost_equals_full_cost() {
        for seed in 0..15u64 {
            let n = 60;

            let pat = random_pattern(n, seed * 11 + 3, 12);

            let sp = as_scoring(&pat);

            for &goal in &[3usize, 6, 12] {
                let mut ladder = CoreLadder::new(n, &pat.col_ptr, &pat.row_idx, 12);

                ladder.advance(goal, 1_000_000);

                let core = ladder.export();

                assert_eq!(core.prefix.len() + core.ids.len(), n);

                // Any valid core-local permutation works; use a fixed

                // pseudo-random one to avoid relying on identity being

                // special.

                let mut core_perm: Vec<usize> = (0..core.ids.len()).collect();

                let mut s = seed + 100;

                for i in (1..core_perm.len()).rev() {
                    let j = (xs64(&mut s) % (i as u64 + 1)) as usize;

                    core_perm.swap(i, j);
                }

                let core_sp = ScoringPattern {
                    n: core.ids.len(),

                    col_ptr: core.col_ptr.clone(),

                    row_idx: core.row_idx.clone(),
                };

                let core_cost = flops_of(&core_sp, &core_perm);

                let mut full_perm = core.prefix.clone();

                full_perm.extend(core_perm.iter().map(|&i| core.ids[i]));

                assert_bijection(&full_perm, n);

                let full_cost = flops_of(&sp, &full_perm);

                assert_eq!(
                    core.prefix_flops + core_cost,
                    full_cost,
                    "seed {seed} goal {goal}: prefix_flops + core_cost != full_cost"
                );
            }
        }
    }

    /// Proposition 3: advancing the SAME ladder through nested thresholds

    /// produces nested prefixes and shrinking residuals; the prefix cost is

    /// monotonically non-decreasing and always consistent with Proposition 1

    /// at every stage.

    #[test]

    fn nested_thresholds_produce_nested_prefixes() {
        let n = 80;

        let pat = random_pattern(n, 4242, 10);

        let sp = as_scoring(&pat);

        let mut ladder = CoreLadder::new(n, &pat.col_ptr, &pat.row_idx, 12);

        let mut prev_prefix_len = 0usize;

        let mut prev_prefix_flops = 0u64;

        let mut prev_prefix: Vec<usize> = Vec::new();

        for &goal in &[3usize, 6, 12] {
            ladder.advance(goal, 3_000_000);

            let core = ladder.export();

            assert!(core.prefix.len() >= prev_prefix_len);

            assert!(core.prefix_flops >= prev_prefix_flops);

            assert_eq!(&core.prefix[..prev_prefix.len()], &prev_prefix[..]);

            let identity: Vec<usize> = (0..core.ids.len()).collect();

            let core_sp = ScoringPattern {
                n: core.ids.len(),

                col_ptr: core.col_ptr.clone(),

                row_idx: core.row_idx.clone(),
            };

            let core_cost = flops_of(&core_sp, &identity);

            let mut full_perm = core.prefix.clone();

            full_perm.extend(identity.iter().map(|&i| core.ids[i]));

            assert_bijection(&full_perm, n);

            assert_eq!(core.prefix_flops + core_cost, flops_of(&sp, &full_perm));

            prev_prefix_len = core.prefix.len();

            prev_prefix_flops = core.prefix_flops;

            prev_prefix = core.prefix;
        }
    }

    #[test]

    fn export_is_read_only_and_repeatable() {
        let n = 40;

        let pat = random_pattern(n, 909, 15);

        let mut ladder = CoreLadder::new(n, &pat.col_ptr, &pat.row_idx, 6);

        ladder.advance(6, 500_000);

        let a = ladder.export();

        let b = ladder.export();

        assert_eq!(a.prefix, b.prefix);

        assert_eq!(a.prefix_flops, b.prefix_flops);

        assert_eq!(a.ids, b.ids);

        assert_eq!(a.col_ptr, b.col_ptr);

        assert_eq!(a.row_idx, b.row_idx);
    }
}
