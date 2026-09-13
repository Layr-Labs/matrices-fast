//! Original-graph chordality certificate using MCS and checked PEO extraction.
//!
//! For a chordal completion H, sum of squared column counts is
//! n + 3|E(H)| + 2T(H). A verified no-fill ordering therefore attains the
//! global FLOP lower bound. Recognition uses O(n + nnz) work, O(n) scratch,
//! and borrowed input adjacency; it never constructs fill or enumerates pairs.

const NONE: usize = usize::MAX;

struct Workspace {
    label: Vec<usize>,
    head: Vec<usize>,
    next: Vec<usize>,
    previous: Vec<usize>,
}

impl Workspace {
    fn new(n: usize) -> Self {
        Self {
            label: vec![NONE; n],
            head: vec![NONE; n],
            next: vec![NONE; n],
            previous: vec![NONE; n],
        }
    }
}

/// Certify a symmetric, diagonal-free CSC pattern. The caller guarantees
/// symmetry; pointer bounds, vertex bounds and duplicate entries are checked.
/// Adjacency lists need not be sorted. Ties are deterministic for the input:
/// initial vertex IDs are ascending, and updated MCS buckets are LIFO.
pub(crate) fn order(n: usize, cp: &[usize], ri: &[usize]) -> Option<Vec<usize>> {
    order_bounded(n, cp, ri, usize::MAX)
}

/// The same certificate with an O(1), input-only refusal before allocation.
/// `max_input_size` bounds n + directed nnz, not elapsed time or FLOPs.
pub(crate) fn order_bounded(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    max_input_size: usize,
) -> Option<Vec<usize>> {
    if n.checked_add(ri.len())? > max_input_size
        || cp.len() != n.checked_add(1)?
        || cp.first().copied() != Some(0)
        || cp.last().copied() != Some(ri.len())
    {
        return None;
    }
    if n == 0 {
        return Some(Vec::new());
    }

    let mut work = Workspace::new(n);
    for v in 0..n {
        let (start, end) = (cp[v], cp[v + 1]);
        if start > end || end > ri.len() {
            return None;
        }
        for &u in &ri[start..end] {
            if u >= n || u == v || work.label[u] == v {
                return None;
            }
            work.label[u] = v;
        }
    }

    let perm = mcs(n, cp, ri, &mut work);
    validate_peo(n, cp, ri, &perm, &mut work).then_some(perm)
}

fn mcs(n: usize, cp: &[usize], ri: &[usize], work: &mut Workspace) -> Vec<usize> {
    work.label.fill(0);
    work.head[0] = 0;
    for v in 0..n {
        work.previous[v] = if v == 0 { NONE } else { v - 1 };
        work.next[v] = if v + 1 == n { NONE } else { v + 1 };
    }

    let mut maximum = 0usize;
    let mut visit = Vec::with_capacity(n);
    for _ in 0..n {
        while work.head[maximum] == NONE {
            maximum -= 1;
        }
        let v = work.head[maximum];
        let after = work.next[v];
        work.head[maximum] = after;
        if after != NONE {
            work.previous[after] = NONE;
        }
        work.label[v] = NONE;
        visit.push(v);

        for &u in &ri[cp[v]..cp[v + 1]] {
            let old = work.label[u];
            if old == NONE {
                continue;
            }
            let before = work.previous[u];
            let after = work.next[u];
            if before == NONE {
                work.head[old] = after;
            } else {
                work.next[before] = after;
            }
            if after != NONE {
                work.previous[after] = before;
            }

            let new = old + 1;
            let first = work.head[new];
            work.label[u] = new;
            work.previous[u] = NONE;
            work.next[u] = first;
            if first != NONE {
                work.previous[first] = u;
            }
            work.head[new] = u;
            maximum = maximum.max(new);
        }
    }
    visit.reverse();
    visit
}

fn validate_peo(
    n: usize,
    cp: &[usize],
    ri: &[usize],
    perm: &[usize],
    work: &mut Workspace,
) -> bool {
    if perm.len() != n {
        return false;
    }
    work.label.fill(NONE);
    for (position, &v) in perm.iter().enumerate() {
        if v >= n || work.label[v] != NONE {
            return false;
        }
        work.label[v] = position;
    }
    work.head.fill(NONE);
    work.previous.fill(NONE);

    for v in 0..n {
        let mut parent = NONE;
        let mut earliest = n;
        for &u in &ri[cp[v]..cp[v + 1]] {
            let position = work.label[u];
            if position > work.label[v] && position < earliest {
                parent = u;
                earliest = position;
            }
        }
        if parent != NONE {
            work.next[v] = work.head[parent];
            work.head[parent] = v;
        }
    }

    // Mark each parent's neighbors once, not once per child: hubs remain
    // linear-work. N+(v) \ {parent(v)} must be contained in N+(parent(v)).
    for parent in 0..n {
        if work.head[parent] == NONE {
            continue;
        }
        for &u in &ri[cp[parent]..cp[parent + 1]] {
            work.previous[u] = parent;
        }
        let mut child = work.head[parent];
        while child != NONE {
            for &u in &ri[cp[child]..cp[child + 1]] {
                if work.label[u] > work.label[child] && u != parent && work.previous[u] != parent {
                    return false;
                }
            }
            child = work.next[child];
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pattern(rows: &[u64]) -> (Vec<usize>, Vec<usize>) {
        let mut cp = vec![0];
        let mut ri = Vec::new();
        for &row in rows {
            for u in 0..rows.len() {
                if row & (1u64 << u) != 0 {
                    ri.push(u);
                }
            }
            cp.push(ri.len());
        }
        (cp, ri)
    }

    fn literal_peo(rows: &[u64], perm: &[usize]) -> bool {
        let mut live = (1u64 << rows.len()) - 1;
        for &v in perm {
            let neighbors = rows[v] & live;
            let mut remaining = neighbors;
            while remaining != 0 {
                let u = remaining.trailing_zeros() as usize;
                remaining &= remaining - 1;
                if neighbors & !(rows[u] | (1u64 << u)) != 0 {
                    return false;
                }
            }
            live &= !(1u64 << v);
        }
        true
    }

    fn literal_flops(rows: &[u64], perm: &[usize]) -> u64 {
        let mut filled = rows.to_vec();
        let mut live = (1u64 << rows.len()) - 1;
        let mut total = 0;
        for &v in perm {
            let neighbors = filled[v] & live;
            let count = neighbors.count_ones() as u64 + 1;
            total += count * count;
            let mut remaining = neighbors;
            while remaining != 0 {
                let u = remaining.trailing_zeros() as usize;
                remaining &= remaining - 1;
                filled[u] |= neighbors & !(1u64 << u);
            }
            live &= !(1u64 << v);
        }
        total
    }

    fn next_permutation(p: &mut [usize]) -> bool {
        let Some(i) = (0..p.len().saturating_sub(1))
            .rev()
            .find(|&i| p[i] < p[i + 1])
        else {
            return false;
        };
        let j = (i + 1..p.len()).rev().find(|&j| p[i] < p[j]).unwrap();
        p.swap(i, j);
        p[i + 1..].reverse();
        true
    }

    #[test]
    fn exhaustive_recognition_validation_and_flop_optimality() {
        for n in 0..=5 {
            let edges: Vec<_> = (0..n)
                .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
                .collect();
            for mask in 0..(1usize << edges.len()) {
                let mut rows = vec![0u64; n];
                for (bit, &(u, v)) in edges.iter().enumerate() {
                    if mask & (1 << bit) != 0 {
                        rows[u] |= 1 << v;
                        rows[v] |= 1 << u;
                    }
                }
                let (cp, ri) = pattern(&rows);
                let mut work = Workspace::new(n);
                let mut perm: Vec<_> = (0..n).collect();
                let mut optimum = u64::MAX;
                let mut chordal = false;
                loop {
                    let peo = literal_peo(&rows, &perm);
                    assert_eq!(validate_peo(n, &cp, &ri, &perm, &mut work), peo);
                    chordal |= peo;
                    optimum = optimum.min(literal_flops(&rows, &perm));
                    if !next_permutation(&mut perm) {
                        break;
                    }
                }
                let result = order(n, &cp, &ri);
                assert_eq!(result.is_some(), chordal, "n={n}, graph={mask}");
                assert_eq!(result, order(n, &cp, &ri));
                if let Some(perm) = result {
                    assert!(literal_peo(&rows, &perm));
                    assert_eq!(literal_flops(&rows, &perm), optimum);
                    let triangles: u64 = (0..n)
                        .map(|v| {
                            (v + 1..n)
                                .map(|u| {
                                    (u + 1..n)
                                        .filter(|&w| {
                                            rows[v] & (1 << u) != 0
                                                && rows[v] & (1 << w) != 0
                                                && rows[u] & (1 << w) != 0
                                        })
                                        .count() as u64
                                })
                                .sum::<u64>()
                        })
                        .sum();
                    assert_eq!(
                        optimum,
                        n as u64 + 3 * (ri.len() / 2) as u64 + 2 * triangles
                    );
                }
            }
        }
    }

    #[test]
    fn malformed_inputs_permutations_and_input_budget() {
        let (cp, ri) = pattern(&[0b110, 0b101, 0b011]);
        assert!(order(3, &cp, &ri).is_some());
        assert!(order_bounded(3, &cp, &ri, 8).is_none());
        assert_eq!(order_bounded(3, &cp, &ri, 9), order(3, &cp, &ri));
        assert!(order(3, &[0, 2, 1, 6], &ri).is_none());
        assert!(order(3, &[0, 2, 7, 6], &ri).is_none());
        assert!(order(3, &[0, 2, 4], &ri).is_none());
        assert!(order(2, &[0, 1, 2], &[1, 2]).is_none());
        assert!(order(2, &[0, 1, 2], &[0, 1]).is_none());
        assert!(order(2, &[0, 2, 4], &[1, 1, 0, 0]).is_none());
        assert!(order(usize::MAX, &[], &[]).is_none());
        let mut work = Workspace::new(3);
        for perm in [&[0, 0, 2][..], &[0, 1][..], &[0, 1, 3][..]] {
            assert!(!validate_peo(3, &cp, &ri, perm, &mut work));
        }
    }

    #[test]
    fn long_path_large_hub_dense_graph_and_unsorted_adjacency() {
        for n in [64usize, 50_000] {
            for star in [false, true] {
                let mut rows = vec![Vec::new(); n];
                for v in 1..n {
                    let u = if star { 0 } else { v - 1 };
                    rows[u].push(v);
                    rows[v].push(u);
                }
                let mut cp = vec![0];
                let mut ri = Vec::new();
                for row in &rows {
                    ri.extend(row.iter().rev().copied());
                    cp.push(ri.len());
                }
                let perm = order(n, &cp, &ri).unwrap();
                assert_eq!(perm, order(n, &cp, &ri).unwrap());
                let mut work = Workspace::new(n);
                assert!(validate_peo(n, &cp, &ri, &perm, &mut work));
            }
        }
        let n = 256;
        let mut cp = vec![0];
        let mut ri = Vec::new();
        for v in 0..n {
            ri.extend((0..n).rev().filter(|&u| u != v));
            cp.push(ri.len());
        }
        assert!(order(n, &cp, &ri).is_some());
        let (cp, ri) = pattern(&[0b1010, 0b0101, 0b1010, 0b0101]);
        assert!(order(4, &cp, &ri).is_none());
    }
}
