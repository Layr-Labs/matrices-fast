//! Test-only linear sorted-permutation experiment.
use super::*;

use crate::ordering::sorted_permutation::SortedPermutation;

#[test]
#[ignore]
fn probe_sorted_permutation() {
    let mut totals = [0.0; 2];
    let mut setup = 0.0;
    let mut checks = 0;
    for (name, pat) in crate::corpus::corpus() {
        let sp = scoring_pattern(&pat);
        let t = Instant::now();
        let mut workspace = SortedPermutation::new(&sp);
        setup += t.elapsed().as_secs_f64();
        let mut times = [0.0; 2];
        for ticket in 0..4 {
            let permutation = match ticket {
                0 => (0..pat.n).collect(),
                1 => (0..pat.n).rev().collect(),
                _ => relabel(pat.n, ticket),
            };
            let expected = permute_pattern(&sp, &permutation);
            let actual = workspace.permute(&permutation);
            assert_eq!(actual.col_ptr, expected.col_ptr, "{name}, ticket={ticket}");
            assert_eq!(actual.row_idx, expected.row_idx, "{name}, ticket={ticket}");
            let mut minima = [f64::MAX; 2];
            for pair in 0..3 {
                for offset in 0..2 {
                    let variant = (pair + offset) % 2;
                    let t = Instant::now();
                    let output = if variant == 0 { permute_pattern(&sp, &permutation) }
                        else { workspace.permute(&permutation) };
                    std::hint::black_box(output);
                    minima[variant] = minima[variant].min(t.elapsed().as_secs_f64());
                }
            }
            for variant in 0..2 { times[variant] += minima[variant]; }
            checks += 1;
        }
        for variant in 0..2 { totals[variant] += times[variant]; }
        println!("SORTED_PERM\t{name}\t{}\t{}\t{:.6}\t{:.6}", pat.n, pat.nnz(), times[0], times[1]);
    }
    assert_eq!(checks, 1_200);
    println!("SORTED_TOTAL checks={checks} old={:.6} new={:.6} setup={setup:.6}", totals[0], totals[1]);
}

#[test]
fn sorted_permutation_matches_nonsymmetric_and_duplicate_patterns() {
    for n in 0..=8 {
        for seed in 0..32 {
            let mut col_ptr = vec![0];
            let mut row_idx = Vec::new();
            for column in 0..n {
                for row in (0..n).rev() {
                    let key = column * 13 + row * 7 + seed * 17;
                    if key % 5 <= 1 { row_idx.push(row); }
                    if key % 11 == 0 { row_idx.push(row); }
                }
                col_ptr.push(row_idx.len());
            }
            let sp = ScoringPattern { n, col_ptr, row_idx };
            let mut workspace = SortedPermutation::new(&sp);
            for ticket in 0..4 {
                let p = relabel(n, ticket);
                let expected = permute_pattern(&sp, &p);
                let actual = workspace.permute(&p);
                assert_eq!(actual.col_ptr, expected.col_ptr);
                assert_eq!(actual.row_idx, expected.row_idx);
            }
        }
    }
}
