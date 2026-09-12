//! Test-only linear sorted-permutation experiment.
use super::*;

use crate::ordering::sorted_permutation::SortedPermutation;

thread_local! { static ALT_LIMIT: std::cell::Cell<usize> = std::cell::Cell::new(PEO_ALT_MAX_N); }
pub(in crate::ordering) fn alt_limit() -> usize { ALT_LIMIT.with(|a| a.get()) }

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

#[test]
#[ignore]
fn probe_sorted_pipeline_pairs() {
    let selected = ["pooling_sppc3pq", "mpbp_48", "faclay75", "crudeoil_pooling_dt3",
        "crudeoil_lee4_10", "chimera_selby-c16-02", "crudeoil_lee4_06", "nuclear104",
        "crudeoil_lee4_09", "arki0016"];
    for (name, pat) in crate::corpus::corpus() {
        if !selected.contains(&name.as_str()) { continue; }
        let mut minima = [f64::MAX; 2];
        for pair in 0..3 {
            let mut outputs = [Vec::new(), Vec::new()];
            for offset in 0..2 {
                let arm = (pair + offset) % 2;
                crate::ordering::sorted_permutation::set_vendor_bypass(arm == 0);
                let t = Instant::now();
                outputs[arm] = order(&pat);
                minima[arm] = minima[arm].min(t.elapsed().as_secs_f64());
            }
            assert_eq!(outputs[0], outputs[1], "{name}, pair={pair}");
        }
        println!("SORTED_PIPELINE\t{name}\t{:.6}\t{:.6}", minima[0], minima[1]);
    }
    crate::ordering::sorted_permutation::set_vendor_bypass(false);
}

#[test]
#[ignore]
fn probe_alt_retirement_pairs() {
    let selected = ["pooling_sppc3pq", "mpbp_48", "crudeoil_pooling_dt3",
        "crudeoil_lee4_10", "chimera_selby-c16-02", "crudeoil_lee4_06", "nuclear104",
        "crudeoil_lee4_09", "arki0016", "gams05"];
    for (name, pat) in crate::corpus::corpus() {
        if !selected.contains(&name.as_str()) { continue; }
        let mut minima = [f64::MAX; 2];
        let mut flops = [0; 2];
        let sp = scoring_pattern(&pat);
        let mut same = true;
        for pair in 0..3 {
            let mut outputs = [Vec::new(), Vec::new()];
            for offset in 0..2 {
                let arm = (pair + offset) % 2;
                ALT_LIMIT.with(|a| a.set(if arm == 0 { 50_000 } else { PEO_ALT_MAX_N }));
                let t = Instant::now();
                outputs[arm] = order(&pat);
                minima[arm] = minima[arm].min(t.elapsed().as_secs_f64());
                assert!(is_bijection(&outputs[arm], pat.n));
                flops[arm] = flops_of(&sp, &outputs[arm]);
            }
            same &= outputs[0] == outputs[1];
        }
        println!("ALT_RETIRE\t{name}\t{:.6}\t{:.6}\t{}\t{}\tperms_equal={same}", minima[0], minima[1], flops[0], flops[1]);
    }
    ALT_LIMIT.with(|a| a.set(PEO_ALT_MAX_N));
}
