//! Exact multi-ancestor elimination-tree promotions. Intermediate rotations may
//! increase cost; only the cheapest prefix of each bounded lift is retained.
use crate::Pattern;
struct Undo {
    v: usize,
    p: usize,
    gp: Option<usize>,
    slot: usize,
    bv: Vec<(usize, u32)>,
    bp: Vec<(usize, u32)>,
    kv: Vec<usize>,
    kp: Vec<usize>,
    cost: u64,
}
struct Tree<'a> {
    pattern: &'a Pattern,
    order: Vec<usize>,
    parent: Vec<Option<usize>>,
    kids: Vec<Vec<usize>>,
    bags: Vec<Vec<(usize, u32)>>,
    cost: u64,
    work: usize,
    stamp: Vec<usize>,
    support: Vec<u32>,
    epoch: usize,
}
impl Tree<'_> {
    fn contains(&self, v: usize, u: usize) -> bool {
        self.bags[v].binary_search_by_key(&u, |a| a.0).is_ok()
    }
    fn rotate(&mut self, v: usize) -> Option<Undo> {
        let p = self.parent[v]?;
        let gp = self.parent[p];
        let slot = gp
            .and_then(|q| self.kids[q].iter().position(|&u| u == p))
            .unwrap_or(0);
        self.epoch += 1;
        let mut pk: Vec<usize> = self.kids[p].iter().copied().filter(|&u| u != v).collect();
        let mut vk = Vec::new();
        let mut moved = 0u32;
        for &ch in &self.kids[v] {
            self.work += 1;
            if self.contains(ch, p) {
                moved += 1;
                pk.push(ch);
                for &(u, _) in &self.bags[ch] {
                    if self.stamp[u] != self.epoch {
                        self.stamp[u] = self.epoch;
                        self.support[u] = 0;
                    }
                    self.support[u] += 1;
                    self.work += 1;
                }
            } else {
                vk.push(ch);
            }
        }
        vk.push(p);
        let mut np = Vec::new();
        let mut nv = Vec::new();
        let mut j = 0;
        for &(a, ct) in &self.bags[p] {
            while j < self.bags[v].len() && self.bags[v][j].0 < a {
                j += 1;
            }
            let cv = if j < self.bags[v].len() && self.bags[v][j].0 == a {
                self.bags[v][j].1
            } else {
                0
            };
            let mc = if self.stamp[a] == self.epoch {
                self.support[a]
            } else {
                0
            };
            let cp = ct.checked_sub(u32::from(cv > 0))?.checked_add(mc)?;
            let cn = cv.checked_sub(mc)?.checked_add(u32::from(cp > 0))?;
            if cn == 0 {
                return None;
            }
            if cp > 0 {
                np.push((a, cp));
            }
            nv.push((a, cn));
            self.work += 1;
        }
        let pv = self.order[p];
        let vv = self.order[v];
        let adjacent = self.pattern.row_idx[self.pattern.col_ptr[pv]..self.pattern.col_ptr[pv + 1]]
            .binary_search(&vv)
            .is_ok();
        let ct = moved + u32::from(adjacent);
        if ct == 0 {
            return None;
        }
        let slotv = np.partition_point(|a| a.0 < v);
        np.insert(slotv, (v, ct));
        let cost = self.cost;
        let before = (self.bags[v].len() + 1) as u64;
        let after = (np.len() + 1) as u64;
        self.cost = self
            .cost
            .checked_sub(before * before)?
            .checked_add(after * after)?;
        let bv = std::mem::replace(&mut self.bags[v], nv);
        let bp = std::mem::replace(&mut self.bags[p], np);
        let kv = std::mem::replace(&mut self.kids[v], vk);
        let kp = std::mem::replace(&mut self.kids[p], pk);
        self.parent[v] = gp;
        self.parent[p] = Some(v);
        if let Some(q) = gp {
            self.kids[q][slot] = v;
        }
        for &ch in &self.kids[p] {
            self.parent[ch] = Some(p);
        }
        for &ch in &self.kids[v] {
            self.parent[ch] = Some(v);
        }
        Some(Undo {
            v,
            p,
            gp,
            slot,
            bv,
            bp,
            kv,
            kp,
            cost,
        })
    }
    fn undo(&mut self, e: Undo) {
        self.bags[e.v] = e.bv;
        self.bags[e.p] = e.bp;
        self.kids[e.v] = e.kv;
        self.kids[e.p] = e.kp;
        self.parent[e.v] = Some(e.p);
        self.parent[e.p] = e.gp;
        if let Some(q) = e.gp {
            self.kids[q][e.slot] = e.p;
        }
        for &c in &self.kids[e.v] {
            self.parent[c] = Some(e.v);
        }
        for &c in &self.kids[e.p] {
            self.parent[c] = Some(e.p);
        }
        self.cost = e.cost;
    }
    fn output(&self) -> Vec<usize> {
        let mut stack = Vec::new();
        for v in (0..self.order.len()).rev() {
            if self.parent[v].is_none() {
                stack.push((v, false));
            }
        }
        let mut out = Vec::new();
        while let Some((v, exit)) = stack.pop() {
            if exit {
                out.push(self.order[v]);
            } else {
                stack.push((v, true));
                for &ch in self.kids[v].iter().rev() {
                    stack.push((ch, false));
                }
            }
        }
        out
    }
}
pub(super) fn refine(pattern: &Pattern, p: &[usize]) -> Option<Vec<usize>> {
    if !(64..=80_000).contains(&pattern.n) || pattern.nnz() > 500_000 {
        return None;
    }
    refine_impl(pattern, p, pattern.n, 2_000_000, 32, 1)
}
pub(super) fn boundary(pattern: &Pattern, p: &[usize], free: usize) -> Option<Vec<usize>> {
    boundary_with_budget(pattern, p, free, 500_000, 128, 1)
}
/// Boundary-fixed lifts of the first `free` vertices. `passes` scans: a single
/// pass visits vertices by descending column count; several passes alternate
/// descending bag size with a fixed-seed shuffle of the current postorder.
pub(super) fn boundary_with_budget(
    pattern: &Pattern,
    p: &[usize],
    free: usize,
    work: usize,
    depth: usize,
    passes: usize,
) -> Option<Vec<usize>> {
    if pattern.n > 10000 || free > pattern.n {
        return None;
    }
    refine_impl(pattern, p, free, work, depth, passes)
}
fn refine_impl(
    pattern: &Pattern,
    p: &[usize],
    free: usize,
    work_limit: usize,
    depth: usize,
    passes: usize,
) -> Option<Vec<usize>> {
    let n = pattern.n;
    let sp = super::ScoringPattern {
        n,
        col_ptr: pattern.col_ptr.clone(),
        row_idx: pattern.row_idx.clone(),
    };
    let (order, counts, par) = super::etree_prep(&sp, p);
    let mut rank = vec![0; n];
    for (j, &v) in order.iter().enumerate() {
        rank[v] = j;
    }
    let parent: Vec<_> = par
        .into_iter()
        .map(|v| if v < 0 { None } else { Some(v as usize) })
        .collect();
    let mut kids = vec![Vec::new(); n];
    for v in 0..n {
        if let Some(q) = parent[v] {
            kids[q].push(v);
        }
    }
    let mut bags: Vec<Vec<(usize, u32)>> = vec![Vec::new(); n];
    let mut budget = 4_000_000usize;
    for v in 0..n {
        let mut tmp = Vec::new();
        let original = order[v];
        for &u in &pattern.row_idx[pattern.col_ptr[original]..pattern.col_ptr[original + 1]] {
            if rank[u] > v {
                tmp.push(rank[u]);
            }
        }
        for &ch in &kids[v] {
            budget = budget.checked_sub(bags[ch].len())?;
            tmp.extend(bags[ch].iter().map(|a| a.0).filter(|&u| u != v));
        }
        budget = budget.checked_sub(tmp.len())?;
        tmp.sort_unstable();
        for u in tmp {
            if let Some(last) = bags[v].last_mut() {
                if last.0 == u {
                    last.1 += 1;
                    continue;
                }
            }
            bags[v].push((u, 1));
        }
        if bags[v].len() + 1 != counts[v] as usize {
            return None;
        }
    }
    let cost = counts.iter().map(|&c| (c as u64).pow(2)).sum();
    let mut tree = Tree {
        pattern,
        order,
        parent,
        kids,
        bags,
        cost,
        work: 0,
        stamp: vec![0; n],
        support: vec![0; n],
        epoch: 0,
    };
    let mut seed = 817293u64;
    for pass in 0..passes.max(1) {
        let start_cost = tree.cost;
        let mut scan: Vec<_> = tree.output().iter().map(|&v| rank[v]).collect();
        if passes <= 1 {
            scan = (0..n).collect();
            scan.sort_by_key(|&v| (std::cmp::Reverse(counts[v]), v));
        } else if pass % 2 == 0 {
            scan.sort_by_key(|&v| std::cmp::Reverse(tree.bags[v].len()));
        } else {
            for j in (1..n).rev() {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                scan.swap(j, seed as usize % (j + 1));
            }
        }
        for v in scan {
            if tree.work >= work_limit {
                break;
            }
            if tree.order[v] >= free {
                continue;
            }
            let mut hist = Vec::new();
            let mut best = tree.cost;
            let mut keep = 0;
            for _ in 0..depth {
                if tree.parent[v].is_none()
                    || tree.parent[v].is_some_and(|p| tree.order[p] >= free)
                    || tree.work >= work_limit
                {
                    break;
                }
                let e = tree.rotate(v)?;
                hist.push(e);
                if tree.cost < best {
                    best = tree.cost;
                    keep = hist.len();
                }
            }
            while hist.len() > keep {
                tree.undo(hist.pop().unwrap());
            }
        }
        if tree.work >= work_limit || tree.cost == start_cost {
            break;
        }
    }
    if tree.cost < cost {
        let out = tree.output();
        Some(out)
    } else {
        None
    }
}
