pub(crate) const MAX_N: usize = 12_000;
const WINDOW_MAX_K: usize = 16;

#[inline]
fn below(s: &mut u64, n: u32) -> u32 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    ((*s >> 32) * n as u64 >> 32) as u32
}

struct Budget {
    remaining: i64,
}

impl Budget {
    fn charge(&mut self, cost: usize) -> bool {
        let Ok(cost) = i64::try_from(cost) else {
            return false;
        };
        if cost > self.remaining {
            return false;
        }
        self.remaining -= cost;
        true
    }

    fn eliminate(&mut self, game: &mut Game<'_>, v: usize) -> Option<u64> {
        self.charge(game.elimination_ops(v))
            .then(|| game.eliminate(v))
    }
}

struct Game<'a> {
    n: usize,
    w: usize,
    adj0: &'a [u64],
    adj: Vec<u64>,
    deg0: Vec<u32>,
    deg: Vec<u32>,
    livelist: Vec<u32>,
    pos: Vec<u32>,
    bhead: Vec<i32>,
    bnext: Vec<i32>,
    bprev: Vec<i32>,
    mind: usize,
    nlive: usize,
    nelim: usize,
    use_buckets: bool,
    update_boundary: bool,
    nlist: Vec<u32>,
    cand: Vec<u32>,
    words: Vec<(usize, u64)>,
    ops: i64,
}

impl<'a> Game<'a> {
    fn build_adj(n: usize, cp: &[usize], ri: &[usize]) -> Option<Vec<u64>> {
        if n == 0 || n > MAX_N {
            return None;
        }
        let w = n.div_ceil(64);
        let mut adj = vec![0; n * w];
        for v in 0..n {
            for &u in &ri[cp[v]..cp[v + 1]] {
                if u >= n {
                    return None;
                }
                if u > v {
                    adj[v * w + (u >> 6)] |= 1u64 << (u & 63);
                    adj[u * w + (v >> 6)] |= 1u64 << (v & 63);
                }
            }
        }
        Some(adj)
    }

    fn new(n: usize, adj0: &'a [u64], nelim: usize) -> Option<Self> {
        if n == 0 || n > MAX_N {
            return None;
        }
        let w = n.div_ceil(64);
        if adj0.len() < n * w {
            return None;
        }
        let deg0 = adj0[..n * w]
            .chunks_exact(w)
            .map(|row| row.iter().map(|x| x.count_ones()).sum())
            .collect();
        Some(Self {
            n,
            w,
            adj0,
            adj: vec![0; n * w],
            deg0,
            deg: vec![0; n],
            livelist: Vec::with_capacity(n),
            pos: vec![0; n],
            bhead: vec![-1; n + 1],
            bnext: vec![-1; n],
            bprev: vec![-1; n],
            mind: 0,
            nlive: 0,
            nelim: nelim.min(n),
            use_buckets: n > 1500,
            update_boundary: true,
            nlist: Vec::with_capacity(n),
            cand: Vec::with_capacity(n),
            words: Vec::with_capacity(w),
            ops: 0,
        })
    }

    fn reset_ops(&self) -> usize {
        2 * self.n * self.w + 8 * self.n
    }

    fn reset(&mut self) {
        let rows = if self.update_boundary {
            self.n
        } else {
            self.nelim
        };
        self.adj[..rows * self.w].copy_from_slice(&self.adj0[..rows * self.w]);
        self.bhead.fill(-1);
        self.livelist.clear();
        self.deg[..rows].copy_from_slice(&self.deg0[..rows]);
        for v in 0..self.nelim {
            if self.use_buckets {
                self.link(v, self.deg[v] as usize);
            } else {
                self.pos[v] = self.livelist.len() as u32;
                self.livelist.push(v as u32);
            }
        }
        self.mind = 0;
        self.nlive = self.nelim;
        self.ops += self.reset_ops() as i64;
    }

    #[inline]
    fn link(&mut self, v: usize, d: usize) {
        let h = self.bhead[d];
        self.bnext[v] = h;
        self.bprev[v] = -1;
        if h >= 0 {
            self.bprev[h as usize] = v as i32;
        }
        self.bhead[d] = v as i32;
    }

    #[inline]
    fn unlink(&mut self, v: usize, d: usize) {
        let (p, next) = (self.bprev[v], self.bnext[v]);
        if p >= 0 {
            self.bnext[p as usize] = next;
        } else {
            self.bhead[d] = next;
        }
        if next >= 0 {
            self.bprev[next as usize] = p;
        }
    }

    #[inline]
    fn elimination_ops(&self, v: usize) -> usize {
        (self.deg[v] as usize + 1) * (3 * self.w + 6) + 24
    }

    fn eliminate(&mut self, v: usize) -> u64 {
        let w = self.w;
        self.nlist.clear();
        self.words.clear();
        self.words.extend(
            self.adj[v * w..(v + 1) * w]
                .iter()
                .copied()
                .enumerate()
                .filter(|&(_, b)| b != 0),
        );
        for &(k, mut bits) in &self.words {
            while bits != 0 {
                self.nlist
                    .push((64 * k + bits.trailing_zeros() as usize) as u32);
                bits &= bits - 1;
            }
        }
        let width = self.nlist.len() as u64 + 1;
        for i in 0..self.nlist.len() {
            let u = self.nlist[i] as usize;
            if !self.update_boundary && u >= self.nelim {
                continue;
            }
            let added =
                super::patch::fill_row(&mut self.adj[u * w..(u + 1) * w], &self.words, u, v);
            let degree = self.deg[u] + added - 2;
            if self.use_buckets && u < self.nelim && degree != self.deg[u] {
                self.unlink(u, self.deg[u] as usize);
                self.link(u, degree as usize);
                self.mind = self.mind.min(degree as usize);
            }
            self.deg[u] = degree;
        }
        self.ops += ((self.nlist.len() + 1) * (3 * w + 6) + 24) as i64;
        self.adj[v * w..(v + 1) * w].fill(0);
        if self.use_buckets {
            self.unlink(v, self.deg[v] as usize);
        } else {
            let p = self.pos[v] as usize;
            let last = *self.livelist.last().unwrap();
            self.livelist[p] = last;
            self.pos[last as usize] = p as u32;
            self.livelist.pop();
        }
        self.deg[v] = 0;
        self.nlive -= 1;
        width
    }

    fn deficiency(&mut self, v: usize) -> u32 {
        let w = self.w;
        let fill = super::patch::deficiency(&self.adj, w, v, self.n, &mut self.words);
        self.ops += ((self.deg[v] as usize + 1) * (2 * w + 4)) as i64;
        fill as u32
    }

    #[inline]
    fn fits(&self, cost: usize, cap: i64) -> bool {
        i64::try_from(cost)
            .ok()
            .and_then(|c| self.ops.checked_add(c))
            .is_some_and(|total| total <= cap)
    }

    fn run(
        &mut self,
        fixed: &[usize],
        policy: (u32, bool),
        rng: &mut u64,
        bound: u64,
        cap: i64,
        out: &mut Vec<usize>,
    ) -> Option<u64> {
        out.clear();
        let (slack, fill) = policy;
        self.update_boundary = fill;
        if !self.fits(self.reset_ops(), cap) {
            return None;
        }
        self.reset();
        let mut cost = 0;
        for &v in fixed {
            if !self.fits(self.elimination_ops(v), cap) {
                return None;
            }
            cost += self.eliminate(v).pow(2);
            out.push(v);
            if cost >= bound {
                return None;
            }
        }
        while self.nlive > 0 {
            if !self.fits(4 * self.n + 8, cap) {
                return None;
            }
            let minimum = if self.use_buckets {
                let mut d = self.mind;
                while d < self.n && self.bhead[d] < 0 {
                    d += 1;
                }
                self.ops += (d - self.mind) as i64 + 2;
                self.mind = d;
                d
            } else {
                self.ops += 4 * self.nlive as i64;
                self.livelist
                    .iter()
                    .map(|&v| self.deg[v as usize])
                    .min()
                    .unwrap() as usize
            };
            let cut = (minimum + slack as usize).min(self.n - 1);
            let pivot;
            if slack == 0 && !fill {
                let mut count = 0;
                let mut pick = 0;
                if self.use_buckets {
                    pick = self.bhead[minimum] as usize;
                    let mut v = self.bhead[minimum];
                    while v >= 0 {
                        count += 1;
                        if below(rng, count) == 0 {
                            pick = v as usize;
                        }
                        v = self.bnext[v as usize];
                    }
                    self.ops += count as i64 * 2 + 4;
                } else {
                    for &v in &self.livelist {
                        if self.deg[v as usize] as usize == minimum {
                            count += 1;
                            if below(rng, count) == 0 {
                                pick = v as usize;
                            }
                        }
                    }
                }
                pivot = pick;
            } else {
                self.cand.clear();
                if self.use_buckets {
                    for d in minimum..=cut {
                        let mut v = self.bhead[d];
                        while v >= 0 {
                            self.cand.push(v as u32);
                            v = self.bnext[v as usize];
                        }
                    }
                    self.ops += self.cand.len() as i64 * 2 + 4;
                } else {
                    for &v in &self.livelist {
                        if self.deg[v as usize] as usize <= cut {
                            self.cand.push(v);
                        }
                    }
                }
                if fill && self.cand.len() > 1 {
                    let candidates = std::mem::take(&mut self.cand);
                    let (mut best, mut count, mut pick) = (u32::MAX, 0, candidates[0]);
                    for &v in &candidates {
                        if !self.fits((self.deg[v as usize] as usize + 1) * (2 * self.w + 4), cap) {
                            return None;
                        }
                        let d = self.deficiency(v as usize);
                        if d < best {
                            best = d;
                            count = 1;
                            pick = v;
                        } else if d == best {
                            count += 1;
                            if below(rng, count) == 0 {
                                pick = v;
                            }
                        }
                    }
                    self.cand = candidates;
                    pivot = pick as usize;
                } else {
                    pivot = self.cand[below(rng, self.cand.len() as u32) as usize] as usize;
                }
            }
            if !self.fits(self.elimination_ops(pivot), cap) {
                return None;
            }
            cost += self.eliminate(pivot).pow(2);
            out.push(pivot);
            if cost >= bound {
                return None;
            }
        }
        Some(cost)
    }
}

fn search_partial(
    n: usize,
    adj: &[u64],
    nelim: usize,
    seed: &[usize],
    seed_cost: u64,
    budget: i64,
    rng_seed: u64,
    prefix_mode: u8,
) -> Option<(Vec<usize>, u64)> {
    let mut game = Game::new(n, adj, nelim)?;
    let mut rng = rng_seed | 1;
    let policies = [
        (0, false),
        (1, false),
        (2, false),
        (1, true),
        (3, false),
        (5, false),
        (8, false),
        (16, false),
    ];
    let cap = budget + budget / 4;
    let mut best = seed_cost;
    let mut best_order = Vec::new();
    let mut out = Vec::with_capacity(n);
    let (mut iteration, mut last_run) = (0, 0);
    while game.ops + last_run <= budget / 16 {
        let before = game.ops;
        let policy = policies[iteration % policies.len()];
        iteration += 1;
        if let Some(cost) = game.run(&[], policy, &mut rng, best, cap, &mut out) {
            if cost < best {
                best = cost;
                best_order = out.clone();
            }
        }
        last_run = last_run.max(game.ops - before);
        if game.ops == before {
            break;
        }
    }
    let mut current = if best_order.is_empty() {
        seed.to_vec()
    } else {
        best_order.clone()
    };
    let mut current_cost = best;
    while game.ops + last_run <= budget {
        let before = game.ops;
        let ne = game.nelim.max(1);
        let prefix = match prefix_mode {
            0 => below(&mut rng, ne as u32) as usize,
            2 if iteration % 2 == 0 => below(&mut rng, ne as u32) as usize,
            _ => {
                let e = below(&mut rng, usize::BITS - ne.leading_zeros());
                let tail = 1 + below(&mut rng, 1u32 << e) as usize;
                ne.saturating_sub(tail.min(ne))
            }
        };
        let policy = policies[iteration % policies.len()];
        iteration += 1;
        let result = game.run(
            &current[..prefix.min(current.len())],
            policy,
            &mut rng,
            current_cost + 1,
            cap,
            &mut out,
        );
        last_run = last_run.max(game.ops - before);
        if game.ops == before {
            break;
        }
        if let Some(cost) = result {
            if cost < best {
                best = cost;
                best_order = out.clone();
            }
            current_cost = cost;
            std::mem::swap(&mut current, &mut out);
        }
    }
    (best < seed_cost && !best_order.is_empty()).then_some((best_order, best))
}

pub(crate) fn search(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
    cost: u64,
    budget: i64,
    rng: u64,
) -> Option<(Vec<usize>, u64)> {
    let adj = Game::build_adj(n, cp, ri)?;
    search_partial(n, &adj, n, seed, cost, budget, rng, 2)
}

fn validate(work: &mut Budget, n: usize, cp: &[usize], ri: &[usize], seed: &[usize]) -> bool {
    if !work.charge((n + 1).saturating_add(ri.len()).saturating_add(2 * n))
        || cp.len() != n + 1
        || seed.len() != n
        || cp.first() != Some(&0)
        || cp.last() != Some(&ri.len())
        || cp.windows(2).any(|p| p[0] > p[1] || p[1] > ri.len())
        || ri.iter().any(|&v| v >= n)
    {
        return false;
    }
    let mut seen = vec![false; n];
    for &v in seed {
        if v >= n || seen[v] {
            return false;
        }
        seen[v] = true;
    }
    true
}

fn prepare(
    work: &mut Budget,
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
) -> Option<Vec<u64>> {
    if !validate(work, n, cp, ri, seed) {
        return None;
    }
    let w = n.div_ceil(64);
    if !work.charge(
        n.saturating_mul(w)
            .saturating_add(2 * ri.len())
            .saturating_add(n),
    ) {
        return None;
    }
    Game::build_adj(n, cp, ri)
}

fn setup_ops(n: usize) -> usize {
    let w = n.div_ceil(64);
    2usize
        .saturating_mul(n)
        .saturating_mul(w)
        .saturating_add(13 * n)
        .saturating_add(w)
}

pub(crate) fn adjacent_pair_descent(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
    sweeps: usize,
    budget: i64,
) -> Option<Vec<usize>> {
    if n < 2 || seed.len() != n || sweeps == 0 || budget <= 0 {
        return None;
    }
    if !super::is_bijection(seed, n) {
        return None;
    }
    let adj = Game::build_adj(n, cp, ri)?;
    let mut game = Game::new(n, &adj, n)?;
    let mut cur = seed.to_vec();
    let mut next = Vec::with_capacity(n);
    let mut changed_any = false;
    for sweep in 0..sweeps {
        game.reset();
        if game.ops > budget {
            return None;
        }
        next.clear();
        let mut i = 0;
        if sweep & 1 == 1 {
            next.push(cur[0]);
            game.eliminate(cur[0]);
            if game.ops > budget {
                return None;
            }
            i = 1;
        }
        let mut changed = false;
        while i + 1 < n {
            let (a, b) = (cur[i], cur[i + 1]);
            let swap = game.adj[a * game.w + (b >> 6)] & (1u64 << (b & 63)) != 0
                && game.deg[b] < game.deg[a];
            changed |= swap;
            for v in if swap { [b, a] } else { [a, b] } {
                next.push(v);
                game.eliminate(v);
                if game.ops > budget {
                    return None;
                }
            }
            i += 2;
        }
        if i < n {
            next.push(cur[i]);
            game.eliminate(cur[i]);
            if game.ops > budget {
                return None;
            }
        }
        if changed {
            changed_any = true;
            std::mem::swap(&mut cur, &mut next);
        }
    }
    changed_any.then_some(cur)
}

struct SmallWindow<const K: usize> {
    union: [u8; 32],
    width: [u32; 32],
}

impl<const K: usize> SmallWindow<K> {
    fn new(game: &Game<'_>, verts: [usize; K], work: &mut Budget) -> Option<Self> {
        let scalar = if K == 4 { 192 * game.w + 4096 } else { 8192 };
        if !work.charge(scalar) {
            return None;
        }
        let rows = verts.map(|v| &game.adj[v * game.w..(v + 1) * game.w]);
        let mut inside = [0u8; K];
        for i in 0..K {
            for j in 0..K {
                if rows[i][verts[j] >> 6] & (1u64 << (verts[j] & 63)) != 0 {
                    inside[i] |= 1 << j;
                }
            }
        }
        let mut result = Self {
            union: [0; 32],
            width: [0; 32],
        };
        let mut connected = [false; 32];
        let mut count = 0;
        for mask in 1usize..1 << K {
            result.union[mask] =
                result.union[mask & (mask - 1)] | inside[mask.trailing_zeros() as usize];
            connected[mask] =
                result.component(mask as u8, mask.trailing_zeros() as usize) as usize == mask;
            count += usize::from(connected[mask] && mask.count_ones() > 1);
        }
        if K == 5 && !work.charge((20 * count + 16) * game.w) {
            return None;
        }
        for i in 0..K {
            result.width[1 << i] = game.deg[verts[i]] + 1;
        }
        for mask in 1usize..1 << K {
            let size = mask.count_ones() as usize;
            if !connected[mask] || size == 1 {
                continue;
            }
            let mut indices = [0; 5];
            let mut len = 0;
            for i in 0..K {
                if mask & (1 << i) != 0 {
                    indices[len] = i;
                    len += 1;
                }
            }
            let a = rows[indices[0]];
            let b = rows[indices[1]];
            let c = rows[indices[2]];
            let d = rows[indices[3]];
            let e = rows[indices[4]];
            let union: u32 = match size {
                2 => a.iter().zip(b).map(|(&a, &b)| (a | b).count_ones()).sum(),
                3 => a
                    .iter()
                    .zip(b)
                    .zip(c)
                    .map(|((&a, &b), &c)| (a | b | c).count_ones())
                    .sum(),
                4 => a
                    .iter()
                    .zip(b)
                    .zip(c)
                    .zip(d)
                    .map(|(((&a, &b), &c), &d)| (a | b | c | d).count_ones())
                    .sum(),
                _ => a
                    .iter()
                    .zip(b)
                    .zip(c)
                    .zip(d)
                    .zip(e)
                    .map(|((((&a, &b), &c), &d), &e)| (a | b | c | d | e).count_ones())
                    .sum(),
            };
            result.width[mask] = union - size as u32 + 1;
        }
        Some(result)
    }

    fn component(&self, eliminated: u8, pivot: usize) -> u8 {
        let mut component = 1 << pivot;
        loop {
            let next = component | (self.union[component as usize] & eliminated);
            if next == component {
                return component;
            }
            component = next;
        }
    }

    fn width(&self, eliminated: u8, pivot: usize) -> u64 {
        self.width[self.component(eliminated, pivot) as usize] as u64
    }

    fn solve(&self) -> ([usize; K], u64, u64) {
        let mut best = [u64::MAX; 32];
        let mut path = [u16::MAX; 32];
        best[0] = 0;
        path[0] = 0;
        let full = (1 << K) - 1;
        for mask in 0..full {
            for v in 0..K {
                let bit = 1 << v;
                if mask & bit != 0 {
                    continue;
                }
                let cost = best[mask] + self.width(mask as u8, v).pow(2);
                let code = (path[mask] << 3) | v as u16;
                let next = mask | bit;
                if cost < best[next] || (cost == best[next] && code < path[next]) {
                    best[next] = cost;
                    path[next] = code;
                }
            }
        }
        let incumbent = (0..K).map(|v| self.width((1 << v) - 1, v).pow(2)).sum();
        let order = std::array::from_fn(|i| {
            if best[full] < incumbent {
                ((path[full] >> (3 * (K - i - 1))) & 7) as usize
            } else {
                i
            }
        });
        (order, best[full], incumbent)
    }
}

fn small_descent<const K: usize>(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
    budget: i64,
) -> Option<Vec<usize>> {
    if n < K || n > MAX_N || budget <= 0 || seed.len() != n || cp.len() != n + 1 {
        return None;
    }
    let mut work = Budget { remaining: budget };
    let adj = prepare(&mut work, n, cp, ri, seed)?;
    if !work.charge(setup_ops(n)) {
        return None;
    }
    let mut game = Game::new(n, &adj, n)?;
    let mut cur = seed.to_vec();
    let mut changed = false;
    for offset in 0..K {
        if offset + K > n {
            break;
        }
        if !work.charge(game.reset_ops()) {
            return changed.then_some(cur);
        }
        game.reset();
        for &v in cur.iter().take(offset) {
            if work.eliminate(&mut game, v).is_none() {
                return changed.then_some(cur);
            }
        }
        let mut i = offset;
        while i + K <= n {
            let window = std::array::from_fn(|j| cur[i + j]);
            if K != 4 || window.windows(2).any(|p| game.deg[p[0]] > game.deg[p[1]]) {
                let Some(kernel) = SmallWindow::<K>::new(&game, window, &mut work) else {
                    return changed.then_some(cur);
                };
                let (order, best, incumbent) = kernel.solve();
                if best < incumbent {
                    cur[i..i + K].copy_from_slice(&order.map(|j| window[j]));
                    changed = true;
                }
            }
            i += K;
            if i + K <= n {
                for &v in &cur[i - K..i] {
                    if work.eliminate(&mut game, v).is_none() {
                        return changed.then_some(cur);
                    }
                }
            }
        }
    }
    changed.then_some(cur)
}

pub(crate) fn adjacent_four_descent(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
    budget: i64,
) -> Option<Vec<usize>> {
    small_descent::<4>(n, cp, ri, seed, budget)
}

pub(crate) fn adjacent_five_descent(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
    budget: i64,
) -> Option<Vec<usize>> {
    small_descent::<5>(n, cp, ri, seed, budget)
}

pub(crate) fn simplicial_promotion(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
    budget: i64,
) -> Option<Vec<usize>> {
    if n < 3 || n > MAX_N || budget <= 0 || seed.len() != n || cp.len() != n + 1 {
        return None;
    }
    let mut work = Budget { remaining: budget };
    if !validate(&mut work, n, cp, ri, seed) {
        return None;
    }
    let w = n.div_ceil(64);
    if !work.charge(n * w + ri.len() + n) {
        return None;
    }
    let adj = Game::build_adj(n, cp, ri)?;
    if !work.charge(n * w + 9 * n + w) {
        return None;
    }
    let mut game = Game::new(n, &adj, n)?;
    if !work.charge(game.reset_ops()) {
        return None;
    }
    game.reset();
    if !work.charge(n) {
        return None;
    }
    let mut cur = seed.to_vec();
    let mut promotions = 0;
    for i in 0..n - 2 {
        let x = cur[i];
        let mut best = None;
        for (j, &v) in cur
            .iter()
            .enumerate()
            .take((i + 16).min(n - 1) + 1)
            .skip(i + 2)
        {
            if !work.charge(4) {
                return None;
            }
            let degree = game.deg[v];
            if degree >= game.deg[x] || game.adj[x * w + (v >> 6)] & (1u64 << (v & 63)) == 0 {
                continue;
            }
            if !work.charge((degree as usize + 1) * (2 * w + 4)) {
                return None;
            }
            if game.deficiency(v) == 0 {
                let key = (std::cmp::Reverse(degree), j, v);
                if best.is_none_or(|old| key < old) {
                    best = Some(key);
                }
            }
        }
        if let Some((_, j, _)) = best {
            if !work.charge(j - i) {
                return None;
            }
            cur[i..=j].rotate_right(1);
            promotions += 1;
            if promotions == 256 {
                return Some(cur);
            }
        }
        if i + 1 < n - 2 {
            work.eliminate(&mut game, cur[i])?;
        }
    }
    (promotions > 0).then_some(cur)
}

struct WindowKernel {
    k: usize,
    sig: Vec<u16>,
    touched: Vec<u32>,
    slot: Vec<i8>,
    hist: Vec<u32>,
    unions: Vec<u32>,
    width: Vec<u32>,
    comp: Vec<u32>,
    dp: Vec<u64>,
    last: Vec<u8>,
}

impl WindowKernel {
    fn new(n: usize, k: usize) -> Self {
        let m = 1 << k;
        Self {
            k,
            sig: vec![0; n],
            touched: Vec::new(),
            slot: vec![-1; n],
            hist: vec![0; m],
            unions: vec![0; m],
            width: vec![0; m],
            comp: vec![0; m],
            dp: vec![0; m],
            last: vec![0; m],
        }
    }

    // Outside vertices contribute only their incidence signatures on the window.
    fn solve(&mut self, game: &Game<'_>, verts: &[usize]) -> (u64, u64) {
        let k = self.k;
        let m = 1usize << k;
        let full = (m - 1) as u32;
        for (i, &v) in verts.iter().enumerate() {
            self.slot[v] = i as i8;
        }
        let mut inside = [0u32; WINDOW_MAX_K];
        self.touched.clear();
        for (i, &v) in verts.iter().enumerate() {
            for (j, &word) in game.adj[v * game.w..(v + 1) * game.w].iter().enumerate() {
                let mut bits = word;
                while bits != 0 {
                    let x = j * 64 + bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    if self.slot[x] >= 0 {
                        inside[i] |= 1 << self.slot[x];
                    } else {
                        if self.sig[x] == 0 {
                            self.touched.push(x as u32);
                        }
                        self.sig[x] |= 1 << i;
                    }
                }
            }
        }
        for &v in verts {
            self.slot[v] = -1;
        }
        self.hist.fill(0);
        for &x in &self.touched {
            self.hist[self.sig[x as usize] as usize] += 1;
            self.sig[x as usize] = 0;
        }
        let boundary = self.touched.len() as u32;
        for i in 0..k {
            for s in 0..m {
                if s & (1 << i) != 0 {
                    self.hist[s] += self.hist[s ^ (1 << i)];
                }
            }
        }
        self.unions[0] = 0;
        for s in 1..m {
            let low = s & s.wrapping_neg();
            let u = self.unions[s ^ low] | inside[low.trailing_zeros() as usize];
            self.unions[s] = u;
            self.width[s] =
                1 + (u & !(s as u32) & full).count_ones() + boundary - self.hist[full as usize ^ s];
            let mut c = low as u32;
            loop {
                let next = c | (self.unions[c as usize] & s as u32);
                if next == c {
                    break;
                }
                c = next;
            }
            self.comp[s] = c;
        }
        self.dp[0] = 0;
        for s in 1..m {
            let c = self.comp[s] as usize;
            if c != s {
                self.dp[s] = self.dp[c] + self.dp[s ^ c];
                self.last[s] = u8::MAX;
                continue;
            }
            let mut best = u64::MAX;
            let mut arg = 0;
            let mut bits = s;
            while bits != 0 {
                let v = bits.trailing_zeros();
                bits &= bits - 1;
                let value = self.dp[s ^ (1usize << v)];
                if value < best {
                    best = value;
                    arg = v as u8;
                }
            }
            self.dp[s] = best + (self.width[s] as u64).pow(2);
            self.last[s] = arg;
        }
        let mut incumbent = 0;
        for j in 0..k {
            let set = (1u32 << (j + 1)) - 1;
            let mut c = 1u32 << j;
            loop {
                let next = c | (self.unions[c as usize] & set);
                if next == c {
                    break;
                }
                c = next;
            }
            incumbent += (self.width[c as usize] as u64).pow(2);
        }
        (self.dp[m - 1], incumbent)
    }

    fn reconstruct(&self, s: usize, out: &mut Vec<usize>) {
        if s == 0 {
            return;
        }
        let c = self.comp[s] as usize;
        if c != s {
            self.reconstruct(c, out);
            self.reconstruct(s ^ c, out);
        } else {
            let v = self.last[s] as usize;
            self.reconstruct(s ^ (1 << v), out);
            out.push(v);
        }
    }
}

pub(crate) fn window_descent(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
    k: usize,
    stride: usize,
    budget: i64,
) -> Option<Vec<usize>> {
    if !(2..=WINDOW_MAX_K).contains(&k)
        || stride == 0
        || stride > k
        || n < k
        || n > MAX_N
        || budget <= 0
        || seed.len() != n
        || cp.len() != n + 1
    {
        return None;
    }
    let mut work = Budget { remaining: budget };
    let adj = prepare(&mut work, n, cp, ri, seed)?;
    if !work.charge(
        setup_ops(n)
            .saturating_add(3 * n)
            .saturating_add(6usize << k),
    ) {
        return None;
    }
    let mut game = Game::new(n, &adj, n)?;
    game.reset();
    let mut kernel = WindowKernel::new(n, k);
    let mut cur = seed.to_vec();
    let mut changed = false;
    let mut order = Vec::with_capacity(k);
    let mut window = vec![0; k];
    let mut i = 0;
    while i + k <= n {
        let incidence: usize = cur[i..i + k].iter().map(|&v| game.deg[v] as usize).sum();
        if !work.charge(k * game.w + incidence + ((k + 4) << k)) {
            return changed.then_some(cur);
        }
        let (best, incumbent) = kernel.solve(&game, &cur[i..i + k]);
        if best < incumbent {
            order.clear();
            kernel.reconstruct((1 << k) - 1, &mut order);
            window.copy_from_slice(&cur[i..i + k]);
            for (j, &v) in order.iter().enumerate() {
                cur[i + j] = window[v];
            }
            changed = true;
        }
        if i + stride + k > n {
            break;
        }
        for &v in &cur[i..i + stride] {
            if work.eliminate(&mut game, v).is_none() {
                return changed.then_some(cur);
            }
        }
        i += stride;
    }
    changed.then_some(cur)
}

fn insertion_profile(
    work: &mut Budget,
    game: &mut Game<'_>,
    cur: &[usize],
    v: usize,
    delta: &mut [i64],
    profile: &mut [u64],
) -> Option<usize> {
    let n = cur.len();
    if !work.charge(game.reset_ops() + 2 * n) {
        return None;
    }
    game.reset();
    let (mut total, mut i, mut old) = (0, 0, n - 1);
    for &u in cur {
        if u == v {
            old = i;
            continue;
        }
        delta[i] = if game.adj[u * game.w + (v >> 6)] & (1u64 << (v & 63)) != 0 {
            (game.deg[v] as i64 + 1).pow(2) - (game.deg[u] as i64 + 1).pow(2)
        } else {
            0
        };
        total += work.eliminate(game, u)?.pow(2);
        i += 1;
    }
    let mut running = total as i64 + 1;
    profile[n - 1] = running as u64;
    for i in (0..n - 1).rev() {
        running += delta[i];
        profile[i] = running as u64;
    }
    Some(old)
}

pub(crate) fn insertion_descent(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    seed: &[usize],
    sweeps: usize,
    budget: i64,
) -> Option<Vec<usize>> {
    if n < 3 || n > MAX_N || sweeps == 0 || budget <= 0 || seed.len() != n || cp.len() != n + 1 {
        return None;
    }
    let mut work = Budget { remaining: budget };
    let adj = prepare(&mut work, n, cp, ri, seed)?;
    if !work.charge(setup_ops(n)) {
        return None;
    }
    let mut game = Game::new(n, &adj, n)?;
    if !work.charge(game.reset_ops()) {
        return None;
    }
    game.reset();
    let mut cur = seed.to_vec();
    let mut cost = 0;
    for &v in &cur {
        cost += work.eliminate(&mut game, v)?.pow(2);
    }
    let mut delta = vec![0; n];
    let mut profile = vec![0; n];
    let mut changed = false;
    for _ in 0..sweeps {
        let scan: Vec<_> = cur.iter().rev().copied().collect();
        let mut sweep_changed = false;
        for v in scan {
            let Some(old) =
                insertion_profile(&mut work, &mut game, &cur, v, &mut delta, &mut profile)
            else {
                return changed.then_some(cur);
            };
            if profile[old] != cost {
                return None;
            }
            let (mut best, mut position) = (cost, old);
            for (j, &value) in profile.iter().enumerate() {
                if value < best {
                    best = value;
                    position = j;
                }
            }
            if best < cost {
                if !work.charge(2 * n) {
                    return changed.then_some(cur);
                }
                cur.remove(old);
                cur.insert(position, v);
                cost = best;
                changed = true;
                sweep_changed = true;
            }
        }
        if !sweep_changed {
            break;
        }
    }
    changed.then_some(cur)
}

fn rank_product(value: u64, len: usize) -> [u64; 6] {
    let mut words = [0u64; 6];
    words[0] = 1;
    for factor in [
        value, value, value, value, len as u64, len as u64, len as u64,
    ] {
        let mut carry = 0;
        for word in &mut words {
            let product = *word as u128 * factor as u128 + carry;
            *word = product as u64;
            carry = product >> 64;
        }
    }
    words
}

#[derive(Clone, Copy)]
pub(crate) struct SubCfg {
    pub(crate) min_s: usize,
    pub(crate) max_s: usize,
    pub(crate) max_sub: usize,
    pub(crate) max_blocks: usize,
    pub(crate) budget: i64,
    pub(crate) streams: usize,
    pub(crate) round: usize,
}

fn collect_subtree_vertices(
    cp: &[usize],
    ri: &[usize],
    block: &[usize],
    max_sub: usize,
    local: &mut [u32],
    touched: &mut Vec<usize>,
    verts: &mut Vec<usize>,
) -> bool {
    verts.clear();
    for &v in touched.iter() {
        local[v] = u32::MAX;
    }
    touched.clear();
    let limit = max_sub.min(MAX_N);
    if block.len() > limit {
        return false;
    }
    for &v in block {
        local[v] = verts.len() as u32;
        touched.push(v);
        verts.push(v);
    }
    for &v in block {
        for &u in &ri[cp[v]..cp[v + 1]] {
            if u < local.len() && local[u] == u32::MAX {
                if verts.len() == limit {
                    return false;
                }
                local[u] = verts.len() as u32;
                touched.push(u);
                verts.push(u);
            }
        }
    }
    true
}

// Each subtree must occupy a contiguous interval in the supplied postorder.
pub(crate) fn subtree_refine(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    perm: &mut [usize],
    counts: &[u32],
    parent: &[i32],
    cfg: SubCfg,
) -> usize {
    let mut size = vec![1u32; n];
    for j in 0..n {
        if parent[j] >= 0 {
            size[parent[j] as usize] += size[j];
        }
    }
    let mut covered = vec![false; n];
    let mut blocks = Vec::new();
    for j in (0..n).rev() {
        let len = size[j] as usize;
        if covered[j] || len < cfg.min_s || len > cfg.max_s {
            continue;
        }
        let start = j + 1 - len;
        covered[start..=j].fill(true);
        blocks.push((start, j));
    }
    {
        let mut ranked: Vec<_> = blocks
            .drain(..)
            .map(|(a, b)| {
                let cost = counts[a..=b]
                    .iter()
                    .map(|&c| (c as u64).pow(2))
                    .sum::<u64>();
                (a, b, cost)
            })
            .collect();
        ranked.sort_by(|a, b| {
            rank_product(b.2, a.1 + 1 - a.0)
                .iter()
                .rev()
                .cmp(rank_product(a.2, b.1 + 1 - b.0).iter().rev())
                .then_with(|| b.2.cmp(&a.2))
                .then_with(|| b.1.cmp(&a.1))
        });
        blocks.extend(
            ranked
                .into_iter()
                .take(cfg.max_blocks)
                .map(|(a, b, _)| (a, b)),
        );
    }
    if blocks.is_empty() {
        return 0;
    }
    let input: &[usize] = perm;
    let output = {
        let mut local = vec![u32::MAX; n];
        let mut touched = Vec::new();
        let mut verts = Vec::new();
        let max_sub = cfg.max_sub.min(MAX_N);
        let mut adj = vec![0u64; max_sub * max_sub.div_ceil(64)];
        let mut output = Vec::new();
        for (rank, &(a, b)) in blocks.iter().enumerate() {
            let len = b + 1 - a;
            if !collect_subtree_vertices(
                cp,
                ri,
                &input[a..=b],
                cfg.max_sub,
                &mut local,
                &mut touched,
                &mut verts,
            ) {
                continue;
            }
            let m = verts.len();
            let w = m.div_ceil(64);
            let needed = m * w;
            if adj.len() < needed {
                adj.resize(needed, 0);
            }
            adj[..needed].fill(0);
            for (i, &v) in verts.iter().enumerate() {
                for &u in &ri[cp[v]..cp[v + 1]] {
                    if u >= n {
                        continue;
                    }
                    let j = local[u];
                    if j == u32::MAX || j as usize <= i {
                        continue;
                    }
                    let j = j as usize;
                    adj[i * w + (j >> 6)] |= 1u64 << (j & 63);
                    adj[j * w + (i >> 6)] |= 1u64 << (i & 63);
                }
            }
            let cost = counts[a..=b].iter().map(|&c| (c as u64).pow(2)).sum();
            let seed: Vec<_> = (0..len).collect();
            let mut best: Option<(Vec<usize>, u64)> = None;
            for k in 0..cfg.streams.max(1) {
                let mut rng =
                    0x9E37_79B9_7F4A_7C15u64.wrapping_mul(2 * k as u64 + 1) ^ (k as u64) << 32;
                if n >= 10_000 && k == 0 && cfg.round == 1 && rank & 1 == 1 {
                    rng ^= 0xE703_7ED1_A0B4_28DB;
                }
                let mode = if k == 0 { 2 } else { 0 };
                if let Some(result) =
                    search_partial(m, &adj[..needed], len, &seed, cost, cfg.budget, rng, mode)
                {
                    if best.as_ref().is_none_or(|b| result.1 < b.1) {
                        best = Some(result);
                    }
                }
            }
            if let Some((order, _)) = best {
                if order.len() == len {
                    output.push((a, order.iter().map(|&i| verts[i]).collect::<Vec<_>>()));
                }
            }
        }
        output
    };
    let mut improved = 0;
    for (start, order) in output {
        perm[start..start + order.len()].copy_from_slice(&order);
        improved += 1;
    }
    improved
}
