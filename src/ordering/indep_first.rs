//! INDEPENDENT-SET-FIRST LIFT — the "normal equations" ordering family.
//!
//! KKT / saddle-point patterns are (nearly) bipartite: constraint rows are
//! pairwise non-adjacent (the (2,2) block is diagonal or zero) and, for LP /
//! separable-QP KKTs, so are the variables. Minimum degree interleaves the two
//! sides; the classic alternative is to eliminate one whole side first (the
//! Schur complement / normal-equations route) and order what is left.
//!
//! For ANY independent set `X` the elimination of `X` first has an ORDER-FREE
//! cost: no eliminated vertex is adjacent to another member of `X`, so no fill
//! edge ever reaches a member of `X`, every `x ∈ X` is eliminated with exactly
//! its original neighbourhood, and
//!
//! ```text
//!   Σ_j c_j²  =  Σ_{x ∈ X} (1 + deg x)²  +  Σ_{w ∈ core} c_w²
//! ```
//!
//! where the core is `V \ X` carrying the original edges plus one clique per
//! `N(x)`. The first term is fixed, the second is computed on the core alone,
//! so core candidates rank exactly on the core graph (as in `core_lift`).
//!
//! DETERMINISM. Sides come from a BFS 2-colouring in ascending vertex order;
//! the greedy independent set walks `(degree, index)`; the edge set is a hash
//! SET that is only membership-tested, never iterated. Same input ⇒ same lift.

use super::core_lift::EdgeHasher;
use feral::sparse::csc::CscPattern as ScoringPattern;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;

type EdgeSet = HashSet<u64, BuildHasherDefault<EdgeHasher>>;

/// Core size above which only the AMD pass runs (see `run`).
const GIANT_CORE_NNZ: usize = 600_000;
/// METIS-on-core envelope in core NODES first (see `run`): ~0.2 s worst.
const METIS_CORE_MAX_N: usize = 30_000;
const METIS_CORE_MAX_NNZ: usize = 1_000_000;

#[inline]
fn key(a: u32, b: u32) -> u64 {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    ((lo as u64) << 32) | (hi as u64)
}

/// A fixed independent-set prefix plus the exact residual core.
pub(crate) struct IndepLift {
    /// Eliminated vertices (the independent set), original ids.
    pub(crate) prefix: Vec<usize>,
    /// Surviving vertices, ascending original id.
    pub(crate) core_ids: Vec<usize>,
    pub(crate) core_col_ptr: Vec<usize>,
    pub(crate) core_row_idx: Vec<usize>,
    /// Σ (1 + deg x)² over the prefix.
    pub(crate) prefix_flops: u64,
}

impl IndepLift {
    #[inline]
    pub(crate) fn core_n(&self) -> usize {
        self.core_ids.len()
    }
    #[inline]
    pub(crate) fn core_nnz(&self) -> usize {
        self.core_row_idx.len()
    }
}

/// BFS 2-colouring. `Some(colour)` when the whole graph is bipartite.
/// Probe-only today: the greedy set subsumes the whole-side sets on every
/// measured bipartite row (see `memory/experiments/0150-*`).
#[allow(dead_code)]
pub(crate) fn bipartite_sides(sp: &ScoringPattern) -> Option<Vec<u8>> {
    let n = sp.n;
    let mut colour: Vec<u8> = vec![u8::MAX; n];
    let mut queue: Vec<usize> = Vec::new();
    for s in 0..n {
        if colour[s] != u8::MAX {
            continue;
        }
        colour[s] = 0;
        queue.clear();
        queue.push(s);
        let mut head = 0;
        while head < queue.len() {
            let v = queue[head];
            head += 1;
            let cv = colour[v];
            for &w in &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]] {
                if colour[w] == u8::MAX {
                    colour[w] = 1 - cv;
                    queue.push(w);
                } else if colour[w] == cv {
                    return None;
                }
            }
        }
    }
    Some(colour)
}

/// Greedy maximal independent set by ascending `(degree, index)`, restricted
/// to vertices of degree ≤ `max_deg`.
pub(crate) fn greedy_independent_set(sp: &ScoringPattern, max_deg: usize) -> Vec<bool> {
    let n = sp.n;
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&v| (sp.col_ptr[v + 1] - sp.col_ptr[v], v));
    let mut in_x = vec![false; n];
    let mut blocked = vec![false; n];
    for v in order {
        let d = sp.col_ptr[v + 1] - sp.col_ptr[v];
        if d > max_deg {
            break;
        }
        if blocked[v] {
            continue;
        }
        in_x[v] = true;
        for &w in &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]] {
            blocked[w] = true;
        }
    }
    in_x
}

/// Restrict a candidate independent set to its cheapest members: admit
/// vertices in ascending `(degree, index)` while the clique-pair budget
/// (Σ deg·(deg−1)/2) lasts. Hubs stay in the core, where they belong.
pub(crate) fn budget_trim(sp: &ScoringPattern, in_x: &mut [bool], max_pairs: u64) {
    let n = sp.n;
    let mut members: Vec<usize> = (0..n).filter(|&v| in_x[v]).collect();
    members.sort_by_key(|&v| (sp.col_ptr[v + 1] - sp.col_ptr[v], v));
    let mut left = max_pairs;
    for v in members {
        let d = (sp.col_ptr[v + 1] - sp.col_ptr[v]) as u64;
        let pairs = d * d.saturating_sub(1) / 2;
        if pairs > left {
            in_x[v] = false;
        } else {
            left -= pairs;
        }
    }
}

/// Eliminate the independent set `in_x` first and build the exact residual
/// core. `None` if `in_x` is not independent, the core is empty, or the core
/// exceeds `max_core_edges`.
pub(crate) fn lift(sp: &ScoringPattern, in_x: &[bool], max_core_edges: usize) -> Option<IndepLift> {
    let n = sp.n;
    if n == 0 || n > u32::MAX as usize || in_x.len() != n {
        return None;
    }
    let mut prefix: Vec<usize> = Vec::new();
    let mut prefix_flops: u64 = 0;
    for v in 0..n {
        if !in_x[v] {
            continue;
        }
        let nb = &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]];
        if nb.iter().any(|&w| in_x[w]) {
            return None;
        }
        let cv = nb.len() as u64 + 1;
        prefix_flops = prefix_flops.checked_add(cv.checked_mul(cv)?)?;
        prefix.push(v);
    }
    let core_ids: Vec<usize> = (0..n).filter(|&v| !in_x[v]).collect();
    let core_n = core_ids.len();
    if core_n == 0 || prefix.is_empty() {
        return None;
    }
    let mut pos_of: Vec<u32> = vec![u32::MAX; n];
    for (k, &v) in core_ids.iter().enumerate() {
        pos_of[v] = k as u32;
    }
    // Core adjacency: original core-core edges plus a clique on each N(x).
    let mut nbrs: Vec<Vec<u32>> = vec![Vec::new(); core_n];
    let mut edges: EdgeSet = EdgeSet::with_capacity_and_hasher(sp.row_idx.len() / 2 + 16, Default::default());
    let mut total_edges: usize = 0;
    for (k, &v) in core_ids.iter().enumerate() {
        for &w in &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]] {
            if in_x[w] {
                continue;
            }
            let kw = pos_of[w];
            if kw as usize > k && edges.insert(key(k as u32, kw)) {
                nbrs[k].push(kw);
                nbrs[kw as usize].push(k as u32);
                total_edges += 1;
            }
        }
    }
    let mut live: Vec<u32> = Vec::new();
    for &x in &prefix {
        live.clear();
        live.extend(sp.row_idx[sp.col_ptr[x]..sp.col_ptr[x + 1]].iter().map(|&w| pos_of[w]));
        for i in 0..live.len() {
            for j in (i + 1)..live.len() {
                let (a, b) = (live[i], live[j]);
                if edges.insert(key(a, b)) {
                    nbrs[a as usize].push(b);
                    nbrs[b as usize].push(a);
                    total_edges += 1;
                }
            }
        }
        if total_edges > max_core_edges {
            return None;
        }
    }
    let mut core_col_ptr: Vec<usize> = Vec::with_capacity(core_n + 1);
    let mut core_row_idx: Vec<usize> = Vec::with_capacity(total_edges * 2);
    core_col_ptr.push(0);
    for k in 0..core_n {
        let row = &mut nbrs[k];
        row.sort_unstable();
        row.dedup();
        core_row_idx.extend(row.iter().map(|&x| x as usize));
        core_col_ptr.push(core_row_idx.len());
    }
    Some(IndepLift { prefix, core_ids, core_col_ptr, core_row_idx, prefix_flops })
}

/// Predicted clique-pair insertions for eliminating `in_x` first — the cost
/// driver of `lift` and an upper bound on the fill it adds to the core.
pub(crate) fn predicted_pairs(sp: &ScoringPattern, in_x: &[bool]) -> u64 {
    (0..sp.n)
        .filter(|&v| in_x[v])
        .map(|v| {
            let d = (sp.col_ptr[v + 1] - sp.col_ptr[v]) as u64;
            d * d.saturating_sub(1) / 2
        })
        .sum()
}

/// Work-ledgered production driver. Tries the greedy maximal independent set
/// at three degree caps (unbounded, 9, 3 — distinct Schur complements: the
/// unbounded set is the whole-side / normal-equations route, the capped ones
/// keep mid-degree vertices in the core), orders each exact core with AMD and
/// AMF, ranks on the core, and returns the cheapest spliced ordering with its
/// exact total. Every step is charged to `ledger` in edge-touch units, so the
/// added time is bounded by structure alone: a set whose predicted pairs or
/// core would exceed the remaining allowance is skipped, never started.
pub(crate) fn run(sp: &ScoringPattern, ledger: u64) -> Option<(u64, Vec<usize>)> {
    let n = sp.n;
    let nnz = sp.row_idx.len();
    if n < 32 || nnz == 0 {
        return None;
    }
    // Admission is decided up front from the pattern alone: the sets run on
    // their own threads, so each gets the whole ledger, and a set is trimmed
    // (hubs back into the core) until its predicted lift + core work fits:
    // lift ~ nnz + pairs, core ≤ nnz + 2·pairs, walked by two ordering passes
    // and two exact scorings, i.e. 5·nnz + 9·pairs ≤ ledger. Two sets with
    // identical size and pair sum are (in practice) the same set; the Schur
    // complement is not paid for twice.
    let share = ledger;
    let max_pairs = share.saturating_sub(5 * nnz as u64) / 9;
    if max_pairs == 0 {
        return None;
    }
    let mut admitted: Vec<Vec<bool>> = Vec::new();
    let mut seen_sizes: Vec<(usize, u64)> = Vec::new();
    // iter155a: extra caps 5 and 15 for more Schur shapes
    for cap in [usize::MAX, 15usize, 9usize, 5usize, 3usize] {
        let mut in_x = greedy_independent_set(sp, cap);
        budget_trim(sp, &mut in_x, max_pairs);
        let xs = in_x.iter().filter(|&&b| b).count();
        if xs == 0 || xs == n {
            continue;
        }
        let pairs = predicted_pairs(sp, &in_x);
        if seen_sizes.contains(&(xs, pairs)) {
            continue;
        }
        seen_sizes.push((xs, pairs));
        let lift_cost = nnz as u64 + pairs;
        let core_bound = nnz as u64 + 2 * pairs;
        if lift_cost.saturating_add(4 * core_bound) > share {
            continue;
        }
        admitted.push(in_x);
    }
    if admitted.is_empty() {
        return None;
    }
    let max_core_edges = (share / 4) as usize;
    let eval_set = |in_x: &[bool]| -> Option<(u64, Vec<usize>)> {
        let il = lift(sp, in_x, max_core_edges)?;
        let cn = il.core_n();
        if cn < 2 || 4 * il.core_nnz() as u64 > share {
            return None;
        }
        let core_pat = ScoringPattern { n: cn, col_ptr: il.core_col_ptr.clone(), row_idx: il.core_row_idx.clone() };
        let ccp: Vec<i32> = il.core_col_ptr.iter().map(|&x| i32::try_from(x).ok()).collect::<Option<_>>()?;
        let cri: Vec<i32> = il.core_row_idx.iter().map(|&x| i32::try_from(x).ok()).collect::<Option<_>>()?;
        let run_pass = |k: usize| -> Option<(u64, Vec<usize>)> {
            let ccore = feral_ordering_core::CscPattern::new(cn, &ccp, &cri)?;
            let p: Vec<i32> = match k {
                0 => feral_amd::amd_order(&ccore).ok()?,
                1 => {
                    let o = feral_amf::AmfOptions { dense_alpha: 10.0, ..Default::default() };
                    feral_amf::amf_order_opts(&ccore, &o).ok()?.0
                }
                _ => feral_metis::metis_order_full(&ccore, &feral_metis::MetisOptions::default()).ok()?.0,
            };
            let cp: Vec<usize> = p.into_iter().map(|x| x as usize).collect();
            if !super::is_bijection(&cp, cn) {
                return None;
            }
            let f = super::flops_of(&core_pat, &cp);
            Some((f, cp))
        };
        // Pass selection by core shape:
        //  * AMD always (one AMD-speed walk).
        //  * AMF only on sparse, non-giant cores: above `GIANT_CORE_NNZ` the
        //    pass alone costs 0.15-0.3 s (cache misses; hub cores worse), and
        //    on dense cores (nnz >= 20 n) it never beat AMD or METIS.
        //  * METIS when the core is small enough in NODES: its cost tracks the
        //    node count and hub structure, not nnz (measured: 12k-node /
        //    872k-nnz pooling core 0.13 s; 138k-node faclay core 4.3 s; 200k-
        //    node acopf core 1.1 s), so the gate is `cn <= 30k`, `nnz <= 1M`,
        //    ~0.2 s worst. Nested dissection on the Schur complement is where
        //    the family's largest wins are: arki0013 0.586 -> 0.439 (AMD/AMF on
        //    the same core 0.62), pooling_sppc3pq 0.392 -> 0.283, sppc1pq
        //    0.190 -> 0.168.
        let cnnz = il.core_nnz();
        let dense = cnnz >= 20 * cn;
        let use_amf = cnnz <= GIANT_CORE_NNZ && !dense;
        let use_metis = cn <= METIS_CORE_MAX_N && cnnz <= METIS_CORE_MAX_NNZ;
        let mut pass_ids: Vec<usize> = vec![0];
        if use_amf {
            pass_ids.push(1);
        }
        if use_metis {
            pass_ids.push(2);
        }
        let mut best_here: Option<(u64, Vec<usize>)> = None;
        for k in pass_ids {
            if let Some((f, cp)) = run_pass(k) {
                let total = il.prefix_flops.saturating_add(f);
                if best_here.as_ref().map_or(true, |(bf, _)| total < *bf) {
                    best_here = Some((total, splice(&il, &cp)));
                }
            }
        }
        // iter158a: (no Scotch — 156g/157a Scotch family failed hidden) 2-seed relabelled AMF on small sparse Schur cores (0143 untested follow-up)
        if use_amf && cn <= 3_000 && cnnz <= 30_000 {
            let mut inv = vec![0usize; cn];
            let mix = |mut x: u64| -> u64 {
                x = x.wrapping_add(0x9E3779B97F4A7C15);
                x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
                x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
                x ^ (x >> 31)
            };
            for seed in 1u64..=2 {
                let mut q: Vec<usize> = (0..cn).collect();
                let mut s = seed;
                for i in (1..cn).rev() {
                    s = mix(s);
                    let j = (s as usize) % (i + 1);
                    q.swap(i, j);
                }
                for (ni, &ov) in q.iter().enumerate() {
                    inv[ov] = ni;
                }
                let mut b_ptr: Vec<usize> = Vec::with_capacity(cn + 1);
                let mut b_idx: Vec<usize> = Vec::with_capacity(cnnz);
                b_ptr.push(0);
                for &old in &q {
                    let s0 = il.core_col_ptr[old];
                    let s1 = il.core_col_ptr[old + 1];
                    let mut col: Vec<usize> = il.core_row_idx[s0..s1].iter().map(|&w| inv[w]).collect();
                    col.sort_unstable();
                    b_idx.extend(col);
                    b_ptr.push(b_idx.len());
                }
                let bcp: Vec<i32> = match b_ptr.iter().map(|&x| i32::try_from(x).ok()).collect::<Option<_>>() { Some(v) => v, None => continue };
                let bri: Vec<i32> = match b_idx.iter().map(|&x| i32::try_from(x).ok()).collect::<Option<_>>() { Some(v) => v, None => continue };
                let Some(bcore) = feral_ordering_core::CscPattern::new(cn, &bcp, &bri) else { continue; };
                let o = feral_amf::AmfOptions { dense_alpha: 10.0, ..Default::default() };
                let Ok((pb, ..)) = feral_amf::amf_order_opts(&bcore, &o) else { continue; };
                let cp: Vec<usize> = pb.into_iter().map(|x| q[x as usize]).collect();
                if !super::is_bijection(&cp, cn) { continue; }
                let f = super::flops_of(&core_pat, &cp);
                let total = il.prefix_flops.saturating_add(f);
                if best_here.as_ref().map_or(true, |(bf, _)| total < *bf) {
                    best_here = Some((total, splice(&il, &cp)));
                }
            }
        }
        best_here
    };
    // One scoped thread per admitted set (at most three); results are merged
    // by set index, so thread timing never reaches the output.
    let results: Vec<Option<(u64, Vec<usize>)>> = std::thread::scope(|sc| {
        let handles: Vec<_> = admitted
            .iter()
            .map(|in_x| {
                let eval_set = &eval_set;
                sc.spawn(move || {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| eval_set(in_x))).ok().flatten()
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().ok().flatten()).collect()
    });
    let mut best: Option<(u64, Vec<usize>)> = None;
    for r in results.into_iter().flatten() {
        if best.as_ref().map_or(true, |(bf, _)| r.0 < *bf) {
            best = Some(r);
        }
    }
    best
}

/// prefix ++ core, mapped back to original ids.
pub(crate) fn splice(il: &IndepLift, core_perm: &[usize]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::with_capacity(il.prefix.len() + core_perm.len());
    out.extend_from_slice(&il.prefix);
    for &k in core_perm {
        out.push(il.core_ids[k]);
    }
    out
}
