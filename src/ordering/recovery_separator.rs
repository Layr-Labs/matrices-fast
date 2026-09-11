//! Connected elimination-tree separators. Hanging branches are represented by
//! their exact boundary cliques. A tree DP combines compatible replacements.
//! Every patch proposal is a pure function of the patch and its task index, so
//! proposals are computed on a fixed-width thread pool and merged in task order.
use super::patch::{Atoms, Dense, Rng};
use crate::Pattern;
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};

thread_local! {
    static PATCH_MAP: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
}

struct Proposal {
    root: usize,
    order: Vec<usize>,
    children: Vec<usize>,
    cost: u64,
    old: u64,
    skeleton: Vec<usize>,
    joint: bool,
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
fn sum_squares(t: u64) -> u64 {
    super::patch::sum_squares(t)
}
pub(super) fn greedy(
    g: Vec<u64>,
    n: usize,
    k: usize,
    mode: usize,
    work: &mut usize,
) -> Option<(Vec<usize>, u64)> {
    greedy_options(g, n, k, mode, work, 8, false)
}
fn greedy_options(
    mut g: Vec<u64>,
    n: usize,
    k: usize,
    mode: usize,
    work: &mut usize,
    shortlist: usize,
    fallback: bool,
) -> Option<(Vec<usize>, u64)> {
    let w = n.div_ceil(64);
    let mut degree: Vec<usize> = g
        .chunks(w)
        .map(|r| r.iter().map(|x| x.count_ones() as usize).sum())
        .collect();
    let mut live = vec![true; n];
    let mut mark = vec![usize::MAX; n];
    let mut heap = BinaryHeap::new();
    for (v, &d) in degree.iter().enumerate().take(k) {
        heap.push(Reverse((d, v)));
    }
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
    let mut candidates = Vec::with_capacity(shortlist);
    let mut sample_rng = Rng::new(0x74A9_3E12_851B_062D);
    let mut neighbors = vec![0u64; w];
    for step in 0..k {
        candidates.clear();
        while candidates.len() < if mode == 0 { 1 } else { shortlist } {
            let Some(Reverse((d, v))) = heap.pop() else {
                break;
            };
            if live[v] && degree[v] == d && mark[v] != step {
                mark[v] = step;
                candidates.push(v);
            }
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
                }
                continue;
            }
            if fallback {
                *work = work.saturating_sub(d.checked_mul(w)?);
            } else {
                charge(work, d.checked_mul(w)?)?;
            }
            let row = &g[v * w..(v + 1) * w];
            let mut twice = 0usize;
            for j in 0..w {
                let mut bits = row[j];
                while bits != 0 {
                    let u = j * 64 + bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    for t in 0..w {
                        twice += (g[u * w + t] & row[t]).count_ones() as usize;
                    }
                }
            }
            let fill = d.checked_mul(d.saturating_sub(1))?.checked_sub(twice)? / 2;
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
            }
        }
        let v = best?;
        for &u in &candidates {
            if u != v {
                heap.push(Reverse((degree[u], u)));
            }
        }
        out.push(v);
        cost = cost.checked_add(((degree[v] + 1) as u64).pow(2))?;
        neighbors.copy_from_slice(&g[v * w..(v + 1) * w]);
        if fallback {
            *work = work.saturating_sub(degree[v].checked_mul(w)?);
        } else {
            charge(work, degree[v].checked_mul(w)?)?;
        }
        for j in 0..w {
            let mut bits = neighbors[j];
            while bits != 0 {
                let u = j * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                let mut added = 0usize;
                for t in 0..w {
                    added += (neighbors[t] & !g[u * w + t]).count_ones() as usize;
                    g[u * w + t] |= neighbors[t];
                }
                g[u * w + u / 64] &= !(1u64 << (u % 64));
                g[u * w + v / 64] &= !(1u64 << (v % 64));
                degree[u] = (degree[u] + added).checked_sub(2)?;
                if u < k {
                    heap.push(Reverse((degree[u], u)));
                }
            }
        }
        live[v] = false;
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
    pub rotation_passes: usize,
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
    pub pool_sources: bool,
    pub bag_search: bool,
    pub prefix_grid: bool,
    /// Cross-level union proposals built per pass; 0 disables.
    pub unions: usize,
    /// Root selection: 0 nested caps, 1 cost density, 2 fill cost, 3 fill
    /// density, 4 fill density with exact patch-potential scheduling.
    pub root_select: usize,
    /// Worker threads for independent patches.
    pub threads: usize,
}

pub(super) const THREADS: usize = 8;

impl RecoveryCfg {
    /// The validated two-round schedule: eight roots, one 2048 patch, modes 0 and 3.
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
            rotation_passes: 1,
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
            pool_sources: false,
            bag_search: false,
            prefix_grid: false,
            unions: 0,
            root_select: 0,
            threads: THREADS,
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
    counts: Vec<u32>,
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
        let n = pattern.n;
        let sp = super::ScoringPattern {
            n,
            col_ptr: pattern.col_ptr.clone(),
            row_idx: pattern.row_idx.clone(),
        };
        let (p, counts, parent) = super::etree_prep(&sp, incumbent);
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
        let mut build_work = 4_000_000usize.max(n * 64);
        for v in 0..n {
            let mut b = Vec::new();
            let mut later = 1u64;
            for &u in &pattern.row_idx[pattern.col_ptr[p[v]]..pattern.col_ptr[p[v] + 1]] {
                if rank[u] > v {
                    b.push(rank[u]);
                    later += 1;
                }
            }
            for &ch in &kids[v] {
                charge(&mut build_work, bag[ch].len())?;
                b.extend(bag[ch].iter().copied().filter(|&u| u != v));
            }
            charge(&mut build_work, b.len())?;
            b.sort_unstable();
            b.dedup();
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
    fn roots_by_cap(&self, max_bag: usize, limit: usize) -> Vec<usize> {
        self.roots_by_cost(max_bag, limit, false)
    }

    fn roots_by_cost(&self, max_bag: usize, limit: usize, fill_only: bool) -> Vec<usize> {
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
                    let cost = if fill_only {
                        self.subavoidable[v]
                    } else {
                        self.old[v]
                    };
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
    fn roots_by_density(&self, max_bag: usize, limit: usize, avoidable: bool) -> Vec<usize> {
        let n = self.p.len();
        let mut cands: Vec<usize> = (0..n)
            .filter(|&v| self.size[v] >= 12 && self.counts[v] >= 2 && self.bag[v].len() <= max_bag
                && (!avoidable || self.subavoidable[v] > 0))
            .collect();
        let key = |v: usize| (if avoidable { self.subavoidable[v] } else { self.old[v] }) as f64
            / (self.size[v] as f64).sqrt();
        cands.sort_by(|&a, &b| key(b).total_cmp(&key(a)).then(a.cmp(&b)));
        let mut roots: Vec<usize> = Vec::new();
        for v in cands {
            let near = roots.iter().any(|&r| {
                (self.inside(v, r) || self.inside(r, v))
                    && self.size[v].abs_diff(self.size[r])
                        < if avoidable { 12.max(self.size[v].min(self.size[r]) / 3) }
                          else { 10.max(self.size[v].min(self.size[r]) / 5) }
            });
            if !near {
                roots.push(v);
            }
            if roots.len() >= limit {
                break;
            }
        }
        if avoidable { return roots; }
        for v in 0..n {
            if self.parent[v] < 0
                && self.size[v] >= 12
                && self.bag[v].len() <= max_bag
                && !roots.contains(&v)
            {
                roots.push(v);
            }
        }
        roots.truncate(limit + 4);
        roots
    }
    /// Skeleton membership priority. Mode 0 grows the patch through the widest
    /// boundaries; 1 prefers narrow frontiers; 2 prefers large subtrees per
    /// boundary vertex; 4 prefers subtree mass; 5 subtree cost per square-root
    /// size; 6 subtree cost per vertex; 7 and 8 add fixed-seed noise to subtree
    /// and column cost; 9 fill-attributable cost; 10 its subtree density.
    /// Distinct skeletons expose distinct exact search regions.
    fn priority(&self, v: usize, mode: usize, rng: &mut Rng) -> u64 {
        let w = self.counts[v] as u64;
        let size = self.size[v] as u64;
        let pw = if self.parent[v] >= 0 {
            self.counts[self.parent[v] as usize] as u64
        } else {
            w
        };
        let noise = |rng: &mut Rng| (rng.next() % 1_000_000) as f64 / 1_000_000.0;
        match mode {
            1 => u64::MAX / 2 - w,
            2 => size * 1_000_000 / (w + 1),
            3 => w * w + (w * w).saturating_sub(pw * pw) * 16,
            4 => size * 1000 + w * w,
            5 => (self.old[v] as f64 / (size as f64).sqrt() * 16.0) as u64,
            6 => self.old[v] * 16 / size,
            7 => ((self.old[v] as f64).sqrt() * (0.5 + noise(rng)) * 1024.0) as u64,
            8 => ((w * w) as f64 * (0.25 + 2.0 * noise(rng)) * 1024.0) as u64,
            9 => self.avoidable[v],
            10 => (self.subavoidable[v] as f64 / (size as f64).sqrt() * 16.0) as u64,
            12 => self.subavoidable[v] * 16 / size,
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
    fn grow(&self, root: usize, cap: usize, mode: usize) -> Vec<usize> {
        if mode == 11 {
            let seed = self.grow(root, (cap / 2).max(1), 0);
            return self.release_support(seed, cap);
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
        &self, root: usize, vertices: &[usize], work: &mut usize,
    ) -> Option<(Vec<usize>, Vec<u64>, usize)> {
        PATCH_MAP.with_borrow_mut(|map| {
            map.resize(self.p.len(), usize::MAX);
            // compress clears every touched entry, including rejected patches.
            self.compress(root, vertices, map, work)
        })
    }

    fn compress(
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
    fn original(&self, vertices: &[usize]) -> u64 {
        vertices
            .iter()
            .map(|&v| (self.counts[v] as u64).pow(2))
            .sum()
    }
}

fn csc_of(g: &[u64], total: usize) -> (Vec<usize>, Vec<usize>) {
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

#[cfg(test)]
pub(super) static PROFILE_NANOS: [std::sync::atomic::AtomicU64; 4] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 4];
#[cfg(test)]
fn profile_add(slot: usize, t: std::time::Instant) -> std::time::Instant {
    PROFILE_NANOS[slot].fetch_add(
        t.elapsed().as_nanos() as u64,
        std::sync::atomic::Ordering::Relaxed,
    );
    std::time::Instant::now()
}
#[cfg(test)]
pub(super) static ATOM_STATS: [std::sync::atomic::AtomicU64; 2] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 2];
#[cfg(test)]
pub(super) fn profile_report(label: &str) {
    let v: Vec<f64> = PROFILE_NANOS
        .iter()
        .map(|a| a.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 * 1e-9)
        .collect();
    let atoms: Vec<u64> = ATOM_STATS
        .iter()
        .map(|a| a.swap(0, std::sync::atomic::Ordering::Relaxed))
        .collect();
    eprintln!("PATCHPROFILE {label} build={:.3} native={:.3} greedy={:.3} rotate={:.3} cpu_seconds atoms_compressed={} atoms_won={}", v[0], v[1], v[2], v[3], atoms[0], atoms[1]);
}

/// Screening proposal: the patch's minimum-degree order alone. Returns the
/// proposal and its exact gain over the incumbent, zero when it does not win.
fn propose_cheap(
    tree: &Tree,
    root: usize,
    vertices: &[usize],
    cfg: &RecoveryCfg,
) -> Option<(Proposal, u64)> {
    if vertices.len() + tree.bag[root].len() > cfg.max_interface { return None; }
    let mut work = cfg.patch_work;
    let (children, g, total) = tree.compress_reusing_map(root, vertices, &mut work)?;
    let k = vertices.len();
    let original = tree.original(vertices);
    let priority = if cfg.root_select == 4 {
        // Every completion adds edges/triangles. Subtracting the fixed exterior
        // clique leaves a lower bound on this patch's exact objective.
        original.saturating_sub(patch_potential(&g, total, k))
            / (k * total.div_ceil(64)).max(1) as u64
    } else { 0 };
    let (order, cost) = greedy_options(g, total, k, 0, &mut 0usize, 1, true)?;
    let gain = if cfg.root_select == 4 { priority } else { original.saturating_sub(cost) };
    Some((
        Proposal {
            root,
            order: order.iter().map(|&j| vertices[j]).collect(),
            children,
            cost,
            old: original,
            skeleton: vertices.to_vec(),
            joint: false,
        },
        gain,
    ))
}

/// n + 3 edges + 2 triangles, excluding the frozen exterior clique.
/// Count boundary-only triangles algebraically, not by scanning clique edges.
fn patch_potential(g: &[u64], n: usize, k: usize) -> u64 {
    let w = n.div_ceil(64);
    let mut edges = 0u64;
    let mut triangles = 0u64;
    for v in 0..k {
        let row = &g[v*w..(v+1)*w];
        let mut exterior = 0u64;
        for word in v/64..w {
            let mut higher = row[word];
            if word == v/64 {
                higher &= if v%64 == 63 { 0 } else { u64::MAX << (v%64+1) };
            }
            edges += higher.count_ones() as u64;
            let free = if word < k/64 { u64::MAX }
                else if word == k/64 { (1u64 << (k%64)) - 1 } else { 0 };
            exterior += (higher & !free).count_ones() as u64;
            let mut inside = higher & free;
            while inside != 0 {
                let u = word*64 + inside.trailing_zeros() as usize;
                inside &= inside - 1;
                for j in u/64..w {
                    let mut common = row[j] & g[u*w+j];
                    if j == u/64 {
                        common &= if u%64 == 63 { 0 } else { u64::MAX << (u%64+1) };
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
    joint: bool,
    seed: u64,
    cfg: &RecoveryCfg,
) -> Option<Proposal> {
    if vertices.len() + tree.bag[root].len() > cfg.max_interface { return None; }
    #[cfg(test)]
    let t = std::time::Instant::now();
    let mut work = cfg.patch_work;
    let (children, g, total) = tree.compress_reusing_map(root, vertices, &mut work)?;
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
    let local_pattern = Pattern {
        n: total,
        col_ptr: cp.clone(),
        row_idx: ri.clone(),
    };
    let sp = super::ScoringPattern {
        n: total,
        col_ptr: cp,
        row_idx: ri,
    };
    let b = (total - k) as u64;
    #[cfg(test)]
    let t = profile_add(0, t);
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
            let mut full = inside.clone();
            full.extend(k..total);
            let cost = super::flops_of(&sp, &full).checked_sub(sum_squares(b))?;
            consider(inside, cost, &mut best);
        }
    }
    #[cfg(test)]
    let t = profile_add(1, t);
    let dense = Dense::from_bits(g.clone(), total, k);
    let mut rng = Rng::new(seed);
    // Twin-quotient search when the boundary-compressed interface compresses
    // by at least a fifth: incumbent projection, weighted greedy, reinsertion.
    let mut compressed = false;
    if cfg.atoms {
        let atoms = Atoms::new(&dense);
        if atoms.k <= cfg.max_atoms && atoms.n <= 384 && (atoms.k * 5 < k * 4 || (cfg.pool_sources && k <= 512)) {
            compressed = atoms.k * 5 < k * 4;
            let mut ao = atoms.project(&(0..k).collect::<Vec<_>>());
            let mut ac = atoms.score(&ao);
            let mut sources = vec![ao.clone()];
            if let Some((order, _)) = &best {
                sources.push(atoms.project(order));
            }
            for mode in [0, 3, 6] {
                let (c, o) = atoms.greedy(mode, rng.next());
                sources.push(o.clone());
                if c < ac {
                    ac = c;
                    ao = o;
                }
            }
            if let Some((cost, order)) = atoms.joint(&ao) {
                if cost < ac { ac = cost; ao = order; }
            }
            if cfg.improve_rounds > 0 {
                let rounds = (atoms.k * 2).min(if atoms.k > 150 { 40 } else { 250 });
                let (c, o) = atoms.improve(ao, rounds, rng.next());
                ac = c;
                ao = o;
                sources.push(ao.clone());
                if cfg.pool_sources {
                    if atoms.k >= 2 && atoms.k <= 192 {
                        let mut profile_rng = Rng::new(seed ^ 0x8245_A017);
                        for hot in [false, true] {
                            sources.extend(atoms.profile_sources(ao.clone(), 32, profile_rng.next(), hot));
                        }
                    }
                    let (c, o) = atoms.pool(&sources);
                    if c < ac {
                        ac = c;
                        ao = o;
                    }
                    if cfg.bag_search {
                        sources.push(ao.clone());
                        let (c, o) = atoms.bag_pool(&sources, cfg.prefix_grid);
                        if c < ac { ac = c; ao = o; }
                    }
                }
            }
            #[cfg(test)]
            {
                ATOM_STATS[0].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if ac < best.as_ref().map_or(original, |b| b.1) {
                    ATOM_STATS[1].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
            consider(atoms.expand(&ao), ac, &mut best);
        }
    }
    if cfg.bag_search {
        let order = best.as_ref().map_or_else(|| (0..k).collect(), |b| b.0.clone());
        let (cost, order) = dense.bag_suffix(&order, seed.wrapping_add(891623), cfg.prefix_grid);
        consider(order, cost, &mut best);
    }
    let dense_modes = if compressed {
        cfg.dense_modes.min(1)
    } else if k > 256 {
        cfg.dense_modes.min(3)
    } else {
        cfg.dense_modes
    };
    for mode in 0..dense_modes {
        let (c, o) = dense.greedy(mode, rng.next());
        consider(o, c, &mut best);
    }
    for &mode in cfg.modes.iter().filter(|_| k <= 512) {
        let mut local_work = cfg.greedy_work;
        if let Some((order, cost)) =
            greedy_options(g.clone(), total, k, mode, &mut local_work, 12, true)
        {
            consider(order, cost, &mut best);
        }
    }
    if cfg.improve_rounds > 0 && k <= 600 {
        let start = best
            .as_ref()
            .map_or_else(|| (0..k).collect::<Vec<_>>(), |b| b.0.clone());
        let rounds = (k * 2)
            .min(4.max(30_000_000 / (k * total * dense.w).max(1)))
            .min(cfg.improve_rounds * k);
        let (c, o) = dense.improve(start, rounds, rng.next());
        consider(o, c, &mut best);
    }
    for it in 0..cfg.restarts {
        let start = best
            .as_ref()
            .map_or_else(|| (0..k).collect::<Vec<_>>(), |b| b.0.clone());
        let prefix = (rng.next() % k as u64) as usize;
        let (c, o) = dense.restart(&start, prefix, it % 8, rng.next());
        consider(o, c, &mut best);
    }
    if cfg.atoms && k <= 256 {
        let mut residual_work = cfg.greedy_work;
        for tail in [24, 12] {
            let start = best
                .as_ref()
                .map_or_else(|| (0..k).collect(), |b| b.0.clone());
            if let Some((cost, order)) = dense.residual_order(&start, tail, &mut residual_work) {
                consider(order, cost, &mut best);
            }
        }
    }
    #[cfg(test)]
    let t = profile_add(2, t);
    let mut initial = best
        .as_ref()
        .map_or_else(|| (0..k).collect::<Vec<_>>(), |b| b.0.clone());
    initial.extend(k..total);
    if cfg.rotation_work > 0 && k <= 512 {
        if let Some(order) = super::promotions::boundary_with_budget(
            &local_pattern,
            &initial,
            k,
            cfg.rotation_work,
            cfg.rotation_depth,
            cfg.rotation_passes,
        ) {
            if order[..k].iter().all(|&v| v < k) {
                let cost = super::flops_of(&sp, &order) - sum_squares(b);
                consider(order[..k].to_vec(), cost, &mut best);
            }
        }
    }
    #[cfg(test)]
    profile_add(3, t);
    let (order, cost) = best?;
    Some(Proposal {
        root,
        order: order.iter().map(|&j| vertices[j]).collect(),
        children,
        cost,
        old: original,
        skeleton: vertices.to_vec(),
        joint,
    })
}

/// Bottom-up choice between each subtree's incumbent and its compatible
/// proposals, then the replaced postorder.
fn assemble(tree: &Tree, proposals: &[Vec<Proposal>]) -> Option<Vec<usize>> {
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
        if p.joint || p.cost >= p.old {
            continue;
        }
        for (j, q) in proposals.iter().enumerate() {
            if i == j
                || q.joint
                || q.cost >= q.old
                || p.root == q.root
                || !tree.inside(q.root, p.root)
            {
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

/// Recovered separator pass: independent patch tasks on a fixed thread pool,
/// optional screening, cross-level unions, exact DP assembly.
pub(super) fn recovered(
    pattern: &Pattern,
    incumbent: &[usize],
    cfg: &RecoveryCfg,
) -> Option<Vec<usize>> {
    let tree = prepare(pattern, incumbent)?;
    recovered_prepared(&tree, cfg)
}

/// Immutable preparation shared by branches starting from the same order.
pub(super) fn prepare<'a>(pattern: &'a Pattern, incumbent: &[usize]) -> Option<Tree<'a>> {
    let n = pattern.n;
    if !(64..=320_000).contains(&n) || pattern.nnz() > 1_500_000 {
        return None;
    }
    Tree::build(pattern, incumbent)
}

pub(super) fn recovered_prepared(tree: &Tree<'_>, cfg: &RecoveryCfg) -> Option<Vec<usize>> {
    let n = tree.pattern.n;
    let roots = if matches!(cfg.root_select, 1 | 3 | 4) {
        tree.roots_by_density(1024, cfg.roots, cfg.root_select != 1)
    } else if cfg.root_select == 2 {
        tree.roots_by_cost(1024, cfg.roots, true)
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
    #[cfg(test)]
    let wall = std::time::Instant::now();
    let mut proposals: Vec<Vec<Proposal>> = (0..n).map(|_| Vec::new()).collect();
    let full: Vec<usize> = if cfg.full_tasks >= tasks.len() {
        (0..tasks.len()).collect()
    } else {
        // Preserve root coverage: cheap degree ordering alone can miss patches
        // whose gains need a different completion. Reserve half the solves for
        // the first viable patch of distinct roots, then rank by measured gain.
        let cheap = super::parallel::map_indexed(&tasks, cfg.threads, |_, (root, vertices)| {
            propose_cheap(&tree, *root, vertices, cfg)
        });
        let mut ranked: Vec<(u64, usize)> = Vec::new();
        let mut covered = BTreeSet::new();
        let mut selected = Vec::new();
        for (index, ((root, _), result)) in tasks.iter().zip(cheap).enumerate() {
            if let Some((pr, gain)) = result {
                proposals[*root].push(pr);
                ranked.push((gain, index));
                if cfg.root_select != 4 && selected.len() < cfg.full_tasks.div_ceil(2) && covered.insert(*root) {
                    selected.push(index);
                }
            }
        }
        ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        if cfg.root_select == 4 {
            for &(priority, index) in &ranked {
                if selected.len() == cfg.full_tasks { break; }
                if priority > 0 && covered.insert(tasks[index].0) { selected.push(index); }
            }
        }
        for (priority, index) in ranked {
            if selected.len() == cfg.full_tasks { break; }
            if cfg.root_select == 4 && priority == 0 { continue; }
            if !selected.contains(&index) { selected.push(index); }
        }
        selected.sort_unstable();
        #[cfg(test)]
        if cfg.root_select == 4 {
            let sizes: Vec<_> = selected.iter().map(|&i| tasks[i].1.len()).collect();
            eprintln!("DIRECTED_PATCHES n={n} sizes={sizes:?}");
        }
        selected
    };
    let results = super::parallel::map_indexed(&full, cfg.threads, |rank, &index| {
        let (root, vertices) = &tasks[index];
        propose(
            &tree,
            *root,
            vertices,
            rank < cfg.metis_tasks,
            false,
            0x5851_F42D_4C95_7F2D ^ (index as u64),
            cfg,
        )
    });
    let mut accepted: Vec<Proposal> = results.into_iter().flatten().collect();
    if cfg.unions > 0 {
        let max_k = cfg.caps.iter().copied().max().unwrap_or(512);
        let unions = union_tasks(&tree, &accepted, max_k, cfg.unions);
        let joint =
            super::parallel::map_indexed(&unions, cfg.threads, |index, (root, vertices)| {
                propose(
                    &tree,
                    *root,
                    vertices,
                    false,
                    true,
                    0x2545_F491_4F6C_DD1D ^ (index as u64),
                    cfg,
                )
            });
        #[cfg(test)]
        eprintln!(
            "UNIONS n={n} built={} improving={}",
            unions.len(),
            joint.iter().flatten().filter(|pr| pr.cost < pr.old).count()
        );
        accepted.extend(joint.into_iter().flatten());
    }
    #[cfg(test)]
    profile_report(&format!(
        "n={n} tasks={} full={} wall={:.3}",
        tasks.len(),
        full.len(),
        wall.elapsed().as_secs_f64()
    ));
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
                    joint: false,
                });
            }
        }
    }
    assemble(&tree, &proposals)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn profile_guided_joint() {
        let selected = [0,22,41,71,155,178,218,252,2,24,60,109,138,185,210,254,
                        50,79,126,268,285,216,150,74,88,131,189,247,267];
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/production_076/neutral_tail_full");
        for (index, (_, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            if !selected.contains(&index) { continue; }
            let bytes = std::fs::read(base.join(format!("{index:03}.0.perm"))).unwrap();
            let order: Vec<_> = bytes.chunks_exact(8).skip(1)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap()) as usize).collect();
            let Some(tree) = Tree::build(&pattern, &order) else { continue; };
            for root in tree.roots_by_density(1024, 2, true) {
                for cap in [128, 256, 512] {
                    let vertices = tree.grow(root, cap, 9);
                    if vertices.len() + tree.bag[root].len() > 2048 { continue; }
                    let Some((_, graph, n)) = tree.compress_reusing_map(root, &vertices, &mut 4_000_000) else { continue; };
                    let dense = Dense::from_bits(graph, n, vertices.len());
                    let atoms = Atoms::new(&dense);
                    if !(8..=256).contains(&atoms.k) || atoms.n > 384 { continue; }
                    let mut order = atoms.project(&(0..vertices.len()).collect::<Vec<_>>());
                    let mut before = atoms.score(&order);
                    for mode in [0,3,6] {
                        let (cost, candidate) = atoms.greedy(mode, 891623);
                        if cost < before { before = cost; order = candidate; }
                    }
                    for guided in [false, true] {
                        let started = std::time::Instant::now();
                        let result = if guided { atoms.joint(&order) } else { atoms.joint_by_charge(&order) };
                        if let Some((cost, candidate)) = result {
                            assert!(cost <= before);
                            assert_eq!(cost, dense.score(&atoms.expand(&candidate)));
                            eprintln!("JOINT_GROUP {index} {root} {cap} {} {guided} {before} {cost} {:.6}", atoms.k, started.elapsed().as_secs_f64());
                        }
                    }
                }
            }
        }
    }
    #[test]
    #[ignore]
    fn sampled_normalized_fill() {
        let selected = [0,22,41,71,155,178,218,252,2,24,60,109,138,185,210,254,
                        50,79,126,268,285,216,150,74];
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/production_076/rich_online_full");
        for (index, (_, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            if !selected.contains(&index) { continue; }
            let bytes = std::fs::read(base.join(format!("{index:03}.0.perm"))).unwrap();
            let order: Vec<_> = bytes.chunks_exact(8).skip(1)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap()) as usize).collect();
            let tree = Tree::build(&pattern, &order).unwrap();
            let mut map = vec![usize::MAX; pattern.n];
            for root in tree.roots_by_cap(1024, 1) {
                for cap in [512, 1024] {
                    let vertices = tree.grow(root, cap, 0);
                    let k = vertices.len();
                    if k < 256 || k + tree.bag[root].len() > 2048 { continue; }
                    let Some((_, graph, n)) = tree.compress(root, &vertices, &mut map, &mut 4_000_000) else { continue; };
                    let dense = Dense::from_bits(graph.clone(), n, k);
                    let atoms = Atoms::new(&dense);
                    eprintln!("QUOTIENT_GATE {index} {k} {n} {} {}", atoms.k, atoms.n);
                    let (cp, ri) = csc_of(&graph, n);
                    let scoring = super::super::ScoringPattern { n, col_ptr: cp, row_idx: ri };
                    let start = std::time::Instant::now();
                    let mut costs = Vec::new();
                    for mode in 0..3 {
                        let (cost, mut order) = dense.greedy(mode, 891623);
                        order.extend(k..n);
                        assert_eq!(cost, super::super::flops_of(&scoring, &order) - sum_squares((n-k) as u64));
                        costs.push(cost);
                    }
                    eprintln!("NORMALIZED_FILL {index} {k} {n} {} {} {} {:.4}", costs[0], costs[1], costs[2], start.elapsed().as_secs_f64());
                }
            }
        }
    }
    // Literal elimination independently checks every returned partial order,
    // including its fixed, live exterior boundary.
    #[test]
    fn boundary_greedy_matches_literal() {
        let mut seed = 7193u64;
        for n in [5usize, 17, 65, 129] {
            for density in [2u64, 5, 9] {
                let w = n.div_ceil(64);
                let mut g = vec![0u64; n * w];
                let mut a = vec![vec![false; n]; n];
                for u in 0..n {
                    for v in u + 1..n {
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        if seed % 10 < density {
                            a[u][v] = true;
                            a[v][u] = true;
                            g[u * w + v / 64] |= 1u64 << (v % 64);
                            g[v * w + u / 64] |= 1u64 << (u % 64);
                        }
                    }
                }
                for k in [n / 2, n] {
                    for mode in [0, 1, 2, 3, 4, 5, 6] {
                        for budget in [0usize, 1, 100_000_000] {
                            let (p, cost) =
                                greedy_options(g.clone(), n, k, mode, &mut { budget }, 12, true)
                                    .unwrap();
                            let mut seen = vec![false; k];
                            let mut b = a.clone();
                            let mut actual = 0u64;
                            for v in p {
                                assert!(v < k && !seen[v]);
                                seen[v] = true;
                                let nb: Vec<_> = (0..n).filter(|&u| b[v][u]).collect();
                                actual += ((nb.len() + 1) as u64).pow(2);
                                for &u in &nb {
                                    for &t in &nb {
                                        if u != t {
                                            b[u][t] = true;
                                        }
                                    }
                                    b[u][v] = false;
                                }
                                b[v].fill(false);
                            }
                            assert_eq!(actual, cost);
                            assert!(seen.into_iter().all(|v| v));
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn exhausted_budget_aborts() {
        let mut g = vec![0; 8];
        clique(&mut g, 1, &(0..8).collect::<Vec<_>>(), &mut 100usize).unwrap();
        assert!(greedy(g, 8, 8, 3, &mut 0usize).is_none());
    }
    #[test]
    fn patch_potential_bounds_every_small_order() {
        let orders = [[0,1,2], [0,2,1], [1,0,2], [1,2,0], [2,0,1], [2,1,0]];
        for mask in 0..1024 {
            let mut g = vec![0u64; 5];
            let mut bit = 0;
            for u in 0..5 {
                for v in u+1..5 {
                    if mask >> bit & 1 != 0 || (u == 3 && v == 4) {
                        g[u] |= 1 << v;
                        g[v] |= 1 << u;
                    }
                    bit += 1;
                }
            }
            let bound = patch_potential(&g, 5, 3);
            let dense = Dense::from_bits(g, 5, 3);
            for order in orders { assert!(bound <= dense.score(&order)); }
        }
        for n in [64usize, 65, 130] {
            let w = n.div_ceil(64);
            let mut g = vec![u64::MAX; n*w];
            for v in 0..n {
                if n%64 != 0 { g[(v+1)*w-1] &= (1 << (n%64))-1; }
                g[v*w+v/64] &= !(1 << (v%64));
            }
            assert_eq!(patch_potential(&g, n, 3), sum_squares(n as u64)-sum_squares((n-3) as u64));
        }
    }

    #[test]
    fn patch_potential_matches_literal_triangle_counts() {
        for n in [63usize, 64, 65, 129, 130] {
            for k in [1, 3, 63, 64, 65].into_iter().filter(|&k| k <= n) {
                let w = n.div_ceil(64);
                let mut g = vec![0u64; n*w];
                let mut rng = Rng::new(541);
                for u in 0..n { for v in u+1..n {
                    if u >= k || rng.next()%3 == 0 {
                        g[u*w+v/64] |= 1 << (v%64);
                        g[v*w+u/64] |= 1 << (u%64);
                    }
                } }
                let has = |u: usize, v: usize| g[u*w+v/64] >> (v%64) & 1 != 0;
                let (mut edges, mut triangles) = (0u64, 0u64);
                for u in 0..k { for v in u+1..n {
                    if has(u, v) {
                        edges += 1;
                        for x in v+1..n {
                            triangles += u64::from(has(u,x) && has(v,x));
                        }
                    }
                } }
                assert_eq!(patch_potential(&g,n,k), k as u64+3*edges+2*triangles);
            }
        }
    }
    #[test]
    fn reused_patch_map_survives_rejections_and_size_changes() {
        for n in [12, 6, 18, 6] {
            let mut col_ptr = vec![0];
            let mut row_idx = Vec::new();
            for v in 0..n {
                let mut row = vec![(v + 1) % n, (v + n - 1) % n];
                row.sort_unstable();
                row_idx.extend(row);
                col_ptr.push(row_idx.len());
            }
            let pattern = Pattern { n, col_ptr, row_idx };
            let tree = Tree::build(&pattern, &(0..n).collect::<Vec<_>>()).unwrap();
            for root in 0..n {
                for cap in [2, 4, 8] {
                    let vertices = tree.grow(root, cap, 0);
                    for budget in [0, 4_000_000, 1, 4_000_000] {
                        let mut fresh = budget;
                        let expected = tree.compress(root, &vertices, &mut vec![usize::MAX; n], &mut fresh);
                        let mut reused = budget;
                        assert_eq!(tree.compress_reusing_map(root, &vertices, &mut reused), expected);
                        assert_eq!(fresh, reused);
                    }
                }
            }
        }
    }
    #[test]
    fn release_exposes_both_supporting_branches() {
        // Two length-three paths between 4 and 5. Exposing only one hidden
        // branch leaves the other branch's induced 4--5 edge intact.
        let adj = [
            vec![1, 4],
            vec![0, 5],
            vec![3, 4],
            vec![2, 5],
            vec![0, 2],
            vec![1, 3],
        ];
        let mut cp = vec![0];
        let mut ri = Vec::new();
        for row in &adj {
            ri.extend(row);
            cp.push(ri.len());
        }
        let pattern = Pattern {
            n: 6,
            col_ptr: cp,
            row_idx: ri,
        };
        let tree = Tree::build(&pattern, &(0..6).collect::<Vec<_>>()).unwrap();
        let seed = vec![tree.rank[4], tree.rank[5]];
        let released = tree.release_support(seed.clone(), 4);
        assert_eq!(released.len(), 4);
        let selected: BTreeSet<_> = released.iter().map(|&v| tree.p[v]).collect();
        assert!(selected.contains(&4) && selected.contains(&5));
        for &v in &released {
            if tree.parent[v] >= 0 {
                assert!(released.contains(&(tree.parent[v] as usize)));
            }
        }
        let mut actual: Vec<BTreeSet<usize>> =
            adj.iter().map(|r| r.iter().copied().collect()).collect();
        for &v in &tree.p {
            if selected.contains(&v) {
                continue;
            }
            let neighbors: Vec<_> = actual[v].iter().copied().collect();
            for &u in &neighbors {
                actual[u].remove(&v);
                for &w in &neighbors {
                    if u != w {
                        actual[u].insert(w);
                    }
                }
            }
            actual[v].clear();
        }
        assert!(!actual[4].contains(&5));
        // A budget of one extra vertex cannot release the pair completely.
        assert_eq!(tree.release_support(seed.clone(), 3), seed);
    }
    fn shuffled_grid() -> (Pattern, Vec<usize>) {
        let side = 40;
        let n = side * side;
        let mut a = vec![Vec::new(); n];
        for v in 0..n {
            let (x, y) = (v % side, v / side);
            if x + 1 < side {
                a[v].push(v + 1);
                a[v + 1].push(v);
            }
            if y + 1 < side {
                a[v].push(v + side);
                a[v + side].push(v);
            }
        }
        let mut cp = vec![0];
        let mut ri = Vec::new();
        for row in &mut a {
            row.sort_unstable();
            ri.extend_from_slice(row);
            cp.push(ri.len());
        }
        (
            Pattern {
                n,
                col_ptr: cp,
                row_idx: ri,
            },
            (0..n).map(|v| (v * 997) % n).collect(),
        )
    }
    // The parallel recovered pass must return a bijection whose exact
    // whole-graph cost is below the incumbent's and identical for one and
    // several threads, with every search component enabled.
    #[test]
    fn recovered_pass_is_exact_and_thread_independent() {
        let (pattern, p) = shuffled_grid();
        let n = pattern.n;
        let sp = super::super::ScoringPattern {
            n,
            col_ptr: pattern.col_ptr.clone(),
            row_idx: pattern.row_idx.clone(),
        };
        let before = super::super::flops_of(&sp, &p);
        let mut cfg = RecoveryCfg::fast(true);
        cfg.skeleton_modes = vec![0, 1, 2, 5, 7, 9];
        cfg.dense_modes = 8;
        cfg.improve_rounds = 1;
        cfg.restarts = 2;
        cfg.atoms = true;
        cfg.unions = 4;
        cfg.root_select = 1;
        cfg.full_tasks = 6;
        cfg.threads = 1;
        let one = recovered(&pattern, &p, &cfg).expect("improvement on shuffled grid");
        cfg.threads = 5;
        let many = recovered(&pattern, &p, &cfg).expect("improvement on shuffled grid");
        assert_eq!(one, many);
        assert!(super::super::is_bijection(&one, n));
        assert!(super::super::flops_of(&sp, &one) < before);
    }
}
