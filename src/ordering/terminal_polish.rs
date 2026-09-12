//! Bounded refinement after the promoted final windows.
use super::{Pattern, ScoringPattern, EliminationTree, column_counts_gnp,
    is_bijection, peo_extract, completion, rgreedy};

#[cfg(test)]
thread_local! { static ENABLED: std::cell::Cell<bool> = std::cell::Cell::new(true); }
#[cfg(test)]
pub(super) fn set_enabled(enabled: bool) { ENABLED.with(|c| c.set(enabled)); }

#[derive(Clone, Copy)]
enum Pass { Peo(usize), Watch(i64), Window(usize, usize, usize, i64), Limit(u64) }

pub(super) fn refine<S, P, C>(pattern: &Pattern, incumbent: &[usize],
    score: S, permute: P, factor_nnz: C) -> Option<Vec<usize>>
where S: Fn(&[usize]) -> u64, P: Fn(&[usize]) -> ScoringPattern, C: Fn() -> u64 {
    #[cfg(test)]
    if !ENABLED.with(|c| c.get()) { return None; }
    let n = pattern.n;
    if n < 6 || n > 12_000 || pattern.nnz() > 200_000
        || !is_bijection(incumbent, n) { return None; }
    let reference = score(incumbent);
    if reference > 20_000_000_000 || factor_nnz() > 150_000 { return None; }
    let mut current = incumbent.to_vec();
    let mut flops = reference;
    let mut limit = 150_000;
    for pass in [
        Pass::Peo(2), Pass::Limit(75_000), Pass::Watch(1_000_000),
        Pass::Window(32, 4, 13, 16_000_000),
        Pass::Window(64, 4, 27, 16_000_000),
        Pass::Window(10, 4, 3, 16_000_000),
        Pass::Window(7, 4, 2, 16_000_000), Pass::Peo(2),
    ] {
        if let Pass::Limit(next) = pass { limit = next; continue; }
        // Refresh the scoring arena before reading its factor count; a rejected
        // candidate may have been the last pattern scored in the previous pass.
        let exact = score(&current);
        if exact > 20_000_000_000 || factor_nnz() > limit { continue; }
        flops = exact;
        match pass {
            Pass::Peo(rounds) => for _ in 0..rounds {
                let before = flops;
                let pp = permute(&current);
                let et = EliminationTree::from_pattern(&pp);
                let counts = column_counts_gnp(&pp, &et);
                if let Some(candidates) = peo_extract::candidates_bounded(n,
                    &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &current,
                    12_000, 200_000, limit as usize) {
                    for candidate in candidates {
                        if is_bijection(&candidate, n) {
                            let f = score(&candidate);
                            if f < flops { current = candidate; flops = f; }
                        }
                    }
                }
                if flops == before { break; }
            },
            Pass::Watch(budget) => {
                let pp = permute(&current);
                let et = EliminationTree::from_pattern(&pp);
                let counts = column_counts_gnp(&pp, &et);
                if let Some(candidate) = completion::refine_limited(n,
                    &pattern.col_ptr, &pattern.row_idx, &pp.col_ptr, &pp.row_idx,
                    &et.parent, &counts, &current, budget) {
                    if is_bijection(&candidate, n) {
                        let f = score(&candidate);
                        if f < flops { current = candidate; flops = f; }
                    }
                }
            },
            Pass::Window(width, sweeps, stride, budget) => {
                if let Some(candidate) = rgreedy::sparse_span_window_descent(n,
                    &pattern.col_ptr, &pattern.row_idx, &current,
                    width, sweeps, stride, budget) {
                    if is_bijection(&candidate, n) {
                        let f = score(&candidate);
                        if f < flops { current = candidate; flops = f; }
                    }
                }
            },
            Pass::Limit(_) => unreachable!(),
        }
    }
    if flops < reference { Some(current) } else { None }
}
