//! Exact recombination of connected regions supplied by local block orders.
//! Region identity uses complete bit vectors or exact interval containment.
use std::collections::BTreeMap;

/// Shortest path through observed prefix sets. Independent forest pivots are
/// first aligned to a common reference, preserving their per-vertex charges.
/// Original paths remain available; equality is checked without fingerprints.
pub(super) fn prefix_pool(sp: &super::ScoringPattern, inputs: &[&[usize]]) -> Option<Vec<usize>> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let n = sp.n;
    if n == 0 || inputs.is_empty() || inputs.len() > 64 {
        return None;
    }
    let mut prepared = Vec::new();
    for &p in inputs {
        if !super::is_bijection(p, n) {
            return None;
        }
        let (post, counts, parent) = super::etree_prep(&sp, p);
        prepared.push((p.to_vec(), post, counts, parent));
    }
    let reference = prepared
        .iter()
        .min_by_key(|(_, _, c, _)| c.iter().map(|&x| (x as u64).pow(2)).sum::<u64>())?;
    let mut rank = vec![0; n];
    for (i, &v) in reference.0.iter().enumerate() {
        rank[v] = i;
    }
    // The reference C++ kernel stores every original source first, then
    // appends the two alignments of each source. This order governs DP ties.
    let mut paths: Vec<(Vec<usize>, Vec<u64>)> = prepared
        .iter()
        .map(|(original, post, counts, _)| {
            let mut charge = vec![0; n];
            for (&v, &c) in post.iter().zip(counts) {
                charge[v] = (c as u64).pow(2);
            }
            (original.clone(), charge)
        })
        .collect();
    for (_, post, counts, parent) in prepared {
        if paths.len() == 64 {
            break;
        }
        let mut charge = vec![0; n];
        let mut children = vec![0; n];
        let mut tree = vec![Vec::new(); n + 1];
        let mut minimum: Vec<_> = post.iter().map(|&v| rank[v]).collect();
        for i in 0..n {
            charge[post[i]] = (counts[i] as u64).pow(2);
            if parent[i] >= 0 {
                let p = parent[i] as usize;
                children[p] += 1;
                tree[p].push(i);
                minimum[p] = minimum[p].min(minimum[i]);
            } else {
                tree[n].push(i);
            }
        }
        for kids in &mut tree {
            kids.sort_unstable_by_key(|&i| (minimum[i], post[i]));
        }
        let mut subtree_order = Vec::with_capacity(n);
        let mut stack: Vec<_> = tree[n].iter().rev().map(|&i| (i, false)).collect();
        while let Some((i, exit)) = stack.pop() {
            if exit {
                subtree_order.push(post[i]);
            } else {
                stack.push((i, true));
                stack.extend(tree[i].iter().rev().map(|&c| (c, false)));
            }
        }
        let mut ready = BinaryHeap::new();
        for i in 0..n {
            if children[i] == 0 {
                ready.push(Reverse((rank[post[i]], i)));
            }
        }
        let mut aligned = Vec::with_capacity(n);
        while let Some(Reverse((_, i))) = ready.pop() {
            aligned.push(post[i]);
            if parent[i] >= 0 {
                let p = parent[i] as usize;
                children[p] -= 1;
                if children[p] == 0 {
                    ready.push(Reverse((rank[post[p]], p)));
                }
            }
        }
        for order in [aligned, subtree_order] {
            if paths.len() == 64 {
                break;
            }
            if !paths.iter().any(|(p, _)| *p == order) {
                paths.push((order, charge.clone()));
            }
        }
    }
    let stride = n + 1;
    let mut states: Vec<_> = (0..paths.len() * stride).collect();
    for b in 0..paths.len() {
        for (j, &v) in paths[b].0.iter().enumerate() {
            rank[v] = j;
        }
        for a in 0..b {
            let x = find(&mut states, a * stride);
            let y = find(&mut states, b * stride);
            states[y] = x;
            let mut maximum = 0;
            for (j, &v) in paths[a].0.iter().enumerate() {
                maximum = maximum.max(rank[v]);
                if maximum == j {
                    let x = find(&mut states, a * stride + j + 1);
                    let y = find(&mut states, b * stride + j + 1);
                    states[y] = x;
                }
            }
        }
    }
    for i in 0..states.len() {
        states[i] = find(&mut states, i);
    }
    let mut distance = vec![u64::MAX; states.len()];
    let mut previous = vec![(usize::MAX, usize::MAX); states.len()];
    distance[states[0]] = 0;
    for j in 0..n {
        for (a, (order, charge)) in paths.iter().enumerate() {
            let from = states[a * stride + j];
            let to = states[a * stride + j + 1];
            let v = order[j];
            let cost = distance[from].saturating_add(charge[v]);
            if cost < distance[to] {
                distance[to] = cost;
                previous[to] = (from, v);
            }
        }
    }
    let mut at = states[n];
    let mut output = Vec::with_capacity(n);
    while at != states[0] {
        let (from, v) = previous[at];
        output.push(v);
        at = from;
    }
    output.reverse();
    Some(output)
}

struct RangeMap {
    base: usize,
    tree: Vec<(usize, usize)>,
}
impl RangeMap {
    fn new(order: &[usize], rank: &[usize]) -> Self {
        let base = order.len().next_power_of_two();
        let mut tree = vec![(usize::MAX, 0); 2 * base];
        for (i, &v) in order.iter().enumerate() {
            tree[base + i] = (rank[v], rank[v]);
        }
        for i in (1..base).rev() {
            tree[i] = (
                tree[2 * i].0.min(tree[2 * i + 1].0),
                tree[2 * i].1.max(tree[2 * i + 1].1),
            );
        }
        Self { base, tree }
    }
    fn bounds(&self, mut lo: usize, mut hi: usize) -> (usize, usize) {
        lo += self.base;
        hi += self.base;
        let mut out = (usize::MAX, 0);
        while lo < hi {
            if lo & 1 != 0 {
                out = (out.0.min(self.tree[lo].0), out.1.max(self.tree[lo].1));
                lo += 1;
            }
            if hi & 1 != 0 {
                hi -= 1;
                out = (out.0.min(self.tree[hi].0), out.1.max(self.tree[hi].1));
            }
            lo /= 2;
            hi /= 2;
        }
        out
    }
}

struct WholeRegion {
    owner: usize,
    start: usize,
    end: usize,
    charge: u64,
    choices: Vec<Choice>,
}

struct PreparedWhole {
    input: Vec<usize>,
    order: Vec<usize>,
    counts: Vec<u32>,
    parent: Vec<i32>,
    rank: Vec<usize>,
}

/// Per-graph symbolic preparation, reused while the caller grows its donor pool.
/// Each combine still uses exactly the supplied donors in their supplied order.
pub(super) struct WholePool<'a> {
    scoring: &'a super::ScoringPattern,
    prepared: Vec<PreparedWhole>,
}

impl<'a> WholePool<'a> {
    pub(super) fn new(scoring: &'a super::ScoringPattern) -> Self {
        Self {
            scoring,
            prepared: Vec::new(),
        }
    }

    pub(super) fn combine(&mut self, inputs: &[&[usize]]) -> Option<Vec<usize>> {
        let n = self.scoring.n;
        if n == 0 {
            return None;
        }
        let mut sources = Vec::new();
        for &p in inputs {
            if !super::is_bijection(p, n) {
                return None;
            }
            let id = if let Some(id) = self.prepared.iter().position(|s| s.input == p) {
                id
            } else {
                let (order, counts, parent) = super::etree_prep(&self.scoring, p);
                let mut rank = vec![0; n];
                for (i, &v) in order.iter().enumerate() {
                    rank[v] = i;
                }
                let id = self.prepared.len();
                self.prepared.push(PreparedWhole {
                    input: p.to_vec(),
                    order,
                    counts,
                    parent,
                    rank,
                });
                id
            };
            if !sources
                .iter()
                .any(|&previous: &usize| self.prepared[previous].order == self.prepared[id].order)
            {
                sources.push(id);
            }
        }
        if sources.len() < 2 {
            return None;
        }
        let mut ranks: Vec<&[usize]> = Vec::new();
        let mut regions: Vec<WholeRegion> = Vec::new();
        let mut lookup: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
        let mut leaves = vec![usize::MAX; n];
        let mut roots = Vec::new();
        for source in sources {
            let prepared = &self.prepared[source];
            let (order, counts, parent) = (&prepared.order, &prepared.counts, &prepared.parent);
            let owner = ranks.len();
            ranks.push(&prepared.rank);
            let mut maps: Vec<Option<RangeMap>> = (0..owner).map(|_| None).collect();
            let mut children = vec![Vec::new(); n];
            let mut size = vec![1; n];
            let mut minimum = order.clone();
            for v in 0..n {
                if parent[v] >= 0 {
                    let q = parent[v] as usize;
                    children[q].push(v);
                    size[q] += size[v];
                    minimum[q] = minimum[q].min(minimum[v]);
                }
            }
            let mut ids = vec![0; n];
            roots.clear();
            for v in 0..n {
                let start = v + 1 - size[v];
                // Singleton regions have one possible root and no child choices.
                // Index them directly instead of repeating ordered-map searches
                // for every leaf in every donor.
                if size[v] == 1 {
                    let vertex = order[v];
                    let mut id = leaves[vertex];
                    if id == usize::MAX {
                        id = regions.len();
                        leaves[vertex] = id;
                        regions.push(WholeRegion {
                            owner,
                            start,
                            end: v + 1,
                            charge: (counts[v] as u64).pow(2),
                            choices: vec![Choice {
                                root: vertex,
                                children: Vec::new(),
                            }],
                        });
                    }
                    ids[v] = id;
                    if parent[v] < 0 {
                        roots.push(id);
                    }
                    continue;
                }
                let key = (size[v], minimum[v]);
                let mut found = None;
                if let Some(candidates) = lookup.get(&key) {
                    for &id in candidates {
                        let region = &regions[id];
                        let equal = if region.owner == owner {
                            region.start == start && region.end == v + 1
                        } else {
                            let map = maps[region.owner]
                                .get_or_insert_with(|| RangeMap::new(&order, &ranks[region.owner]));
                            let (lo, hi) = map.bounds(start, v + 1);
                            lo >= region.start && hi < region.end
                        };
                        if equal {
                            found = Some(id);
                            break;
                        }
                    }
                }
                let id = found.unwrap_or_else(|| {
                    let id = regions.len();
                    regions.push(WholeRegion {
                        owner,
                        start,
                        end: v + 1,
                        charge: (counts[v] as u64).pow(2),
                        choices: Vec::new(),
                    });
                    lookup.entry(key).or_default().push(id);
                    id
                });
                if !regions[id].choices.iter().any(|c| c.root == order[v]) {
                    let mut child_ids = std::mem::take(&mut children[v]);
                    for child in &mut child_ids {
                        *child = ids[*child];
                    }
                    regions[id].choices.push(Choice {
                        root: order[v],
                        children: child_ids,
                    });
                }
                ids[v] = id;
                if parent[v] < 0 {
                    roots.push(id);
                }
            }
        }
        let mut sequence: Vec<_> = (0..regions.len()).collect();
        sequence.sort_by_key(|&r| (regions[r].end - regions[r].start, r));
        let mut costs = vec![u64::MAX; regions.len()];
        let mut selected = vec![0; regions.len()];
        for r in sequence {
            for (j, choice) in regions[r].choices.iter().enumerate() {
                let cost =
                    regions[r].charge + choice.children.iter().map(|&c| costs[c]).sum::<u64>();
                if cost < costs[r] {
                    costs[r] = cost;
                    selected[r] = j;
                }
            }
        }
        let mut output = Vec::with_capacity(n);
        let mut stack: Vec<_> = roots.into_iter().rev().map(|r| (r, false)).collect();
        while let Some((r, exit)) = stack.pop() {
            let choice = &regions[r].choices[selected[r]];
            if exit {
                output.push(choice.root);
            } else {
                stack.push((r, true));
                stack.extend(choice.children.iter().rev().map(|&c| (c, false)));
            }
        }
        Some(output)
    }
}

struct Choice {
    root: usize,
    children: Vec<usize>,
}

fn find(parent: &mut [usize], mut v: usize) -> usize {
    while parent[v] != v {
        parent[v] = parent[parent[v]];
        v = parent[v];
    }
    v
}
