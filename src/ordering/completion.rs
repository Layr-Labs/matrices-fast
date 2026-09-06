//! Bounded terminal chordal-completion cleanup.
//!
//! All graph work uses standard-library containers. The caller supplies the
//! incumbent's permuted pattern, elimination tree and exact column counts.
//! The watcher implementation is retained from this workspace's earlier work.

use super::minl_watch::watcher_minimalize;

/// Reconstruct the incumbent completion from its elimination-tree children,
/// remove only certified redundant fill edges, and extract a new ordering.
/// Limits depend on graph structure, never matrix identity or elapsed time.
pub(super) fn refine(
    n: usize,
    original_cp: &[usize],
    original_ri: &[usize],
    permuted_cp: &[usize],
    permuted_ri: &[usize],
    parent: &[Option<usize>],
    counts: &[usize],
    perm: &[usize],
) -> Option<Vec<usize>> {
    refine_limited(n, original_cp, original_ri, permuted_cp, permuted_ri,
        parent, counts, perm, 8_000_000)
}

pub(super) fn refine_limited(
    n: usize,
    original_cp: &[usize],
    original_ri: &[usize],
    permuted_cp: &[usize],
    permuted_ri: &[usize],
    parent: &[Option<usize>],
    counts: &[usize],
    perm: &[usize],
    ops: i64,
) -> Option<Vec<usize>> {
    const MAX_LNNZ: usize = 300_000;
    const MAX_FILL: usize = 100_000;
    if n == 0 || n > 30_000 || counts.len() != n {
        return None;
    }
    let lnnz = counts.iter().try_fold(0usize, |s, &x| s.checked_add(x))?;
    if lnnz > MAX_LNNZ { return None; }
    let mut children = vec![Vec::<usize>::new(); n];
    for j in 0..n {
        if let Some(p) = parent[j] {
            if p <= j || p >= n { return None; }
            children[p].push(j);
        }
    }
    let mut columns = vec![Vec::<u32>::new(); n];
    let mut adj = vec![Vec::<u32>::new(); n];
    let mut mark = vec![usize::MAX; n];
    let mut total = 0usize;
    for j in 0..n {
        let mut reach = Vec::<u32>::new();
        for &i in &permuted_ri[permuted_cp[j]..permuted_cp[j+1]] {
            if i > j && mark[i] != j {
                mark[i] = j;
                reach.push(i as u32);
            }
        }
        for &c in &children[j] {
            for &i in &columns[c] {
                let i = i as usize;
                if i > j && mark[i] != j {
                    mark[i] = j;
                    reach.push(i as u32);
                }
            }
            columns[c] = Vec::new();
        }
        // Check the reconstruction against exact symbolic column counts.
        if reach.len().checked_add(1)? != counts[j] { return None; }
        total = total.checked_add(reach.len())?;
        if total > MAX_LNNZ { return None; }
        let v = perm[j];
        for &i in &reach {
            let w = perm[i as usize];
            adj[v].push(w as u32);
            adj[w].push(v as u32);
        }
        columns[j] = reach;
    }
    drop(columns);
    drop(children);
    let mut fill = Vec::<(u32,u32)>::new();
    for v in 0..n {
        adj[v].sort_unstable();
        let original = &original_ri[original_cp[v]..original_cp[v+1]];
        for &w in &adj[v] {
            if (w as usize) > v && original.binary_search(&(w as usize)).is_err() {
                fill.push((v as u32,w));
                if fill.len() > MAX_FILL { return None; }
            }
        }
    }
    if fill.is_empty() { return None; }
    let mut ids: Vec<u32> = (0..fill.len() as u32).collect();
    ids.sort_unstable_by_key(|&id| {
        let (u,v) = fill[id as usize];
        (adj[u as usize].len()+adj[v as usize].len(),id)
    });
    let result = watcher_minimalize(&mut adj,&fill,&ids,ops,1024);
    if result.removed == 0 { return None; }
    Some(minl_mcs_peo(n,&adj))
}

fn minl_mcs_peo(n: usize, adj: &[Vec<u32>]) -> Vec<usize> {
    let mut w = vec![0u32; n];
    let mut vis = vec![false; n];
    let mut buckets: Vec<Vec<u32>> = vec![(0..n as u32).rev().collect()];
    let mut maxw = 0usize;
    let mut visit: Vec<usize> = Vec::with_capacity(n);
    while visit.len() < n {
        let Some(x) = buckets[maxw].pop() else {
            if maxw == 0 {
                break; // unreachable on well-formed input
            }
            maxw -= 1;
            continue;
        };
        let xu = x as usize;
        if vis[xu] || (w[xu] as usize) != maxw {
            continue;
        }
        vis[xu] = true;
        visit.push(xu);
        for &uw in &adj[xu] {
            let u = uw as usize;
            if !vis[u] {
                w[u] += 1;
                let nw = w[u] as usize;
                if nw >= buckets.len() {
                    buckets.resize_with(nw + 1, Vec::new);
                }
                buckets[nw].push(uw);
                if nw > maxw {
                    maxw = nw;
                }
            }
        }
    }
    visit.reverse();
    visit
}

