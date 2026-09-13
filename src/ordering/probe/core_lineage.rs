//! Test-only exact residual-core lineage screen. No production state changes.
use super::*;

// Fixed per-core seeds; corpus traversal order cannot affect a candidate.
const LABELS: [&str; 8] = [
    "md_index", "minfill_index", "mcs_index", "md_seed_1",
    "md_seed_2", "md_seed_3", "md_seed_4", "minfill_degree",
];

#[derive(Default)]
struct Work {
    scans: u64,
    popcount_words: u64,
    deficiency_words: u64,
    clique_words: u64,
}

// A live symmetric graph in the same five-word layout as SmallScore. All
// work-driving loops are charged, including full candidate and word scans.
fn greedy(small: &SmallScore, minfill: bool, degree_ties: bool, seed: u64)
    -> (Vec<usize>, u64, Work)
{
    let n = small.n;
    let words = (n + 63) / 64;
    let mut rows = small.rows.clone();
    let mut live = vec![true; n];
    let mut output = Vec::with_capacity(n);
    let mut work = Work::default();
    let mut flops = 0;
    let mut state = seed;
    for _ in 0..n {
        let mut best = usize::MAX;
        let mut best_key = (u64::MAX, usize::MAX, u64::MAX);
        for v in 0..n {
            work.scans += 1;
            if !live[v] { continue; }
            work.popcount_words += words as u64;
            let degree = rows[v][..words].iter().map(|w| w.count_ones() as usize).sum::<usize>();
            let metric = if minfill {
                let mut twice_edges = 0u64;
                for j in 0..words {
                    let mut bits = rows[v][j];
                    while bits != 0 {
                        let u = 64 * j + bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        for k in 0..words {
                            twice_edges += (rows[u][k] & rows[v][k]).count_ones() as u64;
                            work.deficiency_words += 1;
                        }
                    }
                }
                ((degree * degree.saturating_sub(1)) as u64 - twice_edges) / 2
            } else { degree as u64 };
            let tie = if seed == 0 { v as u64 } else { splitmix64(&mut state) };
            let key = (metric, if degree_ties { degree } else { 0 }, tie);
            if key < best_key { best_key = key; best = v; }
        }
        assert_ne!(best, usize::MAX);
        let neighbors = rows[best];
        let degree = neighbors[..words].iter().map(|w| w.count_ones() as u64).sum::<u64>();
        flops += (degree + 1) * (degree + 1);
        output.push(best);
        live[best] = false;
        for j in 0..words {
            let mut bits = neighbors[j];
            while bits != 0 {
                let u = 64 * j + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                for k in 0..words {
                    rows[u][k] |= neighbors[k];
                    work.clique_words += 1;
                }
                rows[u][u / 64] &= !(1 << (u % 64));
                rows[u][best / 64] &= !(1 << (best % 64));
            }
        }
    }
    (output, flops, work)
}

fn mcs(small: &SmallScore) -> (Vec<usize>, Work) {
    let n = small.n;
    let words = (n + 63) / 64;
    let mut weights = vec![0usize; n];
    let mut live = vec![true; n];
    let mut out = Vec::with_capacity(n);
    let mut work = Work::default();
    for _ in 0..n {
        let mut best = usize::MAX;
        for v in 0..n {
            work.scans += 1;
            if live[v] && (best == usize::MAX || weights[v] > weights[best]) { best = v; }
        }
        out.push(best);
        live[best] = false;
        for j in 0..words {
            let mut bits = small.rows[best][j];
            work.popcount_words += 1;
            while bits != 0 {
                let u = 64 * j + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if live[u] { weights[u] += 1; }
                work.clique_words += 1;
            }
        }
    }
    out.reverse();
    (out, work)
}

#[test]
#[ignore]
fn probe_core_lineage_screen() {
    CORE_CAPTURE_ENABLED.with(|enabled| enabled.set(true));
    let corpus = crate::corpus::corpus();
    assert_eq!(corpus.len(), 300);
    let mut counts = [0usize; 3];
    let mut baseline_logs = [0.0f64; 3];
    let mut policy_logs = [[0.0f64; 3]; 9];
    let mut winners = [0usize; 9];
    let mut eligible_rows = 0;
    let mut eligible_captures = 0;
    for (name, pat) in &corpus {
        let _ = take_core_candidates();
        let incumbent = order(pat);
        let captures = take_core_candidates();
        let sp = scoring_pattern(pat);
        let inc = flops_of(&sp, &incumbent);
        let (cp, ri) = core_of(pat);
        let raw = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
        let amd_perm: Vec<usize> = feral_amd::amd_order(&raw).unwrap().into_iter().map(|v| v as usize).collect();
        let amd = flops_of(&sp, &amd_perm);
        let b = bucket(pat.n);
        counts[b] += 1;
        baseline_logs[b] += (inc as f64 / amd as f64).ln();
        let mut row_best = [inc; 9];
        let mut paid = 0;
        let mut row_ns = 0;
        for (capture, c) in captures.iter().enumerate() {
            if !(12..=300).contains(&c.cn) || c.row_idx.len() > 3000 { continue; }
            let start = Instant::now();
            paid += 1;
            eligible_captures += 1;
            let core_pat = Pattern { n: c.cn, col_ptr: c.col_ptr.clone(), row_idx: c.row_idx.clone() };
            let core_sp = scoring_pattern(&core_pat);
            let small = SmallScore::new(&core_pat);
            let core_inc = flops_of(&core_sp, &c.core_perm);
            assert_eq!(core_inc, small.flops(&c.core_perm), "{name}: captured core");
            println!("LINEAGECORE\t{name}\tcapture={capture}\tcn={}\tcnnz={}\tprefix={}\tcore_inc={core_inc}", c.cn, c.row_idx.len(), c.prefix_flops);
            for policy in 0..LABELS.len() {
                let timer = Instant::now();
                let (p, direct, work) = match policy {
                    0 => greedy(&small, false, false, 0),
                    1 => greedy(&small, true, false, 0),
                    2 => { let (p, work) = mcs(&small); (p, u64::MAX, work) },
                    3..=6 => greedy(&small, false, false, 0xC0FFEE + policy as u64 - 3),
                    7 => greedy(&small, true, true, 0),
                    _ => unreachable!(),
                };
                let search_ns = timer.elapsed().as_nanos();
                assert!(is_bijection(&p, c.cn), "{name}: {}", LABELS[policy]);
                let f = flops_of(&core_sp, &p);
                assert_eq!(f, small.flops(&p), "{name}: {}", LABELS[policy]);
                if policy != 2 { assert_eq!(f, direct, "{name}: direct cost"); }
                if policy == 7 {
                    let shipped: Vec<usize> = minfill_order(&core_pat).into_iter().map(|v| v as usize).collect();
                    // Bitset minfill has no fallback. Report its agreement with
                    // the shipped capped pair-scan implementation explicitly.
                    println!("LINEAGECONTROL\t{name}\tcapture={capture}\tflops={}\tbitset_flops={f}\tsame_perm={}", flops_of(&core_sp, &shipped), shipped == p);
                }
                let total = c.prefix_flops + f;
                row_best[policy] = row_best[policy].min(total);
                if policy < 7 { row_best[8] = row_best[8].min(total); }
                println!("LINEAGECAND\t{name}\tcapture={capture}\tpolicy={}\tcore={f}\ttotal={total}\tsearch_ns={search_ns}\tscans={}\tpopcount_words={}\tdeficiency_words={}\tclique_words={}", LABELS[policy], work.scans, work.popcount_words, work.deficiency_words, work.clique_words);
            }
            row_ns += start.elapsed().as_nanos();
        }
        if paid > 0 { eligible_rows += 1; }
        for policy in 0..9 {
            policy_logs[policy][b] += (row_best[policy] as f64 / amd as f64).ln();
            if row_best[policy] < inc { winners[policy] += 1; }
        }
        println!("LINEAGEBASE\t{name}\tn={}\tnnz={}\tbase={amd}\tinc={inc}\tpaid={paid}\tall_captures={}\trow_ns={row_ns}\tbest={}", pat.n, pat.nnz(), captures.len(), row_best[8]);
    }
    CORE_CAPTURE_ENABLED.with(|enabled| enabled.set(false));
    let baseline = aggregate(&baseline_logs, &counts);
    println!("LINEAGESUMMARY\tcontrol={baseline:.16}\teligible_rows={eligible_rows}\teligible_captures={eligible_captures}");
    for policy in 0..9 {
        let label = if policy == 8 { "combined" } else { LABELS[policy] };
        let score = aggregate(&policy_logs[policy], &counts);
        println!("LINEAGESCORE\tpolicy={label}\tscore={score:.16}\tbips={:.8}\twinners={}", (baseline - score) * 10000.0, winners[policy]);
    }
}
