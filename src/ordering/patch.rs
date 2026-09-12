//! Exact elimination of a free prefix with a frozen exterior, including twin blocks.
use std::sync::Arc;

/// Fixed-seed xorshift generator; the pattern is the only input.
pub(super) struct Rng(u64);
impl Rng {
    pub(super) fn new(seed: u64) -> Rng {
        Rng(seed | 1)
    }
    pub(super) fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

pub(super) fn sum_squares(t: u64) -> u64 {
    t * (t + 1) * (2 * t + 1) / 6
}

fn bits(row: &[u64], limit: usize, mut f: impl FnMut(usize)) {
    for (j, &word) in row.iter().enumerate() {
        let mut b = word;
        while b != 0 {
            let u = j * 64 + b.trailing_zeros() as usize;
            b &= b - 1;
            if u < limit {
                f(u);
            }
        }
    }
}

/// Dense patch graph. `def[v]` is the number of missing pairs among the
/// current neighbors of an eliminable `v` once `init_def` has run.
#[derive(Clone)]
pub(super) struct Dense {
    pub n: usize,
    pub k: usize,
    pub w: usize,
    pub a: Vec<u64>,
    pub deg: Vec<usize>,
    pub(super) def: Vec<i64>,
    pub live: Vec<bool>,
    words: Vec<(usize, u64)>,
}

pub(super) fn row_words(row: &[u64]) -> Vec<(usize, u64)> {
    let mut words = Vec::with_capacity(row.len());
    words.extend(
        row.iter()
            .copied()
            .enumerate()
            .filter(|&(_, bits)| bits != 0),
    );
    words
}

/// Count missing pairs once; vertices beyond `k` must form a frozen clique.
pub(super) fn deficiency(
    g: &[u64],
    w: usize,
    v: usize,
    k: usize,
    words: &mut Vec<(usize, u64)>,
) -> usize {
    words.clear();
    words.extend(
        g[v * w..(v + 1) * w]
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, bits)| bits != 0),
    );
    #[cfg(target_arch = "x86_64")]
    if words.len() >= 8 && is_x86_feature_detected!("popcnt") {
        return unsafe { deficiency_popcnt(g, w, k, words) };
    }
    deficiency_words(g, w, k, words)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "popcnt")]
unsafe fn deficiency_popcnt(g: &[u64], w: usize, k: usize, words: &[(usize, u64)]) -> usize {
    deficiency_words(g, w, k, words)
}

#[inline(always)]
fn deficiency_words(g: &[u64], w: usize, k: usize, words: &[(usize, u64)]) -> usize {
    let mut missing = 0;
    for (position, &(j, mut bits)) in words.iter().enumerate() {
        if j * 64 >= k {
            break;
        }
        while bits != 0 {
            let u = j * 64 + bits.trailing_zeros() as usize;
            bits &= bits - 1;
            if u >= k {
                break;
            }
            missing += (bits & !g[u * w + j]).count_ones() as usize;
            missing += words[position + 1..]
                .iter()
                .map(|&(t, neighbors)| (neighbors & !g[u * w + t]).count_ones() as usize)
                .sum::<usize>();
        }
    }
    missing
}

#[inline]
pub(super) fn fill_row(target: &mut [u64], pivot: &[(usize, u64)], u: usize, v: usize) -> u32 {
    #[cfg(target_arch = "x86_64")]
    if pivot.len() >= 8 && is_x86_feature_detected!("popcnt") {
        return unsafe { fill_row_popcnt(target, pivot, u, v) };
    }
    fill_row_words(target, pivot, u, v)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "popcnt")]
unsafe fn fill_row_popcnt(target: &mut [u64], pivot: &[(usize, u64)], u: usize, v: usize) -> u32 {
    fill_row_words(target, pivot, u, v)
}

#[inline(always)]
fn fill_row_words(target: &mut [u64], pivot: &[(usize, u64)], u: usize, v: usize) -> u32 {
    let mut added = 0;
    for &(j, bits) in pivot {
        added += (bits & !target[j]).count_ones();
        target[j] |= bits;
    }
    target[u / 64] &= !(1 << (u % 64));
    target[v / 64] &= !(1 << (v % 64));
    added
}

impl Dense {
    pub(super) fn from_pattern(n: usize, cp: &[usize], ri: &[usize], k: usize) -> Self {
        let w = n.div_ceil(64);
        let mut a = vec![0; n * w];
        for v in 0..n {
            for &u in &ri[cp[v]..cp[v + 1]] {
                if u < n && u != v {
                    a[v * w + u / 64] |= 1 << (u % 64);
                    a[u * w + v / 64] |= 1 << (v % 64);
                }
            }
        }
        Self::from_bits(a, n, k)
    }
    pub(super) fn from_bits(a: Vec<u64>, n: usize, k: usize) -> Dense {
        let w = n.div_ceil(64);
        let deg = (0..n)
            .map(|v| {
                a[v * w..(v + 1) * w]
                    .iter()
                    .map(|x| x.count_ones() as usize)
                    .sum()
            })
            .collect();
        Dense {
            n,
            k,
            w,
            a,
            deg,
            def: vec![0; n],
            live: vec![true; n],
            words: Vec::new(),
        }
    }
    pub(super) fn has(&self, u: usize, v: usize) -> bool {
        (self.a[u * self.w + v / 64] >> (v % 64)) & 1 == 1
    }
    fn edge(&mut self, u: usize, v: usize) {
        if u == v || self.has(u, v) {
            return;
        }
        self.a[u * self.w + v / 64] |= 1u64 << (v % 64);
        self.a[v * self.w + u / 64] |= 1u64 << (u % 64);
        self.deg[u] += 1;
        self.deg[v] += 1;
    }
    pub(super) fn nb(&self, v: usize) -> Vec<usize> {
        let mut r = Vec::with_capacity(self.deg[v]);
        bits(&self.a[v * self.w..(v + 1) * self.w], self.n, |u| r.push(u));
        r
    }
    /// Eliminate `v`: neighbors become a clique. Returns its column cost.
    pub(super) fn elim(&mut self, v: usize) -> u64 {
        let w = self.w;
        let mut row = std::mem::take(&mut self.words);
        row.clear();
        row.extend(
            self.a[v * w..(v + 1) * w]
                .iter()
                .copied()
                .enumerate()
                .filter(|&(_, bits)| bits != 0),
        );
        let q = (self.deg[v] + 1) as u64;
        for &(j, mut bits) in &row {
            while bits != 0 {
                let u = j * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                let added = fill_row(&mut self.a[u * w..(u + 1) * w], &row, u, v);
                self.deg[u] = self.deg[u] + added as usize - 2;
            }
        }
        self.words = row;
        self.a[v * w..(v + 1) * w].fill(0);
        self.live[v] = false;
        self.deg[v] = 0;
        q * q
    }
    pub(super) fn score(&self, order: &[usize]) -> u64 {
        let mut b = self.clone();
        order.iter().map(|&u| b.elim(u)).sum()
    }
    /// Recompress the residual after an incumbent prefix, retaining its exact
    /// incurred cost. Newly identical free/frozen neighborhoods are kept apart.
    pub(super) fn residual_order(
        &self,
        order: &[usize],
        tail: usize,
        budget: &mut usize,
        initial: &Atoms,
    ) -> Option<(u64, Vec<usize>)> {
        let blocks = initial.project(order);
        let normalized = initial.expand(&blocks);
        let prefix: usize = blocks[..blocks.len().saturating_sub(tail)]
            .iter()
            .map(|&v| initial.members[v].len())
            .sum();
        let order = normalized.as_slice();
        let mut b = self.clone();
        let fixed: u64 = order[..prefix].iter().map(|&v| b.elim(v)).sum();
        let mut ids = order[prefix..].to_vec();
        let free = ids.len();
        ids.extend(self.k..self.n);
        let n = ids.len();
        let w = n.div_ceil(64);
        let mut inverse = vec![usize::MAX; self.n];
        for (i, &v) in ids.iter().enumerate() {
            inverse[v] = i;
        }
        let mut graph = vec![0u64; n * w];
        for (i, &v) in ids.iter().enumerate() {
            for u in b.nb(v) {
                graph[i * w + inverse[u] / 64] |= 1 << (inverse[u] % 64);
            }
        }
        let residual = Dense::from_bits(graph, n, free);
        let atoms = Atoms::new(&residual);
        let projected = atoms.project(&(0..free).collect::<Vec<_>>());
        let (cost, block_order) = if atoms.k <= 19 {
            let charge = atoms.k.max(1) * (1usize << atoms.k);
            *budget = budget.checked_sub(charge)?;
            atoms.exact()?
        } else {
            let balanced = atoms.k <= 112;
            let charge = if balanced {
                (atoms.k / 2 + 1) * (atoms.k - atoms.k / 2 + 1) * atoms.n * atoms.w
            } else {
                atoms.k * atoms.n * 32
            };
            *budget = budget.checked_sub(charge)?;
            if balanced {
                atoms.interleave(&projected)
            } else {
                atoms.joint(&projected)?
            }
        };
        let mut candidate = order[..prefix].to_vec();
        candidate.extend(atoms.expand(&block_order).iter().map(|&v| ids[v]));
        Some((fixed + cost, candidate))
    }
    pub(super) fn init_def(&mut self, work: &mut usize) -> Option<()> {
        let mut words = Vec::with_capacity(self.w);
        for v in 0..self.k {
            *work = work.checked_sub((self.deg[v] + 1) * (self.w + 1))?;
            self.def[v] = deficiency(&self.a, self.w, v, self.n, &mut words) as i64;
        }
        Some(())
    }
    /// Add edge (u,v) keeping every eliminable deficiency exact.
    fn defedge(&mut self, u: usize, v: usize) {
        let (w, k) = (self.w, self.k);
        let mut common = 0i64;
        for j in 0..w {
            let mut z = self.a[u * w + j] & self.a[v * w + j];
            common += z.count_ones() as i64;
            if j * 64 >= k {
                continue;
            }
            if j * 64 + 64 > k {
                z &= (1u64 << (k % 64)) - 1;
            }
            while z != 0 {
                let x = j * 64 + z.trailing_zeros() as usize;
                z &= z - 1;
                self.def[x] -= 1;
            }
        }
        if u < k {
            self.def[u] += self.deg[u] as i64 - common;
        }
        if v < k {
            self.def[v] += self.deg[v] as i64 - common;
        }
        self.edge(u, v);
    }
    /// Eliminate `v` while maintaining deficiencies. Returns its column cost.
    pub(super) fn defelim(&mut self, v: usize, work: &mut usize) -> Option<u64> {
        // A large fill event can touch each deficiency once per new edge.
        // Recompute all deficiencies when that exceeds a full bitset recount.
        let incremental = self.def[v].max(0) as usize * (self.k + self.w);
        *work = work.checked_sub(self.k + (self.deg[v] + 1) * (self.w + 3))?;
        if incremental > self.k * self.n * self.w {
            let cost = self.elim(v);
            self.init_def(work)?;
            return Some(cost);
        }
        *work = work.checked_sub(incremental)?;
        Some(self.defelim_incremental(v))
    }

    fn defelim_incremental(&mut self, v: usize) -> u64 {
        let (w, k) = (self.w, self.k);
        let d = self.deg[v];
        let ns = self.nb(v);
        for &u in &ns {
            for j in u / 64..w {
                let mut z = self.a[v * w + j] & !self.a[u * w + j];
                if j == u / 64 {
                    z &= if u % 64 == 63 {
                        0
                    } else {
                        !0u64 << (u % 64 + 1)
                    };
                }
                while z != 0 {
                    let t = j * 64 + z.trailing_zeros() as usize;
                    z &= z - 1;
                    if t < self.n {
                        self.defedge(u, t);
                    }
                }
            }
        }
        for &u in &ns {
            if u < k {
                self.def[u] -= self.deg[u] as i64 - d as i64;
            }
            self.a[u * w + v / 64] &= !(1u64 << (v % 64));
            self.deg[u] -= 1;
        }
        self.a[v * w..(v + 1) * w].fill(0);
        self.deg[v] = 0;
        self.live[v] = false;
        ((d + 1) as u64).pow(2)
    }
    /// Greedy elimination of the eliminable vertices. Mode 0 is minimum degree;
    /// modes 1..=7 combine exact deficiency `f` and degree `d`.
    pub(super) fn greedy(
        &self,
        mode: usize,
        seed: u64,
        work: &mut usize,
    ) -> Option<(u64, Vec<usize>)> {
        let mut b = self.clone();
        let mut rng = Rng::new(seed);
        let tie: Vec<u64> = (0..b.k).map(|_| rng.next()).collect();
        let mut order = Vec::with_capacity(b.k);
        let mut cost = 0u64;
        if mode == 0 {
            for _ in 0..b.k {
                let mut v = usize::MAX;
                for u in 0..b.k {
                    if b.live[u]
                        && (v == usize::MAX
                            || b.deg[u] < b.deg[v]
                            || (b.deg[u] == b.deg[v] && tie[u] < tie[v]))
                    {
                        v = u;
                    }
                }
                *work = work.checked_sub(b.k + (b.deg[v] + 1) * (b.w + 3))?;
                cost += b.elim(v);
                order.push(v);
            }
            return Some((cost, order));
        }
        b.init_def(work)?;
        for _ in 0..b.k {
            let mut v = usize::MAX;
            let mut val = f64::INFINITY;
            for u in 0..b.k {
                if !b.live[u] {
                    continue;
                }
                let d = b.deg[u] as f64;
                let f = b.def[u] as f64;
                let s = match mode {
                    1 => f,
                    2 => f / (d + 1.0),
                    3 => f / ((d + 1.0) * (d + 1.0)),
                    4 => f + 0.05 * d * d,
                    5 => f * (d + 1.0),
                    6 => (d + 1.0) * (d + 1.0) + 2.0 * f,
                    _ => f + 0.5 * d,
                };
                if s < val || (s == val && (v == usize::MAX || tie[u] < tie[v])) {
                    v = u;
                    val = s;
                }
            }
            cost += b.defelim(v, work)?;
            order.push(v);
        }
        Some((cost, order))
    }
    /// Exact best position for `v` in `order` by adjacent-pair cancellation.
    pub(super) fn insert(&self, order: &[usize], v: usize) -> (u64, Vec<usize>) {
        let mut rest: Vec<usize> = order.iter().copied().filter(|&u| u != v).collect();
        let mut b = self.clone();
        let mut delta = Vec::with_capacity(rest.len());
        let mut cost = 0i64;
        for &u in &rest {
            delta.push(if b.has(v, u) {
                ((b.deg[v] + 1) as i64).pow(2) - ((b.deg[u] + 1) as i64).pow(2)
            } else {
                0
            });
            cost += b.elim(u) as i64;
        }
        cost += b.elim(v) as i64;
        let mut best = cost;
        let mut pos = rest.len();
        for i in (0..rest.len()).rev() {
            cost += delta[i];
            if cost < best {
                best = cost;
                pos = i;
            }
        }
        rest.insert(pos, v);
        (best as u64, rest)
    }
    /// Repeated exact single-vertex reinsertion.
    pub(super) fn improve(
        &self,
        mut order: Vec<usize>,
        rounds: usize,
        seed: u64,
    ) -> (u64, Vec<usize>) {
        let mut cost = self.score(&order);
        let mut rng = Rng::new(seed);
        let mut idle = 0;
        for it in 0..rounds {
            let v = if it < self.k {
                order[it]
            } else {
                (rng.next() % self.k as u64) as usize
            };
            let (c, o) = self.insert(&order, v);
            if c < cost {
                cost = c;
                order = o;
                idle = 0;
            } else {
                idle += 1;
            }
            if idle >= self.k && it >= self.k {
                break;
            }
        }
        (cost, order)
    }
    /// Non-monotone restart: keep the first `prefix` pivots of `order`, then
    /// finish greedily by `mode` with mild fixed-seed noise. Exact cost.
    pub(super) fn restart(
        &self,
        order: &[usize],
        prefix: usize,
        mode: usize,
        seed: u64,
        work: &mut usize,
    ) -> Option<(u64, Vec<usize>)> {
        let mut b = self.clone();
        let mut rng = Rng::new(seed);
        let mut out = Vec::with_capacity(b.k);
        let mut cost = 0u64;
        for &v in &order[..prefix] {
            *work = work.checked_sub(b.k + (b.deg[v] + 1) * (b.w + 3))?;
            cost += b.elim(v);
            out.push(v);
        }
        if mode != 0 {
            b.init_def(work)?;
        }
        for _ in prefix..b.k {
            let mut v = usize::MAX;
            let mut val = f64::INFINITY;
            for u in 0..b.k {
                if !b.live[u] {
                    continue;
                }
                let d = b.deg[u] as f64;
                let f = b.def[u] as f64;
                let mut s = match mode {
                    0 => d,
                    1 => f,
                    2 => f / (d + 1.0),
                    3 => f / ((d + 1.0) * (d + 1.0)),
                    4 => f + 0.05 * d * d,
                    5 => f * (d + 1.0),
                    6 => f + 0.5 * d,
                    _ => d * d + 2.0 * f,
                };
                s *= 1.0 + (rng.next() % 1000) as f64 / 5000.0;
                if s < val {
                    val = s;
                    v = u;
                }
            }
            cost += if mode == 0 {
                *work = work.checked_sub(b.k + (b.deg[v] + 1) * (b.w + 3))?;
                b.elim(v)
            } else {
                b.defelim(v, work)?
            };
            out.push(v);
        }
        Some((cost, out))
    }
}

/// Lossless twin quotient of a `Dense` patch: vertices with identical open
/// neighborhoods (false twins, independent) or identical closed neighborhoods
/// (true twins, clique) form weighted blocks. Eliminable and exterior blocks
/// never mix. Block costs are exact for contiguous elimination.
#[derive(Clone)]
pub(super) struct Atoms {
    pub n: usize,
    pub k: usize,
    w: usize,
    pub members: Arc<Vec<Vec<usize>>>,
    wt: Arc<Vec<u64>>,
    nonunit: Arc<Vec<u64>>,
    words: Vec<(usize, u64)>,
    deg: Vec<u64>,
    def: Vec<i64>,
    clique: Vec<bool>,
    live: Vec<bool>,
    a: Vec<u64>,
}

/// A profile never eliminates a boundary block. Keep boundary incidences in
/// free rows, but omit the reverse rows and all boundary-only updates.
#[derive(Clone)]
struct ProfileState<'a> {
    k: usize,
    w: usize,
    wt: &'a [u64],
    nonunit: &'a [u64],
    deg: Vec<u64>,
    clique: Vec<bool>,
    a: Vec<u64>,
}
impl<'a> ProfileState<'a> {
    fn new(atoms: &'a Atoms) -> Self {
        Self {
            k: atoms.k,
            w: atoms.w,
            wt: &atoms.wt,
            nonunit: &atoms.nonunit,
            deg: atoms.deg[..atoms.k].to_vec(),
            clique: atoms.clique[..atoms.k].to_vec(),
            a: atoms.a[..atoms.k * atoms.w].to_vec(),
        }
    }
    fn has(&self, u: usize, v: usize) -> bool {
        self.a[u * self.w + v / 64] & (1 << (v % 64)) != 0
    }
    fn cost(&self, v: usize) -> u64 {
        let d = self.deg[v];
        let z = self.wt[v];
        if self.clique[v] {
            sum_squares(d + z) - sum_squares(d)
        } else {
            z * (d + 1).pow(2)
        }
    }
    fn elim(&mut self, v: usize) -> u64 {
        let cost = self.cost(v);
        let row = row_words(&self.a[v * self.w..(v + 1) * self.w]);
        for &(word, mut bits) in &row {
            if word * 64 >= self.k {
                break;
            }
            while bits != 0 {
                let u = word * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if u >= self.k {
                    break;
                }
                let mut added = 0;
                for &(j, neighbors) in &row {
                    let target = &mut self.a[u * self.w + j];
                    let mut fresh = neighbors & !*target;
                    added += fresh.count_ones() as u64;
                    fresh &= self.nonunit[j];
                    while fresh != 0 {
                        added += self.wt[j * 64 + fresh.trailing_zeros() as usize] - 1;
                        fresh &= fresh - 1;
                    }
                    *target |= neighbors;
                }
                self.a[u * self.w + u / 64] &= !(1 << (u % 64));
                self.a[u * self.w + v / 64] &= !(1 << (v % 64));
                self.deg[u] = self.deg[u] + added - self.wt[u] - self.wt[v];
                self.clique[u] = true;
            }
        }
        self.a[v * self.w..(v + 1) * self.w].fill(0);
        self.deg[v] = 0;
        cost
    }
}

impl Atoms {
    /// Exact interleavings of the incumbent's even and odd position tracks.
    /// Each row advances one shared prefix; only one graph is copied per row.
    pub(super) fn interleave(&self, order: &[usize]) -> (u64, Vec<usize>) {
        let x: Vec<_> = order.iter().step_by(2).copied().collect();
        let y: Vec<_> = order.iter().skip(1).step_by(2).copied().collect();
        let stride = y.len() + 1;
        let mut dp = vec![u64::MAX; (x.len() + 1) * stride];
        let mut back = vec![0u8; dp.len()];
        dp[0] = 0;
        let mut row = ProfileState::new(self);
        for i in 0..=x.len() {
            let mut at = row.clone();
            for j in 0..=y.len() {
                let cost = dp[i * stride + j];
                if i < x.len() {
                    let next = (i + 1) * stride + j;
                    let candidate = cost + at.cost(x[i]);
                    if candidate < dp[next] {
                        dp[next] = candidate;
                        back[next] = 1;
                    }
                }
                if j < y.len() {
                    let next = i * stride + j + 1;
                    let candidate = cost + at.cost(y[j]);
                    if candidate < dp[next] {
                        dp[next] = candidate;
                        back[next] = 2;
                    }
                    at.elim(y[j]);
                }
            }
            if i < x.len() {
                row.elim(x[i]);
            }
        }
        let (mut i, mut j) = (x.len(), y.len());
        let mut out = Vec::with_capacity(order.len());
        while i != 0 || j != 0 {
            if back[i * stride + j] == 1 {
                i -= 1;
                out.push(x[i]);
            } else {
                j -= 1;
                out.push(y[j]);
            }
        }
        out.reverse();
        (*dp.last().unwrap(), out)
    }
    /// Exact conditional block ordering by connected subsets and boundary
    /// signatures. No interface graph is copied per subset.
    pub(super) fn exact(&self) -> Option<(u64, Vec<usize>)> {
        if self.k > 19 {
            return None;
        }
        let count = 1usize << self.k;
        let full = count - 1;
        let mut adjacency = vec![0usize; self.k];
        let mut boundary = vec![0u64; count];
        let mut total_boundary = 0;
        for v in 0..self.n {
            let mut mask = 0usize;
            for u in self.nb(v) {
                if u < self.k {
                    mask |= 1 << u;
                }
            }
            if v < self.k {
                adjacency[v] = mask;
            } else {
                boundary[mask] += self.wt[v];
                total_boundary += self.wt[v];
            }
        }
        for bit in 0..self.k {
            for mask in 0..count {
                if mask & (1 << bit) != 0 {
                    boundary[mask] += boundary[mask ^ (1 << bit)];
                }
            }
        }
        let mut union = vec![0usize; count];
        let mut weight = vec![0u64; count];
        let mut component = vec![0usize; count];
        let mut dp = vec![0u64; count];
        let mut last = vec![0usize; count];
        for s in 1..count {
            let bit = s & s.wrapping_neg();
            let v = bit.trailing_zeros() as usize;
            union[s] = union[s ^ bit] | adjacency[v];
            weight[s] = weight[s ^ bit] + self.wt[v];
        }
        for s in 1..count {
            let bit = s & s.wrapping_neg();
            let v = bit.trailing_zeros() as usize;
            let mut connected = bit;
            loop {
                let next = connected | (union[connected] & s);
                if next == connected {
                    break;
                }
                connected = next;
            }
            component[s] = connected;
            if connected != s {
                dp[s] = dp[connected] + dp[s ^ connected];
                continue;
            }
            let d = total_boundary - boundary[full ^ s] + weight[union[s] & !s & full];
            if s == bit {
                dp[s] = if self.clique[v] {
                    sum_squares(d + self.wt[v]) - sum_squares(d)
                } else {
                    self.wt[v] * (d + 1).pow(2)
                };
                last[s] = v;
                continue;
            }
            dp[s] = u64::MAX;
            let mut remaining = s;
            while remaining != 0 {
                let v = remaining.trailing_zeros() as usize;
                remaining &= remaining - 1;
                let cost = dp[s ^ (1 << v)] + sum_squares(d + self.wt[v]) - sum_squares(d);
                if cost < dp[s] {
                    dp[s] = cost;
                    last[s] = v;
                }
            }
        }
        fn emit(s: usize, component: &[usize], last: &[usize], out: &mut Vec<usize>) {
            if s == 0 {
                return;
            }
            if component[s] != s {
                emit(component[s], component, last, out);
                emit(s ^ component[s], component, last, out);
            } else {
                emit(s ^ (1 << last[s]), component, last, out);
                out.push(last[s]);
            }
        }
        let mut order = Vec::with_capacity(self.k);
        emit(full, &component, &last, &mut order);
        Some((dp[full], order))
    }

    /// Jointly move a few blocks throughout the incumbent's remaining order.
    pub(super) fn joint(&self, order: &[usize]) -> Option<(u64, Vec<usize>)> {
        let mut ids = order.to_vec();
        ids.extend(self.k..self.n);
        let mut inverse = vec![0usize; self.n];
        for (i, &v) in ids.iter().enumerate() {
            inverse[v] = i;
        }
        let mut graph = vec![0u64; self.n * self.w];
        for (i, &v) in ids.iter().enumerate() {
            for u in self.nb(v) {
                graph[i * self.w + inverse[u] / 64] |= 1 << (inverse[u] % 64);
            }
        }
        let weights: Vec<_> = ids.iter().map(|&v| self.wt[v]).collect();
        let cliques: Vec<_> = ids.iter().map(|&v| self.clique[v]).collect();
        let mut replay = ProfileState::new(self);
        let mut ranked: Vec<_> = order
            .iter()
            .enumerate()
            .map(|(i, &v)| (std::cmp::Reverse(replay.elim(v)), i))
            .collect();
        ranked.sort_unstable();
        let mut width = if self.k <= 12 {
            self.k
        } else if self.k <= 180 {
            8
        } else {
            6
        };
        while width > 4 && (self.k - width + 1) * (1usize << width) * (self.n + width) > 2_000_000 {
            width -= 1;
        }
        let selected: Vec<_> = ranked.iter().take(width).map(|&(_, i)| i).collect();
        let (p, cost) =
            super::joint::schedule_weighted(&graph, self.n, self.k, &selected, &weights, &cliques)?;
        Some((cost, p.iter().map(|&v| ids[v]).collect()))
    }

    pub(super) fn new(g: &Dense) -> Atoms {
        let gw = g.w;
        let mut used = vec![false; g.n];
        let mut id = vec![usize::MAX; g.n];
        let mut members: Vec<Vec<usize>> = Vec::new();
        let mut clique = Vec::new();
        let mut k = 0;
        for part in 0..2 {
            let (lo, hi) = if part == 0 { (0, g.k) } else { (g.k, g.n) };
            for closed in [false, true] {
                let compare = |u: usize, v: usize| {
                    if !closed {
                        return g.a[u * gw..(u + 1) * gw].cmp(&g.a[v * gw..(v + 1) * gw]);
                    }
                    for word in 0..gw {
                        let a = g.a[u * gw + word] | if word == u / 64 { 1 << (u % 64) } else { 0 };
                        let b = g.a[v * gw + word] | if word == v / 64 { 1 << (v % 64) } else { 0 };
                        let cmp = a.cmp(&b);
                        if cmp != std::cmp::Ordering::Equal {
                            return cmp;
                        }
                    }
                    std::cmp::Ordering::Equal
                };
                let mut vertices: Vec<_> = (lo..hi).filter(|&v| !used[v]).collect();
                vertices.sort_unstable_by(|&u, &v| compare(u, v).then(u.cmp(&v)));
                let mut start = 0;
                while start < vertices.len() {
                    let mut end = start + 1;
                    while end < vertices.len() && compare(vertices[start], vertices[end]).is_eq() {
                        end += 1;
                    }
                    if closed || end - start > 1 {
                        let vs = vertices[start..end].to_vec();
                        let j = members.len();
                        for &v in &vs {
                            used[v] = true;
                            id[v] = j;
                        }
                        members.push(vs);
                        clique.push(closed);
                    }
                    start = end;
                }
            }
            if part == 0 {
                k = members.len();
            }
        }
        let n = members.len();
        let w = n.div_ceil(64);
        let mut a = vec![0u64; n * w];
        let mut wt = vec![0u64; n];
        for j in 0..n {
            wt[j] = members[j].len() as u64;
            let v = members[j][0];
            bits(&g.a[v * gw..(v + 1) * gw], g.n, |u| {
                if id[u] != j {
                    a[j * w + id[u] / 64] |= 1u64 << (id[u] % 64);
                }
            });
        }
        let mut nonunit = vec![0; w];
        for (v, &weight) in wt.iter().enumerate() {
            if weight != 1 {
                nonunit[v / 64] |= 1 << (v % 64);
            }
        }
        let mut atoms = Atoms {
            n,
            k,
            w,
            members: Arc::new(members),
            wt: Arc::new(wt),
            nonunit: Arc::new(nonunit),
            words: Vec::new(),
            deg: vec![0; n],
            def: vec![0; n],
            clique,
            live: vec![true; n],
            a,
        };
        for v in 0..n {
            atoms.deg[v] = (0..w).map(|j| atoms.weight(atoms.a[v * w + j], j)).sum();
        }
        atoms
    }
    fn nb(&self, v: usize) -> Vec<usize> {
        let mut r = Vec::new();
        bits(&self.a[v * self.w..(v + 1) * self.w], self.n, |u| r.push(u));
        r
    }
    fn weight(&self, mut bits: u64, word: usize) -> u64 {
        let mut sum = bits.count_ones() as u64;
        bits &= self.nonunit[word];
        while bits != 0 {
            sum += self.wt[word * 64 + bits.trailing_zeros() as usize] - 1;
            bits &= bits - 1;
        }
        sum
    }
    /// Exact cost of eliminating the whole block `v` now.
    fn cost(&self, v: usize) -> u64 {
        let d = self.deg[v];
        let z = self.wt[v];
        if self.clique[v] {
            sum_squares(d + z) - sum_squares(d)
        } else {
            z * (d + 1) * (d + 1)
        }
    }
    fn fill(&self, v: usize) -> i64 {
        let ns = self.nb(v);
        let mut f = 0i64;
        for &u in &ns {
            if !self.clique[u] {
                f += (self.wt[u] * (self.wt[u] - 1) / 2) as i64;
            }
            // Visit only missing neighbors above u. Dense exterior cliques
            // contribute no such bits, avoiding a quadratic pair scan.
            for j in u / 64..self.w {
                let mut missing = self.a[v * self.w + j] & !self.a[u * self.w + j];
                if j == u / 64 {
                    missing &= if u % 64 == 63 {
                        0
                    } else {
                        u64::MAX << (u % 64 + 1)
                    };
                }
                f += (self.wt[u] * self.weight(missing, j)) as i64;
            }
        }
        f
    }
    fn elim(&mut self, v: usize) -> u64 {
        let w = self.w;
        let c = self.cost(v);
        let mut row = std::mem::take(&mut self.words);
        row.clear();
        row.extend(
            self.a[v * w..(v + 1) * w]
                .iter()
                .copied()
                .enumerate()
                .filter(|&(_, bits)| bits != 0),
        );
        for &(word, mut neighbors) in &row {
            while neighbors != 0 {
                let u = word * 64 + neighbors.trailing_zeros() as usize;
                neighbors &= neighbors - 1;
                let mut added = 0u64;
                for &(j, bits) in &row {
                    added += self.weight(bits & !self.a[u * w + j], j);
                    self.a[u * w + j] |= bits;
                }
                self.a[u * w + u / 64] &= !(1u64 << (u % 64));
                self.a[u * w + v / 64] &= !(1u64 << (v % 64));
                self.clique[u] = true;
                self.deg[u] = self.deg[u] + added - self.wt[u] - self.wt[v];
            }
        }
        self.words = row;
        self.a[v * w..(v + 1) * w].fill(0);
        self.live[v] = false;
        self.deg[v] = 0;
        c
    }
    fn init_def(&mut self) {
        for v in 0..self.k {
            self.def[v] = self.fill(v);
        }
    }
    fn defedge(&mut self, u: usize, v: usize) {
        let (w, k) = (self.w, self.k);
        let mut common = 0u64;
        for j in 0..w {
            let mut z = self.a[u * w + j] & self.a[v * w + j];
            common += self.weight(z, j);
            if j * 64 >= k {
                continue;
            }
            if (j + 1) * 64 > k {
                z &= (1u64 << (k % 64)) - 1;
            }
            while z != 0 {
                let x = j * 64 + z.trailing_zeros() as usize;
                z &= z - 1;
                self.def[x] -= (self.wt[u] * self.wt[v]) as i64;
            }
        }
        if u < k {
            self.def[u] += (self.wt[v] * (self.deg[u] - common)) as i64;
        }
        if v < k {
            self.def[v] += (self.wt[u] * (self.deg[v] - common)) as i64;
        }
        self.a[u * w + v / 64] |= 1u64 << (v % 64);
        self.a[v * w + u / 64] |= 1u64 << (u % 64);
        self.deg[u] += self.wt[v];
        self.deg[v] += self.wt[u];
    }
    fn defelim(&mut self, v: usize) -> u64 {
        let (w, k) = (self.w, self.k);
        let co = self.cost(v);
        let dv = self.deg[v];
        let wv = self.wt[v];
        let ns = self.nb(v);
        for &u in &ns {
            if !self.clique[u] {
                let loss = (self.wt[u] * (self.wt[u] - 1) / 2) as i64;
                for x in self.nb(u) {
                    if x < k {
                        self.def[x] -= loss;
                    }
                }
                self.clique[u] = true;
            }
        }
        for &u in &ns {
            for j in u / 64..w {
                let mut z = self.a[v * w + j] & !self.a[u * w + j];
                if j == u / 64 {
                    z &= if u % 64 == 63 {
                        0
                    } else {
                        !0u64 << (u % 64 + 1)
                    };
                }
                while z != 0 {
                    let t = j * 64 + z.trailing_zeros() as usize;
                    z &= z - 1;
                    if t < self.n {
                        self.defedge(u, t);
                    }
                }
            }
        }
        for &u in &ns {
            let extra = self.deg[u] as i64 - wv as i64 - dv as i64 + self.wt[u] as i64;
            if u < k {
                self.def[u] -= wv as i64 * extra
                    + if !self.clique[v] {
                        (wv * (wv - 1) / 2) as i64
                    } else {
                        0
                    };
            }
            self.a[u * w + v / 64] &= !(1u64 << (v % 64));
            self.deg[u] -= wv;
        }
        self.a[v * w..(v + 1) * w].fill(0);
        self.live[v] = false;
        self.deg[v] = 0;
        co
    }
    /// Block order by first occurrence in a raw eliminable order. By the
    /// simplicial-batching argument its expanded cost never exceeds the raw cost.
    pub(super) fn project(&self, raw: &[usize]) -> Vec<usize> {
        let raw_k = self.members[..self.k]
            .iter()
            .flatten()
            .copied()
            .max()
            .map_or(0, |m| m + 1);
        let mut ids = vec![usize::MAX; raw_k];
        for j in 0..self.k {
            for &v in &self.members[j] {
                ids[v] = j;
            }
        }
        let mut seen = vec![false; self.k];
        let mut o = Vec::with_capacity(self.k);
        for &v in raw {
            let j = ids[v];
            if !seen[j] {
                seen[j] = true;
                o.push(j);
            }
        }
        o
    }
    pub(super) fn expand(&self, order: &[usize]) -> Vec<usize> {
        order
            .iter()
            .flat_map(|&v| self.members[v].iter().copied())
            .collect()
    }
    pub(super) fn score(&self, order: &[usize]) -> u64 {
        let mut b = ProfileState::new(self);
        order.iter().map(|&v| b.elim(v)).sum()
    }
    /// Weighted greedy block elimination; eight objectives.
    pub(super) fn greedy(&self, mode: usize, seed: u64) -> (u64, Vec<usize>) {
        let mut b = self.clone();
        // Minimum degree never reads deficiency. Updating every affected
        // triangle for that objective adds work without changing any choice.
        if mode != 0 {
            b.init_def();
        }
        let mut rng = Rng::new(seed);
        let tie: Vec<u64> = (0..b.k).map(|_| rng.next()).collect();
        let mut cost = 0u64;
        let mut order = Vec::with_capacity(b.k);
        for _ in 0..b.k {
            let mut v = usize::MAX;
            let mut best = f64::INFINITY;
            for u in 0..b.k {
                if !b.live[u] {
                    continue;
                }
                let d = b.deg[u] as f64;
                let f = b.def[u] as f64;
                let z = b.wt[u] as f64;
                let s = match mode {
                    0 => d + if b.clique[u] { z } else { 1.0 },
                    1 => f,
                    2 => f / z,
                    3 => f / (d + z),
                    4 => f / ((d + z) * (d + z)),
                    5 => f * (d + 1.0) / z,
                    6 => (b.cost(u) as f64 + (2.0 * d + 5.0) * f) / z,
                    _ => f + 0.05 * (d + z) * (d + z),
                };
                if s < best || (s == best && (v == usize::MAX || tie[u] < tie[v])) {
                    best = s;
                    v = u;
                }
            }
            cost += if mode == 0 { b.elim(v) } else { b.defelim(v) };
            order.push(v);
        }
        (cost, order)
    }
    /// Exact cost at every insertion position, from one deferred replay.
    pub(super) fn profile(&self, order: &[usize], v: usize) -> Vec<u64> {
        let rest: Vec<usize> = order.iter().copied().filter(|&u| u != v).collect();
        let mut b = ProfileState::new(self);
        let mut cost = 0i64;
        let mut diff = Vec::with_capacity(rest.len());
        for &u in &rest {
            let mut delta = 0i64;
            if b.has(u, v) {
                let mut du = 0u64;
                for j in 0..b.w {
                    let mut z = b.a[u * b.w + j] | b.a[v * b.w + j];
                    while z != 0 {
                        let x = j * 64 + z.trailing_zeros() as usize;
                        z &= z - 1;
                        if x != u && x != v {
                            du += b.wt[x];
                        }
                    }
                }
                delta = b.cost(v) as i64 - b.cost(u) as i64 + sum_squares(du + b.wt[u]) as i64
                    - sum_squares(du + b.wt[v]) as i64;
            }
            diff.push(delta);
            cost += b.elim(u) as i64;
        }
        cost += b.elim(v) as i64;
        let mut values = vec![0; self.k];
        values[rest.len()] = cost as u64;
        for i in (0..rest.len()).rev() {
            cost += diff[i];
            values[i] = cost as u64;
        }
        values
    }

    /// Exact best position; ties preserve the old rightmost-minimum policy.
    pub(super) fn insert(&self, order: &[usize], v: usize) -> (u64, Vec<usize>) {
        let values = self.profile(order, v);
        let pos = (0..values.len()).rev().min_by_key(|&i| values[i]).unwrap();
        let mut rest: Vec<_> = order.iter().copied().filter(|&u| u != v).collect();
        rest.insert(pos, v);
        (values[pos], rest)
    }

    /// Reinsertion sweep, most expensive blocks first, then fixed-seed picks.
    pub(super) fn improve(
        &self,
        mut order: Vec<usize>,
        rounds: usize,
        seed: u64,
    ) -> (u64, Vec<usize>) {
        let mut cost = self.score(&order);
        let mut rng = Rng::new(seed);
        let mut ranked: Vec<(u64, usize)> = {
            let mut replay = ProfileState::new(self);
            order.iter().map(|&v| (replay.elim(v), v)).collect()
        };
        ranked.sort_by(|a, b| b.cmp(a));
        for it in 0..rounds {
            let v = if it < self.k {
                ranked[it].1
            } else {
                (rng.next() % self.k as u64) as usize
            };
            let (c, o) = self.insert(&order, v);
            if c < cost {
                cost = c;
                order = o;
            }
        }
        (cost, order)
    }
}
