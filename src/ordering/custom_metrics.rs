//! CUSTOM QUOTIENT-GRAPH PIVOT-SELECTION METRICS (win **E**, see
//! `memory/index.md`). `feral_ordering_core::quotient_graph` exposes its
//! `select_pivot_amf` / `create_element_amf` Pass-1/Pass-2 graph bookkeeping
//! (supervariable merges, external-degree accumulation, mass elimination,
//! hash-bucket insertion) as PUBLIC pure-Rust items — none of it is
//! AMD/AMF-specific, it buckets by an arbitrary `i64` score cached in
//! `ws.wf`, quantized via `score_bucket_of`. AMF just happens to be the first
//! thing that writes into that field. This module forks ONLY the ~55-line
//! "compute this candidate's re-insertion score" tail of `finalize_step_amf`
//! (`impl Reinsert for VariantScorer` below) to try four alternative
//! pivot-selection formulas, and drives them through the SAME elimination
//! loop AMF itself runs (`select_pivot_amf` / `create_element_amf`, reused
//! verbatim, unmodified, from the vendor crate; the Pass-1/Pass-2 step body
//! lives in `qg_kernel.rs`, shared with `metric_sweep.rs`).
//! `finalize_permutation` is likewise reused unmodified (it only reads
//! `pe`/`nv`/`elen`, which every variant here still writes in the same
//! places AMF does).
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

use feral_ordering_core::quotient_graph::{finalize_permutation, Workspace, WorkspaceOptions};
use feral_ordering_core::{CscPattern, OrderingError};

use super::qg_kernel::{run_elimination, PowTable, Reinsert};

/// Which score formula the re-insertion computes. See the module doc for
/// what each one estimates and why it was kept.
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

/// `SqDiv`/`SqPure`/the seven win-F variants write `raw_deg` (AMD's own
/// loose-degree estimate) back into `ws.degree[i]` after scoring; the
/// AMF-style variants (`Ammf`/`AmindNorm`) write the saturated/regular RMF
/// degree instead — `ws.degree` must always hold the true AMD-style
/// loose-degree estimate for later steps' monotone-cap logic, regardless of
/// which formula ranks candidates for selection. `VariantScorer::score`
/// returns that write-back value alongside the score; this list documents
/// the partition and is asserted against it in the tests.
#[cfg(test)]
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

/// Re-insertion scorer for one `ScoreVariant`. `pow` memoises
/// `raw_deg.powf(0.75)` / `raw_deg.powf(1.25)` by exact integer `raw_deg`
/// (the only two variants that call `powf` on an integer-valued argument);
/// every other formula is evaluated inline exactly as before.
struct VariantScorer {
    variant: ScoreVariant,
    pow: Option<PowTable>,
}

impl VariantScorer {
    fn new(variant: ScoreVariant, n: usize) -> Self {
        let pow = match variant {
            ScoreVariant::DegP075 => Some(PowTable::new(0.75, n)),
            ScoreVariant::DegP125 => Some(PowTable::new(1.25, n)),
            _ => None,
        };
        Self { variant, pow }
    }
}

impl Reinsert for VariantScorer {
    #[inline(always)]
    fn bucket_div(&self) -> usize {
        8
    }

    /// The fork of `algo::finalize_step_amf`'s score block: returns
    /// `(score_f, degree[i] write-back)`.
    #[inline(always)]
    fn score(&mut self, deg_i: i32, degme_i: i32, nvi_i: i32, wf: i64, nleft: usize) -> (f64, i32) {
        let deg_f = deg_i as f64;
        let wf_f = wf as f64;

        // `raw_deg`: AMD's own loose-degree estimate, used by SqDiv/
        // SqPure. Computed the same way regardless of saturation,
        // mirroring AMD's own cap (not AMF's saturated-RMF branch below,
        // which targets the fill formula specifically).
        let raw_deg = (deg_f + degme_i as f64 - nvi_i as f64)
            .max(0.0)
            .min((nleft - nvi_i as usize) as f64);

        // The AMD-style degree write-back every DEGREE_FAMILY variant does;
        // the Ammf/AmindNorm branch overrides it with its saturated/regular
        // RMF degree.
        let mut new_degree = raw_deg as i32;

        let score_f: f64 = match self.variant {
            ScoreVariant::SqDiv => (raw_deg * raw_deg) / (nvi_i as f64 + 1.0),
            ScoreVariant::SqPure => raw_deg * raw_deg,
            ScoreVariant::Ammf | ScoreVariant::AmindNorm => {
                // Shared AMF-style saturated/regular RMF numerator.
                let rmf_raw = if (deg_i as usize) + (degme_i as usize) > nleft {
                    let rmf1 = deg_f * (deg_f - 1.0 + 2.0 * degme_i as f64) - wf_f;
                    let new_deg = (nleft as i32) - nvi_i;
                    new_degree = new_deg;
                    let nd = new_deg as f64;
                    let rmf_new = nd * (nd - 1.0)
                        - (degme_i - nvi_i) as f64 * (degme_i - nvi_i - 1) as f64;
                    rmf_new.min(rmf1)
                } else {
                    new_degree = deg_i + degme_i - nvi_i;
                    deg_f * (deg_f - 1.0 + 2.0 * degme_i as f64) - wf_f
                };
                match self.variant {
                    ScoreVariant::Ammf => rmf_raw / deg_f.max(1.0),
                    ScoreVariant::AmindNorm => {
                        (deg_f * (deg_f - 1.0) - wf_f).max(0.0) / (nvi_i as f64 + 1.0)
                    }
                    _ => unreachable!(),
                }
            }
            ScoreVariant::DegSqrt => raw_deg.sqrt(),
            ScoreVariant::DegP075 | ScoreVariant::DegP125 => match self.pow.as_mut() {
                Some(t) => t.powf(raw_deg),
                None => unreachable!(),
            },
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
        (score_f, new_degree)
    }
}

/// Public entry point: run `variant` on `core` and return the permutation
/// (`perm[k]` = original index eliminated k-th, matching the `order()`
/// contract). Deterministic: every input (`core`, `dense_alpha`,
/// `aggressive`, `variant`) is a pure function of the pattern and fixed
/// constants, and the elimination loop has no randomness.
pub fn order_variant(
    core: &CscPattern<'_>,
    dense_alpha: f64,
    aggressive: bool,
    variant: ScoreVariant,
) -> Result<Vec<i32>, OrderingError> {
    let opts = WorkspaceOptions { dense_alpha };
    let n_buckets = 2 * core.n + 2;
    let mut ws = Workspace::new_with_n_buckets(core, &opts, n_buckets)?;
    let mut scorer = VariantScorer::new(variant, core.n);
    run_elimination(&mut ws, aggressive, &mut scorer)?;
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
    fn degree_family_partition_matches_scorer() {
        // Every variant either does the raw_deg write-back (DEGREE_FAMILY)
        // or is one of the two AMF-style variants; nothing else exists.
        for &v in &[
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
            let amf_style = matches!(v, ScoreVariant::Ammf | ScoreVariant::AmindNorm);
            assert_eq!(DEGREE_FAMILY.contains(&v), !amf_style, "{v:?}");
        }
        // The AMF-style branch overrides the write-back; the degree family
        // keeps raw_deg. Regular (unsaturated) case: deg 3, degme 2, nvi 1,
        // nleft 10 -> raw_deg 4.
        let mut s = VariantScorer::new(ScoreVariant::Ammf, 16);
        assert_eq!(s.score(3, 2, 1, 0, 10).1, 3 + 2 - 1);
        let mut s = VariantScorer::new(ScoreVariant::SqDiv, 16);
        assert_eq!(s.score(3, 2, 1, 0, 10).1, 4);
        // Saturated case: deg + degme > nleft -> nleft - nvi.
        let mut s = VariantScorer::new(ScoreVariant::AmindNorm, 16);
        assert_eq!(s.score(6, 5, 1, 0, 10).1, 10 - 1);
    }

    #[test]
    fn pow_table_matches_direct_powf() {
        for &(v, p) in &[(ScoreVariant::DegP075, 0.75f64), (ScoreVariant::DegP125, 1.25f64)] {
            let mut s = VariantScorer::new(v, 64);
            for pass in 0..2 {
                for deg in 0..40i32 {
                    // degme 0, nvi 1, nleft 64 -> raw_deg = deg - 1 (clamped at 0).
                    let (got, _) = s.score(deg, 0, 1, 0, 64);
                    let raw = ((deg - 1).max(0)) as f64;
                    assert_eq!(got.to_bits(), raw.powf(p).to_bits(), "{v:?} deg {deg} pass {pass}");
                }
            }
        }
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
