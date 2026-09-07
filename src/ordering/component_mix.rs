//! Exact terminal recombination across disconnected input components.
//!
//! Fill cannot connect separate input components. Thus each component's
//! contribution to sum(c_j^2) depends only on its own induced subsequence,
//! even if the full permutation interleaves components. Existing donors can
//! supply different component subsequences without any new random search.

use super::{is_bijection, scoring_ws::ScoreWorkspace, ScoringPattern};

const MAX_N: usize = 50_000;
const MAX_NNZ: usize = 200_000;
const WORK_LEDGER: usize = 2_000_000;
const MAX_DONORS: usize = 8;

fn components(sp: &ScoringPattern) -> (Vec<usize>, Vec<usize>) {
    let mut labels = vec![usize::MAX; sp.n];
    let mut sizes = Vec::new();
    let mut stack = Vec::new();
    for root in 0..sp.n {
        if labels[root] != usize::MAX {
            continue;
        }
        let component = sizes.len();
        labels[root] = component;
        stack.push(root);
        let mut size = 0;
        while let Some(v) = stack.pop() {
            size += 1;
            for &w in &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]] {
                if labels[w] == usize::MAX {
                    labels[w] = component;
                    stack.push(w);
                }
            }
        }
        sizes.push(size);
    }
    (labels, sizes)
}

fn component_costs(counts: &[i32], perm: &[usize], labels: &[usize], out: &mut [u64]) {
    out.fill(0);
    for (&count, &vertex) in counts.iter().zip(perm) {
        let c = count as u64;
        out[labels[vertex]] += c * c;
    }
}

pub(super) fn refine(
    sp: &ScoringPattern,
    incumbent: &[usize],
    donors: &[(u64, Vec<usize>)],
) -> Option<Vec<usize>> {
    refine_with_ledger(sp, incumbent, donors, WORK_LEDGER)
}

fn refine_with_ledger(
    sp: &ScoringPattern,
    incumbent: &[usize],
    donors: &[(u64, Vec<usize>)],
    ledger: usize,
) -> Option<Vec<usize>> {
    let n = sp.n;
    let nnz = sp.row_idx.len();
    if n < 6 || n > MAX_N || nnz > MAX_NNZ || donors.is_empty() {
        return None;
    }
    let unit = n + nnz;
    // Reserve one input scan plus the incumbent and final verification
    // scores before considering any donor. No factor materialization.
    let tickets = (ledger / unit).saturating_sub(3).min(MAX_DONORS);
    if tickets == 0 || !is_bijection(incumbent, n) {
        return None;
    }
    let (labels, sizes) = components(sp);
    // Components of one or two vertices have order-invariant cost. With
    // only one larger component the exact global winner cannot be improved
    // by recombining globally worse donors and invariant tiny components.
    if sizes.iter().filter(|&&size| size >= 3).count() < 2 {
        return None;
    }

    let mut ws = ScoreWorkspace::new(n, nnz);
    let incumbent_flops = ws.flops(sp, incumbent);
    let mut best_costs = vec![0; sizes.len()];
    component_costs(ws.column_counts(), incumbent, &labels, &mut best_costs);
    let mut costs = vec![0; sizes.len()];
    // 0 denotes the incumbent; donor index i is represented by i + 1.
    let mut selected = vec![0usize; sizes.len()];
    for (i, (_, donor)) in donors.iter().take(tickets).enumerate() {
        if !is_bijection(donor, n) {
            continue;
        }
        let _ = ws.flops(sp, donor);
        component_costs(ws.column_counts(), donor, &labels, &mut costs);
        for c in 0..sizes.len() {
            if costs[c] < best_costs[c] {
                best_costs[c] = costs[c];
                selected[c] = i + 1;
            }
        }
    }
    let expected: u64 = best_costs.iter().sum();
    if expected >= incumbent_flops {
        return None;
    }

    let mut offsets = Vec::with_capacity(sizes.len());
    let mut end = 0;
    for &size in &sizes {
        offsets.push(end);
        end += size;
    }
    let mut candidate = vec![usize::MAX; n];
    for (source, perm) in std::iter::once(incumbent)
        .chain(donors.iter().take(tickets).map(|(_, p)| p.as_slice()))
        .enumerate()
    {
        if !selected.contains(&source) {
            continue;
        }
        for &v in perm {
            let c = labels[v];
            if selected[c] == source {
                candidate[offsets[c]] = v;
                offsets[c] += 1;
            }
        }
    }
    if !is_bijection(&candidate, n) {
        return None;
    }
    let verified = ws.flops(sp, &candidate);
    if verified == expected && verified < incumbent_flops {
        Some(candidate)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pattern;

    fn scoring(p: &Pattern) -> ScoringPattern {
        ScoringPattern {
            n: p.n,
            col_ptr: p.col_ptr.clone(),
            row_idx: p.row_idx.clone(),
        }
    }

    #[test]
    fn equal_global_donors_can_supply_complementary_components() {
        let p = Pattern::from_edges(6, &[(0, 1), (1, 2), (3, 4), (4, 5)]);
        let sp = scoring(&p);
        let incumbent = vec![1, 3, 0, 5, 2, 4];
        let donor = vec![0, 4, 2, 3, 1, 5];
        assert_eq!(super::super::flops_of(&sp, &incumbent), 23);
        assert_eq!(super::super::flops_of(&sp, &donor), 23);
        let donors = vec![(23, donor)];
        let result = refine(&sp, &incumbent, &donors).unwrap();
        assert_eq!(super::super::flops_of(&sp, &result), 18);
        assert_eq!(Some(result), refine(&sp, &incumbent, &donors));
        assert!(refine_with_ledger(&sp, &incumbent, &donors, 3 * (p.n + p.nnz())).is_none());
        assert!(refine(&sp, &incumbent, &[(0, vec![0; 6])]).is_none());
    }

    #[test]
    fn exact_component_sum_for_all_six_vertex_orders() {
        let p = Pattern::from_edges(6, &[(0, 1), (1, 2), (3, 4), (4, 5)]);
        let sp = scoring(&p);
        let mut permutation: Vec<_> = (0..6).collect();
        loop {
            let mut donor = permutation.clone();
            donor.reverse();
            let baseline = super::super::flops_of(&sp, &permutation);
            let donors = vec![(super::super::flops_of(&sp, &donor), donor)];
            if let Some(result) = refine(&sp, &permutation, &donors) {
                assert!(is_bijection(&result, 6));
                assert!(super::super::flops_of(&sp, &result) < baseline);
            }
            let Some(i) = (0..5).rev().find(|&i| permutation[i] < permutation[i + 1]) else {
                break;
            };
            let j = (i + 1..6)
                .rev()
                .find(|&j| permutation[i] < permutation[j])
                .unwrap();
            permutation.swap(i, j);
            permutation[i + 1..].reverse();
        }
    }
}
