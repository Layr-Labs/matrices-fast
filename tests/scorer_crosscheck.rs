//! Scorer cross-check: the shipped scorer vs an independent implementation.
//!
//! Before retiring the prototype Σc² scorer, prove the feral-backed
//! `ssi_scoring::score` agrees with it on an INDEPENDENT implementation. The
//! prototype lives in the `prototype-oracle` dev-dependency crate and shares no
//! code with `ssi-scoring`.
//!
//! Comparison rule: both scorers compute exact nnz(L) (column counts of the
//! Cholesky/LDLᵀ factor) and the same Σc² flop model, so on the SAME pattern
//! and SAME permutation they must agree EXACTLY on both `nnz_l` and `flops`.
//! (Flop models in the literature differ by constant factors, which is why the
//! score is defined as a ratio; here the two models are identical by
//! construction — Σc² on both sides — so we can and do assert exact equality,
//! the strongest form.)

use prototype_oracle as oracle;
use ssi_scoring::Pattern;

/// Rebuild the same pattern under both type systems from one edge list.
fn both(n: usize, edges: &[(usize, usize)]) -> (Pattern, oracle::Pattern) {
    (
        Pattern::from_edges(n, edges),
        oracle::Pattern::from_edges(n, edges),
    )
}

fn check(n: usize, edges: &[(usize, usize)], perm: &[usize]) {
    let (p_feral, p_oracle) = both(n, edges);
    let s_feral = ssi_scoring::score(&p_feral, perm);
    let s_oracle = oracle::analyze(&p_oracle, perm);
    assert_eq!(
        s_feral.nnz_l, s_oracle.nnz_l,
        "nnz(L) disagreement: feral={} oracle={} (n={n})",
        s_feral.nnz_l, s_oracle.nnz_l
    );
    assert_eq!(
        s_feral.flops, s_oracle.flops,
        "flops disagreement: feral={} oracle={} (n={n})",
        s_feral.flops, s_oracle.flops
    );
}

#[test]
fn agree_on_synthetic_suite() {
    // The synthetic families from the original prototype corpus, across several
    // permutations each (identity, reverse, and AMD), exercise grids, KKTs, and
    // the arrow — the full range of fill behaviors.
    let cases: Vec<(usize, Vec<(usize, usize)>)> = vec![
        // arrow
        (200, oracle_edges_arrow(200)),
        // 2D grids
        (900, oracle_edges_grid2d(30)),
        (3600, oracle_edges_grid2d(60)),
        // 3D grids
        (1000, oracle_edges_grid3d(10)),
        (2744, oracle_edges_grid3d(14)),
    ];

    for (n, edges) in &cases {
        let identity: Vec<usize> = (0..*n).collect();
        let reverse: Vec<usize> = (0..*n).rev().collect();
        check(*n, edges, &identity);
        check(*n, edges, &reverse);
        // AMD permutation from the trusted wrapper — a realistic, non-trivial
        // ordering, the most important case to agree on.
        let p = Pattern::from_edges(*n, edges);
        let amd = ssi_scoring::amd_baseline(&p);
        check(*n, edges, &amd);
    }
}

#[test]
fn agree_on_kkt_family() {
    // KKT patterns use the oracle's seeded generator; rebuild the identical
    // edge list for the feral side by reading the oracle pattern's structure.
    for (nh, m, seed) in [(600usize, 200usize, 42u64), (2000, 700, 7)] {
        let kp = oracle::kkt(nh, m, seed);
        let n = kp.n;
        // Reconstruct an edge list from the oracle pattern (both triangles →
        // dedup to each undirected edge once).
        let mut edges = Vec::new();
        for j in 0..n {
            for &i in kp.col(j) {
                if i < j {
                    edges.push((i, j));
                }
            }
        }
        let identity: Vec<usize> = (0..n).collect();
        check(n, &edges, &identity);
        let p = Pattern::from_edges(n, &edges);
        let amd = ssi_scoring::amd_baseline(&p);
        check(n, &edges, &amd);
    }
}

// --- edge-list generators mirroring the oracle's, for the feral side ---

fn oracle_edges_arrow(n: usize) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for v in 1..n {
        edges.push((0, v));
        if v + 1 < n {
            edges.push((v, v + 1));
        }
    }
    edges
}

fn oracle_edges_grid2d(k: usize) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for x in 0..k {
        for y in 0..k {
            let v = x * k + y;
            if x + 1 < k {
                edges.push((v, (x + 1) * k + y));
            }
            if y + 1 < k {
                edges.push((v, x * k + y + 1));
            }
        }
    }
    edges
}

fn oracle_edges_grid3d(k: usize) -> Vec<(usize, usize)> {
    let idx = |x: usize, y: usize, z: usize| (x * k + y) * k + z;
    let mut edges = Vec::new();
    for x in 0..k {
        for y in 0..k {
            for z in 0..k {
                let v = idx(x, y, z);
                if x + 1 < k {
                    edges.push((v, idx(x + 1, y, z)));
                }
                if y + 1 < k {
                    edges.push((v, idx(x, y + 1, z)));
                }
                if z + 1 < k {
                    edges.push((v, idx(x, y, z + 1)));
                }
            }
        }
    }
    edges
}
