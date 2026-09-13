//! Production-side work counter for the residual-core exact-minimum-fill pass.
//!
//! The r7 candidate's gate-5 reading was not resolvable: gate 5 allows 20 rows
//! over +25 ms and 8 over +50 ms, while a base arm compared against *itself* on
//! this box reads 59-60 and 16-20 at two trials per arm. Differencing two
//! 300-row `order()` timings therefore cannot measure the pass, and the round's
//! predicted S1/S2/S3 came instead from `wide_core`'s *reimplementation* of the
//! search — a different code path, run outside `order()`, whose truncated
//! captures finish the elimination that production only sorts.
//!
//! This probe measures the shipped pass itself, at its real call site, on the
//! real cores production builds, in one pass over the corpus:
//!
//! - **charged words per row** — fully deterministic, the ledger's own
//!   denomination, and identical on every run and every host;
//! - **wall time of the shipped search per row** — timed around the call only,
//!   so 133 s of unrelated `order()` work contributes no variance;
//! - the same for [`super::super::minfill_core_order_ref`], the pre-r8
//!   implementation whose dev score and zero-regression row set were measured,
//!   giving the speedup on production's own inputs rather than on synthetic
//!   graphs (whose runaway fill saturates the word summaries);
//! - an **equality assertion** on every real core: same permutation, same
//!   charge. That is the pin that carries the r7 score over unchanged.
//!
//! A per-row search time is not by itself the row's added `order()` time — the
//! pass also spends two exact core scorings, and an accepted pick changes the
//! downstream chain (D10). It *is* an upper bound on the search term and a
//! lower bound on the row's added cost, measured without the noise floor.

use std::cell::{Cell, RefCell};
use std::time::Instant;

use super::{aggregate, bucket, core_of, scoring_pattern, BUCKET_NAMES};
use super::super::{flops_of, minfill_core_order, minfill_core_order_ref, order};

/// One admitted call of the pass, with both implementations timed.
pub(crate) struct Call {
    pub cn: usize,
    pub core_nnz: usize,
    pub budget_before: i64,
    pub charged: i64,
    pub fast_ns: u128,
    pub ref_ns: u128,
    pub truncated: bool,
}

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static CALLS: RefCell<Vec<Call>> = const { RefCell::new(Vec::new()) };
}

/// Called from the pass's own call site in `order_core`, immediately after the
/// production call, with that call's inputs and outputs. Re-runs both
/// implementations on the identical `(core, budget)` pair to time them and to
/// assert the rewrite is observationally identical to the reference.
pub(crate) fn observe(
    cn: usize,
    core_nnz: usize,
    budget_before: i64,
    col_ptr: &[usize],
    row_idx: &[usize],
    perm: &[usize],
    charged: i64,
) {
    if !ENABLED.with(|e| e.get()) {
        return;
    }
    let t = Instant::now();
    let (fast, fast_charge) = minfill_core_order(cn, col_ptr, row_idx, budget_before);
    let fast_ns = t.elapsed().as_nanos();
    let t = Instant::now();
    let (refr, ref_charge) =
        minfill_core_order_ref(cn, col_ptr, row_idx, budget_before);
    let ref_ns = t.elapsed().as_nanos();
    assert_eq!(fast, perm, "the timed re-run must reproduce the production call at cn={cn}");
    assert_eq!(fast_charge, charged, "re-run charge must match production at cn={cn}");
    assert_eq!(
        refr, perm,
        "rewrite and reference must agree on a production core at cn={cn} core_nnz={core_nnz}"
    );
    assert_eq!(
        ref_charge, charged,
        "rewrite and reference must charge alike at cn={cn} core_nnz={core_nnz}"
    );
    CALLS.with(|c| {
        c.borrow_mut().push(Call {
            cn,
            core_nnz,
            budget_before,
            charged,
            fast_ns,
            ref_ns,
            truncated: charged >= budget_before,
        })
    });
}

/// Per-row cost of the shipped residual-core minimum-fill pass, and the
/// pre-r8 reference's cost on the same inputs.
///
/// `MFCALL` is one admitted call; `MFROW` aggregates a row; `MFSTAT` reports
/// the gate-5 statistics the search term alone accounts for, computed from the
/// measured per-row search times rather than differenced out of two noisy
/// 300-row `order()` timings. `MFSCORE` re-reports the dev score so the run
/// doubles as a score control.
#[test]
#[ignore]
fn probe_core_minfill_cost() {
    let corpus = crate::corpus::corpus();
    assert_eq!(corpus.len(), 300, "expected the 300-row dev corpus");
    ENABLED.with(|e| e.set(true));
    let mut log_sums = [0.0f64; 3];
    let mut counts = [0usize; 3];
    let mut fast_row_ms: Vec<(f64, String)> = Vec::new();
    let mut ref_row_ms: Vec<(f64, String)> = Vec::new();
    let mut tot_fast = 0.0f64;
    let mut tot_ref = 0.0f64;
    let mut tot_charged: i64 = 0;
    let mut calls_total = 0usize;
    let mut trunc_total = 0usize;
    let mut rows_touched = 0usize;
    for (name, pat) in &corpus {
        let n = pat.n;
        if n == 0 {
            continue;
        }
        CALLS.with(|c| c.borrow_mut().clear());
        let sp = scoring_pattern(pat);
        let perm = order(pat);
        let calls = CALLS.with(|c| std::mem::take(&mut *c.borrow_mut()));
        let mut row_fast = 0.0f64;
        let mut row_ref = 0.0f64;
        let mut row_charged: i64 = 0;
        for (i, c) in calls.iter().enumerate() {
            row_fast += c.fast_ns as f64 / 1e6;
            row_ref += c.ref_ns as f64 / 1e6;
            row_charged += c.charged;
            println!(
                "MFCALL\t{name}\tcall={i}\tcn={}\tcore_nnz={}\tbudget_before={}\tcharged={}\t\
                 fast_ms={:.4}\tref_ms={:.4}\ttruncated={}",
                c.cn, c.core_nnz, c.budget_before, c.charged,
                c.fast_ns as f64 / 1e6, c.ref_ns as f64 / 1e6, c.truncated as u8
            );
        }
        calls_total += calls.len();
        trunc_total += calls.iter().filter(|c| c.truncated).count();
        if !calls.is_empty() {
            rows_touched += 1;
        }
        tot_fast += row_fast;
        tot_ref += row_ref;
        tot_charged += row_charged;
        fast_row_ms.push((row_fast, name.clone()));
        ref_row_ms.push((row_ref, name.clone()));
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let base = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        );
        let mine = flops_of(&sp, &perm);
        println!(
            "MFROW\t{name}\tn={n}\tnnz={}\tcalls={}\tcharged={row_charged}\t\
             fast_ms={row_fast:.4}\tref_ms={row_ref:.4}\tamd={base}\tmine={mine}",
            pat.nnz(), calls.len()
        );
        let b = bucket(n);
        log_sums[b] += (mine as f64 / base as f64).ln();
        counts[b] += 1;
    }
    ENABLED.with(|e| e.set(false));

    let stat = |rows: &mut Vec<(f64, String)>, label: &str, total: f64| {
        rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let s1 = rows.iter().filter(|r| r.0 > 25.0).count();
        let s2 = rows.iter().filter(|r| r.0 > 50.0).count();
        let s3 = rows.first().map(|r| r.0).unwrap_or(0.0);
        let s3row = rows.first().map(|r| r.1.clone()).unwrap_or_default();
        println!(
            "MFSTAT\t{label}\ttotal_ms={total:.2}\tS1={s1}\tS2={s2}\tS3={s3:.2}\tS3_row={s3row}"
        );
        for r in rows.iter().take(12) {
            println!("MFTOP\t{label}\t{}\t{:.3}", r.1, r.0);
        }
    };
    stat(&mut fast_row_ms, "shipped", tot_fast);
    stat(&mut ref_row_ms, "reference", tot_ref);
    println!(
        "MFSUMMARY\trows_touched={rows_touched}\tcalls={calls_total}\ttruncated={trunc_total}\t\
         charged_total={tot_charged}\tfast_total_ms={tot_fast:.2}\tref_total_ms={tot_ref:.2}\t\
         speedup={:.3}",
        tot_ref / tot_fast.max(1e-9)
    );
    for b in 0..3 {
        if counts[b] > 0 {
            println!(
                "MFBUCKET\t{}\tcount={}\tgeomean={:.16}",
                BUCKET_NAMES[b], counts[b], (log_sums[b] / counts[b] as f64).exp()
            );
        }
    }
    println!("MFSCORE = {:.15}", aggregate(&log_sums, &counts));
}
