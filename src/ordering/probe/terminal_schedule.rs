//! Temporary test-only terminal schedule screen; complete orderings are scored.
use super::*;
use std::cell::{Cell, RefCell};

thread_local! {
    static ACTIVE: Cell<bool> = Cell::new(false);
    static ENTRY: RefCell<Option<Vec<usize>>> = RefCell::new(None);
}

pub(in crate::ordering) fn capture(perm: &[usize]) {
    if ACTIVE.with(|a| a.get()) { ENTRY.with(|e| *e.borrow_mut() = Some(perm.to_vec())); }
}

#[test]
#[ignore]
fn probe_terminal_schedule() {
    let arms: [(usize, usize, i64); 6] = [(8, 3, 32_000_000), (8, 2, 32_000_000),
        (9, 4, 32_000_000), (10, 3, 32_000_000), (13, 5, 32_000_000), (14, 5, 32_000_000)];
    let mut sums = [[0.0; 3]; 6];
    let mut counts = [0; 3];
    let mut wins = [0; 6];
    let mut losses = [0; 6];
    let mut seconds = [0.0; 6];
    ACTIVE.with(|a| a.set(true));
    for (name, pat) in crate::corpus::corpus() {
        ENTRY.with(|e| *e.borrow_mut() = None);
        let incumbent = order(&pat);
        let entry = ENTRY.with(|e| e.borrow_mut().take());
        let sp = scoring_pattern(&pat);
        let mut ws = scoring_ws::ScoreWorkspace::new(pat.n, pat.nnz());
        let reference = ws.flops(&sp, &incumbent);
        let (cp, ri) = core_of(&pat);
        let core = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
        let amd: Vec<usize> = feral_amd::amd_order(&core).unwrap().into_iter().map(|v| v as usize).collect();
        let baseline = ws.flops(&sp, &amd);
        let b = bucket(pat.n);
        counts[b] += 1;
        let Some(mut prefix) = entry else {
            for sum in &mut sums { sum[b] += (reference as f64 / baseline as f64).ln(); }
            continue;
        };
        let mut prefix_score = ws.flops(&sp, &prefix);
        if let Some(cand) = rgreedy::subset_window_descent(pat.n, &pat.col_ptr, &pat.row_idx,
            &prefix, 10, 2, 24_000_000) {
            let f = ws.flops(&sp, &cand);
            if f < prefix_score { prefix_score = f; prefix = cand; }
        }
        if let Some(cand) = rgreedy::subset_window_descent_step(pat.n, &pat.col_ptr, &pat.row_idx,
            &prefix, 12, 4, 5, 64_000_000) {
            let f = ws.flops(&sp, &cand);
            if f < prefix_score { prefix_score = f; prefix = cand; }
        }
        for (arm, &(width, step, budget)) in arms.iter().enumerate() {
            let t = Instant::now();
            let mut candidate = prefix.clone();
            let mut f = prefix_score;
            if let Some(p) = rgreedy::subset_window_descent_step(pat.n, &pat.col_ptr, &pat.row_idx,
                &candidate, width, 4, step, budget) {
                let score = ws.flops(&sp, &p);
                if score < f { f = score; candidate = p; }
            }
            seconds[arm] += t.elapsed().as_secs_f64();
            assert!(is_bijection(&candidate, pat.n));
            if arm == 0 { assert_eq!(candidate, incumbent, "{name}: control replay"); }
            wins[arm] += usize::from(f < reference);
            losses[arm] += usize::from(f > reference);
            sums[arm][b] += (f as f64 / baseline as f64).ln();
            println!("TERMINAL_ARM\t{name}\t{arm}\t{reference}\t{f}");
        }
    }
    ACTIVE.with(|a| a.set(false));
    for (arm, &(width, step, budget)) in arms.iter().enumerate() {
        println!("TERMINAL_SCORE arm={arm} width={width} step={step} budget={budget} score={:.12} wins={} losses={} seconds={:.6}",
            aggregate(&sums[arm], &counts), wins[arm], losses[arm], seconds[arm]);
    }
}
