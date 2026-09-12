//! Frozen priority MCS before the completed-visit hard stop. Test-only.
use super::*;

pub(super) fn reference(adj: &Completion, incumbent: &[usize], counts: &[usize],
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
    let mut heap = std::collections::BinaryHeap::with_capacity(n);
    for v in 0..n { heap.push((0usize, priority[v], v)); }
    let mut weights = vec![0usize; n];
    let mut visited = vec![false; n];
    let mut out = Vec::with_capacity(n);
    while let Some((weight, _, v)) = heap.pop() {
        if visited[v] || weight != weights[v] { continue; }
        visited[v] = true; out.push(v);
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

