//! Reuse identical quotient-metric alpha walks without deleting replay slots.
//!
//! Alpha affects the quotient workspace only through dense classification.
//! The deferred sets {degree > threshold} are nested, so equal cardinality
//! implies equal sets and identical initialized workspaces. Cells are shared
//! only within one (pattern, variant, aggressive) group, never across calls.

use std::sync::{Arc, OnceLock};
use feral_ordering_core::OrderingError;

pub(super) type OrderCell = Arc<OnceLock<Result<Vec<i32>, OrderingError>>>;

pub(super) fn alpha_cells(n: usize, degrees: &[usize], alphas: &[f64]) -> Vec<OrderCell> {
    let mut groups: Vec<(usize, OrderCell)> = Vec::new();
    alphas.iter().map(|&alpha| {
        let count = super::dense_deferred_count(n, degrees, alpha);
        if let Some((_, cell)) = groups.iter().find(|(key, _)| *key == count) {
            return Arc::clone(cell);
        }
        let cell = Arc::new(OnceLock::new());
        groups.push((count, Arc::clone(&cell)));
        cell
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::custom_metrics::{order_variant, ScoreVariant};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn cached_alpha_walks_match_uncached_permutations() {
        let alphas = [10.0, 5.0, 2.5, 1.0];
        let mut checked = 0;
        for n in [0usize, 4, 16, 17, 32, 64, 96] {
            for shape in 0..3 {
                let mut edges = Vec::new();
                for u in 0..n {
                    for v in u + 1..n {
                        if match shape {
                            0 => v == u + 1,
                            1 => u == 0 || v == u + 1,
                            _ => u < n / 3 || v == u + 1,
                        } {
                            edges.push((u, v));
                        }
                    }
                }
                let pat = crate::Pattern::from_edges(n, &edges);
                let degrees: Vec<_> = pat.col_ptr.windows(2).map(|w| w[1] - w[0]).collect();
                let cp: Vec<i32> = pat.col_ptr.iter().map(|&v| v as i32).collect();
                let ri: Vec<i32> = pat.row_idx.iter().map(|&v| v as i32).collect();
                let core = feral_ordering_core::CscPattern::new(n, &cp, &ri).unwrap();
                for variant in [ScoreVariant::SqDiv, ScoreVariant::DegP125, ScoreVariant::Ammf] {
                    let cells = alpha_cells(n, &degrees, &alphas);
                    let calls = AtomicUsize::new(0);
                    let results = std::thread::scope(|scope| {
                        let handles: Vec<_> = alphas.iter().zip(&cells).map(|(&alpha, cell)| {
                            let core = &core;
                            let calls = &calls;
                            scope.spawn(move || {
                                cell.get_or_init(|| {
                                    calls.fetch_add(1, Ordering::Relaxed);
                                    order_variant(core, alpha, true, variant)
                                }).clone()
                            })
                        }).collect();
                        handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>()
                    });
                    let mut counts: Vec<_> = alphas.iter().map(|&a|
                        super::super::dense_deferred_count(n, &degrees, a)).collect();
                    counts.sort_unstable();
                    counts.dedup();
                    assert_eq!(calls.load(Ordering::Relaxed), counts.len());
                    for (&alpha, actual) in alphas.iter().zip(results) {
                        assert_eq!(actual, order_variant(&core, alpha, true, variant));
                        checked += 1;
                    }
                }
            }
        }
        println!("ALPHA_CACHE exact_uncached_comparisons={checked}");
    }

    #[test]
    fn distinct_dense_sets_do_not_share_cells() {
        let degrees: Vec<_> = (0..100).collect();
        let alphas = [10.0, 5.0, 2.5, 1.0];
        let cells = alpha_cells(100, &degrees, &alphas);
        for i in 0..cells.len() {
            for j in 0..i {
                assert!(!Arc::ptr_eq(&cells[i], &cells[j]));
            }
        }
    }
}
