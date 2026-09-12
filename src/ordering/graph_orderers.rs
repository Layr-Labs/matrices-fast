use super::Pattern;
use std::collections::{BinaryHeap, VecDeque};

pub(super) fn minfill_order(pattern: &Pattern) -> Vec<i32> {
    let n = pattern.n;
    if n == 0 {
        return Vec::new();
    }
    let mut g = super::patch::Dense::from_pattern(n, &pattern.col_ptr, &pattern.row_idx, n);
    g.init_def();
    let mut budget = 40_000_000i64;
    let mut order = Vec::with_capacity(n);
    for _ in 0..n {
        if budget < 0 {
            let mut rest: Vec<_> = (0..n).filter(|&v| g.live[v]).collect();
            rest.sort_unstable_by_key(|&v| (g.deg[v], v));
            order.extend(rest.into_iter().map(|v| v as i32));
            break;
        }
        let mut best = None;
        for v in 0..n {
            if !g.live[v] {
                continue;
            }
            budget -= (g.deg[v] as i64).pow(2) / 2 + 1;
            if best.is_none_or(|u| (g.def[v], g.deg[v], v) < (g.def[u], g.deg[u], u)) {
                best = Some(v);
            }
        }
        let Some(v) = best else { break };
        order.push(v as i32);
        g.defelim(v);
    }
    order
}

struct Graph {
    adj: Vec<Vec<usize>>,
    degree: Vec<usize>,
}

impl Graph {
    fn new(pattern: &Pattern) -> Self {
        let n = pattern.n;
        let mut adj = vec![Vec::new(); n];
        for v in 0..n {
            for &u in &pattern.row_idx[pattern.col_ptr[v]..pattern.col_ptr[v + 1]] {
                if u != v && u < n {
                    adj[v].push(u);
                    adj[u].push(v);
                }
            }
        }
        for row in &mut adj {
            row.sort_unstable();
            row.dedup();
        }
        let degree = adj.iter().map(Vec::len).collect();
        Self { adj, degree }
    }

    fn bfs(
        &self,
        start: usize,
        allowed: Option<&[bool]>,
        dist: &mut [u32],
        queue: &mut Vec<usize>,
    ) -> (usize, u32) {
        queue.clear();
        queue.push(start);
        dist[start] = 1;
        let mut head = 0;
        let mut depth = 1;
        while head < queue.len() {
            let u = queue[head];
            head += 1;
            let d = dist[u];
            depth = depth.max(d);
            for &v in &self.adj[u] {
                if dist[v] == 0 && allowed.is_none_or(|mark| mark[v]) {
                    dist[v] = d + 1;
                    queue.push(v);
                }
            }
        }
        let mut best = start;
        let mut degree = usize::MAX;
        for &u in queue.iter() {
            if dist[u] == depth && self.degree[u] < degree {
                degree = self.degree[u];
                best = u;
            }
        }
        (best, depth)
    }

    fn gain(&self, v: usize, active: &[bool], inside: &[bool]) -> i64 {
        let mut gain = 0;
        for &u in &self.adj[v] {
            if active[u] {
                gain += if inside[u] { 1 } else { -1 };
            }
        }
        gain
    }

    fn split_fm(
        &self,
        nodes: &[usize],
        start: usize,
        active: &[bool],
        inside: &mut [bool],
        dist: &mut [u32],
    ) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        inside[start] = true;
        let mut count = 1;
        let mut heap = BinaryHeap::new();
        for &v in &self.adj[start] {
            if active[v] && !inside[v] {
                heap.push((self.gain(v, active, inside), -(v as isize)));
            }
        }
        while count < (nodes.len() + 1) / 2 {
            let Some((gain, neg_v)) = heap.pop() else {
                break;
            };
            let v = (-neg_v) as usize;
            if inside[v] {
                continue;
            }
            let current = self.gain(v, active, inside);
            if current != gain {
                heap.push((current, neg_v));
                continue;
            }
            inside[v] = true;
            count += 1;
            for &u in &self.adj[v] {
                if active[u] && !inside[u] {
                    heap.push((self.gain(u, active, inside), -(u as isize)));
                }
            }
        }
        let mut a = Vec::new();
        let mut b = Vec::new();
        for &v in nodes {
            if inside[v] {
                if self.adj[v].iter().any(|&u| active[u] && !inside[u]) {
                    a.push(v);
                }
            } else if self.adj[v].iter().any(|&u| active[u] && inside[u]) {
                b.push(v);
            }
        }
        let separator = if a.len() <= b.len() { a } else { b };
        for &v in &separator {
            dist[v] = 1;
        }
        let mut left = Vec::new();
        let mut right = Vec::new();
        for &v in nodes {
            if dist[v] != 0 {
                continue;
            }
            if inside[v] {
                left.push(v);
            } else {
                right.push(v);
            }
        }
        (left, separator, right)
    }
}

pub(super) fn nd_order(pattern: &Pattern) -> Vec<i32> {
    nested_dissection(pattern, false)
}
pub(super) fn ndfm_order(pattern: &Pattern) -> Vec<i32> {
    nested_dissection(pattern, true)
}

fn nested_dissection(pattern: &Pattern, fm: bool) -> Vec<i32> {
    let n = pattern.n;
    let graph = Graph::new(pattern);
    let leaf = if fm { 100 } else { 200 };
    let mut budget = if fm {
        96 * n as i64 + 8192
    } else {
        64 * n as i64 + 4096
    };
    let mut order = vec![0; n];
    let mut active = vec![false; n];
    let mut inside = vec![false; if fm { n } else { 0 }];
    let mut dist = vec![0; n];
    let mut queue = Vec::new();
    let mut stack = vec![((0..n).collect::<Vec<_>>(), 0)];
    let degree_order = |order: &mut [usize], lo: usize, mut nodes: Vec<usize>| {
        nodes.sort_by_key(|&v| (graph.degree[v], v));
        order[lo..lo + nodes.len()].copy_from_slice(&nodes);
    };
    while let Some((nodes, lo)) = stack.pop() {
        let size = nodes.len();
        let hi = lo + size;
        if size <= leaf || budget < 0 {
            degree_order(&mut order, lo, nodes);
            continue;
        }
        budget -= size as i64;
        for &v in &nodes {
            active[v] = true;
        }
        let mut start = nodes[0];
        for &v in &nodes {
            if graph.degree[v] < graph.degree[start] {
                start = v;
            }
        }
        start = graph.bfs(start, Some(&active), &mut dist, &mut queue).0;
        for &v in &queue {
            dist[v] = 0;
        }
        let (left, separator, right) = if fm {
            graph.split_fm(&nodes, start, &active, &mut inside, &mut dist)
        } else {
            let (_, depth) = graph.bfs(start, Some(&active), &mut dist, &mut queue);
            let mut levels = vec![0; depth as usize + 1];
            for &v in &queue {
                levels[dist[v] as usize] += 1;
            }
            let mut level = 1;
            let mut count = 0;
            for i in 1..=depth as usize {
                count += levels[i];
                if count >= (queue.len() + 1) / 2 {
                    level = i;
                    break;
                }
            }
            let (mut left, mut separator, mut right) = (Vec::new(), Vec::new(), Vec::new());
            for &v in &nodes {
                let d = dist[v] as usize;
                if d == 0 || d > level {
                    right.push(v);
                } else if d < level {
                    left.push(v);
                } else {
                    separator.push(v);
                }
            }
            (left, separator, right)
        };
        for &v in &nodes {
            active[v] = false;
            dist[v] = 0;
            if fm {
                inside[v] = false;
            }
        }
        if left.is_empty() && right.is_empty() {
            degree_order(&mut order, lo, separator);
            continue;
        }
        let sep_start = hi - separator.len();
        order[sep_start..hi].copy_from_slice(&separator);
        let left_len = left.len();
        if !left.is_empty() {
            stack.push((left, lo));
        }
        if !right.is_empty() {
            stack.push((right, lo + left_len));
        }
    }
    order.into_iter().map(|v| v as i32).collect()
}

pub(super) fn rcm_order(pattern: &Pattern) -> Vec<i32> {
    let n = pattern.n;
    let graph = Graph::new(pattern);
    let mut visited = vec![false; n];
    let mut order = Vec::with_capacity(n);
    let mut dist = vec![0; n];
    let mut touched = Vec::new();
    let mut queue = VecDeque::new();
    let mut neighbors = Vec::new();
    for seed in 0..n {
        if visited[seed] {
            continue;
        }
        let mut start = seed;
        if graph.degree[seed] != 0 {
            let mut previous = 0;
            for _ in 0..5 {
                let (deepest, depth) = graph.bfs(start, None, &mut dist, &mut touched);
                for &v in &touched {
                    dist[v] = 0;
                }
                let eccentricity = depth - 1;
                if eccentricity <= previous {
                    break;
                }
                previous = eccentricity;
                start = deepest;
            }
        }
        queue.clear();
        visited[start] = true;
        order.push(start);
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            neighbors.clear();
            neighbors.extend(graph.adj[v].iter().copied().filter(|&u| !visited[u]));
            neighbors.sort_by_key(|&u| graph.degree[u]);
            for &u in &neighbors {
                if !visited[u] {
                    visited[u] = true;
                    order.push(u);
                    queue.push_back(u);
                }
            }
        }
    }
    order.reverse();
    order.into_iter().map(|v| v as i32).collect()
}
