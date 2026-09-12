//! Full symbolic scoring and strict, stable candidate acceptance.
use super::*;

/// Symmetry lets us scatter rows in increasing new-index order. The resulting
/// columns are already sorted, avoiding a separate sort of every adjacency list.
pub(super) fn permute_pattern(pattern: &ScoringPattern, perm: &[usize]) -> ScoringPattern {
    let n = pattern.n;
    let mut inverse = vec![0; n];
    let mut col_ptr = Vec::with_capacity(n + 1);
    col_ptr.push(0);
    for (i, &v) in perm.iter().enumerate() {
        inverse[v] = i;
        col_ptr.push(col_ptr[i] + pattern.col_ptr[v + 1] - pattern.col_ptr[v]);
    }
    let mut next = col_ptr[..n].to_vec();
    let mut row_idx = vec![0; pattern.row_idx.len()];
    for (i, &v) in perm.iter().enumerate() {
        for &u in &pattern.row_idx[pattern.col_ptr[v]..pattern.col_ptr[v + 1]] {
            let j = inverse[u];
            row_idx[next[j]] = i;
            next[j] += 1;
        }
    }
    ScoringPattern {
        n,
        col_ptr,
        row_idx,
    }
}

/// GNP column counts with one flat child list and one shared postorder.
/// Child visitation matches the library's increasing-index DFS exactly.
pub(super) fn symbolic_counts(
    pattern: &ScoringPattern,
    tree: &EliminationTree,
) -> (Vec<usize>, Vec<usize>) {
    let n = pattern.n;
    let mut head = vec![n; n];
    let mut sibling = vec![n; n];
    let mut delta = vec![1i64; n];
    for v in (0..n).rev() {
        if let Some(p) = tree.parent[v] {
            sibling[v] = head[p];
            head[p] = v;
            delta[p] = 0;
        }
    }
    let mut post = Vec::with_capacity(n);
    let mut stack = Vec::with_capacity(n);
    for root in 0..n {
        if tree.parent[root].is_some() {
            continue;
        }
        stack.push(root);
        while let Some(&v) = stack.last() {
            let child = head[v];
            if child == n {
                post.push(v);
                stack.pop();
            } else {
                head[v] = sibling[child];
                stack.push(child);
            }
        }
    }
    let mut first = vec![n; n];
    for (i, &v) in post.iter().enumerate() {
        first[v] = first[v].min(i);
        if let Some(p) = tree.parent[v] {
            first[p] = first[p].min(first[v]);
        }
    }
    let mut maxfirst = vec![n; n];
    let mut prevleaf = vec![n; n];
    let mut ancestor: Vec<_> = (0..n).collect();
    for &v in &post {
        if let Some(p) = tree.parent[v] {
            delta[p] -= 1;
        }
        for &u in &pattern.row_idx[pattern.col_ptr[v]..pattern.col_ptr[v + 1]] {
            if u <= v || (maxfirst[u] != n && first[v] <= maxfirst[u]) {
                continue;
            }
            delta[v] += 1;
            let previous = prevleaf[u];
            if previous != n {
                let mut root = previous;
                while ancestor[root] != root {
                    root = ancestor[root];
                }
                let mut node = previous;
                while node != root {
                    let next = ancestor[node];
                    ancestor[node] = root;
                    node = next;
                }
                delta[root] -= 1;
            }
            prevleaf[u] = v;
            maxfirst[u] = first[v];
        }
        if let Some(p) = tree.parent[v] {
            ancestor[v] = p;
        }
    }
    for &v in &post {
        if let Some(p) = tree.parent[v] {
            delta[p] += delta[v];
        }
    }
    (post, delta.into_iter().map(|v| v as usize).collect())
}

/// Predicted factorization flops `Σ_j c_j²` for `perm` on `pat`, via feral's
/// pattern-pure symbolic building blocks — the exact quantity the grader ranks.
pub(super) fn flops_of(pat: &ScoringPattern, perm: &[usize]) -> u64 {
    cost_and_entries(pat, perm).0
}

/// Cost and factor entries from one symbolic replay. Equality of factor and
/// input entries certifies a fill-free order without counting triangles.
pub(super) fn cost_and_entries(pat: &ScoringPattern, perm: &[usize]) -> (u64, usize) {
    let permuted = permute_pattern(pat, perm);
    let etree = EliminationTree::from_pattern(&permuted);
    let (_, counts) = symbolic_counts(&permuted, &etree);
    (
        counts.iter().map(|&c| (c as u64) * (c as u64)).sum(),
        counts.iter().sum(),
    )
}

/// Whether `perm` is a bijection of `0..n` (guards a candidate before scoring).
pub(super) fn is_bijection(perm: &[usize], n: usize) -> bool {
    if perm.len() != n {
        return false;
    }
    let mut seen = vec![false; n];
    for &v in perm {
        if v >= n || seen[v] {
            return false;
        }
        seen[v] = true;
    }
    true
}

/// Relabel `perm` into an elimination-tree postorder, then compute that
/// postorder's exact column counts and etree parent array. Every round of the
/// ranked-subtree refinement chain in `order()` needs exactly this triple to
/// hand `rgreedy::subtree_refine` a contiguous-per-subtree numbering.
pub(super) fn etree_prep(
    scoring_pat: &ScoringPattern,
    perm: &[usize],
) -> (Vec<usize>, Vec<u32>, Vec<i32>) {
    let permuted = permute_pattern(scoring_pat, perm);
    let etree = EliminationTree::from_pattern(&permuted);
    let (post, original_counts) = symbolic_counts(&permuted, &etree);
    let candidate: Vec<usize> = post.iter().map(|&j| perm[j]).collect();

    // Tree postordering only exchanges independent elimination subtrees.
    // Column counts and parents therefore transport by the same permutation;
    // rebuilding the permuted graph and its tree is redundant.
    let mut rank = vec![0; perm.len()];
    for (i, &v) in post.iter().enumerate() {
        rank[v] = i;
    }
    let counts = post.iter().map(|&v| original_counts[v] as u32).collect();
    let parent: Vec<i32> = post
        .iter()
        .map(|&v| etree.parent[v].map_or(-1, |j| rank[j] as i32))
        .collect();
    (candidate, counts, parent)
}

/// Score distinct orders once; return the first order's cost for restart screening.
pub(super) fn accept_orders(
    scoring: &ScoringPattern,
    orders: impl IntoIterator<Item = Vec<usize>>,
    best: &mut Vec<usize>,
    cost: &mut u64,
) -> Option<u64> {
    let mut unique = Vec::new();
    let mut first_cost = None;
    let mut first_unique = false;
    for (i, order) in orders.into_iter().enumerate() {
        if i == 0 && order == *best {
            first_cost = Some(*cost);
        }
        if order != *best && is_bijection(&order, scoring.n) && !unique.iter().any(|p| *p == order)
        {
            if i == 0 {
                first_unique = true;
            }
            unique.push(order);
        }
    }
    let threads = if scoring.n + scoring.row_idx.len() < 25_000 {
        1
    } else {
        4
    };
    let costs = parallel::map_indexed(&unique, threads, |_, order| flops_of(scoring, order));
    if first_unique {
        first_cost = costs.first().copied();
    }
    for (order, value) in unique.into_iter().zip(costs) {
        if value < *cost {
            *best = order;
            *cost = value;
        }
    }
    first_cost
}

pub(super) fn accept_if_cheaper(
    candidate: Option<Vec<usize>>,
    n: usize,
    scoring_pat: &ScoringPattern,
    best_perm: &mut Vec<usize>,
    best_flops: &mut u64,
) {
    let Some(perm) = candidate else {
        return;
    };
    if perm == *best_perm {
        return;
    }
    if !is_bijection(&perm, n) {
        return;
    }
    let f = flops_of(scoring_pat, &perm);
    if f < *best_flops {
        *best_flops = f;
        *best_perm = perm;
    }
}
