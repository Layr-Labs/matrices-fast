//! CUSTOM QUOTIENT-GRAPH PIVOT-SELECTION METRICS (win **E**, see
//! `memory/index.md`). `feral_ordering_core::quotient_graph` exposes its
//! `select_pivot_amf` / `create_element_amf` Pass-1/Pass-2 graph bookkeeping
//! (supervariable merges, external-degree accumulation, mass elimination,
//! hash-bucket insertion) as PUBLIC pure-Rust items — none of it is
//! AMD/AMF-specific, it buckets by an arbitrary `i64` score cached in
//! `ws.wf`, quantized via `score_bucket_of`. AMF just happens to be the first
//! thing that writes into that field. This module forks ONLY the ~55-line
//! "compute this candidate's re-insertion score" tail of `finalize_step_amf`
//! (`finalize_step_variant` below) to try four alternative pivot-selection
//! formulas, and drives them through the SAME elimination loop AMF itself
//! runs (`select_pivot_amf` / `create_element_amf`, reused verbatim,
//! unmodified, from the vendor crate). `finalize_permutation` is likewise
//! reused unmodified (it only reads `pe`/`nv`/`elen`, which every variant
//! here still writes in the same places AMF does).
//!
//! Sanity-checked upstream (research sandbox, `task0_mechanism`-style test):
//! wiring plain AMD's own loose-degree formula through this exact plumbing
//! reproduces `feral_amd::amd_order` bit-for-bit, which is why this fork is
//! trusted to be faithful graph bookkeeping rather than a subtly different
//! algorithm.
//!
//! ## Why these four, and why "worse than AMD standalone" is not a bug
//!
//! Every variant here is frequently WORSE than AMD run alone (standalone
//! ratios of 1.13-7.3x measured upstream) — irrelevant, because these are
//! `consider()` candidates under the same best-of-portfolio floor as
//! everything else in this file: a candidate only has to win on ONE matrix,
//! anywhere, to pay for itself, and the four were kept specifically because
//! each wins on a DIFFERENT, largely non-overlapping subset of matrices (the
//! research sandbox swept ~25 formulas and found wins scattered rather than
//! clustered — evidence for keeping several cheap, differently-shaped
//! formulas rather than tuning one).
//!
//! - **`SqDiv`** — `deg² / (nv+1)`: a direct per-step estimate of a
//!   candidate's contribution to `Σ cⱼ²` (the exact objective this whole
//!   competition is scored on) if it were eliminated now: its `nv`
//!   soon-to-be-created columns would each have width ≈ `deg`, so their
//!   combined contribution is ≈ `nv · deg²`, and dividing by `(nv+1)` matches
//!   the AMF/AMMF convention of normalizing by prospective multiplicity
//!   rather than multiplying by it (multiplying — tried as `SqNv` upstream —
//!   was found to have the wrong sign).
//! - **`SqPure`** — `deg²`, no `nv` term at all: isolates whether squaring
//!   degree alone (without any supervariable weighting) still finds
//!   different-enough minima to be worth a candidate slot.
//! - **`Ammf`** — approximate minimum MEAN local fill (Rothberg & Eisenstat
//!   1998): `rmf_raw / deg`, i.e. fill PER NEIGHBOR rather than AMF's fill
//!   per prospective supervariable (`rmf_raw / (nv+1)`).
//! - **`AmindNorm`** — approximate minimum increase in neighbor degree
//!   (Rothberg & Eisenstat 1998), normalized: `(deg·(deg−1) − wf) / (nv+1)`,
//!   the un-normalized local-fill count (dropping AMF's `2·deg·degme` cross
//!   term, which folds in interaction with the pivot front currently being
//!   built) divided by prospective multiplicity, matching `SqDiv`'s
//!   convention.
//!
//! ## Cost — ANALYTIC ONLY, not independently timed
//!
//! Each variant reuses the identical `select_pivot_amf`/`create_element_amf`
//! loop AMF already runs at its call site, with marginally more floating-
//! point work per re-insertion (one extra multiply/divide, no new
//! allocation, no change in asymptotic complexity) — so the same cost CLASS
//! as one existing AMF pass. This has not been isolated-timed; treat it as an
//! unverified analytic bound, priced the same as any other single AMF-class
//! pass in the gate it shares.
//!
//! Wired in under the SAME envelope as the existing AMF `dense_alpha` sweep
//! (`n < AMF_SWEEP_MAX_N && nnz < 130_000`, see `mod.rs`) — no new envelope,
//! per the measurement that motivated this win.

use feral_ordering_core::quotient_graph::{
    clear_flag, create_element_amf, finalize_permutation, flip, select_pivot_amf, StepFlops,
    Workspace, WorkspaceOptions, NONE,
};
use feral_ordering_core::{CscPattern, OrderingError};

/// Which score formula `finalize_step_variant` computes at re-insertion. See
/// the module doc for what each one estimates and why it was kept.
///
/// The seven `Deg*`/`*Wf*`/`*Degme*` variants below (win **F**) are the top 7
/// of a 39-point sweep over powers/roots of loose-degree, mixtures with the
/// AMF fill accumulator `wf`, and a front-size (`degme`) cross term —
/// `metric_sweep.rs`'s `MetricSpec` grid, research sandbox `sbx-metrics2`.
/// Greedy forward-selection against the REAL `order()` portfolio (not AMD
/// alone) found these 7 the best COMBINED addition set at count 7
/// (0.854109 -> 0.840926 on the 427-matrix reconstructed-graded corpus,
/// `probe_diversity_marginal_sweep`); each is a pure `raw_deg`-based formula
/// (no AMF saturated-RMF branch), so all nine `Deg*`/`SqDiv`/`SqPure`
/// variants share the same write-back discipline. Same trust argument as
/// `SqDiv`/`SqPure`: identical Pass-1/Pass-2 bookkeeping, only the
/// re-insertion score differs.
///   - `DegSqrt` = `deg^0.5`
///   - `DegP075` = `deg^0.75`
///   - `DegP125` = `deg^1.25`
///   - `DegDivNvSqrtWf` = `deg/(nv+1) + 0.5 * sign(wf)*|wf|^0.5`
///   - `DegDivNvWfP15` = `deg/(nv+1) + 0.1 * sign(wf)*|wf|^1.5`
///   - `DegPlusDegme` = `deg + 1.0 * degme/(nv+1)`
///   - `DegDivNvDegme` = `deg/(nv+1) + 0.5 * degme/(nv+1)`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreVariant {
    SqDiv,
    SqPure,
    Ammf,
    AmindNorm,
    DegSqrt,
    DegP075,
    DegP125,
    DegDivNvSqrtWf,
    DegDivNvWfP15,
    DegPlusDegme,
    DegDivNvDegme,
}

/// `SqDiv`/`SqPure`/the seven win-F variants need an extra write-back of
/// `ws.degree[i]` after scoring (the AMF-style variants, `Ammf`/`AmindNorm`,
/// already write it inside their saturated/regular branch below) —
/// `ws.degree` must always hold the true AMD-style loose-degree estimate for
/// later steps' monotone-cap logic, regardless of which formula ranks
/// candidates for selection.
const DEGREE_FAMILY: &[ScoreVariant] = &[
    ScoreVariant::SqDiv,
    ScoreVariant::SqPure,
    ScoreVariant::DegSqrt,
    ScoreVariant::DegP075,
    ScoreVariant::DegP125,
    ScoreVariant::DegDivNvSqrtWf,
    ScoreVariant::DegDivNvWfP15,
    ScoreVariant::DegPlusDegme,
    ScoreVariant::DegDivNvDegme,
];

/// Saturation cap mirroring `algo::AMF_DUMMY_I32` (private to the vendor
/// crate and not re-exported, so this is an independent copy of the same
/// quantization convention MUMPS/AMF uses: `i32::MAX - 1`).
const SCORE_DUMMY_I32: i32 = i32::MAX - 1;

/// Quantize an `i64` score into a bucket index in `[0, 2n+1]`. Identical
/// convention to `algo::amf_bucket_of` (private there, so reimplemented here
/// — same fine-region-then-coarse-stride scheme, not a novel idea).
#[inline(always)]
fn score_bucket_of(score: i64, n: usize) -> usize {
    if score <= 0 {
        return 0;
    }
    let s = score as usize;
    if s <= n {
        return s;
    }
    let pas = (n / 8).max(1);
    let nbbuck = 2 * n;
    ((s - n) / pas + n).min(nbbuck)
}

/// Fork of `algo::finalize_step_amf`. Pass-1/Pass-2 graph bookkeeping is
/// copied unchanged from the vendor crate; only the "compute score, quantize,
/// bucket" block near the end (marked below) branches on `variant`.
#[allow(clippy::too_many_arguments)]
fn finalize_step_variant(
    ws: &mut Workspace,
    me: usize,
    pme1: usize,
    pme2_excl: usize,
    nvpiv: i32,
    degme: usize,
    elenme: i32,
    aggressive: bool,
    variant: ScoreVariant,
) -> StepFlops {
    let n = ws.n;
    let mut degme = degme;
    let mut nvpiv = nvpiv;

    // Pass 1: identical to finalize_step_amf.
    for pme in pme1..pme2_excl {
        let i = ws.iw[pme] as usize;
        let eln = ws.elen[i];
        if eln > 0 {
            let nvi = -ws.nv[i];
            let wnvi = ws.wflg - nvi;
            let pi = ws.pe[i] as usize;
            for k in 0..eln as usize {
                let e = ws.iw[pi + k] as usize;
                let mut we = ws.w[e];
                if we >= ws.wflg {
                    we -= nvi;
                } else if we != 0 {
                    we = ws.degree[e] + wnvi;
                    ws.wf[e] = 0;
                }
                ws.w[e] = we;
            }
        }
    }

    // Pass 2: identical structure to finalize_step_amf (deg/wf3/wf4
    // accumulation, aggressive absorption, mass elimination, hash-bucket
    // insertion). Only the loose-degree special case's zeroing target
    // differs cosmetically (still wf3/wf4, used identically below).
    for pme in pme1..pme2_excl {
        let i = ws.iw[pme] as usize;
        let p1 = ws.pe[i] as usize;
        let p2 = p1 + ws.elen[i] as usize;
        let mut pn = p1;
        let mut deg: usize = 0;
        let mut hash: usize = 0;
        let mut wf3: i64 = 0;
        let mut wf4: i64 = 0;
        let nvi = -ws.nv[i];

        if aggressive {
            for p in p1..p2 {
                let e = ws.iw[p] as usize;
                let we = ws.w[e];
                if we != 0 {
                    let dext = we - ws.wflg;
                    if dext > 0 {
                        if ws.wf[e] == 0 {
                            let d = dext as i64;
                            let de = ws.degree[e] as i64;
                            ws.wf[e] = d * (2 * de - d - 1);
                        }
                        wf4 += ws.wf[e];
                        deg += dext as usize;
                        ws.iw[pn] = e as i32;
                        pn += 1;
                        hash = hash.wrapping_add(e);
                    } else {
                        ws.pe[e] = flip(me as i32);
                        ws.w[e] = 0;
                    }
                }
            }
        } else {
            for p in p1..p2 {
                let e = ws.iw[p] as usize;
                let we = ws.w[e];
                if we != 0 {
                    let dext = (we - ws.wflg) as usize;
                    if ws.wf[e] == 0 {
                        let d = dext as i64;
                        let de = ws.degree[e] as i64;
                        ws.wf[e] = d * (2 * de - d - 1);
                    }
                    wf4 += ws.wf[e];
                    deg += dext;
                    ws.iw[pn] = e as i32;
                    pn += 1;
                    hash = hash.wrapping_add(e);
                }
            }
        }

        ws.elen[i] = (pn - p1 + 1) as i32;
        let p3 = pn;
        let p4 = p1 + ws.len[i] as usize;
        for p in p2..p4 {
            let j = ws.iw[p] as usize;
            let nvj = ws.nv[j];
            if nvj > 0 {
                deg += nvj as usize;
                wf3 += nvj as i64;
                ws.iw[pn] = j as i32;
                pn += 1;
                hash = hash.wrapping_add(j);
            }
        }

        if ws.elen[i] == 1 && p3 == pn {
            ws.pe[i] = flip(me as i32);
            let nvi_sv = -ws.nv[i];
            degme -= nvi_sv as usize;
            nvpiv += nvi_sv;
            ws.nel += nvi_sv as usize;
            ws.nv[i] = 0;
            ws.elen[i] = NONE;
            ws.n_mass_elim += 1;
        } else {
            if ws.degree[i] < deg as i32 {
                wf3 = 0;
                wf4 = 0;
            } else {
                ws.degree[i] = deg as i32;
            }
            // wf[i] used downstream as the raw fill-surface accumulator
            // (AMF's B3 term); every variant below re-derives its own
            // score from `ws.degree[i]` (= deg) and this `wf[i]`.
            ws.wf[i] = wf4 + 2 * nvi as i64 * wf3;

            if p1 != pn {
                ws.iw[pn] = ws.iw[p3];
            }
            if p3 != p1 {
                ws.iw[p3] = ws.iw[p1];
            }
            ws.iw[p1] = me as i32;
            ws.len[i] = (pn - p1 + 1) as i32;

            let h = hash % n;
            let j = ws.head[h];
            if j <= NONE {
                ws.next[i] = flip(j);
                ws.head[h] = flip(i as i32);
            } else {
                ws.next[i] = ws.last[j as usize];
                ws.last[j as usize] = i as i32;
            }
            ws.last[i] = h as i32;
        }
    }

    let degme_i32 = degme as i32;
    ws.degree[me] = degme_i32;
    if degme_i32 > ws.lemax {
        ws.lemax = degme_i32;
    }
    ws.wflg += ws.lemax;
    ws.wflg = clear_flag(ws.wflg, ws.wbig, &mut ws.w);

    // Supervariable detection: identical to finalize_step_amf (max-merge of
    // ws.wf, which every variant here treats as "the fill-surface
    // accumulator", so max-merge remains the right semantics regardless of
    // which final normalization turns it into a score).
    for pme in pme1..pme2_excl {
        let i_anchor = ws.iw[pme] as usize;
        if ws.nv[i_anchor] >= 0 {
            continue;
        }
        let h = ws.last[i_anchor] as usize;
        let j_head = ws.head[h];
        let mut i: i32 = if j_head == NONE {
            NONE
        } else if j_head < NONE {
            ws.head[h] = NONE;
            flip(j_head)
        } else {
            let chain_start = ws.last[j_head as usize];
            ws.last[j_head as usize] = NONE;
            chain_start
        };
        while i != NONE && ws.next[i as usize] != NONE {
            let i_u = i as usize;
            let ln = ws.len[i_u];
            let eln = ws.elen[i_u];
            let pi = ws.pe[i_u];
            for p in (pi + 1) as usize..(pi + ln) as usize {
                ws.w[ws.iw[p] as usize] = ws.wflg;
            }
            let mut jlast = i_u;
            let mut jp = ws.next[i_u];
            while jp != NONE {
                let jj = jp as usize;
                let mut ok = ws.len[jj] == ln && ws.elen[jj] == eln;
                if ok {
                    let pj = ws.pe[jj];
                    for p in (pj + 1) as usize..(pj + ln) as usize {
                        if ws.w[ws.iw[p] as usize] != ws.wflg {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok {
                    ws.pe[jj] = flip(i);
                    let wf_j = ws.wf[jj];
                    if wf_j > ws.wf[i_u] {
                        ws.wf[i_u] = wf_j;
                    }
                    ws.nv[i_u] += ws.nv[jj];
                    ws.nv[jj] = 0;
                    ws.elen[jj] = NONE;
                    jp = ws.next[jj];
                    ws.next[jlast] = jp;
                    ws.n_supervar_merge += 1;
                } else {
                    jlast = jj;
                    jp = ws.next[jj];
                }
            }
            ws.wflg += 1;
            i = ws.next[i_u];
        }
    }

    // ── Re-insertion: THE ONLY BLOCK THAT DIFFERS PER VARIANT ──────────
    let dummy_f = SCORE_DUMMY_I32 as f64;
    let n_f = if n == 0 { 1.0 } else { n as f64 };
    let mut p_write = pme1;
    let nleft = ws.n - ws.nel;
    for pme in pme1..pme2_excl {
        let i = ws.iw[pme] as usize;
        let nvi = -ws.nv[i];
        if nvi > 0 {
            ws.nv[i] = nvi;
            let degme_i = degme_i32;
            let nvi_i = nvi;
            let deg_i = ws.degree[i];
            let deg_f = deg_i as f64;
            let wf_f = ws.wf[i] as f64;

            // `raw_deg`: AMD's own loose-degree estimate, used by SqDiv/
            // SqPure. Computed the same way regardless of saturation,
            // mirroring AMD's own cap (not AMF's saturated-RMF branch below,
            // which targets the fill formula specifically).
            let raw_deg = (deg_f + degme_i as f64 - nvi_i as f64)
                .max(0.0)
                .min((nleft - nvi_i as usize) as f64);

            let score_f: f64 = match variant {
                ScoreVariant::SqDiv => (raw_deg * raw_deg) / (nvi_i as f64 + 1.0),
                ScoreVariant::SqPure => raw_deg * raw_deg,
                ScoreVariant::Ammf | ScoreVariant::AmindNorm => {
                    // Shared AMF-style saturated/regular RMF numerator.
                    let rmf_raw = if (deg_i as usize) + (degme_i as usize) > nleft {
                        let rmf1 = deg_f * (deg_f - 1.0 + 2.0 * degme_i as f64) - wf_f;
                        let new_deg = (nleft as i32) - nvi_i;
                        ws.degree[i] = new_deg;
                        let nd = new_deg as f64;
                        let rmf_new = nd * (nd - 1.0)
                            - (degme_i - nvi_i) as f64 * (degme_i - nvi_i - 1) as f64;
                        rmf_new.min(rmf1)
                    } else {
                        ws.degree[i] = deg_i + degme_i - nvi_i;
                        deg_f * (deg_f - 1.0 + 2.0 * degme_i as f64) - wf_f
                    };
                    match variant {
                        ScoreVariant::Ammf => rmf_raw / deg_f.max(1.0),
                        ScoreVariant::AmindNorm => {
                            (deg_f * (deg_f - 1.0) - wf_f).max(0.0) / (nvi_i as f64 + 1.0)
                        }
                        _ => unreachable!(),
                    }
                }
                ScoreVariant::DegSqrt => raw_deg.sqrt(),
                ScoreVariant::DegP075 => raw_deg.powf(0.75),
                ScoreVariant::DegP125 => raw_deg.powf(1.25),
                ScoreVariant::DegDivNvSqrtWf => {
                    let a = wf_f.abs().sqrt();
                    let wf_signed = if wf_f < 0.0 { -a } else { a };
                    raw_deg / (nvi_i as f64 + 1.0) + 0.5 * wf_signed
                }
                ScoreVariant::DegDivNvWfP15 => {
                    let a = wf_f.abs().powf(1.5);
                    let wf_signed = if wf_f < 0.0 { -a } else { a };
                    raw_deg / (nvi_i as f64 + 1.0) + 0.1 * wf_signed
                }
                ScoreVariant::DegPlusDegme => raw_deg + (degme_i as f64) / (nvi_i as f64 + 1.0),
                ScoreVariant::DegDivNvDegme => {
                    raw_deg / (nvi_i as f64 + 1.0) + 0.5 * (degme_i as f64) / (nvi_i as f64 + 1.0)
                }
            };

            // SqDiv/SqPure still need the AMD-style degree write-back (the
            // Ammf/AmindNorm branch already wrote ws.degree[i] above in the
            // saturated/regular branch).
            if DEGREE_FAMILY.contains(&variant) {
                ws.degree[i] = raw_deg as i32;
            }

            let qscore: i32 = if score_f < dummy_f {
                score_f.round() as i32
            } else if score_f / n_f < dummy_f {
                (score_f / n_f).round() as i32
            } else {
                SCORE_DUMMY_I32
            };
            ws.wf[i] = qscore.max(1) as i64;

            let d = score_bucket_of(ws.wf[i], n);
            let inext = ws.head[d];
            if inext != NONE {
                ws.last[inext as usize] = i as i32;
            }
            ws.next[i] = inext;
            ws.last[i] = NONE;
            ws.head[d] = i as i32;
            if d < ws.mindeg {
                ws.mindeg = d;
            }
            ws.iw[p_write] = i as i32;
            p_write += 1;
        }
    }

    ws.nv[me] = nvpiv;
    ws.len[me] = (p_write as i32) - pme1 as i32;
    if ws.len[me] == 0 {
        ws.pe[me] = NONE;
        ws.w[me] = 0;
    }
    if elenme != 0 {
        ws.pfree = p_write;
    }

    let f = nvpiv as f64;
    let r = degme_i32 as f64 + ws.ndense as f64;
    let lnzme = f * r + (f - 1.0) * f / 2.0;
    let s = f * r * r + r * (f - 1.0) * f + (f - 1.0) * f * (2.0 * f - 1.0) / 6.0;
    StepFlops {
        ndiv: lnzme,
        nms_lu: s,
        nms_ldl: (s + lnzme) / 2.0,
    }
}

/// Drive the elimination loop for `variant` to completion. Reuses
/// `select_pivot_amf` / `create_element_amf` verbatim from the vendor crate.
fn run_elimination_variant(
    ws: &mut Workspace,
    aggressive: bool,
    variant: ScoreVariant,
) -> Result<StepFlops, OrderingError> {
    let mut flops = StepFlops::default();
    while ws.nel < ws.n {
        let me = match select_pivot_amf(ws) {
            Some(m) => m,
            None => break,
        };
        let elenme = ws.elen[me];
        let (pme1, pme2, nvpiv, degme) = create_element_amf(ws, me)?;
        let step = finalize_step_variant(
            ws, me, pme1, pme2, nvpiv, degme, elenme, aggressive, variant,
        );
        // `StepFlops::accumulate` is private to the vendor crate; its fields
        // are public, so add manually (a trivial field-wise +=).
        flops.ndiv += step.ndiv;
        flops.nms_lu += step.nms_lu;
        flops.nms_ldl += step.nms_ldl;
    }
    let f = ws.ndense as f64;
    let lnzme = (f - 1.0) * f / 2.0;
    let s = (f - 1.0) * f * (2.0 * f - 1.0) / 6.0;
    flops.ndiv += lnzme;
    flops.nms_lu += s;
    flops.nms_ldl += (s + lnzme) / 2.0;
    Ok(flops)
}

/// Public entry point: run `variant` on `core` and return the permutation
/// (`perm[k]` = original index eliminated k-th, matching the `order()`
/// contract). Deterministic: every input (`core`, `dense_alpha`,
/// `aggressive`, `variant`) is a pure function of the pattern and fixed
/// constants, and the elimination loop below has no randomness.
pub fn order_variant(
    core: &CscPattern<'_>,
    dense_alpha: f64,
    aggressive: bool,
    variant: ScoreVariant,
) -> Result<Vec<i32>, OrderingError> {
    let opts = WorkspaceOptions { dense_alpha };
    let n_buckets = 2 * core.n + 2;
    let mut ws = Workspace::new_with_n_buckets(core, &opts, n_buckets)?;
    run_elimination_variant(&mut ws, aggressive, variant)?;
    Ok(finalize_permutation(&mut ws))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bijection(perm: &[i32], n: usize) {
        assert_eq!(perm.len(), n, "permutation length");
        let mut seen = vec![false; n];
        for &v in perm {
            let v = v as usize;
            assert!(v < n && !seen[v], "not a bijection of 0..{n}");
            seen[v] = true;
        }
    }

    fn sample_core(n: usize, edges: &[(usize, usize)]) -> (Vec<i32>, Vec<i32>) {
        let pat = crate::Pattern::from_edges(n, edges);
        (
            pat.col_ptr.iter().map(|&x| x as i32).collect(),
            pat.row_idx.iter().map(|&x| x as i32).collect(),
        )
    }

    #[test]
    fn every_variant_is_a_valid_bijection() {
        let n = 80;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 9 {
            edges.push((v, v + 9));
        }
        let (cp, ri) = sample_core(n, &edges);
        let core = CscPattern::new(n, &cp, &ri).unwrap();
        for &variant in &[
            ScoreVariant::SqDiv,
            ScoreVariant::SqPure,
            ScoreVariant::Ammf,
            ScoreVariant::AmindNorm,
            ScoreVariant::DegSqrt,
            ScoreVariant::DegP075,
            ScoreVariant::DegP125,
            ScoreVariant::DegDivNvSqrtWf,
            ScoreVariant::DegDivNvWfP15,
            ScoreVariant::DegPlusDegme,
            ScoreVariant::DegDivNvDegme,
        ] {
            let perm = order_variant(&core, 10.0, true, variant)
                .unwrap_or_else(|e| panic!("{variant:?} failed: {e:?}"));
            assert_bijection(&perm, n);
        }
    }

    #[test]
    fn every_variant_is_deterministic() {
        let n = 150;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 11 {
            edges.push((v, v + 11));
        }
        let (cp, ri) = sample_core(n, &edges);
        let core = CscPattern::new(n, &cp, &ri).unwrap();
        for &variant in &[
            ScoreVariant::SqDiv,
            ScoreVariant::SqPure,
            ScoreVariant::Ammf,
            ScoreVariant::AmindNorm,
            ScoreVariant::DegSqrt,
            ScoreVariant::DegP075,
            ScoreVariant::DegP125,
            ScoreVariant::DegDivNvSqrtWf,
            ScoreVariant::DegDivNvWfP15,
            ScoreVariant::DegPlusDegme,
            ScoreVariant::DegDivNvDegme,
        ] {
            let a = order_variant(&core, 10.0, true, variant).unwrap();
            let b = order_variant(&core, 10.0, true, variant).unwrap();
            assert_eq!(a, b, "{variant:?} not deterministic");
        }
    }
}
