//! Deterministic graph-based seed orderings.
use super::Pattern;

pub(super) fn minfill_order(pattern: &Pattern) -> Vec<i32> {
    let n = pattern.n;

    // Symmetric adjacency lists + O(1) membership matrix (self-loops excluded,
    // duplicates suppressed via the membership check).
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut adjm: Vec<bool> = vec![false; n * n];
    for j in 0..n {
        let start = pattern.col_ptr[j];
        let end = pattern.col_ptr[j + 1];
        for &i in &pattern.row_idx[start..end] {
            if i != j && i < n && !adjm[j * n + i] {
                adjm[j * n + i] = true;
                adjm[i * n + j] = true;
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }

    let mut eliminated: Vec<bool> = vec![false; n];
    let mut order: Vec<usize> = Vec::with_capacity(n);

    // Hard pair-check budget: caps total deficiency-scan work so the running
    // time is bounded regardless of input structure.
    let mut budget: i64 = 40_000_000;
    let mut fell_back = false;

    for _ in 0..n {
        if budget < 0 {
            fell_back = true;
            break;
        }

        // Find the live vertex of minimum deficiency (ties → min degree → min
        // index). Scanning `0..n` ascending with strict-improvement replacement
        // keeps the lowest-index winner → deterministic.
        let mut best = usize::MAX;
        let mut best_def = i64::MAX;
        let mut best_deg = usize::MAX;
        for v in 0..n {
            if eliminated[v] {
                continue;
            }
            let nb = &adj[v];
            let deg = nb.len();
            let mut def: i64 = 0;
            for a in 0..deg {
                let base = nb[a] * n;
                for b in (a + 1)..deg {
                    if !adjm[base + nb[b]] {
                        def += 1;
                    }
                }
            }
            // Charge the inner pair work against the budget.
            budget -= (deg as i64 * deg as i64) / 2 + 1;
            if def < best_def || (def == best_def && deg < best_deg) {
                best_def = def;
                best_deg = deg;
                best = v;
            }
        }

        if best == usize::MAX {
            break; // no live vertices left
        }

        // Eliminate `best`: clique its neighborhood (insert new fill edges),
        // then unlink it from every neighbor.
        order.push(best);
        eliminated[best] = true;
        let nbrs = std::mem::take(&mut adj[best]);

        for a in 0..nbrs.len() {
            let x = nbrs[a];
            for b in (a + 1)..nbrs.len() {
                let y = nbrs[b];
                if !adjm[x * n + y] {
                    adjm[x * n + y] = true;
                    adjm[y * n + x] = true;
                    adj[x].push(y);
                    adj[y].push(x);
                }
            }
        }
        for &x in &nbrs {
            adjm[x * n + best] = false;
            adjm[best * n + x] = false;
            if let Some(pos) = adj[x].iter().position(|&z| z == best) {
                adj[x].swap_remove(pos);
            }
        }
    }

    if fell_back {
        // Budget exhausted: append remaining live vertices in ascending
        // current-degree order (ties by index) → still a valid bijection.
        let mut rest: Vec<usize> = (0..n).filter(|&v| !eliminated[v]).collect();
        rest.sort_by(|&a, &b| adj[a].len().cmp(&adj[b].len()).then_with(|| a.cmp(&b)));
        for v in rest {
            order.push(v);
        }
    }

    order.into_iter().map(|x| x as i32).collect()
}

fn subset_gain(v: usize, adj: &[Vec<usize>], in_sub: &[bool], ina: &[bool]) -> i64 {
    let mut g_a = 0i64;
    let mut g_s = 0i64;
    for &w in &adj[v] {
        if in_sub[w] {
            g_s += 1;
            if ina[w] {
                g_a += 1;
            }
        }
    }
    2 * g_a - g_s
}

pub(super) fn ndfm_order(pattern: &Pattern) -> Vec<i32> {
    let n = pattern.n;

    // Symmetric adjacency (exclude self-loops; dedup for accurate degrees).
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for j in 0..n {
        let start = pattern.col_ptr[j];
        let end = pattern.col_ptr[j + 1];
        for &i in &pattern.row_idx[start..end] {
            if i != j && i < n {
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }
    for a in adj.iter_mut() {
        a.sort_unstable();
        a.dedup();
    }
    let degree: Vec<usize> = adj.iter().map(|a| a.len()).collect();

    const NDFM_LEAF: usize = 100;

    let mut order: Vec<usize> = vec![0usize; n];
    let mut in_sub: Vec<bool> = vec![false; n]; // membership in the current subset
    let mut ina: Vec<bool> = vec![false; n]; // membership in the growing part A
    let mut dist: Vec<u32> = vec![0u32; n]; // BFS distance / separator marker scratch
    let mut bfs: Vec<usize> = Vec::new();

    // Hard work budget: caps total per-subset scanning at O(n log n).
    let mut budget: i64 = 96 * n as i64 + 8192;

    // Fill `order[lo..lo+v.len()]` with `v` reordered by ascending degree
    // (min-degree-ish leaf ordering), ties broken by index → deterministic.
    let deg_fill = |order: &mut [usize], lo: usize, mut v: Vec<usize>| {
        v.sort_by(|&a, &b| degree[a].cmp(&degree[b]).then_with(|| a.cmp(&b)));
        for (t, u) in v.into_iter().enumerate() {
            order[lo + t] = u;
        }
    };

    // Explicit task stack: (nodes, lo, hi) with hi-lo == nodes.len(). A task's
    // separator is placed at the TOP of its range (eliminated last).
    let mut stack: Vec<(Vec<usize>, usize, usize)> = Vec::new();
    stack.push(((0..n).collect(), 0, n));

    while let Some((nodes, lo, _hi)) = stack.pop() {
        let sz = nodes.len();
        let hi = lo + sz;

        // Base case / budget exhausted: order this subset by degree and stop.
        if sz <= NDFM_LEAF || budget < 0 {
            deg_fill(&mut order, lo, nodes);
            continue;
        }
        budget -= sz as i64;

        // Mark subset membership.
        for &u in &nodes {
            in_sub[u] = true;
        }

        // Deterministic seed: minimum-degree node in the subset (ties → lowest
        // index), then ONE pseudo-peripheral refinement (jump to the min-degree
        // node in the deepest BFS level within the subset).
        let mut start = nodes[0];
        {
            let mut start_deg = degree[start];
            for &u in &nodes {
                if degree[u] < start_deg {
                    start_deg = degree[u];
                    start = u;
                }
            }
            bfs.clear();
            bfs.push(start);
            dist[start] = 1;
            let mut head = 0;
            let mut maxd = 1u32;
            while head < bfs.len() {
                let u = bfs[head];
                head += 1;
                let d = dist[u];
                if d > maxd {
                    maxd = d;
                }
                for &vtx in &adj[u] {
                    if in_sub[vtx] && dist[vtx] == 0 {
                        dist[vtx] = d + 1;
                        bfs.push(vtx);
                    }
                }
            }
            let mut cand = start;
            let mut cand_deg = usize::MAX;
            for &u in &bfs {
                if dist[u] == maxd && degree[u] < cand_deg {
                    cand_deg = degree[u];
                    cand = u;
                }
            }
            for &u in &bfs {
                dist[u] = 0;
            }
            start = cand;
        }

        let target = (sz + 1) / 2;
        let mut a_list: Vec<usize> = Vec::new();
        ina[start] = true;
        a_list.push(start);
        let mut heap: std::collections::BinaryHeap<(i64, isize)> =
            std::collections::BinaryHeap::new();
        for &w in &adj[start] {
            if in_sub[w] && !ina[w] {
                heap.push((subset_gain(w, &adj, &in_sub, &ina), -(w as isize)));
            }
        }
        while a_list.len() < target {
            let Some((g, neg_w)) = heap.pop() else {
                break; // frontier exhausted (subset locally disconnected)
            };
            let w = (-neg_w) as usize;
            if ina[w] {
                continue; // already absorbed
            }
            let gc = subset_gain(w, &adj, &in_sub, &ina);
            if gc != g {
                heap.push((gc, neg_w)); // stale snapshot; re-insert corrected
                continue;
            }
            ina[w] = true;
            a_list.push(w);
            for &x in &adj[w] {
                if in_sub[x] && !ina[x] {
                    heap.push((subset_gain(x, &adj, &in_sub, &ina), -(x as isize)));
                }
            }
        }

        // Compute the two edge-cut boundaries (scanning `nodes` in ascending
        // order → deterministic lists). boundary_a = A-vertices with a neighbor
        // in B; boundary_b = B-vertices with a neighbor in A.
        let mut boundary_a: Vec<usize> = Vec::new();
        let mut boundary_b: Vec<usize> = Vec::new();
        for &u in &nodes {
            if ina[u] {
                if adj[u].iter().any(|&w| in_sub[w] && !ina[w]) {
                    boundary_a.push(u);
                }
            } else if adj[u].iter().any(|&w| in_sub[w] && ina[w]) {
                boundary_b.push(u);
            }
        }

        // Take the SMALLER boundary as the vertex separator (ties → A-side).
        // Removing it disconnects the two subdomains.
        let use_a = boundary_a.len() <= boundary_b.len();
        let sep: Vec<usize> = if use_a { boundary_a } else { boundary_b };

        // Mark separator vertices (reuse `dist` as a 0/1 flag), then split the
        // remaining subset into the two subdomains by A-membership.
        for &u in &sep {
            dist[u] = 1;
        }
        let mut left: Vec<usize> = Vec::new();
        let mut right: Vec<usize> = Vec::new();
        for &u in &nodes {
            if dist[u] == 1 {
                continue; // separator
            }
            if ina[u] {
                left.push(u);
            } else {
                right.push(u);
            }
        }

        // Reset all scratch for reuse.
        for &u in &sep {
            dist[u] = 0;
        }
        for &u in &a_list {
            ina[u] = false;
        }
        for &u in &nodes {
            in_sub[u] = false;
        }

        // Degenerate: separator is the whole subset — degree-order and stop.
        if left.is_empty() && right.is_empty() {
            deg_fill(&mut order, lo, sep);
            continue;
        }

        // Separator at the TOP of the range (eliminated last); subdomains below.
        let sep_len = sep.len();
        let sep_start = hi - sep_len;
        for (t, u) in sep.iter().enumerate() {
            order[sep_start + t] = *u;
        }

        let left_len = left.len();
        if !left.is_empty() {
            stack.push((left, lo, lo + left_len));
        }
        if !right.is_empty() {
            stack.push((right, lo + left_len, sep_start));
        }
    }

    order.into_iter().map(|x| x as i32).collect()
}

pub(super) fn nd_order(pattern: &Pattern) -> Vec<i32> {
    let n = pattern.n;

    // Symmetric adjacency (exclude self-loops; dedup for accurate degrees).
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for j in 0..n {
        let start = pattern.col_ptr[j];
        let end = pattern.col_ptr[j + 1];
        for &i in &pattern.row_idx[start..end] {
            if i != j && i < n {
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }
    for a in adj.iter_mut() {
        a.sort_unstable();
        a.dedup();
    }
    let degree: Vec<usize> = adj.iter().map(|a| a.len()).collect();

    const ND_LEAF: usize = 200;

    let mut order: Vec<usize> = vec![0usize; n];
    let mut mark: Vec<bool> = vec![false; n]; // membership in the current subset
    let mut dist: Vec<u32> = vec![0u32; n]; // 1-based BFS distance; 0 = unvisited
    let mut bfs: Vec<usize> = Vec::new();

    // Hard work budget: caps total per-subset scanning at O(n), so no adversarial
    // (e.g. highly disconnected) input can drive quadratic blow-up.
    let mut budget: i64 = 64 * n as i64 + 4096;

    // Fill `order[lo..lo+v.len()]` with `v` reordered by ascending degree
    // (min-degree-ish leaf ordering), ties broken by index → deterministic.
    let deg_fill = |order: &mut [usize], lo: usize, mut v: Vec<usize>| {
        v.sort_by(|&a, &b| degree[a].cmp(&degree[b]).then_with(|| a.cmp(&b)));
        for (t, u) in v.into_iter().enumerate() {
            order[lo + t] = u;
        }
    };

    // Explicit task stack: (nodes, lo, hi) with hi-lo == nodes.len(). A task's
    // separator is placed at the TOP of its range (eliminated last).
    let mut stack: Vec<(Vec<usize>, usize, usize)> = Vec::new();
    stack.push(((0..n).collect(), 0, n));

    while let Some((nodes, lo, _hi)) = stack.pop() {
        let sz = nodes.len();
        let hi = lo + sz;

        // Base case / budget exhausted: order this subset by degree and stop.
        if sz <= ND_LEAF || budget < 0 {
            deg_fill(&mut order, lo, nodes);
            continue;
        }
        budget -= sz as i64;

        // Mark subset membership.
        for &u in &nodes {
            mark[u] = true;
        }

        // Deterministic seed: minimum-degree node in the subset (ties → lowest
        // index), then ONE pseudo-peripheral refinement (jump to the min-degree
        // node in the deepest BFS level).
        let mut start = nodes[0];
        {
            let mut start_deg = degree[start];
            for &u in &nodes {
                if degree[u] < start_deg {
                    start_deg = degree[u];
                    start = u;
                }
            }
            bfs.clear();
            bfs.push(start);
            dist[start] = 1;
            let mut head = 0;
            let mut maxd = 1u32;
            while head < bfs.len() {
                let u = bfs[head];
                head += 1;
                let d = dist[u];
                if d > maxd {
                    maxd = d;
                }
                for &vtx in &adj[u] {
                    if mark[vtx] && dist[vtx] == 0 {
                        dist[vtx] = d + 1;
                        bfs.push(vtx);
                    }
                }
            }
            let mut cand = start;
            let mut cand_deg = usize::MAX;
            for &u in &bfs {
                if dist[u] == maxd && degree[u] < cand_deg {
                    cand_deg = degree[u];
                    cand = u;
                }
            }
            for &u in &bfs {
                dist[u] = 0;
            }
            start = cand;
        }

        // BFS from the refined start over the subset.
        bfs.clear();
        bfs.push(start);
        dist[start] = 1;
        let mut head = 0;
        let mut maxd = 1u32;
        while head < bfs.len() {
            let u = bfs[head];
            head += 1;
            let d = dist[u];
            if d > maxd {
                maxd = d;
            }
            for &vtx in &adj[u] {
                if mark[vtx] && dist[vtx] == 0 {
                    dist[vtx] = d + 1;
                    bfs.push(vtx);
                }
            }
        }
        let reached = bfs.len();

        // Median-level separator over the reached component.
        let mut level_count = vec![0usize; (maxd as usize) + 1];
        for &u in &bfs {
            level_count[dist[u] as usize] += 1;
        }
        let half = (reached + 1) / 2;
        let mut sep_level = 1usize;
        let mut cum = 0usize;
        for l in 1..=(maxd as usize) {
            cum += level_count[l];
            if cum >= half {
                sep_level = l;
                break;
            }
        }

        // Partition: left (dist < sep_level), separator (dist == sep_level),
        // right (dist > sep_level OR unreached other components). Scanning
        // `nodes` in ascending order keeps all three lists deterministic.
        let mut left: Vec<usize> = Vec::new();
        let mut sep: Vec<usize> = Vec::new();
        let mut right: Vec<usize> = Vec::new();
        for &u in &nodes {
            let d = dist[u] as usize;
            if d == 0 {
                right.push(u);
            } else if d < sep_level {
                left.push(u);
            } else if d == sep_level {
                sep.push(u);
            } else {
                right.push(u);
            }
        }

        // Reset scratch for reuse.
        for &u in &bfs {
            dist[u] = 0;
        }
        for &u in &nodes {
            mark[u] = false;
        }

        // Unsplittable (separator is the whole subset): degree-order and stop.
        if left.is_empty() && right.is_empty() {
            deg_fill(&mut order, lo, sep);
            continue;
        }

        // Separator at the TOP of the range (eliminated last); subdomains below.
        let sep_len = sep.len();
        let sep_start = hi - sep_len;
        for (t, u) in sep.iter().enumerate() {
            order[sep_start + t] = *u;
        }

        let left_len = left.len();
        if !left.is_empty() {
            stack.push((left, lo, lo + left_len));
        }
        if !right.is_empty() {
            stack.push((right, lo + left_len, sep_start));
        }
    }

    order.into_iter().map(|x| x as i32).collect()
}

pub(super) fn rcm_order(pattern: &Pattern) -> Vec<i32> {
    let n = pattern.n;

    // Symmetric adjacency (exclude self-loops; dedup for accurate degrees).
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for j in 0..n {
        let start = pattern.col_ptr[j];
        let end = pattern.col_ptr[j + 1];
        for &i in &pattern.row_idx[start..end] {
            if i != j && i < n {
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }
    for a in adj.iter_mut() {
        a.sort_unstable();
        a.dedup();
    }
    let degree: Vec<usize> = adj.iter().map(|a| a.len()).collect();

    let mut visited = vec![false; n];
    let mut order: Vec<usize> = Vec::with_capacity(n);
    // Reused BFS distance buffer (0 = unvisited); touched entries reset per call.
    let mut dist: Vec<u32> = vec![0u32; n];
    let mut touched: Vec<usize> = Vec::new();
    let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    let mut nbrs: Vec<usize> = Vec::new();

    for seed in 0..n {
        if visited[seed] {
            continue;
        }
        let start = if degree[seed] == 0 {
            seed
        } else {
            pseudo_peripheral(seed, &adj, &degree, &mut dist, &mut touched)
        };

        // Cuthill–McKee BFS from `start`.
        queue.clear();
        visited[start] = true;
        order.push(start);
        queue.push_back(start);
        while let Some(u) = queue.pop_front() {
            nbrs.clear();
            for &v in &adj[u] {
                if !visited[v] {
                    nbrs.push(v);
                }
            }
            nbrs.sort_by_key(|&v| degree[v]); // stable → deterministic
            for &v in &nbrs {
                if !visited[v] {
                    visited[v] = true;
                    order.push(v);
                    queue.push_back(v);
                }
            }
        }
    }

    order.reverse(); // Cuthill–McKee → Reverse Cuthill–McKee
    order.into_iter().map(|x| x as i32).collect()
}

#[cfg(test)]
pub(super) fn sloan_order(pattern: &Pattern, w1: i64, w2: i64) -> Vec<i32> {
    let n = pattern.n;

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for j in 0..n {
        let start = pattern.col_ptr[j];
        let end = pattern.col_ptr[j + 1];
        for &i in &pattern.row_idx[start..end] {
            if i != j && i < n {
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }
    for a in adj.iter_mut() {
        a.sort_unstable();
        a.dedup();
    }
    let degree: Vec<usize> = adj.iter().map(|a| a.len()).collect();

    const INACTIVE: u8 = 0;
    const PREACTIVE: u8 = 1;
    const ACTIVE: u8 = 2;
    const POSTACTIVE: u8 = 3;

    let mut status: Vec<u8> = vec![INACTIVE; n];
    let mut priority: Vec<i64> = vec![0i64; n];
    let mut order: Vec<usize> = Vec::with_capacity(n);

    // Reused BFS buffers. `dist` is 1-based (0 = unvisited) and is restored to
    // all-zero after every use so it can be reused across components.
    let mut dist: Vec<u32> = vec![0u32; n];
    let mut touched: Vec<usize> = Vec::new();
    let mut comp: Vec<usize> = Vec::new();
    let mut heap: std::collections::BinaryHeap<(i64, usize)> = std::collections::BinaryHeap::new();

    for seed in 0..n {
        if status[seed] == POSTACTIVE {
            continue; // already numbered as part of an earlier component
        }

        // Pseudo-peripheral start node, then its far endpoint = `end`.
        let start = if degree[seed] == 0 {
            seed
        } else {
            pseudo_peripheral(seed, &adj, &degree, &mut dist, &mut touched)
        };
        let (end, _) = bfs_deepest(start, &adj, &degree, &mut dist, &mut touched);

        // BFS from `end`: collect the component and its distances to `end`.
        comp.clear();
        comp.push(end);
        dist[end] = 1;
        let mut head = 0;
        while head < comp.len() {
            let u = comp[head];
            head += 1;
            for &v in &adj[u] {
                if dist[v] == 0 {
                    dist[v] = dist[u] + 1;
                    comp.push(v);
                }
            }
        }

        // Initialize priorities for the component; reset dist for reuse.
        for &u in comp.iter() {
            let de = (dist[u] - 1) as i64; // distance from `u` to `end`
            priority[u] = w1 * de - w2 * (degree[u] as i64 + 1);
            status[u] = INACTIVE;
            dist[u] = 0;
        }

        // Sloan selection loop over this component.
        heap.clear();
        status[start] = PREACTIVE;
        heap.push((priority[start], start));
        while let Some((p, i)) = heap.pop() {
            if status[i] == POSTACTIVE || p != priority[i] {
                continue; // already numbered, or a stale (superseded) entry
            }

            if status[i] == PREACTIVE {
                for &j in &adj[i] {
                    priority[j] += w2;
                    if status[j] == INACTIVE {
                        status[j] = PREACTIVE;
                        heap.push((priority[j], j));
                    } else if status[j] != POSTACTIVE {
                        heap.push((priority[j], j)); // priority increased
                    }
                }
            }

            order.push(i);
            status[i] = POSTACTIVE;

            for &j in &adj[i] {
                if status[j] == PREACTIVE {
                    status[j] = ACTIVE;
                    priority[j] += w2;
                    heap.push((priority[j], j)); // still eligible (active)
                    for &k in &adj[j] {
                        if status[k] != POSTACTIVE {
                            priority[k] += w2;
                            if status[k] == INACTIVE {
                                status[k] = PREACTIVE;
                            }
                            heap.push((priority[k], k));
                        }
                    }
                }
            }
        }
    }

    order.into_iter().map(|x| x as i32).collect()
}

/// Find a pseudo-peripheral node within `seed`'s component: repeatedly BFS to the
/// deepest level and jump to a minimum-degree node there while eccentricity keeps
/// growing (capped iterations). `dist`/`touched` are reused buffers.
fn pseudo_peripheral(
    seed: usize,
    adj: &[Vec<usize>],
    degree: &[usize],
    dist: &mut [u32],
    touched: &mut Vec<usize>,
) -> usize {
    let mut start = seed;
    let mut prev_ecc = 0u32;
    for _ in 0..5 {
        let (deepest, ecc) = bfs_deepest(start, adj, degree, dist, touched);
        if ecc <= prev_ecc {
            break;
        }
        prev_ecc = ecc;
        start = deepest;
    }
    start
}

/// BFS from `start` over the component; returns (minimum-degree node in the
/// deepest level, eccentricity). Uses `dist` as a 1-based visited/distance
/// buffer and `touched` as the queue + reset list, leaving `dist` all-zero on
/// return so it can be reused.
fn bfs_deepest(
    start: usize,
    adj: &[Vec<usize>],
    degree: &[usize],
    dist: &mut [u32],
    touched: &mut Vec<usize>,
) -> (usize, u32) {
    touched.clear();
    touched.push(start);
    dist[start] = 1;
    let mut head = 0;
    let mut max_d = 1u32;
    while head < touched.len() {
        let u = touched[head];
        head += 1;
        let d = dist[u];
        if d > max_d {
            max_d = d;
        }
        for &v in &adj[u] {
            if dist[v] == 0 {
                dist[v] = d + 1;
                touched.push(v);
            }
        }
    }

    let mut best = start;
    let mut best_deg = usize::MAX;
    for &u in touched.iter() {
        if dist[u] == max_d && degree[u] < best_deg {
            best_deg = degree[u];
            best = u;
        }
    }

    for &u in touched.iter() {
        dist[u] = 0; // restore invariant for reuse
    }

    (best, max_d - 1)
}
