//! Connected elimination-tree separators. Hanging branches are represented by
//! their exact boundary cliques. A tree DP combines compatible replacements.
//! Full patch proposals run independently and combine in task order.
use super::patch::{sum_squares, Atoms, Dense, Rng};
use crate::Pattern;
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};

pub(super) struct Proposal {
    pub(super) root: usize,
    pub(super) order: Vec<usize>,
    pub(super) children: Vec<usize>,
    pub(super) cost: u64,
    pub(super) old: u64,
    pub(super) skeleton: Vec<usize>,
}
pub(super) fn charge(work: &mut usize, amount: usize) -> Option<()> {
    *work = work.checked_sub(amount)?;
    Some(())
}
pub(super) fn clique(g: &mut [u64], w: usize, vertices: &[usize], work: &mut usize) -> Option<()> {
    charge(work, vertices.len().checked_mul(w)?)?;
    let mut mask = vec![0u64; w];
    for &v in vertices {
        mask[v / 64] |= 1u64 << (v % 64);
    }
    for &v in vertices {
        for j in 0..w {
            g[v * w + j] |= mask[j];
        }
        g[v * w + v / 64] &= !(1u64 << (v % 64));
    }
    Some(())
}
pub(super) fn greedy(
    g: Vec<u64>,
    n: usize,
    k: usize,
    mode: usize,
    work: &mut usize,
) -> Option<(Vec<usize>, u64)> {
    greedy_options(g, n, k, mode, work, 8, false, usize::MAX)
}
fn greedy_options(
    mut g: Vec<u64>,
    n: usize,
    k: usize,
    mode: usize,
    work: &mut usize,
    shortlist: usize,
    fallback: bool,
    mut replay_work: usize,
) -> Option<(Vec<usize>, u64)> {
    let w = n.div_ceil(64);
    replay_work = replay_work.checked_sub(n * w + n)?;
    let mut degree: Vec<usize> = g
        .chunks(w)
        .map(|r| r.iter().map(|x| x.count_ones() as usize).sum())
        .collect();
    let mut live = vec![true; n];
    let mut mark = vec![usize::MAX; n];
    let mut seed = 918712u64;
    let ties: Vec<u64> = (0..n)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        })
        .collect();
    let mut out = Vec::with_capacity(k);
    let mut cost = 0u64;
    let mut remaining: Vec<usize> = (0..k).collect();
    let mut position: Vec<usize> = (0..k).collect();
    let mut candidates = Vec::with_capacity(k);
    let mut sample_rng = Rng::new(0x74A9_3E12_851B_062D);
    let mut words = Vec::with_capacity(w);
    for step in 0..k {
        if mode == 0 {
            candidates.clear();
            candidates.push(*remaining.iter().min_by_key(|&&v| (degree[v], v))?);
        } else {
            candidates.clone_from(&remaining);
            if candidates.len() > shortlist {
                candidates.select_nth_unstable_by_key(shortlist, |&v| (degree[v], v));
                candidates.truncate(shortlist);
            }
            candidates.sort_unstable_by_key(|&v| (degree[v], v));
        }
        for &v in &candidates {
            mark[v] = step;
        }
        // Global fill modes retain the degree shortlist but also inspect live
        // pivots outside it. Sampling is deterministic and its fill evaluations
        // are charged to the same work budget as the original shortlist.
        if mode >= 5 && *work > 0 {
            if k <= 64 {
                for v in 0..k {
                    if live[v] && mark[v] != step {
                        mark[v] = step;
                        candidates.push(v);
                    }
                }
            } else {
                let target = candidates.len() + 16;
                for _ in 0..64 {
                    let v = sample_rng.next() as usize % k;
                    if live[v] && mark[v] != step {
                        mark[v] = step;
                        candidates.push(v);
                        if candidates.len() == target {
                            break;
                        }
                    }
                }
            }
        }
        let complete = degree[*candidates.first()?] == n - step - 1;
        let mut no_fill = complete;
        let mut best = None;
        let mut best_score = f64::INFINITY;
        for &v in &candidates {
            let d = degree[v];
            if mode == 0 || (fallback && *work == 0) {
                let score = d as f64;
                if score < best_score
                    || (score == best_score && best.is_none_or(|p| ties[v] < ties[p]))
                {
                    best = Some(v);
                    best_score = score;
                    no_fill = complete;
                }
                continue;
            }
            if fallback {
                *work = work.saturating_sub(d.checked_mul(w)?);
            } else {
                charge(work, d.checked_mul(w)?)?;
            }
            let fill = if complete {
                0
            } else {
                super::patch::deficiency(&g, w, v, k, &mut words)
            };
            let score = match mode {
                1 | 5 => fill as f64,
                2 => fill as f64 * (d + 1) as f64,
                3 | 6 => fill as f64 / (d + 1) as f64,
                _ => fill as f64 / ((d + 1) * (d + 1)) as f64,
            };
            if score < best_score || (score == best_score && best.is_none_or(|p| ties[v] < ties[p]))
            {
                best = Some(v);
                best_score = score;
                no_fill = fill == 0;
            }
        }
        let v = best?;
        out.push(v);
        cost = cost.checked_add(((degree[v] + 1) as u64).pow(2))?;
        words.clear();
        words.extend(
            g[v * w..(v + 1) * w]
                .iter()
                .copied()
                .enumerate()
                .filter(|&(_, bits)| bits != 0),
        );
        if fallback {
            *work = work.saturating_sub(degree[v].checked_mul(w)?);
        } else {
            charge(work, degree[v].checked_mul(w)?)?;
        }
        replay_work = replay_work.checked_sub((degree[v] + 1) * (w + 8))?;
        for &(j, mut bits) in &words {
            if j * 64 >= k {
                break;
            }
            while bits != 0 {
                let u = j * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if u >= k {
                    break;
                }
                let added = if no_fill {
                    g[u * w + v / 64] &= !(1 << (v % 64));
                    1
                } else {
                    super::patch::fill_row(&mut g[u * w..(u + 1) * w], &words, u, v) as usize
                };
                degree[u] = (degree[u] + added).checked_sub(2)?;
            }
        }
        live[v] = false;
        let at = position[v];
        remaining.swap_remove(at);
        if at < remaining.len() {
            position[remaining[at]] = at;
        }
        g[v * w..(v + 1) * w].fill(0);
    }
    Some((out, cost))
}

/// Search width of one recovered separator pass. Every field is a fixed count
/// or work budget, never a clock.
#[derive(Clone, Debug)]
pub(super) struct RecoveryCfg {
    /// Number of subtree roots offered patches, by priority.
    pub roots: usize,
    /// Patch vertex caps, in task order.
    pub caps: Vec<usize>,
    /// Roots (by priority rank) that receive patches of cap 2048 or more.
    pub big_cap_roots: usize,
    /// Constrained shortlist greedy objectives evaluated inside each patch.
    pub modes: Vec<usize>,
    /// Scoring budget of each shortlist greedy run before its degree fallback.
    pub greedy_work: usize,
    /// Boundary rotation budget and lift depth inside each patch.
    pub rotation_work: usize,
    pub rotation_depth: usize,
    /// Tasks (in task order) that additionally run a METIS patch candidate.
    pub metis_tasks: usize,
    /// Bitset clique construction budget per patch.
    pub patch_work: usize,
    /// Bound the complete interface, including its frozen exterior, before allocation.
    pub max_interface: usize,
    /// Skeleton membership priorities tried per root (see `Tree::priority`).
    pub skeleton_modes: Vec<usize>,
    /// Native patch orderers: 0 none, 1 AMF, 2 AMF and AMD.
    pub natives: usize,
    /// Patches given the full search, chosen by the exact gain of a cheap
    /// minimum-degree proposal that every patch receives first.
    pub full_tasks: usize,
    /// Dense deficiency-greedy objectives (0..=7) run per patch; 0 disables.
    pub dense_modes: usize,
    /// Exact reinsertion rounds per patch (scaled by patch size); 0 disables.
    pub improve_rounds: usize,
    /// Non-monotone prefix restarts per patch.
    pub restarts: usize,
    /// Twin-quotient search on the boundary-compressed patch.
    pub atoms: bool,
    pub max_atoms: usize,
    /// Cross-level union proposals built per pass; 0 disables.
    pub unions: usize,
    pub directed: bool,
}

impl RecoveryCfg {
    pub(super) fn fast(large_patch: bool) -> RecoveryCfg {
        RecoveryCfg {
            roots: 8,
            caps: if large_patch {
                vec![2048, 128, 512]
            } else {
                vec![128, 512]
            },
            big_cap_roots: 1,
            modes: vec![0, 3],
            greedy_work: 300_000,
            rotation_work: 1_000_000,
            rotation_depth: 64,
            metis_tasks: 2,
            patch_work: 4_000_000,
            max_interface: 4096,
            skeleton_modes: vec![0],
            natives: 2,
            full_tasks: usize::MAX,
            dense_modes: 0,
            improve_rounds: 0,
            restarts: 0,
            atoms: false,
            max_atoms: 600,
            unions: 0,
            directed: false,
        }
    }
}

/// Exact elimination-tree data shared read-only by every patch task.
pub(super) struct Tree<'a> {
    pattern: &'a Pattern,
    /// Postorder: tree position to original vertex.
    p: Vec<usize>,
    /// Original vertex to tree position.
    rank: Vec<usize>,
    pub(super) counts: Vec<u32>,
    parent: Vec<i32>,
    kids: Vec<Vec<usize>>,
    /// Exact later boundary of each subtree, in tree positions.
    bag: Vec<Vec<usize>>,
    size: Vec<usize>,
    /// Subtree cost.
    old: Vec<u64>,
    /// Cost attributable to fill: column cost minus the cost the original
    /// later neighbors alone would give; and its subtree sum.
    avoidable: Vec<u64>,
    subavoidable: Vec<u64>,
}

impl<'a> Tree<'a> {
    fn build(pattern: &'a Pattern, incumbent: &[usize]) -> Option<Tree<'a>> {
        Self::build_limited(pattern, incumbent, 4_000_000usize.max(pattern.n * 64), 0)
    }

    pub(super) fn build_limited(
        pattern: &'a Pattern,
        incumbent: &[usize],
        mut build_work: usize,
        min_cost: u64,
    ) -> Option<Tree<'a>> {
        let n = pattern.n;
        let sp = super::ScoringPattern {
            n,
            col_ptr: pattern.col_ptr.clone(),
            row_idx: pattern.row_idx.clone(),
        };
        let (p, counts, parent) = super::etree_prep(&sp, incumbent);
        if min_cost != 0 && counts.iter().map(|&c| (c as u64).pow(2)).sum::<u64>() < min_cost {
            return None;
        }
        let mut rank = vec![0; n];
        for (j, &v) in p.iter().enumerate() {
            rank[v] = j;
        }
        let mut kids = vec![Vec::new(); n];
        for v in 0..n {
            if parent[v] >= 0 {
                kids[parent[v] as usize].push(v);
            }
        }
        let mut bag: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut size = vec![1usize; n];
        let mut old = vec![0u64; n];
        let mut avoidable = vec![0u64; n];
        let mut subavoidable = vec![0u64; n];
        let mut seen = vec![n; n];
        for v in 0..n {
            let mut b = Vec::with_capacity(counts[v].saturating_sub(1) as usize);
            let mut entries = 0;
            let mut later = 1u64;
            for &u in &pattern.row_idx[pattern.col_ptr[p[v]]..pattern.col_ptr[p[v] + 1]] {
                if rank[u] > v {
                    seen[rank[u]] = v;
                    b.push(rank[u]);
                    entries += 1;
                    later += 1;
                }
            }
            for &ch in &kids[v] {
                charge(&mut build_work, bag[ch].len())?;
                for &u in &bag[ch] {
                    if u != v {
                        entries += 1;
                        if seen[u] != v {
                            seen[u] = v;
                            b.push(u);
                        }
                    }
                }
            }
            charge(&mut build_work, entries)?;
            b.sort_unstable();
            if b.len() + 1 != counts[v] as usize {
                return None;
            }
            let cost = ((b.len() + 1) as u64).pow(2);
            old[v] += cost;
            bag[v] = b;
            avoidable[v] = cost - later * later;
            subavoidable[v] += avoidable[v];
            if parent[v] >= 0 {
                let q = parent[v] as usize;
                size[q] += size[v];
                old[q] += old[v];
                subavoidable[q] += subavoidable[v];
            }
        }
        Some(Tree {
            pattern,
            p,
            rank,
            counts,
            parent,
            kids,
            bag,
            size,
            old,
            avoidable,
            subavoidable,
        })
    }
    fn inside(&self, v: usize, root: usize) -> bool {
        root + 1 - self.size[root] <= v && v <= root
    }
    /// Subtree roots by descending replaced cost, one per nesting cap.
    pub(super) fn roots_by_cap(&self, max_bag: usize, limit: usize) -> Vec<usize> {
        let n = self.p.len();
        let mut seen = vec![false; n];
        let mut roots = Vec::new();
        for cap in [
            64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072,
        ] {
            for v in 0..n {
                if !seen[v]
                    && self.size[v] >= 32
                    && self.size[v] <= cap
                    && self.bag[v].len() <= max_bag
                    && (self.parent[v] < 0 || self.size[self.parent[v] as usize] > cap)
                {
                    seen[v] = true;
                    let cost = self.old[v];
                    let priority =
                        cost as f64 / (1. + self.bag[v].len() as f64 / self.size[v] as f64);
                    roots.push((priority, v));
                }
            }
        }
        roots.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        roots.truncate(limit);
        roots.into_iter().map(|(_, v)| v).collect()
    }
    /// Subtree roots by subtree cost per square-root size, skipping roots
    /// nested inside an already chosen root of similar size; tree roots are
    /// appended.
    fn roots_by_density(&self, max_bag: usize, limit: usize) -> Vec<usize> {
        let n = self.p.len();
        let mut cands: Vec<usize> = (0..n)
            .filter(|&v| {
                self.size[v] >= 12
                    && self.counts[v] >= 2
                    && self.bag[v].len() <= max_bag
                    && self.subavoidable[v] > 0
            })
            .collect();
        let key = |v: usize| self.subavoidable[v] as f64 / (self.size[v] as f64).sqrt();
        cands.sort_by(|&a, &b| key(b).total_cmp(&key(a)).then(a.cmp(&b)));
        let mut roots: Vec<usize> = Vec::new();
        for v in cands {
            let near = roots.iter().any(|&r| {
                (self.inside(v, r) || self.inside(r, v))
                    && self.size[v].abs_diff(self.size[r])
                        < 12.max(self.size[v].min(self.size[r]) / 3)
            });
            if !near {
                roots.push(v);
            }
            if roots.len() >= limit {
                break;
            }
        }
        roots
    }
    fn priority(&self, v: usize, mode: usize, rng: &mut Rng) -> u64 {
        let w = self.counts[v] as u64;
        let size = self.size[v] as u64;
        let noise = |rng: &mut Rng| (rng.next() % 1_000_000) as f64 / 1_000_000.0;
        match mode {
            1 => u64::MAX / 2 - w,
            2 => size * 1_000_000 / (w + 1),
            5 => (self.old[v] as f64 / (size as f64).sqrt() * 16.0) as u64,
            7 => ((self.old[v] as f64).sqrt() * (0.5 + noise(rng)) * 1024.0) as u64,
            9 => self.avoidable[v],
            _ => w * w,
        }
    }
    /// Expose every hidden subtree supporting selected nonoriginal fill edges.
    /// For an ancestor endpoint, membership in a subtree boundary is exactly
    /// the existence of an original edge into that subtree. Descending only
    /// through children with both contacts therefore computes the full support
    /// closure, including overlapping supports from different frontiers.
    fn release_support(&self, mut vertices: Vec<usize>, cap: usize) -> Vec<usize> {
        let mut kept = vec![false; self.p.len()];
        for &v in &vertices {
            kept[v] = true;
        }
        let mut fronts = Vec::new();
        let mut pairs = BTreeSet::new();
        for &v in &vertices {
            for &ch in &self.kids[v] {
                if kept[ch] {
                    continue;
                }
                fronts.push(ch);
                let mut endpoints: Vec<_> =
                    self.bag[ch].iter().copied().filter(|&u| kept[u]).collect();
                endpoints.sort_by_key(|&u| (Reverse(self.counts[u]), u));
                endpoints.truncate(4);
                for (i, &u) in endpoints.iter().enumerate() {
                    for &v in &endpoints[i + 1..] {
                        let original = self.p[u];
                        if self.pattern.row_idx
                            [self.pattern.col_ptr[original]..self.pattern.col_ptr[original + 1]]
                            .binary_search(&self.p[v])
                            .is_err()
                        {
                            pairs.insert((u.min(v), u.max(v)));
                        }
                    }
                }
            }
        }
        let mut pairs: Vec<_> = pairs.into_iter().collect();
        pairs.sort_by_key(|&(u, v)| (Reverse(self.counts[u] as u64 + self.counts[v] as u64), u, v));
        pairs.truncate(64);
        let mut work = 100_000usize;
        let mut closures = Vec::new();
        for (u, v) in pairs {
            let mut stack = fronts.clone();
            let mut closure = Vec::new();
            let mut complete = true;
            while let Some(x) = stack.pop() {
                if charge(&mut work, 1 + self.bag[x].len().max(1).ilog2() as usize).is_none() {
                    complete = false;
                    break;
                }
                if self.bag[x].binary_search(&u).is_err() || self.bag[x].binary_search(&v).is_err()
                {
                    continue;
                }
                closure.push(x);
                if closure.len() > cap - vertices.len() {
                    complete = false;
                    break;
                }
                stack.extend(self.kids[x].iter().copied());
            }
            if complete && !closure.is_empty() {
                closures.push((self.counts[u] as u64 + self.counts[v] as u64 + 1, closure));
            }
            if work == 0 {
                break;
            }
        }
        while vertices.len() < cap {
            let mut best = None;
            let mut best_score = 0u64;
            for (i, (gain, closure)) in closures.iter().enumerate() {
                let fresh = closure.iter().filter(|&&v| !kept[v]).count();
                if fresh == 0 || fresh > cap - vertices.len() {
                    continue;
                }
                let score = gain * 1_000_000 / fresh as u64;
                if score > best_score {
                    best_score = score;
                    best = Some(i);
                }
            }
            let Some(i) = best else {
                break;
            };
            for &v in &closures[i].1 {
                if !kept[v] {
                    kept[v] = true;
                    vertices.push(v);
                }
            }
        }
        vertices.sort_unstable();
        vertices
    }

    /// Ancestor-closed patch of at most `cap` vertices below `root` grown by
    /// `mode` priority, sorted by tree position.
    pub(super) fn grow(&self, root: usize, cap: usize, mode: usize) -> Vec<usize> {
        if mode == 11 {
            let seed = self.grow(root, (cap / 2).max(1), 0);
            return self.release_support(seed, cap);
        }
        if self.size[root] <= cap {
            return (root + 1 - self.size[root]..=root).collect();
        }
        let mut rng =
            Rng::new(0x9E37_79B9_7F4A_7C15 ^ (root as u64 * 1_000_003) ^ ((mode as u64) << 40));
        let mut heap = BinaryHeap::new();
        heap.push((self.priority(root, mode, &mut rng), root));
        let mut vertices = Vec::new();
        while vertices.len() < cap {
            let Some((_, v)) = heap.pop() else { break };
            vertices.push(v);
            for &ch in &self.kids[v] {
                heap.push((self.priority(ch, mode, &mut rng), ch));
            }
        }
        vertices.sort_unstable();
        vertices
    }
    /// Compressed bitset graph of an ancestor-closed patch: original edges,
    /// one clique per distinct hanging interface, and the exterior clique.
    /// Returns (hanging roots, graph, total vertex count) or `None` when the
    /// patch is invalid, over budget, or already fill-free.
    fn compress_reusing_map(
        &self,
        root: usize,
        vertices: &[usize],
        work: &mut usize,
        map: &mut Vec<usize>,
    ) -> Option<(Vec<usize>, Vec<u64>, usize)> {
        map.resize(self.p.len(), usize::MAX);
        self.compress(root, vertices, map, work)
    }

    pub(super) fn compress(
        &self,
        root: usize,
        vertices: &[usize],
        map: &mut [usize],
        work: &mut usize,
    ) -> Option<(Vec<usize>, Vec<u64>, usize)> {
        let k = vertices.len();
        let mut all = vertices.to_vec();
        all.extend_from_slice(&self.bag[root]);
        let total = all.len();
        let w = total.div_ceil(64);
        for (j, &v) in all.iter().enumerate() {
            map[v] = j;
        }
        let mut g = vec![0u64; total * w];
        let mut children = Vec::new();
        let mut valid = true;
        let mut interfaces = BTreeSet::new();
        let pattern = self.pattern;
        for (j, &v) in vertices.iter().enumerate() {
            for &u in &pattern.row_idx[pattern.col_ptr[self.p[v]]..pattern.col_ptr[self.p[v] + 1]] {
                let a = map[self.rank[u]];
                if a != usize::MAX {
                    g[j * w + a / 64] |= 1u64 << (a % 64);
                    g[a * w + j / 64] |= 1u64 << (j % 64);
                }
            }
            for &ch in &self.kids[v] {
                if map[ch] >= k {
                    children.push(ch);
                    let b: Vec<usize> = self.bag[ch].iter().map(|&u| map[u]).collect();
                    if b.iter().any(|&u| u == usize::MAX)
                        || (interfaces.insert(b.clone()) && clique(&mut g, w, &b, work).is_none())
                    {
                        valid = false;
                        break;
                    }
                }
            }
            if !valid {
                break;
            }
        }
        if valid {
            valid = clique(&mut g, w, &(k..total).collect::<Vec<_>>(), work).is_some();
        }
        for &v in &all {
            map[v] = usize::MAX;
        }
        if !valid {
            return None;
        }
        let exterior = (total - k) as u64;
        let filled_nnz = vertices.iter().map(|&v| self.counts[v] as u64).sum::<u64>()
            + exterior * (exterior + 1) / 2;
        let edges = g.iter().map(|v| v.count_ones() as u64).sum::<u64>() / 2;
        if filled_nnz == total as u64 + edges {
            return None;
        }
        children.sort_unstable();
        Some((children, g, total))
    }
    /// Cost the incumbent pays inside the patch.
    pub(super) fn original(&self, vertices: &[usize]) -> u64 {
        vertices
            .iter()
            .map(|&v| (self.counts[v] as u64).pow(2))
            .sum()
    }
}

pub(super) fn csc_of(g: &[u64], total: usize) -> (Vec<usize>, Vec<usize>) {
    let w = total.div_ceil(64);
    let mut cp = vec![0usize];
    let mut ri = Vec::new();
    for v in 0..total {
        for j in 0..w {
            let mut bits = g[v * w + j];
            while bits != 0 {
                ri.push(j * 64 + bits.trailing_zeros() as usize);
                bits &= bits - 1;
            }
        }
        cp.push(ri.len());
    }
    (cp, ri)
}

/// Bounded minimum-degree screening with a separate full-search priority.
fn propose_cheap(
    tree: &Tree,
    root: usize,
    vertices: &[usize],
    cfg: &RecoveryCfg,
    allowance: usize,
    map: &mut Vec<usize>,
) -> Option<(Option<Proposal>, u64, (Vec<usize>, Vec<u64>, usize))> {
    if vertices.len() + tree.bag[root].len() > cfg.max_interface {
        return None;
    }
    let mut work = cfg.patch_work.min(allowance);
    let (children, g, total) = tree.compress_reusing_map(root, vertices, &mut work, map)?;
    let prepared = (children.clone(), g.clone(), total);
    let k = vertices.len();
    let original = tree.original(vertices);
    let priority = if cfg.directed {
        // Every completion adds edges/triangles. Subtracting the fixed exterior
        // clique leaves a lower bound on this patch's exact objective.
        original.saturating_sub(patch_potential(&g, total, k, &mut work))
            / (k * total.div_ceil(64)).max(1) as u64
    } else {
        0
    };
    let Some((order, cost)) = greedy_options(g, total, k, 0, &mut 0usize, 1, true, work) else {
        return Some((None, priority, prepared));
    };
    let gain = if cfg.directed {
        priority
    } else {
        original.saturating_sub(cost)
    };
    Some((
        Some(Proposal {
            root,
            order: order.iter().map(|&j| vertices[j]).collect(),
            children,
            cost,
            old: original,
            skeleton: Vec::new(),
        }),
        gain,
        prepared,
    ))
}

/// Lower bound from counted edges and triangles, excluding the exterior clique.
/// A partial count remains a lower bound when the work allowance is exhausted.
fn patch_potential(g: &[u64], n: usize, k: usize, work: &mut usize) -> u64 {
    let w = n.div_ceil(64);
    let mut edges = 0u64;
    let mut triangles = 0u64;
    for v in 0..k {
        if charge(work, w).is_none() {
            break;
        }
        let row = &g[v * w..(v + 1) * w];
        let mut exterior = 0u64;
        for word in v / 64..w {
            let mut higher = row[word];
            if word == v / 64 {
                higher &= if v % 64 == 63 {
                    0
                } else {
                    u64::MAX << (v % 64 + 1)
                };
            }
            edges += higher.count_ones() as u64;
            let free = if word < k / 64 {
                u64::MAX
            } else if word == k / 64 {
                (1u64 << (k % 64)) - 1
            } else {
                0
            };
            exterior += (higher & !free).count_ones() as u64;
            let mut inside = higher & free;
            while inside != 0 {
                let u = word * 64 + inside.trailing_zeros() as usize;
                inside &= inside - 1;
                if charge(work, w - u / 64).is_none() {
                    return k as u64 + 3 * edges + 2 * triangles;
                }
                for j in u / 64..w {
                    let mut common = row[j] & g[u * w + j];
                    if j == u / 64 {
                        common &= if u % 64 == 63 {
                            0
                        } else {
                            u64::MAX << (u % 64 + 1)
                        };
                    }
                    triangles += common.count_ones() as u64;
                }
            }
        }
        triangles += exterior * exterior.saturating_sub(1) / 2;
    }
    k as u64 + 3 * edges + 2 * triangles
}

/// Full patch proposal on an explicit ancestor-closed skeleton: native
/// orderers, twin-quotient search, deficiency greedy objectives, exact
/// reinsertion, restarts, shortlist greedy and boundary rotations; exact local
/// cost. `seed` fixes every tie-break.
fn propose(
    tree: &Tree,
    root: usize,
    vertices: &[usize],
    metis: bool,
    seed: u64,
    cfg: &RecoveryCfg,
    prepared: Option<&(Vec<usize>, Vec<u64>, usize)>,
    map: &mut Vec<usize>,
) -> Option<Proposal> {
    if vertices.len() + tree.bag[root].len() > cfg.max_interface {
        return None;
    }
    let mut work = cfg.patch_work;
    let owned;
    let (children, g, total) = match prepared {
        Some(value) => value,
        None => {
            owned = tree.compress_reusing_map(root, vertices, &mut work, map)?;
            &owned
        }
    };
    let total = *total;
    let k = vertices.len();
    let original = tree.original(vertices);
    let mut best: Option<(Vec<usize>, u64)> = None;
    let consider = |order: Vec<usize>, cost: u64, best: &mut Option<(Vec<usize>, u64)>| {
        if cost < best.as_ref().map_or(original, |b| b.1) {
            *best = Some((order, cost));
        }
    };
    let (cp, ri) = csc_of(&g, total);
    if ri.len() > 1_000_000 {
        return None;
    }
    let cc: Vec<i32> = cp.iter().map(|&x| x as i32).collect();
    let rr: Vec<i32> = ri.iter().map(|&x| x as i32).collect();
    let graph = feral_ordering_core::CscPattern::new(total, &cc, &rr)?;
    let sp = super::ScoringPattern {
        n: total,
        col_ptr: cp,
        row_idx: ri,
    };
    let b = (total - k) as u64;
    let mut candidates = Vec::new();
    if cfg.natives >= 1 {
        candidates.push(
            feral_amf::amf_order_opts(&graph, &feral_amf::AmfOptions::default()).map(|(p, ..)| p),
        );
    }
    if cfg.natives >= 2 {
        candidates.push(feral_amd::amd_order(&graph));
    }
    if metis {
        let options = feral_metis::MetisOptions {
            niparts: 4,
            ..Default::default()
        };
        candidates.push(feral_metis::metis_order_full(&graph, &options).map(|(p, ..)| p));
    }
    let mut seen = Vec::new();
    for result in candidates {
        if let Ok(order) = result {
            let inside: Vec<usize> = order
                .into_iter()
                .map(|v| v as usize)
                .filter(|&v| v < k)
                .collect();
            if !super::is_bijection(&inside, k) {
                continue;
            }
            if seen.contains(&inside) {
                continue;
            }
            seen.push(inside.clone());
            let mut full = inside.clone();
            full.extend(k..total);
            let cost = super::flops_of(&sp, &full).checked_sub(sum_squares(b))?;
            consider(inside, cost, &mut best);
        }
    }
    let dense = Dense::from_bits(g.clone(), total, k);
    let mut rng = Rng::new(seed);
    // Twin-quotient search when the boundary-compressed interface compresses
    // by at least a fifth: incumbent projection, weighted greedy, reinsertion.
    let mut compressed = false;
    let atoms = cfg.atoms.then(|| Atoms::new(&dense));
    if let Some(atoms) = &atoms {
        if atoms.k <= cfg.max_atoms && atoms.n <= 384 && atoms.k * 5 < k * 4 {
            compressed = atoms.k * 5 < k * 4;
            let mut ao = atoms.project(&(0..k).collect::<Vec<_>>());
            let mut ac = atoms.score(&ao);
            for mode in [0, 3, 6] {
                let (c, o) = atoms.greedy(mode, rng.next());
                if c < ac {
                    ac = c;
                    ao = o;
                }
            }
            if let Some((cost, order)) = atoms.joint(&ao) {
                if cost < ac {
                    ac = cost;
                    ao = order;
                }
            }
            if cfg.improve_rounds > 0 {
                let rounds = (atoms.k * 2).min(if atoms.k > 150 { 40 } else { 250 });
                let (c, o) = atoms.improve(ao, rounds, rng.next());
                ac = c;
                ao = o;
            }
            consider(atoms.expand(&ao), ac, &mut best);
        }
    }
    let dense_modes = if compressed {
        cfg.dense_modes.min(1)
    } else if k > 256 {
        cfg.dense_modes.min(3)
    } else {
        cfg.dense_modes
    };
    let mut dense_work = 8_000_000usize;
    for mode in 0..dense_modes {
        if let Some((c, o)) = dense.greedy(mode, rng.next(), &mut dense_work) {
            consider(o, c, &mut best);
        }
    }
    for &mode in cfg.modes.iter().filter(|_| k <= 512) {
        let mut local_work = cfg.greedy_work;
        if let Some((order, cost)) = greedy_options(
            g.clone(),
            total,
            k,
            mode,
            &mut local_work,
            12,
            true,
            4_000_000,
        ) {
            consider(order, cost, &mut best);
        }
    }
    if cfg.improve_rounds > 0 && k <= 600 {
        let start = best
            .as_ref()
            .map_or_else(|| (0..k).collect::<Vec<_>>(), |b| b.0.clone());
        let rounds = (k * 2)
            .min(30_000_000 / (k * total * dense.w).max(1))
            .min(cfg.improve_rounds * k);
        let seed = rng.next();
        if rounds > 0 {
            let (c, o) = dense.improve(start, rounds, seed);
            consider(o, c, &mut best);
        }
    }
    for it in 0..cfg.restarts {
        let start = best
            .as_ref()
            .map_or_else(|| (0..k).collect::<Vec<_>>(), |b| b.0.clone());
        let prefix = (rng.next() % k as u64) as usize;
        if let Some((c, o)) = dense.restart(&start, prefix, it % 8, rng.next(), &mut dense_work) {
            consider(o, c, &mut best);
        }
    }
    if let Some(atoms) = atoms.as_ref().filter(|_| k <= 256) {
        let mut residual_work = cfg.greedy_work;
        for tail in [24, 12] {
            let start = best
                .as_ref()
                .map_or_else(|| (0..k).collect(), |b| b.0.clone());
            if let Some((cost, order)) =
                dense.residual_order(&start, tail, &mut residual_work, atoms)
            {
                consider(order, cost, &mut best);
            }
        }
    }
    if cfg.rotation_work > 0 && k <= 512 {
        let mut initial = best
            .as_ref()
            .map_or_else(|| (0..k).collect::<Vec<_>>(), |b| b.0.clone());
        initial.extend(k..total);
        let local_pattern = Pattern {
            n: total,
            col_ptr: sp.col_ptr.clone(),
            row_idx: sp.row_idx.clone(),
        };
        if let Some(order) = super::promotions::boundary_with_budget(
            &local_pattern,
            &initial,
            k,
            cfg.rotation_work,
            cfg.rotation_depth,
        ) {
            if order[..k].iter().all(|&v| v < k) {
                let cost = super::flops_of(&sp, &order) - sum_squares(b);
                consider(order[..k].to_vec(), cost, &mut best);
            }
        }
    }
    let (order, cost) = best?;
    Some(Proposal {
        root,
        order: order.iter().map(|&j| vertices[j]).collect(),
        children: children.clone(),
        cost,
        old: original,
        skeleton: vertices.to_vec(),
    })
}

/// Bottom-up choice between each subtree's incumbent and its compatible
/// proposals, then the replaced postorder.
pub(super) fn assemble(tree: &Tree, proposals: &[Vec<Proposal>]) -> Option<Vec<usize>> {
    let n = tree.p.len();
    let mut dp = vec![0u64; n];
    let mut take = vec![None; n];
    let mut any = false;
    for v in 0..n {
        dp[v] = (tree.counts[v] as u64).pow(2) + tree.kids[v].iter().map(|&ch| dp[ch]).sum::<u64>();
        for (j, pr) in proposals[v].iter().enumerate() {
            let cost = pr.cost + pr.children.iter().map(|&ch| dp[ch]).sum::<u64>();
            if cost < dp[v] {
                dp[v] = cost;
                take[v] = Some(j);
                any = true;
            }
        }
    }
    if !any {
        return None;
    }
    let mut stack = Vec::new();
    for v in (0..n).rev() {
        if tree.parent[v] < 0 {
            stack.push((v, false));
        }
    }
    let mut out = Vec::with_capacity(n);
    while let Some((v, exit)) = stack.pop() {
        if exit {
            if let Some(j) = take[v] {
                out.extend(proposals[v][j].order.iter().map(|&u| tree.p[u]));
            } else {
                out.push(tree.p[v]);
            }
            continue;
        }
        stack.push((v, true));
        let children = take[v].map_or(&tree.kids[v], |j| &proposals[v][j].children);
        for &ch in children.iter().rev() {
            stack.push((ch, false));
        }
    }
    if out.len() == n {
        Some(out)
    } else {
        None
    }
}

/// Cross-level unions: for an improving proposal `p` and an improving
/// proposal `q` rooted inside `p`'s subtree, the union of both skeletons plus
/// the ancestor path from `q`'s root into `p`'s skeleton is again an
/// ancestor-closed skeleton of `p`'s root. Returns the unions to evaluate,
/// largest combined gain first, at most `limit`, at most three per root.
fn union_tasks(
    tree: &Tree,
    proposals: &[Proposal],
    max_k: usize,
    limit: usize,
) -> Vec<(usize, Vec<usize>)> {
    let mut pairs = Vec::new();
    for (i, p) in proposals.iter().enumerate() {
        if p.cost >= p.old {
            continue;
        }
        for (j, q) in proposals.iter().enumerate() {
            if i == j || q.cost >= q.old || p.root == q.root || !tree.inside(q.root, p.root) {
                continue;
            }
            pairs.push(((p.old - p.cost) + (q.old - q.cost), i, j));
        }
    }
    pairs.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut seen: BTreeSet<(usize, Vec<usize>)> = BTreeSet::new();
    let mut used: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
    let mut out = Vec::new();
    for (_, i, j) in pairs {
        if out.len() >= limit {
            break;
        }
        let p = &proposals[i];
        let q = &proposals[j];
        if *used.get(&p.root).unwrap_or(&0) >= 3 {
            continue;
        }
        let mut k_set: Vec<usize> = p.skeleton.clone();
        k_set.extend_from_slice(&q.skeleton);
        let mut v = q.root as i32;
        while v >= 0 && p.skeleton.binary_search(&(v as usize)).is_err() {
            k_set.push(v as usize);
            v = tree.parent[v as usize];
        }
        k_set.sort_unstable();
        k_set.dedup();
        if k_set.len() > max_k * 3 || k_set.len() == p.skeleton.len() {
            continue;
        }
        if !seen.insert((p.root, k_set.clone())) {
            continue;
        }
        *used.entry(p.root).or_insert(0) += 1;
        out.push((p.root, k_set));
    }
    out
}

/// Separator proposals, optional screening, cross-level unions and exact DP assembly.
pub(super) fn recovered(
    pattern: &Pattern,
    incumbent: &[usize],
    cfg: &RecoveryCfg,
) -> Option<Vec<usize>> {
    let n = pattern.n;
    if !(64..=320_000).contains(&n) || pattern.nnz() > 1_500_000 {
        return None;
    }
    let tree = Tree::build(pattern, incumbent)?;
    let roots = if cfg.directed {
        tree.roots_by_density(1024, cfg.roots)
    } else {
        tree.roots_by_cap(1024, cfg.roots)
    };
    // A subtree no larger than the cap is the same patch under every priority.
    let mut tasks: Vec<(usize, Vec<usize>)> = Vec::new();
    let mut support_tasks = Vec::new();
    let mut seen: BTreeSet<(usize, Vec<usize>)> = BTreeSet::new();
    for &cap in &cfg.caps {
        for (root_number, &root) in roots.iter().enumerate() {
            if cap >= 2048 && root_number >= cfg.big_cap_roots {
                continue;
            }
            for &mode in &cfg.skeleton_modes {
                if mode == 11 && (root_number >= 2 || cap > 512) {
                    continue;
                }
                if mode != 0 && tree.size[root] <= cap {
                    continue;
                }
                let vertices = tree.grow(root, cap, mode);
                if mode == 11 {
                    support_tasks.push((root, vertices));
                    continue;
                }
                if seen.insert((root, vertices.clone())) {
                    tasks.push((root, vertices));
                }
            }
        }
    }
    // Append new membership proposals so the seeds and METIS allocation of
    // existing tasks are unaffected by enabling support release.
    for task in support_tasks {
        if seen.insert(task.clone()) {
            tasks.push(task);
        }
    }
    let mut proposals: Vec<Vec<Proposal>> = (0..n).map(|_| Vec::new()).collect();
    let mut prepared = vec![None; tasks.len()];
    let full: Vec<usize> = if cfg.full_tasks >= tasks.len() {
        (0..tasks.len()).collect()
    } else {
        // Preserve root coverage: cheap degree ordering alone can miss patches
        // whose gains need a different completion. Reserve half the solves for
        // the first viable patch of distinct roots, then rank by measured gain.
        let allowance = 64_000_000 / tasks.len().max(1);
        let cheap = super::parallel::map(&tasks, 1, |_, (root, vertices), map| {
            propose_cheap(&tree, *root, vertices, cfg, allowance, map)
        });
        let mut ranked: Vec<(u64, usize)> = Vec::new();
        let mut covered = BTreeSet::new();
        let mut selected = Vec::new();
        for (index, ((root, _), result)) in tasks.iter().zip(cheap).enumerate() {
            if let Some((pr, gain, graph)) = result {
                prepared[index] = Some(graph);
                if let Some(pr) = pr {
                    proposals[*root].push(pr);
                }
                ranked.push((gain, index));
                if !cfg.directed
                    && selected.len() < cfg.full_tasks.div_ceil(2)
                    && covered.insert(*root)
                {
                    selected.push(index);
                }
            }
        }
        ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        if cfg.directed {
            for &(priority, index) in &ranked {
                if selected.len() == cfg.full_tasks {
                    break;
                }
                if priority > 0 && covered.insert(tasks[index].0) {
                    selected.push(index);
                }
            }
        }
        for (priority, index) in ranked {
            if selected.len() == cfg.full_tasks {
                break;
            }
            if cfg.directed && priority == 0 {
                continue;
            }
            if !selected.contains(&index) {
                selected.push(index);
            }
        }
        selected.sort_unstable();
        selected
    };
    for (index, graph) in prepared.iter_mut().enumerate() {
        if full.binary_search(&index).is_err() {
            *graph = None;
        }
    }
    let threads = if full.iter().map(|&i| tasks[i].1.len()).sum::<usize>() >= 512 {
        4
    } else {
        1
    };
    let results = super::parallel::map(&full, threads, |rank, &index, map| {
        let (root, vertices) = &tasks[index];
        propose(
            &tree,
            *root,
            vertices,
            rank < cfg.metis_tasks,
            0x5851_F42D_4C95_7F2D ^ (index as u64),
            cfg,
            prepared[index].as_ref(),
            map,
        )
    });
    let mut accepted: Vec<Proposal> = results.into_iter().flatten().collect();
    if cfg.unions > 0 {
        let max_k = cfg.caps.iter().copied().max().unwrap_or(512);
        let unions = union_tasks(&tree, &accepted, max_k, cfg.unions);
        let joint = super::parallel::map(&unions, threads, |index, (root, vertices), map| {
            propose(
                &tree,
                *root,
                vertices,
                false,
                0x2545_F491_4F6C_DD1D ^ (index as u64),
                cfg,
                None,
                map,
            )
        });
        accepted.extend(joint.into_iter().flatten());
    }
    for pr in accepted {
        if pr.cost < pr.old {
            proposals[pr.root].push(pr);
        }
    }
    assemble(&tree, &proposals)
}

/// Small sequential pass: twelve roots, caps 512/1024/256, four constrained
/// greedy objectives sharing one 24M scoring budget.
pub(super) fn refine(pattern: &Pattern, incumbent: &[usize]) -> Option<Vec<usize>> {
    let n = pattern.n;
    if !(64..=80_000).contains(&n) || pattern.nnz() > 500_000 {
        return None;
    }
    let tree = Tree::build(pattern, incumbent)?;
    let roots = tree.roots_by_cap(1024, 12);
    let mut proposals: Vec<Vec<Proposal>> = (0..n).map(|_| Vec::new()).collect();
    let mut work = 24_000_000usize;
    let mut map = vec![usize::MAX; n];
    for &cap in &[512usize, 1024, 256] {
        for &root in &roots {
            if work < 10000 {
                break;
            }
            let vertices = tree.grow(root, cap, 0);
            let Some((children, g, total)) = tree.compress(root, &vertices, &mut map, &mut work)
            else {
                continue;
            };
            let k = vertices.len();
            let original = tree.original(&vertices);
            let mut best: Option<(Vec<usize>, u64)> = None;
            for mode in [3usize, 1, 2, 4] {
                let Some((order, cost)) = greedy(g.clone(), total, k, mode, &mut work) else {
                    break;
                };
                if cost < best.as_ref().map_or(original, |b| b.1) {
                    best = Some((order, cost));
                }
            }
            if let Some((order, cost)) = best {
                proposals[root].push(Proposal {
                    root,
                    order: order.iter().map(|&j| vertices[j]).collect(),
                    children,
                    cost,
                    old: original,
                    skeleton: vertices,
                });
            }
        }
    }
    assemble(&tree, &proposals)
}
