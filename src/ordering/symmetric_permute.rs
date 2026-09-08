//! Linear-time canonical symmetric permutation.
//!
//! For symmetric A, P A P^T equals P A^T P^T. Visit old columns in
//! increasing NEW ROW order and scatter their neighbors into new columns.
//! Each destination receives increasing row indices, so no column sort is
//! needed. Both triangles, diagonal entries, and symmetric multiplicities
//! are preserved. Like the vendor helper, this requires full symmetric CSC
//! and a bijective new-to-old permutation; callers already have that contract.

use super::ScoringPattern;

pub(super) fn permute_pattern(pat: &ScoringPattern, perm: &[usize]) -> ScoringPattern {
    let n = pat.n;
    let mut inverse = vec![0usize; n];
    let mut col_ptr = Vec::with_capacity(n + 1);
    col_ptr.push(0);
    for (new, &old) in perm.iter().enumerate() {
        inverse[old] = new;
        col_ptr.push(col_ptr[new] + pat.col_ptr[old + 1] - pat.col_ptr[old]);
    }
    let mut cursor = col_ptr[..n].to_vec();
    let mut row_idx = vec![0usize; col_ptr[n]];
    for (new_row, &old_row) in perm.iter().enumerate() {
        // A[:,old_row] is A[old_row,:] by structural symmetry.
        for &old_col in &pat.row_idx[pat.col_ptr[old_row]..pat.col_ptr[old_row + 1]] {
            let new_col = inverse[old_col];
            row_idx[cursor[new_col]] = new_row;
            cursor[new_col] += 1;
        }
    }
    ScoringPattern { n, col_ptr, row_idx }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(pat: &ScoringPattern, perm: &[usize]) {
        let expected = feral::ordering::amd::permute_pattern(pat, perm);
        let actual = permute_pattern(pat, perm);
        assert_eq!(actual.n, expected.n);
        assert_eq!(actual.col_ptr, expected.col_ptr);
        assert_eq!(actual.row_idx, expected.row_idx);
    }

    #[test]
    fn exhaustive_symmetric_patterns_and_permutations() {
        check(&ScoringPattern { n: 0, col_ptr: vec![0], row_idx: vec![] }, &[]);
        let n = 4;
        let pairs: Vec<_> = (0..n).flat_map(|i| (i + 1..n).map(move |j| (i, j))).collect();
        for mask in 0..1usize << pairs.len() {
            for diagonal in [false, true] {
                let mut adj = vec![Vec::new(); n];
                for (bit, &(u, v)) in pairs.iter().enumerate() {
                    if mask & (1 << bit) != 0 {
                        adj[u].push(v);
                        adj[v].push(u);
                    }
                }
                if diagonal {
                    for (v, row) in adj.iter_mut().enumerate() { row.push(v); }
                }
                // Intentionally noncanonical input row order.
                for row in &mut adj { row.reverse(); }
                let mut col_ptr = vec![0];
                let mut row_idx = Vec::new();
                for row in adj { row_idx.extend(row); col_ptr.push(row_idx.len()); }
                let pat = ScoringPattern { n, col_ptr, row_idx };
                let mut perm: Vec<_> = (0..n).collect();
                loop {
                    check(&pat, &perm);
                    let Some(i) = (0..n - 1).rev().find(|&i| perm[i] < perm[i + 1]) else { break; };
                    let j = (i + 1..n).rev().find(|&j| perm[j] > perm[i]).unwrap();
                    perm.swap(i, j);
                    perm[i + 1..].reverse();
                }
            }
        }
    }

    #[test]
    fn symmetric_corpus_equivalence() {
        let mut cases = 0;
        let mut checks = 0;
        let mut reference_ns = 0u128;
        let mut linear_ns = 0u128;
        for (name, pat) in crate::corpus::corpus() {
            let sp = ScoringPattern { n: pat.n, col_ptr: pat.col_ptr, row_idx: pat.row_idx };
            for ticket in 0..4 {
                let perm: Vec<_> = match ticket {
                    0 => (0..sp.n).collect(),
                    1 => (0..sp.n).rev().collect(),
                    _ => super::super::relabel(sp.n, ticket),
                };
                let expected;
                let actual;
                if ticket % 2 == 0 {
                    let t = std::time::Instant::now();
                    expected = feral::ordering::amd::permute_pattern(&sp, &perm);
                    reference_ns += t.elapsed().as_nanos();
                    let t = std::time::Instant::now();
                    actual = permute_pattern(&sp, &perm);
                    linear_ns += t.elapsed().as_nanos();
                } else {
                    let t = std::time::Instant::now();
                    actual = permute_pattern(&sp, &perm);
                    linear_ns += t.elapsed().as_nanos();
                    let t = std::time::Instant::now();
                    expected = feral::ordering::amd::permute_pattern(&sp, &perm);
                    reference_ns += t.elapsed().as_nanos();
                }
                assert_eq!(actual.col_ptr, expected.col_ptr, "{name}, ticket {ticket}");
                assert_eq!(actual.row_idx, expected.row_idx, "{name}, ticket {ticket}");
                checks += 1;
            }
            cases += 1;
        }
        assert_eq!(cases, 300);
        println!("SYMMETRIC_PERMUTE cases={cases} checks={checks} reference_ns={reference_ns} linear_ns={linear_ns}");
    }
}
