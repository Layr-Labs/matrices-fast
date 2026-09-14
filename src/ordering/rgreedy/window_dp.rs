//! Exact window search factored by connected components of the live graph.

use super::window_signatures::{ChargeModel, SignatureEngine};
use super::{Game, TripleWork};

const MAX_WIDTH: usize = 14;
/// Stop the sweep loop once a sweep accepts **no** change, for rows at or above
/// [`PRODUCTION_XCH_PLATEAU_MIN_N`]. Measured (`0260-plateau-*`): on `n >= 10 000`
/// this is wall-free value-wise — 45 rows move, 44 keep their ratio exactly and
/// one (`glider400`) pays +1e-4 — while it returns 0.09-0.20 s per large row and
/// 0.095 s on the binding row `crudeoil_lee4_10`, whose ratio is untouched. Below
/// the gate the schedule is worth real value (27 rows lose up to +0.0108 if it is
/// applied to the whole corpus, score +1.9e-4), so the gate is structural: `n`,
/// never matrix identity.
const PRODUCTION_XCH_PLATEAU: usize = 1;
const PRODUCTION_XCH_PLATEAU_MIN_N: usize = 10_000;

/// Fraction of a large exchange budget withheld until the search proves it
/// pays **this call**. Measured and REJECTED at 50 % (`0260-reserve-diff.txt`):
/// the score is bit-identical to the unreserved 4 GiB arm (0 of 300 ratios
/// differ) but the corpus wall is +3.1 s *worse*, because every waster row
/// accepts *internal* window improvements (which never reach the score) and so
/// releases the reserve and then spends it — the released budget is spent, not
/// saved. The signal "this call's spend pays" is not observable inside the
/// call; only the caller knows whether the candidate was adopted. Kept at 0 and
/// wired with `SSI_XCH_RESERVE` for the record.
const PRODUCTION_XCH_RESERVE_PCT: i64 = 0;
/// Only budgets at or above this are worth reserving against; the smaller sites
/// (64 MB, 16 MB) are untouched.
const XCH_RESERVE_MIN_BUDGET: i64 = 1 << 30;

#[inline]
fn xch_reserve_pct() -> i64 {
    #[cfg(test)]
    {
        std::env::var("SSI_XCH_RESERVE")
            .ok()
            .and_then(|v| v.trim().parse::<i64>().ok())
            .unwrap_or(PRODUCTION_XCH_RESERVE_PCT)
            .clamp(0, 90)
    }
    #[cfg(not(test))]
    {
        PRODUCTION_XCH_RESERVE_PCT
    }
}

#[inline]
fn xch_plateau() -> usize {
    #[cfg(test)]
    {
        std::env::var("SSI_XCH_PLATEAU")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(PRODUCTION_XCH_PLATEAU)
    }
    #[cfg(not(test))]
    {
        PRODUCTION_XCH_PLATEAU
    }
}

#[inline]
fn xch_plateau_min_n() -> usize {
    #[cfg(test)]
    {
        std::env::var("SSI_XCH_PLATEAU_N")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(PRODUCTION_XCH_PLATEAU_MIN_N)
    }
    #[cfg(not(test))]
    {
        PRODUCTION_XCH_PLATEAU_MIN_N
    }
}

/// Component *admission* policy for the exchange's per-window exact DP.
///
/// The shipped policy is a **hard stop**: `solve_component` precharges
/// `2^k * (16k + 6w + 24)` before it allocates anything, and the first
/// component the ledger cannot fund ends `refine_window`'s walk — with
/// `Some(true)`/`None`, the whole call when the window had not changed yet.
/// The ledger therefore funds the window's components in *position* order and
/// abandons whatever it could not reach, even when the refused component is the
/// expensive one and the remaining components are cheap.
///
/// Policy 1 skips the unfunded component and keeps walking the window's other
/// components; policy 2 additionally walks them **smallest-first**, so whatever
/// the ledger can still fund is spent on the cheap components — the ones whose
/// reorderings the call actually adopts. Both policies keep the precharge
/// itself, so the ledger still bounds this window's spend; only the *order* in
/// which components are offered to it changes. Determinism is preserved: the
/// walk is a stable sort of a deterministic enumeration (ties broken by the
/// component's bit mask).
/// ── iter68d: SHIPPED AS 1, "skip the unfunded component" ────────────────────
/// Policy 0 ends the window walk at the first component the precharged ledger
/// cannot fund, abandoning the cheap components behind it; policy 1 skips that
/// component and keeps walking in position order. The precharge itself is kept,
/// so the ledger still bounds the window's spend and policy 1 can only spend
/// what policy 0 already budgeted — it changes *which* components get the
/// budget, never how much there is. Measured this session on the crown tree
/// (one binary, one session, all 300 dev rows, graded-closest
/// `SSI_MARK_NOSCORE=1` frame): **score 0.790236 -> 0.790230, 299 of 300 rows
/// bit-identical, one mover** (`crudeoil_lee1_07` 0.745866999 -> 0.744049962,
/// dln -2.44e-3) — strictly monotone, no regression anywhere. Worker frame
/// (one process per row, min of 3, 13 rows including the mover): no systematic
/// wall change; the rows that move are inside the same budget. [0278d]
const PRODUCTION_XCH_ALLOC: usize = 1;

#[inline]
fn xch_alloc() -> usize {
    #[cfg(test)]
    {
        std::env::var("SSI_XCH_ALLOC")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(PRODUCTION_XCH_ALLOC)
            .min(2)
    }
    #[cfg(not(test))]
    {
        PRODUCTION_XCH_ALLOC
    }
}
/// The dimension ceiling this DP obeys. It *is* `rgreedy::MAX_N` in
/// production; in test builds it follows the same `SSI_MAX_N` seam, so one
/// binary can price a ceiling curve (the DP refuses any `n` above it, which is
/// exactly how the 0220 class-gate arm was measured inert).
#[inline(always)]
fn max_dimension() -> usize {
    super::max_n_limit()
}

#[cfg(test)]
#[derive(Clone, Default)]
struct WorkStats {
    calls: [usize; 17],
    completed: [usize; 17],
    components: [usize; 17],
    refused_components: usize,
}

#[cfg(test)]
thread_local! {
    static WORK_STATS: std::cell::RefCell<WorkStats> = std::cell::RefCell::new(WorkStats::default());
}

#[cfg(test)]
#[derive(Default)]
struct WorkReport {
    width: usize,
    completed: bool,
    // iter60 phase attribution (env `SSI_XCH_TIME`): the exchange's own wall
    // split. `build` = Game::build_adj + Game::new, `reset` = every
    // `Game::reset()` (full bitset memcpy + bucket rebuild), `prefix` = the
    // per-sweep prefix eliminations, `refine` = the window refines + their
    // interleaved eliminations.
    n: usize,
    adj_ns: u128,
    new_ns: u128,
    reset_ns: u128,
    prefix_ns: u128,
    refine_ns: u128,
    resets: u32,
    prefixes: u32,
    // Call key: `build_adj` is a pure function of the CSR, so identical
    // (n, col_ptr ptr, row_idx ptr, nnz) inside one row means identical bytes.
    cptr: usize,
    rptr: usize,
    nnz: usize,
}

#[cfg(test)]
impl Drop for WorkReport {
    fn drop(&mut self) {
        WORK_STATS.with(|cell| {
            let mut stats = cell.borrow_mut();
            stats.calls[self.width] += 1;
            stats.completed[self.width] += usize::from(self.completed);
        });
        if std::env::var_os("SSI_XCH_TIME").is_some() {
            let total =
                self.adj_ns + self.new_ns + self.reset_ns + self.prefix_ns + self.refine_ns;
            eprintln!(
                "XCHTIME\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                self.n,
                self.resets,
                self.prefixes,
                total,
                self.adj_ns,
                self.new_ns,
                self.reset_ns,
                self.prefix_ns,
                self.refine_ns,
                self.cptr,
                self.rptr,
                self.nnz,
            );
        }
    }
}

/// ── iter68 TEST-ONLY instrument: the intra-`refine` split.
///
/// `WorkReport::refine_ns` lumps two very different costs together: the window
/// DP itself (`solve_component` / `SignatureEngine::solve_component`) and the
/// interleaved `Game::eliminate` replay that advances the sweep between
/// windows. This module separates them, plus the component-width histogram and
/// the charge-refusal counts, so a device can be aimed at the half that
/// actually carries the wall. Compiled out of the graded worker (`cfg(test)`).
#[cfg(test)]
pub(crate) mod split {
    use std::sync::atomic::{AtomicU64, Ordering};

    pub(crate) static WIN_NS: AtomicU64 = AtomicU64::new(0);
    pub(crate) static DP_NS: AtomicU64 = AtomicU64::new(0);
    pub(crate) static ELIM_NS: AtomicU64 = AtomicU64::new(0);
    pub(crate) static WINDOWS: AtomicU64 = AtomicU64::new(0);
    pub(crate) static COMPONENTS: AtomicU64 = AtomicU64::new(0);
    pub(crate) static REFUSED: AtomicU64 = AtomicU64::new(0);
    pub(crate) static REFUSED_BIG: AtomicU64 = AtomicU64::new(0);
    pub(crate) static ENGINE: AtomicU64 = AtomicU64::new(0);
    pub(crate) static BRUTE: AtomicU64 = AtomicU64::new(0);
    pub(crate) static K2_4: AtomicU64 = AtomicU64::new(0);
    pub(crate) static K5_6: AtomicU64 = AtomicU64::new(0);
    pub(crate) static K7_8: AtomicU64 = AtomicU64::new(0);
    pub(crate) static K9_10: AtomicU64 = AtomicU64::new(0);
    pub(crate) static K11_12: AtomicU64 = AtomicU64::new(0);
    pub(crate) static K13_14: AtomicU64 = AtomicU64::new(0);

    #[inline]
    pub(crate) fn add(c: &AtomicU64, ns: u128) {
        c.fetch_add(ns as u64, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn bump(c: &AtomicU64) {
        c.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn width_bump(k: usize) {
        match k {
            2..=4 => bump(&K2_4),
            5..=6 => bump(&K5_6),
            7..=8 => bump(&K7_8),
            9..=10 => bump(&K9_10),
            11..=12 => bump(&K11_12),
            _ => bump(&K13_14),
        }
    }

    /// seconds: (win, dp, elim); counts: (windows, components, refused,
    /// refused_big, engine, brute); then the six width buckets.
    #[allow(clippy::type_complexity)]
    pub(crate) fn take() -> ((f64, f64, f64), [u64; 6], [u64; 6]) {
        let s = |c: &AtomicU64| c.swap(0, Ordering::Relaxed) as f64 / 1e9;
        let n = |c: &AtomicU64| c.swap(0, Ordering::Relaxed);
        let times = (s(&WIN_NS), s(&DP_NS), s(&ELIM_NS));
        let counts = [
            n(&WINDOWS),
            n(&COMPONENTS),
            n(&REFUSED),
            n(&REFUSED_BIG),
            n(&ENGINE),
            n(&BRUTE),
        ];
        let hist = [
            n(&K2_4),
            n(&K5_6),
            n(&K7_8),
            n(&K9_10),
            n(&K11_12),
            n(&K13_14),
        ];
        (times, counts, hist)
    }
}

fn solve_component(
    game: &Game<'_>,
    vertices: &[usize],
    work: &mut TripleWork,
) -> Option<(Vec<usize>, u64, u64)> {
    let k = vertices.len();
    let states = 1usize << k;
    let cost = states.saturating_mul(
        16usize
            .saturating_mul(k)
            .saturating_add(6usize.saturating_mul(game.w))
            .saturating_add(24),
    );
    if !work.charge(cost) {
        #[cfg(test)]
        WORK_STATS.with(|cell| cell.borrow_mut().refused_components += 1);
        return None;
    }
    #[cfg(test)]
    WORK_STATS.with(|cell| cell.borrow_mut().components[k] += 1);
    let mut inside = vec![0u16; k];
    for (i, &v) in vertices.iter().enumerate() {
        for (j, &u) in vertices.iter().enumerate() {
            if game.adj[v * game.w + u / 64] & (1u64 << (u % 64)) != 0 {
                inside[i] |= 1 << j;
            }
        }
    }
    let mut components = vec![0u16; states * k];
    let mut unions = vec![0u64; states * game.w];
    let mut widths = vec![0u64; states];
    for mask in 1..states {
        let v = mask.trailing_zeros() as usize;
        let bit = 1usize << v;
        let rest = mask ^ bit;
        let mut merged = bit as u16;
        let mut neighbors = inside[v] & rest as u16;
        while neighbors != 0 {
            let u = neighbors.trailing_zeros() as usize;
            neighbors &= neighbors - 1;
            merged |= components[rest * k + u];
        }
        for u in 0..k {
            components[mask * k + u] = if merged & (1 << u) != 0 {
                merged
            } else {
                components[rest * k + u]
            };
        }
        let vertex = vertices[v];
        for word in 0..game.w {
            unions[mask * game.w + word] =
                unions[rest * game.w + word] | game.adj[vertex * game.w + word];
        }
        unions[mask * game.w + vertex / 64] |= 1u64 << (vertex % 64);
        if merged as usize == mask {
            let boundary = unions[mask * game.w..(mask + 1) * game.w]
                .iter()
                .map(|word| word.count_ones() as u64)
                .sum::<u64>();
            widths[mask] = boundary - mask.count_ones() as u64 + 1;
        }
    }

    // A pivot sees the boundary of its eliminated-prefix component. Other
    // eliminated components cannot touch it, so no fill graph replay is needed.
    let mut best = vec![u64::MAX; states];
    let mut path = vec![u64::MAX; states];
    best[0] = 0;
    path[0] = 0;
    for mask in 0..states - 1 {
        for pivot in 0..k {
            let bit = 1usize << pivot;
            if mask & bit != 0 {
                continue;
            }
            let next = mask | bit;
            let width = widths[components[next * k + pivot] as usize];
            let cost = best[mask] + width * width;
            let code = (path[mask] << 4) | pivot as u64;
            if cost < best[next] || (cost == best[next] && code < path[next]) {
                best[next] = cost;
                path[next] = code;
            }
        }
    }
    let incumbent = (0..k)
        .map(|pivot| {
            let mask = (1usize << (pivot + 1)) - 1;
            let width = widths[components[mask * k + pivot] as usize];
            width * width
        })
        .sum();
    let order = if best[states - 1] < incumbent {
        (0..k)
            .map(|i| vertices[((path[states - 1] >> (4 * (k - i - 1))) & 15) as usize])
            .collect()
    } else {
        vertices.to_vec()
    };
    Some((order, best[states - 1], incumbent))
}

fn refine_window(
    game: &Game<'_>,
    window: &mut [usize],
    work: &mut TripleWork,
    engine: &mut Option<SignatureEngine>,
    charge_model: ChargeModel,
) -> Option<bool> {
    let k = window.len();
    if !work.charge(8 * k * k + 8 * k) {
        return None;
    }
    // Component admission (see `xch_alloc`). Enumerate the window's live
    // components in ascending lowest-position order — the shipped order — and
    // collect only the ones this DP may attempt (2..=MAX_WIDTH positions).
    let alloc = xch_alloc();
    let mut unseen = if k == 64 { u64::MAX } else { (1u64 << k) - 1 };
    let mut comps: Vec<(u32, u64)> = Vec::new();
    while unseen != 0 {
        let mut component = 1u64 << unseen.trailing_zeros();
        let mut frontier = component;
        while frontier != 0 {
            let i = frontier.trailing_zeros() as usize;
            frontier &= frontier - 1;
            let v = window[i];
            for (j, &u) in window.iter().enumerate() {
                let bit = 1u64 << j;
                if unseen & bit != 0
                    && component & bit == 0
                    && game.adj[v * game.w + u / 64] & (1u64 << (u % 64)) != 0
                {
                    component |= bit;
                    frontier |= bit;
                }
            }
        }
        unseen &= !component;
        // Large spans are useful when their live induced graph separates into
        // small components. Leave oversized components in their original
        // positions; their elimination cannot affect another component here.
        let size = component.count_ones();
        if size < 2 || size as usize > MAX_WIDTH {
            continue;
        }
        comps.push((size, component));
    }
    if alloc >= 2 {
        // Smallest first: the ledger is a fixed precharge budget, so the cheap
        // components are the ones it can still fund once a big one appears.
        comps.sort_unstable();
    }
    let mut changed = false;
    for &(_, component) in comps.iter() {
        let positions: Vec<usize> = (0..k).filter(|&i| component & (1 << i) != 0).collect();
        let vertices: Vec<usize> = positions.iter().map(|&i| window[i]).collect();
        let incident = vertices.iter().map(|&v| game.deg[v] as usize).sum();
        let (union_cost, signature_cost) =
            SignatureEngine::charge_costs(vertices.len(), game.w, incident);
        #[cfg(test)]
        let t_dp = std::time::Instant::now();
        // iter68 KILL (0268-xchsplit-engine-small): dropping the `k >= 5` floor so
        // the calibrated model could route k<=4 components to the engine is
        // wall-neutral (dp 0.285 -> 0.260 s on procurement1large, <=0.01 s
        // elsewhere) even though it moved 30 340 of 37 009 component calls off
        // the dense-union path, so the `2^k * w` scratch is NOT the exchange's
        // DP wall. Kept at the shipped floor.
        #[cfg(test)]
        let engine_floor: usize = std::env::var("SSI_ENGINE_FLOOR")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(5);
        #[cfg(not(test))]
        let engine_floor: usize = 5;
        let use_engine = vertices.len() >= engine_floor && signature_cost < union_cost;
        let solution = if use_engine {
            let engine = engine.get_or_insert_with(|| SignatureEngine::new(game.n));
            engine.set_charge_model(charge_model);
            engine.solve_component(game, &vertices, work)
        } else {
            solve_component(game, &vertices, work)
        };
        #[cfg(test)]
        {
            split::add(&split::DP_NS, t_dp.elapsed().as_nanos());
            split::bump(&split::COMPONENTS);
            split::width_bump(vertices.len());
            split::bump(if use_engine { &split::ENGINE } else { &split::BRUTE });
            if solution.is_none() {
                split::bump(&split::REFUSED);
                if vertices.len() >= 9 {
                    split::bump(&split::REFUSED_BIG);
                }
            }
        }
        let Some((order, best, incumbent)) = solution else {
            if alloc >= 1 {
                continue;
            }
            return if changed { Some(true) } else { None };
        };
        if best < incumbent {
            for (&position, &v) in positions.iter().zip(&order) {
                window[position] = v;
            }
            changed = true;
        }
    }
    Some(changed)
}

/// Every window leaves the same suffix graph. Keep completed strict gains even
/// when the precharged work allowance cannot fund the rest of a sweep.
pub(crate) fn subset_window_descent(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    budget: i64,
) -> Option<Vec<usize>> {
    subset_window_descent_config(
        n,
        col_ptr,
        row_idx,
        seed,
        width,
        sweeps,
        (width / 2).max(1),
        budget,
        ChargeModel::UnionParity,
        MAX_WIDTH,
    )
}

pub(crate) fn subset_window_descent_step(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    offset_step: usize,
    budget: i64,
) -> Option<Vec<usize>> {
    subset_window_descent_config(
        n,
        col_ptr,
        row_idx,
        seed,
        width,
        sweeps,
        offset_step,
        budget,
        ChargeModel::SignatureTrue,
        MAX_WIDTH,
    )
}

/// Wider contiguous spans, with the same 14-vertex exact component ceiling.
/// Components remain in their original slots, so their interleaving and the
/// residual graph after the span are unchanged. Oversized components are kept.
pub(crate) fn sparse_span_window_descent(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    offset_step: usize,
    budget: i64,
) -> Option<Vec<usize>> {
    subset_window_descent_config(n, col_ptr, row_idx, seed, width, sweeps,
        offset_step, budget, ChargeModel::SignatureTrue, 64)
}

fn subset_window_descent_config(
    n: usize,
    col_ptr: &[usize],
    row_idx: &[usize],
    seed: &[usize],
    width: usize,
    sweeps: usize,
    offset_step: usize,
    budget: i64,
    charge_model: ChargeModel,
    max_span: usize,
) -> Option<Vec<usize>> {
    if n < 2
        || n > max_dimension()
        || !(2..=max_span).contains(&width)
        || offset_step >= width
        || sweeps == 0
        || budget <= 0
        || seed.len() != n
        || col_ptr.len() != n + 1
    {
        return None;
    }
    #[cfg(test)]
    let mut report = WorkReport {
        width: width.min(16),
        n,
        ..WorkReport::default()
    };
    let mut work = TripleWork { remaining: budget };
    let reserve = if budget >= XCH_RESERVE_MIN_BUDGET {
        budget * xch_reserve_pct() / 100
    } else {
        0
    };
    let mut reserved = reserve;
    work.remaining -= reserve;
    #[cfg(test)]
    {
        report.cptr = col_ptr.as_ptr() as usize;
        report.rptr = row_idx.as_ptr() as usize;
        report.nnz = row_idx.len();
    }
    if !work.charge(n + 1 + row_idx.len() + 2 * n)
        || col_ptr.first().copied() != Some(0)
        || col_ptr.last().copied() != Some(row_idx.len())
        || col_ptr.windows(2).any(|p| p[0] > p[1])
        || row_idx.iter().any(|&v| v >= n)
    {
        return None;
    }
    let mut seen = vec![false; n];
    for &v in seed {
        if v >= n || seen[v] {
            return None;
        }
        seen[v] = true;
    }
    let words = n.div_ceil(64);
    let setup = 3 * n * words + 2 * row_idx.len() + 16 * n + words;
    if !work.charge(setup) {
        return None;
    }
    #[cfg(test)]
    let t_adj = std::time::Instant::now();
    let pristine = super::pristine_memo(n, col_ptr, row_idx)?;
    #[cfg(test)]
    {
        report.adj_ns += t_adj.elapsed().as_nanos();
    }
    #[cfg(test)]
    let t_new = std::time::Instant::now();
    let mut game = pristine.game_reset_first()?;
    #[cfg(test)]
    {
        report.new_ns += t_new.elapsed().as_nanos();
    }
    let mut engine = None;
    let mut current = seed.to_vec();
    let mut changed = false;
    let plateau = if n >= xch_plateau_min_n() { xch_plateau() } else { 0 };
    let mut idle_sweeps = 0usize;
    for sweep in 0..sweeps {
        if !work.charge(2 * n * words + 8 * n) {
            return changed.then_some(current);
        }
        #[cfg(test)]
        let t_reset = std::time::Instant::now();
        game.reset();
        #[cfg(test)]
        {
            report.reset_ns += t_reset.elapsed().as_nanos();
            report.resets += 1;
        }
        let offset = (sweep * offset_step) % width;
        for &v in current.iter().take(offset.min(n)) {
            #[cfg(test)]
            let t_prefix = std::time::Instant::now();
            if !work.eliminate(&mut game, v) {
                return changed.then_some(current);
            }
            #[cfg(test)]
            {
                report.prefix_ns += t_prefix.elapsed().as_nanos();
                report.prefixes += 1;
            }
        }
        let mut start = offset;
        let mut sweep_changed = false;
        while start + 1 < n {
            let end = (start + width).min(n);
            #[cfg(test)]
            let t_refine = std::time::Instant::now();
            let outcome = refine_window(
                &game,
                &mut current[start..end],
                &mut work,
                &mut engine,
                charge_model,
            );
            #[cfg(test)]
            {
                let e = t_refine.elapsed().as_nanos();
                report.refine_ns += e;
                split::add(&split::WIN_NS, e);
                split::bump(&split::WINDOWS);
            }
            match outcome {
                Some(improved) => {
                    changed |= improved;
                    sweep_changed |= improved;
                }
                None => return changed.then_some(current),
            }
            if end < n {
                for &v in &current[start..end] {
                    #[cfg(test)]
                    let t_elim = std::time::Instant::now();
                    if !work.eliminate(&mut game, v) {
                        return changed.then_some(current);
                    }
                    #[cfg(test)]
                    {
                        let e = t_elim.elapsed().as_nanos();
                        report.refine_ns += e;
                        split::add(&split::ELIM_NS, e);
                    }
                }
            }
            start = end;
        }
        if sweep_changed {
            idle_sweeps = 0;
            if reserved > 0 {
                // The row's own sweep just paid for the withheld ledger: release it.
                work.remaining += reserved;
                reserved = 0;
            }
        } else {
            idle_sweeps += 1;
        }
        if plateau > 0 && idle_sweeps >= plateau {
            break;
        }
    }
    #[cfg(test)]
    {
        report.completed = true;
    }
    changed.then_some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pattern;

    #[test]
    fn sparse_spans_preserve_large_components_and_handle_high_mask_bits() {
        let n = 65;
        let mut edges = vec![(0, 1), (0, 2), (0, 3), (60, 61), (60, 62),
            (60, 63), (1, 64), (61, 64)];
        for v in 4..22 { for u in v + 1..22 { edges.push((v, u)); } }
        let p = Pattern::from_edges(n, &edges);
        let pristine = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(n, &pristine).unwrap();
        let seed: Vec<_> = (0..n).collect();
        let before = game.replay_flops(&seed);
        let changed = sparse_span_window_descent(n, &p.col_ptr, &p.row_idx,
            &seed, 64, 1, 27, 64_000_000).unwrap();
        assert_eq!(&changed[4..22], &seed[4..22]);
        assert_eq!(changed[64], 64);
        assert!(game.replay_flops(&changed) < before);
        game.reset(); for &v in &seed[..64] { game.eliminate(v); }
        let suffix = game.adj.clone();
        game.reset(); for &v in &changed[..64] { game.eliminate(v); }
        assert_eq!(game.adj, suffix);
        assert!(sparse_span_window_descent(n, &p.col_ptr, &p.row_idx,
            &seed, 65, 1, 27, 64_000_000).is_none());
    }

    #[test]
    fn sparse_spans_match_original_descent_within_exact_width_limit() {
        for n in [7usize, 19, 67, 131] {
            let edges: Vec<_> = (0..n).flat_map(|v| (v+1..n)
                .filter(move |&u| (v * 31 + u * 17) % 23 < 3)
                .map(move |u| (v, u))).collect();
            let p = Pattern::from_edges(n, &edges);
            let seed: Vec<_> = (0..n).rev().collect();
            for (width, step) in [(7, 2), (8, 3), (12, 5), (14, 5)] {
                for budget in [100, 40_000, 16_000_000] {
                    assert_eq!(subset_window_descent_step(n, &p.col_ptr, &p.row_idx,
                        &seed, width, 4, step, budget),
                        sparse_span_window_descent(n, &p.col_ptr, &p.row_idx,
                            &seed, width, 4, step, budget));
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn probe_window_work() {
        for (name, pattern) in crate::corpus::corpus() {
            WORK_STATS.with(|cell| *cell.borrow_mut() = WorkStats::default());
            let _ = super::super::window_signatures::take_probe_stats();
            let permutation = crate::ordering::order(&pattern);
            let flops = ssi_scoring::score(&pattern, &permutation).flops;
            let stats = WORK_STATS.with(|cell| cell.borrow().clone());
            let memo = super::super::window_signatures::take_probe_stats();
            println!(
                "SIGNATURE_MEMO\t{name}\t{}\t{}\t{}\t{}",
                memo.probes, memo.hits, memo.inserted, memo.trivial,
            );
            println!(
                "WINDOW_WORK\t{name}\t{}\t{}\t{flops}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}",
                pattern.n,
                pattern.nnz(),
                stats.calls[8],
                stats.completed[8],
                stats.calls[10],
                stats.completed[10],
                stats.calls[12],
                stats.completed[12],
                stats.refused_components,
                stats.components,
            );
        }
    }

    #[test]
    #[ignore]
    fn probe_next_windows() {
        for (name, pattern) in crate::corpus::corpus() {
            if pattern.n < 6 || pattern.n > max_dimension() || pattern.nnz() > 200_000 {
                continue;
            }

            let incumbent = crate::ordering::order(&pattern);
            let before = ssi_scoring::score(&pattern, &incumbent).flops;
            for (width, sweeps, budget) in [
                (12, 4, 64_000_000),
                (14, 2, 48_000_000),
                (14, 4, 96_000_000),
            ] {
                WORK_STATS.with(|cell| *cell.borrow_mut() = WorkStats::default());
                let start = std::time::Instant::now();
                let candidate = subset_window_descent(
                    pattern.n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &incumbent,
                    width,
                    sweeps,
                    budget,
                );
                let seconds = start.elapsed().as_secs_f64();
                let after = candidate
                    .as_ref()
                    .map_or(before, |p| ssi_scoring::score(&pattern, p).flops);
                assert!(after <= before, "{name}: additional width {width}");
                let completed = WORK_STATS.with(|cell| cell.borrow().completed[width]);
                println!(
                    "NEXT_WINDOW\t{name}\t{}\t{}\t{before}\t{after}\t{seconds:.6}\t{width}\t{sweeps}\t{budget}\t{completed}",
                    pattern.n, pattern.nnz(),
                );
            }
        }
    }

    #[test]
    #[ignore]
    fn probe_window_offsets() {
        for (name, pattern) in crate::corpus::corpus() {
            if pattern.n < 6 || pattern.n > max_dimension() || pattern.nnz() > 200_000 {
                continue;
            }
            let incumbent = crate::ordering::order(&pattern);
            let before = ssi_scoring::score(&pattern, &incumbent).flops;
            for (width, step, budget) in [
                (12, 1, 64_000_000),
                (12, 5, 64_000_000),
                (14, 1, 96_000_000),
                (14, 5, 96_000_000),
            ] {
                let start = std::time::Instant::now();
                let candidate = subset_window_descent_step(
                    pattern.n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &incumbent,
                    width,
                    4,
                    step,
                    budget,
                );
                let seconds = start.elapsed().as_secs_f64();
                let after = candidate
                    .as_ref()
                    .map_or(before, |p| ssi_scoring::score(&pattern, p).flops);
                assert!(after <= before, "{name}: offset {width}/{step}");
                println!(
                    "OFFSET_WINDOW\t{name}\t{}\t{}\t{before}\t{after}\t{seconds:.6}\t{width}\t{step}",
                    pattern.n, pattern.nnz(),
                );
            }
        }
    }

    fn permutations(values: &mut [usize], i: usize, visit: &mut impl FnMut(&[usize])) {
        if i == values.len() {
            visit(values);
        } else {
            for j in i..values.len() {
                values.swap(i, j);
                permutations(values, i + 1, visit);
                values.swap(i, j);
            }
        }
    }

    fn verify_window(p: &Pattern, prefix: &[usize], window: &[usize]) {
        let pristine = Game::build_adj(p.n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(p.n, &pristine).unwrap();
        game.reset();
        for &v in prefix {
            game.eliminate(v);
        }
        let mut work = TripleWork {
            remaining: i64::MAX,
        };
        let (order, best, incumbent) = solve_component(&game, window, &mut work).unwrap();
        let mut expected = u64::MAX;
        let mut original = 0;
        permutations(&mut window.to_vec(), 0, &mut |candidate| {
            game.reset();
            for &v in prefix {
                game.eliminate(v);
            }
            let cost = candidate.iter().map(|&v| game.eliminate(v).pow(2)).sum();
            if candidate == window {
                original = cost;
            }
            expected = expected.min(cost);
        });
        assert_eq!(best, expected);
        assert_eq!(incumbent, original);
        game.reset();
        for &v in prefix {
            game.eliminate(v);
        }
        assert_eq!(
            order.iter().map(|&v| game.eliminate(v).pow(2)).sum::<u64>(),
            best
        );
        if best == incumbent {
            assert_eq!(order, window);
        }
    }

    #[test]
    fn window_dp_exhaustive_four_vertex_graphs() {
        let edges: Vec<_> = (0..4)
            .flat_map(|v| (v + 1..4).map(move |u| (v, u)))
            .collect();
        for mask in 0usize..1 << edges.len() {
            let selected: Vec<_> = edges
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, &e)| e)
                .collect();
            verify_window(&Pattern::from_edges(4, &selected), &[], &[0, 1, 2, 3]);
        }
    }

    #[test]
    fn window_dp_fourteen_pivots_preserves_ties_and_boundary_cost() {
        let n = 17;
        let edges: Vec<_> = (0..n)
            .flat_map(|v| (v + 1..n).map(move |u| (v, u)))
            .collect();
        let p = Pattern::from_edges(n, &edges);
        let pristine = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = Game::new(n, &pristine).unwrap();
        game.reset();
        game.eliminate(0);
        let vertices: Vec<_> = (1..15).collect();
        let mut work = TripleWork {
            remaining: 10_000_000,
        };
        let (order, best, incumbent) = solve_component(&game, &vertices, &mut work).unwrap();
        let expected: u64 = (3u64..=16).map(|c| c * c).sum();
        assert_eq!((best, incumbent), (expected, expected));
        assert_eq!(order, vertices);
    }

    #[test]
    fn window_dp_matches_exhaustive_with_prefix_and_external_boundary() {
        for n in [9, 70] {
            for sample in 0..8 {
                let edges: Vec<_> = (0..n)
                    .flat_map(|v| {
                        (v + 1..n)
                            .filter(move |&u| (v * 31 + u * 17 + sample * 13) % 11 < 3)
                            .map(move |u| (v, u))
                    })
                    .collect();
                verify_window(
                    &Pattern::from_edges(n, &edges),
                    &[0, 1],
                    &[2, 3, 4, 5, 6, 7],
                );
            }
        }
    }

    #[test]
    fn window_dp_descent_is_deterministic_and_never_worsens() {
        for n in [8, 17, 65] {
            let edges: Vec<_> = (0..n)
                .flat_map(|v| {
                    (v + 1..n)
                        .filter(move |&u| (v * 19 + u * 7) % 13 < 3)
                        .map(move |u| (v, u))
                })
                .collect();
            let p = Pattern::from_edges(n, &edges);
            let seed: Vec<_> = (0..n).rev().collect();
            let pristine = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
            let mut game = Game::new(n, &pristine).unwrap();
            let before = game.replay_flops(&seed);
            for budget in [0, 1, 1_000, 10_000, 100_000, 10_000_000] {
                let first = subset_window_descent(n, &p.col_ptr, &p.row_idx, &seed, 8, 3, budget);
                assert_eq!(
                    first,
                    subset_window_descent(n, &p.col_ptr, &p.row_idx, &seed, 8, 3, budget)
                );
                if let Some(order) = first {
                    let mut sorted = order.clone();
                    sorted.sort_unstable();
                    assert_eq!(sorted, (0..n).collect::<Vec<_>>());
                    assert!(game.replay_flops(&order) < before);
                }
            }
        }
    }

    #[test]
    fn window_dp_rejects_malformed_inputs() {
        let p = Pattern::from_edges(4, &[(0, 1)]);
        for seed in [&[0, 1, 2][..], &[0, 1, 1, 3][..], &[0, 1, 2, 4][..]] {
            assert!(
                subset_window_descent(4, &p.col_ptr, &p.row_idx, seed, 8, 1, 1_000_000).is_none()
            );
        }
        assert!(subset_window_descent(
            4,
            &[0, 0, 2, 1, 2],
            &p.row_idx,
            &[0, 1, 2, 3],
            8,
            1,
            1_000_000
        )
        .is_none());
        assert!(subset_window_descent(
            4,
            &p.col_ptr,
            &p.row_idx,
            &[0, 1, 2, 3],
            MAX_WIDTH + 1,
            1,
            1_000_000
        )
        .is_none());
    }

    #[test]
    #[ignore]
    fn probe_window_dp_candidates() {
        let configs = [(8, 2, 16_000_000), (10, 2, 24_000_000), (12, 2, 32_000_000)];
        for (name, pattern) in crate::corpus::corpus() {
            if pattern.n > max_dimension() || pattern.nnz() > 200_000 {
                continue;
            }
            let incumbent = crate::ordering::order(&pattern);
            let before = ssi_scoring::score(&pattern, &incumbent).flops;
            for (width, sweeps, budget) in configs {
                let start = std::time::Instant::now();
                let candidate = subset_window_descent(
                    pattern.n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &incumbent,
                    width,
                    sweeps,
                    budget,
                );
                let elapsed = start.elapsed().as_secs_f64();
                let after = candidate.as_ref().map_or(before, |p| {
                    let mut sorted = p.clone();
                    sorted.sort_unstable();
                    assert_eq!(sorted, (0..pattern.n).collect::<Vec<_>>());
                    ssi_scoring::score(&pattern, p).flops
                });
                assert!(after <= before, "{name}: width {width}");
                println!(
                    "WINDOW_DP\t{name}\t{}\t{}\t{before}\t{after}\t{elapsed:.6}\t{width}",
                    pattern.n,
                    pattern.nnz()
                );
            }
            let start = std::time::Instant::now();
            let mut candidate = incumbent;
            for (width, sweeps, budget) in [configs[0], configs[2], configs[1]] {
                if let Some(next) = subset_window_descent(
                    pattern.n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &candidate,
                    width,
                    sweeps,
                    budget,
                ) {
                    candidate = next;
                }
            }
            let elapsed = start.elapsed().as_secs_f64();
            let after = ssi_scoring::score(&pattern, &candidate).flops;
            assert!(after <= before, "{name}: chained windows");
            println!(
                "WINDOW_DP\t{name}\t{}\t{}\t{before}\t{after}\t{elapsed:.6}\t0",
                pattern.n,
                pattern.nnz()
            );
        }
    }
}
