//! Flat exact symbolic analysis — one allocation-light replacement for the
//! vendor triple `permute_pattern` → `EliminationTree::from_pattern` →
//! `column_counts_gnp` (and the `postorder()` the subtree preambles take).
//!
//! Same algorithm as the vendor path (Liu's path-compressed union-find
//! elimination tree; ascending-child, ascending-root DFS postorder;
//! Gilbert–Ng–Peyton column counts) on flat `u32` arrays:
//!
//! - the permuted pattern is built UNSORTED (bucket fill, one pass) — the
//!   etree of a pattern is unique, and the GNP pass treats each off-diagonal
//!   partner of a row independently (per-partner `maxfirst`/`prevleaf`; a
//!   partner appears at most once per row; DSU path compression never changes
//!   a root), so column order cannot change `parent`, `post` or any count;
//! - the postorder visits roots in ascending index and each node's children
//!   in ascending index, exactly the order the vendor's push-in-ascending-`j`
//!   `children()` lists and ascending `roots()` produce;
//! - for the consumers that walk the permuted columns in order
//!   (`completion::refine_limited`, `peo_extract::candidates_bounded`,
//!   `minl::filled_graph`) the vendor's SORTED columns are reproduced by a
//!   single counting-sort transpose of the symmetric unsorted CSC (a symmetric
//!   pattern is its own transpose, and a transpose built by an ascending
//!   column sweep has ascending row indices in every column) — no sort.
//!
//! Pure function of `(pat, perm)`; per-call allocation only, no shared state.
//! `matches_vendor_triple_synthetic` below asserts counts, parent, postorder
//! AND the sorted permuted pattern against the vendor triple.

use super::ScoringPattern;

/// `(inv_perm, col_ptr, row_idx)` of `P·A·Pᵀ`, columns UNSORTED.
fn permute_unsorted(pat: &ScoringPattern, perm: &[usize]) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let n = pat.n;
    let nnz = pat.col_ptr[n];
    let mut inv = vec![0u32; n];
    for (new, &old) in perm.iter().enumerate() {
        inv[old] = new as u32;
    }
    let mut cp = vec![0u32; n + 1];
    for old_j in 0..n {
        let new_j = inv[old_j] as usize;
        cp[new_j + 1] = (pat.col_ptr[old_j + 1] - pat.col_ptr[old_j]) as u32;
    }
    for j in 0..n {
        cp[j + 1] += cp[j];
    }
    let mut ri = vec![0u32; nnz];
    let mut off: Vec<u32> = cp[..n].to_vec();
    for old_j in 0..n {
        let new_j = inv[old_j] as usize;
        for k in pat.col_ptr[old_j]..pat.col_ptr[old_j + 1] {
            let pos = off[new_j] as usize;
            ri[pos] = inv[pat.row_idx[k]];
            off[new_j] += 1;
        }
    }
    (inv, cp, ri)
}

/// Transpose of a symmetric unsorted CSC = the same pattern with every column
/// ascending (the vendor `permute_pattern` invariant), as `usize` arrays.
fn transpose_sorted(n: usize, cp: &[u32], ri: &[u32]) -> (Vec<usize>, Vec<usize>) {
    let nnz = cp[n] as usize;
    let mut tp = vec![0usize; n + 1];
    for &r in &ri[..nnz] {
        tp[r as usize + 1] += 1;
    }
    for j in 0..n {
        tp[j + 1] += tp[j];
    }
    let mut ti = vec![0usize; nnz];
    let mut off: Vec<usize> = tp[..n].to_vec();
    for j in 0..n {
        for k in cp[j] as usize..cp[j + 1] as usize {
            let r = ri[k] as usize;
            ti[off[r]] = j;
            off[r] += 1;
        }
    }
    (tp, ti)
}

/// Elimination tree (`-1` = root), child CSR (ascending), postorder.
struct Tree {
    parent: Vec<i32>,
    child_ptr: Vec<u32>,
    post: Vec<u32>,
}

fn etree_and_post(n: usize, cp: &[u32], ri: &[u32]) -> Tree {
    // Liu etree, path-compressed union-find (vendor `from_pattern`).
    let mut parent = vec![-1i32; n];
    let mut anc = vec![0u32; n];
    for j in 0..n {
        anc[j] = j as u32;
        for k in cp[j] as usize..cp[j + 1] as usize {
            let i = ri[k] as usize;
            if i >= j {
                continue;
            }
            let mut r = i;
            while anc[r] as usize != r {
                r = anc[r] as usize;
            }
            let mut node = i;
            while node != r {
                let next = anc[node] as usize;
                anc[node] = r as u32;
                node = next;
            }
            if r != j {
                parent[r] = j as i32;
                anc[r] = j as u32;
            }
        }
    }
    // Child CSR by counting sort; filling in ascending j makes each parent's
    // slice ascending (the vendor `children[p].push(j)` order).
    let mut child_ptr = vec![0u32; n + 1];
    for j in 0..n {
        let p = parent[j];
        if p >= 0 {
            child_ptr[p as usize + 1] += 1;
        }
    }
    for j in 0..n {
        child_ptr[j + 1] += child_ptr[j];
    }
    let mut child_idx = vec![0u32; n];
    let mut cursor: Vec<u32> = child_ptr[..n].to_vec();
    for j in 0..n {
        let p = parent[j];
        if p >= 0 {
            let pos = cursor[p as usize] as usize;
            child_idx[pos] = j as u32;
            cursor[p as usize] += 1;
        }
    }
    // Iterative DFS postorder, roots ascending (vendor `postorder()`).
    let mut next_child = vec![0u32; n];
    let mut stack: Vec<u32> = Vec::with_capacity(n);
    let mut post: Vec<u32> = Vec::with_capacity(n);
    for root in 0..n {
        if parent[root] >= 0 {
            continue;
        }
        stack.push(root as u32);
        while let Some(&node) = stack.last() {
            let node = node as usize;
            let k = next_child[node];
            let kids = child_ptr[node + 1] - child_ptr[node];
            if k < kids {
                next_child[node] = k + 1;
                stack.push(child_idx[(child_ptr[node] + k) as usize]);
            } else {
                post.push(node as u32);
                stack.pop();
            }
        }
    }
    Tree { parent, child_ptr, post }
}

/// Gilbert–Ng–Peyton column counts (vendor `column_counts_gnp`).
fn counts_gnp(n: usize, cp: &[u32], ri: &[u32], t: &Tree) -> Vec<usize> {
    let mut post_of = vec![0u32; n];
    for (pnum, &node) in t.post.iter().enumerate() {
        post_of[node as usize] = pnum as u32;
    }
    let mut first = post_of;
    for &node in &t.post {
        let node = node as usize;
        let p = t.parent[node];
        if p >= 0 && first[node] < first[p as usize] {
            first[p as usize] = first[node];
        }
    }
    let mut delta: Vec<i64> = (0..n)
        .map(|i| i64::from(t.child_ptr[i + 1] == t.child_ptr[i]))
        .collect();
    let mut maxfirst = vec![-1i64; n];
    let mut prevleaf = vec![-1i64; n];
    let mut anc: Vec<u32> = (0..n as u32).collect();
    for &i in &t.post {
        let i = i as usize;
        let pi = t.parent[i];
        if pi >= 0 {
            delta[pi as usize] -= 1;
        }
        let fi = first[i] as i64;
        for k in cp[i] as usize..cp[i + 1] as usize {
            let partner = ri[k] as usize;
            if partner <= i {
                continue;
            }
            if fi > maxfirst[partner] {
                delta[i] += 1;
                let pl = prevleaf[partner];
                if pl != -1 {
                    let mut q = pl as usize;
                    while anc[q] as usize != q {
                        q = anc[q] as usize;
                    }
                    let root = q;
                    let mut cur = pl as usize;
                    while cur != root {
                        let next = anc[cur] as usize;
                        anc[cur] = root as u32;
                        cur = next;
                    }
                    delta[root] -= 1;
                }
                prevleaf[partner] = i as i64;
                maxfirst[partner] = fi;
            }
        }
        if pi >= 0 {
            anc[i] = pi as u32;
        }
    }
    for &i in &t.post {
        let i = i as usize;
        let p = t.parent[i];
        if p >= 0 {
            delta[p as usize] += delta[i];
        }
    }
    delta.into_iter().map(|d| d as usize).collect()
}

fn fits_u32(pat: &ScoringPattern) -> bool {
    pat.n < u32::MAX as usize && pat.col_ptr[pat.n] < u32::MAX as usize
}

/// Vendor fallback for a pattern outside the `u32` index range (never reached
/// on this corpus; keeps the panic surface identical to the vendor path).
fn vendor_triple(pat: &ScoringPattern, perm: &[usize]) -> (ScoringPattern, Vec<Option<usize>>, Vec<usize>, Vec<usize>) {
    let pp = feral::ordering::amd::permute_pattern(pat, perm);
    let et = feral::ordering::elimination_tree::EliminationTree::from_pattern(&pp);
    let counts = feral::symbolic::column_counts_gnp(&pp, &et);
    let post = et.postorder();
    (pp, et.parent, counts, post)
}

/// `(counts, parent, post)` of `perm` on `pat`; `parent[j] == usize::MAX` marks a root.
pub(super) fn analyze(pat: &ScoringPattern, perm: &[usize]) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    if !fits_u32(pat) {
        let (_, parent, counts, post) = vendor_triple(pat, perm);
        return (counts, parent.iter().map(|p| p.unwrap_or(usize::MAX)).collect(), post);
    }
    let n = pat.n;
    let (_inv, cp, ri) = permute_unsorted(pat, perm);
    let t = etree_and_post(n, &cp, &ri);
    let counts = counts_gnp(n, &cp, &ri, &t);
    let parent: Vec<usize> = t.parent.iter().map(|&p| if p < 0 { usize::MAX } else { p as usize }).collect();
    let post: Vec<usize> = t.post.iter().map(|&x| x as usize).collect();
    (counts, parent, post)
}

/// `Σ_j c_j²` of `perm` on `pat` — the grader's quantity.
pub(super) fn flops(pat: &ScoringPattern, perm: &[usize]) -> u64 {
    if !fits_u32(pat) {
        let (_, _, counts, _) = vendor_triple(pat, perm);
        return counts.iter().map(|&c| (c as u64) * (c as u64)).sum();
    }
    let n = pat.n;
    let (_inv, cp, ri) = permute_unsorted(pat, perm);
    let t = etree_and_post(n, &cp, &ri);
    let counts = counts_gnp(n, &cp, &ri, &t);
    counts.iter().map(|&c| (c as u64) * (c as u64)).sum()
}

/// The permuted pattern with SORTED columns (vendor invariant), the etree
/// parent as `Option<usize>`, and the column counts — for the consumers that
/// walk the permuted columns in order.
pub(super) fn analyze_sorted(pat: &ScoringPattern, perm: &[usize]) -> (ScoringPattern, Vec<Option<usize>>, Vec<usize>) {
    if !fits_u32(pat) {
        let (pp, parent, counts, _) = vendor_triple(pat, perm);
        return (pp, parent, counts);
    }
    let n = pat.n;
    let (_inv, cp, ri) = permute_unsorted(pat, perm);
    let t = etree_and_post(n, &cp, &ri);
    let counts = counts_gnp(n, &cp, &ri, &t);
    let (col_ptr, row_idx) = transpose_sorted(n, &cp, &ri);
    let parent: Vec<Option<usize>> = t.parent.iter().map(|&p| if p < 0 { None } else { Some(p as usize) }).collect();
    (ScoringPattern { n, col_ptr, row_idx }, parent, counts)
}

/// The subtree-refine preamble: postorder `perm`'s etree into a candidate,
/// then the candidate's counts (`u32`) and etree parent (`i32`, `-1` = root).
pub(super) fn prep_subtree(pat: &ScoringPattern, perm: &[usize]) -> (Vec<usize>, Vec<u32>, Vec<i32>) {
    if !fits_u32(pat) {
        let (_, _, _, post) = vendor_triple(pat, perm);
        let candidate: Vec<usize> = post.iter().map(|&j| perm[j]).collect();
        let (_, parent, counts, _) = vendor_triple(pat, &candidate);
        return (
            candidate,
            counts.into_iter().map(|c| c as u32).collect(),
            parent.iter().map(|p| p.map_or(-1, |j| j as i32)).collect(),
        );
    }
    let n = pat.n;
    let (_inv, cp, ri) = permute_unsorted(pat, perm);
    let t = etree_and_post(n, &cp, &ri);
    let candidate: Vec<usize> = t.post.iter().map(|&j| perm[j as usize]).collect();
    let (_inv2, cp2, ri2) = permute_unsorted(pat, &candidate);
    let t2 = etree_and_post(n, &cp2, &ri2);
    let counts: Vec<u32> = counts_gnp(n, &cp2, &ri2, &t2).into_iter().map(|c| c as u32).collect();
    (candidate, counts, t2.parent)
}

/// Env-gated per-call audit against the vendor triple (probe builds only).
/// Set `SSI_FLAT_AUDIT=1` to compare every `analyze_sorted`/`prep_subtree`/
/// `flops` result with the vendor path and abort on the first mismatch.
#[cfg(test)]
pub(super) fn audit_enabled() -> bool {
    std::env::var("SSI_FLAT_AUDIT").map_or(false, |v| v == "1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pattern;
    use feral::ordering::amd::permute_pattern;
    use feral::ordering::elimination_tree::EliminationTree;
    use feral::symbolic::column_counts_gnp;

    fn scoring_pattern(pattern: &Pattern) -> ScoringPattern {
        ScoringPattern {
            n: pattern.n,
            col_ptr: pattern.col_ptr.clone(),
            row_idx: pattern.row_idx.clone(),
        }
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *state >> 33
    }

    fn random_perm(n: usize, state: &mut u64) -> Vec<usize> {
        let mut p: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = (lcg(state) as usize) % (i + 1);
            p.swap(i, j);
        }
        p
    }

    #[test]
    fn matches_vendor_triple_synthetic() {
        let mut state = 0x9e3779b97f4a7c15u64;
        for n in [1usize, 2, 3, 5, 8, 13, 21, 40, 64, 97, 150, 257] {
            for density in [0usize, 1, 2, 4, 8] {
                let mut edges = Vec::new();
                for i in 0..n {
                    for j in i + 1..n {
                        if density > 0 && (lcg(&mut state) % 16) < density as u64 {
                            edges.push((i, j));
                        }
                    }
                }
                let pat = Pattern::from_edges(n, &edges);
                let sp = scoring_pattern(&pat);
                for trial in 0..6 {
                    let perm: Vec<usize> = if trial == 0 { (0..n).collect() } else { random_perm(n, &mut state) };
                    let pp = permute_pattern(&sp, &perm);
                    let et = EliminationTree::from_pattern(&pp);
                    let vc = column_counts_gnp(&pp, &et);
                    let vpost = et.postorder();
                    let vparent_opt = et.parent.clone();
                    let vflops: u64 = vc.iter().map(|&c| (c as u64) * (c as u64)).sum();

                    let (fc, fparent, fpost) = analyze(&sp, &perm);
                    assert_eq!(fc, vc, "counts n={n} d={density} t={trial}");
                    assert_eq!(fpost, vpost, "post n={n} d={density} t={trial}");
                    let vparent: Vec<usize> = vparent_opt.iter().map(|p| p.unwrap_or(usize::MAX)).collect();
                    assert_eq!(fparent, vparent, "parent n={n} d={density} t={trial}");
                    assert_eq!(flops(&sp, &perm), vflops, "flops n={n} d={density} t={trial}");

                    let (spp, sparent, sc) = analyze_sorted(&sp, &perm);
                    assert_eq!(spp.n, pp.n);
                    assert_eq!(spp.col_ptr, pp.col_ptr, "col_ptr n={n} d={density} t={trial}");
                    assert_eq!(spp.row_idx, pp.row_idx, "row_idx n={n} d={density} t={trial}");
                    assert_eq!(sparent, vparent_opt);
                    assert_eq!(sc, vc);

                    // prep_subtree vs the two-triple vendor preamble
                    let candidate: Vec<usize> = vpost.iter().map(|&j| perm[j]).collect();
                    let pp2 = permute_pattern(&sp, &candidate);
                    let et2 = EliminationTree::from_pattern(&pp2);
                    let vc2: Vec<u32> = column_counts_gnp(&pp2, &et2).into_iter().map(|c| c as u32).collect();
                    let vp2: Vec<i32> = et2.parent.iter().map(|p| p.map_or(-1, |j| j as i32)).collect();
                    let (fcand, fc2, fp2) = prep_subtree(&sp, &perm);
                    assert_eq!(fcand, candidate, "cand n={n} d={density} t={trial}");
                    assert_eq!(fc2, vc2, "cand counts n={n} d={density} t={trial}");
                    assert_eq!(fp2, vp2, "cand parent n={n} d={density} t={trial}");
                }
            }
        }
    }
}
