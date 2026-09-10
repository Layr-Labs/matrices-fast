//! Bounded, OUTPUT-IDENTICAL parallelism for the candidate portfolio.
//!
//! The graded worker runs on a GitHub-hosted `ubuntu-latest` runner (4 vCPU
//! for a public repository) and the rules permit threads (`RLIMIT_NPROC` is
//! 4096), but the whole candidate portfolio in `leader_order` was
//! single-threaded. Every candidate in it is a PURE function of the pattern —
//! `AMF(α)`, the quotient-metric variants, METIS/Scotch/KaHIP, RCM/Sloan/ND,
//! MinFill, and every relabelled-AMD / -AMF restart — so they can all be
//! GENERATED and SCORED concurrently. What must stay sequential is the
//! ACCEPTANCE decision, and this module is built so that it does.
//!
//! ## Why the output is byte-identical, not merely "usually the same"
//!
//! The sequential `consider` closure was
//!
//! ```ignore
//! if f < best_flops { best_flops = f; best_perm = perm; }
//! ```
//!
//! applied to candidates in a FIXED source order, plus a runner-up ledger fed
//! in that same order. [`run_candidates`] computes the `(flops, perm)` pairs
//! in whatever order the threads finish, then the caller REPLAYS the exact
//! sequential acceptance (and the runner-up ledger) by walking the result
//! vector in the ORIGINAL index order. Thread completion order decides
//! nothing. Every producer is the same pure closure it was before, reads no
//! wall-clock time, process address or `HashMap` iteration order, and the
//! feral crates hold no cross-call global state.
//!
//! ## Batches
//!
//! A batch may only contain producers that do not read the incumbent. The
//! portfolio has natural boundaries where a gate reads `best_flops` (the
//! partitioner cascade's `part_extra` / `part_extra2` and the below-anchor
//! `extra_relabel` tier); the caller flushes the queue at each of them, so
//! every gate sees exactly the value the sequential code saw.
//!
//! ## Memory
//!
//! `keep_all = true` retains every valid candidate's permutation until the
//! replay, because the runner-up ledger needs the best few displaced
//! orderings by value, not just the argmin. The queue is largest on small
//! matrices (many relabel restarts at tiny `n`) and shortest on the giants
//! (the restart budget is `budget / nnz`), so the retained set is a few MB at
//! most. With `keep_all = false` each worker consults a shared monotone
//! minimum and drops permutations that can no longer be the argmin. The
//! thread count is hard-capped at [`PAR_MAX_THREADS`] rather than scaled with
//! `available_parallelism`, so local timing on a bigger box does not silently
//! model a different grader.
//! The batch-local score memo also retains one exact comparison key per distinct
//! permutation; all keys are released when the batch returns.

use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex, OnceLock};

use super::scoring_ws::ScoreWorkspace;
use super::{is_bijection, ScoringPattern};

/// Hard cap on worker threads. The grader is a 4-vCPU runner, so more threads
/// than this can only add scheduling and memory cost.
pub(crate) const PAR_MAX_THREADS: usize = 4;

/// Minimum estimated batch work (`tasks x nnz`) before spawning threads.
/// Every candidate family costs roughly `c * nnz`, so `len * nnz` is the
/// natural work proxy; below this the batch runs in well under a millisecond
/// and thread spawn/join would dominate. Output-neutral: it only selects
/// which of two paths computes the identical result.
pub(crate) const PAR_MIN_WORK: usize = 20_000;

/// A deferred candidate producer: the same signature the sequential
/// `consider` closure took, boxed so the portfolio can collect all of them
/// before any runs. `Sync` (not `Send`) is what the workers need: they only
/// ever hold `&CandFn`.
pub(crate) type CandFn<'a> =
    Box<dyn Fn() -> Result<Vec<i32>, feral_ordering_core::OrderingError> + Sync + 'a>;

/// A deferred candidate producer that already returns a `usize` permutation
/// (the post-hoc phases build these directly rather than through the feral
/// `i32` ordering API).
pub(crate) type PermFn<'a> = Box<dyn Fn() -> Option<Vec<usize>> + Sync + 'a>;

/// One candidate's outcome. `flops == None` means the producer panicked,
/// errored, or returned a non-bijection — exactly the cases the sequential
/// `consider` dropped on the floor. `perm == None` with `flops == Some(_)`
/// means the candidate scored but was already known not to be the argmin and
/// its permutation was released.
#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct CandOut {
    pub(crate) flops: Option<u64>,
    pub(crate) perm: Option<Vec<usize>>,
}

struct ScoreEntry {
    perm: Vec<usize>,
    flops: Arc<OnceLock<u64>>,
}

#[derive(Default)]
struct ScoreMemo {
    entries: Mutex<std::collections::HashMap<u128, Vec<ScoreEntry>>>,
}

impl ScoreMemo {
    fn flops(&self, perm: &[usize], compute: impl FnOnce() -> u64) -> u64 {
        let mut a: u64 = 0x243F_6A88_85A3_08D3;
        let mut b: u64 = 0x1319_8A2E_0370_7344;
        for &v in perm {
            let x = v as u64;
            a = (a ^ x).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            a ^= a >> 32;
            b = (b ^ x).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
            b ^= b >> 29;
        }
        self.flops_with_key(((a as u128) << 64) | b as u128, perm, compute)
    }

    fn flops_with_key(
        &self,
        key: u128,
        perm: &[usize],
        compute: impl FnOnce() -> u64,
    ) -> u64 {
        let memo = {
            let Ok(mut entries) = self.entries.lock() else {
                return compute();
            };
            let bucket = entries.entry(key).or_default();
            if let Some(entry) = bucket.iter().find(|entry| entry.perm == perm) {
                Arc::clone(&entry.flops)
            } else {
                let flops = Arc::new(OnceLock::new());
                bucket.push(ScoreEntry {
                    perm: perm.to_vec(),
                    flops: Arc::clone(&flops),
                });
                flops
            }
        };
        // Reserve before scoring, but do not hold the map lock during scoring.
        // Exact key comparison makes reuse independent of hash collisions.
        *memo.get_or_init(compute)
    }
}

/// Generate + score `tasks` on up to [`PAR_MAX_THREADS`] threads.
///
/// `incumbent` is `best_flops` at batch entry; it seeds the shared minimum
/// used by the drop-early optimisation when `keep_all` is false. The returned
/// vector is indexed by TASK INDEX, not by completion order — that is what
/// lets the caller's sequential replay reproduce the original semantics.
pub(crate) fn run_candidates(
    tasks: &[CandFn<'_>],
    sp: &ScoringPattern,
    n: usize,
    nnz: usize,
    incumbent: u64,
    keep_all: bool,
) -> Vec<CandOut> {
    run_generic(
        tasks.len(),
        sp,
        n,
        nnz,
        incumbent,
        keep_all,
        &|i: usize| match std::panic::catch_unwind(AssertUnwindSafe(|| (tasks[i])())) {
            Ok(Ok(perm_i32)) => Some(perm_i32.into_iter().map(|x| x as usize).collect()),
            _ => None,
        },
    )
}

/// Same as [`run_candidates`] for producers that already speak `Vec<usize>`,
/// with the drop-early optimisation on (only the argmin is ever needed).
pub(crate) fn run_perms(
    tasks: &[PermFn<'_>],
    sp: &ScoringPattern,
    n: usize,
    nnz: usize,
    incumbent: u64,
) -> Vec<CandOut> {
    run_generic(
        tasks.len(),
        sp,
        n,
        nnz,
        incumbent,
        false,
        &|i: usize| match std::panic::catch_unwind(AssertUnwindSafe(|| (tasks[i])())) {
            Ok(v) => v,
            Err(_) => None,
        },
    )
}

/// The shared engine: a lock-free `AtomicUsize` work queue over `len` tasks,
/// one lazily-created [`ScoreWorkspace`] per worker, per-worker result buffers
/// merged back into task-index order after the scope joins.
fn run_generic(
    len: usize,
    sp: &ScoringPattern,
    n: usize,
    nnz: usize,
    incumbent: u64,
    keep_all: bool,
    produce: &(dyn Fn(usize) -> Option<Vec<usize>> + Sync),
) -> Vec<CandOut> {
    let mut results: Vec<CandOut> = Vec::with_capacity(len);
    for _ in 0..len {
        results.push(CandOut::default());
    }
    if len == 0 {
        return results;
    }

    // Monotone shared upper bound on the final minimum. Only ever decreases.
    let gmin = AtomicU64::new(incumbent);

    // One exact key and score cell per distinct permutation in this batch.
    // Simultaneous aliases wait for the same symbolic pass rather than all
    // missing a lookup-before-insert cache.
    let dedup = ScoreMemo::default();

    let eval = |i: usize, ws: &mut Option<ScoreWorkspace>| -> CandOut {
        let Some(perm) = produce(i) else {
            return CandOut::default();
        };
        if !is_bijection(&perm, n) {
            return CandOut::default();
        }
        let f = dedup.flops(&perm, || {
            let w = ws.get_or_insert_with(|| ScoreWorkspace::new(n, nnz));
            w.flops(sp, &perm)
        });
        let prev = gmin.fetch_min(f, AtomicOrdering::Relaxed);
        CandOut {
            flops: Some(f),
            // Retain only while this candidate can still be the argmin.
            perm: if keep_all || f <= prev { Some(perm) } else { None },
        }
    };
    // A worker thread that unwound would silently lose a whole batch of
    // results; wrapping here makes a worker panic-free by construction.
    let eval = |i: usize, ws: &mut Option<ScoreWorkspace>| -> CandOut {
        std::panic::catch_unwind(AssertUnwindSafe(|| eval(i, ws))).unwrap_or_default()
    };

    if len == 1
        || len.saturating_mul(nnz.max(1)) < PAR_MIN_WORK
        || in_worker()
        || force_sequential()
    {
        let mut ws: Option<ScoreWorkspace> = None;
        for (i, slot) in results.iter_mut().enumerate() {
            *slot = eval(i, &mut ws);
        }
        return results;
    }

    let next = AtomicUsize::new(0);
    let nthreads = PAR_MAX_THREADS.min(len);
    let collected: Vec<Vec<(usize, CandOut)>> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..nthreads)
            .map(|_| {
                s.spawn(|| {
                    // Nested batches (a TELOS descent's per-round batch inside
                    // the multi-descent batch) run sequentially — see
                    // `IN_WORKER`.
                    IN_WORKER.with(|c| c.set(true));
                    let mut ws: Option<ScoreWorkspace> = None;
                    let mut out: Vec<(usize, CandOut)> = Vec::new();
                    loop {
                        let i = next.fetch_add(1, AtomicOrdering::Relaxed);
                        if i >= len {
                            break;
                        }
                        out.push((i, eval(i, &mut ws)));
                    }
                    out
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_default())
            .collect()
    });
    for part in collected {
        for (i, o) in part {
            results[i] = o;
        }
    }
    results
}

/// Sequential acceptance over [`run_generic`]'s task-indexed results,
/// reproducing the running-minimum semantics of a sequential `if f < best`
/// loop exactly: install the EARLIEST task index attaining the strict minimum
/// below `best_flops`, and return whether anything was installed.
pub(crate) fn accept(results: &mut [CandOut], best_flops: &mut u64, best_perm: &mut Vec<usize>) -> bool {
    let mut win: Option<(usize, u64)> = None;
    for (i, r) in results.iter().enumerate() {
        let Some(f) = r.flops else { continue };
        if f < *best_flops && win.is_none_or(|(_, bf)| f < bf) {
            win = Some((i, f));
        }
    }
    let Some((i, f)) = win else { return false };
    // The retention invariant (module doc) guarantees the winner kept its
    // permutation; declining is the safe response otherwise (the incumbent
    // is always a valid ordering).
    let Some(p) = results[i].perm.take() else {
        debug_assert!(false, "argmin candidate dropped its permutation");
        return false;
    };
    *best_flops = f;
    *best_perm = p;
    true
}

/// TEST-ONLY switch forcing every batch onto the sequential path, so a probe
/// can run the SAME `order()` both ways in one process and compare the
/// returned permutations element by element. Production builds compile
/// [`force_sequential`] down to a constant `false`.
#[cfg(test)]
thread_local! {
    pub(crate) static FORCE_SEQUENTIAL: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
#[inline]
fn force_sequential() -> bool {
    FORCE_SEQUENTIAL.with(|c| c.get())
}

#[cfg(not(test))]
#[inline(always)]
fn force_sequential() -> bool {
    false
}

// Set on a worker thread for as long as it is draining a batch, so a batch
// queued from INSIDE a worker runs sequentially instead of oversubscribing
// the cores. Output-neutral.
thread_local! {
    static IN_WORKER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[inline]
fn in_worker() -> bool {
    IN_WORKER.with(|c| c.get())
}

/// TEST-ONLY record of what stage 1b decided on the current row, so a probe can
/// size the deferred-lift population (the cost driver of running the polish
/// chain on both candidates) rather than infer it from rows whose final
/// permutation happened to change. Fields: `(kind, lift_flops, incumbent_flops,
/// accepted_at_4b)` with `kind` 0 = no lift produced / not better, 1 = adopted
/// at 1b by the size gate, 2 = adopted at 1b by the margin, 3 = deferred.
#[cfg(test)]
thread_local! {
    pub(crate) static INDEP_TRACE: std::cell::Cell<(u8, u64, u64, bool)> =
        const { std::cell::Cell::new((0, 0, 0, false)) };
}

#[cfg(test)]
#[inline]
pub(crate) fn indep_trace_set(kind: u8, lift: u64, incumbent: u64) {
    INDEP_TRACE.with(|c| c.set((kind, lift, incumbent, false)));
}

#[cfg(test)]
#[inline]
pub(crate) fn indep_trace_accept() {
    INDEP_TRACE.with(|c| {
        let (k, l, i, _) = c.get();
        c.set((k, l, i, true));
    });
}

#[cfg(test)]
pub(crate) fn indep_trace_take() -> (u8, u64, u64, bool) {
    INDEP_TRACE.with(|c| c.replace((0, 0, 0, false)))
}

/// TEST-ONLY per-phase wall-clock marks: `phase_mark(label, t0)` appends
/// `(label, secs since t0)`; `phase_take()` drains them. Lets a probe see which
/// stage of `order()` sets a row's critical path. Never compiled into the
/// shipped binary.
#[cfg(test)]
thread_local! {
    pub(crate) static PHASE_MARKS: std::cell::RefCell<Vec<(&'static str, f64, u64)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}
#[cfg(test)]
pub(crate) fn phase_mark(label: &'static str, t0: std::time::Instant, flops: u64) {
    let secs = t0.elapsed().as_secs_f64();
    PHASE_MARKS.with(|m| m.borrow_mut().push((label, secs, flops)));
}
#[cfg(test)]
pub(crate) fn phase_take() -> Vec<(&'static str, f64, u64)> {
    PHASE_MARKS.with(|m| std::mem::take(&mut *m.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;

    #[test]
    fn alias_score_cache_computes_once_across_workers() {
        let memo = ScoreMemo::default();
        let count = AtomicUsize::new(0);
        let barrier = Barrier::new(PAR_MAX_THREADS);
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..PAR_MAX_THREADS)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        memo.flops(&[2, 0, 1], || {
                            count.fetch_add(1, AtomicOrdering::Relaxed);
                            42
                        })
                    })
                })
                .collect();
            for handle in handles {
                assert_eq!(handle.join().unwrap(), 42);
            }
        });
        assert_eq!(count.load(AtomicOrdering::Relaxed), 1);
    }

    #[test]
    fn alias_score_cache_compares_colliding_keys_exactly() {
        let memo = ScoreMemo::default();
        assert_eq!(memo.flops_with_key(0, &[0, 1, 2], || 11), 11);
        assert_eq!(memo.flops_with_key(0, &[2, 0, 1], || 22), 22);
        assert_eq!(memo.flops_with_key(0, &[0, 1, 2], || panic!("rescored")), 11);
        assert_eq!(memo.flops_with_key(0, &[2, 0, 1], || panic!("rescored")), 22);
    }
}
