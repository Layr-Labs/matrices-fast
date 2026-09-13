//! Test-only stage-8 alternate-seed lineage-source screen. No production
//! state changes: the pipeline is observed, and its terminal alternate-seed
//! loop is replayed verbatim from the captured entry state.
use super::*;
use std::cell::{Cell, RefCell};
use std::time::Instant;

/// Production retains `PEO_ALT_SEEDS` displaced orderings. These retain the
/// same pools deeper so the shipped prefix can be checked and extended.
const DISPLACED_KEEP: usize = 16;
const UNIVERSE_KEEP: usize = 24;
/// Appended lineage seeds in the bounded arm, matching the shipped depth.
const LINEAGE_EXTRAS: usize = 8;
const ARMS: [&str; 4] = ["control", "depth16", "lineage8", "lineage_all"];

struct Captured {
    n: usize,
    nnz: usize,
    incumbent: Vec<usize>,
    shipped: Vec<(u64, Vec<usize>)>,
    displaced: Vec<(u64, Vec<usize>)>,
    displaced_scores: Vec<u64>,
    universe: Vec<(u64, Vec<usize>)>,
}

thread_local! {
    static ACTIVE: Cell<bool> = Cell::new(false);
    static DISPLACED: RefCell<Vec<(u64, Vec<usize>)>> = RefCell::new(Vec::new());
    static DISPLACED_SCORES: RefCell<Vec<u64>> = RefCell::new(Vec::new());
    static UNIVERSE: RefCell<Vec<(u64, Vec<usize>)>> = RefCell::new(Vec::new());
    static CAPTURED: RefCell<Option<Captured>> = RefCell::new(None);
}

/// Production's retention rule (sort by score, dedup by score, truncate) with
/// a cheap reject first: a duplicate score, or a score no better than a full
/// pool's last entry, cannot survive that rule either.
fn retain(pool: &mut Vec<(u64, Vec<usize>)>, keep: usize, f: u64, perm: &[usize]) {
    if pool.iter().any(|(s, _)| *s == f) {
        return;
    }
    if pool.len() >= keep && f >= pool[keep - 1].0 {
        return;
    }
    pool.push((f, perm.to_vec()));
    pool.sort_by_key(|(s, _)| *s);
    pool.dedup_by_key(|(s, _)| *s);
    pool.truncate(keep);
}

/// Every full-pattern score the pipeline pays, whatever stage produced it.
pub(in crate::ordering) fn note_scored(f: u64, perm: &[usize]) {
    if !ACTIVE.with(|a| a.get()) {
        return;
    }
    UNIVERSE.with(|u| retain(&mut u.borrow_mut(), UNIVERSE_KEEP, f, perm));
}

/// Arm the capture for ONE `order()` invocation and drop the previous pool.
/// `UNIVERSE` keeps the best `UNIVERSE_KEEP` distinct scores, so the pool's
/// first entry is the pipeline's own minimum over every full-pattern score it
/// paid. Callers must not enable this concurrently on other threads: the
/// capture is thread-local and a worker thread's scores would be missed.
pub(in crate::ordering) fn audit_begin() {
    ACTIVE.with(|a| a.set(true));
    DISPLACED.with(|d| d.borrow_mut().clear());
    DISPLACED_SCORES.with(|s| s.borrow_mut().clear());
    UNIVERSE.with(|u| u.borrow_mut().clear());
    CAPTURED.with(|c| *c.borrow_mut() = None);
}

/// Hand back (and clear) the scores captured since the last `audit_begin`.
pub(in crate::ordering) fn audit_take() -> Vec<(u64, Vec<usize>)> {
    UNIVERSE.with(|u| std::mem::take(&mut *u.borrow_mut()))
}

pub(in crate::ordering) fn audit_end() {
    ACTIVE.with(|a| a.set(false));
}

/// The `consider` funnel's displaced ordering, under production's own rule.
pub(in crate::ordering) fn note_consider(
    f: u64,
    perm: &[usize],
    best_flops: u64,
    best_perm: &[usize],
) {
    if !ACTIVE.with(|a| a.get()) {
        return;
    }
    let (score, ordering) = if f < best_flops { (best_flops, best_perm) } else { (f, perm) };
    DISPLACED_SCORES.with(|s| s.borrow_mut().push(score));
    DISPLACED.with(|d| retain(&mut d.borrow_mut(), DISPLACED_KEEP, score, ordering));
}

/// The state stage 8 starts from, taken before its gate is evaluated.
pub(in crate::ordering) fn capture_entry(
    n: usize,
    nnz: usize,
    incumbent: &[usize],
    shipped: &[(u64, Vec<usize>)],
) {
    if !ACTIVE.with(|a| a.get()) {
        return;
    }
    let captured = Captured {
        n,
        nnz,
        incumbent: incumbent.to_vec(),
        shipped: shipped.to_vec(),
        displaced: DISPLACED.with(|d| std::mem::take(&mut *d.borrow_mut())),
        displaced_scores: DISPLACED_SCORES.with(|s| std::mem::take(&mut *s.borrow_mut())),
        universe: UNIVERSE.with(|u| std::mem::take(&mut *u.borrow_mut())),
    };
    CAPTURED.with(|c| *c.borrow_mut() = Some(captured));
}

fn reset() {
    DISPLACED.with(|d| d.borrow_mut().clear());
    DISPLACED_SCORES.with(|s| s.borrow_mut().clear());
    UNIVERSE.with(|u| u.borrow_mut().clear());
    CAPTURED.with(|c| *c.borrow_mut() = None);
}

#[derive(Default)]
struct Spend {
    ledger: u64,
    rounds: usize,
    setups: usize,
}

/// `leader_order`'s alternate-seed loop, byte for byte in its arithmetic: same
/// gate, same `n + nnz + Lnnz` charge paid before each round, same
/// `PEO_ALT_MAX_LNNZ`, same eight-round cap, same strict `<` acceptance and
/// per-seed leader replacement. Stage 8 is terminal at the frontier, so the
/// returned score is the production score of the corresponding arm.
fn replay(sp: &ScoringPattern, cap: &Captured, seeds: &[Vec<usize>], name: &str) -> (u64, Spend) {
    let (n, nnz) = (cap.n, cap.nnz);
    let mut ws = scoring_ws::ScoreWorkspace::new(n, nnz);
    let mut best_perm = cap.incumbent.clone();
    let mut leader_flops = ws.flops(sp, &best_perm);
    let mut spend = Spend::default();
    if !(n >= 16 && (n as u64 + nnz as u64) < PEO_ALT_LEDGER) || seeds.is_empty() {
        return (leader_flops, spend);
    }
    let mut ledger: u64 = 0;
    for seed in seeds {
        let mut cur = seed.clone();
        let mut cur_flops = u64::MAX;
        for _ in 0..8 {
            let pp = permute_pattern(sp, &cur);
            let et = EliminationTree::from_pattern(&pp);
            let counts = column_counts_gnp(&pp, &et);
            spend.setups += 1;
            let lnnz: u64 = counts.iter().map(|&c| c as u64).sum();
            let cost = n as u64 + nnz as u64 + lnnz;
            if ledger + cost > PEO_ALT_LEDGER {
                break;
            }
            ledger += cost;
            spend.rounds += 1;
            let Some(cands) = peo_extract::candidates_bounded(
                n, &pp.col_ptr, &pp.row_idx, &et.parent, &counts, &cur,
                usize::MAX, usize::MAX, PEO_ALT_MAX_LNNZ,
            ) else { break; };
            let inc: u64 = counts.iter().map(|&c| (c as u64) * (c as u64)).sum();
            let mut fin = inc;
            for c in cands {
                assert!(is_bijection(&c, n), "{name}: PEO candidate");
                let f = ws.flops(sp, &c);
                if f < fin {
                    fin = f;
                    cur = c;
                }
            }
            cur_flops = fin;
            if fin == inc {
                break;
            }
        }
        if cur_flops < leader_flops {
            leader_flops = cur_flops;
            best_perm = cur;
        }
        if ledger >= PEO_ALT_LEDGER {
            break;
        }
    }
    assert!(is_bijection(&best_perm, n), "{name}: arm ordering");
    assert_eq!(ws.flops(sp, &best_perm), leader_flops, "{name}: arm score");
    assert!(ledger <= PEO_ALT_LEDGER, "{name}: shipped allowance");
    spend.ledger = ledger;
    (leader_flops, spend)
}

#[test]
#[ignore]
fn probe_alt_seed_lineage() {
    ACTIVE.with(|a| a.set(true));
    let corpus = crate::corpus::corpus();
    assert_eq!(corpus.len(), 300);
    let mut counts = [0usize; 3];
    let mut baseline_logs = [0.0f64; 3];
    let mut arm_logs = [[0.0f64; 3]; 4];
    let mut winners = [0usize; 4];
    let mut movers = [0usize; 4];
    let mut appended_rows = [0usize; 4];
    let mut ledger_units = [0u64; 4];
    let mut rounds = [0usize; 4];
    let mut setups = [0usize; 4];
    let mut arm_ms = [0.0f64; 4];
    let mut worst_ms = [0.0f64; 4];
    let mut worst_row: Vec<String> = vec![String::new(); 4];
    let mut absent = 0usize;
    let mut ungated = 0usize;
    let mut gated = 0usize;
    let mut control_matches = 0usize;
    let mut lineage_available = 0usize;
    for (name, pat) in &corpus {
        reset();
        let incumbent = order(pat);
        let capture = CAPTURED.with(|c| c.borrow_mut().take());
        let sp = scoring_pattern(pat);
        let inc = flops_of(&sp, &incumbent);
        let (cp, ri) = core_of(pat);
        let raw = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
        let amd_perm: Vec<usize> =
            feral_amd::amd_order(&raw).unwrap().into_iter().map(|v| v as usize).collect();
        let amd = flops_of(&sp, &amd_perm);
        let b = bucket(pat.n);
        counts[b] += 1;
        baseline_logs[b] += (inc as f64 / amd as f64).ln();
        let Some(cap) = capture else {
            // The fill-free certificate returned before stage 8 exists.
            for arm in 0..4 {
                arm_logs[arm][b] += (inc as f64 / amd as f64).ln();
            }
            absent += 1;
            control_matches += 1;
            println!(
                "ALTSEEDBASE\t{name}\tn={}\tnnz={}\tbase={amd}\tinc={inc}\tstage8=absent",
                pat.n, pat.nnz()
            );
            continue;
        };
        assert_eq!(cap.n, pat.n, "{name}: captured n");
        assert_eq!(cap.nnz, pat.nnz(), "{name}: captured nnz");
        let k = cap.shipped.len();
        assert!(k <= PEO_ALT_SEEDS, "{name}: shipped pool depth");
        assert_eq!(cap.shipped[..], cap.displaced[..k], "{name}: shipped pool prefix");
        let entry_flops = flops_of(&sp, &cap.incumbent);
        let shipped: Vec<Vec<usize>> = cap.shipped.iter().map(|(_, p)| p.clone()).collect();
        let deeper: Vec<Vec<usize>> =
            cap.displaced.iter().skip(k).map(|(_, p)| p.clone()).collect();
        // A lineage seed is an already-paid ordering the `consider` funnel
        // never offers: not the stage-8 entry incumbent, and never displaced.
        let lineage: Vec<Vec<usize>> = cap
            .universe
            .iter()
            .filter(|(s, _)| *s != entry_flops && !cap.displaced_scores.contains(s))
            .map(|(_, p)| p.clone())
            .collect();
        let mut bounded = shipped.clone();
        bounded.extend(lineage.iter().take(LINEAGE_EXTRAS).cloned());
        let mut wide = shipped.clone();
        wide.extend(lineage.iter().cloned());
        let mut deep = shipped.clone();
        deep.extend(deeper.iter().cloned());
        let pools = [shipped.clone(), deep, bounded, wide];
        let gate = pat.n >= 16
            && (pat.n as u64 + pat.nnz() as u64) < PEO_ALT_LEDGER
            && !shipped.is_empty();
        if gate {
            gated += 1;
        } else {
            ungated += 1;
        }
        if !lineage.is_empty() {
            lineage_available += 1;
        }
        let mut arm_flops = [0u64; 4];
        for arm in 0..4 {
            let timer = Instant::now();
            let (f, spend) = replay(&sp, &cap, &pools[arm], name);
            let ms = timer.elapsed().as_secs_f64() * 1000.0;
            arm_flops[arm] = f;
            arm_logs[arm][b] += (f as f64 / amd as f64).ln();
            if f < inc {
                winners[arm] += 1;
            }
            ledger_units[arm] += spend.ledger;
            rounds[arm] += spend.rounds;
            setups[arm] += spend.setups;
            arm_ms[arm] += ms;
            if ms > worst_ms[arm] {
                worst_ms[arm] = ms;
                worst_row[arm] = name.clone();
            }
            if pools[arm].len() > k {
                appended_rows[arm] += 1;
            }
            println!(
                "ALTSEEDARM\t{name}\tarm={}\tseeds={}\tflops={f}\tledger={}\trounds={}\tsetups={}\tms={ms:.4}",
                ARMS[arm], pools[arm].len(), spend.ledger, spend.rounds, spend.setups
            );
        }
        assert_eq!(arm_flops[0], inc, "{name}: control replay reproduces production");
        control_matches += 1;
        for arm in 1..4 {
            if arm_flops[arm] < arm_flops[0] {
                movers[arm] += 1;
            }
            assert!(arm_flops[arm] <= arm_flops[0], "{name}: appended seeds are monotone");
        }
        println!(
            "ALTSEEDBASE\t{name}\tn={}\tnnz={}\tbase={amd}\tinc={inc}\tentry={entry_flops}\tgate={gate}\tshipped={k}\tdeeper={}\tlineage={}\tuniverse={}",
            pat.n, pat.nnz(), deeper.len(), lineage.len(), cap.universe.len()
        );
    }
    ACTIVE.with(|a| a.set(false));
    let baseline = aggregate(&baseline_logs, &counts);
    println!(
        "ALTSEEDSUMMARY\tbaseline={baseline:.16}\tcontrol_matches={control_matches}\tgated={gated}\tungated={ungated}\tabsent={absent}\tlineage_rows={lineage_available}"
    );
    for arm in 0..4 {
        let score = aggregate(&arm_logs[arm], &counts);
        println!(
            "ALTSEEDSCORE\tarm={}\tscore={score:.16}\tbips={:.8}\twinners={}\tmovers={}\tappended_rows={}\tledger={}\trounds={}\tsetups={}\ttotal_ms={:.3}\tworst_ms={:.3}\tworst_row={}",
            ARMS[arm], (baseline - score) * 10000.0, winners[arm], movers[arm],
            appended_rows[arm], ledger_units[arm], rounds[arm], setups[arm],
            arm_ms[arm], worst_ms[arm], worst_row[arm]
        );
    }
    assert_eq!(control_matches, 300, "control replay must price every row");
}
