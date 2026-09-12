//! Test-only neighborhood screens on the promoted, finished incumbent.
use super::*;

thread_local! { static ENABLED: std::cell::Cell<bool> = std::cell::Cell::new(true); }
pub(in crate::ordering) fn enabled() -> bool { ENABLED.with(|c| c.get()) }

#[test]
#[ignore]
fn probe_terminal_followup_stress() {
    let mut fixtures = Vec::new();
    for (n, links) in [(2_048, 4), (2_048, 20), (2_048, 40),
        (8_000, 2), (8_000, 4), (8_000, 8), (8_000, 10), (12_000, 4)] {
        let mut state = 0x587a_2834_d139_u64 ^ n as u64 ^ links as u64;
        let mut edges = Vec::new();
        for v in 0..n { for _ in 0..links {
            state ^= state << 13; state ^= state >> 7; state ^= state << 17;
            let u = state as usize % n;
            if u != v { edges.push((v.min(u), v.max(u))); }
        } }
        edges.sort_unstable(); edges.dedup();
        fixtures.push((format!("random_n{n}_links{links}"), Pattern::from_edges(n, &edges)));
    }
    for grid in [32usize, 64] {
        let mut edges = Vec::new();
        for y in 0..grid { for x in 0..grid {
            let v = y * grid + x;
            if x + 1 < grid { edges.push((v, v + 1)); }
            if y + 1 < grid { edges.push((v, v + grid)); }
        } }
        fixtures.push((format!("grid{grid}"), Pattern::from_edges(grid * grid, &edges)));
    }
    fixtures.push(("hub2048".to_owned(), Pattern::from_edges(2_048,
        &(1..2_048).map(|v| (0, v)).collect::<Vec<_>>())));
    for (name, pat) in fixtures {
        let sp = scoring_pattern(&pat);
        let mut ws = scoring_ws::ScoreWorkspace::new(pat.n, pat.nnz());
        let mut minima = [f64::MAX; 2];
        let mut outputs = [Vec::new(), Vec::new()];
        let mut flops = [0; 2];
        for pair in 0..2 {
            for offset in 0..2 {
                let arm = (pair + offset) % 2;
                ENABLED.with(|c| c.set(arm == 1));
                let t = Instant::now();
                let p = order(&pat);
                minima[arm] = minima[arm].min(t.elapsed().as_secs_f64());
                assert!(is_bijection(&p, pat.n));
                if pair != 0 { assert_eq!(p, outputs[arm], "{name} arm={arm}"); }
                outputs[arm] = p;
                flops[arm] = ws.flops(&sp, &outputs[arm]);
            }
        }
        assert!(flops[1] <= flops[0], "{name}: {flops:?}");
        if flops[0] > 20_000_000_000 { assert_eq!(outputs[0], outputs[1]); }
        println!("FOLLOWUP_STRESS\t{name}\t{}\t{}\t{:.6}\t{:.6}\t{}\t{}",
            pat.n, pat.nnz(), minima[0], minima[1], flops[0], flops[1]);
    }
    ENABLED.with(|c| c.set(true));
}

fn completion_step(pat: &Pattern, current: &mut Vec<usize>, flops: &mut u64,
    builder: &mut sorted_permutation::SortedPermutation<'_>,
    ws: &mut scoring_ws::ScoreWorkspace, sp: &ScoringPattern, budget: i64) -> bool {
    let pp = builder.permute(current);
    let et = EliminationTree::from_pattern(&pp);
    let counts = column_counts_gnp(&pp, &et);
    if let Some(p) = completion::refine_limited(pat.n, &pat.col_ptr, &pat.row_idx,
        &pp.col_ptr, &pp.row_idx, &et.parent, &counts, current, budget) {
        assert!(is_bijection(&p, pat.n));
        let f = ws.flops(sp, &p);
        if f < *flops { *current = p; *flops = f; return true; }
    }
    false
}

fn peo_step(pat: &Pattern, current: &mut Vec<usize>, flops: &mut u64,
    builder: &mut sorted_permutation::SortedPermutation<'_>,
    ws: &mut scoring_ws::ScoreWorkspace, sp: &ScoringPattern) {
    for _ in 0..2 {
        let before = *flops;
        let pp = builder.permute(current);
        let et = EliminationTree::from_pattern(&pp);
        let counts = column_counts_gnp(&pp, &et);
        if let Some(ps) = peo_extract::candidates_bounded(pat.n,
            &pp.col_ptr, &pp.row_idx, &et.parent, &counts, current,
            12_000, 200_000, 300_000) {
            for p in ps {
                let f = ws.flops(sp, &p);
                if f < *flops { *current = p; *flops = f; }
            }
        }
        if *flops == before { break; }
    }
}

#[test]
#[ignore]
fn probe_terminal_followup() {
    let arms: &[&[(usize, usize, usize, i64)]] = &[
        &[],
        &[(48, 4, 19, 16_000_000), (9, 4, 4, 16_000_000), (8, 4, 3, 32_000_000)],
        &[(48, 4, 19, 32_000_000), (9, 4, 4, 32_000_000), (8, 8, 3, 64_000_000)],
        &[(48, 4, 19, 16_000_000), (9, 4, 4, 16_000_000), (8, 4, 3, 32_000_000)],
        &[(48, 4, 19, 16_000_000), (9, 4, 4, 16_000_000), (8, 4, 3, 32_000_000)],
        &[(48, 4, 19, 32_000_000), (9, 4, 4, 32_000_000), (8, 8, 3, 64_000_000)],
    ];
    let cache_path = std::env::var("SSI_TERMINAL_SEED_CACHE").ok();
    let mut cache = std::collections::BTreeMap::new();
    if let Some(path) = &cache_path {
        if let Ok(contents) = std::fs::read_to_string(path) {
            for line in contents.lines() {
                let (name, values) = line.split_once('\t').unwrap();
                cache.insert(name.to_owned(), values.split(',')
                    .map(|s| s.parse::<usize>().unwrap()).collect::<Vec<_>>());
            }
        }
    }
    let mut output = String::new();
    let mut sums = [[0.0; 3]; 7];
    let mut counts = [0; 3];
    let mut wins = [[0; 3]; 6];
    let mut seconds = [0.0; 6];
    let mut maxima = [0.0f64; 6];
    for (name, pat) in crate::corpus::corpus() {
        let incumbent = cache.remove(&name).unwrap_or_else(|| order(&pat));
        assert!(is_bijection(&incumbent, pat.n));
        output.push_str(&name); output.push('\t');
        for (i, v) in incumbent.iter().enumerate() {
            if i != 0 { output.push(','); }
            output.push_str(&v.to_string());
        }
        output.push('\n');
        let sp = scoring_pattern(&pat);
        let mut builder = sorted_permutation::SortedPermutation::new(&sp);
        let mut ws = scoring_ws::ScoreWorkspace::new(pat.n, pat.nnz());
        let reference = ws.flops(&sp, &incumbent);
        let (cp, ri) = core_of(&pat);
        let core = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
        let amd: Vec<usize> = feral_amd::amd_order(&core).unwrap()
            .into_iter().map(|v| v as usize).collect();
        let baseline = ws.flops(&sp, &amd);
        let b = bucket(pat.n);
        counts[b] += 1;
        sums[0][b] += (reference as f64 / baseline as f64).ln();
        let gate = pat.n >= 6 && pat.n <= rgreedy::MAX_N && pat.nnz() <= 200_000
            && reference <= 20_000_000_000;
        let mut row = [reference; 6];
        for (arm, chain) in arms.iter().enumerate() {
            if gate {
                let t = Instant::now();
                let mut current = incumbent.clone();
                let max_degree = pat.col_ptr.windows(2).map(|p| p[1] - p[0]).max().unwrap_or(0);
                if pat.nnz() > 16 * pat.n || max_degree > pat.n / 2 {
                    if let Some(p) = rgreedy::subset_window_descent_step(pat.n,
                        &pat.col_ptr, &pat.row_idx, &current, 8, 4, 3, 64_000_000) {
                        let f = ws.flops(&sp, &p);
                        if f < row[arm] { current = p; row[arm] = f; }
                    }
                }
                if arm >= 4 { completion_step(&pat, &mut current, &mut row[arm],
                    &mut builder, &mut ws, &sp, 4_000_000); }
                peo_step(&pat, &mut current, &mut row[arm], &mut builder, &mut ws, &sp);
                if arm == 1 || arm == 2 { completion_step(&pat, &mut current, &mut row[arm],
                    &mut builder, &mut ws, &sp, 4_000_000); }
                for &(width, sweeps, step, budget) in *chain {
                    if let Some(p) = rgreedy::sparse_span_window_descent(pat.n,
                        &pat.col_ptr, &pat.row_idx, &current, width, sweeps, step, budget) {
                        assert!(is_bijection(&p, pat.n));
                        let f = ws.flops(&sp, &p);
                        assert!(f <= row[arm], "{name}, arm={arm}");
                        if f < row[arm] { current = p; row[arm] = f; }
                    }
                }
                let elapsed = t.elapsed().as_secs_f64();
                seconds[arm] += elapsed;
                maxima[arm] = maxima[arm].max(elapsed);
            }
            wins[arm][b] += usize::from(row[arm] < reference);
            sums[arm + 1][b] += (row[arm] as f64 / baseline as f64).ln();
        }
        println!("FOLLOWUP_ROW\t{name}\t{}\t{}\t{baseline}\t{reference}\t{}\t{}\t{}\t{}\t{}\t{}",
            pat.n, pat.nnz(), row[0], row[1], row[2], row[3], row[4], row[5]);
    }
    if let Some(path) = &cache_path { std::fs::write(path, output).unwrap(); }
    let base = aggregate(&sums[0], &counts);
    assert!((base - 0.792029985254).abs() < 1e-11, "cache/base mismatch: {base}");
    println!("FOLLOWUP_BASE score={base:.12}");
    for (arm, chain) in arms.iter().enumerate() {
        println!("FOLLOWUP_TOTAL arm={arm} chain={chain:?} score={:.12} wins={:?} seconds={:.6} max={:.6}",
            aggregate(&sums[arm + 1], &counts), wins[arm], seconds[arm], maxima[arm]);
    }
}
