//! Five-pivot cleanup and exact three-free-pivot interval interleaving.
//! Each invocation owns fresh preparation, an independent allowance and validation.

use super::{FiveWindow, Game, TripleWork};

const MAX_N: usize = 4096;
const MAX_NNZ: usize = 65_536;
const SPAN: usize = 32;
const MAX_BOUNDARY: usize = 512;
const MAX_TILES: usize = 16;
const STATES: usize = 33 * 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Refusal {
    #[default]
    None,
    Gate,
    Input,
    Setup,
    Baseline,
    Reservation,
    Replay,
    Discovery,
    Boundary,
    Snapshot,
    Kernel,
    Invariant,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Stats {
    pub spent: i64,
    pub reserved: i64,
    pub reserve_unused: i64,
    pub windows: u32,
    pub eligible: u32,
    pub solved: u32,
    pub wins: u32,
    pub transitions: u32,
    pub refusal: Refusal,
}

#[cfg(test)]
std::thread_local! {
    static LAST_STATS: std::cell::RefCell<Option<Stats>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn take_last_stats() -> Option<Stats> {
    LAST_STATS.with(|slot| slot.borrow_mut().take())
}

pub(crate) struct Improvement {
    pub order: Vec<usize>,
    pub before: u64,
    pub after: u64,
}

struct Snapshot {
    m: usize,
    internal: [u32; SPAN],
    boundary: [[u64; MAX_BOUNDARY / 64]; SPAN],
    boundary_words: usize,
}

struct Solution {
    order: [usize; SPAN],
    before: u64,
    after: u64,
}

impl Snapshot {
    #[cfg(test)]
    fn new(game: &Game<'_>, window: &[usize], work: &mut TripleWork) -> Option<Self> {
        Self::new_checked(game, window, work).ok()
    }

    fn new_checked(
        game: &Game<'_>, window: &[usize], work: &mut TripleWork,
    ) -> Result<Self, Refusal> {
        let m = window.len();
        if m == 0 || m > SPAN || game.n > MAX_N {
            return Err(Refusal::Input);
        }
        // Pay before local-edge tests, full-row unions, membership clearing,
        // boundary cardinality inspection and all fixed-array initialization.
        if !work.charge(2048 + 4 * m * m + 4 * m * game.w + 8 * game.w + 8 * m) {
            return Err(Refusal::Snapshot);
        }
        let mut internal = [0u32; SPAN];
        let mut exterior = [0u64; MAX_N / 64];
        for (i, &u) in window.iter().enumerate() {
            if u >= game.n || window[..i].contains(&u) {
                return Err(Refusal::Input);
            }
            let row = &game.adj[u * game.w..(u + 1) * game.w];
            for (j, &v) in window.iter().enumerate() {
                if v >= game.n { return Err(Refusal::Input); }
                if row[v / 64] & (1u64 << (v % 64)) != 0 {
                    internal[i] |= 1u32 << j;
                }
            }
            for (out, &word) in exterior[..game.w].iter_mut().zip(row) {
                *out |= word;
            }
        }
        for &v in window { exterior[v / 64] &= !(1u64 << (v % 64)); }
        let boundary_n: usize = exterior[..game.w].iter().map(|x| x.count_ones() as usize).sum();
        if boundary_n > MAX_BOUNDARY { return Err(Refusal::Boundary); }
        // Compress only after the identity-preserving boundary union passes.
        if !work.charge(1024 + 8 * game.w + 8 * boundary_n + 4 * m * boundary_n) {
            return Err(Refusal::Snapshot);
        }
        let mut labels = [0usize; MAX_BOUNDARY];
        let mut used = 0;
        for (word_i, &value) in exterior[..game.w].iter().enumerate() {
            let mut word = value;
            while word != 0 {
                labels[used] = word_i * 64 + word.trailing_zeros() as usize;
                used += 1;
                word &= word - 1;
            }
        }
        let mut boundary = [[0u64; MAX_BOUNDARY / 64]; SPAN];
        for (i, &u) in window.iter().enumerate() {
            let row = &game.adj[u * game.w..(u + 1) * game.w];
            for (j, &v) in labels[..used].iter().enumerate() {
                if row[v / 64] & (1u64 << (v % 64)) != 0 {
                    boundary[i][j / 64] |= 1u64 << (j % 64);
                }
            }
        }
        Ok(Self { m, internal, boundary, boundary_words: boundary_n.div_ceil(64) })
    }

    // Only eliminated interval vertices may connect to the pivot's component.
    // Outside vertices remain live and contribute union width, never paths.
    fn width(&self, eliminated: u32, pivot: usize) -> u64 {
        let mut component = 1u32 << pivot;
        let mut pending = component;
        let mut inside = 0u32;
        let mut outside = [0u64; MAX_BOUNDARY / 64];
        while pending != 0 {
            let v = pending.trailing_zeros() as usize;
            pending &= pending - 1;
            inside |= self.internal[v];
            let next = self.internal[v] & eliminated & !component;
            component |= next;
            pending |= next;
            for (out, &word) in outside[..self.boundary_words].iter_mut()
                .zip(&self.boundary[v][..self.boundary_words]) {
                *out |= word;
            }
        }
        1 + (inside & !component).count_ones() as u64
            + outside[..self.boundary_words].iter().map(|x| x.count_ones() as u64).sum::<u64>()
    }

    fn solve(&self, selected: &[usize], work: &mut TripleWork) -> Option<Solution> {
        let r = selected.len();
        if r == 0 || r > 3 || r > self.m { return None; }
        for (i, &v) in selected.iter().enumerate() {
            if v >= self.m || selected[..i].contains(&v) { return None; }
        }
        let t = self.m - r;
        let masks = 1usize << r;
        let states = (t + 1) * masks;
        let transitions = masks * t + r * (masks / 2) * (t + 1);
        // A width visit processes each of at most m component vertices once.
        // Cover bit work, row ORs, initialization and popcounts conservatively;
        // include before/candidate width replays, DP predecessors and copying.
        let width_work = 32 + self.m * (20 + 4 * self.boundary_words) + 4 * self.boundary_words;
        let charge = (transitions + 2 * self.m) * width_work
            + 32 * transitions + 64 * states + 32 * self.m + 2048;
        if !work.charge(charge) { return None; }
        let mut chain = [0usize; SPAN];
        let mut chain_n = 0;
        for v in 0..self.m {
            if !selected.contains(&v) { chain[chain_n] = v; chain_n += 1; }
        }
        let mut prefix = [0u32; SPAN + 1];
        for j in 0..t { prefix[j + 1] = prefix[j] | (1u32 << chain[j]); }
        let mut selected_set = [0u32; 8];
        for mask in 1usize..masks {
            let bit = mask.trailing_zeros() as usize;
            selected_set[mask] = selected_set[mask & (mask - 1)] | (1u32 << selected[bit]);
        }
        let mut cost = [u64::MAX; STATES];
        let mut prev = [u16::MAX; STATES];
        let mut pivot = [u8::MAX; STATES];
        cost[0] = 0;
        for j in 0..=t {
            for mask in 0..masks {
                let state = j * masks + mask;
                let eliminated = prefix[j] | selected_set[mask];
                // Fixed ordering plus strict replacement gives deterministic
                // improving ties; an incumbent objective tie is restored below.
                for choice in 0..=r {
                    let (v, next) = if choice == 0 {
                        if j == t { continue; }
                        (chain[j], (j + 1) * masks + mask)
                    } else {
                        let k = choice - 1;
                        if mask & (1 << k) != 0 { continue; }
                        (selected[k], j * masks + (mask | (1 << k)))
                    };
                    let c = self.width(eliminated, v);
                    let trial = cost[state].checked_add(c.checked_mul(c)?)?;
                    if trial < cost[next] {
                        cost[next] = trial;
                        prev[next] = state as u16;
                        pivot[next] = v as u8;
                    }
                }
            }
        }
        let mut before = 0u64;
        let mut eliminated = 0u32;
        let mut order = [0usize; SPAN];
        for v in 0..self.m {
            let c = self.width(eliminated, v);
            before = before.checked_add(c.checked_mul(c)?)?;
            eliminated |= 1u32 << v;
            order[v] = v;
        }
        let after = cost[states - 1];
        if after < before {
            let mut state = states - 1;
            for i in (0..self.m).rev() {
                order[i] = pivot[state] as usize;
                state = prev[state] as usize;
            }
            if state != 0 { return None; }
        }
        // Independent traversal of the reconstructed path catches predecessor
        // mistakes before any caller commits it (not an independent oracle).
        let mut check = 0u64;
        let mut eliminated = 0u32;
        for &v in &order[..self.m] {
            if v >= self.m || eliminated & (1u32 << v) != 0 { return None; }
            let c = self.width(eliminated, v);
            check = check.checked_add(c.checked_mul(c)?)?;
            eliminated |= 1u32 << v;
        }
        if check != after || after > before { return None; }
        Some(Solution { order, before, after })
    }
}

// Discovery control: free costly current pivots anywhere in the interval.
// The caller pays the same fixed discovery ticket before this bounded scan.
// No adjacency, distance, lower-degree or homogeneous-degree filter applies.
fn select_free_vertices(game: &Game<'_>, window: &[usize]) -> Option<[usize; 3]> {
    use std::cmp::Reverse;
    let mut choice = [(Reverse(0u32), usize::MAX, usize::MAX); 3];
    let mut choices = 0usize;
    for (local, &v) in window.iter().enumerate() {
        let key = (Reverse(game.deg[v]), local, v);
        let at = (0..choices).find(|&i| key < choice[i]).unwrap_or(choices);
        if at < 3 {
            for i in (at + 1..(choices + 1).min(3)).rev() { choice[i] = choice[i - 1]; }
            choice[at] = key;
            choices = (choices + 1).min(3);
        }
    }
    if choices != 3 { return None; }
    let mut selected = choice.map(|(_, local, _)| local);
    selected.sort_unstable();
    Some(selected)
}

fn reset_work(n: usize, words: usize) -> usize { 2 * n * words + 8 * n }

// Same one-cycle offset/stride traversal as adjacent_five_descent, using the
// outer wrapper's already prepaid Game/draft. The optimized window kernel and
// every reset/replay tariff are unchanged; no second graph setup is hidden.
fn five_cycle(
    game: &mut Game<'_>, draft: &mut [usize], work: &mut TripleWork, stats: &mut Stats,
) -> u64 {
    let n = draft.len();
    let mut gain = 0u64;
    for offset in 0..5 {
        if offset + 5 > n { break; }
        if !work.charge(reset_work(n, game.w)) {
            stats.refusal = Refusal::Replay; return gain;
        }
        game.reset();
        for &v in draft.iter().take(offset) {
            if !work.eliminate(game, v) { stats.refusal = Refusal::Replay; return gain; }
        }
        let mut k = offset;
        while k + 4 < n {
            stats.windows += 1;
            stats.eligible += 1;
            let window = [draft[k], draft[k + 1], draft[k + 2], draft[k + 3], draft[k + 4]];
            let Some(kernel) = FiveWindow::new(game, window, work) else {
                stats.refusal = Refusal::Kernel; return gain;
            };
            let (order, after, before) = kernel.solve();
            stats.solved += 1;
            stats.transitions += 80;
            if after < before {
                draft[k..k + 5].copy_from_slice(&order.map(|i| window[i]));
                gain += before - after;
                stats.wins += 1;
            }
            k += 5;
            if k + 4 < n {
                for &v in &draft[k - 5..k] {
                    if !work.eliminate(game, v) { stats.refusal = Refusal::Replay; return gain; }
                }
            }
        }
    }
    gain
}

fn floor_sqrt(value: u64) -> u64 {
    if value == 0 { return 0; }
    let (mut lo, mut hi) = (1u64, 1u64 << 32);
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        if mid <= value / mid { lo = mid; } else { hi = mid; }
    }
    lo
}

/// A complete independent terminal allowance: setup, fresh baseline replay,
/// bounded discovery/DP and reserved full candidate replay all pay here.
/// Refusals retain only completed edits whose fresh final score validates.
pub(crate) fn refine(
    n: usize, col_ptr: &[usize], row_idx: &[usize], seed: &[usize],
    budget: i64, stats: &mut Stats,
) -> Option<Improvement> {
    refine_with_search(n, col_ptr, row_idx, seed, budget, stats, Search::Interval)
}

/// One existing five-offset cycle with the same fresh/final scoring wrapper.
/// A separate call owns a separate allowance; scratch and scores are not shared
/// with a preceding or following interval invocation.
pub(crate) fn refine_five(
    n: usize, col_ptr: &[usize], row_idx: &[usize], seed: &[usize],
    budget: i64, stats: &mut Stats,
) -> Option<Improvement> {
    refine_with_search(n, col_ptr, row_idx, seed, budget, stats, Search::Five)
}

#[derive(Clone, Copy)]
enum Search { Interval, Five }

fn refine_with_search(
    n: usize, col_ptr: &[usize], row_idx: &[usize], seed: &[usize],
    budget: i64, stats: &mut Stats, search: Search,
) -> Option<Improvement> {
    *stats = Stats::default();
    let mut work = TripleWork { remaining: budget.max(0) };
    let mut reserved_left = 0i64;
    let result = (|| {
        if !(16..=MAX_N).contains(&n) || row_idx.is_empty() || row_idx.len() > MAX_NNZ || budget <= 0 {
            stats.refusal = Refusal::Gate; return None;
        }
        if !work.charge(n + 1 + row_idx.len() + 2 * n) {
            stats.refusal = Refusal::Setup; return None;
        }
        if seed.len() != n || col_ptr.len() != n + 1 || col_ptr[0] != 0
            || col_ptr[n] != row_idx.len()
            || col_ptr.windows(2).any(|p| p[0] > p[1] || p[1] > row_idx.len())
            || row_idx.iter().any(|&v| v >= n) {
            stats.refusal = Refusal::Input; return None;
        }
        let mut seen = vec![false; n];
        for &v in seed {
            if v >= n || seen[v] { stats.refusal = Refusal::Input; return None; }
            seen[v] = true;
        }
        let words = n.div_ceil(64);
        if !work.charge(n * words + 2 * row_idx.len() + n) {
            stats.refusal = Refusal::Setup; return None;
        }
        let adj = Game::build_adj(n, col_ptr, row_idx)?;
        if !work.charge(2 * n * words + 17 * n + words + 1024) {
            stats.refusal = Refusal::Setup; return None;
        }
        let mut game = Game::new(n, &adj)?;
        let mut counts = vec![0u32; n];
        let mut draft = seed.to_vec();
        if !work.charge(reset_work(n, words) + 8 * n) {
            stats.refusal = Refusal::Baseline; return None;
        }
        game.reset();
        let mut before = 0u64;
        for (i, &v) in seed.iter().enumerate() {
            let c = game.deg[v] as u64 + 1;
            if !work.eliminate(&mut game, v) {
                stats.refusal = Refusal::Baseline; return None;
            }
            counts[i] = c as u32;
            before = before.checked_add(c.checked_mul(c)?)?;
        }
        if !work.charge(256) { stats.refusal = Refusal::Reservation; return None; }
        // Every committed interval lowers F. Cauchy therefore reserves enough
        // pivot work for ANY final draft, without a fresh max-degree gate.
        let width_sum_bound = floor_sqrt((n as u64).checked_mul(before)?);
        let reserve = (reset_work(n, words) as u64).checked_add((10 * n + 64) as u64)?
            .checked_add(width_sum_bound.checked_mul((3 * words + 6) as u64)?)?
            .checked_add((24 * n) as u64)?;
        let reserve = usize::try_from(reserve).ok()?;
        if !work.charge(reserve) { stats.refusal = Refusal::Reservation; return None; }
        reserved_left = reserve as i64;
        stats.reserved = reserved_left;

        let predicted = match search {
            Search::Interval => {
                // Rank a fixed number of disjoint tiles by freshly measured width mass.
                // All ranking, fixed-array sorting and per-tile score checks are paid.
                if !work.charge(8 * n + 128 * n.div_ceil(SPAN) + 8192) {
                    stats.refusal = Refusal::Discovery; return None;
                }
                let mut tiles = [(0u64, usize::MAX); MAX_TILES];
                let mut tile_n = 0;
                for start in (0..n).step_by(SPAN) {
                    let end = (start + SPAN).min(n);
                    if end - start < 8 { continue; }
                    let mass = counts[start..end].iter().map(|&c| (c as u64) * (c as u64)).sum();
                    let item = (mass, start);
                    let mut at = tile_n.min(MAX_TILES);
                    for i in 0..tile_n {
                        if mass > tiles[i].0 || (mass == tiles[i].0 && start < tiles[i].1) { at = i; break; }
                    }
                    if at < MAX_TILES {
                        for i in (at + 1..(tile_n + 1).min(MAX_TILES)).rev() { tiles[i] = tiles[i - 1]; }
                        tiles[at] = item;
                        tile_n = (tile_n + 1).min(MAX_TILES);
                    }
                }
                tiles[..tile_n].sort_unstable_by_key(|x| x.1);
                let mut predicted = before;
                if !work.charge(reset_work(n, words)) { stats.refusal = Refusal::Replay; return None; }
                game.reset();
                let mut position = 0;
                'tiles: for &(mass, start) in &tiles[..tile_n] {
                    while position < start {
                        if !work.eliminate(&mut game, seed[position]) { stats.refusal = Refusal::Replay; break 'tiles; }
                        position += 1;
                    }
                    stats.windows += 1;
                    let end = (start + SPAN).min(n);
                    let window = &seed[start..end];
                    if !work.charge(128 + 24 * window.len()) { stats.refusal = Refusal::Discovery; break; }
                    let Some(selected) = select_free_vertices(&game, window) else { continue; };
                    stats.eligible += 1;
                    let snapshot = match Snapshot::new_checked(&game, window, &mut work) {
                        Ok(snapshot) => snapshot,
                        Err(reason) => { stats.refusal = reason; break; }
                    };
                    let Some(solution) = snapshot.solve(&selected, &mut work) else {
                        stats.refusal = Refusal::Kernel; break;
                    };
                    stats.solved += 1;
                    stats.transitions += (8 * (window.len() - 3) + 12 * (window.len() - 2)) as u32;
                    if solution.before != mass { stats.refusal = Refusal::Invariant; return None; }
                    if solution.after < solution.before {
                        for i in 0..window.len() { draft[start + i] = window[solution.order[i]]; }
                        predicted -= solution.before - solution.after;
                        stats.wins += 1;
                    }
                    // Replay follows the ORIGINAL seed. A completed interval removes
                    // the same set, so its endpoint is identical after any accepted DP.
                }
                predicted
            }
            Search::Five => {
                let gain = five_cycle(&mut game, &mut draft, &mut work, stats);
                let Some(predicted) = before.checked_sub(gain) else {
                    stats.refusal = Refusal::Invariant; return None;
                };
                predicted
            }
        };
        if stats.wins == 0 { return None; }
        let mut final_work = TripleWork { remaining: reserved_left };
        let final_result = (|| {
            if !final_work.charge(reset_work(n, words) + 10 * n + 64) { return None; }
            game.reset();
            seen.fill(false);
            let mut after = 0u64;
            for &v in &draft {
                if v >= n || seen[v] { return None; }
                seen[v] = true;
                let c = game.deg[v] as u64 + 1;
                if !final_work.eliminate(&mut game, v) { return None; }
                after = after.checked_add(c.checked_mul(c)?)?;
            }
            Some(after)
        })();
        reserved_left = final_work.remaining;
        let Some(after) = final_result else { stats.refusal = Refusal::Invariant; return None; };
        if after != predicted || after >= before { stats.refusal = Refusal::Invariant; return None; }
        Some(Improvement { order: draft, before, after })
    })();
    stats.reserve_unused = reserved_left;
    stats.spent = budget.max(0) - work.remaining - reserved_left;
    #[cfg(test)]
    LAST_STATS.with(|slot| *slot.borrow_mut() = Some(stats.clone()));
    result
}

#[cfg(test)]
mod tests;
