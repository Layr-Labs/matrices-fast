//! Reuse identical quotient-graph generators without removing their replay slots.

use std::sync::{Arc, OnceLock};

use feral_ordering_core::OrderingError;

use super::custom_metrics::ScoreVariant;
use super::metric_sweep::MetricSpec;
use super::parallel::CandFn;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Family {
    Amf,
    Custom(ScoreVariant),
    Generic {
        coefficients: [u64; 5],
        perverse: bool,
        bucket_div: usize,
    },
}

impl Family {
    pub(crate) fn generic(spec: &MetricSpec) -> Self {
        Self::Generic {
            coefficients: [
                spec.deg_pow.to_bits(),
                spec.nv_pow.to_bits(),
                spec.wf_weight.to_bits(),
                spec.wf_pow.to_bits(),
                spec.degme_weight.to_bits(),
            ],
            perverse: spec.perverse,
            bucket_div: spec.bucket_div,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    family: Family,
    aggressive: bool,
    dense_count: usize,
    seed: Option<u64>,
}

type Memo = Arc<OnceLock<Result<Vec<i32>, OrderingError>>>;

pub(crate) struct CandidateCache<'p> {
    n: usize,
    degrees: &'p [usize],
    dense_counts: Vec<(u64, usize)>,
    entries: Vec<(Key, Memo)>,
}

impl<'p> CandidateCache<'p> {
    pub(crate) fn new(n: usize, degrees: &'p [usize]) -> Self {
        Self {
            n,
            degrees,
            dense_counts: Vec::new(),
            entries: Vec::new(),
        }
    }

    /// Alpha affects these generators only through the workspace's deferred set.
    /// Threshold sets are nested, so equal counts imply identical sets. A seed
    /// identifies the exact relabeling, not merely its degree distribution.
    pub(crate) fn wrap<'a>(
        &mut self,
        family: Family,
        alpha: f64,
        aggressive: bool,
        seed: Option<u64>,
        produce: CandFn<'a>,
    ) -> CandFn<'a> {
        let alpha_bits = alpha.to_bits();
        let dense_count = match self.dense_counts.iter().find(|(a, _)| *a == alpha_bits) {
            Some((_, count)) => *count,
            None => {
                // Includes the initializer's n-2 threshold for negative alpha:
                // universal hubs can still be deferred when alpha is negative.
                let count = super::dense_deferred_count(self.n, self.degrees, alpha);
                self.dense_counts.push((alpha_bits, count));
                count
            }
        };
        let key = Key {
            family,
            aggressive,
            dense_count,
            seed,
        };
        let memo = match self.entries.iter().find(|(k, _)| *k == key) {
            Some((_, memo)) => Arc::clone(memo),
            None => {
                let memo = Arc::new(OnceLock::new());
                self.entries.push((key, Arc::clone(&memo)));
                memo
            }
        };
        Box::new(move || memo.get_or_init(|| produce()).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::super::{parallel, ScoringPattern};
    use crate::Pattern;

    #[test]
    fn alias_cache_preserves_all_parallel_result_slots_and_errors() {
        let n = 256;
        let edges: Vec<_> = (0..n).map(|v| (v, (v + 1) % n)).collect();
        let pattern = Pattern::from_edges(n, &edges);
        let sp = ScoringPattern {
            n,
            col_ptr: pattern.col_ptr.clone(),
            row_idx: pattern.row_idx.clone(),
        };
        let degrees = vec![2; n];
        let calls: Vec<_> = (0..4).map(|_| AtomicUsize::new(0)).collect();
        let mut cache = CandidateCache::new(n, &degrees);
        let mut plain: Vec<CandFn<'_>> = Vec::new();
        let mut cached: Vec<CandFn<'_>> = Vec::new();
        for i in 0..64 {
            let kind = i % 4;
            let produce = move || match kind {
                0 => Ok((0..n as i32).collect()),
                1 => Ok((0..n as i32).rev().collect()),
                2 => Err(OrderingError::Internal("alias test")),
                _ => Ok(vec![0; n]),
            };
            plain.push(Box::new(produce));
            let count = &calls[kind];
            cached.push(cache.wrap(
                Family::Amf,
                [10.0, 5.0, 2.5, 1.0][i / 4 % 4],
                true,
                Some(kind as u64),
                Box::new(move || {
                    count.fetch_add(1, Ordering::Relaxed);
                    produce()
                }),
            ));
        }
        assert!(cached.len() * pattern.nnz() >= parallel::PAR_MIN_WORK);
        let expected = parallel::run_candidates(&plain, &sp, n, pattern.nnz(), u64::MAX, true);
        let actual = parallel::run_candidates(&cached, &sp, n, pattern.nnz(), u64::MAX, true);
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 64);
        assert!(actual[0].perm.is_some() && actual[4].perm.is_some());
        for count in &calls {
            assert_eq!(count.load(Ordering::Relaxed), 1);
        }
        // The invocation-local generator memo also survives a batch boundary.
        let replay = parallel::run_candidates(&cached, &sp, n, pattern.nnz(), u64::MAX, true);
        assert_eq!(replay, expected);
        for count in &calls {
            assert_eq!(count.load(Ordering::Relaxed), 1);
        }
    }

    #[test]
    fn alias_cache_keeps_negative_alpha_hubs_and_distinct_families() {
        let mut degrees = vec![1; 20];
        degrees[0] = 19;
        let mut cache = CandidateCache::new(20, &degrees);
        let negative = cache.wrap(Family::Amf, -1.0, true, None, Box::new(|| Ok(vec![1])));
        let tight = cache.wrap(Family::Amf, 1.0, true, None, Box::new(|| Ok(vec![2])));
        let loose = cache.wrap(Family::Amf, 10.0, true, None, Box::new(|| Ok(vec![3])));
        let metric = cache.wrap(
            Family::Custom(ScoreVariant::SqDiv),
            1.0,
            true,
            None,
            Box::new(|| Ok(vec![4])),
        );
        let relabelled = cache.wrap(Family::Amf, 1.0, true, Some(0), Box::new(|| Ok(vec![5])));
        let nonaggressive = cache.wrap(Family::Amf, 1.0, false, None, Box::new(|| Ok(vec![6])));
        assert_eq!(negative().unwrap(), vec![1]);
        assert_eq!(tight().unwrap(), vec![1]);
        assert_eq!(loose().unwrap(), vec![3]);
        assert_eq!(metric().unwrap(), vec![4]);
        assert_eq!(relabelled().unwrap(), vec![5]);
        assert_eq!(nonaggressive().unwrap(), vec![6]);
    }

    #[test]
    fn alias_cache_matches_uncached_quotient_generators() {
        let n = 40;
        let edges: Vec<_> = (1..n)
            .flat_map(|v| [(0, v), (v, if v + 1 == n { 1 } else { v + 1 })])
            .collect();
        let pattern = Pattern::from_edges(n, &edges);
        let sp = ScoringPattern {
            n,
            col_ptr: pattern.col_ptr.clone(),
            row_idx: pattern.row_idx.clone(),
        };
        let cp: Vec<i32> = pattern.col_ptr.iter().map(|&v| v as i32).collect();
        let ri: Vec<i32> = pattern.row_idx.iter().map(|&v| v as i32).collect();
        let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
        let degrees: Vec<_> = pattern.col_ptr.windows(2).map(|p| p[1] - p[0]).collect();
        let variants = [
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
        ];
        let specs = super::super::metric_sweep::EXTRA_METRICS;
        let family_count = 1 + variants.len() + specs.len();
        let calls = AtomicUsize::new(0);
        let mut cache = CandidateCache::new(n, &degrees);
        let mut plain: Vec<CandFn<'_>> = Vec::new();
        let mut cached: Vec<CandFn<'_>> = Vec::new();
        for kind in 0..family_count {
            let family = if kind == 0 {
                Family::Amf
            } else if kind <= variants.len() {
                Family::Custom(variants[kind - 1])
            } else {
                Family::generic(&specs[kind - variants.len() - 1])
            };
            for alpha in [10.0, 5.0, 2.5, 1.0, -1.0] {
                let produce = move || {
                    if kind == 0 {
                        let opts = feral_amf::AmfOptions { dense_alpha: alpha };
                        feral_amf::amf_order_opts(&core, &opts).map(|(perm, _)| perm)
                    } else if kind <= variants.len() {
                        super::super::custom_metrics::order_variant(
                            &core,
                            alpha,
                            true,
                            variants[kind - 1],
                        )
                    } else {
                        super::super::metric_sweep::order_generic(
                            &core,
                            alpha,
                            true,
                            &specs[kind - variants.len() - 1],
                        )
                    }
                };
                plain.push(Box::new(produce));
                let calls = &calls;
                cached.push(cache.wrap(
                    family,
                    alpha,
                    true,
                    None,
                    Box::new(move || {
                        calls.fetch_add(1, Ordering::Relaxed);
                        produce()
                    }),
                ));
            }
        }
        assert!(cached.len() * pattern.nnz() >= parallel::PAR_MIN_WORK);
        let expected = parallel::run_candidates(&plain, &sp, n, pattern.nnz(), u64::MAX, true);
        let actual = parallel::run_candidates(&cached, &sp, n, pattern.nnz(), u64::MAX, true);
        assert!(expected.iter().all(|out| out.perm.is_some()));
        assert_eq!(actual, expected);
        assert_eq!(calls.load(Ordering::Relaxed), 2 * family_count);
    }

    #[test]
    fn alias_cache_replays_incumbent_equal_donor_slots() {
        let n = 16;
        let edges: Vec<_> = (1..n).map(|v| (0, v)).collect();
        let pattern = Pattern::from_edges(n, &edges);
        let sp = ScoringPattern {
            n,
            col_ptr: pattern.col_ptr.clone(),
            row_idx: pattern.row_idx.clone(),
        };
        let degrees: Vec<_> = pattern.col_ptr.windows(2).map(|p| p[1] - p[0]).collect();
        let mut cache = CandidateCache::new(n, &degrees);
        let mut plain: Vec<CandFn<'_>> = Vec::new();
        let mut cached: Vec<CandFn<'_>> = Vec::new();
        for alpha in [10.0, 1.0] {
            let produce = move || Ok((0..n as i32).rev().collect());
            plain.push(Box::new(produce));
            cached.push(cache.wrap(Family::Amf, alpha, true, None, Box::new(produce)));
        }
        let mut expected_perm: Vec<_> = (0..n).collect();
        let mut actual_perm = expected_perm.clone();
        let mut ws = super::super::scoring_ws::ScoreWorkspace::new(n, pattern.nnz());
        let mut expected_flops = ws.flops(&sp, &expected_perm);
        let mut actual_flops = expected_flops;
        let expected_donors = std::cell::RefCell::new(Vec::new());
        let actual_donors = std::cell::RefCell::new(Vec::new());
        super::super::flush_batch(
            &mut plain,
            &sp,
            n,
            pattern.nnz(),
            &expected_donors,
            &mut expected_flops,
            &mut expected_perm,
            None,
        );
        super::super::flush_batch(
            &mut cached,
            &sp,
            n,
            pattern.nnz(),
            &actual_donors,
            &mut actual_flops,
            &mut actual_perm,
            None,
        );
        assert_eq!(actual_perm, expected_perm);
        assert_eq!(actual_flops, expected_flops);
        assert_eq!(actual_donors, expected_donors);
        let donors = actual_donors.borrow();
        assert_eq!(donors.len(), 2);
        // This entry exists only because the second, equal candidate replays.
        assert_eq!(donors[0], (actual_flops, actual_perm));
    }

    #[test]
    fn alias_cache_generic_key_includes_all_computational_options() {
        let mut spec = super::super::metric_sweep::EXTRA_METRICS[0];
        let original = Family::generic(&spec);
        spec.name = "reporting-only";
        assert!(original == Family::generic(&spec));
        spec.bucket_div += 1;
        assert!(original != Family::generic(&spec));
        spec.bucket_div -= 1;
        spec.wf_weight = 0.5;
        assert!(original != Family::generic(&spec));
    }
}
