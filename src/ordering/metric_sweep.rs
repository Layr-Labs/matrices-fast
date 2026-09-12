//! SHOT C: a GENERIC quotient-graph pivot-score family (`MetricSpec` +
//! `order_generic`), ported from the `sbx-metrics2` research sweep (originally
//! test-cfg-only) so a wide grid of diversity candidates can be added without
//! hand-writing a new `ScoreVariant` per formula. Only `order_generic` and its
//! dependencies are kept here — the sweep-grid generation, corpus-probing, and
//! greedy-selection research code (which depended on test-only corpus loaders)
//! were dropped; `mod.rs`'s `EXTRA_METRICS` constant hard-codes the 15
//! specific specs this shot ships, chosen offline by that research sweep.
//!
//! Shares `qg_kernel.rs`'s elimination step with `custom_metrics.rs` —
//! Pass-1/Pass-2 graph bookkeeping is the vendor's UNCHANGED (same trust
//! argument: reuses `select_pivot_amf`/`create_element_amf` verbatim from the
//! vendor crate, and the raw-degree write-back path is the same one
//! `custom_metrics::SqDiv`/`SqPure` already ship). Only the re-insertion
//! score formula (`impl Reinsert for SpecScorer`) is parameterized by
//! [`MetricSpec`] instead of hard-coded per variant, and the bucket stride
//! is also parameterizable (default matches the shipped `/8` convention
//! exactly; only a small, separately-reported sub-sweep varies it).
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

use feral_ordering_core::quotient_graph::{finalize_permutation, Workspace, WorkspaceOptions};
use feral_ordering_core::{CscPattern, OrderingError};

use super::qg_kernel::{run_elimination, PowTable, Reinsert, SCORE_DUMMY_I32};

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

/// Re-insertion scorer for one [`MetricSpec`]. The two `powf` calls whose
/// argument is integer-valued (`raw_deg.powf(deg_pow)` and
/// `(nvi + 1).powf(nv_pow)`) are memoised by exact integer key; the `wf`
/// power (arbitrary magnitude) is evaluated directly as before.
struct SpecScorer {
    spec: MetricSpec,
    deg_pow: PowTable,
    nv_pow: PowTable,
}

impl SpecScorer {
    fn new(spec: &MetricSpec, n: usize) -> Self {
        Self {
            spec: *spec,
            deg_pow: PowTable::new(spec.deg_pow, n),
            nv_pow: PowTable::new(spec.nv_pow, n + 1),
        }
    }
}

impl Reinsert for SpecScorer {
    #[inline(always)]
    fn bucket_div(&self) -> usize {
        self.spec.bucket_div
    }

    /// Generalized re-insertion score block: returns `(score_f, degree[i]
    /// write-back)`; the write-back is always `raw_deg` here.
    #[inline(always)]
    fn score(&mut self, deg_i: i32, degme_i: i32, nvi_i: i32, wf: i64, nleft: usize) -> (f64, i32) {
        let spec = &self.spec;
        let dummy_f = SCORE_DUMMY_I32 as f64;
        let deg_f = deg_i as f64;
        let wf_f = wf as f64;

        let raw_deg = (deg_f + degme_i as f64 - nvi_i as f64)
            .max(0.0)
            .min((nleft - nvi_i as usize) as f64);
        let new_degree = raw_deg as i32;

        let deg_term = if spec.nv_pow == 0.0 {
            self.deg_pow.powf(raw_deg)
        } else {
            self.deg_pow.powf(raw_deg) / self.nv_pow.powf(nvi_i as f64 + 1.0)
        };
        let wf_term = if spec.wf_weight != 0.0 {
            let a = wf_f.abs().powf(spec.wf_pow);
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
        (score_f, new_degree)
    }
}

pub fn order_generic(
    core: &CscPattern<'_>,
    dense_alpha: f64,
    aggressive: bool,
    spec: &MetricSpec,
) -> Result<Vec<i32>, OrderingError> {
    let opts = WorkspaceOptions { dense_alpha };
    let n_buckets = 2 * core.n + 2;
    let mut ws = Workspace::new_with_n_buckets(core, &opts, n_buckets)?;
    let mut scorer = SpecScorer::new(spec, core.n);
    run_elimination(&mut ws, aggressive, &mut scorer)?;
    Ok(finalize_permutation(&mut ws))
}

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
    fn spec_scorer_matches_direct_formula() {
        // The memoised deg / nv powers reproduce the direct expression bit
        // for bit, on first and repeated evaluation.
        for spec in EXTRA_METRICS {
            let mut s = SpecScorer::new(spec, 64);
            for pass in 0..2 {
                for deg in 0..30i32 {
                    for nvi in 1..4i32 {
                        // degme 2 -> raw_deg = deg + 2 - nvi (clamped to [0, 64 - nvi]).
                        let (got, wb) = s.score(deg, 2, nvi, -7, 64);
                        let raw = ((deg + 2 - nvi).max(0) as f64).min((64 - nvi) as f64);
                        assert_eq!(wb, raw as i32);
                        let deg_term = if spec.nv_pow == 0.0 {
                            raw.powf(spec.deg_pow)
                        } else {
                            raw.powf(spec.deg_pow) / (nvi as f64 + 1.0).powf(spec.nv_pow)
                        };
                        let wf_f = -7f64;
                        let wf_term = if spec.wf_weight != 0.0 {
                            let a = wf_f.abs().powf(spec.wf_pow);
                            spec.wf_weight * (if wf_f < 0.0 { -a } else { a })
                        } else {
                            0.0
                        };
                        let degme_term = if spec.degme_weight != 0.0 {
                            spec.degme_weight * 2.0 / (nvi as f64 + 1.0)
                        } else {
                            0.0
                        };
                        let want = deg_term + wf_term + degme_term;
                        assert_eq!(got.to_bits(), want.to_bits(), "{} deg {deg} nvi {nvi} pass {pass}", spec.name);
                    }
                }
            }
        }
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
