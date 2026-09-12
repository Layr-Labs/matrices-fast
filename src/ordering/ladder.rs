use super::seeds::splitmix64;
use std::collections::BTreeSet;

pub(super) fn dense_greedy(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    mode: usize,
    seed: u64,
    work: &mut usize,
) -> Vec<usize> {
    if n == 0 {
        return Vec::new();
    }
    let mut g = super::patch::Dense::from_pattern(n, cp, ri, n);
    if mode != 0 && g.init_def(work).is_none() {
        return Vec::new();
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
            let Some(left) = work.checked_sub(n + (g.deg[pick] + 1) * (g.w + 3)) else {
                return Vec::new();
            };
            *work = left;
            g.elim(pick);
        } else if g.defelim(pick, work).is_none() {
            return Vec::new();
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

pub(super) struct CoreLadder<'a> {
    n: usize,
    max_cap: usize,
    adj: Vec<Vec<usize>>,
    col_ptr: &'a [usize],
    row_idx: &'a [usize],
    deg: Vec<usize>,
    alive: Vec<bool>,
    fill_edges: Vec<FillEdges>,
    queue: DegreeQueue,
    prefix: Vec<usize>,
    prefix_flops: u64,
    pairs: u64,
}

struct DegreeQueue {
    bits: Vec<u64>,
    words: Vec<u64>,
    groups: Vec<u64>,
}

impl DegreeQueue {
    fn new(size: usize) -> Self {
        let words = size.div_ceil(64);
        let groups = words.div_ceil(64);
        Self {
            bits: vec![0; words],
            words: vec![0; groups],
            groups: vec![0; groups.div_ceil(64)],
        }
    }

    fn insert(&mut self, key: usize) {
        let word = key / 64;
        let group = word / 64;
        self.bits[word] |= 1 << (key % 64);
        self.words[group] |= 1 << (word % 64);
        self.groups[group / 64] |= 1 << (group % 64);
    }

    fn remove(&mut self, key: usize) {
        let word = key / 64;
        self.bits[word] &= !(1 << (key % 64));
        if self.bits[word] == 0 {
            let group = word / 64;
            self.words[group] &= !(1 << (word % 64));
            if self.words[group] == 0 {
                self.groups[group / 64] &= !(1 << (group % 64));
            }
        }
    }

    fn first(&self) -> Option<usize> {
        let top = self.groups.iter().position(|&v| v != 0)?;
        let group = top * 64 + self.groups[top].trailing_zeros() as usize;
        let word = group * 64 + self.words[group].trailing_zeros() as usize;
        Some(word * 64 + self.bits[word].trailing_zeros() as usize)
    }
}

#[derive(Clone)]
enum FillEdges {
    Small(Vec<usize>),
    Large(BTreeSet<usize>),
}

impl FillEdges {
    fn insert(&mut self, v: usize) -> bool {
        match self {
            Self::Large(edges) => edges.insert(v),
            Self::Small(edges) => {
                let Err(at) = edges.binary_search(&v) else {
                    return false;
                };
                if edges.len() < 32 {
                    edges.insert(at, v);
                } else {
                    let mut tree: BTreeSet<_> = edges.iter().copied().collect();
                    tree.insert(v);
                    *self = Self::Large(tree);
                }
                true
            }
        }
    }
}

impl<'a> CoreLadder<'a> {
    pub(super) fn new(n: usize, cp: &'a [usize], ri: &'a [usize], max_cap: usize) -> Self {
        let deg: Vec<_> = cp.windows(2).map(|p| p[1] - p[0]).collect();
        let mut queue = DegreeQueue::new((max_cap + 1) * n);
        for (v, &d) in deg.iter().enumerate() {
            if d <= max_cap {
                queue.insert(d * n + v);
            }
        }
        Self {
            n,
            max_cap,
            adj: vec![Vec::new(); n],
            col_ptr: cp,
            row_idx: ri,
            deg,
            alive: vec![true; n],
            fill_edges: vec![FillEdges::Small(Vec::new()); n],
            queue,
            prefix: Vec::new(),
            prefix_flops: 0,
            pairs: 0,
        }
    }

    pub(super) fn advance(&mut self, goal_degree: usize, pair_budget: u64) {
        let mut neighbors = Vec::new();
        let mut before = Vec::new();
        loop {
            let Some(key) = self.queue.first() else {
                break;
            };
            let (degree, v) = (key / self.n, key % self.n);
            if degree > goal_degree {
                break;
            }
            let pairs = degree as u64 * degree.saturating_sub(1) as u64 / 2;
            if self.pairs + pairs > pair_budget {
                break;
            }
            self.queue.remove(key);
            neighbors.clear();
            neighbors.extend(
                self.row_idx[self.col_ptr[v]..self.col_ptr[v + 1]]
                    .iter()
                    .chain(&self.adj[v])
                    .copied()
                    .filter(|&u| self.alive[u]),
            );
            neighbors.sort_unstable();
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
                    if self.row_idx[self.col_ptr[u]..self.col_ptr[u + 1]]
                        .binary_search(&w)
                        .is_err()
                        && self.fill_edges[u].insert(w)
                    {
                        self.adj[u].push(w);
                        self.adj[w].push(u);
                        self.deg[u] += 1;
                        self.deg[w] += 1;
                    }
                }
            }
            for (i, &u) in neighbors.iter().enumerate() {
                if self.deg[u] != before[i] {
                    if before[i] <= self.max_cap {
                        self.queue.remove(before[i] * self.n + u);
                    }
                    if self.deg[u] <= self.max_cap {
                        self.queue.insert(self.deg[u] * self.n + u);
                    }
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
            let start = row_idx.len();
            row_idx.extend(
                self.row_idx[self.col_ptr[v]..self.col_ptr[v + 1]]
                    .iter()
                    .chain(&self.adj[v])
                    .copied()
                    .filter(|&u| self.alive[u])
                    .map(|u| inverse[u]),
            );
            row_idx[start..].sort_unstable();
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
