//! Bounded chordal-completion exchange, accepted by the caller's exact scorer.
//! Neutral flips and bounded block insertion are independently exact-scored.

use super::minl::{repair_probe_graph, repair_probe_peo};
use super::minl_watch::watcher_minimalize;
use super::*;

// Validated against the current-frontier public screen34172289321.
const PRODUCTION_MAX_FLIPS: usize = 4096;

pub(super) fn refine(sp: &ScoringPattern, seed: &[usize]) -> Vec<Vec<usize>> {
    if !(16..35_000).contains(&sp.n) || sp.row_idx.len() >= 130_000 { return Vec::new(); }
    let Some(adj) = repair_probe_graph(sp, seed) else { return Vec::new(); };
    let original: Vec<Vec<u32>> = (0..sp.n).map(|u| {
        let mut row: Vec<_> = sp.row_idx[sp.col_ptr[u]..sp.col_ptr[u + 1]]
            .iter().copied().filter(|&v| v != u).map(|v| v as u32).collect();
        row.sort_unstable(); row.dedup(); row
    }).collect();
    // Both start from the same incumbent completion, matching the independent
    // public screen. Each has its own8M credits; total allowance is16M.
    let mut candidates = Vec::with_capacity(2);
    if let Some(result) = plateau_descent_bounded(&original, &adj, 8_000_000, PRODUCTION_MAX_FLIPS) {
        candidates.push(result.permutation);
    }
    if let Some(perm) = block_repair(&original, &adj, 8_000_000) { candidates.push(perm); }
    candidates
}

fn charge(ops: &mut i64, amount: usize) -> bool {
    *ops -= amount.min(i64::MAX as usize) as i64;
    *ops >= 0
}

fn common(adj: &[Vec<u32>], u: usize, v: usize, ops: &mut i64) -> Option<Vec<u32>> {
    if !charge(ops, adj[u].len() + adj[v].len()) { return None; }
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < adj[u].len() && j < adj[v].len() {
        match adj[u][i].cmp(&adj[v][j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => { out.push(adj[u][i]); i += 1; j += 1; }
        }
    }
    Some(out)
}

// H+xy is chordal iff the common neighbors separate x from y in chordal H.
// Fail closed on exhausted work; never test connectivity by vertex count alone.
#[cfg(test)]
fn insertable(adj: &[Vec<u32>], x: usize, y: usize, ops: &mut i64) -> Option<bool> {
    let cut = common(adj, x, y, ops)?;
    if !charge(ops, adj.len() + cut.len()) { return None; }
    let mut seen = vec![false; adj.len()];
    for &v in &cut { seen[v as usize] = true; }
    let mut stack = vec![x];
    seen[x] = true;
    while let Some(u) = stack.pop() {
        if !charge(ops, adj[u].len() + 1) { return None; }
        for &v in &adj[u] {
            let v = v as usize;
            if v == y { return Some(false); }
            if !seen[v] { seen[v] = true; stack.push(v); }
        }
    }
    Some(true)
}

fn chordal_cost(adj: &[Vec<u32>], perm: &[usize]) -> u64 {
    let mut rank = vec![0; adj.len()];
    for (i, &v) in perm.iter().enumerate() { rank[v] = i; }
    (0..adj.len()).map(|u| {
        let width = 1 + adj[u].iter().filter(|&&v| rank[v as usize] > rank[u]).count() as u64;
        width * width
    }).sum()
}

#[cfg(test)]
fn repair(original: &[Vec<u32>], adj: &[Vec<u32>], mut ops: i64) -> Option<Vec<usize>> {
    let n = adj.len();
    let entries: usize = adj.iter().map(Vec::len).sum();
    if !charge(&mut ops, 4 * (n + entries)) { return None; }
    let mut best = repair_probe_peo(adj);
    let initial = chordal_cost(adj, &best);
    let mut best_cost = initial;
    let mut fill = Vec::new();
    for u in 0..n {
        let depth = usize::BITS - original[u].len().max(1).leading_zeros();
        for &v in &adj[u] {
            if !charge(&mut ops, depth as usize) { return None; }
            if v as usize > u && original[u].binary_search(&v).is_err() {
                fill.push((u as u32, v));
            }
        }
    }
    // A bounded witness census, retaining at most 64 different missing edges.
    let mut pairs: Vec<(u32, u32, usize)> = Vec::new();
    let mut scan_ops = ops.min(1_000_000);
    let reserved = scan_ops;
    for &(u, v) in &fill {
        let Some(c) = common(adj, u as usize, v as usize, &mut scan_ops) else { break; };
        if c.len() > 256 { continue; }
        let mut missing = None;
        'pairs: for (i, &a) in c.iter().enumerate() {
            let row = &adj[a as usize];
            let depth = usize::BITS - row.len().max(1).leading_zeros();
            for &b in &c[i + 1..] {
                if !charge(&mut scan_ops, depth as usize) { break 'pairs; }
                if row.binary_search(&b).is_err() { missing = Some((a, b)); break 'pairs; }
            }
        }
        if scan_ops < 0 { break; }
        if let Some((x, y)) = missing {
            if let Some(p) = pairs.iter_mut().find(|p| p.0 == x && p.1 == y) { p.2 += 1; }
            else if pairs.len() < 64 { pairs.push((x, y, 1)); }
        }
    }
    ops -= reserved; // reserve the full census allowance, even on an early exit
    pairs.sort_unstable_by_key(|&(x, y, count)| (std::cmp::Reverse(count), x, y));
    for &(x, y, support) in pairs.iter().take(4) {
        if support < 2 { continue; }
        if insertable(adj, x as usize, y as usize, &mut ops) != Some(true) { continue; }
        let sort_depth = usize::BITS - fill.len().max(1).leading_zeros();
        if !charge(&mut ops, 8 * (n + entries) + fill.len() * sort_depth as usize) { break; }
        let allowance = ops.min(1_000_000);
        if allowance <= 0 { break; }
        ops -= allowance;
        let mut trial = adj.to_vec();
        for (u, v) in [(x, y), (y, x)] {
            let row = &mut trial[u as usize];
            let pos = row.binary_search(&v).expect_err("witness must be a nonedge");
            row.insert(pos, v);
        }
        let mut ids: Vec<u32> = (0..fill.len() as u32).collect();
        ids.sort_unstable_by_key(|&i| {
            let (u, v) = fill[i as usize];
            (trial[u as usize].len() + trial[v as usize].len(), i)
        });
        // Deliberately exclude the inserted edge from deletions in this trial.
        let result = watcher_minimalize(&mut trial, &fill, &ids, allowance, 1024);
        if result.removed < 2 { continue; }
        let perm = repair_probe_peo(&trial);
        let cost = chordal_cost(&trial, &perm);
        if cost < best_cost { best_cost = cost; best = perm; }
    }
    (best_cost < initial).then_some(best)
}

enum Defect { Clique, Single(u32, u32), Multiple }

// The maximal cliques containing uv form a connected clique-tree subtree.
// Merging that subtree saturates C=N(u) intersect N(v), preserving chordality.
// Unlike a single neutral flip, this can cross a positive-cost barrier.
fn block_repair(original: &[Vec<u32>], adj: &[Vec<u32>], mut ops: i64) -> Option<Vec<usize>> {
    let n = adj.len();
    let entries: usize = adj.iter().map(Vec::len).sum();
    if !charge(&mut ops, 4 * (n + entries)) { return None; }
    let initial_perm = repair_probe_peo(adj);
    let initial = chordal_cost(adj, &initial_perm);
    let mut best = initial;
    let mut answer = None;
    let mut fill = Vec::new();
    for u in 0..n {
        let depth = usize::BITS - original[u].len().max(1).leading_zeros();
        for &v in &adj[u] {
            if !charge(&mut ops, depth as usize) { return None; }
            if v as usize > u && original[u].binary_search(&v).is_err() {
                fill.push((u as u32, v));
            }
        }
    }
    let allowance = ops.min(1_000_000);
    ops -= allowance;
    let mut census = allowance;
    let mut proposals: Vec<(Vec<(u32, u32)>, usize)> = Vec::new();
    'census: for &(u, v) in &fill {
        let Some(c) = common(adj, u as usize, v as usize, &mut census) else { break; };
        if c.len() > 32 { continue; }
        let mut absent = Vec::new();
        for (i, &x) in c.iter().enumerate() {
            let row = &adj[x as usize];
            let depth = usize::BITS - row.len().max(1).leading_zeros();
            for &y in &c[i + 1..] {
                if !charge(&mut census, depth as usize) { break 'census; }
                if row.binary_search(&y).is_err() { absent.push((x, y)); }
                if absent.len() > 12 { continue 'census; }
            }
        }
        if absent.len() < 2 { continue; }
        if !charge(&mut census, proposals.len() * absent.len()) { break; }
        if let Some(p) = proposals.iter_mut().find(|p| p.0 == absent) { p.1 += 1; }
        else if proposals.len() < 64 { proposals.push((absent, 1)); }
    }
    // Support minus inserted edges is a proposal heuristic, not a bound.
    proposals.sort_unstable_by_key(|(edges, support)|
        (std::cmp::Reverse(*support as i64 - edges.len() as i64), edges.clone()));
    for (edges, support) in proposals.into_iter().take(4) {
        if support <= edges.len() { continue; }
        let depth = usize::BITS - fill.len().max(1).leading_zeros();
        let mutation_work: usize = edges.iter().map(|&(u, v)|
            adj[u as usize].len() + adj[v as usize].len() + 2 * edges.len()).sum();
        if !charge(&mut ops, 8 * (n + entries) + fill.len() * depth as usize + mutation_work) { break; }
        let allowance = ops.min(1_000_000);
        if allowance <= 0 { break; }
        ops -= allowance;
        let mut trial = adj.to_vec();
        for (u, v) in edges {
            for (a, b) in [(u, v), (v, u)] {
                let row = &mut trial[a as usize];
                let pos = row.binary_search(&b).expect_err("block fill must be absent");
                row.insert(pos, b);
            }
        }
        let mut ids: Vec<u32> = (0..fill.len() as u32).collect();
        ids.sort_unstable_by_key(|&id| {
            let (u, v) = fill[id as usize];
            (trial[u as usize].len() + trial[v as usize].len(), id)
        });
        // Protect inserted block edges during this trial; only old fill may go.
        watcher_minimalize(&mut trial, &fill, &ids, allowance, 1024);
        let perm = repair_probe_peo(&trial);
        let cost = chordal_cost(&trial, &perm);
        if cost < best { best = cost; answer = Some(perm); }
    }
    answer
}

fn defect(adj: &[Vec<u32>], c: &[u32], marks: &mut [u32], epoch: u32, ops: &mut i64) -> Option<Defect> {
    if !charge(ops, c.len()) { return None; }
    for &v in c { marks[v as usize] = epoch; }
    let mut missing = None;
    for (i, &a) in c.iter().enumerate() {
        let row = &adj[a as usize];
        if !charge(ops, row.len() + 1) { return None; }
        let present = row.iter().filter(|&&v| v > a && marks[v as usize] == epoch).count();
        let absent = c.len() - i - 1 - present;
        if absent > 1 || (absent == 1 && missing.is_some()) { return Some(Defect::Multiple); }
        if absent == 1 {
            let depth = usize::BITS - row.len().max(1).leading_zeros();
            for &b in &c[i + 1..] {
                if !charge(ops, depth as usize) { return None; }
                if row.binary_search(&b).is_err() { missing = Some((a, b)); break; }
            }
        }
    }
    Some(match missing { Some((x, y)) => Defect::Single(x, y), None => Defect::Clique })
}

struct PlateauResult {
    graph: Vec<Vec<u32>>,
    permutation: Vec<usize>,
    flips: usize,
    deletion_gain: u64,
}

// If C=N(u) intersect N(v) is a clique minus xy, H-uv+xy is chordal
// and has exactly the same F. Then ordinary fill deletions strictly lower F.
// The removed uv must be a fill edge; original pattern edges are immutable.
#[cfg(test)]
fn plateau_descent(original: &[Vec<u32>], adj: &[Vec<u32>], ops: i64) -> Option<PlateauResult> {
    plateau_descent_bounded(original, adj, ops, 64)
}

fn plateau_descent_bounded(original: &[Vec<u32>], adj: &[Vec<u32>], mut ops: i64, max_flips: usize) -> Option<PlateauResult> {
    let n = adj.len();
    let entries: usize = adj.iter().map(Vec::len).sum();
    // Reserve cloning, queue/mark storage and final MCS before scanning.
    if !charge(&mut ops, 8 * (n + entries)) { return None; }
    let mut fill = Vec::new();
    for u in 0..n {
        let depth = usize::BITS - original[u].len().max(1).leading_zeros();
        for &v in &adj[u] {
            if !charge(&mut ops, depth as usize) { return None; }
            if v as usize > u && original[u].binary_search(&v).is_err() {
                fill.push((u as u32, v));
            }
        }
    }
    let mut trial = adj.to_vec();
    let mut alive = vec![true; fill.len()];
    let mut marks = vec![0u32; n];
    let mut epoch = 0u32;
    let mut taboo = std::collections::BTreeSet::new();
    let mut flips = 0;
    let mut deletion_gain = 0;
    'rounds: for _ in 0..2 {
        let mut changed = false;
        let mut i = 0;
        while i < fill.len() {
            let id = i; i += 1;
            if !charge(&mut ops, 1) { break 'rounds; }
            if !alive[id] { continue; }
            let (u, v) = fill[id];
            let Some(c) = common(&trial, u as usize, v as usize, &mut ops) else { break 'rounds; };
            if c.len() > 256 { continue; }
            epoch = epoch.wrapping_add(1);
            if epoch == 0 {
                if !charge(&mut ops, n) { break 'rounds; }
                marks.fill(0); epoch = 1;
            }
            let Some(kind) = defect(&trial, &c, &mut marks, epoch, &mut ops) else { break 'rounds; };
            let insert = match kind {
                Defect::Multiple => continue,
                Defect::Clique => None,
                Defect::Single(x, y) => {
                    if flips >= max_flips || taboo.contains(&(x, y)) { continue; }
                    Some((x, y))
                }
            };
            let mut mutation_work = trial[u as usize].len() + trial[v as usize].len() + 16;
            if let Some((x, y)) = insert { mutation_work += trial[x as usize].len() + trial[y as usize].len() + 16; }
            if !charge(&mut ops, mutation_work) { break 'rounds; }
            // All certificate checks finish before an atomic graph transaction.
            for (a, b) in [(u, v), (v, u)] {
                let row = &mut trial[a as usize];
                let pos = row.binary_search(&b).expect("alive fill edge missing");
                row.remove(pos);
            }
            alive[id] = false;
            if let Some((x, y)) = insert {
                for (a, b) in [(x, y), (y, x)] {
                    let row = &mut trial[a as usize];
                    let pos = row.binary_search(&b).expect_err("single defect must be missing");
                    row.insert(pos, b);
                }
                taboo.insert((u, v));
                fill.push((x, y)); alive.push(true);
                flips += 1;
            } else { deletion_gain += 3 + 2 * c.len() as u64; }
            changed = true;
        }
        if !changed { break; }
    }
    if flips == 0 && deletion_gain == 0 { return None; }
    let permutation = repair_probe_peo(&trial);
    Some(PlateauResult { graph: trial, permutation, flips, deletion_gain })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adjacency(n: usize, edges: &[(usize, usize)]) -> Vec<Vec<u32>> {
        let mut adj = vec![Vec::new(); n];
        for &(u, v) in edges { adj[u].push(v as u32); adj[v].push(u as u32); }
        for row in &mut adj { row.sort_unstable(); row.dedup(); }
        adj
    }

    fn chordal(adj: &[Vec<u32>]) -> bool {
        let perm = repair_probe_peo(adj);
        let mut rank = vec![0; adj.len()];
        for (i, &v) in perm.iter().enumerate() { rank[v] = i; }
        for u in 0..adj.len() {
            let c: Vec<_> = adj[u].iter().copied().filter(|&v| rank[v as usize] > rank[u]).collect();
            for (i, &v) in c.iter().enumerate() {
                for &w in &c[i + 1..] { if adj[v as usize].binary_search(&w).is_err() { return false; } }
            }
        }
        true
    }

    #[test]
    fn one_insertion_unlocks_multiple_deletions() {
        let edges: Vec<_> = (0..4).flat_map(|u| [(u, 4), (u, 5)]).collect();
        let original = adjacency(6, &edges);
        let mut filled_edges = edges.clone();
        for u in 0..4 { for v in u + 1..4 { filled_edges.push((u, v)); } }
        let filled = adjacency(6, &filled_edges);
        assert_eq!(chordal_cost(&filled, &repair_probe_peo(&filled)), 80);
        let perm = repair(&original, &filled, 8_000_000).expect("separator exchange must improve");
        let pat = crate::Pattern::from_edges(6, &edges);
        let sp = ScoringPattern { n: 6, col_ptr: pat.col_ptr.clone(), row_idx: pat.row_idx.clone() };
        let realized = repair_probe_graph(&sp, &perm).unwrap();
        assert!(chordal(&realized));
        assert_eq!(chordal_cost(&realized, &perm), 41);
        assert!(repair(&original, &filled, 0).is_none());
    }

    #[test]
    fn separator_certificate_matches_exhaustive_chordality() {
        let edges: Vec<_> = (0..5).flat_map(|u| (u + 1..5).map(move |v| (u, v))).collect();
        let mut checked = 0;
        for mask in 0usize..1 << edges.len() {
            let selected: Vec<_> = edges.iter().enumerate().filter_map(|(i, &e)|
                (mask & (1 << i) != 0).then_some(e)).collect();
            let adj = adjacency(5, &selected);
            if !chordal(&adj) { continue; }
            for (i, &(u, v)) in edges.iter().enumerate() {
                if mask & (1 << i) != 0 { continue; }
                let mut augmented = selected.clone(); augmented.push((u, v));
                assert_eq!(insertable(&adj, u, v, &mut 100_000), Some(chordal(&adjacency(5, &augmented))));
                checked += 1;
            }
        }
        println!("SEPARATOR_CERTIFICATE checked={checked}");
        assert!(checked > 1000);
    }

    #[test]
    fn a_block_exchange_crosses_a_positive_cost_barrier() {
        let edges: Vec<_> = (0..4).flat_map(|u| (4..7).map(move |v| (u, v))).collect();
        let original = adjacency(7, &edges);
        let mut filled_edges = edges.clone();
        for u in 0..4 { for v in u + 1..4 { filled_edges.push((u, v)); } }
        let filled = adjacency(7, &filled_edges);
        assert_eq!(chordal_cost(&filled, &repair_probe_peo(&filled)), 105);
        assert!(plateau_descent(&original, &filled, 8_000_000).is_none());
        let perm = block_repair(&original, &filled, 8_000_000).expect("block switch must improve");
        let pat = crate::Pattern::from_edges(7, &edges);
        let sp = ScoringPattern { n: 7, col_ptr: pat.col_ptr, row_idx: pat.row_idx };
        assert_eq!(flops_of(&sp, &perm), 78);
        assert!(block_repair(&original, &filled, 0).is_none());
    }

    #[test]
    fn production_wrapper_is_deterministic_and_nonincreasing() {
        for n in [17, 32, 65] {
            let edges: Vec<_> = (0..n).flat_map(|u|
                [(u, (u + 1) % n), (u, (u + 3) % n), (u, (u * 7 + 5) % n)])
                .filter(|&(u, v)| u != v).collect();
            let pat = crate::Pattern::from_edges(n, &edges);
            let sp = ScoringPattern { n, col_ptr: pat.col_ptr, row_idx: pat.row_idx };
            for seed in [(0..n).collect::<Vec<_>>(), (0..n).rev().collect()] {
                let first = refine(&sp, &seed);
                assert_eq!(first, refine(&sp, &seed));
                for perm in first {
                    assert!(is_bijection(&perm, n));
                    assert!(flops_of(&sp, &perm) <= flops_of(&sp, &seed));
                }
            }
        }
        for n in [0, 15, 35_000] {
            let sp = ScoringPattern { n, col_ptr: vec![0; n + 1], row_idx: Vec::new() };
            assert!(refine(&sp, &(0..n).collect::<Vec<_>>()).is_empty());
        }
    }

    #[test]
    fn block_saturation_preserves_chordality_exhaustively() {
        let edges: Vec<_> = (0..5).flat_map(|u| (u + 1..5).map(move |v| (u, v))).collect();
        let mut checked = 0;
        for mask in 0usize..1 << edges.len() {
            let selected: Vec<_> = edges.iter().enumerate().filter_map(|(i, &e)|
                (mask & (1 << i) != 0).then_some(e)).collect();
            let adj = adjacency(5, &selected);
            if !chordal(&adj) { continue; }
            for &(u, v) in &selected {
                let c = common(&adj, u, v, &mut 100_000).unwrap();
                let mut augmented = selected.clone();
                for (i, &x) in c.iter().enumerate() {
                    for &y in &c[i + 1..] { augmented.push((x as usize, y as usize)); }
                }
                assert!(chordal(&adjacency(5, &augmented)));
                checked += 1;
            }
        }
        println!("BLOCK_CERTIFICATE checked={checked}");
        assert!(checked > 1000);
    }

    #[test]
    fn plateau_transactions_preserve_chordality_and_exact_objective() {
        let edges: Vec<_> = (0..5).flat_map(|u| (u + 1..5).map(move |v| (u, v))).collect();
        let mut checked = 0;
        let mut flipped = 0;
        for mask in 0usize..1 << edges.len() {
            let selected: Vec<_> = edges.iter().enumerate().filter_map(|(i, &e)|
                (mask & (1 << i) != 0).then_some(e)).collect();
            let adj = adjacency(5, &selected);
            if !chordal(&adj) { continue; }
            let old = chordal_cost(&adj, &repair_probe_peo(&adj));
            // Every choice of one or two designated fill edges.
            for i in 0..selected.len() {
                for j in i..selected.len() {
                    let raw: Vec<_> = selected.iter().enumerate().filter_map(|(k, &e)|
                        (k != i && k != j).then_some(e)).collect();
                    let original = adjacency(5, &raw);
                    if let Some(result) = plateau_descent(&original, &adj, 8_000_000) {
                        assert!(chordal(&result.graph));
                        assert!(is_bijection(&result.permutation, 5));
                        assert_eq!(old - chordal_cost(&result.graph, &result.permutation), result.deletion_gain);
                        for &(u, v) in &raw { assert!(result.graph[u].binary_search(&(v as u32)).is_ok()); }
                        flipped += result.flips;
                    }
                    checked += 1;
                }
            }
        }
        println!("PLATEAU_CERTIFICATE checked={checked} flips={flipped}");
        assert!(checked > 10_000 && flipped > 0);
    }

    #[test]
    #[ignore]
    fn probe_public_separator_repair() {
        let mut before = [0.0f64; 3];
        let mut after = [0.0f64; 3];
        let mut after_long = [0.0f64; 3];
        let mut after_block = [0.0f64; 3];
        let mut counts = [0usize; 3];
        let mut wins = 0;
        let mut worst = 0.0f64;
        for (name, pat) in crate::corpus::corpus() {
            let sp = ScoringPattern { n: pat.n, col_ptr: pat.col_ptr.clone(), row_idx: pat.row_idx.clone() };
            let cp: Vec<i32> = pat.col_ptr.iter().map(|&v| v as i32).collect();
            let ri: Vec<i32> = pat.row_idx.iter().map(|&v| v as i32).collect();
            let core = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
            let amd: Vec<usize> = feral_amd::amd_order(&core).unwrap().into_iter().map(|v| v as usize).collect();
            let reference = flops_of(&sp, &amd);
            let seed = order(&pat);
            let old = flops_of(&sp, &seed);
            let mut new = old;
            let mut long = old;
            let mut block = old;
            let start = std::time::Instant::now();
            if (16..35_000).contains(&pat.n) && pat.nnz() < 130_000 {
                if let Some(adj) = repair_probe_graph(&sp, &seed) {
                    let original: Vec<Vec<u32>> = (0..pat.n).map(|u| {
                        let mut row: Vec<_> = pat.row_idx[pat.col_ptr[u]..pat.col_ptr[u + 1]]
                            .iter().copied().filter(|&v| v != u).map(|v| v as u32).collect();
                        row.sort_unstable(); row.dedup(); row
                    }).collect();
                    if let Some(result) = plateau_descent(&original, &adj, 8_000_000) {
                        assert!(is_bijection(&result.permutation, pat.n));
                        let completion = chordal_cost(&result.graph, &result.permutation);
                        assert_eq!(old.checked_sub(completion), Some(result.deletion_gain),
                            "completion objective identity failed for {name}");
                        let realized = flops_of(&sp, &result.permutation);
                        assert!(realized <= completion,
                            "original-pattern realization exceeds completion for {name}");
                        println!("PLATEAU\t{name}\t{}\t{}\t{completion}\t{realized}",
                            result.flips, result.deletion_gain);
                        new = realized;
                    }
                    let long_start = std::time::Instant::now();
                    if let Some(result) = plateau_descent_bounded(&original, &adj, 8_000_000, 4096) {
                        assert!(is_bijection(&result.permutation, pat.n));
                        let completion = chordal_cost(&result.graph, &result.permutation);
                        assert_eq!(old.checked_sub(completion), Some(result.deletion_gain),
                            "long completion objective identity failed for {name}");
                        long = flops_of(&sp, &result.permutation);
                        assert!(long <= completion,
                            "long original realization exceeds completion for {name}");
                        println!("LONG_PLATEAU\t{name}\t{}\t{}\t{completion}\t{long}\t{:.6}",
                            result.flips, result.deletion_gain, long_start.elapsed().as_secs_f64());
                    }
                    let block_start = std::time::Instant::now();
                    if let Some(perm) = block_repair(&original, &adj, 8_000_000) {
                        assert!(is_bijection(&perm, pat.n));
                        block = flops_of(&sp, &perm);
                        assert!(block < old, "certified block gain did not realize for {name}");
                    }
                    println!("BLOCK_TIME\t{name}\t{:.6}", block_start.elapsed().as_secs_f64());
                }
            }
            let seconds = start.elapsed().as_secs_f64();
            worst = worst.max(seconds);
            wins += usize::from(new < old);
            println!("REPAIR\t{name}\t{}\t{}\t{reference}\t{old}\t{new}\t{seconds:.6}", pat.n, pat.nnz());
            println!("LONG_REPAIR\t{name}\t{}\t{}\t{reference}\t{old}\t{long}", pat.n, pat.nnz());
            println!("BLOCK_REPAIR\t{name}\t{}\t{}\t{reference}\t{old}\t{block}", pat.n, pat.nnz());
            let b = if pat.n < 1000 { 0 } else if pat.n < 10_000 { 1 } else { 2 };
            before[b] += (old as f64 / reference as f64).ln();
            after[b] += (new as f64 / reference as f64).ln();
            after_long[b] += (long as f64 / reference as f64).ln();
            after_block[b] += (block as f64 / reference as f64).ln();
            counts[b] += 1;
        }
        let score = |logs: &[f64; 3]| -> f64 {
            [0.3, 0.3, 0.4].iter().enumerate()
                .map(|(b, &w)| w * (logs[b] / counts[b] as f64).exp()).sum()
        };
        println!("REPAIR_SUMMARY before={:.9} after={:.9} wins={wins} worst_extra={worst:.6}", score(&before), score(&after));
        println!("LONG_REPAIR_SUMMARY after={:.9}", score(&after_long));
        println!("BLOCK_REPAIR_SUMMARY after={:.9}", score(&after_block));
    }
}
