//! Diagnostic exact-window screen after the promoted terminal greedy search.
use super::*;

thread_local! { static TAIL_ENABLED: std::cell::Cell<bool> = std::cell::Cell::new(true); }
pub(in crate::ordering) fn exchange_enabled() -> bool { TAIL_ENABLED.with(|c| c.get()) }

#[test]
#[ignore]
fn probe_fenced_terminal_stress() {
    use crate::Pattern;
    let mut fixtures = Vec::new();
    for (n, links) in [(2_048, 4), (8_000, 2), (8_000, 4), (8_000, 8),
        (8_000, 10), (12_000, 4)] {
        let mut state = 0x587a_2834_d139_u64 ^ n as u64 ^ links as u64;
        let mut edges = Vec::new();
        for v in 0..n {
            for _ in 0..links {
                state ^= state << 13; state ^= state >> 7; state ^= state << 17;
                let u = state as usize % n;
                if u != v { edges.push((v.min(u), v.max(u))); }
            }
        }
        edges.sort_unstable(); edges.dedup();
        fixtures.push((format!("random_n{n}_links{links}"), Pattern::from_edges(n, &edges)));
    }
    let grid = 64;
    let mut edges = Vec::new();
    for y in 0..grid { for x in 0..grid {
        let v = y * grid + x;
        if x + 1 < grid { edges.push((v, v + 1)); }
        if y + 1 < grid { edges.push((v, v + grid)); }
    } }
    fixtures.push(("grid64".to_owned(), Pattern::from_edges(grid * grid, &edges)));
    fixtures.push(("hub2048".to_owned(), Pattern::from_edges(2_048,
        &(1..2_048).map(|v| (0, v)).collect::<Vec<_>>())));
    for (name, pat) in fixtures {
        let sp = scoring_pattern(&pat);
        let mut ws = scoring_ws::ScoreWorkspace::new(pat.n, pat.nnz());
        let mut outputs = [Vec::new(), Vec::new()];
        let mut seconds = [0.0; 2];
        let mut flops = [0; 2];
        for arm in 0..2 {
            sorted_permutation::set_vendor_bypass(arm == 0);
            peo_extract::set_pooled_reference(arm == 0);
            TAIL_ENABLED.with(|c| c.set(arm == 1));
            let t = Instant::now();
            outputs[arm] = order(&pat);
            seconds[arm] = t.elapsed().as_secs_f64();
            assert!(is_bijection(&outputs[arm], pat.n));
            flops[arm] = ws.flops(&sp, &outputs[arm]);
        }
        assert!(flops[1] <= flops[0], "{name}: {:?}", flops);
        println!("FENCED_STRESS\t{name}\t{}\t{}\t{:.6}\t{:.6}\t{}\t{}",
            pat.n, pat.nnz(), seconds[0], seconds[1], flops[0], flops[1]);
    }
    sorted_permutation::set_vendor_bypass(false);
    peo_extract::set_pooled_reference(false);
    TAIL_ENABLED.with(|c| c.set(true));
}

#[test]
#[ignore]
fn probe_leader_kernel_pairs() {
    TAIL_ENABLED.with(|c| c.set(false));
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
                sorted_permutation::set_vendor_bypass(arm == 0);
                peo_extract::set_pooled_reference(arm == 0);
                let t = Instant::now();
                outputs[arm] = order(&pat);
                minima[arm] = minima[arm].min(t.elapsed().as_secs_f64());
            }
            assert_eq!(outputs[0], outputs[1], "{name}, pair={pair}");
        }
        println!("LEADER_KERNEL_PAIR\t{name}\t{:.6}\t{:.6}", minima[0], minima[1]);
    }
    sorted_permutation::set_vendor_bypass(false);
    peo_extract::set_pooled_reference(false);
    TAIL_ENABLED.with(|c| c.set(true));
}

#[test]
#[ignore]
fn probe_leader_terminal_windows() {
    TAIL_ENABLED.with(|c| c.set(false));
    let arms = [(8, 3, 32_000_000), (8, 3, 64_000_000),
        (7, 2, 32_000_000), (10, 3, 32_000_000)];
    let mut sums = [[0.0; 3]; 5];
    let mut counts = [0; 3];
    let mut wins = [[0; 3]; 4];
    let mut seconds = [0.0; 4];
    for (name, pat) in crate::corpus::corpus() {
        let incumbent = order(&pat);
        let sp = scoring_pattern(&pat);
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
        let max_degree = pat.col_ptr.windows(2).map(|p| p[1] - p[0]).max().unwrap_or(0);
        let gate = pat.n >= 6 && pat.n <= rgreedy::MAX_N && pat.nnz() <= 200_000
            && pat.nnz() <= 16 * pat.n && max_degree <= pat.n / 2;
        let mut row = [reference; 4];
        for (arm, &(width, step, budget)) in arms.iter().enumerate() {
            if gate {
                let t = Instant::now();
                if let Some(p) = rgreedy::subset_window_descent_step(pat.n,
                    &pat.col_ptr, &pat.row_idx, &incumbent, width, 4, step, budget) {
                    assert!(is_bijection(&p, pat.n));
                    row[arm] = ws.flops(&sp, &p);
                    assert!(row[arm] <= reference, "{name}, arm={arm}");
                }
                seconds[arm] += t.elapsed().as_secs_f64();
            }
            wins[arm][b] += usize::from(row[arm] < reference);
            sums[arm + 1][b] += (row[arm] as f64 / baseline as f64).ln();
        }
        println!("LEADER_TAIL_ROW\t{name}\t{}\t{}\t{baseline}\t{reference}\t{}\t{}\t{}\t{}",
            pat.n, pat.nnz(), row[0], row[1], row[2], row[3]);
    }
    println!("LEADER_TAIL_BASE score={:.12}", aggregate(&sums[0], &counts));
    for (arm, &(width, step, budget)) in arms.iter().enumerate() {
        println!("LEADER_TAIL_TOTAL width={width} step={step} budget={budget} score={:.12} wins={:?} seconds={:.6}",
            aggregate(&sums[arm + 1], &counts), wins[arm], seconds[arm]);
    }
    TAIL_ENABLED.with(|c| c.set(true));
}
