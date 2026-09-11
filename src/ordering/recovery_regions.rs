//! Sparse whole-subtree replacement: only selected interiors and their actual
//! exterior neighborhoods are materialized, even for very large input graphs.
use crate::Pattern;

#[derive(Clone, Copy)]
enum Mode {
    Legacy,
    Recovered,
    #[cfg(test)]
    Global,
}

pub(super) fn refine_legacy(pattern: &Pattern, p: &[usize]) -> Option<Vec<usize>> {
    refine_impl(pattern, p, &[], 16, Mode::Legacy)
}
pub(super) fn crossover_legacy(pattern: &Pattern, p: &[usize], donor: &[usize]) -> Option<Vec<usize>> {
    if p == donor { return None; }
    refine_impl(pattern, p, &[donor], 16, Mode::Legacy)
}
pub(super) fn refine(pattern: &Pattern, p: &[usize]) -> Option<Vec<usize>> {
    refine_impl(pattern, p, &[], 16, Mode::Recovered)
}
#[cfg(test)]
pub(super) fn refine_global(pattern: &Pattern, p: &[usize]) -> Option<Vec<usize>> {
    if pattern.nnz() > pattern.n.saturating_mul(16) {
        return None;
    }
    refine_impl(pattern, p, &[], 16, Mode::Global)
}
#[cfg(test)]
pub(super) fn crossover(pattern: &Pattern, p: &[usize], donor: &[usize]) -> Option<Vec<usize>> {
    if p == donor {
        return None;
    }
    refine_impl(pattern, p, &[donor], 16, Mode::Recovered)
}
/// Region crossover with several donors at once: every selected region takes
/// the cheapest donor-induced interior order, combined by the exact tree DP.
#[cfg(test)]
pub(super) fn crossover_many(
    pattern: &Pattern,
    p: &[usize],
    donors: &[&[usize]],
    regions: usize,
) -> Option<Vec<usize>> {
    let donors: Vec<&[usize]> = donors.iter().copied().filter(|d| *d != p).collect();
    if donors.is_empty() {
        return None;
    }
    refine_impl(pattern, p, &donors, regions, Mode::Recovered)
}
fn refine_impl(
    pattern: &Pattern,
    p: &[usize],
    donors: &[&[usize]],
    regions: usize,
    mode: Mode,
) -> Option<Vec<usize>> {
    let n = pattern.n;
    let legacy = matches!(mode, Mode::Legacy);
    let global = !matches!(mode, Mode::Legacy | Mode::Recovered);
    let min_n = if legacy { 1000 } else { 64 };
    let max_count = if legacy { 257 } else { 1025 };
    if !(min_n..=350_000).contains(&n)
        || pattern.nnz() > 2_000_000
    {
        return None;
    }
    let sp = super::ScoringPattern {
        n,
        col_ptr: pattern.col_ptr.clone(),
        row_idx: pattern.row_idx.clone(),
    };
    let (order, counts, parent) = super::etree_prep(&sp, p);
    let donor = donors.first().copied();
    let mut donor_ranks: Vec<Vec<usize>> = Vec::new();
    for d in donors {
        if !super::is_bijection(d, n) {
            return None;
        }
        let mut r = vec![0; n];
        for (j, &v) in d.iter().enumerate() {
            r[v] = j;
        }
        donor_ranks.push(r);
    }
    let mut rank = vec![0; n];
    for (j, &v) in order.iter().enumerate() {
        rank[v] = j;
    }
    let mut size = vec![1usize; n];
    let mut old = vec![0u64; n];
    let mut potential = vec![0u64; if global { n } else { 0 }];
    let mut children = vec![Vec::new(); n];
    for v in 0..n {
        old[v] += (counts[v] as u64).pow(2);
        if global {
            let original = order[v];
            let width = 1 + pattern.row_idx
                [pattern.col_ptr[original]..pattern.col_ptr[original + 1]]
                .iter()
                .filter(|&&u| rank[u] > v)
                .count() as u64;
            potential[v] += (counts[v] as u64).pow(2) - width * width;
        }
        if parent[v] >= 0 {
            let q = parent[v] as usize;
            size[q] += size[v];
            old[q] += old[v];
            if global {
                potential[q] += potential[v];
            }
            children[q].push(v);
        }
    }
    let mut seen = vec![false; n];
    let mut choices = Vec::new();
    for cap in if global {
        [64usize, 128, 256]
    } else {
        [128, 512, 1024]
    } {
        for v in 0..n {
            if !seen[v]
                && (!global || potential[v] > 0)
                && size[v] >= 32
                && size[v] <= cap
                && counts[v] <= max_count
                && (parent[v] < 0 || size[parent[v] as usize] > cap)
            {
                seen[v] = true;
                choices.push(v);
            }
        }
    }
    choices.sort_by(|&a, &b| {
        let weights = if global { &potential } else { &old };
        let aa = weights[a] as f64 / (1. + counts[a] as f64 / size[a] as f64);
        let bb = weights[b] as f64 / (1. + counts[b] as f64 / size[b] as f64);
        bb.total_cmp(&aa).then(a.cmp(&b))
    });
    choices.truncate(regions);
    let mut replacements: Vec<Option<(Vec<usize>, u64)>> = (0..n).map(|_| None).collect();
    let mut map = vec![usize::MAX; n];
    let mut budget = if global && n >= 10_000 {
        2_000_000usize
    } else {
        8_000_000usize
    };
    for v in choices {
        if budget < 10000 {
            break;
        }
        let start = v + 1 - size[v];
        let vertices: Vec<_> = (start..=v).collect();
        let mut boundary = Vec::new();
        for &x in &vertices {
            for &u in &pattern.row_idx[pattern.col_ptr[order[x]]..pattern.col_ptr[order[x] + 1]] {
                let j = rank[u];
                if j < start || j > v {
                    boundary.push(j);
                }
            }
        }
        boundary.sort_unstable();
        boundary.dedup();
        if boundary.len() + 1 != counts[v] as usize {
            continue;
        }
        let k = vertices.len();
        let mut all = vertices.clone();
        all.extend(boundary);
        let total = all.len();
        let w = total.div_ceil(64);
        if global && total > 512 {
            continue;
        }
        let mut g = vec![0u64; total * w];
        for (j, &x) in all.iter().enumerate() {
            map[x] = j;
        }
        for (j, &x) in vertices.iter().enumerate() {
            for &u in &pattern.row_idx[pattern.col_ptr[order[x]]..pattern.col_ptr[order[x] + 1]] {
                let q = map[rank[u]];
                if q == usize::MAX {
                    return None;
                }
                g[j * w + q / 64] |= 1u64 << (q % 64);
                g[q * w + j / 64] |= 1u64 << (j % 64);
            }
        }
        for a in k..total {
            for b in k..total {
                if a != b {
                    g[a * w + b / 64] |= 1u64 << (b % 64);
                }
            }
        }
        for &x in &all {
            map[x] = usize::MAX;
        }
        let b = (total - k) as u64;
        let filled_nnz = vertices.iter().map(|&v| counts[v] as u64).sum::<u64>() + b * (b + 1) / 2;
        if !legacy && filled_nnz == total as u64 + g.iter().map(|v| v.count_ones() as u64).sum::<u64>() / 2 {
            continue;
        }
        let mut best = old[v];
        let mut candidate = None;
        if global {
            let dense = super::patch::Dense::from_bits(g.clone(), total, k);
            let mut incumbent: Vec<_> = (0..k).collect();
            for tail in [usize::MAX, 32, 16] {
                if let Some((c, p)) = dense.residual_order(&incumbent, tail, &mut budget) {
                    if c < best {
                        best = c;
                        candidate = Some(p.iter().map(|&j| order[vertices[j]]).collect::<Vec<_>>());
                        incumbent = p;
                    }
                }
            }
            for mode in [2, 1] {
                let charge = k * total * w * 4;
                if charge > budget {
                    break;
                }
                budget -= charge;
                let (c, p) = dense.greedy(mode, 0x6718_ABC2 ^ v as u64);
                if c < best {
                    best = c;
                    candidate = Some(p.iter().map(|&j| order[vertices[j]]).collect::<Vec<_>>());
                }
            }
        }
        if !donor_ranks.is_empty() {
            let mut cp = vec![0usize];
            let mut ri = Vec::new();
            for v in 0..total {
                for j in 0..w {
                    let mut bits = g[v * w + j];
                    while bits != 0 {
                        ri.push(j * 64 + bits.trailing_zeros() as usize);
                        bits &= bits - 1;
                    }
                }
                cp.push(ri.len());
            }
            let local = super::ScoringPattern {
                n: total,
                col_ptr: cp,
                row_idx: ri,
            };
            let b = (total - k) as u64;
            for donor_rank in &donor_ranks {
                let mut inside: Vec<_> = (0..k).collect();
                inside.sort_by_key(|&j| donor_rank[order[vertices[j]]]);
                let mut q = inside.clone();
                q.extend(k..total);
                let c = super::flops_of(&local, &q) - b * (b + 1) * (2 * b + 1) / 6;
                if c < best {
                    best = c;
                    candidate = Some(inside.iter().map(|&j| order[vertices[j]]).collect());
                }
            }
        }
        for mode in if donor.is_none() && !global {
            vec![3, 1]
        } else {
            Vec::new()
        } {
            if let Some((p, c)) = super::separator::greedy(g.clone(), total, k, mode, &mut budget) {
                if c < best {
                    best = c;
                    candidate = Some(p.iter().map(|&j| order[vertices[j]]).collect::<Vec<_>>());
                }
            }
        }
        if let Some(p) = candidate {
            replacements[v] = Some((p, best));
        }
    }
    let mut dp = vec![0u64; n];
    let mut take = vec![false; n];
    let mut any = false;
    for v in 0..n {
        dp[v] = (counts[v] as u64).pow(2) + children[v].iter().map(|&c| dp[c]).sum::<u64>();
        if let Some((_, cost)) = &replacements[v] {
            if *cost < dp[v] {
                dp[v] = *cost;
                take[v] = true;
                any = true;
            }
        }
    }
    if !any {
        return None;
    }
    let mut stack = Vec::new();
    for v in (0..n).rev() {
        if parent[v] < 0 {
            stack.push((v, false));
        }
    }
    let mut out = Vec::with_capacity(n);
    while let Some((v, exit)) = stack.pop() {
        if exit {
            out.push(order[v]);
        } else if take[v] {
            out.extend_from_slice(&replacements[v].as_ref()?.0);
        } else {
            stack.push((v, true));
            for &c in children[v].iter().rev() {
                stack.push((c, false));
            }
        }
    }
    #[cfg(test)]
    {
        let expected: u64 = (0..n).filter(|&v| parent[v] < 0).map(|v| dp[v]).sum();
        assert_eq!(super::flops_of(&sp, &out), expected);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "real-corpus equivalence against the preserved legacy engine"]
    fn shared_engine_matches_legacy() {
        let selected = [0,22,41,71,155,178,218,252,2,24,60,109,138,185,210,254,
                        50,79,126,268,285,216,150,74,208,209];
        for (index, (_, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            if !selected.contains(&index) { continue; }
            let cp: Vec<_> = pattern.col_ptr.iter().map(|&v| v as i32).collect();
            let ri: Vec<_> = pattern.row_idx.iter().map(|&v| v as i32).collect();
            let graph = feral_ordering_core::CscPattern::new(pattern.n, &cp, &ri).unwrap();
            let p: Vec<_> = feral_amd::amd_order(&graph).unwrap().into_iter().map(|v| v as usize).collect();
            let mut donor = p.clone();
            donor.reverse();
            assert_eq!(super::refine_legacy(&pattern, &p), crate::ordering::regions::refine(&pattern, &p), "refine {index}");
            assert_eq!(super::crossover_legacy(&pattern, &p, &donor), crate::ordering::regions::crossover(&pattern, &p, &donor), "crossover {index}");
        }
    }
}
