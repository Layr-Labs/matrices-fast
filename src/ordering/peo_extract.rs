//! Two linear-work PEO tie variations of one incumbent chordal completion.
//!
//! This is a sibling of the terminal fill-edge watcher: it never erases edges
//! or changes the watcher's candidate, tie order, or work allowance.

pub(super) const MAX_N: usize = 30_000;
pub(super) const MAX_INPUT_NNZ: usize = 180_000;
pub(super) const MAX_LNNZ: usize = 300_000;

#[cfg(test)]
mod pooled_reference;
#[cfg(test)]
mod priority_reference;
mod indexed_priority;
mod lex_bfs;
#[cfg(test)]
thread_local! {static USE_INDEXED_PRIORITY:std::cell::Cell<bool>=std::cell::Cell::new(true);}
#[cfg(test)]
pub(super) fn set_indexed_priority(active:bool) {USE_INDEXED_PRIORITY.with(|c|c.set(active));}

#[cfg(test)]
thread_local! { static USE_POOLED_REFERENCE: std::cell::Cell<bool> = std::cell::Cell::new(false); }
#[cfg(test)]
pub(super) fn set_pooled_reference(active: bool) {
    if active { pooled_reference::reset(); }
    USE_POOLED_REFERENCE.with(|c| c.set(active));
}

#[cfg(test)]
pub(super) mod prof {
    use std::cell::Cell;
    thread_local! {
        static RECON: Cell<f64> = const { Cell::new(0.0) };
        static MCS: Cell<f64> = const { Cell::new(0.0) };
        static RECON_CALLS: Cell<u64> = const { Cell::new(0) };
    }
    pub(crate) fn add_recon(secs: f64) {
        RECON.with(|c| c.set(c.get() + secs));
        RECON_CALLS.with(|c| c.set(c.get() + 1));
    }
    pub(crate) fn add_mcs(secs: f64) {
        MCS.with(|c| c.set(c.get() + secs));
    }
    /// (recon secs, mcs secs, recon calls) since the last drain.
    pub(crate) fn take() -> (f64, f64, u64) {
        (
            RECON.with(|c| c.replace(0.0)),
            MCS.with(|c| c.replace(0.0)),
            RECON_CALLS.with(|c| c.replace(0)),
        )
    }
}


struct Completion {
    row_ptr: Vec<usize>,
    neighbors: Vec<u32>,
}

impl Completion {
    fn len(&self) -> usize { self.row_ptr.len() - 1 }
}

impl std::ops::Index<usize> for Completion {
    type Output = [u32];
    fn index(&self, vertex: usize) -> &[u32] {
        &self.neighbors[self.row_ptr[vertex]..self.row_ptr[vertex + 1]]
    }
}

pub(super) fn candidates(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    parent: &[Option<usize>],
    counts: &[usize],
    incumbent: &[usize],
) -> Option<[Vec<usize>; 2]> {
    candidates_bounded(n, cp, ri, parent, counts, incumbent, MAX_N, MAX_INPUT_NNZ, MAX_LNNZ)
}

/// The same two MCS extractions under caller-supplied structural limits (dimension,
/// input nonzeros, factor nonzeros). Limits are structure, never identity.
pub(super) fn candidates_bounded(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    parent: &[Option<usize>],
    counts: &[usize],
    incumbent: &[usize],
    max_n: usize,
    max_nnz: usize,
    max_lnnz: usize,
) -> Option<[Vec<usize>; 2]> {
    #[cfg(test)]
    if USE_POOLED_REFERENCE.with(|c| c.get()) {
        return pooled_reference::candidates_bounded(n, cp, ri, parent, counts,
            incumbent, max_n, max_nnz, max_lnnz);
    }
    #[cfg(test)]
    let tr = std::time::Instant::now();
    let adj = reconstruct(n, cp, ri, parent, counts, incumbent, max_n, max_nnz, max_lnnz)?;
    #[cfg(test)]
    prof::add_recon(tr.elapsed().as_secs_f64());
    #[cfg(test)]
    let tm = std::time::Instant::now();
    let forward = mcs_peo(&adj, incumbent, false);
    let reverse = mcs_peo(&adj, incumbent, true);
    #[cfg(test)]
    prof::add_mcs(tm.elapsed().as_secs_f64());
    if !super::is_bijection(&forward, n) || !super::is_bijection(&reverse, n) {
        return None;
    }
    Some([forward, reverse])
}

fn reconstruct(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    parent: &[Option<usize>],
    counts: &[usize],
    incumbent: &[usize],
    max_n: usize,
    max_nnz: usize,
    max_lnnz: usize,
) -> Option<Completion> {
    if n == 0 || n > max_n || ri.len() > max_nnz
        || cp.len() != n + 1 || parent.len() != n || counts.len() != n
        || cp.first().copied() != Some(0) || cp.last().copied() != Some(ri.len())
        || !super::is_bijection(incumbent, n)
    {
        return None;
    }
    if cp.windows(2).any(|w| w[0] > w[1]) || ri.iter().any(|&v| v >= n) {
        return None;
    }
    let lnnz = counts.iter().enumerate().try_fold(0usize, |sum, (j, &c)| {
        if c == 0 || c > n - j { None } else { sum.checked_add(c) }
    })?;
    if lnnz > max_lnnz { return None; }

    let mut child_ptr = vec![0usize; n + 1];
    for (j, &p) in parent.iter().enumerate() {
        if let Some(p) = p {
            if p <= j || p >= n { return None; }
            child_ptr[p + 1] += 1;
        }
    }
    for j in 0..n { child_ptr[j + 1] += child_ptr[j]; }
    let mut cursor = child_ptr[..n].to_vec();
    let mut children = vec![0u32; child_ptr[n]];
    for (j, &p) in parent.iter().enumerate() {
        if let Some(p) = p {
            children[cursor[p]] = j as u32;
            cursor[p] += 1;
        }
    }

    let edges = lnnz - n;
    let mut column_ptr = Vec::with_capacity(n + 1);
    column_ptr.push(0usize);
    for &count in counts {
        column_ptr.push(column_ptr.last().copied()?.checked_add(count - 1)?);
    }
    let mut columns = Vec::<u32>::with_capacity(edges);
    let mut degrees = vec![0usize; n];
    let mut mark = vec![usize::MAX; n];
    for j in 0..n {
        for &i in &ri[cp[j]..cp[j + 1]] {
            if i > j && mark[i] != j {
                mark[i] = j;
                columns.push(i as u32);
            }
        }
        for &child in &children[child_ptr[j]..child_ptr[j + 1]] {
            let child = child as usize;
            for offset in column_ptr[child]..column_ptr[child + 1] {
                let i = columns[offset] as usize;
                if i > j && mark[i] != j {
                    mark[i] = j;
                    columns.push(i as u32);
                }
            }
        }
        // Never extract from an unchecked reconstruction. Each column is
        // scanned at most once again, at its unique elimination-tree parent.
        if columns.len() != column_ptr[j + 1] { return None; }
        let v = incumbent[j];
        degrees[v] += counts[j] - 1;
        for &i in &columns[column_ptr[j]..column_ptr[j + 1]] {
            let w = incumbent[i as usize];
            degrees[w] += 1;
        }
    }
    if columns.len() != edges { return None; }

    let mut row_ptr = vec![0usize; n + 1];
    for v in 0..n { row_ptr[v + 1] = row_ptr[v].checked_add(degrees[v])?; }
    if row_ptr[n] != edges.checked_mul(2)? { return None; }
    let mut neighbors = vec![0u32; row_ptr[n]];
    cursor.copy_from_slice(&row_ptr[..n]);
    // Replay the original column/neighbor append events. Every row retains
    // its exact former neighbor order, including the reverse MCS variation.
    for j in 0..n {
        let v = incumbent[j];
        for &i in &columns[column_ptr[j]..column_ptr[j + 1]] {
            let w = incumbent[i as usize];
            neighbors[cursor[v]] = w as u32;
            cursor[v] += 1;
            neighbors[cursor[w]] = v as u32;
            cursor[w] += 1;
        }
    }
    Some(Completion { row_ptr, neighbors })
}

/// MCS with a fixed structural priority for ties in visited-neighbor count.
/// The completed graph is reconstructed once; lazy heap updates are bounded
/// by its edge count. Existing linear/LIFO candidates remain unchanged.
#[cfg(test)]
pub(super) fn priority_candidate_bounded(
    n: usize, cp: &[usize], ri: &[usize], parent: &[Option<usize>],
    counts: &[usize], incumbent: &[usize], original_degree: &[usize],
    policy: usize,
) -> Option<Vec<usize>> {
    priority_candidate_with_limits(n,cp,ri,parent,counts,incumbent,original_degree,
        policy,12_000,200_000,150_000)
}

pub(super) fn priority_candidate_with_limits(
    n:usize,cp:&[usize],ri:&[usize],parent:&[Option<usize>],counts:&[usize],
    incumbent:&[usize],original_degree:&[usize],policy:usize,
    max_n:usize,max_input:usize,max_factor:usize,
) -> Option<Vec<usize>> {
    if original_degree.len() != n || policy > 11 { return None; }
    let adj = reconstruct(n, cp, ri, parent, counts, incumbent,
        max_n,max_input,max_factor)?;
    Some(priority_mcs(&adj, incumbent, counts, original_degree, policy))
}

pub(super) fn lex_candidates_with_limits(n:usize,cp:&[usize],ri:&[usize],
    parent:&[Option<usize>],counts:&[usize],incumbent:&[usize],degrees:&[usize],
    modes:&[usize],max_n:usize,max_input:usize,max_factor:usize)->Option<Vec<Vec<usize>>> {
    if degrees.len()!=n ||modes.iter().any(|&mode|mode>5) {return None;}
    let adj=reconstruct(n,cp,ri,parent,counts,incumbent,max_n,max_input,max_factor)?;
    let mut reverse=incumbent.to_vec();reverse.reverse();
    let mut degree=incumbent.to_vec();
    if modes.iter().any(|&mode|mode>=4) {degree.sort_by_key(|&v|std::cmp::Reverse(degrees[v]));}
    Some(modes.iter().map(|&mode|lex_bfs::order(&adj,
        match mode/2 {0=>incumbent,1=>&reverse,_=>&degree},mode%2!=0)).collect())
}

fn priority_mcs(adj: &Completion, incumbent: &[usize], counts: &[usize],
    original_degree: &[usize], policy: usize) -> Vec<usize> {
    let n = adj.len();
    let mut priority = vec![0u64; n];
    for (pos, &v) in incumbent.iter().enumerate() {
        let rank = (n - pos) as u64;
        priority[v] = match policy {
            0 => ((n.saturating_sub(original_degree[v]) as u64) << 32) | rank,
            1 => ((original_degree[v] as u64) << 32) | rank,
            2 => ((n.saturating_sub(adj[v].len()) as u64) << 32) | rank,
            3 => ((n.saturating_sub(counts[pos]) as u64) << 32) | rank,
            4 => pos as u64,
            6 => ((original_degree[v] as u64) << 32) | pos as u64,
            7 => ((adj[v].len() as u64) << 32) | rank,
            8 => ((adj[v].len() as u64) << 32) | pos as u64,
            9 => ((n.saturating_sub(adj[v].len().saturating_sub(original_degree[v])) as u64) << 32) | rank,
            10 => ((adj[v].len().saturating_sub(original_degree[v]) as u64) << 32) | rank,
            11 => (((original_degree[v] as u64).saturating_mul(65_536)
                / (adj[v].len() as u64+1)).min(u32::MAX as u64) << 32) | rank,
            _ => {
                let mut h = (pos as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
                h = (h ^ (h >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                h = (h ^ (h >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                h ^ (h >> 31)
            }
        };
    }
    #[cfg(test)]
    if !USE_INDEXED_PRIORITY.with(|c|c.get()) {return priority_mcs_stale_keys(adj,&priority);}
    indexed_priority::order(adj,&priority)
}

#[cfg(test)]
fn priority_mcs_stale_keys(adj:&Completion,priority:&[u64])->Vec<usize> {
    let n=adj.len();
    let mut heap = std::collections::BinaryHeap::with_capacity(n);
    for v in 0..n { heap.push((0usize, priority[v], v)); }
    let mut weights = vec![0usize; n];
    let mut visited = vec![false; n];
    let mut out = Vec::with_capacity(n);
    while let Some((weight, _, v)) = heap.pop() {
        if visited[v] || weight != weights[v] { continue; }
        visited[v] = true; out.push(v);
        if out.len()==n {break;}
        for &u in &adj[v] {
            let u = u as usize;
            if !visited[u] {
                weights[u] += 1;
                heap.push((weights[u], priority[u], u));
            }
        }
    }
    out.reverse(); out
}

#[cfg(test)]
fn mcs_peo_stale(adj: &Completion, incumbent: &[usize], reverse_adj: bool) -> Vec<usize> {
    let n = adj.len();
    let mut weight = vec![0usize; n];
    let mut visited = vec![false; n];
    // The first visited zero-weight vertex follows the incumbent, rather than
    // a label ordering. Updated buckets are deterministic LIFO stacks.
    let mut buckets: Vec<Vec<u32>> = vec![incumbent.iter().rev().map(|&v| v as u32).collect()];
    let mut max_weight = 0usize;
    let mut visit = Vec::with_capacity(n);
    while visit.len() < n {
        let Some(v) = buckets[max_weight].pop() else {
            if max_weight == 0 { break; }
            max_weight -= 1;
            continue;
        };
        let v = v as usize;
        if visited[v] || weight[v] != max_weight { continue; }
        visited[v] = true;
        visit.push(v);
        let neighbors = &adj[v];
        for offset in 0..neighbors.len() {
            let index = if reverse_adj { neighbors.len() - 1 - offset } else { offset };
            let u = neighbors[index] as usize;
            if !visited[u] {
                weight[u] += 1;
                let new_weight = weight[u];
                if new_weight >= buckets.len() {
                    buckets.resize_with(new_weight + 1, Vec::new);
                }
                buckets[new_weight].push(u as u32);
                max_weight = max_weight.max(new_weight);
            }
        }
    }
    // MCS visits a reverse PEO. Every edge causes exactly one bucket insertion;
    // stale entries do not exceed that count, so work and storage are O(n+|E|).
    visit.reverse();
    visit
}

// Keep only the current entry for each unvisited vertex. Removing its former
// bucket entry preserves exactly the order obtained by skipping stale LIFO
// entries, while all bucket storage is O(n) instead of O(n + |E|).
fn mcs_peo(adj: &Completion, incumbent: &[usize], reverse_adj: bool) -> Vec<usize> {
    let n = adj.len();
    let none = u32::MAX;
    let mut head = vec![none; n + 1];
    let mut next = vec![none; n];
    let mut previous = vec![none; n];
    let mut weight = vec![0usize; n];
    let mut visited = vec![false; n];
    head[0] = incumbent[0] as u32;
    for pair in incumbent.windows(2) {
        next[pair[0]] = pair[1] as u32;
        previous[pair[1]] = pair[0] as u32;
    }
    let mut max_weight = 0usize;
    let mut visit = Vec::with_capacity(n);
    while visit.len() < n {
        while head[max_weight] == none && max_weight > 0 { max_weight -= 1; }
        let v = head[max_weight];
        if v == none { break; }
        let v = v as usize;
        let successor = next[v];
        head[max_weight] = successor;
        if successor != none { previous[successor as usize] = none; }
        visited[v] = true;
        visit.push(v);
        let neighbors = &adj[v];
        for offset in 0..neighbors.len() {
            let index = if reverse_adj { neighbors.len() - 1 - offset } else { offset };
            let u = neighbors[index] as usize;
            if visited[u] { continue; }
            let old_weight = weight[u];
            let predecessor = previous[u];
            let successor = next[u];
            if predecessor == none { head[old_weight] = successor; }
            else { next[predecessor as usize] = successor; }
            if successor != none { previous[successor as usize] = predecessor; }
            let new_weight = old_weight + 1;
            weight[u] = new_weight;
            let successor = head[new_weight];
            next[u] = successor;
            previous[u] = none;
            if successor != none { previous[successor as usize] = u as u32; }
            head[new_weight] = u as u32;
            max_weight = max_weight.max(new_weight);
        }
    }
    visit.reverse();
    visit
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{column_counts_gnp, flops_of, is_bijection,
        permute_pattern, EliminationTree, ScoringPattern};

    // Frozen former representation: independent child lists and per-column
    // reach buffers protect the append order, not just the final edge set.
    fn legacy_rows(n: usize, cp: &[usize], ri: &[usize], parent: &[Option<usize>],
        counts: &[usize], incumbent: &[usize]) -> Vec<Vec<u32>>
    {
        let mut children = vec![Vec::<usize>::new(); n];
        for (j, &p) in parent.iter().enumerate() {
            if let Some(p) = p { children[p].push(j); }
        }
        let mut columns = vec![Vec::<u32>::new(); n];
        let mut adj = vec![Vec::<u32>::new(); n];
        let mut mark = vec![usize::MAX; n];
        for j in 0..n {
            let mut reach = Vec::<u32>::with_capacity(counts[j] - 1);
            for &i in &ri[cp[j]..cp[j + 1]] {
                if i > j && mark[i] != j {
                    mark[i] = j;
                    reach.push(i as u32);
                }
            }
            for &child in &children[j] {
                for &i in &columns[child] {
                    let i = i as usize;
                    if i > j && mark[i] != j {
                        mark[i] = j;
                        reach.push(i as u32);
                    }
                }
                columns[child] = Vec::new();
            }
            assert_eq!(reach.len() + 1, counts[j]);
            let v = incumbent[j];
            for &i in &reach {
                let w = incumbent[i as usize];
                adj[v].push(w as u32);
                adj[w].push(v as u32);
            }
            columns[j] = reach;
        }
        adj
    }

    fn pattern(adj: &[Vec<bool>]) -> ScoringPattern {
        let mut cp = vec![0];
        let mut ri = Vec::new();
        for row in adj {
            ri.extend(row.iter().enumerate().filter_map(|(v, &edge)| edge.then_some(v)));
            cp.push(ri.len());
        }
        ScoringPattern { n: adj.len(), col_ptr: cp, row_idx: ri }
    }

    fn complete(adj: &[Vec<bool>], order: &[usize]) -> Vec<Vec<bool>> {
        let mut filled = adj.to_vec();
        let mut live = vec![true; adj.len()];
        for &v in order {
            let neighbors: Vec<_> = (0..adj.len()).filter(|&u| live[u] && filled[v][u]).collect();
            for &u in &neighbors {
                for &w in &neighbors {
                    if u != w { filled[u][w] = true; }
                }
            }
            live[v] = false;
        }
        filled
    }

    fn is_peo(adj: &[Vec<bool>], order: &[usize]) -> bool {
        if !is_bijection(order, adj.len()) { return false; }
        for (k, &v) in order.iter().enumerate() {
            let neighbors: Vec<_> = order[k + 1..].iter().copied().filter(|&u| adj[v][u]).collect();
            for &u in &neighbors {
                for &w in &neighbors {
                    if u != w && !adj[u][w] { return false; }
                }
            }
        }
        true
    }

    fn next_permutation(p: &mut [usize]) -> bool {
        let Some(i) = (0..p.len().saturating_sub(1)).rev().find(|&i| p[i] < p[i + 1]) else { return false; };
        let j = (i + 1..p.len()).rev().find(|&j| p[j] > p[i]).unwrap();
        p.swap(i, j);
        p[i + 1..].reverse();
        true
    }

    #[test]
    fn peo_extraction_exhaustive_graphs_and_orders() {
        set_indexed_priority(false);
        for n in 1..=5 {
            let edges: Vec<_> = (0..n).flat_map(|u| (u + 1..n).map(move |v| (u, v))).collect();
            for mask in 0..(1usize << edges.len()) {
                let mut graph = vec![vec![false; n]; n];
                for (bit, &(u, v)) in edges.iter().enumerate() {
                    if mask & (1 << bit) != 0 { graph[u][v] = true; graph[v][u] = true; }
                }
                let pat = pattern(&graph);
                let mut incumbent: Vec<_> = (0..n).collect();
                loop {
                    let pp = permute_pattern(&pat, &incumbent);
                    let et = EliminationTree::from_pattern(&pp);
                    let counts = column_counts_gnp(&pp, &et);
                    let filled = complete(&graph, &incumbent);
                    let reconstructed = reconstruct(n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &incumbent, MAX_N, MAX_INPUT_NNZ, MAX_LNNZ).unwrap();
                    let former = legacy_rows(n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &incumbent);
                    for v in 0..n {
                        assert_eq!(&reconstructed[v], former[v].as_slice());
                        let mut row = vec![false; n];
                        for &u in &reconstructed[v] { assert!(!row[u as usize]); row[u as usize] = true; }
                        assert_eq!(row, filled[v]);
                    }
                    let baseline = flops_of(&pat, &incumbent);
                    let mut best = baseline;
                    for reverse in [false, true] {
                        let candidate = mcs_peo(&reconstructed, &incumbent, reverse);
                        assert_eq!(candidate, mcs_peo_stale(&reconstructed, &incumbent, reverse));
                        assert!(is_peo(&filled, &candidate));
                        // Independent symbolic scorer, not MCS weights/counts.
                        let f = flops_of(&pat, &candidate);
                        best = best.min(f);
                        assert!(f <= baseline, "n={n}, mask={mask}, order={incumbent:?}");
                    }
                    let degrees: Vec<_> = graph.iter().map(|r| r.iter().filter(|&&e| e).count()).collect();
                    for policy in 0..12 {
                        let candidate = priority_mcs(&reconstructed, &incumbent, &counts, &degrees, policy);
                        assert_eq!(candidate,priority_reference::reference(&reconstructed,
                            &incumbent,&counts,&degrees,policy));
                        set_indexed_priority(true);
                        let indexed=priority_mcs(&reconstructed,&incumbent,&counts,&degrees,policy);
                        set_indexed_priority(false);
                        assert_eq!(candidate,indexed,"indexed priority={policy}");
                        assert!(is_peo(&filled, &candidate), "policy={policy}");
                        assert_eq!(candidate, priority_mcs(&reconstructed, &incumbent, &counts, &degrees, policy));
                        assert!(flops_of(&pat, &candidate) <= baseline,
                            "n={n}, mask={mask}, order={incumbent:?}, policy={policy}");
                    }
                    for seed in 0..3 {
                        let mut initial=incumbent.clone();
                        if seed==1 {initial.reverse();}
                        if seed==2 {initial.sort_by_key(|&v|std::cmp::Reverse(degrees[v]));}
                        for reverse in [false,true] {
                            let candidate=lex_bfs::order(&reconstructed,&initial,reverse);
                            assert_eq!(candidate,lex_bfs::reference(&reconstructed,&initial,reverse),"LexBFS seed={seed} reverse={reverse}");
                            assert!(is_peo(&filled,&candidate),"LexBFS seed={seed}");
                            assert!(flops_of(&pat,&candidate)<=baseline,"LexBFS n={n} mask={mask} seed={seed}");
                        }
                    }
                    assert!(best <= baseline);
                    if !next_permutation(&mut incumbent) { break; }
                }
            }
        }
        set_indexed_priority(true);
    }

    #[test]
    fn flat_completion_preserves_public_neighbor_order() {
        let mut cases = 0;
        for (name, pat) in crate::corpus::corpus() {
            if pat.n == 0 || pat.n > 50_000 || pat.nnz() > 1_200_000 { continue; }
            let sp = ScoringPattern { n: pat.n, col_ptr: pat.col_ptr.clone(), row_idx: pat.row_idx.clone() };
            for ticket in 0..2 {
                let incumbent = if ticket == 0 {
                    let cp: Vec<i32> = pat.col_ptr.iter().map(|&v| v as i32).collect();
                    let ri: Vec<i32> = pat.row_idx.iter().map(|&v| v as i32).collect();
                    let core = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
                    feral_amd::amd_order(&core).unwrap().into_iter().map(|v| v as usize).collect()
                } else { super::super::relabel(pat.n, ticket) };
                let pp = permute_pattern(&sp, &incumbent);
                let et = EliminationTree::from_pattern(&pp);
                let counts = column_counts_gnp(&pp, &et);
                let Some(flat) = reconstruct(pat.n, &pp.col_ptr, &pp.row_idx, &et.parent,
                    &counts, &incumbent, 50_000, 1_200_000, 4_000_000) else { continue; };
                let former = legacy_rows(pat.n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &incumbent);
                let mut scratch = pooled_reference::Scratch::default();
                let pooled = pooled_reference::candidates_with_scratch(pat.n,
                    &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &incumbent,
                    50_000, 1_200_000, 4_000_000, &mut scratch).unwrap();
                for v in 0..pat.n {
                    assert_eq!(&flat[v], former[v].as_slice(), "{name}, ticket={ticket}, vertex={v}");
                }
                for reverse in [false, true] {
                    assert_eq!(mcs_peo_stale(&flat, &incumbent, reverse),
                        mcs_peo(&flat, &incumbent, reverse), "{name}, ticket={ticket}");
                    assert_eq!(pooled[usize::from(reverse)],
                        mcs_peo(&flat, &incumbent, reverse), "{name}, pooled ticket={ticket}");
                }
                cases += 1;
            }
        }
        assert!(cases > 300);
        println!("FLAT_COMPLETION ordered_rows_preserved_cases={cases}");
    }

    #[test]
    #[ignore]
    fn probe_flat_completion_reconstruction() {
        use std::{hint::black_box, time::Instant};
        let mut old_total = 0.0;
        let mut new_total = 0.0;
        let mut cases = 0;
        for (name, pat) in crate::corpus::corpus() {
            if pat.n < 1_000 || pat.n > 50_000 || pat.nnz() > 1_200_000 { continue; }
            let sp = ScoringPattern { n: pat.n, col_ptr: pat.col_ptr.clone(), row_idx: pat.row_idx.clone() };
            let incumbent = super::super::order(&pat);
            let pp = permute_pattern(&sp, &incumbent);
            let et = EliminationTree::from_pattern(&pp);
            let counts = column_counts_gnp(&pp, &et);
            let Some(flat) = reconstruct(pat.n, &pp.col_ptr, &pp.row_idx, &et.parent,
                &counts, &incumbent, 50_000, 1_200_000, 4_000_000) else { continue; };
            let former = legacy_rows(pat.n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &incumbent);
            for v in 0..pat.n { assert_eq!(&flat[v], former[v].as_slice()); }
            drop(flat);
            drop(former);
            let mut elapsed = [f64::MAX; 2];
            for pair in 0..3 {
                for offset in 0..2 {
                    let variant = (pair + offset) % 2;
                    let t = Instant::now();
                    if variant == 0 {
                        black_box(legacy_rows(pat.n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &incumbent));
                    } else {
                        black_box(reconstruct(pat.n, &pp.col_ptr, &pp.row_idx, &et.parent,
                            &counts, &incumbent, 50_000, 1_200_000, 4_000_000).unwrap());
                    }
                    elapsed[variant] = elapsed[variant].min(t.elapsed().as_secs_f64());
                }
            }
            old_total += elapsed[0];
            new_total += elapsed[1];
            cases += 1;
            println!("FLAT_PAIR\t{name}\t{}\t{}\t{:.6}\t{:.6}", pat.n,
                counts.iter().sum::<usize>(), elapsed[0], elapsed[1]);
        }
        println!("FLAT_TOTAL cases={cases} old={old_total:.6} new={new_total:.6}");
    }

    #[test]
    #[ignore]
    fn probe_linked_mcs() {
        use std::{hint::black_box, time::Instant};
        let mut totals = [0.0; 2];
        for (name, pat) in crate::corpus::corpus() {
            if pat.n < 1_000 || pat.n > 50_000 || pat.nnz() > 1_200_000 { continue; }
            let cp: Vec<i32> = pat.col_ptr.iter().map(|&v| v as i32).collect();
            let ri: Vec<i32> = pat.row_idx.iter().map(|&v| v as i32).collect();
            let core = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
            let incumbent: Vec<usize> = feral_amd::amd_order(&core).unwrap().into_iter().map(|v| v as usize).collect();
            let sp = ScoringPattern { n: pat.n, col_ptr: pat.col_ptr.clone(), row_idx: pat.row_idx.clone() };
            let pp = permute_pattern(&sp, &incumbent);
            let et = EliminationTree::from_pattern(&pp);
            let counts = column_counts_gnp(&pp, &et);
            let Some(flat) = reconstruct(pat.n, &pp.col_ptr, &pp.row_idx, &et.parent,
                &counts, &incumbent, 50_000, 1_200_000, 4_000_000) else { continue; };
            let mut elapsed = [f64::MAX; 2];
            for reverse in [false, true] {
                assert_eq!(mcs_peo_stale(&flat, &incumbent, reverse), mcs_peo(&flat, &incumbent, reverse));
            }
            for pair in 0..3 {
                for offset in 0..2 {
                    let variant = (pair + offset) % 2;
                    let t = Instant::now();
                    for reverse in [false, true] {
                        let output = if variant == 0 { mcs_peo_stale(&flat, &incumbent, reverse) }
                            else { mcs_peo(&flat, &incumbent, reverse) };
                        black_box(output);
                    }
                    elapsed[variant] = elapsed[variant].min(t.elapsed().as_secs_f64());
                }
            }
            totals[0] += elapsed[0];
            totals[1] += elapsed[1];
            println!("MCS_PAIR\t{name}\t{}\t{}\t{:.6}\t{:.6}", pat.n,
                counts.iter().sum::<usize>(), elapsed[0], elapsed[1]);
        }
        println!("MCS_TOTAL old={:.6} linked={:.6}", totals[0], totals[1]);
    }

    #[test]
    fn peo_extraction_strict_path_witness_and_bad_inputs() {
        let graph = vec![vec![false, true, false], vec![true, false, true], vec![false, true, false]];
        let pat = pattern(&graph);
        let incumbent = vec![1, 0, 2];
        let pp = permute_pattern(&pat, &incumbent);
        let et = EliminationTree::from_pattern(&pp);
        let counts = column_counts_gnp(&pp, &et);
        let orders = candidates(3, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &incumbent).unwrap();
        assert_eq!(flops_of(&pat, &incumbent), 14);
        assert_eq!(orders.iter().map(|p| flops_of(&pat, p)).min(), Some(9));
        assert!(candidates(3, &pp.col_ptr, &pp.row_idx, &et.parent, &[1, 1, 1], &incumbent).is_none());
        assert!(candidates(3, &[0, 4, 2, 4], &pp.row_idx, &et.parent, &counts, &incumbent).is_none());
        assert!(candidates(3, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &[1, 1, 2]).is_none());
        assert!(candidates(3, &pp.col_ptr, &pp.row_idx, &[], &counts, &incumbent).is_none());
        assert!(candidates(MAX_N + 1, &[], &[], &[], &[], &[]).is_none());
    }
}
