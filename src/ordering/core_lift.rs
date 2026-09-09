//! CORE-LIFT — a fixed exact-elimination prefix, then the portfolio on the
//! residual core.
//!
//! Peel degree-≤1 vertices (pendants) and eliminate every vertex whose live
//! degree is ≤ `max_row_deg`, in ascending `(degree, index)` order, adding the
//! clique on each eliminated vertex's live neighbours. That is the EXACT
//! elimination step, so the residual graph is the exact fill graph after the
//! prefix, and
//!
//! ```text
//!   Σ_j c_j²  =  Σ_{v ∈ prefix} c_v²  +  Σ_{w ∈ core} c_w²
//! ```
//!
//! splits with the first term FIXED (the prefix is one fixed sequence) and the
//! second term computed on the core graph alone. So a candidate ordering of the
//! core can be ranked on the core graph — ~100× cheaper than a full-graph
//! scoring pass — and only the winner needs the trusted global scorer.
//!
//! DETERMINISM. The elimination order is a strict total order on
//! `(degree, index)` (indices are unique, so there are no ties); the edge set
//! is a hash SET that is only ever membership-tested, never iterated, so no
//! hash order reaches the output; the core is numbered by ascending original
//! index. Same input ⇒ same `CoreLift`, byte for byte.

use feral::sparse::csc::CscPattern as ScoringPattern;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, Hasher};

/// Deterministic multiplicative hasher for the `u64` edge keys.
///
/// The default `RandomState` is deterministic *within* a process too, but it
/// is also ~2× the cost of this on the ~2M membership tests the reduction runs
/// on a `gt_10k` giant, and that cost lands directly on the hard time
/// constraint. Fixed constants, no randomness, no crate.
#[derive(Default)]
pub(crate) struct EdgeHasher(u64);

impl Hasher for EdgeHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ b as u64).wrapping_mul(0x1000_0000_01B3);
        }
    }
    #[inline]
    fn write_u64(&mut self, v: u64) {
        let mut x = v ^ 0x9E37_79B9_7F4A_7C15;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        self.0 = x ^ (x >> 31);
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

type EdgeSet = HashSet<u64, BuildHasherDefault<EdgeHasher>>;

/// A reduction of a pattern into a fixed elimination prefix plus a residual
/// core, with the core's fill graph as a standalone symmetric CSC.
pub(crate) struct CoreLift {
    /// Eliminated vertices, in the order they were eliminated. Original ids.
    pub(crate) prefix: Vec<usize>,
    /// Surviving vertices, ascending original id. `core_ids[k]` is the
    /// original id of core vertex `k`.
    pub(crate) core_ids: Vec<usize>,
    /// The core's fill graph, symmetric, diagonal omitted (same convention as
    /// `Pattern`/`ScoringPattern`), indexed `0..core_ids.len()`.
    pub(crate) core_col_ptr: Vec<usize>,
    pub(crate) core_row_idx: Vec<usize>,
    /// Σ c_v² over the prefix — the fixed part of the objective.
    pub(crate) prefix_flops: u64,
}

impl CoreLift {
    #[inline]
    pub(crate) fn core_n(&self) -> usize {
        self.core_ids.len()
    }
    #[inline]
    pub(crate) fn core_nnz(&self) -> usize {
        self.core_row_idx.len()
    }
}

#[inline]
fn key(a: u32, b: u32) -> u64 {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    ((lo as u64) << 32) | (hi as u64)
}

/// Reduce `sp` to (prefix, core). Returns `None` when the pattern is empty, is
/// too large for the `u32` vertex ids used internally, or when the residual
/// core misses the size gates — every `None` path leaves the caller's
/// incumbent untouched.
#[inline]
pub(crate) fn reduce(
    sp: &ScoringPattern,
    max_row_deg: usize,
    max_core_n: usize,
    max_core_edges: usize,
) -> Option<CoreLift> {
    reduce_impl::<false>(sp, max_row_deg, max_core_n, max_core_edges, 0)
}

/// Checked higher-degree variant. The const-generic split is deliberate: the
/// ordinary degree-three production path instantiates `CHECK_PAIRS = false`,
/// so clique-pair budget arithmetic is compiled out of its elimination loop.
#[inline]
pub(crate) fn reduce_checked(
    sp: &ScoringPattern,
    max_row_deg: usize,
    max_core_n: usize,
    max_core_edges: usize,
    max_clique_pair_checks: u64,
) -> Option<CoreLift> {
    reduce_impl::<true>(
        sp,
        max_row_deg,
        max_core_n,
        max_core_edges,
        max_clique_pair_checks,
    )
}

#[inline]
fn reduce_impl<const CHECK_PAIRS: bool>(
    sp: &ScoringPattern,
    max_row_deg: usize,
    max_core_n: usize,
    max_core_edges: usize,
    max_clique_pair_checks: u64,
) -> Option<CoreLift> {
    let n = sp.n;
    if n == 0 || n > u32::MAX as usize {
        return None;
    }

    let mut nbrs: Vec<Vec<u32>> = Vec::with_capacity(n);
    let mut deg: Vec<u32> = vec![0; n];
    let mut edges: EdgeSet =
        EdgeSet::with_capacity_and_hasher(sp.row_idx.len() / 2 + 16, Default::default());
    for j in 0..n {
        let s = sp.col_ptr[j];
        let e = sp.col_ptr[j + 1];
        let list: Vec<u32> = sp.row_idx[s..e]
            .iter()
            .map(|&x| u32::try_from(x).ok())
            .collect::<Option<_>>()?;
        deg[j] = u32::try_from(list.len()).ok()?;
        for &w in &list {
            if (w as usize) > j {
                edges.insert(key(j as u32, w));
            }
        }
        nbrs.push(list);
    }

    let mut alive = vec![true; n];
    let mut prefix: Vec<usize> = Vec::new();
    let mut prefix_flops: u64 = 0;
    let mut clique_pair_checks_left = max_clique_pair_checks;

    // Ascending (degree, index). Pendants (degree ≤ 1) therefore always come
    // out before any degree-2/3 vertex, at every point in the run — the
    // "peel pendants first, to exhaustion" phase is the head of this order,
    // and it re-runs automatically whenever an elimination creates a new one.
    let mut heap: BinaryHeap<Reverse<(u32, u32)>> = BinaryHeap::new();
    for v in 0..n {
        if deg[v] as usize <= max_row_deg {
            heap.push(Reverse((deg[v], v as u32)));
        }
    }

    let mut live: Vec<u32> = Vec::with_capacity(max_row_deg.checked_add(1)?);
    while let Some(Reverse((d, v))) = heap.pop() {
        let vu = v as usize;
        // Stale entry (the vertex was eliminated, or its degree moved since
        // this entry was pushed).
        if !alive[vu] || deg[vu] != d || d as usize > max_row_deg {
            continue;
        }
        live.clear();
        for &w in &nbrs[vu] {
            let wu = w as usize;
            if wu != vu && alive[wu] && edges.contains(&key(v, w)) {
                live.push(w);
            }
        }
        live.sort_unstable();
        live.dedup();
        // The input contract is symmetric CSC with no diagonal or duplicate
        // entries. Keep that assumption fail-closed in release too: otherwise
        // a malformed adjacency (or an update-accounting bug) could make the
        // stored degree diverge from the exact live-neighbour set and turn the
        // purported degree-bounded step into a different elimination.
        if live.len() != deg[vu] as usize || live.len() > max_row_deg {
            return None;
        }
        if CHECK_PAIRS {
            let clique_pair_checks = live
                .len()
                .checked_mul(live.len().saturating_sub(1))?
                / 2;
            clique_pair_checks_left = clique_pair_checks_left
                .checked_sub(u64::try_from(clique_pair_checks).ok()?)?;
        }
        // c_v = its live degree at elimination time (Liu): the column count of
        // v in the factor is exactly |live| + 1 counting the diagonal. The
        // grader's Σ c_j² counts the diagonal, matching `column_counts_gnp`.
        let cv = live.len() as u64 + 1;
        prefix_flops = prefix_flops.checked_add(cv.checked_mul(cv)?)?;
        // Close the neighbourhood into a clique — the exact elimination step.
        for i in 0..live.len() {
            for j in (i + 1)..live.len() {
                let (a, b) = (live[i], live[j]);
                if edges.insert(key(a, b)) {
                    nbrs[a as usize].push(b);
                    nbrs[b as usize].push(a);
                    deg[a as usize] = deg[a as usize].checked_add(1)?;
                    deg[b as usize] = deg[b as usize].checked_add(1)?;
                }
            }
        }
        for &w in live.iter() {
            edges.remove(&key(v, w));
            deg[w as usize] = deg[w as usize].checked_sub(1)?;
        }
        alive[vu] = false;
        deg[vu] = 0;
        nbrs[vu] = Vec::new();
        prefix.push(vu);
        for &w in live.iter() {
            if deg[w as usize] as usize <= max_row_deg {
                heap.push(Reverse((deg[w as usize], w)));
            }
        }
    }

    let core_ids: Vec<usize> = (0..n).filter(|&v| alive[v]).collect();
    let core_n = core_ids.len();
    if core_n == 0 || core_n > max_core_n {
        return None;
    }
    let mut pos_of: Vec<u32> = vec![u32::MAX; n];
    for (k, &v) in core_ids.iter().enumerate() {
        pos_of[v] = k as u32;
    }
    let mut core_col_ptr: Vec<usize> = Vec::with_capacity(core_n + 1);
    let mut core_row_idx: Vec<usize> = Vec::new();
    core_col_ptr.push(0);
    let mut row: Vec<u32> = Vec::new();
    let core_nnz_cap = max_core_edges.checked_mul(2)?;
    for &v in core_ids.iter() {
        row.clear();
        for &w in &nbrs[v] {
            let wu = w as usize;
            if wu != v && alive[wu] && edges.contains(&key(v as u32, w)) {
                row.push(pos_of[wu]);
            }
        }
        row.sort_unstable();
        row.dedup();
        core_row_idx.extend(row.iter().map(|&x| x as usize));
        core_col_ptr.push(core_row_idx.len());
        if core_row_idx.len() > core_nnz_cap {
            return None;
        }
    }

    Some(CoreLift {
        prefix,
        core_ids,
        core_col_ptr,
        core_row_idx,
        prefix_flops,
    })
}

/// prefix ++ core, mapped back to original vertex ids. `core_perm[k]` is the
/// core vertex eliminated `k`-th.
pub(crate) fn splice(cl: &CoreLift, core_perm: &[usize]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::with_capacity(cl.prefix.len() + core_perm.len());
    out.extend_from_slice(&cl.prefix);
    for &k in core_perm {
        out.push(cl.core_ids[k]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use feral::ordering::amd::permute_pattern;
    use feral::ordering::elimination_tree::EliminationTree;
    use feral::symbolic::column_counts_gnp;

    fn pattern(n: usize, edges: &[(usize, usize)]) -> ScoringPattern {
        let mut cols = vec![Vec::new(); n];
        for &(a, b) in edges {
            assert!(a < n && b < n && a != b);
            cols[a].push(b);
            cols[b].push(a);
        }
        let mut col_ptr = Vec::with_capacity(n + 1);
        let mut row_idx = Vec::new();
        col_ptr.push(0);
        for col in &mut cols {
            col.sort_unstable();
            col.dedup();
            row_idx.extend_from_slice(col);
            col_ptr.push(row_idx.len());
        }
        ScoringPattern {
            n,
            col_ptr,
            row_idx,
        }
    }

    fn all_edges(n: usize) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for a in 0..n {
            for b in (a + 1)..n {
                out.push((a, b));
            }
        }
        out
    }

    fn assert_bijection(perm: &[usize], n: usize) {
        assert_eq!(perm.len(), n);
        let mut seen = vec![false; n];
        for &v in perm {
            assert!(v < n && !seen[v]);
            seen[v] = true;
        }
    }

    fn assert_symmetric_core(cl: &CoreLift) {
        assert_eq!(cl.core_col_ptr.len(), cl.core_n() + 1);
        assert_eq!(cl.core_col_ptr[0], 0);
        assert_eq!(cl.core_col_ptr[cl.core_n()], cl.core_row_idx.len());
        for v in 0..cl.core_n() {
            let col = &cl.core_row_idx[cl.core_col_ptr[v]..cl.core_col_ptr[v + 1]];
            assert!(col.windows(2).all(|w| w[0] < w[1]));
            for &u in col {
                assert!(u < cl.core_n() && u != v);
                let other = &cl.core_row_idx[cl.core_col_ptr[u]..cl.core_col_ptr[u + 1]];
                assert!(
                    other.binary_search(&v).is_ok(),
                    "missing reverse edge {u}-{v}"
                );
            }
        }
    }

    fn flops(sp: &ScoringPattern, perm: &[usize]) -> u64 {
        let permuted = permute_pattern(sp, perm);
        let etree = EliminationTree::from_pattern(&permuted);
        column_counts_gnp(&permuted, &etree)
            .iter()
            .map(|&c| (c as u64) * (c as u64))
            .sum()
    }

    #[test]
    fn path_star_and_k4_obey_degree_thresholds_and_full_drain() {
        let path = pattern(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
        let untouched_path = reduce(&path, 0, 5, 4).expect("degree-zero threshold keeps path");
        assert!(untouched_path.prefix.is_empty());
        assert_eq!(untouched_path.core_ids, (0..5).collect::<Vec<_>>());
        assert!(
            reduce(&path, 1, 5, 4).is_none(),
            "pendant peeling drains path"
        );

        let star = pattern(6, &[(0, 1), (0, 2), (0, 3), (0, 4), (0, 5)]);
        assert!(
            reduce(&star, 1, 6, 5).is_none(),
            "leaf peeling drains star"
        );

        let k4 = pattern(4, &all_edges(4));
        let untouched_k4 = reduce(&k4, 2, 4, 6).expect("K4 minimum degree is three");
        assert!(untouched_k4.prefix.is_empty());
        assert_eq!(untouched_k4.core_nnz(), 12);
        assert!(
            reduce(&k4, 3, 4, 6).is_none(),
            "degree-three pass drains K4"
        );
    }

    #[test]
    fn degree_three_fill_builds_exact_symmetric_core_and_score_split() {
        // Vertices 1..=6 are K6 with the triangle (1,2,3) deleted. Vertex 0
        // has exactly those three vertices as neighbours. Eliminating 0 fills
        // the missing triangle, leaving an exact K6 residual core.
        let mut edges = all_edges(7)
            .into_iter()
            .filter(|&(a, b)| !((a == 0 && b >= 4) || matches!((a, b), (1, 2) | (1, 3) | (2, 3))))
            .collect::<Vec<_>>();
        edges.sort_unstable();
        let sp = pattern(7, &edges);
        let cl = reduce(&sp, 3, 7, 32).expect("one degree-three elimination leaves K6");
        assert_eq!(cl.prefix, vec![0]);
        assert_eq!(cl.prefix_flops, 16);
        assert_eq!(cl.core_ids, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(cl.core_nnz(), 30);
        assert_symmetric_core(&cl);

        for core_perm in [
            (0..cl.core_n()).collect::<Vec<_>>(),
            (0..cl.core_n()).rev().collect::<Vec<_>>(),
        ] {
            let whole = splice(&cl, &core_perm);
            assert_bijection(&whole, sp.n);
            let core_sp = ScoringPattern {
                n: cl.core_n(),
                col_ptr: cl.core_col_ptr.clone(),
                row_idx: cl.core_row_idx.clone(),
            };
            assert_eq!(
                flops(&sp, &whole),
                cl.prefix_flops + flops(&core_sp, &core_perm)
            );
        }
    }

    #[test]
    fn partial_reduction_and_splice_are_deterministic_bijections() {
        let mut edges = all_edges(4);
        edges.extend([(3, 4), (4, 5)]);
        let sp = pattern(6, &edges);
        let a = reduce(&sp, 1, 6, 8).expect("pendant chain leaves K4");
        let b = reduce(&sp, 1, 6, 8).expect("same reduction repeats");
        assert_eq!(a.prefix, b.prefix);
        assert_eq!(a.core_ids, b.core_ids);
        assert_eq!(a.core_col_ptr, b.core_col_ptr);
        assert_eq!(a.core_row_idx, b.core_row_idx);
        assert_eq!(a.prefix_flops, b.prefix_flops);
        assert_eq!(a.prefix, vec![5, 4]);
        assert_eq!(a.core_ids, vec![0, 1, 2, 3]);
        assert_symmetric_core(&a);
        let whole = splice(&a, &[3, 1, 0, 2]);
        assert_bijection(&whole, 6);
        assert_eq!(whole, vec![5, 4, 3, 1, 0, 2]);
    }

    #[test]
    fn core_size_edge_and_arithmetic_caps_fail_closed() {
        let k4 = pattern(4, &all_edges(4));
        assert!(reduce(&k4, 2, 3, 6).is_none());
        assert!(reduce(&k4, 2, 4, 5).is_none());
        assert!(reduce(&k4, usize::MAX, 4, 6).is_none());
        assert!(reduce(&k4, 2, 4, usize::MAX).is_none());
    }

    #[test]
    fn malformed_degree_accounting_fails_closed_in_release_path() {
        // Duplicate entries violate Pattern's upstream invariant. The reducer
        // must decline rather than relying on a debug-only assertion.
        let duplicate = ScoringPattern {
            n: 2,
            col_ptr: vec![0, 2, 4],
            row_idx: vec![1, 1, 0, 0],
        };
        assert!(reduce(&duplicate, 3, 2, 1).is_none());
    }

    #[test]
    fn clique_pair_budget_is_exact_and_fail_closed() {
        let mut edges = all_edges(7)
            .into_iter()
            .filter(|&(a, b)| {
                !((a == 0 && b >= 4)
                    || matches!((a, b), (1, 2) | (1, 3) | (2, 3)))
            })
            .collect::<Vec<_>>();
        edges.sort_unstable();
        let sp = pattern(7, &edges);
        assert!(reduce_checked(&sp, 3, 7, 32, 2).is_none());
        let cl = reduce_checked(&sp, 3, 7, 32, 3)
            .expect("one degree-three clique costs three pairs");
        assert_eq!(cl.prefix, vec![0]);
        assert_eq!(cl.core_nnz(), 30);
    }
}
