//! SHOT C: a GENERIC quotient-graph pivot-score family (`MetricSpec` +
//! `order_generic`), ported from the `sbx-metrics2` research sweep (originally
//! test-cfg-only) so a wide grid of diversity candidates can be added without
//! hand-writing a new `ScoreVariant` per formula. Only `order_generic` and its
//! dependencies are kept here — the sweep-grid generation, corpus-probing, and
//! greedy-selection research code (which depended on test-only corpus loaders)
//! were dropped; `mod.rs`'s `EXTRA_METRICS` constant hard-codes the 15
//! specific specs this shot ships, chosen offline by that research sweep.
//!
//! Forked from `custom_metrics.rs`'s `finalize_step_variant` — Pass-1/Pass-2
//! graph bookkeeping is copied UNCHANGED (same trust argument: reuses
//! `select_pivot_amf`/`create_element_amf` verbatim from the vendor crate,
//! and the raw-degree write-back path is the same one `custom_metrics::SqDiv`
//! /`SqPure` already ship). Only the re-insertion score formula is
//! parameterized by [`MetricSpec`] instead of hard-coded per variant, and the
//! bucket stride is also parameterizable (default matches the shipped `/8`
//! convention exactly; only a small, separately-reported sub-sweep varies it).
//!
//! This module exists to answer the open question in
//! `memory/open-questions.md`: "Does candidate diversity saturate?" — by
//! measuring each swept formula's MARGINAL contribution against the real
//! `order()` portfolio (not against AMD alone, which is what the prior
//! 25-variant sweep measured and which this file's sibling experiment
//! [0010] flagged as the wrong baseline).
//!
//! Every variant here uses the SAME "world" as `SqDiv`/`SqPure`: it always
//! computes AMD's own loose-degree estimate (`raw_deg`) and writes it back to
//! `ws.degree[i]` unconditionally, regardless of which formula ranks
//! candidates. That sidesteps the AMF-style saturated-RMF branch entirely —
//! the branch implicated in the `AmindNorm` cost cliff (subtraction of two
//! similarly-scaled terms produces a heavy-tailed score distribution that
//! crowds one coarse bucket). It does NOT prove every formula here is
//! cliff-safe (a `deg^3`-shaped score still has wide dynamic range on
//! high-degree matrices), so every candidate is still gated `nnz < 130_000`
//! by default, matching the existing custom-metrics envelope, unless
//! specifically noted otherwise after an isolated check.

use feral_ordering_core::quotient_graph::{
    clear_flag, create_element_amf, finalize_permutation, flip, select_pivot_amf, StepFlops,
    Workspace, WorkspaceOptions, NONE,
};
use feral_ordering_core::{CscPattern, OrderingError};
use super::pivot_powers::{FloatPower, IntegerPower};

struct ScorePowers {
    degree: IntegerPower,
    supernode: IntegerPower,
    fill: FloatPower,
}

/// One point in the sweep grid. All fields are pure numbers so the grid can
/// be generated programmatically; `name` is only for reporting.
#[derive(Clone, Copy)]
pub struct MetricSpec {
    pub name: &'static str,
    /// Power applied to `raw_deg` (AMD's own loose-degree estimate).
    pub deg_pow: f64,
    /// Power applied to the `(nvi + 1)` denominator. `0.0` disables the
    /// denominator entirely (pure degree term, no supervariable weighting).
    pub nv_pow: f64,
    /// Coefficient on the `wf` (raw fill-surface accumulator, AMF's B3/B4
    /// term) mixture. `0.0` = pure-degree metric, no fill information at all.
    pub wf_weight: f64,
    /// Signed power applied to `wf` before weighting (`wf` can be negative;
    /// a signed power avoids NaN: `sign(wf) * |wf|^wf_pow`).
    pub wf_pow: f64,
    /// Coefficient on an extra `degme / (nvi+1)` cross term (the current
    /// pivot's external degree) — an asymmetric "how big is the front being
    /// built" signal absent from AMD's own formula.
    pub degme_weight: f64,
    /// If true, invert the ranking (`1 / (1 + score)`) — a deliberately
    /// perverse "prefer high-degree, worst-first" pivot rule.
    pub perverse: bool,
    /// Bucket coarse-region stride divisor (`pas = max(n / bucket_div, 1)`).
    /// `8` reproduces the shipped convention exactly. Only varied in the
    /// dedicated tie-break sub-sweep, never crossed with the main grid.
    pub bucket_div: usize,
}

const DEFAULT_BUCKET_DIV: usize = 8;

/// Saturation cap mirroring `algo::AMF_DUMMY_I32` — identical convention to
/// `custom_metrics::SCORE_DUMMY_I32`, copied rather than shared (private to
/// its module).
const SCORE_DUMMY_I32: i32 = i32::MAX - 1;

/// Quantize an `i64` score into a bucket index, generalized over the coarse
/// stride divisor. `bucket_div = 8` is byte-identical to
/// `custom_metrics::score_bucket_of` / the vendor AMF convention.
#[inline(always)]
fn score_bucket_of_generic(score: i64, n: usize, bucket_div: usize) -> usize {
    if score <= 0 {
        return 0;
    }
    let s = score as usize;
    if s <= n {
        return s;
    }
    let pas = (n / bucket_div.max(1)).max(1);
    let nbbuck = 2 * n;
    ((s - n) / pas + n).min(nbbuck)
}

/// Fork of `custom_metrics::finalize_step_variant`, generalized re-insertion
/// block. Pass-1/Pass-2 bookkeeping copied unchanged.
#[allow(clippy::too_many_arguments)]
fn finalize_step_generic(
    ws: &mut Workspace,
    me: usize,
    pme1: usize,
    pme2_excl: usize,
    nvpiv: i32,
    degme: usize,
    elenme: i32,
    aggressive: bool,
    spec: &MetricSpec,
    powers: &mut ScorePowers,
) -> StepFlops {
    let n = ws.n;
    let mut degme = degme;
    let mut nvpiv = nvpiv;

    // Pass 1: identical to finalize_step_amf / finalize_step_variant.
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

    // Pass 2: identical structure to finalize_step_amf / finalize_step_variant.
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

    // Supervariable detection: identical to finalize_step_amf.
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

    // ── Re-insertion: THE GENERIC SCORE BLOCK ───────────────────────────
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

            // AMD's own loose-degree estimate, computed and written back
            // unconditionally (the `SqDiv`/`SqPure` world, not the AMF
            // saturated-RMF branch — see module doc).
            let raw_deg = (deg_f + degme_i as f64 - nvi_i as f64)
                .max(0.0)
                .min((nleft - nvi_i as usize) as f64);
            ws.degree[i] = raw_deg as i32;

            let deg_term = if spec.nv_pow == 0.0 {
                powers.degree.get(raw_deg as usize)
            } else {
                powers.degree.get(raw_deg as usize) / powers.supernode.get(nvi_i as usize + 1)
            };
            let wf_term = if spec.wf_weight != 0.0 {
                let a = powers.fill.get(wf_f.abs());
                let signed = if wf_f < 0.0 { -a } else { a };
                spec.wf_weight * signed
            } else {
                0.0
            };
            let degme_term = if spec.degme_weight != 0.0 {
                spec.degme_weight * (degme_i as f64) / (nvi_i as f64 + 1.0)
            } else {
                0.0
            };
            let mut score_f = deg_term + wf_term + degme_term;
            if spec.perverse {
                score_f = 1.0 / (1.0 + score_f.max(0.0));
            }
            if !score_f.is_finite() {
                score_f = dummy_f;
            }

            let qscore: i32 = if score_f < dummy_f {
                score_f.round() as i32
            } else if score_f / n_f < dummy_f {
                (score_f / n_f).round() as i32
            } else {
                SCORE_DUMMY_I32
            };
            ws.wf[i] = qscore.max(1) as i64;

            let d = score_bucket_of_generic(ws.wf[i], n, spec.bucket_div);
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

fn run_elimination_generic(
    ws: &mut Workspace,
    aggressive: bool,
    spec: &MetricSpec,
) -> Result<StepFlops, OrderingError> {
    let mut flops = StepFlops::default();
    let mut powers = ScorePowers {
        degree: IntegerPower::new(ws.n, spec.deg_pow),
        supernode: IntegerPower::new(ws.n + 1, spec.nv_pow),
        fill: FloatPower::new(spec.wf_pow),
    };
    while ws.nel < ws.n {
        let me = match select_pivot_amf(ws) {
            Some(m) => m,
            None => break,
        };
        let elenme = ws.elen[me];
        let (pme1, pme2, nvpiv, degme) = create_element_amf(ws, me)?;
        let step = finalize_step_generic(ws, me, pme1, pme2, nvpiv, degme, elenme, aggressive, spec, &mut powers);
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

/// Public entry point: run `spec` on `core`, deterministic pure function of
/// `(core, dense_alpha, aggressive, spec)`.
pub fn order_generic(
    core: &CscPattern<'_>,
    dense_alpha: f64,
    aggressive: bool,
    spec: &MetricSpec,
) -> Result<Vec<i32>, OrderingError> {
    let opts = WorkspaceOptions { dense_alpha };
    let n_buckets = 2 * core.n + 2;
    let mut ws = Workspace::new_with_n_buckets(core, &opts, n_buckets)?;
    run_elimination_generic(&mut ws, aggressive, spec)?;
    Ok(finalize_permutation(&mut ws))
}

/// Abandonable sibling used to screen late residual-core candidates. The
/// original elimination entry point and all pivot bookkeeping stay unchanged.
#[cfg(test)]
pub(super) fn order_generic_limited(core:&CscPattern<'_>,dense_alpha:f64,
    aggressive:bool,spec:&MetricSpec,allowance:u64)->Option<Vec<i32>> {
    let setup=(5*core.n).saturating_add(2*core.row_idx.len()) as u64;
    let mut remaining=allowance.checked_sub(setup)?;
    let mut ws=Workspace::new_with_n_buckets(core,&WorkspaceOptions {dense_alpha},
        2*core.n+2).ok()?;
    let mut powers=ScorePowers {degree:IntegerPower::new(ws.n,spec.deg_pow),
        supernode:IntegerPower::new(ws.n+1,spec.nv_pow),fill:FloatPower::new(spec.wf_pow)};
    while ws.nel<ws.n {
        let Some(me)=select_pivot_amf(&mut ws) else {break};
        let mass=ws.nv[me].max(1) as u64;
        let width=(ws.degree[me].max(0) as u64).saturating_add(mass);
        let charge=width.saturating_mul(width).saturating_mul(mass)
            .max(ws.n.saturating_sub(ws.nel) as u64);
        remaining=remaining.checked_sub(charge)?;
        let elenme=ws.elen[me];
        let (p1,p2,nvpiv,degme)=create_element_amf(&mut ws,me).ok()?;
        let actual_mass=nvpiv.max(1) as u64;
        let actual_width=(degme as u64).saturating_add(actual_mass);
        let actual_charge=actual_width.saturating_mul(actual_width).saturating_mul(actual_mass);
        remaining=remaining.checked_sub(actual_charge.saturating_sub(charge))?;
        finalize_step_generic(&mut ws,me,p1,p2,nvpiv,degme,elenme,aggressive,spec,&mut powers);
    }
    Some(finalize_permutation(&mut ws))
}

/// SHOT C — the next 15 positive-marginal specs (of the 42-point research
/// grid) beyond the 7 already shipped as `custom_metrics::ScoreVariant`.
/// Ranked by greedy forward-selection against the REAL portfolio on the
/// 427-matrix reconstructed corpus (`sbx-metrics2`'s
/// `probe_diversity_marginal_sweep`, `nnz<130_000`-gated matrices only): the
/// full 30-variant selection flattens the portfolio from 0.840926 (top-7) to
/// 0.840468 and adds nothing past that point — this array is exactly picks
/// #8-#22 of that greedy order (the ones that still moved the combined
/// score). All use the same unconditional raw-degree write-back "world" as
/// the shipped `SqDiv`/`SqPure`/win-F variants (never the saturated-RMF
/// branch responsible for the `AmindNorm` cost cliff).
pub const EXTRA_METRICS: &[MetricSpec] = &[
    MetricSpec { name: "extra_deg3_div_nv", deg_pow: 3.00, nv_pow: 1.00, wf_weight: 0.00, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg2_div_nv_wf002", deg_pow: 2.00, nv_pow: 1.00, wf_weight: 0.02, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg_plus_wf", deg_pow: 1.00, nv_pow: 0.00, wf_weight: 1.00, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg_plus_wf01", deg_pow: 1.00, nv_pow: 0.00, wf_weight: 0.10, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg_p15", deg_pow: 1.50, nv_pow: 0.00, wf_weight: 0.00, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg_div_nv_p05", deg_pow: 1.00, nv_pow: 0.50, wf_weight: 0.00, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg2_div_nv_degme05", deg_pow: 2.00, nv_pow: 1.00, wf_weight: 0.00, wf_pow: 1.00, degme_weight: 0.50, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg15_div_nv", deg_pow: 1.50, nv_pow: 1.00, wf_weight: 0.00, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg_div_nv_wf05", deg_pow: 1.00, nv_pow: 1.00, wf_weight: 0.50, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg2_div_nv_wf05", deg_pow: 2.00, nv_pow: 1.00, wf_weight: 0.50, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg_mul_nv", deg_pow: 1.00, nv_pow: -1.00, wf_weight: 0.00, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg_p175", deg_pow: 1.75, nv_pow: 0.00, wf_weight: 0.00, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg_div_nv_degme2", deg_pow: 1.00, nv_pow: 1.00, wf_weight: 0.00, wf_pow: 1.00, degme_weight: 2.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg_div_nv_wf01", deg_pow: 1.00, nv_pow: 1.00, wf_weight: 0.10, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
    MetricSpec { name: "extra_deg_div_nv_wf2", deg_pow: 1.00, nv_pow: 1.00, wf_weight: 2.00, wf_pow: 1.00, degme_weight: 0.00, perverse: false, bucket_div: DEFAULT_BUCKET_DIV },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limited_generic_preserves_completed_orders_and_abandons_exhaustion() {
        let n=49;let mut random=0x71ac_684b_225du64;
        let mut fixtures=vec![(0..n-1).map(|v|(v,v+1)).collect::<Vec<_>>(),
            (1..n).map(|v|(0,v)).collect::<Vec<_>>()];
        let mut edges=Vec::new();
        for v in 0..n {
            if v%7<6 {edges.push((v,v+1));}
            if v+7<n {edges.push((v,v+7));}
            for _ in 0..2 {
                random^=random<<13;random^=random>>7;random^=random<<17;
                let u=random as usize%n;if u!=v {edges.push((u,v));}
            }
        }
        fixtures.push(edges);
        for edges in fixtures {
            let (cp,ri)=sample_core(n,&edges);let core=CscPattern::new(n,&cp,&ri).unwrap();
            let setup=(5*n+2*ri.len()) as u64;
            for spec in EXTRA_METRICS {
                for alpha in [10.0,2.5,-1.0] {
                    for aggressive in [false,true] {
                        let original=order_generic(&core,alpha,aggressive,spec).unwrap();
                        assert_eq!(Some(original.clone()),order_generic_limited(&core,alpha,
                            aggressive,spec,u64::MAX),"{}",spec.name);
                        if let Some(p)=order_generic_limited(&core,alpha,aggressive,spec,64_000_000) {
                            assert_eq!(p,original,"{}",spec.name);
                        }
                        assert!(order_generic_limited(&core,alpha,aggressive,spec,0).is_none());
                        assert!(order_generic_limited(&core,alpha,aggressive,spec,setup+1).is_none());
                    }
                }
            }
        }
    }

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
    fn every_extra_metric_is_a_valid_bijection() {
        let n = 90;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 7 {
            edges.push((v, v + 7));
        }
        let (cp, ri) = sample_core(n, &edges);
        let core = CscPattern::new(n, &cp, &ri).unwrap();
        for spec in EXTRA_METRICS {
            let perm = order_generic(&core, 10.0, true, spec)
                .unwrap_or_else(|e| panic!("{} failed: {e:?}", spec.name));
            assert_bijection(&perm, n);
        }
    }

    #[test]
    fn every_extra_metric_is_deterministic() {
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
        for spec in EXTRA_METRICS {
            let a = order_generic(&core, 10.0, true, spec).unwrap();
            let b = order_generic(&core, 10.0, true, spec).unwrap();
            assert_eq!(a, b, "{} not deterministic", spec.name);
        }
    }
}
