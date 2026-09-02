//! Score aggregation and Stage-C validation — shared by the harness, the
//! grader, and the private baseline tools — one scoring code path. Moved
//! verbatim from the harness main.rs; the score DEFINITION is frozen, and
//! these constants and functions are it.

/// Number of size buckets the score is aggregated over.
pub const BUCKETS: usize = 3;
/// Stable metric keys for the buckets, in index order (see `size_bucket`).
pub const BUCKET_KEYS: [&str; BUCKETS] = ["lt_1k", "1k_10k", "gt_10k"];
/// Weights per bucket. Real-world value and algorithmic difficulty concentrate
/// in the large matrices, so `gt_10k` carries the most weight. Empty buckets are
/// renormalized out in `combine`, so these need not be pre-normalized.
pub const BUCKET_WEIGHTS: [f64; BUCKETS] = [0.30, 0.30, 0.40];

/// Classify a matrix by its dimension `n` into a bucket index (half-open):
/// `n < 1000 → 0` (lt_1k), `1000 ≤ n < 10000 → 1` (1k_10k), `n ≥ 10000 → 2` (gt_10k).
pub fn size_bucket(n: usize) -> usize {
    if n < 1_000 {
        0
    } else if n < 10_000 {
        1
    } else {
        2
    }
}

/// Per-bucket accumulator: sums of log-ratios (for the geomean) and a count.
#[derive(Default, Clone, Copy)]
pub struct BucketAcc {
    pub log_ratio_sum: f64,
    pub log_fill_sum: f64,
    pub count: usize,
}

/// Geometric mean from a sum of natural logs and a count. `None` for an empty
/// bucket (no matrices), so `combine` can renormalize it out.
pub fn geomean(log_sum: f64, count: usize) -> Option<f64> {
    if count == 0 {
        None
    } else {
        Some((log_sum / count as f64).exp())
    }
}

/// Weighted mean of the per-bucket geomeans, renormalizing the weights over the
/// populated (`Some`) buckets. Returns `NaN` if every bucket is empty.
pub fn combine(geomeans: &[Option<f64>; BUCKETS], weights: &[f64; BUCKETS]) -> f64 {
    let mut num = 0.0_f64;
    let mut den = 0.0_f64;
    for i in 0..BUCKETS {
        if let Some(g) = geomeans[i] {
            num += weights[i] * g;
            den += weights[i];
        }
    }
    if den == 0.0 {
        f64::NAN
    } else {
        num / den
    }
}

/// Stage C: the permutation must be a true bijection of 0..n.
pub fn validate_permutation(perm: &[usize], n: usize) -> Result<(), String> {
    if perm.len() != n {
        return Err(format!("permutation has length {}, expected {}", perm.len(), n));
    }
    let mut seen = vec![false; n];
    for &v in perm {
        if v >= n {
            return Err(format!("index {} out of range 0..{}", v, n));
        }
        if seen[v] {
            return Err(format!("index {} appears more than once", v));
        }
        seen[v] = true;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_bucket_boundaries() {
        assert_eq!(size_bucket(0), 0);
        assert_eq!(size_bucket(999), 0);
        assert_eq!(size_bucket(1000), 1);
        assert_eq!(size_bucket(9999), 1);
        assert_eq!(size_bucket(10000), 2);
        assert_eq!(size_bucket(340_000), 2);
    }

    #[test]
    fn geomean_empty_is_none() {
        assert_eq!(geomean(0.0, 0), None);
    }

    #[test]
    fn geomean_matches_exp_mean() {
        // two ratios 0.5 and 0.8 → geomean = sqrt(0.4) ≈ 0.632455
        let ls = 0.5_f64.ln() + 0.8_f64.ln();
        let g = geomean(ls, 2).unwrap();
        assert!((g - (0.4_f64).sqrt()).abs() < 1e-12, "g = {g}");
    }

    #[test]
    fn combine_all_populated_matches_worked_example() {
        let gms = [Some(0.8), Some(0.9), Some(0.7)];
        let got = combine(&gms, &BUCKET_WEIGHTS);
        let want = 0.30 * 0.8 + 0.30 * 0.9 + 0.40 * 0.7;
        assert!((got - want).abs() < 1e-12, "got = {got}, want = {want}");
    }

    #[test]
    fn combine_one_empty_renormalizes() {
        let gms = [None, Some(0.9), Some(0.7)];
        let got = combine(&gms, &BUCKET_WEIGHTS);
        let want = (0.30 * 0.9 + 0.40 * 0.7) / (0.30 + 0.40);
        assert!((got - want).abs() < 1e-12, "got = {got}, want = {want}");
    }

    #[test]
    fn combine_only_one_populated_is_that_geomean() {
        let gms = [Some(0.873), None, None];
        let got = combine(&gms, &BUCKET_WEIGHTS);
        assert!((got - 0.873).abs() < 1e-12, "got = {got}");
    }

    #[test]
    fn combine_all_empty_is_nan() {
        let gms = [None, None, None];
        assert!(combine(&gms, &BUCKET_WEIGHTS).is_nan());
    }

    #[test]
    fn validate_permutation_accepts_bijection() {
        assert!(validate_permutation(&[2, 0, 1], 3).is_ok());
        assert!(validate_permutation(&[], 0).is_ok());
    }

    #[test]
    fn validate_permutation_rejects_bad_perms() {
        assert!(validate_permutation(&[0, 1], 3).is_err()); // wrong length
        assert!(validate_permutation(&[0, 3, 1], 3).is_err()); // out of range
        assert!(validate_permutation(&[0, 1, 1], 3).is_err()); // duplicate
    }
}
