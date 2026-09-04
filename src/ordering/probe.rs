//! TEST-ONLY measurement harness. Never compiled into the shipped binary
//! (`#[cfg(test)] mod probe;`), never read by the grader — it exists so a
//! session can measure, rather than guess, two things the scored harness
//! deliberately hides:
//!
//!   1. **Timing headroom.** The harness prints `(capped)` instead of a time,
//!      so the only way to know how close `order()` runs to the enforced 2 s
//!      SIGKILL is to time it here.
//!   2. **Candidate value.** Before wiring a new candidate into `order()`, run
//!      it here across the corpus and compute what the score *would* become.
//!
//! Run with:
//! ```sh
//! cargo test --release -- --ignored --nocapture probe_
//! ```

use super::*;
use std::time::Instant;

/// Buckets exactly as the harness does (lt_1k / 1k_10k / gt_10k).
fn bucket(n: usize) -> usize {
    if n < 1_000 {
        0
    } else if n < 10_000 {
        1
    } else {
        2
    }
}

const BUCKET_NAMES: [&str; 3] = ["lt_1k", "1k_10k", "gt_10k"];
const BUCKET_WEIGHTS: [f64; 3] = [0.30, 0.30, 0.40];

/// Weighted mean of per-bucket geomeans, with empty buckets renormalized out —
/// the harness's exact aggregation.
fn aggregate(log_sums: &[f64; 3], counts: &[usize; 3]) -> f64 {
    let mut num = 0.0;
    let mut den = 0.0;
    for b in 0..3 {
        if counts[b] == 0 {
            continue;
        }
        num += BUCKET_WEIGHTS[b] * (log_sums[b] / counts[b] as f64).exp();
        den += BUCKET_WEIGHTS[b];
    }
    if den == 0.0 {
        f64::NAN
    } else {
        num / den
    }
}

fn scoring_pattern(pattern: &Pattern) -> ScoringPattern {
    ScoringPattern {
        n: pattern.n,
        col_ptr: pattern.col_ptr.clone(),
        row_idx: pattern.row_idx.clone(),
    }
}

fn core_of(pattern: &Pattern) -> (Vec<i32>, Vec<i32>) {
    (
        pattern.col_ptr.iter().map(|&x| x as i32).collect(),
        pattern.row_idx.iter().map(|&x| x as i32).collect(),
    )
}

/// Time `order()` on every corpus matrix and report the slowest instances plus
/// the current per-bucket score. This is the safety probe: the number that
/// matters is `worst`, which must stay far under the 2 s cap (the grader's
/// machine is slower than local).
#[test]
#[ignore]
fn probe_timing_and_score() {
    let corpus = match std::env::var("SSI_CORPUS_FILE") {
        Ok(path) if !path.trim().is_empty() => {
            ssi_scoring::load_corpus_jsonl(std::path::Path::new(&path))
                .unwrap_or_else(|_| crate::corpus::corpus())
        }
        _ => crate::corpus::corpus(),
    };
    let mut rows: Vec<(f64, String, usize, usize, f64)> = Vec::new();
    let mut log_sums = [0.0f64; 3];
    let mut counts = [0usize; 3];

    for (name, pat) in &corpus {
        let n = pat.n;
        if n == 0 {
            continue;
        }
        let sp = scoring_pattern(pat);
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

        let t0 = Instant::now();
        let perm = order(pat);
        let secs = t0.elapsed().as_secs_f64();
        let mine = flops_of(&sp, &perm);
        let ratio = mine as f64 / base as f64;

        let b = bucket(n);
        log_sums[b] += ratio.ln();
        counts[b] += 1;
        rows.push((secs, name.clone(), n, pat.nnz(), ratio));
    }

    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    println!("\n--- every order() call, slowest first (TSV) ---");
    println!("secs\tmatrix\tn\tnnz\tratio");
    for (secs, name, n, nnz, ratio) in rows.iter() {
        println!("{secs:.4}\t{name}\t{n}\t{nnz}\t{ratio:.4}");
    }

    // How much of the corpus lives in each cost tier — this is what decides
    // where extra candidates can be afforded.
    for thresh in [0.010f64, 0.050, 0.100, 0.200, 0.400] {
        let k = rows.iter().filter(|r| r.0 < thresh).count();
        println!("matrices under {thresh:.3}s: {k}/{}", rows.len());
    }

    println!("\n--- per-bucket ---");
    for b in 0..3 {
        if counts[b] > 0 {
            println!(
                "{:<8} count={:<5} geomean={:.4}",
                BUCKET_NAMES[b],
                counts[b],
                (log_sums[b] / counts[b] as f64).exp()
            );
        }
    }
    println!("SCORE = {:.6}", aggregate(&log_sums, &counts));
    println!("WORST order() = {:.3} s", rows[0].0);
}

/// List the matrices where the current `order()` is still tied at (or above)
/// the AMD baseline. Every tie is pure upside for a new candidate, so this is
/// the target list for the next experiment.
#[test]
#[ignore]
fn probe_ties() {
    let corpus = crate::corpus::corpus();
    let mut ties: Vec<(String, usize, usize, f64)> = Vec::new();
    let mut per_bucket = [0usize; 3];
    let mut total = [0usize; 3];

    for (name, pat) in &corpus {
        let n = pat.n;
        if n == 0 {
            continue;
        }
        let sp = scoring_pattern(pat);
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
        let mine = flops_of(&sp, &order(pat));
        let ratio = mine as f64 / base as f64;
        let b = bucket(n);
        total[b] += 1;
        if ratio > 0.9999 {
            per_bucket[b] += 1;
            ties.push((name.clone(), n, pat.nnz(), ratio));
        }
    }

    println!("\n--- matrices tied at AMD (ratio >= 0.9999) ---");
    println!("{:<28} {:>8} {:>10} {:>8}", "matrix", "n", "nnz", "ratio");
    for (name, n, nnz, ratio) in &ties {
        println!("{name:<28} {n:>8} {nnz:>10} {ratio:>8.4}");
    }
    for b in 0..3 {
        println!(
            "{:<8} tied {}/{}",
            BUCKET_NAMES[b], per_bucket[b], total[b]
        );
    }
}

#[test]
#[ignore]
fn probe_subtree_rounds() {
    let corpus = crate::corpus::corpus();
    let mut base_log_sum = [0.0f64; 3];
    let mut r1_log_sum = [0.0f64; 3];
    let mut r2_log_sum = [[0.0f64; 3]; 3];
    let mut counts = [0usize; 3];

    for (_name, pat) in &corpus {
        let n = pat.n;
        let nnz = pat.nnz();
        let b = bucket(n);
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let amd_flops = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        );

        let inc = order(pat);
        let inc_flops = flops_of(&sp, &inc);
        counts[b] += 1;
        base_log_sum[b] += (inc_flops as f64 / amd_flops as f64).ln();

        if !(1_000..=350_000).contains(&n) || nnz > 1_500_000 {
            r1_log_sum[b] += (inc_flops as f64 / amd_flops as f64).ln();
            for i in 0..3 {
                r2_log_sum[i][b] += (inc_flops as f64 / amd_flops as f64).ln();
            }
            continue;
        }

        // inc is already the result of round 1 from order().
        // So inc_flops is already r1_flops!
        let r1_flops = inc_flops;
        r1_log_sum[b] += (r1_flops as f64 / amd_flops as f64).ln();

        // Now test round 2 on top of inc!
        let permuted = permute_pattern(&sp, &inc);
        let etree = EliminationTree::from_pattern(&permuted);
        let post = etree.postorder();
        let candidate: Vec<usize> = post.iter().map(|&j| inc[j]).collect();
        let post_pattern = permute_pattern(&sp, &candidate);
        let post_etree = EliminationTree::from_pattern(&post_pattern);
        let r_counts: Vec<u32> = column_counts_gnp(&post_pattern, &post_etree)
            .into_iter()
            .map(|c| c as u32)
            .collect();
        let parent: Vec<i32> = post_etree
            .parent
            .iter()
            .map(|p| p.map_or(-1, |j| j as i32))
            .collect();

        // Test Round 2 with max_blocks = 16, 24, 32
        for (i, &mb) in [16, 24, 32].iter().enumerate() {
            let mut cand = candidate.clone();
            let mut cfg2 = SUBTREE_CFG;
            cfg2.round = 1;
            cfg2.max_blocks = mb;
            let improved2 = rgreedy::subtree_refine(
                n,
                &pat.col_ptr,
                &pat.row_idx,
                &mut cand,
                &r_counts,
                &parent,
                cfg2,
            );
            let mut f_out = r1_flops;
            if improved2 > 0 && is_bijection(&cand, n) {
                let f = flops_of(&sp, &cand);
                if f < f_out {
                    f_out = f;
                }
            }
            r2_log_sum[i][b] += (f_out as f64 / amd_flops as f64).ln();
        }
    }

    println!("\n--- SCORES ---");
    println!("Base (Round 1): {:.6}", aggregate(&r1_log_sum, &counts));
    for (i, &mb) in [16, 24, 32].iter().enumerate() {
        let score = aggregate(&r2_log_sum[i], &counts);
        println!("Round 2 max_blocks={mb:<2}: {score:.6} (diff: {:+.6})", score - aggregate(&r1_log_sum, &counts));
    }
}



#[test]
#[ignore]
fn probe_medium_variations() {
    let corpus = crate::corpus::corpus();
    let qualifying: Vec<_> = corpus
        .iter()
        .filter(|(_, pat)| pat.n > 1_000 && pat.n <= 6_000 && pat.nnz() <= 30_000)
        .collect();

    println!("Found {} qualifying medium matrices", qualifying.len());

    let mut scores = [0.0f64; 5];
    let n_variants = 5;

    for (name, pat) in &qualifying {
        let n = pat.n;
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let amd_flops = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        );

        // Run full order() pipeline up to medium search
        // We can get the base perm by running order() with medium search temporarily bypassed
        // or by testing the exact search stages from the current order() incumbent.
        let incumbent = order(pat);
        let inc_flops = flops_of(&sp, &incumbent);

        // Test the variants starting from incumbent
        for v in 0..n_variants {
            let mut perm = incumbent.clone();
            let mut flops = inc_flops;

            let stages: Vec<(i64, u64, rgreedy::Params)> = match v {
                0 => vec![
                    (100_000_000, 0xD1B5_4A32_D192_ED03, rgreedy::Params::DEFAULT),
                    (50_000_000, 0xD1B5_4A32_D192_ED03, rgreedy::Params::DEFAULT),
                ],
                1 => vec![
                    (100_000_000, 0xD1B5_4A32_D192_ED03, rgreedy::Params::DEFAULT),
                    (50_000_000, rgreedy::stream_rng(1), rgreedy::Params::DEFAULT),
                ],
                2 => vec![
                    (100_000_000, 0xD1B5_4A32_D192_ED03, rgreedy::Params::DEFAULT),
                    (50_000_000, rgreedy::stream_rng(1), rgreedy::stream_params(1)),
                ],
                3 => vec![
                    (75_000_000, 0xD1B5_4A32_D192_ED03, rgreedy::Params::DEFAULT),
                    (75_000_000, rgreedy::stream_rng(1), rgreedy::stream_params(1)),
                ],
                4 => vec![
                    (100_000_000, 0xD1B5_4A32_D192_ED03, rgreedy::Params::DEFAULT),
                    (50_000_000, rgreedy::stream_rng(2), rgreedy::stream_params(2)),
                ],
                _ => unreachable!(),
            };

            for (budget, seed, params) in stages {
                let adj0 = rgreedy::Game::build_adj(n, &pat.col_ptr, &pat.row_idx).unwrap();
                if let Some((cand, _)) = rgreedy::search_with(
                    n,
                    &adj0,
                    &perm,
                    flops,
                    budget,
                    seed,
                    params,
                ) {
                    if is_bijection(&cand, n) {
                        let f = flops_of(&sp, &cand);
                        if f < flops {
                            flops = f;
                            perm = cand;
                        }
                    }
                }
            }

            let ratio = flops as f64 / amd_flops as f64;
            scores[v] += ratio.ln();
            if v > 0 && flops < inc_flops {
                println!("  v{v} improved {name}: {inc_flops} -> {flops} ({:.4} -> {:.4})", inc_flops as f64 / amd_flops as f64, ratio);
            }
        }
    }

    println!("\n--- Variant geomean ratio on qualifying matrices ---");
    let base_geomean = (scores[0] / qualifying.len() as f64).exp();
    println!("v0 (current): {base_geomean:.6}");
    for v in 1..n_variants {
        let g = (scores[v] / qualifying.len() as f64).exp();
        println!("v{v}: {g:.6} (diff vs v0: {:+.6})", g - base_geomean);
    }
}

#[test]
#[ignore]
fn probe_small_variations() {
    let corpus = crate::corpus::corpus();
    let qualifying: Vec<_> = corpus
        .iter()
        .filter(|(_, pat)| pat.n <= 1_000 && pat.nnz() <= 30_000 && pat.n > 0)
        .collect();

    println!("Found {} qualifying small matrices", qualifying.len());

    let mut scores = [0.0f64; 4];
    let n_variants = 4;

    for (name, pat) in &qualifying {
        let n = pat.n;
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let amd_flops = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        );

        let incumbent = order(pat);
        let inc_flops = flops_of(&sp, &incumbent);

        for v in 0..n_variants {
            let mut perm = incumbent.clone();
            let mut flops = inc_flops;

            let stages: Vec<(i64, u64, rgreedy::Params)> = match v {
                0 => vec![
                    (100_000_000, 0x9E37_79B9_7F4A_7C15, rgreedy::Params::DEFAULT),
                ],
                1 => vec![
                    (50_000_000, 0x9E37_79B9_7F4A_7C15, rgreedy::Params::DEFAULT),
                    (50_000_000, rgreedy::stream_rng(1), rgreedy::Params::DEFAULT),
                ],
                2 => vec![
                    (50_000_000, 0x9E37_79B9_7F4A_7C15, rgreedy::Params::DEFAULT),
                    (50_000_000, rgreedy::stream_rng(1), rgreedy::stream_params(1)),
                ],
                3 => vec![
                    (50_000_000, 0x9E37_79B9_7F4A_7C15, rgreedy::Params::DEFAULT),
                    (50_000_000, rgreedy::stream_rng(2), rgreedy::stream_params(2)),
                ],
                _ => unreachable!(),
            };

            for (budget, seed, params) in stages {
                let adj0 = rgreedy::Game::build_adj(n, &pat.col_ptr, &pat.row_idx).unwrap();
                if let Some((cand, _)) = rgreedy::search_with(
                    n,
                    &adj0,
                    &perm,
                    flops,
                    budget,
                    seed,
                    params,
                ) {
                    if is_bijection(&cand, n) {
                        let f = flops_of(&sp, &cand);
                        if f < flops {
                            flops = f;
                            perm = cand;
                        }
                    }
                }
            }

            let ratio = flops as f64 / amd_flops as f64;
            scores[v] += ratio.ln();
            if v > 0 && flops < inc_flops {
                println!("  v{v} improved {name}: {inc_flops} -> {flops} ({:.4} -> {:.4})", inc_flops as f64 / amd_flops as f64, ratio);
            }
        }
    }

    println!("\n--- Variant geomean ratio on qualifying small matrices ---");
    let base_geomean = (scores[0] / qualifying.len() as f64).exp();
    println!("v0 (current): {base_geomean:.6}");
    for v in 1..n_variants {
        let g = (scores[v] / qualifying.len() as f64).exp();
        println!("v{v}: {g:.6} (diff vs v0: {:+.6})", g - base_geomean);
    }
}





// `splitmix64`, `relabel` and `relabel_restarts` now live in the shipped module
// (`super`) and reach this file through `use super::*` — the probe must exercise
// the exact same functions `order()` uses, or its predictions stop being valid.

/// RANDOMIZED-RESTART minimum degree, for free, using the library AMD.
///
/// AMD's result depends on its tie-breaking, and its tie-breaking depends on the
/// vertex NUMBERING. So running feral's own AMD on a relabelled copy of the
/// pattern (`B = P A Pᵀ`) and composing the result back through `P` yields a
/// genuinely different minimum-degree ordering — a multi-start MD without
/// writing an MD implementation. That matters because 122 of the 300 corpus
/// matrices are still tied at exactly 1.000, i.e. AMD beats every separator- and
/// profile-based candidate on them; a different *AMD* is the one thing not yet
/// tried on that set.
///
/// This probe reports, per restart count, the score it would reach and what it
/// costs.
#[test]
#[ignore]
fn probe_relabel_amd() {
    const MAX_N: usize = 40_000;
    const MAX_NNZ: usize = 200_000;
    const RESTARTS: usize = 24;

    let corpus = crate::corpus::corpus();
    // Score after 0 (=current), 4, 8, 16 and 24 restarts.
    let stops = [4usize, 8, 16, 24];
    let mut cur = ([0.0f64; 3], [0usize; 3]);
    let mut at: Vec<([f64; 3], [usize; 3])> = vec![([0.0; 3], [0; 3]); stops.len()];
    let mut rows: Vec<(f64, String, usize, usize, f64, f64)> = Vec::new();

    for (name, pat) in &corpus {
        let n = pat.n;
        if n == 0 {
            continue;
        }
        let nnz = pat.nnz();
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let base = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        ) as f64;
        let cur_flops = flops_of(&sp, &order(pat)) as f64;

        let mut best = cur_flops;
        let mut marks = vec![cur_flops; stops.len()];
        let t0 = Instant::now();
        if n < MAX_N && nnz < MAX_NNZ {
            for r in 0..RESTARTS {
                let q = relabel(n, r as u64 + 1);
                let b = permute_pattern(&sp, &q);
                let bcp: Vec<i32> = b.col_ptr.iter().map(|&x| x as i32).collect();
                let bri: Vec<i32> = b.row_idx.iter().map(|&x| x as i32).collect();
                if let Some(bcore) = feral_ordering_core::CscPattern::new(n, &bcp, &bri) {
                    if let Ok(pb) = feral_amd::amd_order(&bcore) {
                        let perm: Vec<usize> =
                            pb.iter().map(|&x| q[x as usize]).collect();
                        if is_bijection(&perm, n) {
                            best = best.min(flops_of(&sp, &perm) as f64);
                        }
                    }
                }
                for (si, &s) in stops.iter().enumerate() {
                    if r + 1 == s {
                        marks[si] = best;
                    }
                }
            }
        }
        let secs = t0.elapsed().as_secs_f64();

        let b = bucket(n);
        let rc = cur_flops / base;
        cur.0[b] += rc.ln();
        cur.1[b] += 1;
        for si in 0..stops.len() {
            at[si].0[b] += (marks[si] / base).ln();
            at[si].1[b] += 1;
        }
        rows.push((secs, name.clone(), n, nnz, rc, best / base));
    }

    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    println!("\n--- relabelled-AMD restarts: biggest improvements ---");
    let mut imp: Vec<_> = rows.iter().filter(|r| r.5 < r.4 - 1e-9).collect();
    imp.sort_by(|a, b| (a.5 / a.4).partial_cmp(&(b.5 / b.4)).unwrap());
    for (secs, name, n, nnz, rc, rn) in imp.iter().take(40) {
        println!("{name:<30} n={n:<7} nnz={nnz:<8} {rc:.4} -> {rn:.4}  ({secs:.3}s for {RESTARTS})");
    }
    println!("improved {} of {}", imp.len(), rows.len());

    println!("\n--- 12 most expensive ({RESTARTS} restarts) ---");
    for (secs, name, n, nnz, _, _) in rows.iter().take(12) {
        println!("{secs:8.3}s  {name:<30} n={n:<7} nnz={nnz}");
    }

    println!("\nSCORE cur          = {:.6}", aggregate(&cur.0, &cur.1));
    for (si, &s) in stops.iter().enumerate() {
        println!("SCORE {s:>2} restarts  = {:.6}", aggregate(&at[si].0, &at[si].1));
    }
}

/// Per-FAMILY cost/benefit. The blanket sweep in [`probe_multiseed`] showed the
/// wins are concentrated in a few variants but the total cost (up to 2.4 s) is
/// unaffordable. This probe times each variant SEPARATELY and records the ratio
/// it alone would achieve, so a gate can be chosen per family from measurement
/// instead of by guesswork.
///
/// Output is one TSV row per matrix: `cur_s cur_ratio` then `(secs, ratio)` for
/// every labelled variant, in `FAMILY_LABELS` order.
#[test]
#[ignore]
fn probe_family() {
    const MAX_N: usize = 30_000;
    const MAX_NNZ: usize = 60_000;

    let corpus = crate::corpus::corpus();
    let mut header = String::from("matrix\tn\tnnz\tcur_s\tcur_r");
    for l in FAMILY_LABELS {
        header.push_str(&format!("\t{l}_s\t{l}_r"));
    }
    println!("\n{header}");

    for (name, pat) in &corpus {
        let n = pat.n;
        if n == 0 || n >= MAX_N || pat.nnz() >= MAX_NNZ {
            continue;
        }
        let nnz = pat.nnz();
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let base = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        ) as f64;

        let t0 = Instant::now();
        let cur = flops_of(&sp, &order(pat)) as f64 / base;
        let cur_s = t0.elapsed().as_secs_f64();

        let mut row = format!("{name}\t{n}\t{nnz}\t{cur_s:.4}\t{cur:.4}");
        for (i, _) in FAMILY_LABELS.iter().enumerate() {
            let t = Instant::now();
            let p = family_perm(i, &core);
            let secs = t.elapsed().as_secs_f64();
            let r = match p {
                Some(p) => {
                    let p: Vec<usize> = p.into_iter().map(|x| x as usize).collect();
                    if is_bijection(&p, n) {
                        flops_of(&sp, &p) as f64 / base
                    } else {
                        f64::NAN
                    }
                }
                None => f64::NAN,
            };
            row.push_str(&format!("\t{secs:.4}\t{r:.4}"));
        }
        println!("{row}");
    }
}

/// Labels for the variants measured by [`probe_family`], in index order.
const FAMILY_LABELS: [&str; 12] = [
    "kahip_fast2",
    "kahip_fast3",
    "kahip_eco",
    "kahip_strong",
    "metis_s21",
    "metis_s2",
    "metis_imb10",
    "metis_imb05",
    "metis_sw100",
    "metis_sw400",
    "metis_dq",
    "scotch_s2",
];

/// The variant at `idx`, run on `core`. Kept in one place so [`probe_family`]
/// and any follow-up probe agree on what a label means.
fn family_perm(idx: usize, core: &feral_ordering_core::CscPattern<'_>) -> Option<Vec<i32>> {
    use feral_kahip::{KahipMode, KahipOptions};
    use feral_metis::MetisOptions;
    match idx {
        0 => feral_kahip::kahip_order_full(core, &KahipOptions { seed: 2, ..Default::default() })
            .ok()
            .map(|(p, _, _)| p),
        1 => feral_kahip::kahip_order_full(core, &KahipOptions { seed: 3, ..Default::default() })
            .ok()
            .map(|(p, _, _)| p),
        2 => feral_kahip::kahip_order_full(
            core,
            &KahipOptions { mode: KahipMode::Eco, ..Default::default() },
        )
        .ok()
        .map(|(p, _, _)| p),
        3 => feral_kahip::kahip_order_full(
            core,
            &KahipOptions { mode: KahipMode::Strong, ..Default::default() },
        )
        .ok()
        .map(|(p, _, _)| p),
        4 => feral_metis::metis_order_full(core, &MetisOptions { seed: 21, ..Default::default() })
            .ok()
            .map(|(p, _, _)| p),
        5 => feral_metis::metis_order_full(core, &MetisOptions { seed: 2, ..Default::default() })
            .ok()
            .map(|(p, _, _)| p),
        6 => feral_metis::metis_order_full(
            core,
            &MetisOptions { max_imbalance: 0.10, ..Default::default() },
        )
        .ok()
        .map(|(p, _, _)| p),
        7 => feral_metis::metis_order_full(
            core,
            &MetisOptions { max_imbalance: 0.05, ..Default::default() },
        )
        .ok()
        .map(|(p, _, _)| p),
        8 => feral_metis::metis_order_full(
            core,
            &MetisOptions { nd_to_amd_switch: 100, ..Default::default() },
        )
        .ok()
        .map(|(p, _, _)| p),
        9 => feral_metis::metis_order_full(
            core,
            &MetisOptions { nd_to_amd_switch: 400, ..Default::default() },
        )
        .ok()
        .map(|(p, _, _)| p),
        10 => feral_metis::metis_order_full(
            core,
            &MetisOptions { dense_quotient_enabled: true, ..Default::default() },
        )
        .ok()
        .map(|(p, _, _)| p),
        11 => feral_scotch::scotch_order_full(
            core,
            &feral_scotch::ScotchOptions { seed: 2, ..Default::default() },
        )
        .ok()
        .map(|(p, _, _)| p),
        _ => None,
    }
}

/// The LARGE end of the corpus is where the `n` caps in `order()` shut every
/// candidate off — `acopf_case9241pegase_qcqp` (n=313k) gets nothing but the AMD
/// baseline. But the cost driver is nnz, not n, so some of those matrices may
/// have budget going unused. This probe reports, for every large matrix, how
/// long `order()` actually takes today and what a single extra AMF / METIS pass
/// would cost and buy.
#[test]
#[ignore]
fn probe_large() {
    let corpus = crate::corpus::corpus();
    println!("\nmatrix\tn\tnnz\tcur_s\tcur_ratio\tamf5_s\tamf5_r\tamfnd_s\tamfnd_r\tmetis_s\tmetis_r");
    for (name, pat) in &corpus {
        let n = pat.n;
        if n < 100_000 {
            continue;
        }
        let nnz = pat.nnz();
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let base = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        ) as f64;

        let t0 = Instant::now();
        let cur = flops_of(&sp, &order(pat)) as f64 / base;
        let cur_s = t0.elapsed().as_secs_f64();

        // One AMF pass at dense_alpha 5, and one with dense detection disabled.
        let mut out = Vec::new();
        for da in [5.0f64, -1.0] {
            let o = feral_amf::AmfOptions { dense_alpha: da, ..Default::default() };
            let t = Instant::now();
            let r = match feral_amf::amf_order_opts(&core, &o) {
                Ok((p, ..)) => {
                    let p: Vec<usize> = p.into_iter().map(|x| x as usize).collect();
                    if is_bijection(&p, n) { flops_of(&sp, &p) as f64 / base } else { f64::NAN }
                }
                Err(_) => f64::NAN,
            };
            out.push((t.elapsed().as_secs_f64(), r));
        }
        // One default METIS pass.
        let t = Instant::now();
        let mr = match feral_metis::metis_order_full(&core, &feral_metis::MetisOptions::default()) {
            Ok((p, _, _)) => {
                let p: Vec<usize> = p.into_iter().map(|x| x as usize).collect();
                if is_bijection(&p, n) { flops_of(&sp, &p) as f64 / base } else { f64::NAN }
            }
            Err(_) => f64::NAN,
        };
        let ms = t.elapsed().as_secs_f64();

        println!(
            "{name}\t{n}\t{nnz}\t{cur_s:.3}\t{cur:.4}\t{:.3}\t{:.4}\t{:.3}\t{:.4}\t{ms:.3}\t{mr:.4}",
            out[0].0, out[0].1, out[1].0, out[1].1
        );
    }
}

/// Measure what MULTI-SEED restarts of the seeded partitioners would buy.
///
/// METIS / Scotch / KaHIP all take a deterministic `seed`, and each seed yields
/// a genuinely different nested-dissection ordering (different coarsening
/// matchings and initial bisections). `order()` currently uses a single fixed
/// seed for each. This probe scores the best over several seeds *in addition to*
/// the current `order()` result, and reports both the score it would produce and
/// the extra wall-clock it costs — the two numbers needed to choose a gate.
#[test]
#[ignore]
fn probe_multiseed() {
    // Only sweep the genuinely cheap region; the slow tier (measured worst
    // order() = 1.02 s of a 2 s cap) has no slack for extra candidates.
    const SWEEP_MAX_NNZ: usize = 60_000;
    const SWEEP_MAX_N: usize = 30_000;

    let corpus = crate::corpus::corpus();
    let mut cur = ([0.0f64; 3], [0usize; 3]);
    let mut new = ([0.0f64; 3], [0usize; 3]);
    let mut rows: Vec<(f64, String, usize, usize, f64, f64, String)> = Vec::new();
    // How often each labelled variant is the unique/joint best — the histogram
    // that decides which ones are worth their runtime.
    let mut wins: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();

    for (name, pat) in &corpus {
        let n = pat.n;
        if n == 0 {
            continue;
        }
        let nnz = pat.nnz();
        let sp = scoring_pattern(pat);
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
        let cur_flops = flops_of(&sp, &order(pat));
        let mut best = cur_flops;
        let mut best_label = String::from("current");

        let t0 = Instant::now();
        if n < SWEEP_MAX_N && nnz < SWEEP_MAX_NNZ {
            let mut try_perm = |label: String, p: Vec<i32>| {
                let p: Vec<usize> = p.into_iter().map(|x| x as usize).collect();
                if !is_bijection(&p, n) {
                    return;
                }
                let f = flops_of(&sp, &p);
                if f < best {
                    best = f;
                    best_label = label;
                }
            };

            // (a) METIS seed restarts — each seed is a different coarsening
            //     matching and a different set of initial bisections.
            for seed in [2u64, 3, 5, 8, 13, 21, 34] {
                let o = feral_metis::MetisOptions { seed, ..Default::default() };
                if let Ok((p, _, _)) = feral_metis::metis_order_full(&core, &o) {
                    try_perm(format!("metis.seed{seed}"), p);
                }
            }
            // (b) METIS imbalance variants — a looser/tighter balance constraint
            //     changes every separator on the recursion.
            for imb in [0.05f64, 0.10, 0.40] {
                let o = feral_metis::MetisOptions { max_imbalance: imb, ..Default::default() };
                if let Ok((p, _, _)) = feral_metis::metis_order_full(&core, &o) {
                    try_perm(format!("metis.imb{imb}"), p);
                }
            }
            // (c) METIS ND→AMD switch point — how much of the tail is handed to
            //     minimum degree instead of further dissection.
            for sw in [40u32, 100, 400] {
                let o = feral_metis::MetisOptions { nd_to_amd_switch: sw, ..Default::default() };
                if let Ok((p, _, _)) = feral_metis::metis_order_full(&core, &o) {
                    try_perm(format!("metis.sw{sw}"), p);
                }
            }
            // (d) METIS quasi-dense quotient — pulls near-dense columns out of the
            //     ND graph. KKT patterns have exactly those dense coupling rows.
            {
                let o = feral_metis::MetisOptions {
                    dense_quotient_enabled: true,
                    ..Default::default()
                };
                if let Ok((p, _, _)) = feral_metis::metis_order_full(&core, &o) {
                    try_perm("metis.dq".into(), p);
                }
            }
            // (e) Scotch seed restarts.
            for seed in [1u64, 2, 3, 7] {
                let o = feral_scotch::ScotchOptions { seed, ..Default::default() };
                if let Ok((p, _, _)) = feral_scotch::scotch_order_full(&core, &o) {
                    try_perm(format!("scotch.seed{seed}"), p);
                }
            }
            // (f) KaHIP seeds and the two stronger modes.
            for seed in [2u64, 3, 5] {
                let o = feral_kahip::KahipOptions { seed, ..Default::default() };
                if let Ok((p, _, _)) = feral_kahip::kahip_order_full(&core, &o) {
                    try_perm(format!("kahip.seed{seed}"), p);
                }
            }
            for (tag, mode) in [("eco", feral_kahip::KahipMode::Eco), ("strong", feral_kahip::KahipMode::Strong)] {
                let o = feral_kahip::KahipOptions { mode, ..Default::default() };
                if let Ok((p, _, _)) = feral_kahip::kahip_order_full(&core, &o) {
                    try_perm(format!("kahip.{tag}"), p);
                }
            }
        }
        let extra = t0.elapsed().as_secs_f64();

        let b = bucket(n);
        let rc = cur_flops as f64 / base as f64;
        let rn = best as f64 / base as f64;
        cur.0[b] += rc.ln();
        cur.1[b] += 1;
        new.0[b] += rn.ln();
        new.1[b] += 1;
        *wins.entry(best_label.clone()).or_insert(0) += 1;
        rows.push((extra, name.clone(), n, nnz, rc, rn, best_label));
    }

    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    println!("\n--- sweep cost + effect, most expensive first (TSV) ---");
    println!("extra_s\tmatrix\tn\tnnz\tcur\tnew\twinner");
    for (e, name, n, nnz, rc, rn, lab) in rows.iter() {
        println!("{e:.4}\t{name}\t{n}\t{nnz}\t{rc:.4}\t{rn:.4}\t{lab}");
    }

    println!("\n--- winner histogram ---");
    let mut w: Vec<_> = wins.into_iter().collect();
    w.sort_by(|a, b| b.1.cmp(&a.1));
    for (label, k) in w {
        println!("{k:>4}  {label}");
    }

    for b in 0..3 {
        if cur.1[b] > 0 {
            println!(
                "{:<8} cur={:.4}  new={:.4}",
                BUCKET_NAMES[b],
                (cur.0[b] / cur.1[b] as f64).exp(),
                (new.0[b] / new.1[b] as f64).exp()
            );
        }
    }
    println!("SCORE cur = {:.6}", aggregate(&cur.0, &cur.1));
    println!("SCORE new = {:.6}", aggregate(&new.0, &new.1));
    println!("worst EXTRA sweep time = {:.3} s on {}", rows[0].0, rows[0].1);
}

/// Score the BUDGETED relabelled-AMD multi-start and measure its true combined
/// cost.
///
/// [`probe_relabel_amd`] established the family works — 41 of 300 matrices
/// improved, 0.883906 -> 0.874024 at a flat 24 restarts. But a flat count is
/// unshippable: 24 restarts costs 1.444 s on `nuclear10a` and 0.658 s on
/// `crudeoil_lee4_10`, on top of each matrix's own `order()` time, which would
/// put the heavy tier near or past the 2 s SIGKILL.
///
/// This probe evaluates the fix — spend a fixed budget per matrix instead of a
/// fixed count (see [`relabel_restarts`]) — across several `(budget, cap)`
/// settings. It runs enough restarts per matrix to satisfy every policy, records
/// the running-best flops and cumulative cost after each restart, then reads off
/// each policy from that one sweep. `worst_s` is the real number that decides
/// shippability: measured `order()` time PLUS measured restart time, per matrix.
#[test]
#[ignore]
fn probe_relabel_budget() {
    const POLICIES: [(usize, usize); 8] = [
        (150_000, 24),
        (300_000, 24),
        (300_000, 48),
        (450_000, 24),
        (450_000, 48),
        (600_000, 48),
        (600_000, 96),
        (900_000, 96),
    ];

    let corpus = crate::corpus::corpus();
    let np = POLICIES.len();
    let mut cur = ([0.0f64; 3], [0usize; 3]);
    let mut pol: Vec<([f64; 3], [usize; 3])> = vec![([0.0; 3], [0; 3]); np];
    let mut worst: Vec<(f64, String)> = vec![(0.0, String::new()); np];
    let mut improved = vec![0usize; np];

    for (name, pat) in &corpus {
        let n = pat.n;
        if n == 0 {
            continue;
        }
        let nnz = pat.nnz();
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let base = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        ) as f64;

        let t_ord = Instant::now();
        let cur_flops = flops_of(&sp, &order(pat)) as f64;
        let ord_secs = t_ord.elapsed().as_secs_f64();

        let rmax = POLICIES
            .iter()
            .map(|&(b, c)| relabel_restarts(b, c, nnz))
            .max()
            .unwrap_or(0);

        // best_after[r] / cum[r]: best flops and seconds spent after r restarts.
        let mut best_after = vec![cur_flops; rmax + 1];
        let mut cum = vec![0.0f64; rmax + 1];
        let mut best = cur_flops;
        let t0 = Instant::now();
        for r in 0..rmax {
            let q = relabel(n, r as u64 + 1);
            let b = permute_pattern(&sp, &q);
            let bcp: Vec<i32> = b.col_ptr.iter().map(|&x| x as i32).collect();
            let bri: Vec<i32> = b.row_idx.iter().map(|&x| x as i32).collect();
            if let Some(bcore) = feral_ordering_core::CscPattern::new(n, &bcp, &bri) {
                if let Ok(pb) = feral_amd::amd_order(&bcore) {
                    let perm: Vec<usize> = pb.iter().map(|&x| q[x as usize]).collect();
                    if is_bijection(&perm, n) {
                        best = best.min(flops_of(&sp, &perm) as f64);
                    }
                }
            }
            best_after[r + 1] = best;
            cum[r + 1] = t0.elapsed().as_secs_f64();
        }

        let bkt = bucket(n);
        cur.0[bkt] += (cur_flops / base).ln();
        cur.1[bkt] += 1;
        for (pi, &(bd, cap)) in POLICIES.iter().enumerate() {
            let r = relabel_restarts(bd, cap, nnz);
            let f = best_after[r];
            pol[pi].0[bkt] += (f / base).ln();
            pol[pi].1[bkt] += 1;
            if f < cur_flops - 1e-9 {
                improved[pi] += 1;
            }
            let combined = ord_secs + cum[r];
            if combined > worst[pi].0 {
                worst[pi] = (combined, name.clone());
            }
        }
    }

    println!("\nSCORE cur = {:.6}", aggregate(&cur.0, &cur.1));
    println!(
        "\n{:>9} {:>4} {:>10} {:>9} {:>9}  {}",
        "budget", "cap", "score", "worst_s", "improved", "worst matrix"
    );
    for (pi, &(bd, cap)) in POLICIES.iter().enumerate() {
        println!(
            "{bd:>9} {cap:>4} {:>10.6} {:>9.3} {:>9}  {}",
            aggregate(&pol[pi].0, &pol[pi].1),
            worst[pi].0,
            improved[pi],
            worst[pi].1
        );
    }
    println!("\nper-bucket for each policy:");
    for (pi, &(bd, cap)) in POLICIES.iter().enumerate() {
        print!("{bd:>9} {cap:>4} ");
        for b in 0..3 {
            if pol[pi].1[b] > 0 {
                print!(
                    " {}={:.4}",
                    BUCKET_NAMES[b],
                    (pol[pi].0[b] / pol[pi].1[b] as f64).exp()
                );
            }
        }
        println!();
    }
}


/// Search-policy modes for [`probe_relabel_search`].
/// * `FIXED` — perturb the accepted base by `max(1, n/div)` transpositions.
/// * `DECAY` — geometrically shrinking strength `n/2, n/4, n/8, …` (the first
///   exploit step is nearly a uniform draw, so little breadth is given up).
/// * `NOCHAIN` — always perturb the best *i.i.d.* relabeling and never adopt a
///   perturbation as the new base: pure neighbourhood sampling, no hill climb.
const FIXED: u8 = 0;
const DECAY: u8 = 1;
const NOCHAIN: u8 = 2;
/// * `DECAY0` — like `DECAY` but one step wider (`n, n/2, n/4, …`), so the first
///   exploit step is a near-uniform draw and no breadth at all is given up.
const DECAY0: u8 = 3;
/// * `RESET` — variable-neighbourhood search: `n, n/2, n/4, …`, but the shrink
///   counter resets to its widest whenever the base improves.
const RESET: u8 = 4;

/// Compare SEARCH POLICIES for the relabelled-AMD multi-start at a FIXED restart
/// count — i.e. at identical cost.
///
/// The shipped policy draws every relabeling i.i.d. uniformly, which is
/// memoryless: a relabeling AMD happens to like teaches the next restart nothing.
/// The alternative is to spend part of the budget hill-climbing — perturb the best
/// relabeling found so far and accept the perturbation when it lowers flops.
/// Because both cost exactly one AMD pass per restart, the comparison is
/// cost-neutral and the only question is which SAMPLES are worth more.
///
/// Each policy is `(name, num, den, div, mode)`: the first
/// `ceil(restarts * num / den)` restarts are i.i.d.; the rest perturb per `mode`.
///
/// Scores here are the PURE relabel family measured against AMD — the portfolio's
/// other candidates are not run, because they are identical across policies and
/// would only mask the differences under a `min`. Timing is not measured: every
/// policy performs exactly `restarts` AMD passes plus one O(n) relabeling each, so
/// the shipped cost model is unchanged by construction. That also makes the probe
/// cheap (~10 s for the whole corpus), so the space is worth sweeping rather than
/// guessing.
#[test]
#[ignore]
fn probe_relabel_search() {
    const POLICIES: [(&str, usize, usize, usize, u8); 17] = [
        ("iid (shipped)", 1, 1, 0, FIXED),
        ("7/8 n/2", 7, 8, 2, FIXED),
        ("3/4 n/2", 3, 4, 2, FIXED),
        ("2/3 n/2", 2, 3, 2, FIXED),
        ("1/2 n/2", 1, 2, 2, FIXED),
        ("3/4 n/2 nochain", 3, 4, 2, NOCHAIN),
        ("7/8 decay", 7, 8, 0, DECAY),
        ("3/4 decay", 3, 4, 0, DECAY),
        ("2/3 decay", 2, 3, 0, DECAY),
        ("1/2 decay", 1, 2, 0, DECAY),
        ("7/8 decay0", 7, 8, 0, DECAY0),
        ("3/4 decay0", 3, 4, 0, DECAY0),
        ("2/3 decay0", 2, 3, 0, DECAY0),
        ("1/2 decay0", 1, 2, 0, DECAY0),
        ("3/4 reset", 3, 4, 0, RESET),
        ("2/3 reset", 2, 3, 0, RESET),
        ("1/2 reset", 1, 2, 0, RESET),
    ];

    let corpus = crate::corpus::corpus();
    let np = POLICIES.len();
    let mut pol: Vec<([f64; 3], [usize; 3])> = vec![([0.0; 3], [0; 3]); np];
    let mut better = vec![0usize; np];
    let mut worse = vec![0usize; np];
    // ROBUSTNESS: the same accumulators over disjoint halves of the corpus
    // (even/odd position), so a policy's advantage can be checked for being one
    // lucky matrix rather than a real effect.
    let mut half: Vec<[([f64; 3], [usize; 3]); 2]> = vec![[([0.0; 3], [0; 3]); 2]; np];
    // Per-matrix log-ratio delta vs i.i.d., to re-score with the single biggest
    // contributor dropped.
    let mut contrib: Vec<Vec<(f64, usize, f64, f64)>> = vec![Vec::new(); np];
    // Per-matrix movement vs i.i.d. for the leading policy, for attribution.
    let mut moves: Vec<(f64, String, usize, usize)> = Vec::new();
    let mut idx = 0usize;
    const ATTRIB: usize = 7; // index of "3/4 decay" above

    for (name, pat) in &corpus {
        let n = pat.n;
        if n == 0 {
            continue;
        }
        let nnz = pat.nnz();
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let amd: Vec<usize> = feral_amd::amd_order(&core)
            .unwrap()
            .into_iter()
            .map(|x| x as usize)
            .collect();
        let amd_flops = flops_of(&sp, &amd);
        let base_f = amd_flops as f64;
        let bkt = bucket(n);

        // One AMD-under-relabeling evaluation, in flops.
        let eval = |q: &[usize]| -> Option<u64> {
            let b = permute_pattern(&sp, q);
            let bcp: Vec<i32> = b.col_ptr.iter().map(|&x| x as i32).collect();
            let bri: Vec<i32> = b.row_idx.iter().map(|&x| x as i32).collect();
            let bcore = feral_ordering_core::CscPattern::new(n, &bcp, &bri)?;
            let pb = feral_amd::amd_order(&bcore).ok()?;
            let perm: Vec<usize> = pb.iter().map(|&x| q[x as usize]).collect();
            if !is_bijection(&perm, n) {
                return None;
            }
            Some(flops_of(&sp, &perm))
        };

        let restarts = relabel_restarts(RELABEL_BUDGET, RELABEL_MAX_RESTARTS, nnz);

        // The i.i.d. prefix is shared by every policy, so evaluate it ONCE.
        let mut iid: Vec<(Vec<usize>, u64)> = Vec::with_capacity(restarts);
        for r in 0..restarts {
            let q = relabel(n, r as u64 + 1);
            let f = eval(&q).unwrap_or(u64::MAX);
            iid.push((q, f));
        }

        let mut iid_only = u64::MAX;
        for pi in 0..np {
            let (_, num, den, div, mode) = POLICIES[pi];
            let explore = (restarts * num).div_ceil(den).min(restarts);

            // Replay the accept logic over the shared i.i.d. prefix.
            let mut base_q: Vec<usize> = (0..n).collect();
            let mut base_flops = amd_flops;
            let mut best = amd_flops;
            for r in 0..explore {
                let (q, f) = &iid[r];
                if *f < base_flops {
                    base_flops = *f;
                    base_q = q.clone();
                }
                best = best.min(*f);
            }
            let anchor_q = base_q.clone();
            let mut since = 0usize; // exploit steps since the base last improved
            // Spend what is left perturbing.
            for (t, r) in (explore..restarts).enumerate() {
                let swaps = match mode {
                    DECAY => (n >> (t + 1).min(20)).max(1),
                    DECAY0 => (n >> t.min(20)).max(1),
                    RESET => (n >> since.min(20)).max(1),
                    _ => (n / div).max(1),
                };
                let from: &[usize] = if mode == NOCHAIN { &anchor_q } else { &base_q };
                let q = perturb(from, swaps, r as u64 + 1);
                let Some(f) = eval(&q) else {
                    since += 1;
                    continue;
                };
                if mode != NOCHAIN && f < base_flops {
                    base_flops = f;
                    base_q = q;
                    since = 0;
                } else {
                    since += 1;
                }
                best = best.min(f);
            }

            if pi == 0 {
                iid_only = best;
            } else if best < iid_only {
                better[pi] += 1;
            } else if best > iid_only {
                worse[pi] += 1;
            }
            let lr = (best as f64 / base_f).ln();
            pol[pi].0[bkt] += lr;
            pol[pi].1[bkt] += 1;
            let h = idx % 2;
            half[pi][h].0[bkt] += lr;
            half[pi][h].1[bkt] += 1;
            let lr_iid = (iid_only as f64 / base_f).ln();
            contrib[pi].push((lr - lr_iid, bkt, lr, lr_iid));

            if pi == ATTRIB && best != iid_only {
                moves.push((best as f64 / iid_only as f64 - 1.0, name.clone(), n, restarts));
            }
        }
        idx += 1;
    }

    println!(
        "\n{:>18} {:>10} {:>9} {:>8} {:>8}",
        "policy", "score", "d_vs_iid", "better", "worse"
    );
    let base = aggregate(&pol[0].0, &pol[0].1);
    for pi in 0..np {
        let s = aggregate(&pol[pi].0, &pol[pi].1);
        println!(
            "{:>18} {s:>10.6} {:>+9.6} {:>8} {:>8}",
            POLICIES[pi].0,
            s - base,
            better[pi],
            worse[pi]
        );
    }
    // ROBUSTNESS. `dA`/`dB` are the policy's advantage over i.i.d. measured on two
    // disjoint halves of the corpus; `d_drop1` is the full-corpus advantage with
    // the single largest-contributing matrix removed. A real effect shows the same
    // sign in both halves and survives `drop1`. An advantage that is one lucky
    // matrix collapses under `drop1` and flips sign between halves.
    println!(
        "\n{:>18} {:>10} {:>10} {:>10}   (robustness: same sign in both halves + survives drop1)",
        "policy", "dA", "dB", "d_drop1"
    );
    let base_a = aggregate(&half[0][0].0, &half[0][0].1);
    let base_b = aggregate(&half[0][1].0, &half[0][1].1);
    for pi in 0..np {
        // Re-score BOTH this policy and i.i.d. with the single matrix that moved
        // the most (in either direction) removed from each.
        let mut p = pol[pi];
        let mut q = pol[0];
        if let Some(&(d, dbkt, lr_p, lr_i)) = contrib[pi]
            .iter()
            .max_by(|a, b| a.0.abs().partial_cmp(&b.0.abs()).unwrap())
        {
            let _ = d;
            p.0[dbkt] -= lr_p;
            p.1[dbkt] -= 1;
            q.0[dbkt] -= lr_i;
            q.1[dbkt] -= 1;
        }
        println!(
            "{:>18} {:>+10.6} {:>+10.6} {:>+10.6}",
            POLICIES[pi].0,
            aggregate(&half[pi][0].0, &half[pi][0].1) - base_a,
            aggregate(&half[pi][1].0, &half[pi][1].1) - base_b,
            aggregate(&p.0, &p.1) - aggregate(&q.0, &q.1)
        );
    }

    println!("\nper-bucket:");
    for pi in 0..np {
        print!("{:>18} ", POLICIES[pi].0);
        for b in 0..3 {
            if pol[pi].1[b] > 0 {
                print!(
                    " {}={:.4}",
                    BUCKET_NAMES[b],
                    (pol[pi].0[b] / pol[pi].1[b] as f64).exp()
                );
            }
        }
        println!();
    }

    moves.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    println!("\n{} vs iid — biggest per-matrix relabel-flops moves:", POLICIES[ATTRIB].0);
    for (d, name, n, r) in moves.iter().take(8) {
        println!("  {name:36} n={n:<8} restarts={r:<3} {:+.2}%", d * 100.0);
    }
    println!("  ...");
    for (d, name, n, r) in moves.iter().rev().take(8) {
        println!("  {name:36} n={n:<8} restarts={r:<3} {:+.2}%", d * 100.0);
    }
}

/// One extra bounded subtree pass on top of the complete shipped chain.
#[test]
#[ignore]
fn probe_next_subtree_variants() {
    let corpus = crate::corpus::corpus();
    let mut base = ([0.0f64; 3], [0usize; 3]);
    // label, round, blocks, min_s, max_s, max_sub, budget, streams, ranked
    let variants = [
        ("b4.x2m.768", 5, 4, 16, 768, 1_200, 2_000_000, 1, true),
        ("b4.x4m.768", 5, 4, 16, 768, 1_200, 4_000_000, 1, true),
        ("b4.x6m.768", 5, 4, 16, 768, 1_200, 6_000_000, 1, true),
        ("b4.x8m.768", 5, 4, 16, 768, 1_200, 8_000_000, 1, true),
        ("b8.x1m.1200", 5, 8, 16, 1_200, 1_200, 1_000_000, 1, true),
        ("b8.x2m.1200", 5, 8, 16, 1_200, 1_200, 2_000_000, 1, true),
        ("b12.x1m.1200", 5, 12, 16, 1_200, 1_200, 1_000_000, 1, true),
    ];
    let n_variants = variants.len();
    let mut vsum = vec![([0.0f64; 3], [0usize; 3]); n_variants];
    let mut improved: Vec<Vec<(String, usize, usize, u64, u64)>> =
        vec![Vec::new(); n_variants];

    for (name, pat) in &corpus {
        let n = pat.n;
        if n == 0 {
            continue;
        }
        let nnz = pat.nnz();
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let amd_flops = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        );
        let bkt = bucket(n);

        let inc = order(pat);
        let inc_flops = flops_of(&sp, &inc);
        base.0[bkt] += (inc_flops as f64 / amd_flops as f64).ln();
        base.1[bkt] += 1;

        let in_gate = (1_000..=80_000).contains(&n) && nnz <= 250_000;
        for vi in 0..n_variants {
            let mut out_flops = inc_flops;
            if in_gate {
                // Rebuild postorder/counts/parent for the shipped incumbent, then
                // run one extra subtree_refine with the variant config.
                let permuted = permute_pattern(&sp, &inc);
                let etree = EliminationTree::from_pattern(&permuted);
                let post = etree.postorder();
                let mut candidate: Vec<usize> = post.iter().map(|&j| inc[j]).collect();
                let post_pattern = permute_pattern(&sp, &candidate);
                let post_etree = EliminationTree::from_pattern(&post_pattern);
                let counts: Vec<u32> = column_counts_gnp(&post_pattern, &post_etree)
                    .into_iter()
                    .map(|c| c as u32)
                    .collect();
                let parent: Vec<i32> = post_etree
                    .parent
                    .iter()
                    .map(|p| p.map_or(-1, |j| j as i32))
                    .collect();
                let (_, round, blocks, min_s, max_s, max_sub, budget, streams, ranked) =
                    variants[vi];
                let mut cfg = SUBTREE_CFG;
                cfg.round = round;
                cfg.max_blocks = blocks;
                cfg.min_s = min_s;
                cfg.max_s = max_s;
                cfg.max_sub = max_sub;
                cfg.budget = budget;
                cfg.streams = streams;
                cfg.rank_blocks = ranked;
                let improved3 = rgreedy::subtree_refine(
                    n,
                    &pat.col_ptr,
                    &pat.row_idx,
                    &mut candidate,
                    &counts,
                    &parent,
                    cfg,
                );
                if improved3 > 0 && is_bijection(&candidate, n) {
                    let f = flops_of(&sp, &candidate);
                    if f < out_flops {
                        out_flops = f;
                    }
                }
            }
            let r = out_flops as f64 / amd_flops as f64;
            vsum[vi].0[bkt] += r.ln();
            vsum[vi].1[bkt] += 1;
            if out_flops < inc_flops {
                improved[vi].push((name.clone(), n, nnz, inc_flops, out_flops));
            }
        }
    }

    let base_score = aggregate(&base.0, &base.1);
    println!("\nshipped order(): {base_score:.6}");
    for (vi, (lbl, ..)) in variants.iter().enumerate() {
        let s = aggregate(&vsum[vi].0, &vsum[vi].1);
        println!(
            "{lbl:>24} score {s:.6}  d {:+.6}  buckets {:.6}/{:.6}/{:.6}  improved {} matrices",
            s - base_score,
            (vsum[vi].0[0] / vsum[vi].1[0] as f64).exp(),
            (vsum[vi].0[1] / vsum[vi].1[1] as f64).exp(),
            (vsum[vi].0[2] / vsum[vi].1[2] as f64).exp(),
            improved[vi].len()
        );
        let mut rows = improved[vi].clone();
        rows.sort_by(|a, b| {
            (a.4 as f64 / a.3 as f64)
                .partial_cmp(&(b.4 as f64 / b.3 as f64))
                .unwrap()
        });
        for (name, n, nnz, cf, xf) in rows.iter().take(14) {
            println!(
                "    {name:34} n={n:<7} nnz={nnz:<8} {cf} -> {xf} ({:.4})",
                *xf as f64 / *cf as f64
            );
        }
    }
}

/// Cost AND benefit of tie-breaking candidates on the matrices still tied at the
/// AMD baseline in the two leverage-rich buckets (`1k_10k`, `gt_10k`).
///
/// Every tie is pure upside under the best-of floor, so the only question a new
/// candidate raises is whether its wall-clock fits the budget. This prints, per
/// (tied matrix, candidate), the seconds it costs and the ratio it would reach,
/// which is exactly the pair needed to choose a gate from data instead of
/// guessing. Ties are detected by running the shipped `order()` first, so the
/// `cur_s` column also says how much headroom that matrix still has.
#[test]
#[ignore]
fn probe_tie_breakers() {
    let corpus = crate::corpus::corpus();
    println!("\nmatrix\tn\tnnz\tcur_s\tcand\tcand_s\tcand_r");
    for (name, pat) in &corpus {
        let n = pat.n;
        if n < 1_000 {
            continue;
        }
        let nnz = pat.nnz();
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let base = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        ) as f64;

        let t0 = Instant::now();
        let cur = flops_of(&sp, &order(pat)) as f64 / base;
        let cur_s = t0.elapsed().as_secs_f64();
        // Only the ties: where AMD still beats the whole shipped portfolio.
        if cur < 0.9999 {
            continue;
        }

        let run = |label: &str,
                   f: &dyn Fn() -> Result<Vec<i32>, feral_ordering_core::OrderingError>| {
            let t = Instant::now();
            let r = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
                Ok(Ok(p)) => {
                    let p: Vec<usize> = p.into_iter().map(|x| x as usize).collect();
                    if is_bijection(&p, n) {
                        flops_of(&sp, &p) as f64 / base
                    } else {
                        f64::NAN
                    }
                }
                _ => f64::NAN,
            };
            let s = t.elapsed().as_secs_f64();
            println!("{name}\t{n}\t{nnz}\t{cur_s:.3}\t{label}\t{s:.3}\t{r:.4}");
        };

        // METIS: more WORK (trials / refinement).
        let m_tuned = feral_metis::MetisOptions {
            niparts: 16,
            fm_passes: 20,
            ..Default::default()
        };
        run("metis_tuned", &|| {
            feral_metis::metis_order_full(&core, &m_tuned).map(|(p, _, _)| p)
        });
        let m_hi = feral_metis::MetisOptions {
            niparts: 32,
            fm_passes: 30,
            ..Default::default()
        };
        run("metis_hi", &|| {
            feral_metis::metis_order_full(&core, &m_hi).map(|(p, _, _)| p)
        });
        // METIS: different SHAPE (seed / crossover / imbalance).
        for sd in [7u64, 21] {
            let o = feral_metis::MetisOptions {
                seed: sd,
                ..Default::default()
            };
            run(&format!("metis_seed{sd}"), &|| {
                feral_metis::metis_order_full(&core, &o).map(|(p, _, _)| p)
            });
        }
        for sw in [100u32, 400, 1000] {
            let o = feral_metis::MetisOptions {
                nd_to_amd_switch: sw,
                ..Default::default()
            };
            run(&format!("metis_sw{sw}"), &|| {
                feral_metis::metis_order_full(&core, &o).map(|(p, _, _)| p)
            });
        }
        for imb in [0.05f64, 0.30] {
            let o = feral_metis::MetisOptions {
                max_imbalance: imb,
                ..Default::default()
            };
            run(&format!("metis_imb{imb}"), &|| {
                feral_metis::metis_order_full(&core, &o).map(|(p, _, _)| p)
            });
        }
        // Scotch / KaHIP — distinct separator engines.
        run("scotch", &|| feral_scotch::scotch_order(&core));
        let sc_tuned = feral_scotch::ScotchOptions {
            n_sep_trials: 10,
            ..Default::default()
        };
        run("scotch_tuned", &|| {
            feral_scotch::scotch_order_full(&core, &sc_tuned).map(|(p, _, _)| p)
        });
        run("kahip", &|| feral_kahip::kahip_order(&core));
        let kh_eco = feral_kahip::KahipOptions {
            mode: feral_kahip::KahipMode::Eco,
            ..Default::default()
        };
        run("kahip_eco", &|| {
            feral_kahip::kahip_order_full(&core, &kh_eco).map(|(p, _, _)| p)
        });
        // AMF at other dense_alpha — a different objective, not just more work.
        for da in [2.0f64, -1.0, 16.0] {
            let o = feral_amf::AmfOptions {
                dense_alpha: da,
                ..Default::default()
            };
            run(&format!("amf_a{da}"), &|| {
                feral_amf::amf_order_opts(&core, &o).map(|(p, ..)| p)
            });
        }
    }
}

/// Fast `gt_10k`-only score and timing for sweeping large-tier parameters.
/// Prints stable name-sorted rows so disjoint-half and drop-top robustness can
/// be calculated without rerunning the expensive candidate portfolio.
#[test]
#[ignore]
fn probe_gt10k() {
    let corpus = crate::corpus::corpus();
    let mut log_sum = 0.0f64;
    let mut count = 0usize;
    let mut worst = 0.0f64;
    let mut worst_name = String::new();
    let mut total_s = 0.0f64;
    let mut rows: Vec<(String, f64)> = Vec::new();

    for (name, pat) in &corpus {
        if pat.n < 10_000 {
            continue;
        }
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
        let base = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        ) as f64;

        let t0 = Instant::now();
        let perm = order(pat);
        let secs = t0.elapsed().as_secs_f64();
        total_s += secs;
        let ratio = flops_of(&sp, &perm) as f64 / base;
        log_sum += ratio.ln();
        count += 1;
        rows.push((name.clone(), ratio));
        if secs > worst {
            worst = secs;
            worst_name = name.clone();
        }
    }

    let geo = (log_sum / count as f64).exp();
    println!("\nGT10K_GEOMEAN = {geo:.6}  (count {count})");
    println!("GT10K_WORST = {worst:.3} s on {worst_name}");
    println!("GT10K_TOTAL = {total_s:.1} s");
    println!("--- GT10K rows ---");
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, ratio) in &rows {
        println!("ROW\t{name}\t{ratio:.6}");
    }
}

/// Fast `1k_10k` score and timing for sweeping medium-tier chain parameters.
#[test]
#[ignore]
fn probe_1k10k() {
    let corpus = crate::corpus::corpus();
    let mut log_sum = 0.0f64;
    let mut count = 0usize;
    let mut worst = 0.0f64;
    let mut worst_name = String::new();
    let mut total_s = 0.0f64;
    let mut rows: Vec<(String, f64)> = Vec::new();

    for (name, pat) in &corpus {
        if pat.n < 1_000 || pat.n >= 10_000 {
            continue;
        }
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
        let base = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        ) as f64;

        let t0 = Instant::now();
        let perm = order(pat);
        let secs = t0.elapsed().as_secs_f64();
        total_s += secs;
        let ratio = flops_of(&sp, &perm) as f64 / base;
        log_sum += ratio.ln();
        count += 1;
        rows.push((name.clone(), ratio));
        if secs > worst {
            worst = secs;
            worst_name = name.clone();
        }
    }

    let geo = (log_sum / count as f64).exp();
    println!("\n1K10K_GEOMEAN = {geo:.6}  (count {count})");
    println!("1K10K_WORST = {worst:.3} s on {worst_name}");
    println!("1K10K_TOTAL = {total_s:.1} s");
    println!("--- 1K10K rows ---");
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, ratio) in &rows {
        println!("ROW\t{name}\t{ratio:.6}");
    }
}

/// Fast `lt_1k` score and timing for bounded small-tier sweeps.
#[test]
#[ignore]
fn probe_lt1k() {
    let corpus = crate::corpus::corpus();
    let mut log_sum = 0.0f64;
    let mut count = 0usize;
    let mut worst = 0.0f64;
    let mut worst_name = String::new();
    let mut total_s = 0.0f64;
    let mut rows: Vec<(String, f64)> = Vec::new();

    for (name, pat) in &corpus {
        if pat.n == 0 || pat.n >= 1_000 {
            continue;
        }
        let sp = scoring_pattern(pat);
        let (cp, ri) = core_of(pat);
        let core = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
        let base = flops_of(
            &sp,
            &feral_amd::amd_order(&core)
                .unwrap()
                .into_iter()
                .map(|x| x as usize)
                .collect::<Vec<_>>(),
        ) as f64;

        let t0 = Instant::now();
        let perm = order(pat);
        let secs = t0.elapsed().as_secs_f64();
        total_s += secs;
        let ratio = flops_of(&sp, &perm) as f64 / base;
        log_sum += ratio.ln();
        count += 1;
        rows.push((name.clone(), ratio));
        if secs > worst {
            worst = secs;
            worst_name = name.clone();
        }
    }

    let geo = (log_sum / count as f64).exp();
    println!("\nLT1K_GEOMEAN = {geo:.6}  (count {count})");
    println!("LT1K_WORST = {worst:.3} s on {worst_name}");
    println!("LT1K_TOTAL = {total_s:.1} s");
    println!("--- LT1K rows ---");
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, ratio) in &rows {
        println!("ROW\t{name}\t{ratio:.6}");
    }
}
