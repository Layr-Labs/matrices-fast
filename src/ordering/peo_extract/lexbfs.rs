//! LexBFS by ordered partition refinement, with O(n + |E|) work and O(n) scratch.
//! Every block represents one visited-neighbor history. Refining a block puts
//! its neighbors before its nonneighbors without disturbing other histories.

const NONE: usize = usize::MAX;

#[derive(Clone, Copy)]
struct Block {
    first: usize,
    prev: usize,
    next: usize,
    split: usize,
    epoch: usize,
}

impl Block {
    const EMPTY: Self = Self {
        first: NONE,
        prev: NONE,
        next: NONE,
        split: NONE,
        epoch: NONE,
    };
}

struct Partition {
    blocks: Vec<Block>,
    first: usize,
    vertex_block: Vec<usize>,
    prev: Vec<usize>,
    next: Vec<usize>,
    free: Vec<usize>,
}

impl Partition {
    fn new(incumbent: &[usize]) -> Self {
        let n = incumbent.len();
        let mut result = Self {
            // At most n nonempty blocks, plus one temporary empty split.
            blocks: vec![Block::EMPTY; n + 1],
            first: 0,
            vertex_block: vec![0; n],
            prev: vec![NONE; n],
            next: vec![NONE; n],
            free: (1..=n).rev().collect(),
        };
        result.blocks[0].first = incumbent[0];
        for pair in incumbent.windows(2) {
            result.next[pair[0]] = pair[1];
            result.prev[pair[1]] = pair[0];
        }
        result
    }

    fn unlink_vertex(&mut self, v: usize) {
        let b = self.vertex_block[v];
        let prev = self.prev[v];
        let next = self.next[v];
        if prev == NONE {
            self.blocks[b].first = next;
        } else {
            self.next[prev] = next;
        }
        if next != NONE {
            self.prev[next] = prev;
        }
    }

    fn remove_empty_block(&mut self, b: usize) {
        let prev = self.blocks[b].prev;
        let next = self.blocks[b].next;
        if prev == NONE {
            self.first = next;
        } else {
            self.blocks[prev].next = next;
        }
        if next != NONE {
            self.blocks[next].prev = prev;
        }
        self.free.push(b);
    }

    fn pop_first(&mut self) -> Option<usize> {
        if self.first == NONE {
            return None;
        }
        let b = self.first;
        let v = self.blocks[b].first;
        self.unlink_vertex(v);
        self.vertex_block[v] = NONE;
        if self.blocks[b].first == NONE {
            self.remove_empty_block(b);
        }
        Some(v)
    }

    fn promote_neighbor(&mut self, u: usize, epoch: usize) -> Option<()> {
        let b = self.vertex_block[u];
        if b == NONE {
            return Some(());
        }
        if self.blocks[b].epoch != epoch {
            // Blocks are recycled rather than accumulating O(|E|) records.
            // Reset the epoch when reusing an id: it may have died this step.
            let split = self.free.pop()?;
            let prev = self.blocks[b].prev;
            self.blocks[split] = Block {
                prev,
                next: b,
                ..Block::EMPTY
            };
            if prev == NONE {
                self.first = split;
            } else {
                self.blocks[prev].next = split;
            }
            self.blocks[b].prev = split;
            self.blocks[b].split = split;
            self.blocks[b].epoch = epoch;
        }
        let split = self.blocks[b].split;
        self.unlink_vertex(u);
        let old_first = self.blocks[split].first;
        self.prev[u] = NONE;
        self.next[u] = old_first;
        if old_first != NONE {
            self.prev[old_first] = u;
        }
        self.blocks[split].first = u;
        self.vertex_block[u] = split;
        if self.blocks[b].first == NONE {
            self.remove_empty_block(b);
        }
        Some(())
    }
}

/// The caller supplies checked, duplicate-free symmetric adjacency and a
/// bijective incumbent. Reverse LexBFS is a PEO when this graph is chordal.
pub(super) fn peo(adj: &[Vec<u32>], incumbent: &[usize]) -> Option<Vec<usize>> {
    let n = adj.len();
    if incumbent.len() != n {
        return None;
    }
    if n == 0 {
        return Some(Vec::new());
    }
    let mut partition = Partition::new(incumbent);
    let mut visit = Vec::with_capacity(n);
    for epoch in 0..n {
        let v = partition.pop_first()?;
        visit.push(v);
        // Head insertion reverses encounter order, as in the replaced
        // reverse-adjacency MCS slot; histories, not cardinalities, rank blocks.
        for &u in adj[v].iter().rev() {
            partition.promote_neighbor(u as usize, epoch)?;
        }
    }
    visit.reverse();
    Some(visit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_lexbfs(adj: &[Vec<u32>], peo: &[usize]) {
        let n = adj.len();
        let mut labels = vec![Vec::<usize>::new(); n];
        let mut live = vec![true; n];
        for (step, &v) in peo.iter().rev().enumerate() {
            assert!(live[v]);
            for u in 0..n {
                if live[u] {
                    assert!(labels[v] >= labels[u], "step={step} v={v} u={u}");
                }
            }
            live[v] = false;
            for &u in &adj[v] {
                if live[u as usize] {
                    labels[u as usize].push(n - step);
                }
            }
        }
        assert!(live.iter().all(|&v| !v));
    }

    #[test]
    fn partitions_match_lexicographic_labels_on_all_small_graphs() {
        for n in 1..=5 {
            let edges: Vec<_> = (0..n)
                .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
                .collect();
            for mask in 0..1usize << edges.len() {
                let mut adj = vec![Vec::new(); n];
                for (bit, &(u, v)) in edges.iter().enumerate() {
                    if mask & (1 << bit) != 0 {
                        adj[u].push(v as u32);
                        adj[v].push(u as u32);
                    }
                }
                for incumbent in [(0..n).collect::<Vec<_>>(), (0..n).rev().collect()] {
                    let order = peo(&adj, &incumbent).unwrap();
                    assert_lexbfs(&adj, &order);
                    assert_eq!(peo(&adj, &incumbent).unwrap(), order);
                }
            }
        }
        assert_eq!(peo(&[], &[]), Some(Vec::new()));
    }

    #[test]
    fn recycled_blocks_handle_long_paths_and_disconnected_vertices() {
        let n = 4096;
        let mut adj = vec![Vec::new(); n];
        for v in 0..n - 1 {
            adj[v].push((v + 1) as u32);
            adj[v + 1].push(v as u32);
        }
        let incumbent: Vec<_> = (0..n).collect();
        let expected: Vec<_> = (0..n).rev().collect();
        assert_eq!(peo(&adj, &incumbent), Some(expected.clone()));
        for row in &mut adj {
            row.clear();
        }
        assert_eq!(peo(&adj, &incumbent), Some(expected));
    }
}
