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

/// Greedy maximal independent set by ascending predicted NEW FILL — the
/// number of non-adjacent pairs in `N(v)` (exact for degree ≤ `exact_deg`,
/// the all-pairs bound above it) — then `(degree, index)`. Eliminating a
/// vertex whose neighbours are already mutually adjacent adds no edge to the
/// Schur complement, so this prefers members that leave the core sparse
/// rather than members that are merely cheap to eliminate.
#[allow(dead_code)]
pub(crate) fn fill_greedy_independent_set(sp: &ScoringPattern, max_deg: usize, exact_deg: usize) -> Vec<bool> {
    let n = sp.n;
    if n > u32::MAX as usize {
        return vec![false; n];
    }
    let mut edges: EdgeSet = EdgeSet::with_capacity_and_hasher(sp.row_idx.len() / 2 + 16, Default::default());
    for v in 0..n {
        for &w in &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]] {
            if w > v {
                edges.insert(key(v as u32, w as u32));
            }
        }
    }
    let fill_of = |v: usize| -> u64 {
        let nb = &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]];
        let d = nb.len() as u64;
        if nb.len() > exact_deg {
            return d * d.saturating_sub(1) / 2;
        }
        let mut miss = 0u64;
        for i in 0..nb.len() {
            for j in (i + 1)..nb.len() {
                if !edges.contains(&key(nb[i] as u32, nb[j] as u32)) {
                    miss += 1;
                }
            }
        }
        miss
    };
    let mut keyed: Vec<(u64, usize, usize)> = (0..n)
        .filter(|&v| sp.col_ptr[v + 1] - sp.col_ptr[v] <= max_deg)
        .map(|v| (fill_of(v), sp.col_ptr[v + 1] - sp.col_ptr[v], v))
        .collect();
    keyed.sort_unstable();
    let mut in_x = vec![false; n];
    let mut blocked = vec![false; n];
    for (_, _, v) in keyed {
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

/// Greedy maximal independent set by ascending `(degree, index)` among the
/// vertices NOT in `excluded` and not adjacent to any excluded vertex — the
/// "second colour class" once `excluded` has been taken.
pub(crate) fn greedy_independent_set_excluding(sp: &ScoringPattern, max_deg: usize, excluded: &[bool]) -> Vec<bool> {
    let n = sp.n;
    let mut order: Vec<usize> = (0..n).filter(|&v| !excluded[v]).collect();
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

/// Core-size envelope for the quotient-graph metric passes (see `run`):
/// their cost grows faster than AMD's on grid-like cores (0.6 s per pass on
/// the 80k-node cont6-qq core versus 10 ms on the 10k-node lee4 cores).
const METRIC_CORE_MAX_N: usize = 20_000; // iter265a RC
const METRIC_CORE_MAX_NNZ: usize = 350_000; // iter265a
/// Expensive passes (AMF, METIS, metrics) run only on cores whose AMD total
/// is within this factor of the best AMD total over all sets, `(num, den)`.
const COMPETITIVE_MARGIN: (u64, u64) = (3, 2);
/// METIS runs on this many cores per pattern (the lowest AMD totals).
const METIS_TOP_CORES: usize = 1;
/// The metric walks run on this many cores per pattern (the lowest AMD totals).
const METRIC_TOP_CORES: usize = 3; // iter265a RC

/// Ordering passes on a lifted core. `Amd` runs first on every core; the rest
/// run only on competitive cores (see `run`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pass {
    Amd,
    Amf,
    Metis,
    Metric(super::custom_metrics::ScoreVariant),
}

fn run_pass(ccore: &feral_ordering_core::CscPattern<'_>, pass: Pass) -> Option<Vec<i32>> {
    match pass {
        Pass::Amd => feral_amd::amd_order(ccore).ok(),
        Pass::Amf => {
            let o = feral_amf::AmfOptions { dense_alpha: 10.0, ..Default::default() };
            feral_amf::amf_order_opts(ccore, &o).ok().map(|(p, ..)| p)
        }
        Pass::Metis => feral_metis::metis_order_full(ccore, &feral_metis::MetisOptions::default()).ok().map(|(p, ..)| p),
        Pass::Metric(v) => super::custom_metrics::order_variant(ccore, 10.0, true, v).ok(),
    }
}

/// Deterministic parallel map: `f(i)` for every `i < len` on up to
/// `PAR_MAX_THREADS` scoped threads pulling indices from a shared counter;
/// results are stored by index, so thread timing never reaches the output.
fn par_map<T: Send>(len: usize, f: impl Fn(usize) -> T + Sync) -> Vec<Option<T>> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let mut out: Vec<Option<T>> = (0..len).map(|_| None).collect();
    if len == 0 {
        return out;
    }
    let nthreads = super::parallel::PAR_MAX_THREADS.min(len);
    let next = AtomicUsize::new(0);
    let slots: Vec<std::sync::Mutex<Option<T>>> = (0..len).map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|sc| {
        for _ in 0..nthreads {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= len {
                    break;
                }
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(i))).ok();
                if let Ok(mut g) = slots[i].lock() {
                    *g = r;
                }
            });
        }
    });
    for (i, slot) in slots.into_iter().enumerate() {
        out[i] = slot.into_inner().ok().flatten();
    }
    out
}

/// A lifted core with its i32 CSC copy for the feral orderers.
struct LiftedCore {
    il: IndepLift,
    core_pat: ScoringPattern,
    ccp: Vec<i32>,
    cri: Vec<i32>,
}

/// Work-ledgered production driver.
///
/// Sets: the greedy maximal independent set by `(degree, index)` at caps
/// unbounded / 9 / 3 (the unbounded set is the whole-side / normal-equations
/// route, the capped ones keep mid-degree vertices in the core) and the
/// SECOND COLOUR CLASS — the greedy set among the vertices the unbounded set
/// did not take, at caps unbounded / 9. On KKT patterns that is the other side
/// of the bipartition, generalised to non-bipartite graphs; it is the set that
/// exposes the crudeoil_lee family (lee4_06 0.660 -> 0.542).
///
/// Passes: AMD on every core first (one AMD-speed walk each, in parallel).
/// Then, only on cores whose AMD total is within `COMPETITIVE_MARGIN` of the
/// best — dense whole-side cores lose by 2-100x and never recover under any
/// orderer — AMF (sparse non-giant cores), METIS (`METIS_CORE_*` envelope) and
/// two quotient-graph metrics (`METRIC_CORE_*` envelope: DegDivNvSqrtWf,
/// DegPlusDegme — the ones that win on lifted cores; they beat
/// minimum degree on the lee cores by 10-15 %). All passes are one flat task
/// list on `PAR_MAX_THREADS` threads, ranked exactly on the core; the cheapest
/// spliced ordering is returned with its exact total. Admission is charged to
/// `ledger` in edge-touch units, so the added time is bounded by structure
/// alone: a set whose predicted pairs or core would exceed the allowance is
/// skipped, never started.
pub(crate) fn run(sp: &ScoringPattern, ledger: u64) -> Option<(u64, Vec<usize>)> {
    use super::custom_metrics::ScoreVariant as V;
    let n = sp.n;
    let nnz = sp.row_idx.len();
    if n < 32 || nnz == 0 {
        return None;
    }
    // iter235a: hydro-class — 180a sequential AMF α5+relabel (tip misses 0.8529 indep)
    if (1800..=2500).contains(&n) {
        return run_sequential_180(sp, ledger);
    }
    // Admission is decided up front from the pattern alone. A set is trimmed
    // (hubs back into the core) until its predicted lift + core work fits:
    // lift ~ nnz + pairs, core ≤ nnz + 2·pairs, walked by the ordering passes
    // and exact scorings, i.e. 5·nnz + 9·pairs ≤ ledger. Two sets with
    // identical size and pair sum are (in practice) the same set; the Schur
    // complement is not paid for twice.
    let share = ledger;
    let max_pairs = share.saturating_sub(5 * nnz as u64) / 9;
    if max_pairs == 0 {
        return None;
    }
    let g_inf = greedy_independent_set(sp, usize::MAX);
    // Degree-greedy caps. Extra mid-caps (15/5, plus 20/7 on dense inputs)
    // are the promoted 180a family; they only add AMD walks in phase 1 when
    // the set is new, and they are skipped when (xs, pairs) matches a
    // cheaper cap. Second-colour-class sets are gated to n<=12k so the
    // lee4_09/10 critical path (already 1.04 s on the tip) does not pay
    // two extra lifts + metric walks — that is what killed hidden timing
    // on fe871f1. lee4_06 (n=10429) and lee2_06 (n=6418) still get them.
    // Extra mid-caps are cheap AMD walks on medium patterns and are what
    // moved gams05 / gabriel09 on the 180a tip. On giant-dense KKTs
    // (pooling_sppc3pq) they add ~0.12 s of lift+AMD and crowd METIS off
    // the winning core — skip them there.
    let mut candidates: Vec<Vec<bool>> = vec![g_inf.clone()];
    if n <= 20_000 || nnz <= 400_000 {
        let dense_input = nnz >= 12 * n;
        let extra_caps: &[usize] = if dense_input {
            &[20, 15, 9, 7, 5, 3]
        } else {
            &[15, 9, 5, 3]
        };
        for &cap in extra_caps {
            candidates.push(greedy_independent_set(sp, cap));
        }
    } else {
        for &cap in &[9usize, 3] {
            candidates.push(greedy_independent_set(sp, cap));
        }
    }
    // iter265a RC x-sets; iter313a: x-15/x-5 only n<=10k (avoid re-roll lee4_06 4b)
    if n <= 18_000 && nnz <= 80_000 {
        candidates.push(greedy_independent_set_excluding(sp, usize::MAX, &g_inf));
        candidates.push(greedy_independent_set_excluding(sp, 9, &g_inf));
        if n <= 10_000 {
            candidates.push(greedy_independent_set_excluding(sp, 15, &g_inf));
            candidates.push(greedy_independent_set_excluding(sp, 5, &g_inf));
        }
    }
    let mut admitted: Vec<Vec<bool>> = Vec::new();
    let mut seen_sizes: Vec<(usize, u64)> = Vec::new();
    for mut in_x in candidates {
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

    // Phase 1: lift + AMD on every admitted set.
    let phase1 = par_map(admitted.len(), |i| -> Option<(LiftedCore, u64, Vec<usize>)> {
        let il = lift(sp, &admitted[i], max_core_edges)?;
        let cn = il.core_n();
        if cn < 2 || 4 * il.core_nnz() as u64 > share {
            return None;
        }
        let core_pat = ScoringPattern { n: cn, col_ptr: il.core_col_ptr.clone(), row_idx: il.core_row_idx.clone() };
        let ccp: Vec<i32> = il.core_col_ptr.iter().map(|&x| i32::try_from(x).ok()).collect::<Option<_>>()?;
        let cri: Vec<i32> = il.core_row_idx.iter().map(|&x| i32::try_from(x).ok()).collect::<Option<_>>()?;
        let ccore = feral_ordering_core::CscPattern::new(cn, &ccp, &cri)?;
        let p = run_pass(&ccore, Pass::Amd)?;
        let cp: Vec<usize> = p.into_iter().map(|x| x as usize).collect();
        if !super::is_bijection(&cp, cn) {
            return None;
        }
        let f = il.prefix_flops.saturating_add(super::flops_of(&core_pat, &cp));
        Some((LiftedCore { il, core_pat, ccp, cri }, f, cp))
    });
    let cores: Vec<(LiftedCore, u64, Vec<usize>)> = phase1.into_iter().flatten().flatten().collect();
    if cores.is_empty() {
        return None;
    }
    let best_amd = cores.iter().map(|c| c.1).min()?;
    // METIS is the expensive pass (45-140 ms per core against ~10 ms for a
    // metric walk); it runs on the two cores with the lowest AMD totals only.
    // Every dev METIS win sits on one of those two (arki0013 cap-9, the
    // pooling cap-inf / cap-9 pair); the third-best core never won.
    let mut by_amd: Vec<usize> = (0..cores.len()).collect();
    by_amd.sort_by_key(|&i| (cores[i].1, i));
    // Second METIS only on nnz-heavy patterns (pooling): lee4_09/10 stay at
    // one METIS so their 1.03 s critical path does not grow. Hidden fe871f1
    // died in that band.
    let metis_k = if nnz >= 300_000 { 2 } else { METIS_TOP_CORES };
    let metis_ok: Vec<bool> = (0..cores.len()).map(|i| by_amd.iter().take(metis_k).any(|&j| j == i)).collect();
    // iter273a: METRIC top-4 only on n<=16k (lee4_09 class); else top-3 — protect lee4_10 timing
    let metric_k = if n <= 16_000 { 4 } else { METRIC_TOP_CORES };
    let metric_ok: Vec<bool> = (0..cores.len()).map(|i| by_amd.iter().take(metric_k).any(|&j| j == i)).collect();

    // Phase 2: the expensive passes on competitive cores, one flat task list.
    let mut tasks: Vec<(usize, Pass)> = Vec::new();
    for (i, (lc, amd_total, _)) in cores.iter().enumerate() {
        if amd_total.saturating_mul(COMPETITIVE_MARGIN.1) > best_amd.saturating_mul(COMPETITIVE_MARGIN.0) {
            continue;
        }
        let cn = lc.il.core_n();
        let cnnz = lc.il.core_nnz();
        let dense = cnnz >= 20 * cn;
        if cnnz <= GIANT_CORE_NNZ && !dense {
            tasks.push((i, Pass::Amf));
        }
        if metis_ok[i] && cn <= METIS_CORE_MAX_N && cnnz <= METIS_CORE_MAX_NNZ {
            tasks.push((i, Pass::Metis));
        }
        if metric_ok[i] && cn <= METRIC_CORE_MAX_N && cnnz <= METRIC_CORE_MAX_NNZ {
            // iter265a RC: broader quotient-metric family (0145 census winners)
            for v in [V::DegDivNvSqrtWf, V::DegPlusDegme, V::DegSqrt, V::SqDiv, V::DegDivNvDegme, V::DegP075] {
                tasks.push((i, Pass::Metric(v)));
            }
        }
    }
    let phase2 = par_map(tasks.len(), |t| -> Option<(usize, u64, Vec<usize>)> {
        let (i, pass) = tasks[t];
        let lc = &cores[i].0;
        let cn = lc.il.core_n();
        let ccore = feral_ordering_core::CscPattern::new(cn, &lc.ccp, &lc.cri)?;
        let p = run_pass(&ccore, pass)?;
        let cp: Vec<usize> = p.into_iter().map(|x| x as usize).collect();
        if !super::is_bijection(&cp, cn) {
            return None;
        }
        let f = lc.il.prefix_flops.saturating_add(super::flops_of(&lc.core_pat, &cp));
        Some((i, f, cp))
    });

    // Rank: AMD results first (set order), then phase-2 results (task order).
    let mut best: Option<(u64, usize, Vec<usize>)> = None;
    for (i, (_, f, cp)) in cores.iter().enumerate() {
        if best.as_ref().map_or(true, |(bf, _, _)| *f < *bf) {
            best = Some((*f, i, cp.clone()));
        }
    }
    for (i, f, cp) in phase2.into_iter().flatten().flatten() {
        if best.as_ref().map_or(true, |(bf, _, _)| f < *bf) {
            best = Some((f, i, cp));
        }
    }
    let (f, i, cp) = best?;
    Some((f, splice(&cores[i].0.il, &cp)))
}

fn run_sequential_180(sp: &ScoringPattern, ledger: u64) -> Option<(u64, Vec<usize>)> {
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
    // iter161a: base caps + 20/7 only on dense full patterns (no Scotch)
    let dense_input = nnz >= 12 * n;
    let caps: &[usize] = if dense_input {
        &[usize::MAX, 20, 15, 9, 7, 5, 3]
    } else {
        &[usize::MAX, 15, 9, 5, 3]
    };
    for &cap in caps {
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
                2 => feral_metis::metis_order_full(&ccore, &feral_metis::MetisOptions::default()).ok()?.0,
                _ => {
                    // iter159a: AMF α5 mid-core pass
                    let o = feral_amf::AmfOptions { dense_alpha: 5.0, ..Default::default() };
                    feral_amf::amf_order_opts(&ccore, &o).ok()?.0
                }
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
            if cn <= 4_000 && cnnz <= 60_000 {
                pass_ids.push(3); // AMF α5 tight iter176a
            }
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
