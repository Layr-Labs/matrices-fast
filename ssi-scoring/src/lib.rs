//! THE SCORING WRAPPER — the one scoring code path.
//!
//! A thin library wrapping feral's symbolic analysis and AMD. It is the ONLY
//! code in this workspace that calls into `feral`. Both the public harness
//! (what contestants run locally) and the private grader score by calling the
//! functions here, so a contestant's local score is structurally identical to
//! the graded score (see `tests/exact_equivalence.rs`).
//!
//! ## The pattern-pure building-block path
//!
//! `score()` does NOT call `feral::symbolic_factorize` — that high-level entry
//! point is value-dependent (its `LdltCompress`/MC64 ordering preprocess reads
//! numeric values). Instead it composes feral's already-public, pattern-only
//! building blocks, which is feral's own fill computation:
//!
//! ```text
//!   Pattern (full symmetric, usize)
//!     → feral CscPattern (usize)
//!     → permute_pattern(pattern, P)            // apply contestant permutation
//!     → EliminationTree::from_pattern          // Liu 1986
//!     → column_counts_gnp                       // Gilbert–Ng–Peyton, exact c_j
//!     → nnz_l = Σ c_j  (total_factor_nnz)
//!       flops = Σ c_j²  (the deterministic Σc² model)
//! ```
//!
//! Both the contestant permutation AND the AMD baseline go through this exact
//! path, so the leaderboard ratio is computed identically on both sides.
//!
//! ## Why there is no amalgamation knob
//!
//! Supernode amalgamation is a NON-ISSUE here, not a setting to tune: column
//! counts are computed BEFORE amalgamation in feral's pipeline, and fill is
//! invariant under the within-subtree relabeling amalgamation performs. This building-block path never touches amalgamation at all, so
//! there is nothing to pin. Do not "re-add" a pinning knob.
//!
//! ## Why `flops = Σ c_j²` and `nnz_l = Σ c_j`
//!
//! feral exposes no exact symbolic LDLᵀ flop field. It exposes exact per-column
//! counts; the flop count is *derived* as Σ c_j². Do NOT use
//! `factor_nnz_estimate` (carries a 1.2× slack) or `estimate_assembly_flops` /
//! AMD's `nms_*` counters (they measure other things).

use feral::ordering::elimination_tree::EliminationTree;
use feral::sparse::csc::CscPattern;
use feral::symbolic::{column_counts_gnp, total_factor_nnz};

mod loader;
mod pattern;
mod aggregate;

pub use loader::{load_corpus_jsonl, pattern_from_jsonl_line, LoadError};
pub use pattern::Pattern;
pub use aggregate::{
    combine, geomean, size_bucket, validate_permutation, BucketAcc, BUCKETS,
    BUCKET_KEYS, BUCKET_WEIGHTS,
};

/// The symbolic factorization cost of an ordering, derived purely from the
/// sparsity pattern and the permutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Score {
    /// nnz(L), including the diagonal: `Σ_j c_j`.
    pub nnz_l: u64,
    /// Deterministic flop proxy: `Σ_j c_j²`, where `c_j` is the column count of
    /// L (including diagonal). Proportional to the dense-operation cost of the
    /// LDLᵀ factorization under this ordering.
    pub flops: u64,
}

/// Convert a stdlib `Pattern` (full symmetric, `usize`) into feral's owned
/// `usize`-indexed `CscPattern` used by the symbolic building blocks. This is
/// a structural re-wrap; values never enter (NARROW INPUT preserved).
fn to_feral_pattern(p: &Pattern) -> CscPattern {
    CscPattern {
        n: p.n,
        col_ptr: p.col_ptr.clone(),
        row_idx: p.row_idx.clone(),
    }
}

/// Score a permutation `P` of pattern `A` via the pattern-pure building-block
/// path. `perm[k]` is the original index eliminated k-th (new-to-old). The
/// permutation must already be a validated bijection of `0..n`.
///
/// This is a pure function of `(pattern, permutation)` — the structural
/// guarantee behind exact-grader equivalence.
pub fn score(a: &Pattern, perm: &[usize]) -> Score {
    let pat = to_feral_pattern(a);
    let permuted = feral::ordering::amd::permute_pattern(&pat, perm);
    let etree = EliminationTree::from_pattern(&permuted);
    let counts = column_counts_gnp(&permuted, &etree);
    let nnz_l = total_factor_nnz(&counts) as u64;
    let flops: u64 = counts.iter().map(|&c| (c as u64) * (c as u64)).sum();
    Score { nnz_l, flops }
}

/// The competition's AMD baseline: pinned to `feral_amd::amd_order` on the raw
/// full-symmetric pattern with DEFAULT `AmdOptions` — deterministic and
/// pattern-pure.
///
/// This is NOT feral's *default* ordering (`symbolic_factorize`'s `Auto`
/// resolves to AMF/MetisND and can route through value-reading MC64 matching).
/// We deliberately call the AMD crate directly so the baseline is reproducible
/// from a pattern file alone.
///
/// Returns a new-to-old permutation as `Vec<usize>` (converted from feral's
/// `i32`). Panics only on an internal AMD error or an `i32`-overflow-sized
/// matrix; the caller (harness/grader) treats either as a hard failure.
pub fn amd_baseline(a: &Pattern) -> Vec<usize> {
    // feral_ordering_core's CscPattern is borrowed + i32-indexed. Build the
    // i32 buffers at this trusted boundary, never in contestant code.
    let col_ptr: Vec<i32> = a
        .col_ptr
        .iter()
        .map(|&x| i32::try_from(x).expect("matrix too large for i32-indexed AMD"))
        .collect();
    let row_idx: Vec<i32> = a
        .row_idx
        .iter()
        .map(|&x| i32::try_from(x).expect("matrix too large for i32-indexed AMD"))
        .collect();
    let pat = feral_ordering_core::CscPattern::new(a.n, &col_ptr, &row_idx)
        .expect("ssi-scoring built a malformed CscPattern for AMD (bug in Pattern invariants)");
    let perm_i32 = feral_amd::amd_order(&pat).expect("feral AMD baseline failed");
    perm_i32.into_iter().map(|x| x as usize).collect()
}

/// Convenience: load a `.mtx` file and AMD-order it, returning the pattern, the
/// baseline permutation, and the baseline score. Used by the harness/grader to
/// anchor the leaderboard at 1.00. Kept here so the corpus-loading + baseline +
/// scoring path is exercised by one call in tests.
pub fn baseline_score(a: &Pattern) -> Score {
    score(a, &amd_baseline(a))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(n: usize) -> Vec<usize> {
        (0..n).collect()
    }

    // ---- The closed-form tests below are mathematical facts. ----
    // A scorer that fails them is wrong by definition. These are ported from
    // the prototype symbolic.rs and must pass against the feral-backed scorer.

    #[test]
    fn dense_3x3_flops_14() {
        // Full 3×3: column counts 3, 2, 1 → nnz(L)=6, flops=9+4+1=14.
        let p = Pattern::from_edges(3, &[(0, 1), (0, 2), (1, 2)]);
        let s = score(&p, &identity(3));
        assert_eq!(s.nnz_l, 6);
        assert_eq!(s.flops, 14);
    }

    #[test]
    fn star_5_hub_first_flops_55() {
        // 5×5 star: hub node 0 joined to leaves 1,2,3,4 (leaves otherwise
        // disconnected). Eliminating the hub FIRST fills the four leaves into a
        // clique → fully dense factor. Column counts 5, 4, 3, 2, 1 → nnz(L)=15
        // (= n(n+1)/2), flops=25+16+9+4+1=55.
        let p = Pattern::from_edges(5, &[(0, 1), (0, 2), (0, 3), (0, 4)]);
        let s = score(&p, &identity(5));
        assert_eq!(s.nnz_l, 15);
        assert_eq!(s.flops, 55);
    }

    #[test]
    fn tridiagonal_zero_fill() {
        // tridiagonal n → nnz(L) = 2n − 1 (zero fill).
        let n = 100;
        let edges: Vec<_> = (0..n - 1).map(|i| (i, i + 1)).collect();
        let p = Pattern::from_edges(n, &edges);
        let s = score(&p, &identity(n));
        assert_eq!(s.nnz_l, (2 * n - 1) as u64);
    }

    #[test]
    fn arrow_hub_first_is_fully_dense() {
        // arrow with hub eliminated first → fully dense factor, n(n+1)/2.
        let n = 50;
        let mut edges = Vec::new();
        for v in 1..n {
            edges.push((0, v));
            if v + 1 < n {
                edges.push((v, v + 1));
            }
        }
        let p = Pattern::from_edges(n, &edges);
        let dense = score(&p, &identity(n));
        assert_eq!(dense.nnz_l, (n * (n + 1) / 2) as u64);
    }

    #[test]
    fn arrow_hub_last_is_near_zero_fill() {
        // arrow with hub eliminated last → near-zero fill.
        let n = 50;
        let mut edges = Vec::new();
        for v in 1..n {
            edges.push((0, v));
            if v + 1 < n {
                edges.push((v, v + 1));
            }
        }
        let p = Pattern::from_edges(n, &edges);
        let mut hub_last: Vec<usize> = (1..n).collect();
        hub_last.push(0);
        let sparse = score(&p, &hub_last);
        assert!(sparse.nnz_l < (3 * n) as u64, "nnz_l = {}", sparse.nnz_l);
    }

    #[test]
    fn amd_baseline_is_a_valid_bijection() {
        let n = 50;
        let mut edges = Vec::new();
        for v in 1..n {
            edges.push((0, v));
            if v + 1 < n {
                edges.push((v, v + 1));
            }
        }
        let p = Pattern::from_edges(n, &edges);
        let perm = amd_baseline(&p);
        assert_eq!(perm.len(), n);
        let mut seen = vec![false; n];
        for &v in &perm {
            assert!(v < n && !seen[v]);
            seen[v] = true;
        }
    }

    #[test]
    fn amd_beats_hub_first_on_arrow() {
        // AMD should defer the dense hub, beating natural order by orders of
        // magnitude — the canonical ordering-matters demonstration.
        let n = 200;
        let mut edges = Vec::new();
        for v in 1..n {
            edges.push((0, v));
            if v + 1 < n {
                edges.push((v, v + 1));
            }
        }
        let p = Pattern::from_edges(n, &edges);
        let amd = baseline_score(&p);
        let natural = score(&p, &identity(n));
        assert!(amd.flops * 50 < natural.flops, "amd={:?} natural={:?}", amd, natural);
    }
}
