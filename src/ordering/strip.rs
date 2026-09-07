//! Deterministic STRIP candidates for sparse symmetric orderings.
//!
//! A STRIP candidate removes a small prefix of structurally high-ranked
//! vertices, orders the induced graph on the remaining vertices with AMD, and
//! appends the removed vertices in ascending `(degree, vertex)` order. Every
//! choice is a pure function of the input sparsity pattern with total tie
//! breaks.

use crate::Pattern;

/// Largest removal first so one incremental builder can restore vertices as
/// the family walks toward progressively larger induced graphs.
pub(crate) const DEGREE_KS: [usize; 5] = [96, 64, 48, 24, 6];

/// A deliberately smaller ladder for each additional structural ranking.
pub(crate) const EXTRA_KS: [usize; 3] = [64, 24, 6];

/// A vertex-induced graph in local CSC numbering.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct InducedSubgraph {
    pub(crate) col_ptr: Vec<i32>,
    pub(crate) row_idx: Vec<i32>,
    pub(crate) local_to_global: Vec<usize>,
}

/// Rank by descending structural degree, breaking ties by ascending vertex id.
pub(crate) fn degree_ranking(pattern: &Pattern) -> Vec<usize> {
    let mut ranking: Vec<usize> = (0..pattern.n).collect();
    ranking.sort_unstable_by(|&a, &b| degree(pattern, b).cmp(&degree(pattern, a)).then(a.cmp(&b)));
    ranking
}

/// Rank by descending degeneracy core number, then original degree and
/// one-hop degree mass. Core numbers are obtained by deterministic minimum
/// residual-degree peeling; the heap's `(degree, vertex)` key is a total order.
pub(crate) fn core_ranking(pattern: &Pattern) -> Vec<usize> {
    let (degrees, neighbor_sums) = degrees_and_neighbor_sums(pattern);
    let core = degeneracy_core_scores(pattern, &degrees);
    let mut ranking: Vec<usize> = (0..pattern.n).collect();
    ranking.sort_unstable_by(|&a, &b| {
        core[b]
            .cmp(&core[a])
            .then(degrees[b].cmp(&degrees[a]))
            .then(neighbor_sums[b].cmp(&neighbor_sums[a]))
            .then(a.cmp(&b))
    });
    ranking
}

fn degeneracy_core_scores(pattern: &Pattern, degrees: &[usize]) -> Vec<usize> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let n = pattern.n;
    let mut residual = degrees.to_vec();
    let mut removed = vec![false; n];
    let mut core = vec![0usize; n];
    let mut heap = BinaryHeap::with_capacity(n);
    for (v, &d) in residual.iter().enumerate() {
        heap.push(Reverse((d, v)));
    }

    let mut level = 0usize;
    while let Some(Reverse((d, v))) = heap.pop() {
        if removed[v] || residual[v] != d {
            continue;
        }
        removed[v] = true;
        level = level.max(d);
        core[v] = level;
        for &w in pattern.col(v) {
            if !removed[w] {
                residual[w] = residual[w].saturating_sub(1);
                heap.push(Reverse((residual[w], w)));
            }
        }
    }
    core
}

fn degrees_and_neighbor_sums(pattern: &Pattern) -> (Vec<usize>, Vec<u64>) {
    let degrees: Vec<usize> = (0..pattern.n).map(|v| degree(pattern, v)).collect();
    let neighbor_sums: Vec<u64> = (0..pattern.n)
        .map(|v| {
            pattern.col(v).iter().fold(0u64, |sum, &w| {
                sum.saturating_add(degrees[w] as u64)
            })
        })
        .collect();
    (degrees, neighbor_sums)
}

/// Build one STRIP candidate independently from the full pattern.
pub(crate) fn strip_candidate(pattern: &Pattern, ranking: &[usize], k: usize) -> Option<Vec<i32>> {
    let sub = induced_from_scratch(pattern, ranking, k)?;
    strip_from_induced(pattern, &sub, &ranking[..k])
}

/// Order a prebuilt induced graph and append its removed vertices.
pub(crate) fn strip_from_induced(
    pattern: &Pattern,
    subgraph: &InducedSubgraph,
    removed_prefix: &[usize],
) -> Option<Vec<i32>> {
    let n = pattern.n;
    let k = removed_prefix.len();
    if k == 0 || k >= n || subgraph.local_to_global.len() != n - k {
        return None;
    }
    for (i, &v) in removed_prefix.iter().enumerate() {
        if v >= n || removed_prefix[..i].contains(&v) {
            return None;
        }
    }

    let core = feral_ordering_core::CscPattern::new(
        subgraph.local_to_global.len(),
        &subgraph.col_ptr,
        &subgraph.row_idx,
    )?;
    let local_perm = feral_amd::amd_order(&core).ok()?;
    let mut perm = Vec::with_capacity(n);
    for local in local_perm {
        let local = usize::try_from(local).ok()?;
        perm.push(i32::try_from(*subgraph.local_to_global.get(local)?).ok()?);
    }

    let mut tail = removed_prefix.to_vec();
    tail.sort_unstable_by_key(|&v| (degree(pattern, v), v));
    for v in tail {
        perm.push(i32::try_from(v).ok()?);
    }
    Some(perm)
}

/// Materialize every valid degree-ranked stage using one nested builder.
pub(crate) fn degree_induced_schedule(
    pattern: &Pattern,
) -> (Vec<usize>, Vec<(usize, InducedSubgraph)>) {
    induced_schedule(pattern, degree_ranking(pattern), &DEGREE_KS)
}

/// Materialize a descending fixed-k schedule for any deterministic ranking.
pub(crate) fn induced_schedule(
    pattern: &Pattern,
    ranking: Vec<usize>,
    ks: &[usize],
) -> (Vec<usize>, Vec<(usize, InducedSubgraph)>) {
    let valid_ks: Vec<usize> = ks
        .iter()
        .copied()
        .filter(|&k| k > 0 && k < pattern.n)
        .collect();
    if valid_ks.windows(2).any(|pair| pair[0] <= pair[1]) {
        return (ranking, Vec::new());
    }
    let Some((&first_k, remaining_ks)) = valid_ks.split_first() else {
        return (ranking, Vec::new());
    };

    let Some((mut builder, first)) = NestedSubgraphBuilder::new(pattern, &ranking, first_k) else {
        return (ranking, Vec::new());
    };
    let mut stages = Vec::with_capacity(valid_ks.len());
    stages.push((first_k, first));
    for &k in remaining_ks {
        let Some(subgraph) = builder.advance_to(k) else {
            return (ranking, Vec::new());
        };
        stages.push((k, subgraph));
    }
    (ranking, stages)
}

/// Incrementally maintains adjacency among the currently kept vertices.
pub(crate) struct NestedSubgraphBuilder<'a> {
    pattern: &'a Pattern,
    ranking: &'a [usize],
    removed: Vec<bool>,
    adjacency: Vec<Vec<usize>>,
    current_k: usize,
}

impl<'a> NestedSubgraphBuilder<'a> {
    pub(crate) fn new(
        pattern: &'a Pattern,
        ranking: &'a [usize],
        k_max: usize,
    ) -> Option<(Self, InducedSubgraph)> {
        let n = pattern.n;
        if k_max == 0 || k_max >= n || ranking.len() < k_max {
            return None;
        }

        let mut removed = vec![false; n];
        for &v in &ranking[..k_max] {
            if v >= n || removed[v] {
                return None;
            }
            removed[v] = true;
        }

        let mut adjacency = vec![Vec::new(); n];
        for v in 0..n {
            if removed[v] {
                continue;
            }
            for &w in pattern.col(v) {
                if w > v && !removed[w] {
                    adjacency[v].push(w);
                    adjacency[w].push(v);
                }
            }
        }

        let builder = Self {
            pattern,
            ranking,
            removed,
            adjacency,
            current_k: k_max,
        };
        let first = builder.emit();
        Some((builder, first))
    }

    pub(crate) fn advance_to(&mut self, k: usize) -> Option<InducedSubgraph> {
        if k >= self.current_k {
            return None;
        }
        let restoring = &self.ranking[k..self.current_k];
        if restoring
            .iter()
            .any(|&v| v >= self.pattern.n || !self.removed[v])
        {
            return None;
        }

        for &v in restoring {
            self.removed[v] = false;
            for &w in self.pattern.col(v) {
                if !self.removed[w] {
                    self.adjacency[v].push(w);
                    self.adjacency[w].push(v);
                }
            }
        }
        self.current_k = k;
        Some(self.emit())
    }

    fn emit(&self) -> InducedSubgraph {
        let n = self.pattern.n;
        let mut local_to_global = Vec::with_capacity(n - self.current_k);
        let mut global_to_local = vec![usize::MAX; n];
        for (v, &is_removed) in self.removed.iter().enumerate() {
            if !is_removed {
                global_to_local[v] = local_to_global.len();
                local_to_global.push(v);
            }
        }

        let m = local_to_global.len();
        let mut col_ptr = Vec::with_capacity(m + 1);
        col_ptr.push(0i32);
        for &v in &local_to_global {
            let next = col_ptr.last().copied().unwrap_or(0)
                + i32::try_from(self.adjacency[v].len())
                    .expect("induced subgraph exceeds i32 CSC capacity");
            col_ptr.push(next);
        }

        let mut row_idx = vec![0i32; col_ptr[m] as usize];
        for (local_v, &global_v) in local_to_global.iter().enumerate() {
            let start = col_ptr[local_v] as usize;
            let end = col_ptr[local_v + 1] as usize;
            for (dst, &global_w) in row_idx[start..end]
                .iter_mut()
                .zip(self.adjacency[global_v].iter())
            {
                *dst = i32::try_from(global_to_local[global_w])
                    .expect("induced subgraph local index exceeds i32");
            }
            row_idx[start..end].sort_unstable();
        }

        InducedSubgraph {
            col_ptr,
            row_idx,
            local_to_global,
        }
    }
}

/// Independent reference construction used by tests and the L-count candidate.
fn induced_from_scratch(pattern: &Pattern, ranking: &[usize], k: usize) -> Option<InducedSubgraph> {
    let n = pattern.n;
    if k == 0 || k >= n || ranking.len() < k {
        return None;
    }

    let mut removed = vec![false; n];
    for &v in &ranking[..k] {
        if v >= n || removed[v] {
            return None;
        }
        removed[v] = true;
    }

    let mut local_to_global = Vec::with_capacity(n - k);
    let mut global_to_local = vec![usize::MAX; n];
    for (v, &is_removed) in removed.iter().enumerate() {
        if !is_removed {
            global_to_local[v] = local_to_global.len();
            local_to_global.push(v);
        }
    }

    let mut col_ptr = Vec::with_capacity(n - k + 1);
    let mut row_idx = Vec::new();
    col_ptr.push(0i32);
    for &v in &local_to_global {
        for &w in pattern.col(v) {
            if !removed[w] {
                row_idx.push(i32::try_from(global_to_local[w]).ok()?);
            }
        }
        let start = col_ptr.last().copied()? as usize;
        row_idx[start..].sort_unstable();
        col_ptr.push(i32::try_from(row_idx.len()).ok()?);
    }

    Some(InducedSubgraph {
        col_ptr,
        row_idx,
        local_to_global,
    })
}

#[inline]
fn degree(pattern: &Pattern, v: usize) -> usize {
    pattern.col_ptr[v + 1] - pattern.col_ptr[v]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bijection(perm: &[i32], n: usize) {
        assert_eq!(perm.len(), n);
        let mut seen = vec![false; n];
        for &v in perm {
            let v = usize::try_from(v).expect("negative vertex in permutation");
            assert!(v < n);
            assert!(!seen[v]);
            seen[v] = true;
        }
        assert!(seen.into_iter().all(|v| v));
    }

    fn synthetic_pattern(n: usize) -> Pattern {
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 2..n {
            if v % 2 == 0 {
                edges.push((0, v));
            }
            if v % 3 == 0 {
                edges.push((1, v));
            }
            if v % 5 == 0 {
                edges.push((2, v));
            }
            if v + 17 < n && v % 7 == 0 {
                edges.push((v, v + 17));
            }
        }
        Pattern::from_edges(n, &edges)
    }

    #[test]
    fn degree_ranking_has_total_tie_break() {
        let pattern = Pattern::from_edges(6, &[(0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (1, 2)]);
        assert_eq!(degree_ranking(&pattern), vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(degree_ranking(&pattern), degree_ranking(&pattern));
    }

    #[test]
    fn core_ranking_is_deterministic_and_matches_simple_core_numbers() {
        let pattern = synthetic_pattern(140);
        let a = core_ranking(&pattern);
        let b = core_ranking(&pattern);
        assert_eq!(a, b);
        assert_bijection(&a.iter().map(|&v| v as i32).collect::<Vec<_>>(), pattern.n);

        let star = Pattern::from_edges(6, &[(0, 1), (0, 2), (0, 3), (0, 4), (0, 5)]);
        let star_degrees: Vec<usize> = (0..star.n).map(|v| degree(&star, v)).collect();
        assert_eq!(degeneracy_core_scores(&star, &star_degrees), vec![1; 6]);

        let mut clique_edges = Vec::new();
        for v in 0..5 {
            for w in v + 1..5 {
                clique_edges.push((v, w));
            }
        }
        let clique = Pattern::from_edges(5, &clique_edges);
        let clique_degrees: Vec<usize> = (0..clique.n).map(|v| degree(&clique, v)).collect();
        assert_eq!(degeneracy_core_scores(&clique, &clique_degrees), vec![4; 5]);

        let (ranking, stages) = induced_schedule(&pattern, a, &EXTRA_KS);
        for (k, subgraph) in stages {
            let incremental = strip_from_induced(&pattern, &subgraph, &ranking[..k]).unwrap();
            let rebuilt = strip_candidate(&pattern, &ranking, k).unwrap();
            assert_eq!(incremental, rebuilt);
        }
    }

    #[test]
    fn incremental_subgraphs_and_candidates_match_naive_bytes() {
        let pattern = synthetic_pattern(140);
        let ranking = degree_ranking(&pattern);
        let (mut builder, first) =
            NestedSubgraphBuilder::new(&pattern, &ranking, DEGREE_KS[0]).unwrap();

        let mut subgraph = first;
        for (stage, &k) in DEGREE_KS.iter().enumerate() {
            if stage != 0 {
                subgraph = builder.advance_to(k).unwrap();
            }
            let naive = induced_from_scratch(&pattern, &ranking, k).unwrap();
            assert_eq!(subgraph, naive, "induced CSC differs at k={k}");

            let incremental = strip_from_induced(&pattern, &subgraph, &ranking[..k]).unwrap();
            let rebuilt = strip_candidate(&pattern, &ranking, k).unwrap();
            assert_eq!(incremental, rebuilt, "candidate differs at k={k}");
            assert_bijection(&incremental, pattern.n);
        }
    }

    #[test]
    fn scheduled_family_is_deterministic_and_handles_small_patterns() {
        let pattern = synthetic_pattern(140);
        let (ranking_a, stages_a) = degree_induced_schedule(&pattern);
        let (ranking_b, stages_b) = degree_induced_schedule(&pattern);
        assert_eq!(ranking_a, ranking_b);
        assert_eq!(stages_a, stages_b);

        let candidates_a: Vec<Vec<i32>> = stages_a
            .iter()
            .map(|(k, sub)| strip_from_induced(&pattern, sub, &ranking_a[..*k]).unwrap())
            .collect();
        let candidates_b: Vec<Vec<i32>> = stages_b
            .iter()
            .map(|(k, sub)| strip_from_induced(&pattern, sub, &ranking_b[..*k]).unwrap())
            .collect();
        assert_eq!(candidates_a, candidates_b);
        for candidate in &candidates_a {
            assert_bijection(candidate, pattern.n);
        }

        let small = synthetic_pattern(25);
        let (_, stages) = degree_induced_schedule(&small);
        assert_eq!(
            stages.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
            vec![24, 6]
        );
        let tiny = Pattern::from_edges(6, &[(0, 1), (1, 2)]);
        assert!(degree_induced_schedule(&tiny).1.is_empty());
    }

    #[test]
    fn invalid_rankings_fail_without_panicking() {
        let pattern = synthetic_pattern(20);
        assert!(strip_candidate(&pattern, &[0, 0], 2).is_none());
        assert!(strip_candidate(&pattern, &[pattern.n], 1).is_none());
        assert!(strip_candidate(&pattern, &[0], 0).is_none());
        assert!(strip_candidate(&pattern, &[0], pattern.n).is_none());
    }
}
