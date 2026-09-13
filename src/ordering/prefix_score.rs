//! Exact elimination costs without constructing the fill graph.
//!
//! After activating the next pivot, let C be its connected component in the
//! original graph induced by the eliminated prefix. Its live neighbors are
//! exactly N(C) \ C: fill corresponds to paths with eliminated internal
//! vertices. Thus its column count is |union_{v in C} N[v]| - |C| + 1.

use crate::Pattern;
use std::cell::RefCell;

const WORDS: usize = 16;

pub(super) struct PrefixScore {
    pub(super) n: usize,
    words: usize,
    pub(super) rows: Vec<[u64; WORDS]>,
    offsets: Vec<usize>,
    neighbors: Vec<u16>,
    degree: Vec<u32>,
    work: RefCell<Workspace>,
}

struct Workspace {
    rows: Vec<[u64; WORDS]>,
    parent: Vec<usize>,
    size: Vec<u32>,
    cardinality: Vec<u32>,
    active: Vec<u32>,
    epoch: u32,
    #[cfg(test)]
    stats: WorkStats,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
struct WorkStats {
    pivots: usize,
    merges: usize,
    copied_words: usize,
    merged_words: usize,
    absorbed_words: usize,
    sparse_inserts: usize,
    neighbor_visits: usize,
}

impl Workspace {
    fn new(n: usize) -> Self {
        Self {
            rows: vec![[0; WORDS]; n],
            parent: vec![0; n],
            size: vec![0; n],
            cardinality: vec![0; n],
            active: vec![0; n],
            epoch: 0,
            #[cfg(test)]
            stats: WorkStats::default(),
        }
    }

    fn begin(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.active.fill(0);
            self.epoch = 1;
        }
        #[cfg(test)]
        {
            self.stats = WorkStats::default();
        }
    }

    #[inline]
    fn root(&mut self, mut v: usize) -> usize {
        while self.parent[v] != v {
            self.parent[v] = self.parent[self.parent[v]];
            v = self.parent[v];
        }
        v
    }

    fn merge(&mut self, mut a: usize, mut b: usize, words: usize) -> usize {
        if a == b {
            return a;
        }
        if self.size[a] < self.size[b] || (self.size[a] == self.size[b] && a > b) {
            std::mem::swap(&mut a, &mut b);
        }
        let (dst, src) = if a < b {
            let (left, right) = self.rows.split_at_mut(b);
            (&mut left[a], &right[0])
        } else {
            let (left, right) = self.rows.split_at_mut(a);
            (&mut right[0], &left[b])
        };
        let mut added = 0;
        for k in 0..words {
            added += (src[k] & !dst[k]).count_ones();
            dst[k] |= src[k];
        }
        self.parent[b] = a;
        self.size[a] += self.size[b];
        self.cardinality[a] += added;
        #[cfg(test)]
        {
            self.stats.merges += 1;
            self.stats.merged_words += words;
        }
        a
    }
}

#[inline]
fn square_sum(n: u64) -> u64 {
    n * (n + 1) * (2 * n + 1) / 6
}

impl PrefixScore {
    pub(super) fn new(p: &Pattern) -> Self {
        assert!(p.n <= WORDS * 64);
        let words = p.n.div_ceil(64);
        let mut rows = vec![[0u64; WORDS]; p.n];
        for c in 0..p.n {
            for &r in &p.row_idx[p.col_ptr[c]..p.col_ptr[c + 1]] {
                if r != c {
                    rows[c][r >> 6] |= 1u64 << (r & 63);
                    rows[r][c >> 6] |= 1u64 << (c & 63);
                }
            }
        }
        let degree: Vec<u32> = rows
            .iter()
            .map(|row| row[..words].iter().map(|word| word.count_ones()).sum())
            .collect();
        let mut offsets = Vec::with_capacity(p.n + 1);
        let mut neighbors = Vec::with_capacity(degree.iter().map(|&d| d as usize).sum());
        offsets.push(0);
        for row in &rows {
            for (k, &word) in row[..words].iter().enumerate() {
                let mut bits = word;
                while bits != 0 {
                    neighbors.push((64 * k + bits.trailing_zeros() as usize) as u16);
                    bits &= bits - 1;
                }
            }
            offsets.push(neighbors.len());
        }
        Self {
            n: p.n,
            words,
            rows,
            offsets,
            neighbors,
            degree,
            work: RefCell::new(Workspace::new(p.n)),
        }
    }

    pub(super) fn flops(&self, perm: &[usize]) -> u64 {
        self.flops_bounded(perm, u64::MAX)
    }

    pub(super) fn flops_bounded(&self, perm: &[usize], bound: u64) -> u64 {
        let mut work = self.work.borrow_mut();
        work.begin();
        let mut total = 0u64;
        for (step, &v) in perm.iter().enumerate() {
            debug_assert_ne!(work.active[v], work.epoch);
            let neighbors = &self.neighbors[self.offsets[v]..self.offsets[v + 1]];
            let mut component = None;
            if step != 0 {
                for &u in neighbors {
                    let u = u as usize;
                    #[cfg(test)]
                    {
                        work.stats.neighbor_visits += 1;
                    }
                    if work.active[u] == work.epoch {
                        let root = work.root(u);
                        let root = match component {
                            Some(old) => work.merge(old, root, self.words),
                            None => root,
                        };
                        component = Some(root);
                        if work.size[root] as usize == step {
                            break;
                        }
                    }
                }
            }

            let root = if let Some(root) = component {
                // An active neighbor already contributed v to this closed union.
                debug_assert_ne!(work.rows[root][v >> 6] & (1u64 << (v & 63)), 0);
                work.parent[v] = root;
                work.size[root] += 1;
                let mut added = 0;
                if neighbors.len() < self.words {
                    for &u in neighbors {
                        let u = u as usize;
                        let bit = 1u64 << (u & 63);
                        let word = &mut work.rows[root][u >> 6];
                        added += u32::from(*word & bit == 0);
                        *word |= bit;
                    }
                    #[cfg(test)]
                    {
                        work.stats.sparse_inserts += neighbors.len();
                    }
                } else {
                    for k in 0..self.words {
                        let bits = self.rows[v][k];
                        added += (bits & !work.rows[root][k]).count_ones();
                        work.rows[root][k] |= bits;
                    }
                    #[cfg(test)]
                    {
                        work.stats.absorbed_words += self.words;
                    }
                }
                work.cardinality[root] += added;
                root
            } else {
                work.parent[v] = v;
                work.size[v] = 1;
                work.cardinality[v] = self.degree[v] + 1;
                work.rows[v][..self.words].copy_from_slice(&self.rows[v][..self.words]);
                work.rows[v][v >> 6] |= 1u64 << (v & 63);
                #[cfg(test)]
                {
                    work.stats.copied_words += self.words;
                }
                v
            };
            work.active[v] = work.epoch;
            let count = u64::from(work.cardinality[root] - work.size[root] + 1);
            total += count * count;
            #[cfg(test)]
            {
                work.stats.pivots += 1;
            }

            let remaining = (self.n - step - 1) as u64;
            let d = count - 1;
            debug_assert!(d <= remaining);
            let suffix_floor = square_sum(d) + remaining - d;
            if total + suffix_floor > bound {
                return bound.saturating_add(1);
            }
            // Only a complete permutation may replace its suffix by this sum.
            if d == remaining && perm.len() == self.n {
                return total + square_sum(d);
            }
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pattern(n: usize, edges: &[(usize, usize)]) -> Pattern {
        let mut rows = vec![Vec::new(); n];
        for &(u, v) in edges {
            rows[u].push(v);
            rows[v].push(u);
        }
        let mut col_ptr = vec![0];
        let mut row_idx = Vec::new();
        for row in rows {
            row_idx.extend(row);
            col_ptr.push(row_idx.len());
        }
        Pattern {
            n,
            col_ptr,
            row_idx,
        }
    }

    fn oracle(p: &Pattern, perm: &[usize]) -> (u64, usize) {
        let mut edges = vec![vec![false; p.n]; p.n];
        for v in 0..p.n {
            for &u in &p.row_idx[p.col_ptr[v]..p.col_ptr[v + 1]] {
                if u != v {
                    edges[v][u] = true;
                    edges[u][v] = true;
                }
            }
        }
        let mut live = vec![true; p.n];
        let mut total = 0;
        let mut reference_word_updates = 0;
        for &v in perm {
            let neighbors: Vec<_> = (0..p.n).filter(|&u| live[u] && edges[v][u]).collect();
            let count = neighbors.len() as u64 + 1;
            total += count * count;
            reference_word_updates += neighbors.len() * p.n.div_ceil(64);
            for (i, &u) in neighbors.iter().enumerate() {
                for &t in &neighbors[..i] {
                    edges[u][t] = true;
                    edges[t][u] = true;
                }
            }
            live[v] = false;
        }
        (total, reference_word_updates)
    }

    fn next_permutation(perm: &mut [usize]) -> bool {
        if perm.len() < 2 {
            return false;
        }
        let Some(i) = (0..perm.len() - 1).rev().find(|&i| perm[i] < perm[i + 1]) else {
            return false;
        };
        let j = (i + 1..perm.len())
            .rev()
            .find(|&j| perm[j] > perm[i])
            .unwrap();
        perm.swap(i, j);
        perm[i + 1..].reverse();
        true
    }

    fn check(score: &PrefixScore, perm: &[usize], expected: u64) {
        assert_eq!(score.flops(perm), expected, "{perm:?}");
        assert_eq!(score.flops_bounded(perm, expected), expected, "{perm:?}");
        assert_eq!(
            score.flops_bounded(perm, expected + 1),
            expected,
            "{perm:?}"
        );
        assert_eq!(score.flops_bounded(perm, u64::MAX), expected, "{perm:?}");
        if expected != 0 {
            let bound = expected - 1;
            assert_eq!(score.flops_bounded(perm, bound), bound + 1, "{perm:?}");
            assert_eq!(score.flops_bounded(perm, 0), 1, "{perm:?}");
        }
    }

    fn random(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    fn shuffle(perm: &mut [usize], state: &mut u64) {
        for i in (1..perm.len()).rev() {
            let j = (random(state) % (i + 1) as u64) as usize;
            perm.swap(i, j);
        }
    }

    #[test]
    fn exhaustive_graphs_and_permutations() {
        for n in 0..=5 {
            let pairs: Vec<_> = (0..n)
                .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
                .collect();
            for mask in 0usize..1usize << pairs.len() {
                let edges: Vec<_> = pairs
                    .iter()
                    .enumerate()
                    .filter_map(|(bit, &edge)| (mask & (1 << bit) != 0).then_some(edge))
                    .collect();
                let p = pattern(n, &edges);
                let score = PrefixScore::new(&p);
                let mut perm: Vec<_> = (0..n).collect();
                loop {
                    check(&score, &perm, oracle(&p, &perm).0);
                    if !next_permutation(&mut perm) {
                        break;
                    }
                }
            }
        }
    }

    #[test]
    fn random_graphs_and_cross_word_labels() {
        let mut state = 0x976413ae0b29c5d1;
        for n in [7, 17, 63, 64, 65, 127, 128, 129] {
            for threshold in [1, 4, 12] {
                let mut edges = Vec::new();
                for u in 0..n {
                    for v in u + 1..n {
                        if random(&mut state) % 16 < threshold {
                            edges.push((u, v));
                        }
                    }
                }
                let p = pattern(n, &edges);
                let score = PrefixScore::new(&p);
                let mut perm: Vec<_> = (0..n).collect();
                for _ in 0..3 {
                    shuffle(&mut perm, &mut state);
                    check(&score, &perm, oracle(&p, &perm).0);
                    score.flops(&perm);
                    let stats = score.work.borrow().stats;
                    assert!(stats.merges < n);
                    assert!(
                        stats.copied_words + stats.merged_words + stats.absorbed_words
                            <= (2 * n - 1) * score.words
                    );
                    assert!(stats.neighbor_visits <= score.neighbors.len());
                }
            }
        }
    }

    #[test]
    fn disconnected_hubs_and_shared_boundaries() {
        let n = 130;
        let mut edges = Vec::new();
        for u in 1..64 {
            edges.push((0, u));
        }
        for u in 66..129 {
            edges.push((65, u));
        }
        edges.extend([(1, 2), (2, 3), (3, 1), (67, 68), (68, 69), (69, 67)]);
        let p = pattern(n, &edges);
        let score = PrefixScore::new(&p);
        let mut state = 0xa061fed3552819cb;
        let mut perm: Vec<_> = (0..n).collect();
        check(&score, &perm, oracle(&p, &perm).0);
        perm.reverse();
        check(&score, &perm, oracle(&p, &perm).0);
        for _ in 0..6 {
            shuffle(&mut perm, &mut state);
            check(&score, &perm, oracle(&p, &perm).0);
        }

        let p = pattern(8, &[(0, 4), (0, 5), (1, 4), (1, 5), (2, 5), (3, 5), (4, 6)]);
        let score = PrefixScore::new(&p);
        for perm in [
            [0, 1, 2, 3, 4, 5, 6, 7],
            [1, 3, 0, 2, 5, 4, 7, 6],
            [4, 5, 0, 1, 2, 3, 6, 7],
        ] {
            check(&score, &perm, oracle(&p, &perm).0);
        }
    }

    #[test]
    fn duplicates_diagonal_and_one_sided_pattern() {
        let p = Pattern {
            n: 5,
            col_ptr: vec![0, 4, 5, 7, 8, 9],
            row_idx: vec![0, 1, 1, 4, 1, 0, 3, 3, 4],
        };
        let score = PrefixScore::new(&p);
        let mut perm: Vec<_> = (0..p.n).collect();
        loop {
            check(&score, &perm, oracle(&p, &perm).0);
            if !next_permutation(&mut perm) {
                break;
            }
        }
    }

    #[test]
    fn shared_live_neighbors_do_not_join_prefix_components() {
        let score = PrefixScore::new(&pattern(5, &[(0, 2), (1, 2), (2, 3)]));
        assert_eq!(score.flops(&[0, 1]), 8);
        assert_eq!(score.work.borrow().stats.merges, 0);
        assert_eq!(score.flops(&[0, 1, 2]), 12);
        assert_eq!(score.work.borrow().stats.merges, 1);
        assert_eq!(score.flops(&[0, 1, 2, 3, 4]), 14);
    }

    #[test]
    fn maximum_clique_bounds_and_empty_input() {
        let empty = PrefixScore::new(&pattern(0, &[]));
        assert_eq!(empty.flops(&[]), 0);
        assert_eq!(empty.flops_bounded(&[], 0), 0);
        assert_eq!(empty.flops_bounded(&[], u64::MAX), 0);
        let singleton = PrefixScore::new(&pattern(1, &[]));
        check(&singleton, &[0], 1);

        let n = WORDS * 64;
        let edges: Vec<_> = (0..n)
            .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
            .collect();
        let score = PrefixScore::new(&pattern(n, &edges));
        let perm: Vec<_> = (0..n).rev().collect();
        let expected = 358_438_400;
        check(&score, &perm, expected);
        assert_eq!(score.flops_bounded(&perm, u64::MAX - 1), expected);
        assert_eq!(score.work.borrow().stats.pivots, 1);
    }

    #[test]
    #[should_panic]
    fn rejects_dimensions_above_small_score_limit() {
        let n = WORDS * 64 + 1;
        PrefixScore::new(&Pattern {
            n,
            col_ptr: vec![0; n + 1],
            row_idx: vec![],
        });
    }

    #[test]
    fn rejected_trials_and_epoch_wrap_do_not_leak_state() {
        let p = pattern(9, &[(0, 1), (1, 2), (2, 3), (3, 0), (0, 4), (2, 5), (6, 7)]);
        let score = PrefixScore::new(&p);
        let forward: Vec<_> = (0..p.n).collect();
        let backward: Vec<_> = forward.iter().copied().rev().collect();
        let expected = oracle(&p, &forward).0;
        for _ in 0..16 {
            assert_eq!(score.flops_bounded(&backward, 0), 1);
            assert_eq!(score.flops(&forward), expected);
        }
        {
            let mut work = score.work.borrow_mut();
            work.epoch = u32::MAX;
            work.active.fill(1);
        }
        check(&score, &forward, expected);
        check(&score, &backward, oracle(&p, &backward).0);
    }

    #[test]
    fn partial_orders_and_structural_work_bound() {
        let n = 130;
        let edges: Vec<_> = (0..n)
            .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
            .collect();
        let p = pattern(n, &edges);
        let score = PrefixScore::new(&p);
        let prefix: Vec<_> = (0..n - 1).collect();
        let (expected, old_word_updates) = oracle(&p, &prefix);
        assert_eq!(score.flops(&prefix), expected);
        let stats = score.work.borrow().stats;
        assert_eq!(stats.pivots, prefix.len());
        assert_eq!(stats.merges, 0);
        let new_word_work = stats.copied_words + stats.merged_words + stats.absorbed_words;
        assert!(
            new_word_work * 16 < old_word_updates,
            "{stats:?}, old={old_word_updates}"
        );
        assert_eq!(score.flops(&[]), 0);

        let p = pattern(
            9,
            &[
                (0, 1),
                (0, 3),
                (1, 4),
                (2, 3),
                (2, 5),
                (4, 6),
                (5, 6),
                (6, 7),
            ],
        );
        let score = PrefixScore::new(&p);
        let perm = [3, 1, 5, 7, 0, 2, 4, 6, 8];
        for end in 0..=perm.len() {
            assert_eq!(score.flops(&perm[..end]), oracle(&p, &perm[..end]).0);
        }
    }

    fn refine(
        start: Vec<usize>,
        neutral: bool,
        mut score: impl FnMut(&[usize], u64) -> u64,
    ) -> Vec<usize> {
        let n = start.len();
        let mut best = start;
        let mut best_f = score(&best, u64::MAX);
        let mut state = 0x917ad73;
        for _ in 0..512 {
            let positions =
                std::array::from_fn::<_, 4, _>(|_| (random(&mut state) % n as u64) as usize);
            if (0..4).any(|i| (i + 1..4).any(|j| positions[i] == positions[j])) {
                continue;
            }
            let mut candidate = best.clone();
            candidate.swap(positions[0], positions[1]);
            candidate.swap(positions[2], positions[3]);
            let f = score(&candidate, best_f);
            if f < best_f {
                best_f = f;
                best = candidate;
            }
        }
        let mut current = best.clone();
        state = 0xa839d37;
        for _ in 0..1024 {
            let a = (random(&mut state) % n as u64) as usize;
            let b = (random(&mut state) % n as u64) as usize;
            if a == b {
                continue;
            }
            current.swap(a, b);
            let f = score(&current, best_f);
            if f < best_f {
                best_f = f;
                best = current.clone();
            } else if f > best_f || !neutral {
                current.swap(a, b);
            }
        }
        best
    }

    #[test]
    fn paired_and_neutral_walks_match_exact_elimination() {
        let mut state = 0xdb1647a80c3ef529;
        for n in [4, 7, 11] {
            for threshold in [2, 5, 12] {
                let mut edges = Vec::new();
                for u in 0..n {
                    for v in u + 1..n {
                        if random(&mut state) % 16 < threshold {
                            edges.push((u, v));
                        }
                    }
                }
                let p = pattern(n, &edges);
                let score = PrefixScore::new(&p);
                let mut start: Vec<_> = (0..n).collect();
                shuffle(&mut start, &mut state);
                for neutral in [false, true] {
                    let expected = refine(start.clone(), neutral, |perm, _| oracle(&p, perm).0);
                    let actual = refine(start.clone(), neutral, |perm, bound| {
                        score.flops_bounded(perm, bound)
                    });
                    assert_eq!(
                        actual, expected,
                        "n={n}, threshold={threshold}, neutral={neutral}"
                    );
                }
            }
        }
    }
}
