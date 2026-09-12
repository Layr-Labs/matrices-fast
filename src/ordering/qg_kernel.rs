//! Shared quotient-graph elimination step for the custom pivot-score
//! families (`custom_metrics::order_variant`, `metric_sweep::order_generic`).
//!
//! This is the SAME algorithm both callers used to carry as a private copy
//! of the vendor's `finalize_step_amf` (Pass-1 external-degree seeding,
//! Pass-2 degree / fill-surface accumulation with aggressive absorption,
//! mass elimination and hash-bucket insertion, supervariable detection, and
//! the score-quantize-bucket re-insertion), with two purely mechanical
//! changes that do not alter a single store or a single floating-point
//! operation:
//!
//! 1. The workspace arrays are split-borrowed into local slices once per
//!    step instead of being indexed through `ws.field[..]` on every access.
//!    Each `Vec` header (pointer, length) then lives in a register for the
//!    whole step; through `&mut Workspace` the compiler had to reload it
//!    after every store because it cannot prove a heap store does not alias
//!    the struct. Same indices, same bounds checks, same values.
//! 2. The per-candidate score formula is a trait call ([`Reinsert::score`])
//!    that returns the score AND the value to write into `degree[i]`, so
//!    one loop serves every family. Each caller's `score` body keeps its
//!    original expression text and evaluation order verbatim (see the two
//!    `impl Reinsert` blocks); only the `powf` calls whose argument is an
//!    integer-valued `f64` are memoised by exact integer key, which returns
//!    the identical bits `powf` itself returned the first time.
//!
//! Output identity was verified against the tip on all 300 dev rows
//! (permutation digest + exact flop count), and the per-pass digests of
//! every `(variant, alpha)` pair on the ten slowest rows are unchanged.

use feral_ordering_core::quotient_graph::{
    clear_flag, create_element_amf, flip, select_pivot_amf, StepFlops, Workspace, NONE,
};
use feral_ordering_core::OrderingError;

/// Saturation cap mirroring `algo::AMF_DUMMY_I32` (private to the vendor
/// crate and not re-exported, so this is an independent copy of the same
/// quantization convention MUMPS/AMF uses: `i32::MAX - 1`).
pub(crate) const SCORE_DUMMY_I32: i32 = i32::MAX - 1;

/// Quantize an `i64` score into a bucket index in `[0, 2n+1]`, generalized
/// over the coarse stride divisor. `bucket_div = 8` is the vendor AMF
/// convention (`algo::amf_bucket_of`, private there).
#[inline(always)]
pub(crate) fn score_bucket_of(score: i64, n: usize, bucket_div: usize) -> usize {
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

/// The per-family re-insertion score. `score` receives exactly the values
/// the original inline block read (`degree[i]` before write-back, the
/// pivot's external degree, `nv[i]`, `wf[i]`, `n - nel`) and returns the
/// floating-point score plus the value the block wrote into `degree[i]`.
pub(crate) trait Reinsert {
    /// Coarse-bucket stride divisor for [`score_bucket_of`].
    fn bucket_div(&self) -> usize;
    fn score(&mut self, deg_i: i32, degme: i32, nvi: i32, wf: i64, nleft: usize) -> (f64, i32);
}

/// Lazily memoised `x.powf(p)` for integer-valued `x >= 0` (`x <= cap`).
/// Returns the bits `powf` produced on the first evaluation of that `x`;
/// a `NaN` slot means "not yet evaluated" (`powf` of a non-negative finite
/// input never yields `NaN`).
pub(crate) struct PowTable {
    p: f64,
    tab: Vec<f64>,
}

impl PowTable {
    pub(crate) fn new(p: f64, cap: usize) -> Self {
        Self {
            p,
            tab: vec![f64::NAN; cap + 1],
        }
    }

    /// `x` must be integer-valued and within `0..=cap`; anything else falls
    /// back to a direct `powf` call (still the identical value).
    #[inline(always)]
    pub(crate) fn powf(&mut self, x: f64) -> f64 {
        let k = x as usize;
        if k as f64 == x && k < self.tab.len() {
            let v = self.tab[k];
            if !v.is_nan() {
                return v;
            }
            let v = x.powf(self.p);
            self.tab[k] = v;
            v
        } else {
            x.powf(self.p)
        }
    }
}

/// Unchecked read. Callers establish `i < a.len()` from the quotient-graph
/// list invariants (see the `SAFETY` notes at the two call sites); the
/// `debug_assert!` keeps a debug build fail-fast.
#[inline(always)]
unsafe fn at(a: &[i32], i: usize) -> i32 {
    debug_assert!(i < a.len());
    *a.get_unchecked(i)
}

/// Unchecked write; same contract as [`at`].
#[inline(always)]
unsafe fn set(a: &mut [i32], i: usize, v: i32) {
    debug_assert!(i < a.len());
    *a.get_unchecked_mut(i) = v;
}

/// Unchecked `i64` read; same contract as [`at`].
#[inline(always)]
unsafe fn at64(a: &[i64], i: usize) -> i64 {
    debug_assert!(i < a.len());
    *a.get_unchecked(i)
}

/// Unchecked `i64` write; same contract as [`at`].
#[inline(always)]
unsafe fn set64(a: &mut [i64], i: usize, v: i64) {
    debug_assert!(i < a.len());
    *a.get_unchecked_mut(i) = v;
}

/// One elimination step after `create_element_amf`: the fork of
/// `algo::finalize_step_amf` whose only family-specific block is the
/// re-insertion score (delegated to `scorer`).
#[allow(clippy::too_many_arguments)]
#[inline(never)]
fn finalize_step<R: Reinsert>(
    ws: &mut Workspace,
    me: usize,
    pme1: usize,
    pme2_excl: usize,
    nvpiv: i32,
    degme: usize,
    elenme: i32,
    aggressive: bool,
    scorer: &mut R,
) -> StepFlops {
    let n = ws.n;
    let wbig = ws.wbig;
    let ndense = ws.ndense;
    let mut wflg = ws.wflg;
    let mut nel = ws.nel;
    let mut lemax = ws.lemax;
    let mut mindeg = ws.mindeg;
    let mut n_mass_elim = ws.n_mass_elim;
    let mut n_supervar_merge = ws.n_supervar_merge;
    let mut degme = degme;
    let mut nvpiv = nvpiv;

    // Split borrows: one register-resident (ptr, len) per array for the
    // whole step. Every index below is the same index the checked
    // `ws.field[..]` form used, so bounds checking is unchanged.
    let iw: &mut [i32] = &mut ws.iw;
    let pe: &mut [i32] = &mut ws.pe;
    let len: &mut [i32] = &mut ws.len;
    let nv: &mut [i32] = &mut ws.nv;
    let elen: &mut [i32] = &mut ws.elen;
    let degree: &mut [i32] = &mut ws.degree;
    let w: &mut [i32] = &mut ws.w;
    let head: &mut [i32] = &mut ws.head;
    let next: &mut [i32] = &mut ws.next;
    let last: &mut [i32] = &mut ws.last;
    let wf: &mut [i64] = &mut ws.wf;

    // SAFETY (Pass 1 and Pass 2): every index below is one the checked
    // `ws.field[..]` form of this same loop used, and the vendor algorithm
    // guarantees each is in range:
    //   - `iw[pme]` for `pme in pme1..pme2_excl` (the new element's member
    //     list, written by `create_element_amf` from live vertex ids), and
    //     every entry of a member's element list `iw[pe[i]..pe[i]+elen[i]]`
    //     and variable list `iw[pe[i]+elen[i]..pe[i]+len[i]]`, is a vertex
    //     id in `0..n` — the lists are seeded from a validated `CscPattern`
    //     (`row_idx < n`) and only ever rewritten with vertex ids by this
    //     step and by `create_element_amf` (whose compaction markers are
    //     un-flipped before it returns). `w`, `wf`, `degree`, `nv`, `pe`,
    //     `len`, `elen`, `next`, `last` all have length `n`.
    //   - list positions `pe[i] + k` for `k < len[i]` are `< pfree <= iwlen
    //     = iw.len()` (the free-space invariant `create_element_amf` keeps),
    //     and the compaction writes `iw[pn]` with `p1 <= pn <= p`.
    //   - `hash % n < n <= head.len()` (`head` has `2n + 2` slots).
    // A debug build re-checks every index via `debug_assert!`.
    unsafe {
        // Pass 1: identical to finalize_step_amf.
        for pme in pme1..pme2_excl {
            let i = at(iw, pme) as usize;
            let eln = at(elen, i);
            if eln > 0 {
                let nvi = -at(nv, i);
                let wnvi = wflg - nvi;
                let pi = at(pe, i) as usize;
                for k in 0..eln as usize {
                    let e = at(iw, pi + k) as usize;
                    let mut we = at(w, e);
                    if we >= wflg {
                        we -= nvi;
                    } else if we != 0 {
                        we = at(degree, e) + wnvi;
                        set64(wf, e, 0);
                    }
                    set(w, e, we);
                }
            }
        }

        // Pass 2: identical structure to finalize_step_amf (deg/wf3/wf4
        // accumulation, aggressive absorption, mass elimination, hash-bucket
        // insertion).
        for pme in pme1..pme2_excl {
            let i = at(iw, pme) as usize;
            let p1 = at(pe, i) as usize;
            let p2 = p1 + at(elen, i) as usize;
            let mut pn = p1;
            let mut deg: usize = 0;
            let mut hash: usize = 0;
            let mut wf3: i64 = 0;
            let mut wf4: i64 = 0;
            let nvi = -at(nv, i);

            if aggressive {
                for p in p1..p2 {
                    let e = at(iw, p) as usize;
                    let we = at(w, e);
                    if we != 0 {
                        let dext = we - wflg;
                        if dext > 0 {
                            if at64(wf, e) == 0 {
                                let d = dext as i64;
                                let de = at(degree, e) as i64;
                                set64(wf, e, d * (2 * de - d - 1));
                            }
                            wf4 += at64(wf, e);
                            deg += dext as usize;
                            set(iw, pn, e as i32);
                            pn += 1;
                            hash = hash.wrapping_add(e);
                        } else {
                            set(pe, e, flip(me as i32));
                            set(w, e, 0);
                        }
                    }
                }
            } else {
                for p in p1..p2 {
                    let e = at(iw, p) as usize;
                    let we = at(w, e);
                    if we != 0 {
                        let dext = (we - wflg) as usize;
                        if at64(wf, e) == 0 {
                            let d = dext as i64;
                            let de = at(degree, e) as i64;
                            set64(wf, e, d * (2 * de - d - 1));
                        }
                        wf4 += at64(wf, e);
                        deg += dext;
                        set(iw, pn, e as i32);
                        pn += 1;
                        hash = hash.wrapping_add(e);
                    }
                }
            }

            set(elen, i, (pn - p1 + 1) as i32);
            let p3 = pn;
            let p4 = p1 + at(len, i) as usize;
            for p in p2..p4 {
                let j = at(iw, p) as usize;
                let nvj = at(nv, j);
                if nvj > 0 {
                    deg += nvj as usize;
                    wf3 += nvj as i64;
                    set(iw, pn, j as i32);
                    pn += 1;
                    hash = hash.wrapping_add(j);
                }
            }

            if at(elen, i) == 1 && p3 == pn {
                set(pe, i, flip(me as i32));
                let nvi_sv = -at(nv, i);
                degme -= nvi_sv as usize;
                nvpiv += nvi_sv;
                nel += nvi_sv as usize;
                set(nv, i, 0);
                set(elen, i, NONE);
                n_mass_elim += 1;
            } else {
                if at(degree, i) < deg as i32 {
                    wf3 = 0;
                    wf4 = 0;
                } else {
                    set(degree, i, deg as i32);
                }
                // wf[i] used downstream as the raw fill-surface accumulator
                // (AMF's B3 term); every family re-derives its own score from
                // `degree[i]` (= deg) and this `wf[i]`.
                set64(wf, i, wf4 + 2 * nvi as i64 * wf3);

                if p1 != pn {
                    set(iw, pn, at(iw, p3));
                }
                if p3 != p1 {
                    set(iw, p3, at(iw, p1));
                }
                set(iw, p1, me as i32);
                set(len, i, (pn - p1 + 1) as i32);

                let h = hash % n;
                let j = at(head, h);
                if j <= NONE {
                    set(next, i, flip(j));
                    set(head, h, flip(i as i32));
                } else {
                    set(next, i, at(last, j as usize));
                    set(last, j as usize, i as i32);
                }
                set(last, i, h as i32);
            }
        }
    }

    let degme_i32 = degme as i32;
    degree[me] = degme_i32;
    if degme_i32 > lemax {
        lemax = degme_i32;
    }
    wflg += lemax;
    wflg = clear_flag(wflg, wbig, w);

    // Supervariable detection: identical to finalize_step_amf (max-merge of
    // wf, which every family treats as "the fill-surface accumulator").
    for pme in pme1..pme2_excl {
        let i_anchor = iw[pme] as usize;
        if nv[i_anchor] >= 0 {
            continue;
        }
        let h = last[i_anchor] as usize;
        let j_head = head[h];
        let mut i: i32 = if j_head == NONE {
            NONE
        } else if j_head < NONE {
            head[h] = NONE;
            flip(j_head)
        } else {
            let chain_start = last[j_head as usize];
            last[j_head as usize] = NONE;
            chain_start
        };
        while i != NONE && next[i as usize] != NONE {
            let i_u = i as usize;
            let ln = len[i_u];
            let eln = elen[i_u];
            let pi = pe[i_u];
            for p in (pi + 1) as usize..(pi + ln) as usize {
                w[iw[p] as usize] = wflg;
            }
            let mut jlast = i_u;
            let mut jp = next[i_u];
            while jp != NONE {
                let jj = jp as usize;
                let mut ok = len[jj] == ln && elen[jj] == eln;
                if ok {
                    let pj = pe[jj];
                    for p in (pj + 1) as usize..(pj + ln) as usize {
                        if w[iw[p] as usize] != wflg {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok {
                    pe[jj] = flip(i);
                    let wf_j = wf[jj];
                    if wf_j > wf[i_u] {
                        wf[i_u] = wf_j;
                    }
                    nv[i_u] += nv[jj];
                    nv[jj] = 0;
                    elen[jj] = NONE;
                    jp = next[jj];
                    next[jlast] = jp;
                    n_supervar_merge += 1;
                } else {
                    jlast = jj;
                    jp = next[jj];
                }
            }
            wflg += 1;
            i = next[i_u];
        }
    }

    // Re-insertion: score (family-specific, via `scorer`), quantize, bucket.
    let dummy_f = SCORE_DUMMY_I32 as f64;
    let n_f = if n == 0 { 1.0 } else { n as f64 };
    let bucket_div = scorer.bucket_div();
    let mut p_write = pme1;
    let nleft = n - nel;
    for pme in pme1..pme2_excl {
        let i = iw[pme] as usize;
        let nvi = -nv[i];
        if nvi > 0 {
            nv[i] = nvi;
            let (score_f, new_degree) = scorer.score(degree[i], degme_i32, nvi, wf[i], nleft);
            degree[i] = new_degree;

            let qscore: i32 = if score_f < dummy_f {
                score_f.round() as i32
            } else if score_f / n_f < dummy_f {
                (score_f / n_f).round() as i32
            } else {
                SCORE_DUMMY_I32
            };
            wf[i] = qscore.max(1) as i64;

            let d = score_bucket_of(wf[i], n, bucket_div);
            let inext = head[d];
            if inext != NONE {
                last[inext as usize] = i as i32;
            }
            next[i] = inext;
            last[i] = NONE;
            head[d] = i as i32;
            if d < mindeg {
                mindeg = d;
            }
            iw[p_write] = i as i32;
            p_write += 1;
        }
    }

    nv[me] = nvpiv;
    len[me] = (p_write as i32) - pme1 as i32;
    if len[me] == 0 {
        pe[me] = NONE;
        w[me] = 0;
    }

    ws.wflg = wflg;
    ws.nel = nel;
    ws.lemax = lemax;
    ws.mindeg = mindeg;
    ws.n_mass_elim = n_mass_elim;
    ws.n_supervar_merge = n_supervar_merge;
    if elenme != 0 {
        ws.pfree = p_write;
    }

    let f = nvpiv as f64;
    let r = degme_i32 as f64 + ndense as f64;
    let lnzme = f * r + (f - 1.0) * f / 2.0;
    let s = f * r * r + r * (f - 1.0) * f + (f - 1.0) * f * (2.0 * f - 1.0) / 6.0;
    StepFlops {
        ndiv: lnzme,
        nms_lu: s,
        nms_ldl: (s + lnzme) / 2.0,
    }
}

/// Drive the elimination loop to completion. Reuses `select_pivot_amf` /
/// `create_element_amf` verbatim from the vendor crate.
pub(crate) fn run_elimination<R: Reinsert>(
    ws: &mut Workspace,
    aggressive: bool,
    scorer: &mut R,
) -> Result<StepFlops, OrderingError> {
    let mut flops = StepFlops::default();
    while ws.nel < ws.n {
        let me = match select_pivot_amf(ws) {
            Some(m) => m,
            None => break,
        };
        let elenme = ws.elen[me];
        let (pme1, pme2, nvpiv, degme) = create_element_amf(ws, me)?;
        let step = finalize_step(ws, me, pme1, pme2, nvpiv, degme, elenme, aggressive, scorer);
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
