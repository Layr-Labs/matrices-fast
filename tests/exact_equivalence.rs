//! Exact-grader equivalence: local and graded scores are the same number.
//!
//! The number the local harness prints for a given (ordering, corpus) must be
//! IDENTICAL to the number the grader computes for the same ordering on the same
//! matrices — not merely "computed the same way". That is guaranteed structurally
//! (harness and grader both call `ssi_scoring::score`, a pure function of
//! (pattern, permutation)). This test PINS that guarantee from OUTSIDE the
//! `ssi-scoring` crate — through its public API (`Pattern::from_edges` + `score`),
//! the same surface the harness and grader use.
//!
//! ## Why the expected values are CLOSED-FORM, not code-derived
//!
//! A pinned number copied from the scorer's own output can only detect *drift*;
//! it can never prove the scorer is *correct*, because it would pass even if the
//! scorer were wrong at the moment the number was recorded (the assertion would
//! just enshrine the wrong number). So every expected value here is derived by
//! HAND from the matrix structure under the identity ordering and the Σc² model,
//! then asserted against `score`. Because the expectation comes from mathematics
//! rather than the code, this test checks correctness AND detects drift — and it
//! is corpus-independent, so replacing `corpus/dev/patterns.jsonl` never breaks
//! it (the real-corpus loader path is covered by the `ssi-scoring` loader tests).
//!
//! Under the model, for column j of the factor L, `c_j` is the number of
//! nonzeros in that column INCLUDING the diagonal; `nnz_l = Σ_j c_j` and
//! `flops = Σ_j c_j²`. If any of these hand-derived facts and the scorer ever
//! disagree, investigate the derivation OR the scorer — do NOT "fix" the numbers.

use ssi_scoring::{score, Pattern};

/// Score the identity (natural) ordering on a pattern built from an edge list,
/// through the public scoring API — the same path the harness and grader use.
fn score_identity(p: &Pattern) -> ssi_scoring::Score {
    let identity: Vec<usize> = (0..p.n).collect();
    score(p, &identity)
}

#[test]
fn dense_3x3_identity_is_closed_form() {
    // Full 3×3 (every off-diagonal present). Under identity, no ordering can
    // reduce a complete graph: column counts are 3, 2, 1.
    //   nnz_l = 3 + 2 + 1 = 6
    //   flops = 3² + 2² + 1² = 9 + 4 + 1 = 14
    let p = Pattern::from_edges(3, &[(0, 1), (0, 2), (1, 2)]);
    let s = score_identity(&p);
    assert_eq!(s.nnz_l, 6, "nnz_l drift on dense 3×3: got {}", s.nnz_l);
    assert_eq!(s.flops, 14, "flops drift on dense 3×3: got {}", s.flops);
}

#[test]
fn tridiagonal_identity_is_closed_form() {
    // Path graph on n nodes (edges i–i+1). Under the natural order this is the
    // textbook ZERO-FILL case: eliminating node j only ever couples j to j+1,
    // which is already an edge. Column j (j < n−1) keeps its diagonal + the one
    // subdiagonal entry → c_j = 2; the last column has only its diagonal → c = 1.
    //   nnz_l = 2·(n−1) + 1 = 2n − 1
    //   flops = 2²·(n−1) + 1² = 4n − 3
    let n = 100;
    let edges: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
    let p = Pattern::from_edges(n, &edges);
    let s = score_identity(&p);
    assert_eq!(s.nnz_l, (2 * n - 1) as u64, "nnz_l drift on tridiagonal: got {}", s.nnz_l);
    assert_eq!(s.flops, (4 * n - 3) as u64, "flops drift on tridiagonal: got {}", s.flops);
}

#[test]
fn star_hub_last_identity_is_closed_form() {
    // Pure star: leaves 0..n−1 each joined ONLY to the hub (node n−1). Under the
    // identity order the hub is eliminated LAST, so every leaf is eliminated
    // while its sole neighbour (the hub) is still present — a degree-1
    // elimination adds no fill. Each leaf column has its diagonal + the hub entry
    // → c = 2; the hub column has only its diagonal → c = 1.
    //   nnz_l = 2·(n−1) + 1 = 2n − 1
    //   flops = 2²·(n−1) + 1² = 4n − 3
    // With n = 1001 this reproduces the historical `gilbert` pin (2001 / 4001):
    // gilbert was exactly a hub-last star of this size.
    let n = 1001;
    let hub = n - 1;
    let edges: Vec<(usize, usize)> = (0..hub).map(|leaf| (leaf, hub)).collect();
    let p = Pattern::from_edges(n, &edges);
    let s = score_identity(&p);
    assert_eq!(s.nnz_l, 2001, "nnz_l drift on hub-last star (n=1001): got {}", s.nnz_l);
    assert_eq!(s.flops, 4001, "flops drift on hub-last star (n=1001): got {}", s.flops);
}
