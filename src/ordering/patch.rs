//! Exact kernels on a compressed separator patch: a dense bitset graph whose
//! first `k` vertices are eliminable and whose remaining vertices form the
//! frozen exterior, plus its lossless true/false-twin quotient with integer
//! block weights. Every cost is the exact `sum (degree + 1)^2` of the expanded
//! order; the quotient identities are asserted in tests against literal
//! elimination.

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
    def: Vec<i64>,
    pub live: Vec<bool>,
}

impl Dense {
    pub(super) fn from_bits(a: Vec<u64>, n: usize, k: usize) -> Dense {
        let w = n.div_ceil(64);
        debug_assert_eq!(a.len(), n * w);
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
        debug_assert!(self.live[v]);
        let w = self.w;
        let ns = self.nb(v);
        let row: Vec<u64> = self.a[v * w..(v + 1) * w].to_vec();
        let q = (ns.len() + 1) as u64;
        for &u in &ns {
            let mut dd = 0usize;
            for j in 0..w {
                self.a[u * w + j] |= row[j];
                dd += self.a[u * w + j].count_ones() as usize;
            }
            self.a[u * w + u / 64] &= !(1u64 << (u % 64));
            self.a[u * w + v / 64] &= !(1u64 << (v % 64));
            self.deg[u] = dd - 2;
        }
        self.a[v * w..(v + 1) * w].fill(0);
        self.live[v] = false;
        self.deg[v] = 0;
        q * q
    }
    pub(super) fn score(&self, order: &[usize]) -> u64 {
        let mut b = self.clone();
        order.iter().map(|&u| b.elim(u)).sum()
    }
    /// Preserve a prefix, force simplicial vertices, and rediscover twins
    /// before pooling clique bags in the remaining interface.
    pub(super) fn bag_suffix(&self, initial: &[usize], seed: u64, prefix_grid: bool) -> (u64, Vec<usize>) {
        let mut best = (self.score(initial), initial.to_vec());
        let mut cuts = vec![0, self.k / 4, self.k / 2,
            self.k.saturating_sub(200), self.k.saturating_sub(128), self.k.saturating_sub(64)];
        cuts.sort_unstable(); cuts.dedup();
        let mut used = 0;
        let mut rng = Rng::new(seed);
        for cut in cuts {
            if used == 3 || cut >= self.k { break; }
            let mut state = self.clone();
            let mut prefix = best.1[..cut].to_vec();
            let mut fixed: u64 = prefix.iter().map(|&v| state.elim(v)).sum();
            state.init_def();
            loop {
                let mut changed = false;
                for &v in &best.1[cut..] {
                    if state.live[v] && state.def[v] == 0 {
                        fixed += state.defelim(v);
                        prefix.push(v);
                        changed = true;
                    }
                }
                if !changed { break; }
            }
            let mut ids: Vec<_> = best.1.iter().copied().filter(|&v| state.live[v]).collect();
            let free = ids.len();
            if free == 0 {
                if prefix_grid && fixed < best.0 {
                    assert_eq!(self.score(&prefix), fixed);
                    best = (fixed, prefix);
                }
                continue;
            }
            ids.extend(self.k..self.n);
            // Extended search limits the quotient, not its expanded boundary.
            if !prefix_grid && ids.len() > 900 { continue; }
            let mut inverse = vec![usize::MAX; self.n];
            for (i, &v) in ids.iter().enumerate() { inverse[v] = i; }
            let n = ids.len();
            let w = n.div_ceil(64);
            let mut graph = vec![0; n*w];
            for (i, &v) in ids.iter().enumerate() {
                for u in state.nb(v) { graph[i*w+inverse[u]/64] |= 1 << (inverse[u]%64); }
            }
            let atoms = Atoms::new(&Dense::from_bits(graph, n, free));
            if atoms.k > 240 || atoms.n > 900 { continue; }
            used += 1;
            let mut sources = vec![atoms.project(&(0..free).collect::<Vec<_>>())];
            for mode in 0..8 { sources.push(atoms.greedy(mode, rng.next()).1); }
            let (cost, order) = atoms.bag_pool(&sources, prefix_grid);
            prefix.extend(atoms.expand(&order).iter().map(|&v| ids[v]));
            assert_eq!(self.score(&prefix), fixed + cost);
            if fixed + cost < best.0 { best = (fixed + cost, prefix); }
        }
        best
    }
    /// Recompress the residual after an incumbent prefix, retaining its exact
    /// incurred cost. Newly identical free/frozen neighborhoods are kept apart.
    pub(super) fn residual_order(
        &self,
        order: &[usize],
        tail: usize,
        budget: &mut usize,
    ) -> Option<(u64, Vec<usize>)> {
        let initial = Atoms::new(self);
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
    fn init_def(&mut self) {
        let w = self.w;
        for v in 0..self.k {
            let mut z = 0i64;
            for u in self.nb(v) {
                for j in 0..w {
                    z += (self.a[v * w + j] & !self.a[u * w + j]).count_ones() as i64;
                }
            }
            self.def[v] = (z - self.deg[v] as i64) / 2;
        }
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
    fn defelim(&mut self, v: usize) -> u64 {
        // A large fill event can touch each deficiency once per new edge.
        // Recompute all deficiencies when that exceeds a full bitset recount.
        let incremental = self.def[v].max(0) as usize * (self.k + self.w);
        if incremental > self.k * self.n * self.w {
            let cost = self.elim(v);
            self.init_def();
            return cost;
        }
        self.defelim_incremental(v)
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
    pub(super) fn greedy(&self, mode: usize, seed: u64) -> (u64, Vec<usize>) {
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
                cost += b.elim(v);
                order.push(v);
            }
            return (cost, order);
        }
        b.init_def();
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
            cost += b.defelim(v);
            order.push(v);
        }
        (cost, order)
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
    ) -> (u64, Vec<usize>) {
        let mut b = self.clone();
        let mut rng = Rng::new(seed);
        let mut out = Vec::with_capacity(b.k);
        let mut cost = 0u64;
        for &v in &order[..prefix] {
            cost += b.elim(v);
            out.push(v);
        }
        b.init_def();
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
            cost += b.defelim(v);
            out.push(v);
        }
        (cost, out)
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
    pub members: Vec<Vec<usize>>,
    wt: Vec<u64>,
    deg: Vec<u64>,
    def: Vec<i64>,
    clique: Vec<bool>,
    live: Vec<bool>,
    a: Vec<u64>,
}

/// A profile never eliminates a boundary block. Keep boundary incidences in
/// free rows, but omit the reverse rows and all boundary-only updates.
struct ProfileState<'a> {
    k: usize,
    w: usize,
    wt: &'a [u64],
    deg: Vec<u64>,
    clique: Vec<bool>,
    a: Vec<u64>,
}
impl<'a> ProfileState<'a> {
    fn new(atoms: &'a Atoms) -> Self {
        Self {
            k: atoms.k, w: atoms.w, wt: &atoms.wt,
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
        if self.clique[v] { sum_squares(d + z) - sum_squares(d) }
        else { z * (d + 1).pow(2) }
    }
    fn elim(&mut self, v: usize) -> u64 {
        let cost = self.cost(v);
        let row = self.a[v * self.w..(v + 1) * self.w].to_vec();
        for word in 0..self.k.div_ceil(64) {
            let mut bits = row[word];
            while bits != 0 {
                let u = word * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if u >= self.k { break; }
                let mut added = 0;
                for (j, &neighbors) in row.iter().enumerate() {
                    let target = &mut self.a[u * self.w + j];
                    let mut fresh = neighbors & !*target;
                    while fresh != 0 {
                        added += self.wt[j * 64 + fresh.trailing_zeros() as usize];
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
    /// Source-head DP: each state is a union of two donor prefixes. Common
    /// prefix sets split the grid into independent smaller searches.
    fn prefix_mix(&self, x: &[usize], y: &[usize]) -> (u64, Vec<usize>) {
        let mut delta = vec![0i32; self.k];
        let mut mismatch = 0;
        let mut cuts = vec![0];
        for j in 0..self.k {
            for (v, d) in [(x[j], 1), (y[j], -1)] {
                mismatch -= usize::from(delta[v] != 0);
                delta[v] += d;
                mismatch += usize::from(delta[v] != 0);
            }
            if mismatch == 0 { cuts.push(j + 1); }
        }
        let mut row = self.clone();
        let mut output = Vec::with_capacity(self.k);
        let mut total = 0;
        for segment in cuts.windows(2) {
            let (lo, hi) = (segment[0], segment[1]);
            let t = hi - lo;
            let w = t + 1;
            let mut dp = vec![u64::MAX; w*w];
            let mut back = vec![0u8; w*w];
            dp[0] = 0;
            for i in 0..=t {
                let mut state = row.clone();
                for j in 0..=t {
                    let at = i*w+j;
                    if i < t {
                        let v = x[lo+i];
                        let live = state.live[v];
                        let cost = dp[at] + if live { state.cost(v) } else { 0 };
                        if cost < dp[at+w] { dp[at+w] = cost; back[at+w] = 1 | if live { 4 } else { 0 }; }
                    }
                    if j < t {
                        let v = y[lo+j];
                        let live = state.live[v];
                        let cost = dp[at] + if live { state.cost(v) } else { 0 };
                        if cost < dp[at+1] { dp[at+1] = cost; back[at+1] = 2 | if live { 4 } else { 0 }; }
                        if live { state.elim(v); }
                    }
                }
                if i < t { row.elim(x[lo+i]); }
            }
            total += dp[w*w-1];
            let (mut i, mut j) = (t, t);
            let mut part = Vec::with_capacity(t);
            while i != 0 || j != 0 {
                let step = back[i*w+j];
                let v = if step & 3 == 1 { i -= 1; x[lo+i] } else { j -= 1; y[lo+j] };
                if step & 4 != 0 { part.push(v); }
            }
            output.extend(part.into_iter().rev());
        }
        assert!(super::is_bijection(&output, self.k));
        assert_eq!(total, self.score(&output));
        assert!(total <= self.score(x).min(self.score(y)));
        (total, output)
    }
    /// Exact interleavings of the incumbent's even and odd position tracks.
    /// Each row advances one shared prefix; only one graph is copied per row.
    pub(super) fn interleave(&self, order: &[usize]) -> (u64, Vec<usize>) {
        let x: Vec<_> = order.iter().step_by(2).copied().collect();
        let y: Vec<_> = order.iter().skip(1).step_by(2).copied().collect();
        let stride = y.len() + 1;
        let mut dp = vec![u64::MAX; (x.len() + 1) * stride];
        let mut back = vec![0u8; dp.len()];
        dp[0] = 0;
        let mut row = self.clone();
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
        self.joint_group(order, true)
    }

    #[cfg(test)]
    pub(super) fn joint_by_charge(&self, order: &[usize]) -> Option<(u64, Vec<usize>)> {
        self.joint_group(order, false)
    }

    fn joint_group(&self, order: &[usize], wide: bool) -> Option<(u64, Vec<usize>)> {
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
        let mut replay = self.clone();
        let mut ranked: Vec<_> = order
            .iter()
            .enumerate()
            .map(|(i, &v)| (std::cmp::Reverse(replay.elim(v)), i))
            .collect();
        ranked.sort_unstable();
        let mut width = if wide { if self.k <= 12 { self.k } else if self.k <= 180 { 8 } else { 6 } } else { 4.min(self.k) };
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
            let mut false_groups: std::collections::BTreeMap<Vec<u64>, Vec<usize>> =
                std::collections::BTreeMap::new();
            for v in lo..hi {
                false_groups
                    .entry(g.a[v * gw..(v + 1) * gw].to_vec())
                    .or_default()
                    .push(v);
            }
            for (_, vs) in false_groups {
                if vs.len() > 1 {
                    let j = members.len();
                    for &v in &vs {
                        used[v] = true;
                        id[v] = j;
                    }
                    members.push(vs);
                    clique.push(false);
                }
            }
            let mut true_groups: std::collections::BTreeMap<Vec<u64>, Vec<usize>> =
                std::collections::BTreeMap::new();
            for v in lo..hi {
                if used[v] {
                    continue;
                }
                let mut key = g.a[v * gw..(v + 1) * gw].to_vec();
                key[v / 64] |= 1u64 << (v % 64);
                true_groups.entry(key).or_default().push(v);
            }
            for (_, vs) in true_groups {
                let j = members.len();
                for &v in &vs {
                    used[v] = true;
                    id[v] = j;
                }
                members.push(vs);
                clique.push(true);
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
            for &v in &members[j] {
                for u in g.nb(v) {
                    if id[u] != j {
                        a[j * w + id[u] / 64] |= 1u64 << (id[u] % 64);
                    }
                }
            }
        }
        let mut atoms = Atoms {
            n,
            k,
            w,
            members,
            wt,
            deg: vec![0; n],
            def: vec![0; n],
            clique,
            live: vec![true; n],
            a,
        };
        for v in 0..n {
            atoms.deg[v] = atoms.nb(v).iter().map(|&u| atoms.wt[u]).sum();
        }
        atoms
    }
    #[cfg(test)]
    fn has(&self, u: usize, v: usize) -> bool {
        (self.a[u * self.w + v / 64] >> (v % 64)) & 1 == 1
    }
    fn nb(&self, v: usize) -> Vec<usize> {
        let mut r = Vec::new();
        bits(&self.a[v * self.w..(v + 1) * self.w], self.n, |u| r.push(u));
        r
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
                    missing &= if u % 64 == 63 { 0 } else { u64::MAX << (u % 64 + 1) };
                }
                while missing != 0 {
                    let x = j * 64 + missing.trailing_zeros() as usize;
                    missing &= missing - 1;
                    f += (self.wt[u] * self.wt[x]) as i64;
                }
            }
        }
        f
    }
    fn elim(&mut self, v: usize) -> u64 {
        debug_assert!(self.live[v]);
        let w = self.w;
        let c = self.cost(v);
        let ns = self.nb(v);
        let row: Vec<u64> = self.a[v * w..(v + 1) * w].to_vec();
        for &u in &ns {
            let mut added = 0u64;
            for j in 0..w {
                let mut z = row[j] & !self.a[u * w + j];
                while z != 0 {
                    let x = j * 64 + z.trailing_zeros() as usize;
                    z &= z - 1;
                    added += self.wt[x];
                }
                self.a[u * w + j] |= row[j];
            }
            self.a[u * w + u / 64] &= !(1u64 << (u % 64));
            self.a[u * w + v / 64] &= !(1u64 << (v % 64));
            self.clique[u] = true;
            self.deg[u] = self.deg[u] + added - self.wt[u] - self.wt[v];
        }
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
            while z != 0 {
                let x = j * 64 + z.trailing_zeros() as usize;
                z &= z - 1;
                common += self.wt[x];
                if x < k {
                    self.def[x] -= (self.wt[u] * self.wt[v]) as i64;
                }
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
                debug_assert!(self.def[u] >= 0, "negative weighted deficiency");
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
        let mut b = self.clone();
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

    pub(super) fn pool(&self, sources: &[Vec<usize>]) -> (u64, Vec<usize>) {
        super::decomposition::recombine(
            &self.a, self.n, self.k, &self.wt, &self.clique, sources,
        )
    }

    pub(super) fn bag_pool(&self, sources: &[Vec<usize>], prefix_grid: bool) -> (u64, Vec<usize>) {
        let mut best = (self.score(&sources[0]), sources[0].clone());
        if self.k > 240 || self.n > 900 { return best; }
        let mut crossed = Vec::new();
        if prefix_grid && self.k <= 128 {
            for source in sources.iter().skip(1).take(3) {
                crossed.push(self.prefix_mix(&sources[0], source).1);
            }
        }
        let mut bags = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for source in sources.iter().chain(&crossed) {
            let mut state = self.clone();
            for &v in source {
                let mut bag = state.a[v*self.w..(v+1)*self.w].to_vec();
                bag[v/64] |= 1 << (v%64);
                if seen.insert(bag.clone()) { bags.push(bag); }
                state.elim(v);
            }
            let cost = self.score(source);
            if cost < best.0 { best = (cost, source.clone()); }
        }
        let (bound, order) = super::bags::recombine(&self.a, self.n, self.k, &self.wt, &self.clique, &bags, &sources[0]);
        assert!(super::is_bijection(&order, self.k));
        let cost = self.score(&order);
        assert!(cost <= bound, "clique-bag cost bound");
        if cost < best.0 { best = (cost, order); }
        best
    }

    /// Complementary exact-profile trajectories. Keep both the best state and
    /// the terminal state: a losing whole order may supply a useful region.
    pub(super) fn profile_sources(
        &self, mut order: Vec<usize>, steps: usize, seed: u64, hot: bool,
    ) -> [Vec<usize>; 2] {
        let mut rng = Rng::new(seed);
        let mut best = self.score(&order);
        let mut best_order = order.clone();
        let initial_temperature = (best as f64 / (100 * self.k.max(1)) as f64).max(1.0);
        for step in 0..steps {
            let v = (rng.next() % self.k as u64) as usize;
            let values = self.profile(&order, v);
            let low = *values.iter().min().unwrap();
            let position = if hot {
                let temperature = initial_temperature
                    * 0.001f64.powf(step as f64 / steps.max(1) as f64);
                let weights: Vec<_> = values.iter()
                    .map(|&c| (-((c - low) as f64) / temperature).exp()).collect();
                let mut draw = (rng.next() as f64 / (u64::MAX as f64 + 1.0))
                    * weights.iter().sum::<f64>();
                let mut position = values.len() - 1;
                for (i, &weight) in weights.iter().enumerate() {
                    draw -= weight;
                    if draw < 0.0 { position = i; break; }
                }
                position
            } else {
                let choices: Vec<_> = (0..values.len()).filter(|&i| values[i] == low).collect();
                choices[(rng.next() % choices.len() as u64) as usize]
            };
            order.retain(|&u| u != v);
            order.insert(position, v);
            if values[position] < best {
                best = values[position];
                best_order.clone_from(&order);
            }
        }
        [best_order, order]
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
            let mut replay = self.clone();
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bulk_deficiency_updates_match_edge_updates() {
        let mut rng = Rng::new(9817);
        for n in [16usize, 32, 64, 96] {
            let k = n / 2;
            let w = n.div_ceil(64);
            for density in [1, 3, 5, 7] {
                let mut g = vec![0; n*w];
                for u in 0..n {
                    for v in u+1..n {
                        if u >= k || rng.next()%8 < density {
                            g[u*w+v/64] |= 1 << (v%64);
                            g[v*w+u/64] |= 1 << (u%64);
                        }
                    }
                }
                let mut bulk = Dense::from_bits(g, n, k);
                bulk.init_def();
                let mut incremental = bulk.clone();
                for v in 0..k {
                    assert_eq!(bulk.defelim(v), incremental.defelim_incremental(v));
                    assert_eq!(bulk.a, incremental.a);
                    assert_eq!(bulk.deg, incremental.deg);
                    for u in v+1..k { assert_eq!(bulk.def[u], incremental.def[u]); }
                }
            }
        }
    }
    #[test]
    fn suffix_retains_forced_completion_and_compresses_large_boundary() {
        for wheel in [false, true] {
            let k = if wheel { 6 } else { 4 };
            let n: usize = k + 901;
            let w = n.div_ceil(64);
            let mut bits = vec![0u64; n*w];
            let mut edge = |u: usize, v: usize| {
                bits[u*w+v/64] |= 1 << (v%64);
                bits[v*w+u/64] |= 1 << (u%64);
            };
            for u in k..n { for v in u+1..n { edge(u, v); } }
            for v in 1..k { edge(0, v); }
            for v in k..n {
                edge(0, v);
                if wheel { for u in 1..k { edge(u, v); } }
            }
            if wheel { for v in 1..k { edge(v, if v+1 == k { 1 } else { v+1 }); } }
            let graph = Dense::from_bits(bits, n, k);
            let order: Vec<_> = (0..k).collect();
            let initial = graph.score(&order);
            let (cost, candidate) = graph.bag_suffix(&order, 17, true);
            assert!(cost < initial);
            assert_eq!(cost, graph.score(&candidate));
            assert!(super::super::is_bijection(&candidate, k));
            if !wheel { assert_eq!(cost, 3*4 + 902u64.pow(2)); }
        }
    }
    #[test]
    fn weighted_exact_and_joint_match_exhaustive_orders() {
        fn enumerate(a: &Atoms, p: &mut Vec<usize>, mask: usize, best: &mut u64,
                     sources: &mut Vec<Vec<usize>>) {
            if p.len() == a.k {
                *best = (*best).min(a.score(p));
                sources.push(p.clone());
                return;
            }
            for v in 0..a.k {
                if mask & (1 << v) == 0 {
                    p.push(v);
                    enumerate(a, p, mask | (1 << v), best, sources);
                    p.pop();
                }
            }
        }
        for seed in 1..65 {
            let weights = [2usize, 3, 1, 2, 2, 1];
            let mut offsets = vec![0];
            for w in weights {
                offsets.push(offsets.last().unwrap() + w);
            }
            let n = offsets[6];
            let k = offsets[4];
            let mut rng = Rng::new(seed);
            let mut edges = [[false; 6]; 6];
            for u in 0..6 {
                edges[u][u] = rng.next() % 2 == 0;
                for v in u + 1..6 {
                    let edge = rng.next() % 3 == 0 || (u >= 4 && v >= 4);
                    edges[u][v] = edge;
                    edges[v][u] = edge;
                }
            }
            let mut bits = vec![0u64; n];
            for u in 0..6 {
                for v in 0..6 {
                    if edges[u][v] {
                        for x in offsets[u]..offsets[u + 1] {
                            for y in offsets[v]..offsets[v + 1] {
                                if x != y {
                                    bits[x] |= 1 << y;
                                }
                            }
                        }
                    }
                }
            }
            let dense = Dense::from_bits(bits, n, k);
            let initial: Vec<_> = (0..k).collect();
            let (suffix_cost, suffix_order) = dense.bag_suffix(&initial, seed, true);
            assert!(super::super::is_bijection(&suffix_order, k));
            assert!(suffix_cost <= dense.score(&initial));
            assert_eq!(suffix_cost, dense.score(&suffix_order));
            let atoms = Atoms::new(&dense);
            let mut best = u64::MAX;
            let mut sources = Vec::new();
            enumerate(&atoms, &mut Vec::new(), 0, &mut best, &mut sources);
            let (pooled, pooled_order) = atoms.pool(&sources);
            assert_eq!(pooled, best);
            assert_eq!(pooled, dense.score(&atoms.expand(&pooled_order)));
            let (bag_cost, bag_order) = atoms.bag_pool(&sources, false);
            assert_eq!(bag_cost, best);
            assert_eq!(bag_cost, dense.score(&atoms.expand(&bag_order)));
            for pair in sources.chunks(2) {
                let (cost, order) = atoms.bag_pool(pair, true);
                assert!(cost >= best);
                assert!(cost <= pair.iter().map(|p| atoms.score(p)).min().unwrap());
                assert_eq!(cost, dense.score(&atoms.expand(&order)));
            }
            let (cost, p) = atoms.exact().unwrap();
            assert_eq!(cost, best);
            assert_eq!(cost, dense.score(&atoms.expand(&p)));
            let incumbent: Vec<_> = (0..atoms.k).collect();
            for v in 0..atoms.k {
                for (position, value) in atoms.profile(&incumbent, v).into_iter().enumerate() {
                    let mut p: Vec<_> = incumbent.iter().copied().filter(|&u| u != v).collect();
                    p.insert(position, v);
                    assert_eq!(value, dense.score(&atoms.expand(&p)));
                }
            }
            for hot in [false, true] {
                let walked = atoms.profile_sources(incumbent.clone(), 16, seed, hot);
                assert!(atoms.score(&walked[0]) <= atoms.score(&incumbent));
                let (c, p) = atoms.pool(&walked);
                assert!(c <= atoms.score(&walked[0]));
                assert_eq!(c, dense.score(&atoms.expand(&p)));
            }
            let (interleaved, q) = atoms.interleave(&incumbent);
            assert!(interleaved >= best && interleaved <= atoms.score(&incumbent));
            assert_eq!(interleaved, dense.score(&atoms.expand(&q)));
            let (joint, q) = atoms.joint(&incumbent).unwrap();
            assert_eq!(joint, best);
            assert_eq!(joint, dense.score(&atoms.expand(&q)));
            let raw: Vec<_> = (0..k).collect();
            for tail in [2, 4, k] {
                let (c, q) = dense.residual_order(&raw, tail, &mut 1_000_000).unwrap();
                assert_eq!(c, dense.score(&q));
                assert!(c <= dense.score(&raw));
            }
        }
    }
    fn random_patch(n: usize, k: usize, density: u64, seed: u64) -> (Dense, Vec<Vec<bool>>) {
        let w = n.div_ceil(64);
        let mut a = vec![0u64; n * w];
        let mut adj = vec![vec![false; n]; n];
        let mut rng = Rng::new(seed);
        for u in 0..n {
            for v in u + 1..n {
                if rng.next() % 10 < density || (u >= k && v >= k) {
                    adj[u][v] = true;
                    adj[v][u] = true;
                    a[u * w + v / 64] |= 1 << (v % 64);
                    a[v * w + u / 64] |= 1 << (u % 64);
                }
            }
        }
        (Dense::from_bits(a, n, k), adj)
    }
    fn literal(adj: &[Vec<bool>], order: &[usize]) -> u64 {
        let mut b = adj.to_vec();
        let mut cost = 0u64;
        for &v in order {
            let ns: Vec<usize> = (0..b.len()).filter(|&u| b[v][u]).collect();
            cost += ((ns.len() + 1) as u64).pow(2);
            for &u in &ns {
                for &t in &ns {
                    if u != t {
                        b[u][t] = true;
                    }
                }
                b[u][v] = false;
            }
            b[v].fill(false);
        }
        cost
    }
    #[test]
    fn dense_kernels_match_literal_elimination() {
        for (n, k, density) in [(9, 6, 5), (40, 30, 3), (70, 60, 6), (130, 100, 2)] {
            for seed in [3u64, 17, 99] {
                let (d, adj) = random_patch(n, k, density, seed);
                for mode in 0..8 {
                    let (cost, order) = d.greedy(mode, seed);
                    let mut sorted = order.clone();
                    sorted.sort_unstable();
                    assert_eq!(sorted, (0..k).collect::<Vec<_>>());
                    assert_eq!(cost, literal(&adj, &order), "greedy mode {mode}");
                    let (ic, io) = d.insert(&order, order[k / 2]);
                    assert_eq!(ic, literal(&adj, &io));
                    assert!(ic <= cost);
                    let (rc, ro) = d.restart(&order, k / 3, mode, seed);
                    assert_eq!(rc, literal(&adj, &ro));
                }
                let (c, o) = d.improve((0..k).collect(), 3 * k, seed);
                assert_eq!(c, literal(&adj, &o));
                assert!(c <= literal(&adj, &(0..k).collect::<Vec<_>>()));
            }
        }
    }
    #[test]
    fn weighted_missing_edge_scan_matches_pair_oracle() {
        for n in [17, 65, 129] {
            for seed in 1..=12 {
                let (graph, _) = random_patch(n, n * 2 / 3, seed % 9 + 1, seed);
                let mut atoms = Atoms::new(&graph);
                for v in 0..atoms.n {
                    atoms.wt[v] = 1 + (v as u64 * 7 + seed) % 5;
                    atoms.clique[v] = v % 3 != 0;
                }
                for v in 0..atoms.k {
                    let neighbors = atoms.nb(v);
                    let mut oracle = 0i64;
                    for (i, &u) in neighbors.iter().enumerate() {
                        if !atoms.clique[u] { oracle += (atoms.wt[u] * (atoms.wt[u] - 1) / 2) as i64; }
                        for &x in &neighbors[i + 1..] {
                            if !atoms.has(u, x) { oracle += (atoms.wt[u] * atoms.wt[x]) as i64; }
                        }
                    }
                    assert_eq!(atoms.fill(v), oracle);
                }
            }
        }
    }

    #[test]
    fn twin_quotient_is_exact_and_projection_never_worsens() {
        // Duplicate vertices so both twin types occur among eliminable and exterior vertices.
        for seed in [5u64, 21, 77, 123] {
            let (base, _) = random_patch(24, 18, 4, seed);
            let copies = 3;
            let n = base.n * copies;
            let k = base.k * copies;
            let w = n.div_ceil(64);
            let mut a = vec![0u64; n * w];
            let mut adj = vec![vec![false; n]; n];
            let expanded = |v: usize, c: usize| {
                if v < base.k {
                    v * copies + c
                } else {
                    base.k * copies + (v - base.k) * copies + c
                }
            };
            let mut rng = Rng::new(seed);
            for u in 0..base.n {
                for v in 0..base.n {
                    let twin_clique = u == v && rng.next() % 2 == 0;
                    for cu in 0..copies {
                        for cv in 0..copies {
                            let (x, y) = (expanded(u, cu), expanded(v, cv));
                            if x == y {
                                continue;
                            }
                            if (u != v && base.has(u, v)) || (u == v && twin_clique) {
                                adj[x][y] = true;
                                a[x * w + y / 64] |= 1 << (y % 64);
                            }
                        }
                    }
                }
            }
            let d = Dense::from_bits(a, n, k);
            let atoms = Atoms::new(&d);
            assert!(
                atoms.k <= base.k,
                "eliminable blocks {} > {}",
                atoms.k,
                base.k
            );
            let raw: Vec<usize> = (0..k)
                .map(|i| (i * 7 + seed as usize) % k)
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            let raw: Vec<usize> = {
                let mut r = raw;
                let mut g = Rng::new(seed);
                for i in (1..r.len()).rev() {
                    let j = (g.next() % (i as u64 + 1)) as usize;
                    r.swap(i, j);
                }
                r
            };
            let projected = atoms.project(&raw);
            assert_eq!(projected.len(), atoms.k);
            let expanded_order = atoms.expand(&projected);
            assert_eq!(atoms.score(&projected), literal(&adj, &expanded_order));
            assert!(
                atoms.score(&projected) <= literal(&adj, &raw),
                "projection worsened"
            );
            for mode in 0..8 {
                let (c, o) = atoms.greedy(mode, seed);
                assert_eq!(
                    c,
                    literal(&adj, &atoms.expand(&o)),
                    "atom greedy mode {mode}"
                );
                let (ic, io) = atoms.insert(&o, o[0]);
                assert_eq!(ic, literal(&adj, &atoms.expand(&io)));
                assert!(ic <= c);
            }
            let (c, o) = atoms.improve(projected, 2 * atoms.k, seed);
            assert_eq!(c, literal(&adj, &atoms.expand(&o)));
        }
    }
}
