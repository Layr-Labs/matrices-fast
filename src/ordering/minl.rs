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
pub(crate) const MINL_MAX_LNNZ: usize = 1_500_000;
/// Deterministic op budget for the whole deletion scan.
pub(crate) const MINL_OPS_BUDGET: i64 = 40_000_000;
pub(crate) const MINL_MAX_ROUNDS: usize = 8;
pub(crate) const MINL_MAX_COMMON: usize = 2_000;
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

/// One completion-lattice descent from `seed`: returns up to two candidate
/// orderings (MCS perfect elimination order of the minimalized completion,
/// and AMD on that completion), or `None` when no fill edge was removable or
/// a gate refused. Pure function of `(sp, seed)`.
pub(crate) fn minl_candidates(sp: &ScoringPattern, seed: &[usize]) -> Option<Vec<Vec<usize>>> {
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
    // Cheapest-first: counting sort by degree sum, stable in (u, v) id order.
    let scan_order: Vec<u32> = {
        let mut keys: Vec<usize> = Vec::with_capacity(fill.len());
        let mut maxk = 0usize;
        for &(u, v) in &fill {
            let k = adj[u as usize].len() + adj[v as usize].len();
            maxk = maxk.max(k);
            keys.push(k);
        }
        let mut cnt = vec![0u32; maxk + 2];
        for &k in &keys {
            cnt[k + 1] += 1;
        }
        for i in 1..cnt.len() {
            cnt[i] += cnt[i - 1];
        }
        let mut out = vec![0u32; fill.len()];
        for (ei, &k) in keys.iter().enumerate() {
            out[cnt[k] as usize] = ei as u32;
            cnt[k] += 1;
        }
        out
    };
    let mut ops = MINL_OPS_BUDGET;
    let mut alive = vec![true; fill.len()];
    let mut removed_total = 0usize;
    'rounds: for _round in 0..MINL_MAX_ROUNDS {
        let mut removed_this = 0usize;
        for &ei in &scan_order {
            let ei = ei as usize;
            if !alive[ei] {
                continue;
            }
            if ops < 0 {
                break 'rounds;
            }
            let (u, v) = fill[ei];
            if !minl_try_delete_edge(&mut adj, u, v, &mut ops, MINL_MAX_COMMON) {
                continue;
            }
            alive[ei] = false;
            removed_this += 1;
        }
        if removed_this == 0 {
            break;
        }
        removed_total += removed_this;
    }
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
    Some(out)
}
