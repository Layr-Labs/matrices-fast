//! Connected elimination-tree separators. Hanging branches are represented by
//! their exact boundary cliques. A tree DP combines compatible replacements.
use crate::Pattern;
use std::collections::{BTreeSet, BinaryHeap};
use super::recovery_separator::{charge, clique};

struct Proposal {
    order: Vec<usize>,
    children: Vec<usize>,
    cost: u64,
}
pub(super) fn greedy(
    g: Vec<u64>,
    n: usize,
    k: usize,
    mode: usize,
    work: &mut usize,
) -> Option<(Vec<usize>, u64)> {
    let objective = if (1..=3).contains(&mode) { mode } else { 4 };
    super::recovery_separator::greedy(g, n, k, objective, work)
}

pub(super) fn refine(pattern: &Pattern, incumbent: &[usize]) -> Option<Vec<usize>> {
    refine_impl(pattern, incumbent, false, false)
}
pub(super) fn refine_large(pattern: &Pattern, incumbent: &[usize]) -> Option<Vec<usize>> {
    refine_impl(pattern, incumbent, true, false)
}
pub(super) fn refine_core(pattern: &Pattern, incumbent: &[usize]) -> Option<Vec<usize>> {
    refine_impl(pattern, incumbent, true, true)
}
fn refine_impl(
    pattern: &Pattern,
    incumbent: &[usize],
    large: bool,
    core: bool,
) -> Option<Vec<usize>> {
    let n = pattern.n;
    if !(1000..=80_000).contains(&n) || pattern.nnz() > 500_000 {
        return None;
    }
    if large && !core && n < 8000 { return None; }
    let sp = super::ScoringPattern {
        n,
        col_ptr: pattern.col_ptr.clone(),
        row_idx: pattern.row_idx.clone(),
    };
    let (p, counts, parent) = super::etree_prep(&sp, incumbent);
    if large
        && counts.iter().map(|&c| (c as u64).pow(2)).sum::<u64>() < 256 * n as u64
    {
        return None;
    }
    let mut rank = vec![0; n];
    for (j, &v) in p.iter().enumerate() {
        rank[v] = j;
    }
    let mut kids = vec![Vec::new(); n];
    for v in 0..n {
        if parent[v] >= 0 {
            kids[parent[v] as usize].push(v);
        }
    }
    let mut bag: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut size = vec![1usize; n];
    let mut old = vec![0u64; n];
    let mut build_work = 4_000_000usize;
    for v in 0..n {
        let mut b = Vec::new();
        for &u in &pattern.row_idx[pattern.col_ptr[p[v]]..pattern.col_ptr[p[v] + 1]] {
            if rank[u] > v {
                b.push(rank[u]);
            }
        }
        for &ch in &kids[v] {
            charge(&mut build_work, bag[ch].len())?;
            b.extend(bag[ch].iter().copied().filter(|&u| u != v));
        }
        charge(&mut build_work, b.len())?;
        b.sort_unstable();
        b.dedup();
        if b.len() + 1 != counts[v] as usize {
            return None;
        }
        old[v] += ((b.len() + 1) as u64).pow(2);
        bag[v] = b;
        if parent[v] >= 0 {
            let q = parent[v] as usize;
            size[q] += size[v];
            old[q] += old[v];
        }
    }
    let mut seen = vec![false; n];
    let mut roots = Vec::new();
    for cap in [
        64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072,
    ] {
        for v in 0..n {
            if !seen[v]
                && size[v] >= 32
                && size[v] <= cap
                && bag[v].len() <= 512
                && (parent[v] < 0 || size[parent[v] as usize] > cap)
            {
                seen[v] = true;
                let priority = old[v] as f64 / (1. + bag[v].len() as f64 / size[v] as f64);
                roots.push((priority, v));
            }
        }
    }
    roots.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    roots.truncate(if core {
        1
    } else if large {
        3
    } else {
        12
    });
    let mut proposals: Vec<Vec<Proposal>> = (0..n).map(|_| Vec::new()).collect();
    let mut work = 24_000_000usize;
    let mut map = vec![usize::MAX; n];
    let mut metis_tickets = 2usize;
    let mut joint_tickets = 2usize;
    let mut flip_tickets = 1usize;
    let mut seen_regions = BTreeSet::new();
    let caps: &[usize] = if large {
        &[8192, 2048, 512, 128]
    } else {
        &[512, 1024, 256]
    };
    for &cap in caps {
        for &(_, root) in &roots {
            if work < 10000 {
                break;
            }
            let mut heap = BinaryHeap::new();
            heap.push((counts[root], root));
            let mut vertices = Vec::new();
            while vertices.len() < cap {
                let Some((_, v)) = heap.pop() else { break };
                vertices.push(v);
                for &ch in &kids[v] {
                    heap.push((counts[ch], ch));
                }
            }
            vertices.sort_unstable();
            if !seen_regions.insert((root, vertices.clone())) { continue; }
            let k = vertices.len();
            let mut all = vertices.clone();
            all.extend_from_slice(&bag[root]);
            let total = all.len();
            let w = total.div_ceil(64);
            for (j, &v) in all.iter().enumerate() {
                map[v] = j;
            }
            let mut g = vec![0u64; total * w];
            let mut children = Vec::new();
            let mut valid = true;
            let mut interfaces = BTreeSet::new();
            for (j, &v) in vertices.iter().enumerate() {
                for &u in &pattern.row_idx[pattern.col_ptr[p[v]]..pattern.col_ptr[p[v] + 1]] {
                    let a = map[rank[u]];
                    if a != usize::MAX {
                        g[j * w + a / 64] |= 1u64 << (a % 64);
                        g[a * w + j / 64] |= 1u64 << (j % 64);
                    }
                }
                for &ch in &kids[v] {
                    if map[ch] >= k {
                        children.push(ch);
                        let b: Vec<usize> = bag[ch].iter().map(|&u| map[u]).collect();
                        if b.iter().any(|&u| u == usize::MAX)
                            || (interfaces.insert(b.clone())
                                && clique(&mut g, w, &b, &mut work).is_none())
                        {
                            valid = false;
                            break;
                        }
                    }
                }
                if !valid {
                    break;
                }
            }
            if valid {
                valid = clique(&mut g, w, &(k..total).collect::<Vec<_>>(), &mut work).is_some();
            }
            for &v in &all {
                map[v] = usize::MAX;
            }
            if !valid {
                continue;
            }
            // The incumbent completion contains this interface graph. Equal
            // entry counts certify no added fill, hence no ordering can lower
            // its chordal potential with the same frozen exterior clique.
            let exterior = (total - k) as u64;
            let edges = g.iter().map(|row| row.count_ones() as u64).sum::<u64>() / 2;
            let entries = vertices.iter().map(|&v| counts[v] as u64).sum::<u64>();
            if entries == k as u64 + edges - exterior * exterior.saturating_sub(1) / 2 {
                continue;
            }
            let mut best: Option<(Vec<usize>, u64)> = None;
            let original: u64 = vertices.iter().map(|&v| (counts[v] as u64).pow(2)).sum();
            if large && k >= 8 && k <= 512 && total <= 1024 && joint_tickets > 0 {
                let mut selected: Vec<_> = (0..k).collect();
                selected.sort_by_key(|&v| {
                    (
                        std::cmp::Reverse(
                            g[v * w..(v + 1) * w]
                                .iter()
                                .map(|a| a.count_ones())
                                .sum::<u32>(),
                        ),
                        v,
                    )
                });
                selected.truncate(8);
                joint_tickets -= 1;
                if let Some((order, cost)) = super::joint::schedule(&g, total, k, &selected) {
                    if cost < original {
                        best = Some((order, cost));
                    }
                }
            }
            if large {
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
                if ri.len() <= 1_000_000 {
                    let cc: Vec<i32> = cp.iter().map(|&x| x as i32).collect();
                    let rr: Vec<i32> = ri.iter().map(|&x| x as i32).collect();
                    if let Some(graph) = feral_ordering_core::CscPattern::new(total, &cc, &rr) {
                        let sources: &[usize] = if metis_tickets > 0 {
                            metis_tickets -= 1;
                            &[0, 1, 2]
                        } else { &[0, 1] };
                        let candidates = super::parallel::map_indexed(sources, 3, |_, &source| {
                            match source {
                                0 => feral_amf::amf_order_opts(&graph, &feral_amf::AmfOptions::default()).map(|(p, ..)| p),
                                1 => feral_amd::amd_order(&graph),
                                _ => feral_metis::metis_order_full(&graph, &feral_metis::MetisOptions {
                                    niparts: 4, ..Default::default()
                                }).map(|(p, ..)| p),
                            }
                        });
                        let local_pattern = Pattern {
                            n: total,
                            col_ptr: cp.clone(),
                            row_idx: ri.clone(),
                        };
                        let sp = super::ScoringPattern {
                            n: total,
                            col_ptr: cp,
                            row_idx: ri,
                        };
                        for result in candidates {
                            if let Ok(order) = result {
                                let inside: Vec<usize> = order
                                    .into_iter()
                                    .map(|v| v as usize)
                                    .filter(|&v| v < k)
                                    .collect();
                                if !super::is_bijection(&inside, k) {
                                    continue;
                                }
                                let mut full = inside.clone();
                                full.extend(k..total);
                                let b = (total - k) as u64;
                                let cost = super::flops_of(&sp, &full)
                                    .checked_sub(b * (b + 1) * (2 * b + 1) / 6)?;
                                if cost < best.as_ref().map_or(original, |b| b.1) {
                                    best = Some((inside, cost));
                                }
                            }
                        }
                        if k <= 512 {
                            if let Some((order, cost)) = greedy(g.clone(), total, k, 3, &mut work) {
                                if cost < best.as_ref().map_or(original, |b| b.1) {
                                    best = Some((order, cost));
                                }
                            }
                        }
                        let mut initial = best
                            .as_ref()
                            .map_or_else(|| (0..k).collect::<Vec<_>>(), |b| b.0.clone());
                        initial.extend(k..total);
                        if k <= 512 && total <= 1200 && flip_tickets > 0 {
                            flip_tickets -= 1;
                            let permuted = super::permute_pattern(&sp, &initial);
                            let tree = super::EliminationTree::from_pattern(&permuted);
                            let counts: Vec<u32> = super::symbolic_counts(&permuted, &tree).1
                                .into_iter()
                                .map(|v| v as u32)
                                .collect();
                            if let Some(candidates) = super::completion::flip_candidates(
                                total,
                                &local_pattern.col_ptr,
                                &local_pattern.row_idx,
                                &initial,
                                &counts,
                                k,
                            ) {
                                for q in candidates {
                                    let b = (total - k) as u64;
                                    let cost =
                                        super::flops_of(&sp, &q) - b * (b + 1) * (2 * b + 1) / 6;
                                    if cost < best.as_ref().map_or(original, |b| b.1) {
                                        best = Some((q[..k].to_vec(), cost));
                                        initial = q;
                                    }
                                }
                            }
                        }
                        if let Some(order) =
                            super::promotions::boundary(&local_pattern, &initial, k)
                        {
                            if order[..k].iter().all(|&v| v < k) {
                                let b = (total - k) as u64;
                                let cost =
                                    super::flops_of(&sp, &order) - b * (b + 1) * (2 * b + 1) / 6;
                                if cost < best.as_ref().map_or(original, |b| b.1) {
                                    best = Some((order[..k].to_vec(), cost));
                                }
                            }
                        }
                    }
                }
            }
            for mode in if large { Vec::new() } else { vec![3, 1, 2, 4] } {
                let Some((order, cost)) = greedy(g.clone(), total, k, mode, &mut work) else {
                    break;
                };
                if cost < best.as_ref().map_or(original, |b| b.1) {
                    best = Some((order, cost));
                }
            }
            if let Some((order, cost)) = best {
                children.sort_unstable();
                proposals[root].push(Proposal {
                    order: order.iter().map(|&j| vertices[j]).collect(),
                    children,
                    cost,
                });
            }
        }
    }
    let mut dp = vec![0u64; n];
    let mut take = vec![None; n];
    let mut any = false;
    for v in 0..n {
        dp[v] = (counts[v] as u64).pow(2) + kids[v].iter().map(|&ch| dp[ch]).sum::<u64>();
        for (j, pr) in proposals[v].iter().enumerate() {
            let cost = pr.cost + pr.children.iter().map(|&ch| dp[ch]).sum::<u64>();
            if cost < dp[v] {
                dp[v] = cost;
                take[v] = Some(j);
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
            if let Some(j) = take[v] {
                out.extend(proposals[v][j].order.iter().map(|&u| p[u]));
            } else {
                out.push(p[v]);
            }
            continue;
        }
        stack.push((v, true));
        let children = take[v].map_or(&kids[v], |j| &proposals[v][j].children);
        for &ch in children.iter().rev() {
            stack.push((ch, false));
        }
    }
    if out.len() == n {
        Some(out)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_free_boundary_order_has_minimum_cost() {
        fn replay(mut a: [[bool; 5]; 5], order: &[usize]) -> (u64, u64) {
            let (mut cost, mut entries) = (0, 0);
            for &v in order {
                let neighbors: Vec<_> = (0..5).filter(|&u| a[v][u]).collect();
                entries += neighbors.len() as u64 + 1;
                cost += (neighbors.len() as u64 + 1).pow(2);
                for &u in &neighbors {
                    for &w in &neighbors { if u != w { a[u][w] = true; } }
                    a[u][v] = false;
                }
                a[v].fill(false);
            }
            (cost, entries)
        }
        let orders = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];
        // Exhaust all free/interior edge patterns with a fixed two-vertex
        // exterior clique, including disconnected and dense interfaces.
        for mask in 0..512usize {
            let mut a = [[false; 5]; 5];
            a[3][4] = true; a[4][3] = true;
            let mut bit = 0;
            for u in 0..5 {
                for v in 0..u {
                    if v == 3 { continue; }
                    if mask & (1 << bit) != 0 { a[u][v] = true; a[v][u] = true; }
                    bit += 1;
                }
            }
            let edges = a.iter().flatten().filter(|&&edge| edge).count() as u64 / 2;
            for incumbent in &orders {
                let (cost, entries) = replay(a, incumbent);
                if entries == 3 + edges - 1 {
                    for order in &orders { assert!(replay(a, order).0 >= cost); }
                }
            }
        }
    }
    // Literal elimination independently checks every returned partial order,
    // including its fixed, live exterior boundary.
    #[test]
    fn boundary_greedy_matches_literal() {
        let mut seed = 7193u64;
        for n in [5usize, 17, 65, 129] {
            for density in [2u64, 5, 9] {
                let w = n.div_ceil(64);
                let mut g = vec![0u64; n * w];
                let mut a = vec![vec![false; n]; n];
                for u in 0..n {
                    for v in u + 1..n {
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        if seed % 10 < density {
                            a[u][v] = true;
                            a[v][u] = true;
                            g[u * w + v / 64] |= 1u64 << (v % 64);
                            g[v * w + u / 64] |= 1u64 << (u % 64);
                        }
                    }
                }
                for k in [n / 2, n] {
                    for mode in [1, 2, 3, 4] {
                        let (p, cost) =
                            greedy(g.clone(), n, k, mode, &mut 100_000_000usize).unwrap();
                        let mut seen = vec![false; k];
                        let mut b = a.clone();
                        let mut actual = 0u64;
                        for v in p {
                            assert!(v < k && !seen[v]);
                            seen[v] = true;
                            let nb: Vec<_> = (0..n).filter(|&u| b[v][u]).collect();
                            actual += ((nb.len() + 1) as u64).pow(2);
                            for &u in &nb {
                                for &t in &nb {
                                    if u != t {
                                        b[u][t] = true;
                                    }
                                }
                                b[u][v] = false;
                            }
                            b[v].fill(false);
                        }
                        assert_eq!(actual, cost);
                        assert!(seen.into_iter().all(|v| v));
                    }
                }
            }
        }
    }
    #[test]
    fn exhausted_budget_aborts() {
        let mut g = vec![0; 8];
        clique(&mut g, 1, &(0..8).collect::<Vec<_>>(), &mut 100usize).unwrap();
        assert!(greedy(g, 8, 8, 3, &mut 0usize).is_none());
    }
}
