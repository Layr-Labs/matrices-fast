use crate::Pattern;

pub(super) fn refine_legacy(pattern: &Pattern, p: &[usize]) -> Option<Vec<usize>> {
    refine_impl(pattern, p, None, true)
}

pub(super) fn crossover_legacy(
    pattern: &Pattern,
    p: &[usize],
    donor: &[usize],
) -> Option<Vec<usize>> {
    if p == donor {
        return None;
    }
    refine_impl(pattern, p, Some(donor), true)
}

pub(super) fn refine(pattern: &Pattern, p: &[usize]) -> Option<Vec<usize>> {
    refine_impl(pattern, p, None, false)
}

fn refine_impl(
    pattern: &Pattern,
    p: &[usize],
    donor: Option<&[usize]>,
    legacy: bool,
) -> Option<Vec<usize>> {
    let n = pattern.n;
    let min_n = if legacy { 1000 } else { 64 };
    let max_count = if legacy { 257 } else { 1025 };
    if !(min_n..=350_000).contains(&n) || pattern.nnz() > 2_000_000 {
        return None;
    }
    let sp = super::ScoringPattern {
        n,
        col_ptr: pattern.col_ptr.clone(),
        row_idx: pattern.row_idx.clone(),
    };
    let (order, counts, parent) = super::etree_prep(&sp, p);
    let donor_rank = if let Some(donor) = donor {
        if !super::is_bijection(donor, n) {
            return None;
        }
        let mut rank = vec![0; n];
        for (j, &v) in donor.iter().enumerate() {
            rank[v] = j;
        }
        Some(rank)
    } else {
        None
    };
    let mut rank = vec![0; n];
    for (j, &v) in order.iter().enumerate() {
        rank[v] = j;
    }
    let mut size = vec![1usize; n];
    let mut old = vec![0u64; n];
    let mut children = vec![Vec::new(); n];
    for v in 0..n {
        old[v] += (counts[v] as u64).pow(2);
        if parent[v] >= 0 {
            let q = parent[v] as usize;
            size[q] += size[v];
            old[q] += old[v];
            children[q].push(v);
        }
    }
    let mut seen = vec![false; n];
    let mut choices = Vec::new();
    for cap in [128, 512, 1024] {
        for v in 0..n {
            if !seen[v]
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
        let aa = old[a] as f64 / (1. + counts[a] as f64 / size[a] as f64);
        let bb = old[b] as f64 / (1. + counts[b] as f64 / size[b] as f64);
        bb.total_cmp(&aa).then(a.cmp(&b))
    });
    choices.truncate(16);
    let mut replacements: Vec<Option<(Vec<usize>, u64)>> = (0..n).map(|_| None).collect();
    let mut map = vec![usize::MAX; n];
    let mut budget = 8_000_000usize;
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
        let mut clique_work = usize::MAX;
        super::recovery_separator::clique(
            &mut g,
            w,
            &(k..total).collect::<Vec<_>>(),
            &mut clique_work,
        )?;
        for &x in &all {
            map[x] = usize::MAX;
        }
        let b = (total - k) as u64;
        let entries = vertices.iter().map(|&v| counts[v] as u64).sum::<u64>() + b * (b + 1) / 2;
        if !legacy
            && entries == total as u64 + g.iter().map(|v| v.count_ones() as u64).sum::<u64>() / 2
        {
            continue;
        }
        let mut best = old[v];
        let mut candidate = None;
        if let Some(rank) = &donor_rank {
            let (col_ptr, row_idx) = super::recovery_separator::csc_of(&g, total);
            let local = super::ScoringPattern {
                n: total,
                col_ptr,
                row_idx,
            };
            let mut inside: Vec<_> = (0..k).collect();
            inside.sort_by_key(|&j| rank[order[vertices[j]]]);
            let mut q = inside.clone();
            q.extend(k..total);
            let cost = super::flops_of(&local, &q) - b * (b + 1) * (2 * b + 1) / 6;
            if cost < best {
                best = cost;
                candidate = Some(inside.iter().map(|&j| order[vertices[j]]).collect());
            }
        } else {
            for mode in [3, 1] {
                if let Some((p, cost)) =
                    super::recovery_separator::greedy(g.clone(), total, k, mode, &mut budget)
                {
                    if cost < best {
                        best = cost;
                        candidate = Some(p.iter().map(|&j| order[vertices[j]]).collect());
                    }
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
    Some(out)
}
