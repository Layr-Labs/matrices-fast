//! Exact local costs from internal adjacency and an external-incidence histogram.
//!
//! For a connected eliminated subset C, the pivot column has width
//! 1 + |N_internal(C) minus C| + boundary_count - zeta[complement(C)].
//! External IDs and external-external edges do not affect this conditional
//! problem. Exact keys therefore permit reuse across repeated live blocks,
//! while preserving boundary multiplicity and all prefix-induced fill.
#![allow(dead_code)]

use super::{Game, TripleWork};
use std::collections::HashMap;

/// Same width ceiling as `window_dp`. Do not raise without the audit noted above.
const MAX_WIDTH: usize = 14;
const MAX_MEMO_ENTRIES: usize = 2_048;
const MAX_MEMO_BYTES: usize = 4 * 1024 * 1024;

/// Counters for the memoization probe (per engine instance / per order).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MemoStats {
    /// Components looked up (one per `solve_component` call that clears budget).
    pub(crate) probes: u64,
    /// Lookups that found a stored solution.
    pub(crate) hits: u64,
    /// Hits that were recomputed and matched the stored solution exactly.
    pub(crate) verified_hits: u64,
    /// New solutions inserted (misses).
    pub(crate) inserted: u64,
    pub(crate) trivial: u64,
}

#[cfg(test)]
thread_local! {
    static PROBE_STATS: std::cell::RefCell<MemoStats> =
        std::cell::RefCell::new(MemoStats::default());
}

#[cfg(test)]
pub(crate) fn take_probe_stats() -> MemoStats {
    PROBE_STATS.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

#[cfg(test)]
impl Drop for SignatureEngine {
    fn drop(&mut self) {
        PROBE_STATS.with(|cell| {
            let mut total = cell.borrow_mut();
            total.probes += self.stats.probes;
            total.hits += self.stats.hits;
            total.verified_hits += self.stats.verified_hits;
            total.inserted += self.stats.inserted;
            total.trivial += self.stats.trivial;
        });
    }
}

/// Which price `solve_component` charges the shared budget. The choice affects
/// ONLY budget gating (which components are funded), never the per-component
/// result, which is identical under both models whenever the budget clears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChargeModel {
    /// Charge the union kernel's exact price `states*(16k + 6w + 24)` so budget
    /// and skip decisions match `window_dp` byte-for-byte. Default; preserves the
    /// production descent exactly while still cutting real CPU/memory work.
    UnionParity,
    /// Price extraction, per-neighbor work, transforms and dynamic programming.
    /// This can exceed the union price for small components with large halos.
    SignatureTrue,
}

/// Exact, label-sensitive key: internal adjacency (as given) plus the external
/// incidence multiset. Deliberately NOT isomorphism-canonicalized — that keeps
/// the key sound with zero false-hit risk, at the cost of missing hits between
/// differently labelled but isomorphic blocks (a documented tradeoff).
#[derive(Clone, PartialEq, Eq, Hash)]
struct SigKey {
    k: u8,
    inside: [u16; MAX_WIDTH],
    /// `(signature, count)` pairs for nonzero buckets, ascending by signature.
    hist: Vec<(u16, u32)>,
}

#[derive(Clone)]
struct MemoEntry {
    best: u64,
    incumbent: u64,
    /// The chosen local-index order, including the owning engine's
    /// immutable tie policy.
    order_local: Vec<u8>,
}

/// Reusable engine. Allocate once per order (parent), reuse across every window
/// and connected component in that order, then drop. `reset_memo` clears the
/// memo between orders while retaining the scratch buffers.
pub(crate) struct SignatureEngine {
    n: usize,

    // ── n-sized scratch, cleared in O(touched) via an epoch stamp ────────────
    comp_stamp: Vec<u32>,
    sig: Vec<u16>,
    sig_stamp: Vec<u32>,
    epoch: u32,
    touched: Vec<usize>,

    // ── 2^MAX_WIDTH-sized scratch, active region cleared per call ────────────
    hist: Vec<u32>, // pre-zeta counts, then zeta (Z) in place
    nbr_union: Vec<u16>,
    widths: Vec<u64>,
    components: Vec<u16>,
    best: Vec<u64>,
    path: Vec<u64>,

    // ── within-invocation memo (no static/persistent state) ──────────────────
    memo: HashMap<SigKey, MemoEntry>,
    memo_bytes: usize,
    verify_on_hit: bool,
    // Fixed at construction: strict and neutral entries never share a memo.
    neutral_largest: bool,
    charge_model: ChargeModel,
    stats: MemoStats,
}

impl SignatureEngine {
    /// `n` is the vertex count of the graph the engine will be queried against.
    /// It sizes the per-vertex scratch, so it MUST be `>= game.n` for every game
    /// passed to `solve_component` (use the game's own `n`); otherwise a live
    /// external neighbour could fall outside the scratch and be miscounted.
    pub(crate) fn new(n: usize) -> Self {
        SignatureEngine {
            n,
            comp_stamp: vec![0; n],
            sig: vec![0; n],
            sig_stamp: vec![0; n],
            epoch: 0,
            touched: Vec::new(),
            hist: Vec::new(),
            nbr_union: Vec::new(),
            widths: Vec::new(),
            components: Vec::new(),
            best: Vec::new(),
            path: Vec::new(),
            memo: HashMap::new(),
            memo_bytes: 0,
            // Verify every hit by exact recompute in debug builds; production
            // wiring can flip this on to sample the certificate.
            verify_on_hit: cfg!(debug_assertions),
            neutral_largest: false,
            // Default to production-identical budget gating; opt in to the honest
            // envelope explicitly once the parent wants to extend exploration.
            charge_model: ChargeModel::UnionParity,
            stats: MemoStats::default(),
        }
    }

    /// A separate engine and memo for largest local-index optimal orders.
    /// The tie policy has no setter and remains fixed across memo resets.
    pub(crate) fn new_neutral_largest(n: usize) -> Self {
        let mut engine = Self::new(n);
        engine.neutral_largest = true;
        engine
    }

    pub(crate) fn stats(&self) -> MemoStats {
        self.stats
    }

    /// Clear the memo and counters between distinct orders. Scratch is kept.
    pub(crate) fn reset_memo(&mut self) {
        self.memo.clear();
        self.memo_bytes = 0;
        self.stats = MemoStats::default();
    }

    pub(crate) fn set_verify_on_hit(&mut self, verify: bool) {
        self.verify_on_hit = verify;
    }

    /// Legacy charges preserve the original trajectory; the alternative prices
    /// extraction and uncached solving separately.
    pub(crate) fn set_charge_model(&mut self, model: ChargeModel) {
        self.charge_model = model;
    }

    /// Worst-case prices for one uncached component, including live incidences.
    pub(crate) fn charge_costs(k: usize, w: usize, incident: usize) -> (usize, usize) {
        let states = 1usize << k;
        let per_state = 16usize.saturating_mul(k);
        let union = states.saturating_mul(
            per_state
                .saturating_add(6usize.saturating_mul(w))
                .saturating_add(24),
        );
        let signature = states
            .saturating_mul(32usize.saturating_mul(k).saturating_add(64))
            .saturating_add(8usize.saturating_mul(k).saturating_mul(w))
            .saturating_add(16usize.saturating_mul(incident))
            .saturating_add(32usize.saturating_mul(k));
        (union, signature)
    }

    fn begin_epoch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            for s in self.comp_stamp.iter_mut() {
                *s = 0;
            }
            for s in self.sig_stamp.iter_mut() {
                *s = 0;
            }
            self.epoch = 1;
        }
    }

    /// Exact drop-in replacement for `window_dp::solve_component`: returns the
    /// improved (or unchanged) order over `vertices`, its cost `best`, and the
    /// original-order cost `incumbent`. `None` iff the input is malformed or the
    /// (union-identical) budget cannot fund the component.
    pub(crate) fn solve_component(
        &mut self,
        game: &Game<'_>,
        vertices: &[usize],
        work: &mut TripleWork,
    ) -> Option<(Vec<usize>, u64, u64)> {
        let k = vertices.len();
        if k == 0 || k > MAX_WIDTH || game.n > self.n {
            return None;
        }
        // The scratch (`comp_stamp`/`sig`/`sig_stamp`) is indexed by live vertex
        // id, so it must span the whole graph or a real external neighbour would
        // be silently dropped from the boundary histogram (undercounting width).
        // Callers must build the engine with `n >= game.n` (typically `game.n`).
        debug_assert!(
            game.n <= self.n,
            "SignatureEngine scratch n={} too small for graph n={}",
            self.n,
            game.n
        );
        for (i, &v) in vertices.iter().enumerate() {
            if v >= self.n || v >= game.n || vertices[..i].contains(&v) {
                return None;
            }
        }
        let states = 1usize << k;

        let incident = vertices.iter().map(|&v| game.deg[v] as usize).sum();
        let (union_cost, signature_cost) = Self::charge_costs(k, game.w, incident);
        let solve_cost = states * (32 * k + 56);
        let cost = match self.charge_model {
            ChargeModel::UnionParity => union_cost,
            ChargeModel::SignatureTrue => signature_cost - solve_cost,
        };
        if !work.charge(cost) {
            return None;
        }

        // ── internal (within-window) adjacency, k-bit masks ──────────────────
        let mut inside = [0u16; MAX_WIDTH];
        let w = game.w;
        for (i, &v) in vertices.iter().enumerate() {
            for (j, &u) in vertices.iter().enumerate() {
                if i != j && game.adj[v * w + u / 64] & (1u64 << (u % 64)) != 0 {
                    inside[i] |= 1u16 << j;
                }
            }
        }

        // ── boundary signature histogram (external incidence multiset) ───────
        self.begin_epoch();
        for &v in vertices {
            self.comp_stamp[v] = self.epoch;
        }
        self.touched.clear();
        for (i, &v) in vertices.iter().enumerate() {
            let row = &game.adj[v * w..v * w + w];
            for (word_idx, &word) in row.iter().enumerate() {
                let mut bits = word;
                while bits != 0 {
                    let u = word_idx * 64 + bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    if self.comp_stamp[u] == self.epoch {
                        continue;
                    }
                    if self.sig_stamp[u] != self.epoch {
                        self.sig_stamp[u] = self.epoch;
                        self.sig[u] = 0;
                        self.touched.push(u);
                    }
                    self.sig[u] |= 1u16 << i;
                }
            }
        }
        let total = self.touched.len() as u32;

        // pre-zeta histogram over the active region, plus the exact memo key.
        self.hist.resize(states, 0);
        self.hist.fill(0);
        for idx in 0..self.touched.len() {
            let u = self.touched[idx];
            let s = self.sig[u] as usize;
            self.hist[s] += 1;
        }
        let mut hist_pairs: Vec<(u16, u32)> = Vec::new();
        for m in 1..states {
            if self.hist[m] != 0 {
                hist_pairs.push((m as u16, self.hist[m])); // ascending by signature
            }
        }
        let key = SigKey {
            k: k as u8,
            inside,
            hist: hist_pairs,
        };

        self.stats.probes += 1;
        if let Some(entry) = self.memo.get(&key).cloned() {
            self.stats.hits += 1;
            if self.verify_on_hit {
                // Exact recompute certificate: a hit must reproduce the stored
                // solution bit-for-bit, or the key is unsound.
                let (best, incumbent, order_local) = self.run_dp(k, states, &inside, total);
                assert_eq!(best, entry.best, "memo certificate: best mismatch");
                assert_eq!(
                    incumbent, entry.incumbent,
                    "memo certificate: incumbent mismatch"
                );
                assert_eq!(
                    order_local, entry.order_local,
                    "memo certificate: order mismatch"
                );
                self.stats.verified_hits += 1;
            }
            let order = map_order(vertices, &entry.order_local);
            return Some((order, entry.best, entry.incumbent));
        }

        let full = (states - 1) as u16;
        let complete = (0..k).all(|i| inside[i] == (full ^ (1 << i)))
            && key.hist.iter().all(|&(signature, _)| signature == full);
        let (best, incumbent, order_local) = if complete {
            self.stats.trivial += 1;
            let cost = (1..=k).map(|i| (total as u64 + i as u64).pow(2)).sum();
            let order = if self.neutral_largest {
                (0..k as u8).rev().collect()
            } else {
                (0..k as u8).collect()
            };
            (cost, cost, order)
        } else {
            if self.charge_model == ChargeModel::SignatureTrue && !work.charge(solve_cost) {
                return None;
            }
            self.run_dp(k, states, &inside, total)
        };
        let entry_bytes =
            128 + key.hist.capacity() * std::mem::size_of::<(u16, u32)>() + order_local.len();
        if self.memo.len() < MAX_MEMO_ENTRIES
            && entry_bytes <= MAX_MEMO_BYTES.saturating_sub(self.memo_bytes)
        {
            self.memo.insert(
                key,
                MemoEntry {
                    best,
                    incumbent,
                    order_local: order_local.clone(),
                },
            );
            self.memo_bytes += entry_bytes;
            self.stats.inserted += 1;
        }
        let order = map_order(vertices, &order_local);
        Some((order, best, incumbent))
    }

    /// Consumes the pre-zeta `self.hist`, fills `widths`/`components`/`nbr_union`,
    /// runs the identical pivot DP, and returns `(best, incumbent, order_local)`.
    fn run_dp(
        &mut self,
        k: usize,
        states: usize,
        inside: &[u16; MAX_WIDTH],
        total: u32,
    ) -> (u64, u64, Vec<u8>) {
        if self.neutral_largest {
            self.run_dp_policy::<true>(k, states, inside, total)
        } else {
            self.run_dp_policy::<false>(k, states, inside, total)
        }
    }

    fn run_dp_policy<const NEUTRAL: bool>(
        &mut self,
        k: usize,
        states: usize,
        inside: &[u16; MAX_WIDTH],
        total: u32,
    ) -> (u64, u64, Vec<u8>) {
        let full = states - 1;
        self.nbr_union.resize(states, 0);
        self.widths.resize(states, 0);
        self.components.resize(states * k, 0);
        self.best.resize(states, 0);
        self.path.resize(states, 0);

        // zeta / sum-over-subsets in place: hist[m] -> Z[m] = Σ_{s⊆m} hist[s].
        for i in 0..k {
            let bit = 1usize << i;
            for m in 0..states {
                if m & bit != 0 {
                    let lower = self.hist[m ^ bit];
                    self.hist[m] += lower;
                }
            }
        }

        // Fresh connectivity + widths for this component.
        for u in 0..k {
            self.components[u] = 0; // mask 0 row
        }
        self.nbr_union[0] = 0;
        for m in 0..states {
            self.widths[m] = 0;
        }

        for mask in 1..states {
            let v = mask.trailing_zeros() as usize;
            let bit = 1usize << v;
            let rest = mask ^ bit;

            // connected component of `mask` containing its lowest vertex `v`.
            let mut merged = bit as u16;
            let mut neighbors = inside[v] & rest as u16;
            while neighbors != 0 {
                let u = neighbors.trailing_zeros() as usize;
                neighbors &= neighbors - 1;
                merged |= self.components[rest * k + u];
            }
            for u in 0..k {
                self.components[mask * k + u] = if merged & (1 << u) != 0 {
                    merged
                } else {
                    self.components[rest * k + u]
                };
            }

            // internal neighbour union (k-bit), incremental like the union table.
            self.nbr_union[mask] = self.nbr_union[rest] | inside[v];

            if merged as usize == mask {
                let intbound = (self.nbr_union[mask] & !(mask as u16)).count_ones() as u64;
                let extbound = (total - self.hist[full ^ mask]) as u64;
                self.widths[mask] = 1 + intbound + extbound;
            }
        }

        // ── pivot DP (identical mask/path rules to window_dp) ────────────────
        self.best[0] = 0;
        self.path[0] = 0;
        for m in 1..states {
            self.best[m] = u64::MAX;
            self.path[m] = if NEUTRAL { 0 } else { u64::MAX };
        }
        for mask in 0..states - 1 {
            if NEUTRAL && self.best[mask] == u64::MAX {
                continue;
            }
            for pivot in 0..k {
                let bit = 1usize << pivot;
                if mask & bit != 0 {
                    continue;
                }
                let next = mask | bit;
                let width = self.widths[self.components[next * k + pivot] as usize];
                let cost = self.best[mask] + width * width;
                let code = (self.path[mask] << 4) | pivot as u64;
                let preferred_tie = if NEUTRAL {
                    code > self.path[next]
                } else {
                    code < self.path[next]
                };
                if cost < self.best[next] || (cost == self.best[next] && preferred_tie) {
                    self.best[next] = cost;
                    self.path[next] = code;
                }
            }
        }

        let incumbent: u64 = (0..k)
            .map(|pivot| {
                let mask = (1usize << (pivot + 1)) - 1;
                let width = self.widths[self.components[mask * k + pivot] as usize];
                width * width
            })
            .sum();

        let best = self.best[states - 1];
        let order_local: Vec<u8> = if best < incumbent || (NEUTRAL && best == incumbent) {
            (0..k)
                .map(|i| ((self.path[states - 1] >> (4 * (k - i - 1))) & 15) as u8)
                .collect()
        } else {
            (0..k as u8).collect()
        };
        (best, incumbent, order_local)
    }

    /// Test-only: run a fresh (miss) solve and expose the exact `widths` and
    /// `components` buffers for a byte-for-byte differential against the
    /// union-based kernel.
    #[cfg(test)]
    pub(crate) fn widths_components_for_test(
        &mut self,
        game: &Game<'_>,
        vertices: &[usize],
    ) -> Option<(Vec<u64>, Vec<u16>)> {
        self.reset_memo();
        let mut work = TripleWork {
            remaining: i64::MAX,
        };
        let _ = self.solve_component(game, vertices, &mut work)?;
        let k = vertices.len();
        let states = 1usize << k;
        if self.stats.trivial != 0 {
            let mut inside = [0u16; MAX_WIDTH];
            for (i, mask) in inside.iter_mut().enumerate().take(k) {
                *mask = (states - 1) as u16 ^ (1 << i);
            }
            self.run_dp(k, states, &inside, self.touched.len() as u32);
        }
        Some((
            self.widths[..states].to_vec(),
            self.components[..states * k].to_vec(),
        ))
    }
}

/// Map a local order (`0..k` permutation) back onto the actual window vertices.
fn map_order(vertices: &[usize], order_local: &[u8]) -> Vec<usize> {
    order_local.iter().map(|&i| vertices[i as usize]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pattern;

    /// Faithful copy of the production union kernel (`window_dp::solve_component`
    /// body) used as the differential oracle. Kept in-file because that fn is
    /// private to a sibling module; the parent can later add a direct diff test
    /// inside `window_dp` where it is reachable.
    fn ref_solution(
        game: &Game<'_>,
        vertices: &[usize],
    ) -> (Vec<u64>, Vec<u16>, u64, u64, Vec<usize>) {
        let k = vertices.len();
        let states = 1usize << k;
        let w = game.w;
        let mut inside = vec![0u16; k];
        for (i, &v) in vertices.iter().enumerate() {
            for (j, &u) in vertices.iter().enumerate() {
                if i != j && game.adj[v * w + u / 64] & (1u64 << (u % 64)) != 0 {
                    inside[i] |= 1 << j;
                }
            }
        }
        let mut components = vec![0u16; states * k];
        let mut unions = vec![0u64; states * w];
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
            for word in 0..w {
                unions[mask * w + word] = unions[rest * w + word] | game.adj[vertex * w + word];
            }
            unions[mask * w + vertex / 64] |= 1u64 << (vertex % 64);
            if merged as usize == mask {
                let boundary: u64 = unions[mask * w..(mask + 1) * w]
                    .iter()
                    .map(|word| word.count_ones() as u64)
                    .sum();
                widths[mask] = boundary - mask.count_ones() as u64 + 1;
            }
        }
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
        let incumbent: u64 = (0..k)
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
        (widths, components, best[states - 1], incumbent, order)
    }

    fn build_game<'a>(p: &Pattern, adj: &'a [u64]) -> Game<'a> {
        Game::new(p.n, adj).unwrap()
    }

    fn eliminate_prefix(game: &mut Game<'_>, prefix: &[usize]) {
        game.reset();
        for &v in prefix {
            game.eliminate(v);
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

    fn sparse_pattern(n: usize, sample: usize) -> Pattern {
        let edges: Vec<_> = (0..n)
            .flat_map(|v| {
                (v + 1..n)
                    .filter(move |&u| (v * 31 + u * 17 + sample * 13) % 11 < 3)
                    .map(move |u| (v, u))
            })
            .collect();
        Pattern::from_edges(n, &edges)
    }

    /// The signature/zeta kernel reproduces the union kernel's widths and
    /// components exactly, across prefixes and external boundaries.
    #[test]
    fn signature_widths_match_union_kernel() {
        for &n in &[9usize, 40, 70] {
            for sample in 0..6 {
                let p = sparse_pattern(n, sample);
                let adj = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
                let mut game = build_game(&p, &adj);
                let mut engine = SignatureEngine::new(n);
                for prefix in [&[][..], &[0, 1][..], &[0, 1, 2, 3][..]] {
                    for window in [
                        &[2usize, 3, 4, 5][..],
                        &[4, 5, 6, 7, 8][..],
                        &[3, 5, 7, 8][..],
                    ] {
                        if window.iter().any(|&v| v >= n) || prefix.iter().any(|&v| v >= n) {
                            continue;
                        }
                        if window.iter().any(|w| prefix.contains(w)) {
                            continue;
                        }
                        eliminate_prefix(&mut game, prefix);
                        let (rw, rc, _, _, _) = ref_solution(&game, window);
                        let (sw, sc) = engine.widths_components_for_test(&game, window).unwrap();
                        assert_eq!(
                            sw, rw,
                            "widths mismatch n={n} prefix={prefix:?} win={window:?}"
                        );
                        assert_eq!(sc, rc, "components mismatch n={n} win={window:?}");
                    }
                }
            }
        }
    }

    /// Full pipeline against brute-force permutation cost (small windows),
    /// mirroring the production `verify_window` oracle.
    #[test]
    fn signature_solve_matches_bruteforce() {
        for &n in &[9usize, 60] {
            for sample in 0..6 {
                let p = sparse_pattern(n, sample);
                let adj = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
                let mut game = build_game(&p, &adj);
                let mut engine = SignatureEngine::new(n);
                let prefix = [0usize, 1];
                let window = [2usize, 3, 4, 5, 6];
                let mut work = TripleWork {
                    remaining: i64::MAX,
                };
                eliminate_prefix(&mut game, &prefix);
                let (order, best, incumbent) =
                    engine.solve_component(&game, &window, &mut work).unwrap();

                let mut expected = u64::MAX;
                let mut original = 0u64;
                permutations(&mut window.to_vec(), 0, &mut |candidate| {
                    game.reset();
                    for &v in &prefix {
                        game.eliminate(v);
                    }
                    let cost: u64 = candidate.iter().map(|&v| game.eliminate(v).pow(2)).sum();
                    if candidate == &window[..] {
                        original = cost;
                    }
                    expected = expected.min(cost);
                });
                assert_eq!(best, expected, "n={n} sample={sample}");
                assert_eq!(incumbent, original, "n={n} sample={sample}");

                game.reset();
                for &v in &prefix {
                    game.eliminate(v);
                }
                let replay: u64 = order.iter().map(|&v| game.eliminate(v).pow(2)).sum();
                assert_eq!(replay, best);
                if best == incumbent {
                    assert_eq!(order, window);
                }
                let mut sorted = order.clone();
                sorted.sort_unstable();
                let mut expect_ids = window.to_vec();
                expect_ids.sort_unstable();
                assert_eq!(sorted, expect_ids);
            }
        }
    }

    /// Memo probe: an identical component is a verified hit; a different one is a
    /// miss. Counters and results are exact.
    #[test]
    fn memo_hit_is_exact_and_counted() {
        let n = 60;
        let p = sparse_pattern(n, 2);
        let adj = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = build_game(&p, &adj);
        game.reset();
        game.eliminate(0);
        game.eliminate(1);

        let mut engine = SignatureEngine::new(n);
        engine.set_verify_on_hit(true);
        let window = [2usize, 3, 4, 5, 6];

        let mut work = TripleWork {
            remaining: i64::MAX,
        };
        let first = engine.solve_component(&game, &window, &mut work).unwrap();
        let mut work2 = TripleWork {
            remaining: i64::MAX,
        };
        let second = engine.solve_component(&game, &window, &mut work2).unwrap();
        assert_eq!(first, second, "hit must reproduce the stored solution");

        let s = engine.stats();
        assert_eq!(s.probes, 2);
        assert_eq!(s.hits, 1);
        assert_eq!(s.verified_hits, 1);
        assert_eq!(s.inserted, 1);

        // A third probe on another component: whatever the structural outcome,
        // every probe is exactly one of hit-or-insert, so the counters must
        // still balance. (The guaranteed miss/insert path is covered by the
        // very first probe above, which inserted into an empty memo.)
        let other = [7usize, 8, 9, 10, 11];
        let mut work3 = TripleWork {
            remaining: i64::MAX,
        };
        let _ = engine.solve_component(&game, &other, &mut work3);
        let s2 = engine.stats();
        assert_eq!(s2.probes, 3);
        assert_eq!(s2.hits + s2.inserted, s2.probes, "counters must balance");
        assert_eq!(s2.verified_hits, s2.hits, "every hit is verified here");
    }

    /// Budget is charged identically to the union kernel: too small a budget
    /// yields `None` and consumes nothing extra.
    #[test]
    fn budget_charge_matches_union_and_binds() {
        let n = 40;
        let p = sparse_pattern(n, 1);
        let adj = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = build_game(&p, &adj);
        game.reset();
        let mut engine = SignatureEngine::new(n);
        let window = [0usize, 1, 2, 3, 4];
        let k = window.len();
        let states = 1usize << k;
        let need = states * (16 * k + 6 * game.w + 24);

        let mut too_small = TripleWork {
            remaining: need as i64 - 1,
        };
        assert!(engine
            .solve_component(&game, &window, &mut too_small)
            .is_none());

        let mut exact = TripleWork {
            remaining: need as i64,
        };
        assert!(engine.solve_component(&game, &window, &mut exact).is_some());
        assert_eq!(
            exact.remaining, 0,
            "charge must equal the union-kernel cost"
        );
    }

    /// Determinism and input validation.
    #[test]
    fn deterministic_and_rejects_malformed() {
        let n = 32;
        let p = sparse_pattern(n, 3);
        let adj = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = build_game(&p, &adj);
        game.reset();
        let mut engine = SignatureEngine::new(n);
        let window = [1usize, 2, 3, 4];

        let mut w1 = TripleWork {
            remaining: i64::MAX,
        };
        let a = engine.solve_component(&game, &window, &mut w1).unwrap();
        engine.reset_memo();
        let mut w2 = TripleWork {
            remaining: i64::MAX,
        };
        let b = engine.solve_component(&game, &window, &mut w2).unwrap();
        assert_eq!(a, b);

        // width > MAX_WIDTH rejected.
        let too_wide: Vec<usize> = (0..MAX_WIDTH + 1).collect();
        let mut w3 = TripleWork {
            remaining: i64::MAX,
        };
        assert!(engine.solve_component(&game, &too_wide, &mut w3).is_none());

        // out-of-range vertex rejected.
        let mut w4 = TripleWork {
            remaining: i64::MAX,
        };
        assert!(engine.solve_component(&game, &[0, n], &mut w4).is_none());
    }

    #[test]
    fn signature_charge_is_cheaper_and_result_invariant() {
        for k in 1..=MAX_WIDTH {
            for &w in &[1usize, 4, 32, 188] {
                let incident = 2 * k;
                let (union, sig) = SignatureEngine::charge_costs(k, w, incident);
                let states = 1usize << k;
                assert_eq!(union, states * (16 * k + 6 * w + 24));
                assert_eq!(
                    sig,
                    states * (32 * k + 64) + 8 * k * w + 16 * incident + 32 * k
                );
            }
        }

        let n = 4_096;
        let edges: Vec<_> = (0..n - 1).map(|v| (v, v + 1)).collect();
        let p = crate::Pattern::from_edges(n, &edges);
        let adj = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = build_game(&p, &adj);
        game.reset();
        game.eliminate(0);
        let window = [10usize, 11, 12, 13, 14, 15, 16, 17];
        let k = window.len();
        let incident = window.iter().map(|&v| game.deg[v] as usize).sum();
        let (union_cost, sig_cost) = SignatureEngine::charge_costs(k, game.w, incident);
        assert!(sig_cost < union_cost);

        // Same result under either model when the budget clears.
        let mut parity = SignatureEngine::new(n);
        let mut w_parity = TripleWork {
            remaining: i64::MAX,
        };
        let r_parity = parity
            .solve_component(&game, &window, &mut w_parity)
            .unwrap();

        let mut honest = SignatureEngine::new(n);
        honest.set_charge_model(ChargeModel::SignatureTrue);
        let mut w_honest = TripleWork {
            remaining: i64::MAX,
        };
        let r_honest = honest
            .solve_component(&game, &window, &mut w_honest)
            .unwrap();

        assert_eq!(
            r_parity, r_honest,
            "charge model must not change the result"
        );
        assert_eq!(
            (i64::MAX - w_parity.remaining) as usize,
            union_cost,
            "parity charges the union price"
        );
        assert_eq!(
            (i64::MAX - w_honest.remaining) as usize,
            sig_cost,
            "SignatureTrue charges the honest price"
        );

        // The honest model funds a component that parity would starve: pick a
        // budget between the two prices.
        let mid = (sig_cost + union_cost) / 2;
        assert!(sig_cost <= mid && mid < union_cost);
        let mut parity_starved = SignatureEngine::new(n);
        let mut wp = TripleWork {
            remaining: mid as i64,
        };
        assert!(parity_starved
            .solve_component(&game, &window, &mut wp)
            .is_none());
        let mut honest_funded = SignatureEngine::new(n);
        honest_funded.set_charge_model(ChargeModel::SignatureTrue);
        let mut wh = TripleWork {
            remaining: mid as i64,
        };
        assert!(honest_funded
            .solve_component(&game, &window, &mut wh)
            .is_some());
    }

    #[test]
    fn bounded_memo_and_invalid_workspace() {
        let p = sparse_pattern(64, 4);
        let adj = Game::build_adj(p.n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = build_game(&p, &adj);
        game.reset();
        let vertices = [1, 2, 3, 4, 5];
        let mut short = SignatureEngine::new(8);
        let mut budget = TripleWork {
            remaining: i64::MAX,
        };
        assert!(short
            .solve_component(&game, &vertices, &mut budget)
            .is_none());
        let mut engine = SignatureEngine::new(p.n);
        assert!(engine
            .solve_component(&game, &[1, 1], &mut budget)
            .is_none());
        engine.memo_bytes = MAX_MEMO_BYTES;
        let before = engine
            .solve_component(&game, &vertices, &mut budget)
            .unwrap();
        assert!(engine.memo.is_empty());
        engine.memo_bytes = 0;
        let after = engine
            .solve_component(&game, &vertices, &mut budget)
            .unwrap();
        assert_eq!(before, after);
        assert_eq!(engine.memo.len(), 1);
        assert!(engine.memo_bytes <= MAX_MEMO_BYTES);
    }

    #[test]
    fn complete_boundary_problem_skips_dynamic_programming() {
        let n = 17;
        let edges: Vec<_> = (0..n)
            .flat_map(|v| (v + 1..n).map(move |u| (v, u)))
            .collect();
        let p = crate::Pattern::from_edges(n, &edges);
        let adj = Game::build_adj(n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = build_game(&p, &adj);
        game.reset();
        let vertices: Vec<_> = (0..14).collect();
        let mut engine = SignatureEngine::new(n);
        engine.set_charge_model(ChargeModel::SignatureTrue);
        let mut budget = TripleWork {
            remaining: i64::MAX,
        };
        let (order, best, incumbent) = engine
            .solve_component(&game, &vertices, &mut budget)
            .unwrap();
        let expected = (4u64..=17).map(|x| x * x).sum();
        assert_eq!((best, incumbent), (expected, expected));
        assert_eq!(order, vertices);
        assert_eq!(engine.stats.trivial, 1);
        assert!(engine.best.is_empty());
    }

    #[test]
    fn fourteen_pivot_signatures_match_union_after_prefix() {
        let p = sparse_pattern(80, 3);
        let adj = Game::build_adj(p.n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = build_game(&p, &adj);
        eliminate_prefix(&mut game, &[0, 1]);
        let vertices: Vec<_> = (2..16).collect();
        let (_, _, best, incumbent, order) = ref_solution(&game, &vertices);
        let mut engine = SignatureEngine::new(p.n);
        engine.set_verify_on_hit(true);
        let mut work = TripleWork {
            remaining: i64::MAX,
        };
        let actual = engine.solve_component(&game, &vertices, &mut work).unwrap();
        assert_eq!(actual, (order, best, incumbent));
        assert_eq!(
            engine.solve_component(&game, &vertices, &mut work).unwrap(),
            actual
        );
        assert_eq!(engine.stats.verified_hits, 1);
    }

    #[test]
    fn repeated_blocks_reuse_solutions_but_changed_halos_do_not() {
        let block = [(0, 1), (1, 2), (0, 3), (1, 3), (2, 4)];
        let mut edges: Vec<_> = [0, 5, 10]
            .into_iter()
            .flat_map(|offset| block.map(|(u, v)| (u + offset, v + offset)))
            .collect();
        edges.push((3, 4));
        edges.push((12, 13));
        let p = crate::Pattern::from_edges(15, &edges);
        let adj = Game::build_adj(p.n, &p.col_ptr, &p.row_idx).unwrap();
        let mut game = build_game(&p, &adj);
        game.reset();
        let mut engine = SignatureEngine::new(p.n);
        engine.set_verify_on_hit(true);
        let mut work = TripleWork {
            remaining: i64::MAX,
        };
        let (first, best, incumbent) = engine
            .solve_component(&game, &[0, 1, 2], &mut work)
            .unwrap();
        let (second, second_best, second_incumbent) = engine
            .solve_component(&game, &[5, 6, 7], &mut work)
            .unwrap();
        assert_eq!(second, first.iter().map(|v| v + 5).collect::<Vec<_>>());
        assert_eq!((best, incumbent), (second_best, second_incumbent));
        assert_eq!(engine.stats.hits, 1);
        let third = engine
            .solve_component(&game, &[10, 11, 12], &mut work)
            .unwrap();
        let (_, _, expected_best, expected_incumbent, expected_order) =
            ref_solution(&game, &[10, 11, 12]);
        assert_eq!(third, (expected_order, expected_best, expected_incumbent));
        assert_eq!(engine.stats.hits, 1);
        assert_eq!(engine.stats.inserted, 2);
    }
}
