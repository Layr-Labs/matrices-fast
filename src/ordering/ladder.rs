use super::seeds::splitmix64;
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};

pub(super) fn dense_greedy(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    mode: usize,
    seed: u64,
) -> Vec<usize> {
    if n == 0 {
        return Vec::new();
    }
    let mut g = super::patch::Dense::from_pattern(n, cp, ri, n);
    if mode != 0 {
        g.init_def();
    }
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
            let score = match mode {
                0 => d,
                1 => f,
                2 => f / (d + 1.0),
                3 => f + 0.25 * d * d,
                4 => f + d,
                _ => f / (1.0 + d) + 0.02 * d,
            };
            if score < best || (score == best && (pick == usize::MAX || tie[v] < tie[pick])) {
                best = score;
                pick = v;
            }
        }
        order.push(pick);
        if mode == 0 {
            g.elim(pick);
        } else {
            g.defelim(pick);
        }
    }
    order
}

pub(super) fn dense_greedy_cost_estimate(k: usize, m: usize) -> u64 {
    if k == 0 {
        0
    } else {
        (m as u64 * m as u64) / (16 * k as u64)
    }
}

pub(super) struct WorkLedger {
    budget: u64,
    spent: u64,
}

impl WorkLedger {
    pub(super) fn new(budget: u64) -> Self {
        Self { budget, spent: 0 }
    }
    pub(super) fn try_charge(&mut self, cost: u64) -> bool {
        if self.spent + cost > self.budget {
            return false;
        }
        self.spent += cost;
        true
    }
}

#[derive(Clone)]
pub(super) struct Core {
    pub prefix: Vec<usize>,
    pub prefix_flops: u64,
    pub ids: Vec<usize>,
    pub col_ptr: Vec<usize>,
    pub row_idx: Vec<usize>,
}

fn edge_key(a: usize, b: usize) -> u64 {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    ((lo as u64) << 32) | hi as u64
}

pub(super) struct CoreLadder {
    n: usize,
    max_cap: usize,
    adj: Vec<Vec<usize>>,
    deg: Vec<usize>,
    alive: Vec<bool>,
    original_degree: Vec<usize>,
    fill_edges: BTreeSet<u64>,
    heap: BinaryHeap<Reverse<(usize, usize)>>,
    prefix: Vec<usize>,
    prefix_flops: u64,
    pairs: u64,
}

impl CoreLadder {
    pub(super) fn new(n: usize, cp: &[usize], ri: &[usize], max_cap: usize) -> Self {
        let adj: Vec<Vec<usize>> = (0..n).map(|v| ri[cp[v]..cp[v + 1]].to_vec()).collect();
        let deg: Vec<_> = adj.iter().map(Vec::len).collect();
        let heap = deg
            .iter()
            .enumerate()
            .filter(|&(_, &d)| d <= max_cap)
            .map(|(v, &d)| Reverse((d, v)))
            .collect();
        Self {
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

    pub(super) fn advance(&mut self, goal_degree: usize, pair_budget: u64) {
        let mut neighbors = Vec::new();
        let mut before = Vec::new();
        loop {
            let Some(&Reverse((degree, v))) = self.heap.peek() else {
                break;
            };
            if !self.alive[v] || self.deg[v] != degree {
                self.heap.pop();
                continue;
            }
            if degree > goal_degree {
                break;
            }
            let pairs = degree as u64 * degree.saturating_sub(1) as u64 / 2;
            if self.pairs + pairs > pair_budget {
                break;
            }
            self.heap.pop();
            neighbors.clear();
            neighbors.extend(self.adj[v].iter().copied().filter(|&u| self.alive[u]));
            neighbors.sort_unstable();
            neighbors.dedup();
            before.clear();
            before.extend(neighbors.iter().map(|&u| self.deg[u]));
            self.prefix.push(v);
            self.prefix_flops += ((degree + 1) * (degree + 1)) as u64;
            self.alive[v] = false;
            self.pairs += pairs;
            for &u in &neighbors {
                self.deg[u] -= 1;
            }
            for i in 0..neighbors.len() {
                for j in i + 1..neighbors.len() {
                    let (u, w) = (neighbors[i], neighbors[j]);
                    let original = &self.adj[u][..self.original_degree[u]];
                    if original.binary_search(&w).is_err() && self.fill_edges.insert(edge_key(u, w))
                    {
                        self.adj[u].push(w);
                        self.adj[w].push(u);
                        self.deg[u] += 1;
                        self.deg[w] += 1;
                    }
                }
            }
            for (i, &u) in neighbors.iter().enumerate() {
                if self.deg[u] <= self.max_cap && self.deg[u] != before[i] {
                    self.heap.push(Reverse((self.deg[u], u)));
                }
            }
        }
    }

    pub(super) fn export(&self) -> Core {
        let mut ids = Vec::new();
        let mut inverse = vec![usize::MAX; self.n];
        for v in 0..self.n {
            if self.alive[v] {
                inverse[v] = ids.len();
                ids.push(v);
            }
        }
        let mut col_ptr = Vec::with_capacity(ids.len() + 1);
        let mut row_idx = Vec::new();
        col_ptr.push(0);
        for &v in &ids {
            let mut row: Vec<_> = self.adj[v]
                .iter()
                .copied()
                .filter(|&u| self.alive[u])
                .map(|u| inverse[u])
                .collect();
            row.sort_unstable();
            row.dedup();
            row_idx.extend_from_slice(&row);
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
