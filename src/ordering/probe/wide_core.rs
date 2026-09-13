//! Test-only exact residual-core policy screen above the closed `cn <= 300`
//! band. Consumes production `order_core` captures; no production state changes.
use super::*;

/// `combined` is the best of the first three; `minfill_degree` is production's
/// tie law and is reported outside the combined treatment.
const LABELS: [&str; 4] = ["minfill_index", "md_index", "mcs_index", "minfill_degree"];
const COMBINED: usize = 3;
/// Production's own core-portfolio envelope (`mod.rs:2685`, `refine_core`),
/// unioned with the dense small cores the closed screen excluded on `core_nnz`.
const BAND_MAX_CN: usize = 4_000;
const BAND_MAX_CORE_NNZ: usize = 30_000;
const DENSE_MAX_CN: usize = 1_200;
const DENSE_MAX_CORE_NNZ: usize = 200_000;
/// Runaway guard: a policy that charges past this many deficiency words is
/// abandoned and contributes no candidate. Reported per capture.
const DEFICIENCY_WORD_CAP: u64 = 8_000_000_000;

/// Screen configuration. `minfill_words` mirrors production's hard MinFill
/// budget (`minfill_order`'s 40M degree-pair allowance) in deficiency words:
/// when it runs out the greedy stops and the remaining live vertices are
/// appended in ascending current degree, ties by index — production's own
/// fallback rule (`mod.rs:3113-3121`). `u64::MAX` disables it.
struct Config {
    minfill_words: u64,
    row_ledger: u64,
    max_cn: usize,
    max_cnnz: usize,
    dense_max_cn: usize,
}

fn config() -> Config {
    let get = |key: &str, default: u64| -> u64 {
        std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
    };
    Config {
        minfill_words: get("WIDE_CORE_MINFILL_WORDS", u64::MAX),
        // One shared allowance per ORIGINAL row, spent across its captures in
        // capture order, per policy. `u64::MAX` leaves only the per-capture cap.
        row_ledger: get("WIDE_CORE_ROW_LEDGER", u64::MAX),
        max_cn: get("WIDE_CORE_MAX_CN", BAND_MAX_CN as u64) as usize,
        max_cnnz: get("WIDE_CORE_MAX_CNNZ", BAND_MAX_CORE_NNZ as u64) as usize,
        dense_max_cn: get("WIDE_CORE_DENSE_MAX_CN", DENSE_MAX_CN as u64) as usize,
    }
}
/// The closed screen's admission set: value inside it is reported separately.
const CLOSED_MAX_CN: usize = 300;
const CLOSED_MAX_CORE_NNZ: usize = 3_000;

#[derive(Default)]
struct Work {
    scans: u64,
    popcount_words: u64,
    deficiency_words: u64,
    clique_words: u64,
}

/// A live symmetric graph in a dynamic bitset layout (`SmallScore` is fixed at
/// five words, i.e. `n <= 320`, so it cannot serve this band).
struct Graph {
    n: usize,
    w: usize,
    rows: Vec<u64>,
}

impl Graph {
    fn new(pat: &Pattern) -> Graph {
        let n = pat.n;
        let w = n.div_ceil(64);
        let mut rows = vec![0u64; n * w];
        for j in 0..n {
            for &i in &pat.row_idx[pat.col_ptr[j]..pat.col_ptr[j + 1]] {
                if i != j && i < n {
                    rows[j * w + i / 64] |= 1u64 << (i % 64);
                    rows[i * w + j / 64] |= 1u64 << (j % 64);
                }
            }
        }
        Graph { n, w, rows }
    }
}

#[inline]
fn row(rows: &[u64], w: usize, v: usize) -> &[u64] {
    &rows[v * w..v * w + w]
}

#[inline]
fn popcount(slice: &[u64]) -> usize {
    slice.iter().map(|x| x.count_ones() as usize).sum()
}

fn bits_of(slice: &[u64], out: &mut Vec<usize>) {
    out.clear();
    for (j, &word) in slice.iter().enumerate() {
        let mut bits = word;
        while bits != 0 {
            out.push(64 * j + bits.trailing_zeros() as usize);
            bits &= bits - 1;
        }
    }
}

/// `C(d,2) - |E(N(v))|` on the current live graph, charged in words.
fn deficiency(rows: &[u64], w: usize, v: usize, deg: usize, nb: &mut Vec<usize>, work: &mut Work) -> u64 {
    bits_of(row(rows, w, v), nb);
    let mut twice_edges = 0u64;
    for &u in nb.iter() {
        let (a, b) = (u * w, v * w);
        for k in 0..w {
            twice_edges += (rows[a + k] & rows[b + k]).count_ones() as u64;
        }
        work.deficiency_words += w as u64;
    }
    (deg as u64) * (deg as u64).saturating_sub(1) / 2 - twice_edges / 2
}

/// Greedy elimination under one exact policy. Returns the ordering, its
/// elimination-game flop count and the charged work. `minfill` selects exact
/// minimum fill, `degree_ties` adds production's degree tie-break; ties are
/// otherwise broken by ascending index (the ascending scan with strict `<`
/// keeps the lowest index).
fn greedy(g: &Graph, minfill: bool, degree_ties: bool, budget: u64) -> (Option<Vec<usize>>, u64, Work, bool) {
    let (n, w) = (g.n, g.w);
    let mut rows = g.rows.clone();
    let mut live = vec![true; n];
    let mut deg = vec![0usize; n];
    let mut defic = vec![0u64; n];
    let mut work = Work::default();
    let mut nb: Vec<usize> = Vec::with_capacity(n);
    let mut scratch: Vec<usize> = Vec::with_capacity(n);
    for v in 0..n {
        deg[v] = popcount(row(&rows, w, v));
        work.popcount_words += w as u64;
    }
    if minfill {
        for v in 0..n {
            defic[v] = deficiency(&rows, w, v, deg[v], &mut nb, &mut work);
        }
    }
    if work.deficiency_words > DEFICIENCY_WORD_CAP {
        return (None, 0, work, false);
    }
    let mut out = Vec::with_capacity(n);
    let mut flops = 0u64;
    let mut dirty = vec![0u64; w];
    let mut pivot_row = vec![0u64; w];
    let mut truncated = false;
    for _ in 0..n {
        if minfill && work.deficiency_words > budget {
            // Production's fallback: append the remaining live vertices in
            // ascending current degree, ties by index. Still a bijection.
            truncated = true;
            let mut rest: Vec<usize> = (0..n).filter(|&v| live[v]).collect();
            rest.sort_by_key(|&v| (deg[v], v));
            for v in rest {
                let d = deg[v] as u64;
                flops += (d + 1) * (d + 1);
                out.push(v);
                live[v] = false;
                bits_of(row(&rows, w, v), &mut nb);
                pivot_row.copy_from_slice(row(&rows, w, v));
                for &u in nb.iter() {
                    for k in 0..w {
                        rows[u * w + k] |= pivot_row[k];
                    }
                    rows[u * w + u / 64] &= !(1u64 << (u % 64));
                    rows[u * w + v / 64] &= !(1u64 << (v % 64));
                }
                for &u in nb.iter() {
                    deg[u] = popcount(row(&rows, w, u));
                }
            }
            break;
        }
        let mut best = usize::MAX;
        let mut best_key = (u64::MAX, usize::MAX);
        for v in 0..n {
            work.scans += 1;
            if !live[v] {
                continue;
            }
            let metric = if minfill { defic[v] } else { deg[v] as u64 };
            let key = (metric, if degree_ties { deg[v] } else { 0 });
            if key < best_key {
                best_key = key;
                best = v;
            }
        }
        assert_ne!(best, usize::MAX);
        let d = deg[best] as u64;
        flops += (d + 1) * (d + 1);
        out.push(best);
        live[best] = false;
        bits_of(row(&rows, w, best), &mut nb);
        // Clique the neighbourhood, then unlink the pivot from it.
        pivot_row.copy_from_slice(row(&rows, w, best));
        for &u in nb.iter() {
            for k in 0..w {
                rows[u * w + k] |= pivot_row[k];
            }
            rows[u * w + u / 64] &= !(1u64 << (u % 64));
            rows[u * w + best / 64] &= !(1u64 << (best % 64));
            work.clique_words += w as u64;
        }
        for &u in nb.iter() {
            deg[u] = popcount(row(&rows, w, u));
            work.popcount_words += w as u64;
        }
        if minfill {
            // Eliminating `best` changes no other vertex's neighbourhood and
            // adds edges only inside N(best), so only N(best) and its
            // neighbours can change deficiency.
            for word in dirty.iter_mut() {
                *word = 0;
            }
            for &u in nb.iter() {
                dirty[u / 64] |= 1u64 << (u % 64);
                for k in 0..w {
                    dirty[k] |= rows[u * w + k];
                }
            }
            bits_of(&dirty, &mut scratch);
            let touched = std::mem::take(&mut scratch);
            for &x in touched.iter() {
                if live[x] {
                    defic[x] = deficiency(&rows, w, x, deg[x], &mut nb, &mut work);
                }
            }
            scratch = touched;
            if work.deficiency_words > DEFICIENCY_WORD_CAP {
                return (None, 0, work, false);
            }
        }
    }
    (Some(out), flops, work, truncated)
}

/// Reverse maximum-cardinality search on the original core graph.
fn mcs(g: &Graph) -> (Option<Vec<usize>>, Work, bool) {
    let (n, w) = (g.n, g.w);
    let mut weights = vec![0usize; n];
    let mut live = vec![true; n];
    let mut out = Vec::with_capacity(n);
    let mut work = Work::default();
    let mut nb: Vec<usize> = Vec::with_capacity(n);
    for _ in 0..n {
        let mut best = usize::MAX;
        for v in 0..n {
            work.scans += 1;
            if live[v] && (best == usize::MAX || weights[v] > weights[best]) {
                best = v;
            }
        }
        out.push(best);
        live[best] = false;
        bits_of(row(&g.rows, w, best), &mut nb);
        work.popcount_words += w as u64;
        for &u in nb.iter() {
            if live[u] {
                weights[u] += 1;
            }
            work.clique_words += 1;
        }
    }
    out.reverse();
    (Some(out), work, false)
}

#[test]
#[ignore]
fn probe_wide_core_band() {
    CORE_CAPTURE_ENABLED.with(|enabled| enabled.set(true));
    let corpus = crate::corpus::corpus();
    assert_eq!(corpus.len(), 300);
    let mut counts = [0usize; 3];
    let mut baseline_logs = [0.0f64; 3];
    // 0..4 single policies, 4 = combined over the band, 5 = combined restricted
    // to the closed screen's admission set, 6 = combined outside it.
    let mut policy_logs = [[0.0f64; 3]; 7];
    let mut winners = [0usize; 7];
    let mut eligible_rows = 0usize;
    let mut eligible_captures = 0usize;
    let mut new_captures = 0usize;
    // Reproduction knob for a subset smoke run. When set the aggregation covers
    // only the listed rows, so the analyzer rejects the run as non-authoritative.
    let cfg = config();
    println!(
        "BANDCONFIG\tminfill_words={}\trow_ledger={}\tmax_cn={}\tmax_cnnz={}\tdense_max_cn={}",
        cfg.minfill_words, cfg.row_ledger, cfg.max_cn, cfg.max_cnnz, cfg.dense_max_cn
    );
    let only = std::env::var("WIDE_CORE_ROWS").unwrap_or_default();
    if !only.is_empty() {
        println!("BANDFILTER\trows={only}");
    }
    for (name, pat) in &corpus {
        if !only.is_empty() && !only.split(',').any(|r| r == name.as_str()) {
            continue;
        }
        let _ = take_core_candidates();
        let incumbent = order(pat);
        let captures = take_core_candidates();
        let sp = scoring_pattern(pat);
        let inc = flops_of(&sp, &incumbent);
        let (cp, ri) = core_of(pat);
        let raw = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
        let amd_perm: Vec<usize> = feral_amd::amd_order(&raw)
            .unwrap()
            .into_iter()
            .map(|v| v as usize)
            .collect();
        let amd = flops_of(&sp, &amd_perm);
        let b = bucket(pat.n);
        counts[b] += 1;
        baseline_logs[b] += (inc as f64 / amd as f64).ln();
        let mut row_best = [inc; 7];
        let mut row_ledger = [cfg.row_ledger; LABELS.len()];
        let mut paid = 0usize;
        let mut row_ns = 0u128;
        for (capture, c) in captures.iter().enumerate() {
            let cnnz = c.row_idx.len();
            let in_band = (12..=cfg.max_cn).contains(&c.cn)
                && (cnnz <= cfg.max_cnnz
                    || (c.cn <= cfg.dense_max_cn && cnnz <= DENSE_MAX_CORE_NNZ));
            let in_closed = (12..=CLOSED_MAX_CN).contains(&c.cn) && cnnz <= CLOSED_MAX_CORE_NNZ;
            println!(
                "BANDCAPTURE\t{name}\tcapture={capture}\tcn={}\tcnnz={cnnz}\trecurse={}\tin_band={}\tin_closed={}",
                c.cn,
                c.recurse as usize,
                in_band as usize,
                in_closed as usize
            );
            if !in_band {
                continue;
            }
            let start = Instant::now();
            paid += 1;
            eligible_captures += 1;
            if !in_closed {
                new_captures += 1;
            }
            let core_pat = Pattern {
                n: c.cn,
                col_ptr: c.col_ptr.clone(),
                row_idx: c.row_idx.clone(),
            };
            let core_sp = scoring_pattern(&core_pat);
            let g = Graph::new(&core_pat);
            let core_inc = flops_of(&core_sp, &c.core_perm);
            println!(
                "BANDCORE\t{name}\tcapture={capture}\tcn={}\tcnnz={cnnz}\trecurse={}\tprefix={}\tcore_inc={core_inc}",
                c.cn, c.recurse as usize, c.prefix_flops
            );
            for policy in 0..LABELS.len() {
                let timer = Instant::now();
                let allowance = cfg.minfill_words.min(row_ledger[policy]);
                let (p, direct, work, truncated) = match policy {
                    0 => greedy(&g, true, false, allowance),
                    1 => greedy(&g, false, false, u64::MAX),
                    2 => {
                        let (p, work, t) = mcs(&g);
                        (p, u64::MAX, work, t)
                    }
                    3 => greedy(&g, true, true, allowance),
                    _ => unreachable!(),
                };
                row_ledger[policy] = row_ledger[policy].saturating_sub(work.deficiency_words);
                let search_ns = timer.elapsed().as_nanos();
                let Some(p) = p else {
                    println!(
                        "BANDABORT\t{name}\tcapture={capture}\tpolicy={}\tdeficiency_words={}",
                        LABELS[policy], work.deficiency_words
                    );
                    continue;
                };
                assert!(is_bijection(&p, c.cn), "{name}: {}", LABELS[policy]);
                let f = flops_of(&core_sp, &p);
                if policy != 2 {
                    assert_eq!(f, direct, "{name}: {} direct cost", LABELS[policy]);
                }
                if policy == 3 && c.cn <= 1_000 {
                    let shipped: Vec<usize> = minfill_order(&core_pat)
                        .into_iter()
                        .map(|v| v as usize)
                        .collect();
                    println!(
                        "BANDCONTROL\t{name}\tcapture={capture}\tcn={}\tflops={}\tbitset_flops={f}\tsame_perm={}",
                        c.cn,
                        flops_of(&core_sp, &shipped),
                        (shipped == p) as usize
                    );
                }
                let total = c.prefix_flops + f;
                row_best[policy] = row_best[policy].min(total);
                if policy < COMBINED {
                    row_best[4] = row_best[4].min(total);
                    if in_closed {
                        row_best[5] = row_best[5].min(total);
                    } else {
                        row_best[6] = row_best[6].min(total);
                    }
                }
                println!(
                    "BANDCAND\t{name}\tcapture={capture}\tpolicy={}\tcore={f}\ttotal={total}\tsearch_ns={search_ns}\tscans={}\tpopcount_words={}\tdeficiency_words={}\tclique_words={}\ttruncated={}",
                    LABELS[policy], work.scans, work.popcount_words, work.deficiency_words, work.clique_words, truncated as usize
                );
            }
            row_ns += start.elapsed().as_nanos();
        }
        if paid > 0 {
            eligible_rows += 1;
        }
        for policy in 0..7 {
            policy_logs[policy][b] += (row_best[policy] as f64 / amd as f64).ln();
            if row_best[policy] < inc {
                winners[policy] += 1;
            }
        }
        println!(
            "BANDBASE\t{name}\tn={}\tnnz={}\tbase={amd}\tinc={inc}\tpaid={paid}\tall_captures={}\trow_ns={row_ns}\tbest={}\tbest_new={}",
            pat.n,
            pat.nnz(),
            captures.len(),
            row_best[4],
            row_best[6]
        );
    }
    CORE_CAPTURE_ENABLED.with(|enabled| enabled.set(false));
    let baseline = aggregate(&baseline_logs, &counts);
    println!(
        "BANDSUMMARY\tcontrol={baseline:.16}\teligible_rows={eligible_rows}\teligible_captures={eligible_captures}\tnew_captures={new_captures}"
    );
    for policy in 0..7 {
        let label = match policy {
            4 => "combined_band",
            5 => "combined_closed",
            6 => "combined_new",
            _ => LABELS[policy],
        };
        let score = aggregate(&policy_logs[policy], &counts);
        println!(
            "BANDSCORE\tpolicy={label}\tscore={score:.16}\tbips={:.8}\twinners={}",
            (baseline - score) * 10000.0,
            winners[policy]
        );
    }
}
