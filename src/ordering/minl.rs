//! COMPLETION-LATTICE DESCENT (MINL), ported from the SSI ordering challenge.
//!
//! The PEO-invariance theorem says `Σ c_j²` is a pure function of the chordal
//! completion, so the true search space is the lattice of completions of A
//! ordered by INCLUSION. Every candidate family in the portfolio — AMD
//! variants, partitioners, quotient metrics, relabelled restarts — builds a
//! completion FROM SCRATCH by a new elimination; the chains refine an order.
//! This phase moves DOWNWARD from the incumbent's completion inside the
//! lattice:
//!
//!   1. take the finished incumbent's filled graph `G+ = A ∪ fill(L)`;
//!   2. minimalize it by greedy fill-edge deletion — for chordal `H`,
//!      `H − uv` stays chordal iff `N_H(u) ∩ N_H(v)` is a clique (local and
//!      exact: a chordless cycle created by deleting `uv` must be a 4-cycle
//!      `u-x-v-y` with `x, y` common neighbours and `xy ∉ E`);
//!   3. realize the shrunken completion `M ⊆ G+` by a maximum-cardinality-
//!      search perfect elimination order (for minimal `M`, `fill(A, peo(M)) = M`
//!      exactly) and, as a second realization, AMD on `M`;
//!   4. the caller keeps a candidate only on a strict exact decrease.
//!
//! Chordality bookkeeping is not safety-critical: a wrong removal merely
//! yields a candidate the exact scorer rejects. Every operation of the scan
//! (the common-neighbourhood intersects as well as the clique-check rows) is
//! charged against one deterministic op budget with a hard break, so the
//! phase's cost is bounded whatever the structure; the filled-graph build
//! bails before any O(nnz(L)) work when the incumbent's fill exceeds the
//! gate. Cheapest-first scan order (ascending degree sum) spends the budget
//! where removals are densest; the crown's watcher variant (`minl_watch`)
//! is a different, witness-driven walk of the same lattice.

use super::*;

/// nnz(A) ceiling: above it the scan measured zero removals in the old
/// challenge (the budget exhausts before one edge of a completion that large
/// turns out to be removable).
pub(crate) const MINL_MAX_NNZ: usize = 700_000;
/// Incumbent fill ceiling, checked before the filled graph is built.
pub(crate) const MINL_MAX_LNNZ: usize = 600_000;
/// Deterministic op budget for the whole deletion scan.
pub(crate) const MINL_OPS_BUDGET: i64 = 40_000_000;
pub(crate) const MINL_MAX_ROUNDS: usize = 16;
pub(crate) const MINL_MAX_COMMON: usize = 4_000;
/// A second realization (AMD on the descended completion) only while the
/// completion is small enough for one AMD-class pass to be cheap.
pub(crate) const MINL_AMD_REALIZE_MAX_M: usize = 600_000;

fn filled_graph(pat: &ScoringPattern, perm: &[usize], max_lnnz: usize) -> Option<(Vec<i32>, Vec<i32>)> {
    let n = pat.n;
    let permuted = permute_pattern(pat, perm);
    let etree = EliminationTree::from_pattern(&permuted);
    let counts = column_counts_gnp(&permuted, &etree);
    let lnnz: usize = counts.iter().sum();
    if lnnz > max_lnnz {
        return None;
    }
    let mut children: Vec<Vec<u32>> = vec![Vec::new(); n];
    for j in 0..n {
        if let Some(p) = etree.parent[j] {
            children[p].push(j as u32);
        }
    }
    // TWO-PASS control form (V1): clique-merge once for degrees, once to
    // scatter, freeing each child column as soon as its parent consumes it.
    // Peak resident column memory is O(frontier), not O(nnz(L)) — the
    // single-pass fleet-r form retains EVERY merged column and is the prime
    // suspect for the hidden-ultra-giant cap failures. Output (cp, ri) is
    // byte-identical to the single-pass form.
    // SINGLE-PASS build (was two identical O(nnz(L)) merge sweeps: one for
    // degrees, one to emit). The caller's `max_lnnz` gate (checked above)
    // bounds retained column memory at ~4*lnnz bytes, so keeping every
    // column from the one merge and emitting from storage halves the
    // phase's dominant cost. Byte-identical output: same merge order, same
    // per-column emit order, same final per-vertex sort.
    let mut cols: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut mark = vec![u32::MAX; n];
    let mut deg = vec![0usize; n];
    let mut total_edges = 0usize;
    for j in 0..n {
        let mj = j as u32;
        let mut s: Vec<u32> = Vec::new();
        for k in permuted.col_ptr[j]..permuted.col_ptr[j + 1] {
            let i = permuted.row_idx[k];
            if i > j && mark[i] != mj {
                mark[i] = mj;
                s.push(i as u32);
            }
        }
        for &c in &children[j] {
            for &i in &cols[c as usize] {
                let iu = i as usize;
                if iu > j && mark[iu] != mj {
                    mark[iu] = mj;
                    s.push(i);
                }
            }
        }
        let vg = perm[j];
        for &i in &s {
            let wg = perm[i as usize];
            deg[vg] += 1;
            deg[wg] += 1;
            total_edges += 1;
        }
        cols[j] = s;
    }
    let mut cp: Vec<i32> = Vec::with_capacity(n + 1);
    cp.push(0);
    let mut acc = 0usize;
    for v in 0..n {
        acc += deg[v];
        cp.push(acc as i32);
    }
    let mut ri: Vec<i32> = vec![0; 2 * total_edges];
    let mut fill_pos: Vec<usize> = cp[..n].iter().map(|&x| x as usize).collect();
    for j in 0..n {
        let vg = perm[j];
        for &i in &cols[j] {
            let wg = perm[i as usize];
            ri[fill_pos[vg]] = wg as i32;
            fill_pos[vg] += 1;
            ri[fill_pos[wg]] = vg as i32;
            fill_pos[wg] += 1;
        }
    }
    for v in 0..n {
        let lo = cp[v] as usize;
        let hi = cp[v + 1] as usize;
        ri[lo..hi].sort_unstable();
    }
    Some((cp, ri))
}

/// Exact clique check on sorted adjacency for the completion-lattice
/// descent, budgeted. Fail-closed: a blown budget reports "not a clique",
/// which merely skips the candidate edge (the completion stays chordal).
fn minl_is_clique(adj: &[Vec<u32>], c: &[u32], ops: &mut i64) -> bool {
    let k = c.len();
    if k <= 1 {
        return true;
    }
    for &a in c {
        if adj[a as usize].len() < k - 1 {
            return false;
        }
    }
    for &a in c {
        let na = &adj[a as usize];
        *ops -= (na.len() + k) as i64;
        if *ops < 0 {
            return false;
        }
        let (mut i, mut j) = (0usize, 0usize);
        while i < na.len() && j < k {
            if na[i] < c[j] {
                i += 1;
            } else if na[i] > c[j] {
                if c[j] == a {
                    j += 1;
                } else {
                    return false;
                }
            } else {
                i += 1;
                j += 1;
            }
        }
        while j < k {
            if c[j] != a {
                return false;
            }
            j += 1;
        }
    }
    true
}

/// Maximum-cardinality search on sorted adjacency (O(n+m) bucket queue,
/// deterministic pop order). Returns the ELIMINATION order — the reverse
/// MCS visit order — which is a perfect elimination order whenever `adj`
/// is chordal. If `adj` is somehow not chordal the result is still a valid
/// bijection; the oracle simply rejects it.
fn minl_mcs_peo(n: usize, adj: &[Vec<u32>]) -> Vec<usize> {
    let mut w = vec![0u32; n];
    let mut vis = vec![false; n];
    let mut buckets: Vec<Vec<u32>> = vec![(0..n as u32).rev().collect()];
    let mut maxw = 0usize;
    let mut visit: Vec<usize> = Vec::with_capacity(n);
    while visit.len() < n {
        let Some(x) = buckets[maxw].pop() else {
            if maxw == 0 {
                break; // unreachable on well-formed input
            }
            maxw -= 1;
            continue;
        };
        let xu = x as usize;
        if vis[xu] || (w[xu] as usize) != maxw {
            continue;
        }
        vis[xu] = true;
        visit.push(xu);
        for &uw in &adj[xu] {
            let u = uw as usize;
            if !vis[u] {
                w[u] += 1;
                let nw = w[u] as usize;
                if nw >= buckets.len() {
                    buckets.resize_with(nw + 1, Vec::new);
                }
                buckets[nw].push(uw);
                if nw > maxw {
                    maxw = nw;
                }
            }
        }
    }
    visit.reverse();
    visit
}

/// Shared by BOTH completion-lattice descent phases (the MINL scan and
/// Component B's orbit-batched scan, see the `ORBIT_*` constants' doc
/// comment): test fill edge `uv` for deletability — `N_adj(u) ∩ N_adj(v)`
/// is a clique, the exact LOCAL chordality-preserving condition, see the
/// `MINL_*` constants' doc comment — and, if deletable, remove it from
/// `adj` in place. Charges the common-neighborhood intersect AND the
/// clique check to `*ops` (the caller's budget, shared across the whole
/// scan); fails closed (`false`, `adj` untouched) on budget exhaustion or a
/// too-large common neighborhood. Pure extraction of the original inline
/// MINL-scan body — behavior is byte-identical, only factored out so
/// Component B can reuse it without duplicating the pair logic.
fn minl_try_delete_edge(
    adj: &mut [Vec<u32>],
    u: u32,
    v: u32,
    ops: &mut i64,
    max_common: usize,
) -> bool {
    let (uu, vv) = (u as usize, v as usize);
    let (au, av) = (&adj[uu], &adj[vv]);
    // Common neighborhood, charged to the op budget.
    *ops -= (au.len() + av.len()) as i64;
    let mut c: Vec<u32> = Vec::with_capacity(au.len().min(av.len()));
    let (mut i, mut j) = (0usize, 0usize);
    while i < au.len() && j < av.len() {
        if au[i] < av[j] {
            i += 1;
        } else if au[i] > av[j] {
            j += 1;
        } else {
            c.push(au[i]);
            i += 1;
            j += 1;
        }
    }
    if c.len() > max_common {
        return false;
    }
    if !minl_is_clique(adj, &c, ops) {
        return false;
    }
    if let Ok(p) = adj[uu].binary_search(&v) {
        adj[uu].remove(p);
    }
    if let Ok(p) = adj[vv].binary_search(&u) {
        adj[vv].remove(p);
    }
    true
}

fn minl_stamp_neighbors(row: &[u32], marks: &mut [u32], stamp: &mut u32, ops: &mut i64) {
    *stamp = stamp.wrapping_add(1);
    if *stamp == 0 {
        *ops -= marks.len() as i64;
        marks.fill(0);
        *stamp = 1;
    }
    *ops -= row.len() as i64;
    for &w in row {
        marks[w as usize] = *stamp;
    }
}

// Every required clique neighbor must occur in this sorted adjacency row.
// Choose between probing only the required vertices and scanning the row.
// The caller retains its full-row logical charge whichever route is cheaper.
fn minl_member_contains_clique(na: &[u32], a: u32, c: &[u32], marks: &[u32], stamp: u32) -> bool {
    let required = c.len() - 1;
    let search_depth = usize::BITS - na.len().max(1).leading_zeros();
    if required.saturating_mul(search_depth as usize) < na.len() {
        return c.iter().all(|&w| w == a || na.binary_search(&w).is_ok());
    }
    let mut count = 0;
    for (i, &w) in na.iter().enumerate() {
        if marks[w as usize] == stamp {
            count += 1;
        }
        if count == required {
            return true;
        }
        if count + (na.len() - i - 1) < required {
            return false;
        }
    }
    count == required
}

/// One completion-lattice descent from `seed`: returns up to two candidate
/// orderings (MCS perfect elimination order of the minimalized completion,
/// and AMD on that completion), or `None` when no fill edge was removable or
/// a gate refused. Pure function of `(sp, seed)`.
pub(crate) fn minl_candidates(sp: &ScoringPattern, seed: &[usize]) -> Option<(Vec<Vec<usize>>, bool)> {
    let n = sp.n;
    let (fcp, fri) = filled_graph(sp, seed, MINL_MAX_LNNZ)?;
    let mut adj: Vec<Vec<u32>> = (0..n)
        .map(|v| fri[fcp[v] as usize..fcp[v + 1] as usize].iter().map(|&x| x as u32).collect())
        .collect();
    let asort: Vec<Vec<u32>> = (0..n)
        .map(|v| {
            let mut c: Vec<u32> = sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]]
                .iter()
                .map(|&x| x as u32)
                .filter(|&x| x as usize != v)
                .collect();
            c.sort_unstable();
            c
        })
        .collect();
    let mut fill: Vec<(u32, u32)> = Vec::new();
    for u in 0..n {
        for &wv in &adj[u] {
            if (wv as usize) > u && asort[u].binary_search(&wv).is_err() {
                fill.push((u as u32, wv));
            }
        }
    }
    if fill.is_empty() {
        return None;
    }
    // Scan order: coarse cheapest-first (log2 buckets of the degree sum), and
    // inside a bucket grouped by the lower endpoint `u`, so consecutive edges
    // share `N(u)` and its stamp is built once per group instead of once per
    // edge (the intersect then costs deg(v), not deg(u) + deg(v)).
    let mut scan_order: Vec<u32> = (0..fill.len() as u32).collect();
    {
        let key = |ei: u32| -> (u32, u32, u32) {
            let (u, v) = fill[ei as usize];
            let ds = (adj[u as usize].len() + adj[v as usize].len()) as u32;
            (32 - ds.max(1).leading_zeros(), u, v)
        };
        scan_order.sort_unstable_by_key(|&ei| key(ei));
    }
    let mut ops = MINL_OPS_BUDGET;
    let mut alive = vec![true; fill.len()];
    let mut removed_total = 0usize;
    // `false` when the op budget cut the descent short: the completion is
    // then not minimal and the (expensive) post-descent refinement is skipped.
    let mut completed = true;
    // Every group rebuild has its own epoch: a vertex can be revisited after
    // neighbors were deleted while processing another endpoint or bucket.
    let mut umark: Vec<u32> = vec![0; n];
    let mut ustamp: u32 = 0;
    let mut umark_for: u32 = u32::MAX;
    // `cmark[w] == stamp` ⇔ w ∈ c for the edge under test.
    let mut cmark: Vec<u32> = vec![0; n];
    let mut cstamp: u32 = 0;
    let mut c: Vec<u32> = Vec::new();
    // An edge uv that failed the test can become deletable only if
    // N(u) ∩ N(v) shrank, i.e. an edge at u or at v was deleted: later rounds
    // re-test only the edges with a DIRTY endpoint (exact, and it leaves the
    // op budget to the first full scan instead of to repeated no-op passes).
    let mut dirty_prev = vec![true; n];
    let mut dirty_next = vec![false; n];
    'rounds: for _round in 0..MINL_MAX_ROUNDS {
        let mut removed_this = 0usize;
        umark_for = u32::MAX;
        for &ei in &scan_order {
            let ei = ei as usize;
            if !alive[ei] {
                continue;
            }
            let (u, v) = fill[ei];
            if !dirty_prev[u as usize] && !dirty_prev[v as usize] {
                continue;
            }
            if ops < 0 {
                // Budget exhausted mid-round: the edges already deleted in
                // this round are a valid (partial) descent — keep them.
                removed_total += removed_this;
                completed = false;
                break 'rounds;
            }
            let (uu, vv) = (u as usize, v as usize);
            if umark_for != u {
                minl_stamp_neighbors(&adj[uu], &mut umark, &mut ustamp, &mut ops);
                umark_for = u;
            }
            // c = N(u) ∩ N(v), walking only N(v).
            ops -= adj[vv].len() as i64;
            c.clear();
            for &w in &adj[vv] {
                if umark[w as usize] == ustamp {
                    c.push(w);
                }
            }
            let k = c.len();
            if k > MINL_MAX_COMMON {
                continue;
            }
            let mut is_clique = true;
            if k >= 2 {
                // Degree pre-check, then the exact membership count per member,
                // poorest member first (the likeliest to expose a missing edge).
                for &a in &c {
                    if adj[a as usize].len() < k - 1 {
                        is_clique = false;
                        break;
                    }
                }
                if is_clique {
                    cstamp = cstamp.wrapping_add(1);
                    if cstamp == 0 {
                        for x in cmark.iter_mut() {
                            *x = 0;
                        }
                        cstamp = 1;
                    }
                    for &w in &c {
                        cmark[w as usize] = cstamp;
                    }
                    let mut order: Vec<u32> = c.clone();
                    order.sort_unstable_by_key(|&a| (adj[a as usize].len(), a));
                    'members: for &a in &order {
                        let na = &adj[a as usize];
                        ops -= na.len() as i64;
                        if !minl_member_contains_clique(na, a, &c, &cmark, cstamp) {
                            is_clique = false;
                            break 'members;
                        }
                        if ops < 0 {
                            is_clique = false;
                            break 'members;
                        }
                    }
                }
            }
            if !is_clique {
                continue;
            }
            if let Ok(p) = adj[uu].binary_search(&v) {
                adj[uu].remove(p);
            }
            if let Ok(p) = adj[vv].binary_search(&u) {
                adj[vv].remove(p);
            }
            umark[vv] = 0;
            alive[ei] = false;
            dirty_next[uu] = true;
            dirty_next[vv] = true;
            removed_this += 1;
        }
        if removed_this == 0 {
            break;
        }
        removed_total += removed_this;
        std::mem::swap(&mut dirty_prev, &mut dirty_next);
        for d in dirty_next.iter_mut() {
            *d = false;
        }
    }
    let _ = umark_for;
    if removed_total == 0 {
        return None;
    }
    let mut out: Vec<Vec<usize>> = vec![minl_mcs_peo(n, &adj)];
    let m: usize = adj.iter().map(|r| r.len()).sum();
    if m <= MINL_AMD_REALIZE_MAX_M {
        let mut cp: Vec<i32> = Vec::with_capacity(n + 1);
        let mut ri: Vec<i32> = Vec::with_capacity(m);
        cp.push(0);
        for row in &adj {
            ri.extend(row.iter().map(|&v| v as i32));
            cp.push(ri.len() as i32);
        }
        if let Some(core) = feral_ordering_core::CscPattern::new(n, &cp, &ri) {
            if let Ok(perm) = feral_amd::amd_order(&core) {
                out.push(perm.into_iter().map(|v| v as usize).collect());
            }
        }
    }
    Some((out, completed))
}

#[cfg(test)]
mod stamp_tests {
    use super::{minl_member_contains_clique, minl_stamp_neighbors};

    #[test]
    fn adaptive_clique_membership_matches_direct_set_test() {
        // Covers sparse probe and dense scan paths, both hits and misses.
        for mask in 1usize..256 {
            let c: Vec<u32> = (0..8).filter(|&w| mask & (1 << w) != 0).collect();
            if c.len() < 2 { continue; }
            let mut marks = [0u32; 64];
            for &w in &c { marks[w as usize] = 1; }
            for &a in &c {
                for missing in 0..=8 {
                    for extra in [0u32, 8, 32, 56] {
                        let na: Vec<u32> = (0..8 + extra)
                            .filter(|&w| w != a && w != missing)
                            .collect();
                        let expected = c.iter().all(|&w| w == a || na.contains(&w));
                        assert_eq!(minl_member_contains_clique(&na, a, &c, &marks, 1), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn rebuilding_after_other_endpoint_deletion_drops_stale_neighbor() {
        let mut marks = [0; 8];
        let mut epoch = 0;
        let mut ops = 100;
        minl_stamp_neighbors(&[1, 3, 5], &mut marks, &mut epoch, &mut ops);
        let previous = epoch;
        // Another group's processing can delete (u, 1) without clearing
        // marks[1] for u. Revisit u with its new adjacency and retain no ghosts.
        minl_stamp_neighbors(&[2, 6], &mut marks, &mut epoch, &mut ops);
        minl_stamp_neighbors(&[3, 5], &mut marks, &mut epoch, &mut ops);
        assert_ne!(epoch, previous);
        let actual: Vec<_> = (0..marks.len()).filter(|&i| marks[i] == epoch).collect();
        assert_eq!(actual, vec![3, 5]);
        assert_eq!(ops, 93);
    }

    #[test]
    fn epoch_wrap_clears_previous_marks_and_charges_clear() {
        let mut marks = [1, u32::MAX, 1, 0];
        let mut epoch = u32::MAX;
        let mut ops = 20;
        minl_stamp_neighbors(&[3], &mut marks, &mut epoch, &mut ops);
        assert_eq!(epoch, 1);
        assert_eq!(marks, [0, 0, 0, 1]);
        assert_eq!(ops, 15);
    }
}
