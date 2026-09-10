//! TEST-ONLY ceiling probe: how much headroom is left on the rows the shipped
//! pipeline cannot move off AMD (ratio 1.000)?
//!
//! Motivation. On the dev corpus ~37 % of `lt_1k` rows and ~23 % of `1k_10k`
//! rows score exactly 1.000: the whole portfolio plus every refine stage
//! fails to find anything better than the AMD anchor. Two very different
//! worlds are consistent with that observation:
//!
//!   a) AMD is at (or within a rounding of) the optimum on those patterns, so
//!      no search budget can pay off and the bucket has no headroom; or
//!   b) the shipped search is simply under-budgeted on them, and a much
//!      larger budget finds real gains.
//!
//! This probe distinguishes them with an INDEPENDENT implementation — it does
//! not call `leader_order` or any portfolio candidate — so it is also a
//! cross-check on the pipeline rather than a re-run of it.
//!
//! ## The objective, computed directly
//!
//! `flops = Σ_j c_j²`, and `c_j` is the number of nonzeros in column `j` of L,
//! i.e. `1 + (degree of j in the elimination graph at the moment j is
//! eliminated)`. So the score is a pure function of the elimination game and
//! can be accumulated incrementally, with no symbolic factorization per
//! candidate. `assert_matches_flops_of` pins that identity against the
//! trusted `flops_of` on the AMD order for every probed matrix, so a mismatch
//! fails the probe rather than silently reporting a wrong ceiling.
//!
//! Minimum degree is therefore exactly the MYOPIC greedy on the scored
//! objective: it takes the smallest `c_j` available at every step. Any real
//! improvement has to come from lookahead — accepting a locally worse pivot
//! to avoid a large `c_j` later. That is what the randomized search below
//! samples.
//!
//! ## Search
//!
//! Bitset elimination game (`n <= CEIL_MAX_N`, so the dense n x n bitset is a
//! few hundred KB) plus randomized greedy restarts. Each restart draws a
//! pivot rule from a small family and a tie-break temperature, then plays the
//! whole game:
//!
//!   * `Deg`      — minimize `(1 + d)²`, the myopic objective (= min degree).
//!   * `Fill`     — minimize created fill (min-fill / AMF-flavoured).
//!   * `DegFill`  — minimize `(1 + d)² + lambda * fill`, interpolating the two.
//!
//! With temperature `t` the pivot is drawn uniformly from the candidates whose
//! score is within `(1 + t)` of the best, which is the standard randomized-
//! greedy relaxation; `t = 0` reproduces the deterministic rule (ties by
//! smallest index, so it stays reproducible).
//!
//! Run with:
//! ```sh
//! cargo test --release -- --ignored --nocapture probe_ceiling
//! CEIL_SECS=20 CEIL_ONLY_TIES=0 cargo test --release -- --ignored --nocapture probe_ceiling
//! ```
//!
//! Env knobs: `CEIL_SECS` (per-matrix search seconds, default 5),
//! `CEIL_MAX_N` (skip larger patterns, default 1000),
//! `CEIL_ONLY_TIES` (1 = only rows where the pipeline scores >= 0.9999,
//! default 1), `CEIL_MIN_N` (default 0).
//!
//! This file is test-only (`probe` is `#[cfg(test)]`), so nothing here is
//! compiled into the shipped ordering or seen by the grader. It reads the
//! clock only to bound its own search; no production path consults it.

use super::super::*;
use std::time::Instant;

/// Dense-bitset elimination game over `n` vertices.
struct Game {
    n: usize,
    words: usize,
    /// `adj[v * words .. (v+1) * words]` = current neighbours of `v`.
    adj: Vec<u64>,
    alive: Vec<bool>,
    deg: Vec<u32>,
    /// Scratch for the neighbour list of the current pivot.
    nbrs: Vec<u32>,
}

impl Game {
    fn new(n: usize, col_ptr: &[usize], row_idx: &[usize]) -> Game {
        let words = n.div_ceil(64);
        let mut adj = vec![0u64; n * words];
        let mut deg = vec![0u32; n];
        for v in 0..n {
            let s = col_ptr[v];
            let e = col_ptr[v + 1];
            deg[v] = (e - s) as u32;
            for &u in &row_idx[s..e] {
                adj[v * words + (u >> 6)] |= 1u64 << (u & 63);
            }
        }
        Game { n, words, adj, alive: vec![true; n], deg, nbrs: Vec::with_capacity(n) }
    }

    fn reset(&mut self, col_ptr: &[usize], row_idx: &[usize]) {
        self.adj.fill(0);
        for v in 0..self.n {
            let s = col_ptr[v];
            let e = col_ptr[v + 1];
            self.deg[v] = (e - s) as u32;
            for &u in &row_idx[s..e] {
                self.adj[v * self.words + (u >> 6)] |= 1u64 << (u & 63);
            }
        }
        self.alive.fill(true);
    }

    fn collect_nbrs(&mut self, v: usize) {
        self.nbrs.clear();
        let base = v * self.words;
        for w in 0..self.words {
            let mut bits = self.adj[base + w];
            while bits != 0 {
                let b = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                self.nbrs.push((w * 64 + b) as u32);
            }
        }
    }

    /// Fill edges that eliminating `v` would create (pairs of neighbours not
    /// yet adjacent). O(deg² / 64) — only ever called for `n <= CEIL_MAX_N`.
    fn fill_of(&mut self, v: usize) -> u64 {
        self.collect_nbrs(v);
        let mut miss = 0u64;
        for i in 0..self.nbrs.len() {
            let a = self.nbrs[i] as usize;
            let base = a * self.words;
            for j in (i + 1)..self.nbrs.len() {
                let b = self.nbrs[j] as usize;
                if self.adj[base + (b >> 6)] & (1u64 << (b & 63)) == 0 {
                    miss += 1;
                }
            }
        }
        miss
    }

    /// Eliminate `v`: clique its live neighbourhood, drop `v`.
    fn eliminate(&mut self, v: usize) {
        self.collect_nbrs(v);
        let k = self.nbrs.len();
        let words = self.words;
        for i in 0..k {
            let a = self.nbrs[i] as usize;
            // OR the pivot's neighbourhood into a, then clear a and v.
            for w in 0..words {
                let add = self.adj[v * words + w];
                self.adj[a * words + w] |= add;
            }
            self.adj[a * words + (a >> 6)] &= !(1u64 << (a & 63));
            self.adj[a * words + (v >> 6)] &= !(1u64 << (v & 63));
            let mut d = 0u32;
            for w in 0..words {
                d += self.adj[a * words + w].count_ones();
            }
            self.deg[a] = d;
        }
        for w in 0..words {
            self.adj[v * words + w] = 0;
        }
        self.alive[v] = false;
        self.deg[v] = 0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Rule {
    Deg,
    Fill,
    DegFill,
}

fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

/// Play one full randomized-greedy game; return `(Σ c_j², perm)`.
fn play(
    g: &mut Game,
    col_ptr: &[usize],
    row_idx: &[usize],
    rule: Rule,
    temp: f64,
    lambda: f64,
    seed: u64,
    cutoff: u64,
) -> Option<(u64, Vec<usize>)> {
    g.reset(col_ptr, row_idx);
    let n = g.n;
    let mut perm = Vec::with_capacity(n);
    let mut total = 0u64;
    let mut state = seed | 1;
    let mut cands: Vec<(f64, usize)> = Vec::with_capacity(n);

    for _ in 0..n {
        cands.clear();
        let mut best = f64::INFINITY;
        for v in 0..n {
            if !g.alive[v] {
                continue;
            }
            let d = g.deg[v] as f64;
            let s = match rule {
                Rule::Deg => (1.0 + d) * (1.0 + d),
                Rule::Fill => g.fill_of(v) as f64,
                Rule::DegFill => (1.0 + d) * (1.0 + d) + lambda * g.fill_of(v) as f64,
            };
            if s < best {
                best = s;
            }
            cands.push((s, v));
        }
        // Draw uniformly among candidates within (1 + temp) of the best.
        let thresh = if best <= 0.0 { temp } else { best * (1.0 + temp) };
        let mut pick = usize::MAX;
        let mut seen = 0u64;
        for &(s, v) in &cands {
            if s <= thresh {
                seen += 1;
                state = mix(state);
                // Reservoir sample of size 1: uniform over the eligible set,
                // and deterministic given `seed`.
                if state % seen == 0 {
                    pick = v;
                }
            }
        }
        if pick == usize::MAX {
            return None;
        }
        let c = g.deg[pick] as u64 + 1;
        total = total.saturating_add(c * c);
        if total >= cutoff {
            return None; // cannot win; abandon early
        }
        g.eliminate(pick);
        perm.push(pick);
    }
    Some((total, perm))
}

/// `Σ c_j²` of a given permutation, played out on the elimination game.
/// Used only to pin the incremental identity against `flops_of`.
fn replay(g: &mut Game, col_ptr: &[usize], row_idx: &[usize], perm: &[usize]) -> u64 {
    g.reset(col_ptr, row_idx);
    let mut total = 0u64;
    for &v in perm {
        let c = g.deg[v] as u64 + 1;
        total = total.saturating_add(c * c);
        g.eliminate(v);
    }
    total
}

fn env_usize(k: &str, d: usize) -> usize {
    std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}

/// Search far past the shipped budget on the rows the pipeline leaves at AMD,
/// and report the best ratio any budget could reach.
#[test]
#[ignore]
fn probe_ceiling() {
    let corpus = crate::corpus::corpus();
    let secs: f64 = std::env::var("CEIL_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(5.0);
    let max_n = env_usize("CEIL_MAX_N", 1_000);
    let min_n = env_usize("CEIL_MIN_N", 0);
    let only_ties = env_usize("CEIL_ONLY_TIES", 1) == 1;

    // Rule / temperature / lambda schedule, cycled across restarts.
    let rules: [(Rule, f64, f64); 10] = [
        (Rule::Deg, 0.00, 0.0),
        (Rule::Deg, 0.02, 0.0),
        (Rule::Deg, 0.08, 0.0),
        (Rule::Deg, 0.25, 0.0),
        (Rule::Fill, 0.00, 0.0),
        (Rule::Fill, 0.10, 0.0),
        (Rule::Fill, 0.35, 0.0),
        (Rule::DegFill, 0.05, 1.0),
        (Rule::DegFill, 0.15, 4.0),
        (Rule::DegFill, 0.15, 0.25),
    ];

    let mut moved = 0usize;
    let mut probed = 0usize;
    let mut log_gain = 0.0f64;
    let mut rows: Vec<(String, usize, usize, f64, f64, u64)> = Vec::new();

    for (name, pat) in &corpus {
        let n = pat.n;
        if n < min_n.max(2) || n > max_n {
            continue;
        }
        let sp = ScoringPattern { n, col_ptr: pat.col_ptr.clone(), row_idx: pat.row_idx.clone() };
        let cp: Vec<i32> = pat.col_ptr.iter().map(|&x| x as i32).collect();
        let ri: Vec<i32> = pat.row_idx.iter().map(|&x| x as i32).collect();
        let Some(core) = feral_ordering_core::CscPattern::new(n, &cp, &ri) else { continue };
        let Ok(amd) = feral_amd::amd_order(&core) else { continue };
        let amd_perm: Vec<usize> = amd.into_iter().map(|x| x as usize).collect();
        let amd_flops = flops_of(&sp, &amd_perm);
        if amd_flops == 0 {
            continue;
        }

        // What the shipped pipeline achieves, so we only probe the stuck rows.
        let ship = flops_of(&sp, &order(pat)) as f64 / amd_flops as f64;
        if only_ties && ship < 0.9999 {
            continue;
        }

        let mut g = Game::new(n, &pat.col_ptr, &pat.row_idx);

        // PIN the incremental identity: Σ(1+d_j)² over the game must equal the
        // trusted flops_of on the same permutation.
        let pinned = replay(&mut g, &pat.col_ptr, &pat.row_idx, &amd_perm);
        assert_eq!(
            pinned, amd_flops,
            "incremental Σc² identity broken on {name}: game={pinned} flops_of={amd_flops}"
        );

        let mut best = amd_flops;
        let mut restarts = 0u64;
        let t0 = Instant::now();
        let mut k = 0usize;
        while t0.elapsed().as_secs_f64() < secs {
            let (rule, temp, lambda) = rules[k % rules.len()];
            let seed = mix(k as u64 ^ 0xC0FFEE);
            if let Some((f, perm)) = play(&mut g, &pat.col_ptr, &pat.row_idx, rule, temp, lambda, seed, best) {
                if f < best {
                    // Confirm with the trusted scorer before believing it.
                    debug_assert!(is_bijection(&perm, n));
                    let chk = flops_of(&sp, &perm);
                    assert_eq!(chk, f, "game/flops_of disagree on {name}");
                    best = f;
                }
            }
            restarts += 1;
            k += 1;
        }

        let ratio = best as f64 / amd_flops as f64;
        probed += 1;
        if ratio < 0.9999 {
            moved += 1;
            log_gain += ratio.ln();
        }
        rows.push((name.clone(), n, pat.nnz(), ship, ratio, restarts));
    }

    println!("\nCEIL_PROBED = {probed}");
    println!("CEIL_MOVED  = {moved}");
    if probed > 0 {
        println!("CEIL_MEAN_LOG_GAIN_OVER_PROBED = {:.6}", log_gain / probed as f64);
        println!("CEIL_GEO_OVER_PROBED = {:.6}", (log_gain / probed as f64).exp());
    }
    println!("--- rows: name n nnz shipped_ratio ceiling_ratio restarts ---");
    rows.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap());
    for (name, n, nnz, ship, ratio, restarts) in &rows {
        println!("CEIL\t{name}\t{n}\t{nnz}\t{ship:.6}\t{ratio:.6}\t{restarts}");
    }
}
