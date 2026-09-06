//! A bounded terminal PEO chain that can change its seed on exact score ties.
//! Equal-score PEOs preserve the completion, but can change deterministic MCS
//! traversal on the next extraction. Only a strict best result leaves this module.

use super::{column_counts_gnp, is_bijection, peo_extract, permute_pattern,
    EliminationTree, ScoringPattern};

const WORK_LIMIT: u64 = 2_000_000;
const MAX_ROUNDS: usize = 6;
const MAX_CONSECUTIVE_TIES: usize = 2;

pub(super) fn refine<F: Fn(&[usize]) -> u64>(
    pattern: &ScoringPattern, incumbent: &[usize], score: &F,
) -> Option<Vec<usize>> {
    if pattern.n < 16 {
        #[cfg(test)]
        LAST_SEARCH.with(|last| last.set(Some(SearchTelemetry::gate())));
        return None;
    }
    let out = search(pattern, incumbent, score, WORK_LIMIT, MAX_CONSECUTIVE_TIES);
    #[cfg(test)]
    LAST_SEARCH.with(|last| last.set(Some(SearchTelemetry {
        spent: out.spent, rounds: out.rounds, tie_steps: out.tie_steps,
        strict_steps: out.strict_steps, repeats_rejected: out.repeats_rejected,
        output_changed: out.candidate.is_some(), stop: out.stop,
    })));
    out.candidate
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StopReason { Gate, Factor, Budget, Stall, RoundCap }

#[cfg(test)]
impl StopReason {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Gate => "gate", Self::Factor => "factor", Self::Budget => "budget",
            Self::Stall => "stall", Self::RoundCap => "round_cap",
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SearchTelemetry {
    pub(super) spent: u64,
    pub(super) rounds: usize,
    pub(super) tie_steps: usize,
    pub(super) strict_steps: usize,
    pub(super) repeats_rejected: usize,
    pub(super) output_changed: bool,
    pub(super) stop: StopReason,
}

#[cfg(test)]
impl SearchTelemetry {
    pub(super) const fn gate() -> Self {
        Self { spent: 0, rounds: 0, tie_steps: 0, strict_steps: 0,
            repeats_rejected: 0, output_changed: false, stop: StopReason::Gate }
    }
}

#[cfg(test)]
std::thread_local! {
    static LAST_SEARCH: std::cell::Cell<Option<SearchTelemetry>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(super) fn clear_last_search() { LAST_SEARCH.with(|last| last.set(None)); }

#[cfg(test)]
pub(super) fn take_last_search() -> Option<SearchTelemetry> { LAST_SEARCH.with(|last| last.take()) }


// Telemetry also makes refusal and the strict-only matched control testable.
#[allow(dead_code)]
struct SearchResult {
    candidate: Option<Vec<usize>>,
    spent: u64,
    rounds: usize,
    tie_steps: usize,
    strict_steps: usize,
    repeats_rejected: usize,
    #[cfg(test)]
    stop: StopReason,
}

fn charge(spent: &mut u64, limit: u64, cost: u64) -> bool {
    let Some(next) = spent.checked_add(cost) else { return false; };
    if next > limit { return false; }
    *spent = next;
    true
}

fn search<F: Fn(&[usize]) -> u64>(
    pattern: &ScoringPattern, incumbent: &[usize], score: &F,
    limit: u64, max_ties: usize,
) -> SearchResult {
    let mut out = SearchResult { candidate: None, spent: 0, rounds: 0,
        tie_steps: 0, strict_steps: 0, repeats_rejected: 0,
        #[cfg(test)]
        stop: StopReason::RoundCap,
    };
    let n = pattern.n;
    let nnz = pattern.row_idx.len();
    if n == 0 || n > peo_extract::MAX_N || nnz > peo_extract::MAX_INPUT_NNZ {
        #[cfg(test)]
        { out.stop = StopReason::Gate; }
        return out;
    }
    // Pay for validation's n-entry zeroed scratch and n-entry input scan.
    if !charge(&mut out.spent, limit, 2 * n as u64) || !is_bijection(incumbent, n) {
        #[cfg(test)]
        { out.stop = if out.spent == 0 { StopReason::Budget } else { StopReason::Gate }; }
        return out;
    }
    // Separately pay for the two retained seed copies before allocating them.
    if !charge(&mut out.spent, limit, 2 * n as u64) {
        #[cfg(test)]
        { out.stop = StopReason::Budget; }
        return out;
    }
    let mut current = incumbent.to_vec();
    let mut visited = vec![incumbent.to_vec()];
    let mut best_flops = u64::MAX;
    let mut consecutive_ties = 0;
    let base_work = n as u64 + nnz as u64;
    for _ in 0..MAX_ROUNDS {
        // Split the existing measured full-round bill n+nnz+Lnnz so a refused
        // reconstruction still pays for its permutation/tree/count preparation.
        if !charge(&mut out.spent, limit, base_work) {
            #[cfg(test)]
            { out.stop = StopReason::Budget; }
            break;
        }
        let pp = permute_pattern(pattern, &current);
        let et = EliminationTree::from_pattern(&pp);
        let counts = column_counts_gnp(&pp, &et);
        let lnnz: u64 = counts.iter().map(|&c| c as u64).sum();
        let current_flops: u64 = counts.iter().map(|&c| (c as u64) * (c as u64)).sum();
        if out.rounds == 0 { best_flops = current_flops; }
        if lnnz > peo_extract::MAX_LNNZ as u64 {
            #[cfg(test)]
            { out.stop = StopReason::Factor; }
            break;
        }
        // The measured round bill includes reconstruction, both MCS traversals
        // and both exact candidate scores. Additionally pay for comparing each
        // candidate with all visited seeds and two possible retained copies.
        let history_work = (2 * visited.len() as u64 + 2) * n as u64;
        if !charge(&mut out.spent, limit, lnnz + history_work) {
            #[cfg(test)]
            { out.stop = StopReason::Budget; }
            break;
        }
        let Some(candidates) = peo_extract::candidates(
            n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &current,
        ) else {
            #[cfg(test)]
            { out.stop = StopReason::Factor; }
            break;
        };
        out.rounds += 1;
        let mut selected: Option<(u64, Vec<usize>)> = None;
        for candidate in candidates {
            let f = score(&candidate);
            if f > current_flops { continue; }
            if f == current_flops && consecutive_ties >= max_ties { continue; }
            if visited.iter().any(|prior| prior == &candidate) {
                out.repeats_rejected += 1;
                continue;
            }
            // A strict candidate always outranks a tied one. Equal candidates
            // keep forward-first source order, including on a plateau.
            if selected.as_ref().map_or(true, |(old_f, _)| f < *old_f) {
                selected = Some((f, candidate));
            }
        }
        let Some((f, next)) = selected else {
            #[cfg(test)]
            { out.stop = StopReason::Stall; }
            break;
        };
        if f < current_flops {
            consecutive_ties = 0;
            out.strict_steps += 1;
            if f < best_flops {
                best_flops = f;
                out.candidate = Some(next.clone());
            }
        } else {
            consecutive_ties += 1;
            out.tie_steps += 1;
        }
        current = next;
        visited.push(current.clone());
        // This round's reconstruction and symbolic scratch end here. Only the
        // seed history and strict best permutation survive into the next round.
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::flops_of;
    use std::cell::Cell;

    fn graph(n: usize, edges: &[(usize, usize)]) -> ScoringPattern {
        let mut adj = vec![vec![false; n]; n];
        for &(u, v) in edges { adj[u][v] = true; adj[v][u] = true; }
        let mut col_ptr = vec![0];
        let mut row_idx = Vec::new();
        for row in adj {
            row_idx.extend(row.iter().enumerate().filter_map(|(u, &edge)| edge.then_some(u)));
            col_ptr.push(row_idx.len());
        }
        ScoringPattern { n, col_ptr, row_idx }
    }

    fn tied_fixture() -> (ScoringPattern, Vec<usize>) {
        (graph(6, &[(0,3),(0,4),(1,2),(1,3),(1,4),(2,3),(2,5)]),
            vec![5,3,2,4,0,1])
    }

    #[test]
    fn tied_peo_seed_opens_a_strict_next_extraction() {
        let (p, seed) = tied_fixture();
        let score = |q: &[usize]| flops_of(&p, q);
        assert_eq!(score(&seed), 43);
        let pp = permute_pattern(&p, &seed);
        let et = EliminationTree::from_pattern(&pp);
        let counts = column_counts_gnp(&pp, &et);
        let first = peo_extract::candidates(6, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &seed).unwrap();
        assert_eq!(first, [vec![4,3,1,0,2,5], vec![4,1,0,3,2,5]]);
        assert!(first.iter().all(|q| score(q) == 43));
        let strict = search(&p, &seed, &score, WORK_LIMIT, 0);
        assert!(strict.candidate.is_none());
        assert_eq!(strict.rounds, 1);
        assert_eq!(strict.stop, StopReason::Stall);
        let tied = search(&p, &seed, &score, WORK_LIMIT, MAX_CONSECUTIVE_TIES);
        let q = tied.candidate.unwrap();
        assert_eq!(score(&q), 36);
        assert!(is_bijection(&q, p.n));
        assert!(tied.tie_steps >= 1 && tied.strict_steps >= 1);
        assert!(tied.rounds <= MAX_ROUNDS && tied.spent <= WORK_LIMIT);
        assert_eq!(search(&p, &seed, &score, WORK_LIMIT, MAX_CONSECUTIVE_TIES).candidate, Some(q));
        // This six-vertex witness establishes kernel reach, not an end-to-end
        // portfolio improvement; the production lower dimension gate rejects it.
        assert!(refine(&p, &seed, &score).is_none());
    }

    #[test]
    fn tied_peo_budget_refusal_and_strict_result_preservation() {
        let (p, seed) = tied_fixture();
        let calls = Cell::new(0);
        let score = |q: &[usize]| { calls.set(calls.get() + 1); flops_of(&p, q) };
        // 12 validation + 12 seed copies + 20 preparation + 15 Lnnz + 24 history = 83.
        for limit in [0, 11, 12, 23, 24, 43, 44, 82] {
            calls.set(0);
            let result = search(&p, &seed, &score, limit, MAX_CONSECUTIVE_TIES);
            assert!(result.candidate.is_none());
            assert!(result.spent <= limit);
            assert_eq!(result.stop, StopReason::Budget);
            assert_eq!(calls.get(), 0);
        }
        calls.set(0);
        let first_only = search(&p, &seed, &score, 83, MAX_CONSECUTIVE_TIES);
        assert_eq!(first_only.rounds, 1);
        assert_eq!(first_only.tie_steps, 1);
        assert!(first_only.candidate.is_none());
        assert_eq!(calls.get(), 2);
        // The second tied-seed round needs 20 + 15 + 36 = 71 more units.
        let two = search(&p, &seed, &score, 154, MAX_CONSECUTIVE_TIES);
        assert_eq!(two.rounds, 2);
        assert_eq!(two.spent, 154);
        assert_eq!(two.stop, StopReason::Budget);
        assert_eq!(flops_of(&p, &two.candidate.unwrap()), 36);
    }

    #[test]
    fn tied_peo_rejects_cycles_and_never_returns_a_tied_seed() {
        let p = graph(2, &[(0, 1)]);
        let seed = vec![0, 1];
        let result = search(&p, &seed, &|q| flops_of(&p, q), WORK_LIMIT, MAX_CONSECUTIVE_TIES);
        assert!(result.candidate.is_none());
        assert_eq!(result.tie_steps, 1);
        assert_eq!(result.rounds, 2);
        assert_eq!(result.repeats_rejected, 2);
        assert_eq!(result.stop, StopReason::Stall);
    }

    #[test]
    fn tied_peo_small_graphs_are_deterministic_bijections_and_nonincreasing() {
        for n in 1..=4 {
            let edges: Vec<_> = (0..n).flat_map(|u| (u+1..n).map(move |v| (u,v))).collect();
            for mask in 0..(1usize << edges.len()) {
                let active: Vec<_> = edges.iter().enumerate().filter_map(|(i, &e)|
                    (mask & (1 << i) != 0).then_some(e)).collect();
                let p = graph(n, &active);
                for seed in [(0..n).collect::<Vec<_>>(), (0..n).rev().collect::<Vec<_>>()] {
                    let score = |q: &[usize]| flops_of(&p, q);
                    let first = search(&p, &seed, &score, WORK_LIMIT, MAX_CONSECUTIVE_TIES);
                    let again = search(&p, &seed, &score, WORK_LIMIT, MAX_CONSECUTIVE_TIES);
                    assert_eq!(first.candidate, again.candidate);
                    assert!(first.rounds <= MAX_ROUNDS && first.spent <= WORK_LIMIT);
                    if let Some(q) = first.candidate {
                        assert!(is_bijection(&q, n));
                        assert!(score(&q) < score(&seed));
                    }
                }
            }
        }
    }
    #[test]
    fn tied_peo_last_search_telemetry_is_consumed_and_does_not_steer() {
        let p = graph(16, &[]);
        let seed: Vec<_> = (0..p.n).collect();
        let score = |q: &[usize]| flops_of(&p, q);
        clear_last_search();
        assert!(take_last_search().is_none());
        let first = refine(&p, &seed, &score);
        let snapshot = take_last_search().unwrap();
        assert_eq!(snapshot.output_changed, first.is_some());
        assert_eq!(snapshot.stop, StopReason::Stall);
        assert!(snapshot.rounds > 0 && snapshot.spent > 0);
        assert!(take_last_search().is_none());
        assert_eq!(refine(&p, &seed, &score), first);
        assert_eq!(take_last_search(), Some(snapshot));
        let (small, small_seed) = tied_fixture();
        assert!(refine(&small, &small_seed, &|q| flops_of(&small, q)).is_none());
        assert_eq!(take_last_search(), Some(SearchTelemetry::gate()));
        assert!(take_last_search().is_none());
    }

}
