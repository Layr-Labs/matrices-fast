use super::{Candidate, Pattern, ScoringPattern};
use feral_ordering_core::quotient_graph::{
    create_element_amf, finalize_permutation, finalize_step_amf, select_pivot_amf, Metric, MinFill,
    Workspace, WorkspaceOptions, NONE,
};

struct Core {
    prefix: Vec<usize>,
    ids: Vec<usize>,
    graph: ScoringPattern,
    prefix_cost: u64,
}

fn select(pattern: &Pattern, ranked: &[usize], excluded: &[bool], cap: usize) -> Vec<bool> {
    let mut blocked = vec![false; pattern.n];
    let mut selected = vec![false; pattern.n];
    for &v in ranked {
        if pattern.col_ptr[v + 1] - pattern.col_ptr[v] > cap {
            break;
        }
        if excluded[v] || blocked[v] {
            continue;
        }
        selected[v] = true;
        for &u in &pattern.row_idx[pattern.col_ptr[v]..pattern.col_ptr[v + 1]] {
            blocked[u] = true;
        }
    }
    selected
}

fn lift(pattern: &Pattern, selected: &[bool]) -> Option<Core> {
    let n = pattern.n;
    let mut prefix = Vec::new();
    let mut ids = Vec::new();
    let mut position = vec![usize::MAX; n];
    let mut prefix_cost = 0;
    for v in 0..n {
        if selected[v] {
            let row = &pattern.row_idx[pattern.col_ptr[v]..pattern.col_ptr[v + 1]];
            if row.iter().any(|&u| selected[u]) {
                return None;
            }
            prefix_cost += (row.len() as u64 + 1).pow(2);
            prefix.push(v);
        } else {
            position[v] = ids.len();
            ids.push(v);
        }
    }
    if prefix.is_empty() || ids.is_empty() {
        return None;
    }
    let mut edges = Vec::new();
    for (v, &original) in ids.iter().enumerate() {
        for &u in &pattern.row_idx[pattern.col_ptr[original]..pattern.col_ptr[original + 1]] {
            let u = position[u];
            if u != usize::MAX && u > v {
                edges.push((v, u));
            }
        }
    }
    for &v in &prefix {
        let row = &pattern.row_idx[pattern.col_ptr[v]..pattern.col_ptr[v + 1]];
        for i in 0..row.len() {
            for j in i + 1..row.len() {
                let (a, b) = (position[row[i]], position[row[j]]);
                edges.push((a.min(b), a.max(b)));
            }
        }
    }
    edges.sort_unstable();
    edges.dedup();
    let k = ids.len();
    let mut cp = vec![0; k + 1];
    for &(a, b) in &edges {
        cp[a + 1] += 1;
        cp[b + 1] += 1;
    }
    for i in 0..k {
        cp[i + 1] += cp[i];
    }
    let mut next = cp.clone();
    let mut ri = vec![0; 2 * edges.len()];
    for (a, b) in edges {
        ri[next[a]] = b;
        next[a] += 1;
        ri[next[b]] = a;
        next[b] += 1;
    }
    Some(Core {
        prefix,
        ids,
        graph: ScoringPattern {
            n: k,
            col_ptr: cp,
            row_idx: ri,
        },
        prefix_cost,
    })
}

fn metric_order(graph: &feral_ordering_core::CscPattern<'_>, rerank: bool) -> Option<Vec<i32>> {
    let mut ws = Workspace::new_with_n_buckets(
        graph,
        &WorkspaceOptions { dense_alpha: 10.0 },
        2 * graph.n + 2,
    )
    .ok()?;
    let mut work = 3_000_000usize;
    while ws.nel < ws.n {
        let me = select_pivot_amf(&mut ws)?;
        let elen = ws.elen[me];
        let (start, end, mass, degree) = create_element_amf(&mut ws, me).ok()?;
        let charge = (end - start).saturating_mul(16).saturating_add(64);
        work = work.checked_sub(charge)?;
        finalize_step_amf(&mut ws, me, start, end, mass, degree, elen, true);
        if !rerank {
            continue;
        }
        let end = start + ws.len[me] as usize;
        // Remove all updated candidates before reinserting in their original order.
        for p in start..end {
            let v = ws.iw[p] as usize;
            let bucket = MinFill::bucket(ws.wf[v] as i32, ws.n);
            let (before, after) = (ws.last[v], ws.next[v]);
            if before == NONE {
                ws.head[bucket] = after;
            } else {
                ws.next[before as usize] = after;
            }
            if after != NONE {
                ws.last[after as usize] = before;
            }
        }
        for p in start..end {
            let v = ws.iw[p] as usize;
            let degree = ws.degree[v] as f64;
            let mass = ws.nv[v] as f64 + 1.0;
            let front = ws.degree[me] as f64;
            let score = (degree + front / mass).round().max(1.0) as i32;
            ws.wf[v] = score as i64;
            let bucket = MinFill::bucket(score, ws.n);
            let after = ws.head[bucket];
            if after != NONE {
                ws.last[after as usize] = v as i32;
            }
            ws.next[v] = after;
            ws.last[v] = NONE;
            ws.head[bucket] = v as i32;
            ws.mindeg = ws.mindeg.min(bucket);
        }
    }
    Some(finalize_permutation(&mut ws))
}

fn donors(pattern: &Pattern, incumbent: u64) -> Vec<Vec<usize>> {
    let n = pattern.n;
    let mut ranked: Vec<_> = (0..n).collect();
    ranked.sort_unstable_by_key(|&v| (pattern.col_ptr[v + 1] - pattern.col_ptr[v], v));
    let empty = vec![false; n];
    let first = select(pattern, &ranked, &empty, usize::MAX);
    let mut seen = Vec::new();
    let mut cores = Vec::new();
    let mut work = 12_000_000usize;
    for (second, cap) in [
        (false, usize::MAX),
        (true, usize::MAX),
        (false, 9),
        (true, 9),
        (false, 3),
    ] {
        let mut set = select(pattern, &ranked, if second { &first } else { &empty }, cap);
        let mut pairs = 0;
        for &v in &ranked {
            if !set[v] {
                continue;
            }
            let d = pattern.col_ptr[v + 1] - pattern.col_ptr[v];
            let count = d * d.saturating_sub(1) / 2;
            if pairs + count > 100_000 {
                set[v] = false;
            } else {
                pairs += count;
            }
        }
        let core_n = n - set.iter().filter(|&&v| v).count();
        if core_n < 2 || core_n > 20_000 || seen.contains(&set) {
            continue;
        }
        let entries = pattern.nnz() / 2 + pairs;
        let charge = 3 * n + pattern.nnz() + entries * (entries.max(1).ilog2() as usize + 8);
        let Some(left) = work.checked_sub(charge) else {
            continue;
        };
        work = left;
        seen.push(set.clone());
        let Some(core) = lift(pattern, &set) else {
            continue;
        };
        if core.graph.n > 20_000 || core.graph.row_idx.len() > 350_000 {
            continue;
        }
        let cp: Vec<_> = core.graph.col_ptr.iter().map(|&v| v as i32).collect();
        let ri: Vec<_> = core.graph.row_idx.iter().map(|&v| v as i32).collect();
        let Some(graph) = feral_ordering_core::CscPattern::new(core.graph.n, &cp, &ri) else {
            continue;
        };
        let Ok(order) = feral_amd::amd_order(&graph) else {
            continue;
        };
        let order: Vec<_> = order.into_iter().map(|v| v as usize).collect();
        let cost = core.prefix_cost + super::flops_of(&core.graph, &order);
        cores.push((core, cp, ri, cost, order));
    }
    cores.sort_by_key(|c| c.3);
    cores.retain(|c| c.3 <= incumbent.saturating_mul(2));
    cores.truncate(2);
    let candidates = super::parallel::map_indexed(&cores, 2, |i, (core, cp, ri, ..)| {
        let graph = feral_ordering_core::CscPattern::new(core.graph.n, cp, ri)?;
        let order: Vec<_> = metric_order(&graph, true)?
            .into_iter()
            .map(|v| v as usize)
            .collect();
        if !super::is_bijection(&order, core.graph.n) {
            return None;
        }
        Some((
            i,
            core.prefix_cost + super::flops_of(&core.graph, &order),
            order,
        ))
    });
    for (i, cost, order) in candidates.into_iter().flatten() {
        if cost < cores[i].3 {
            cores[i].3 = cost;
            cores[i].4 = order;
        }
    }
    cores
        .into_iter()
        .map(|(core, _, _, _, order)| {
            let mut full = core.prefix;
            full.extend(order.into_iter().map(|v| core.ids[v]));
            full
        })
        .collect()
}

pub(super) fn refine(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) {
    if !(32..=50_000).contains(&pattern.n)
        || pattern.nnz() > 400_000
        || best.cost < 32 * pattern.n as u64
    {
        return;
    }
    let alternatives = donors(pattern, best.cost);
    if alternatives.is_empty() {
        return;
    }
    let baseline = best.perm.clone();
    let mut pool = super::decomposition::WholePool::new(pattern);
    let mut inputs = vec![baseline.as_slice()];
    inputs.extend(alternatives.iter().map(Vec::as_slice));
    best.consider(scoring, pool.combine(&inputs));
    for order in alternatives {
        best.consider(scoring, Some(order));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_and_residual_costs_are_exact() {
        for mask in 0..1024 {
            let mut edges = Vec::new();
            let mut bit = 0;
            for u in 0..5 {
                for v in u + 1..5 {
                    if mask & (1 << bit) != 0 {
                        edges.push((u, v));
                    }
                    bit += 1;
                }
            }
            let p = Pattern::from_edges(5, &edges);
            let scoring = ScoringPattern {
                n: 5,
                col_ptr: p.col_ptr.clone(),
                row_idx: p.row_idx.clone(),
            };
            for bits in 1..31 {
                let set: Vec<_> = (0..5).map(|v| bits & (1 << v) != 0).collect();
                let Some(core) = lift(&p, &set) else {
                    continue;
                };
                for reverse in [false, true] {
                    let mut order: Vec<_> = (0..core.ids.len()).collect();
                    if reverse {
                        order.reverse();
                    }
                    let expected = core.prefix_cost + super::super::flops_of(&core.graph, &order);
                    let mut full = core.prefix.clone();
                    full.extend(order.iter().map(|&v| core.ids[v]));
                    assert_eq!(expected, super::super::flops_of(&scoring, &full));
                }
                assert!(core
                    .graph
                    .col_ptr
                    .windows(2)
                    .all(|w| core.graph.row_idx[w[0]..w[1]]
                        .windows(2)
                        .all(|p| p[0] < p[1])));
            }
        }
    }

    #[test]
    fn metric_control_matches_amf_and_variants_are_deterministic() {
        for n in [1, 5, 65, 150] {
            let edges: Vec<_> = (0..n)
                .flat_map(|u| {
                    [1, 3, 9]
                        .into_iter()
                        .filter_map(move |d| (u + d < n).then_some((u, u + d)))
                })
                .collect();
            let p = Pattern::from_edges(n, &edges);
            let cp: Vec<_> = p.col_ptr.iter().map(|&v| v as i32).collect();
            let ri: Vec<_> = p.row_idx.iter().map(|&v| v as i32).collect();
            let g = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
            let options = feral_amf::AmfOptions {
                dense_alpha: 10.0,
                ..Default::default()
            };
            assert_eq!(
                metric_order(&g, false).unwrap(),
                feral_amf::amf_order_opts(&g, &options).unwrap().0
            );
            let order = metric_order(&g, true).unwrap();
            assert_eq!(Some(order.clone()), metric_order(&g, true));
            assert!(super::super::is_bijection(
                &order.into_iter().map(|v| v as usize).collect::<Vec<_>>(),
                n
            ));
        }
    }
}
