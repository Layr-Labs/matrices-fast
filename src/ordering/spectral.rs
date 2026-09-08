//! SPECTRAL ORDERING (Fiedler) — a genuinely different permutation family.
//!
//! Every other ordering in the portfolio is degree-driven (AMD/AMF/minfill),
//! separator-driven (ND/METIS/Scotch/KaHIP), bandwidth-driven (RCM/Sloan) or
//! a local search thereof. Spectral ordering sorts by the Fiedler vector
//! (second-smallest Laplacian eigenvector): a *global* continuous relaxation
//! of the cut, unrelated to any greedy elimination rule. As one best-of
//! candidate it can only win where every other family ties or misses.
//!
//! DETERMINISM. Fixed start vector (centered indices), fixed step count,
//! full reorthogonalisation in fixed index order, cyclic Jacobi in fixed
//! order, index tie-breaks. Same input ⇒ same permutation, byte for byte.
//! No randomness, no hash iteration, std only.
//!
//! COST. `k` Laplacian matvecs (`k ≤ 24`) at O(nnz) each plus O(k²·n)
//! reorthogonalisation, then a k×k Jacobi eigensolve. Caller gates size.

/// Spectral (Fiedler) ordering of a symmetric pattern with omitted diagonal.
///
/// Returns `None` only for empty input; on any degenerate numeric case it
/// falls back to a deterministic degree order (still a valid candidate —
/// the caller's best-of floor discards it if it loses).
pub(crate) fn spectral_order(col_ptr: &[usize], row_idx: &[usize], n: usize) -> Option<Vec<i32>> {
    if n == 0 || col_ptr.len() != n + 1 {
        return None;
    }
    // Degrees (pattern has no diagonal by convention).
    let mut deg = vec![0usize; n];
    for j in 0..n {
        let (s, e) = (col_ptr[j], col_ptr[j + 1].min(row_idx.len()));
        if e < s {
            return None;
        }
        deg[j] = e - s;
    }
    let order_degree = || -> Vec<i32> {
        let mut v: Vec<usize> = (0..n).collect();
        v.sort_by_key(|&u| (deg[u], u));
        v.into_iter().map(|u| u as i32).collect()
    };
    if n < 3 {
        return Some(order_degree());
    }

    // Laplacian matvec: (Lx)[j] = deg[j]*x[j] - sum_{w in N(j)} x[w].
    let matvec = |x: &[f64], out: &mut [f64]| {
        for j in 0..n {
            let mut acc = deg[j] as f64 * x[j];
            for &w in &row_idx[col_ptr[j]..col_ptr[j + 1]] {
                if w < n {
                    acc -= x[w];
                }
            }
            out[j] = acc;
        }
    };

    // Deterministic start: raw indices, normalised. Deliberately NOT
    // orthogonal to the ones vector (and never projected off it below):
    // the Krylov space must contain the Laplacian null direction so the
    // smallest Ritz value comes out ≈ 0 and the SECOND smallest is the
    // Fiedler value. (A centered start is exactly ⊥ ones on symmetric
    // graphs; the nullspace then never enters and idx[1] is wrong.)
    let mut v0: Vec<f64> = (0..n).map(|j| j as f64).collect();
    let mut norm: f64 = v0.iter().map(|x| x * x).sum::<f64>().sqrt();
    if !(norm > 0.0) {
        return Some(order_degree());
    }
    for x in v0.iter_mut() {
        *x /= norm;
    }

    // Lanczos with full reorthogonalisation, fixed step count. Builds
    // orthonormal V (n×k) and tridiagonal T (k×k).
    let k = 24usize.min(n);
    let mut vs: Vec<Vec<f64>> = Vec::with_capacity(k);
    let mut alpha = vec![0.0f64; k];
    let mut beta = vec![0.0f64; k.saturating_sub(1)];
    let mut w = vec![0.0f64; n];
    vs.push(v0);
    let mut m = 0usize; // achieved dimension
    for j in 0..k {
        matvec(&vs[j], &mut w);
        // alpha FIRST: <v_j, A v_j> before any subtraction. (Computing it
        // after orthogonalising against v_j would read ~0 and silently
        // zero the tridiagonal diagonal.)
        alpha[j] = vs[j].iter().zip(w.iter()).map(|(a, b)| a * b).sum();
        for (a, b) in w.iter_mut().zip(vs[j].iter()) {
            *a -= alpha[j] * b;
        }
        // Full reorthogonalisation against all previous vectors, fixed
        // order, two passes. No ones-projection: the null direction must
        // stay in the Krylov space (see start-vector note above).
        for _ in 0..2 {
            for v in vs.iter() {
                let d: f64 = v.iter().zip(w.iter()).map(|(a, b)| a * b).sum();
                for (a, b) in w.iter_mut().zip(v.iter()) {
                    *a -= d * b;
                }
            }
        }
        m = j + 1;
        if j + 1 >= k {
            break;
        }
        let b: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        if !(b > 1e-12) {
            break; // Krylov exhaustion: invariant subspace reached.
        }
        beta[j] = b;
        vs.push(w.iter().map(|x| x / b).collect());
    }
    if m < 2 {
        return Some(order_degree());
    }
    // Tridiagonal T (m×m) from alpha/beta.
    let mut t = vec![vec![0.0f64; m]; m];
    for j in 0..m {
        t[j][j] = alpha[j];
        if j + 1 < m {
            t[j][j + 1] = beta[j];
            t[j + 1][j] = beta[j];
        }
    }
    // Cyclic Jacobi eigensolve, fixed sweep order and count. Deterministic.
    let mut eigvec = vec![vec![0.0f64; m]; m];
    for j in 0..m {
        eigvec[j][j] = 1.0;
    }
    for _ in 0..50 {
        let mut off = 0.0;
        for p in 0..m {
            for q in (p + 1)..m {
                off += t[p][q] * t[p][q];
            }
        }
        if !(off > 1e-24) {
            break;
        }
        for p in 0..m {
            for q in (p + 1)..m {
                let apq = t[p][q];
                if apq == 0.0 {
                    continue;
                }
                let app = t[p][p];
                let aqq = t[q][q];
                let theta = (aqq - app) / (2.0 * apq);
                let sgn = if theta >= 0.0 { 1.0 } else { -1.0 };
                let tt = sgn / (theta.abs() + (theta * theta + 1.0).sqrt());
                let c = 1.0 / (tt * tt + 1.0).sqrt();
                let sn = tt * c;
                for r in 0..m {
                    let trp = t[r][p];
                    let trq = t[r][q];
                    t[r][p] = c * trp - sn * trq;
                    t[r][q] = sn * trp + c * trq;
                }
                for r in 0..m {
                    let tpr = t[p][r];
                    let tqr = t[q][r];
                    t[p][r] = c * tpr - sn * tqr;
                    t[q][r] = sn * tpr + c * tqr;
                }
                for r in 0..m {
                    let vpr = eigvec[r][p];
                    let vqr = eigvec[r][q];
                    eigvec[r][p] = c * vpr - sn * vqr;
                    eigvec[r][q] = sn * vpr + c * vqr;
                }
            }
        }
    }
    // Smallest Ritz value ≈ 0 (nullspace); Fiedler = second smallest.
    let mut idx: Vec<usize> = (0..m).collect();
    idx.sort_by(|&a, &b| {
        t[a][a]
            .partial_cmp(&t[b][b])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(&b))
    });
    if m < 2 {
        return Some(order_degree());
    }
    let fi = idx[1];
    // Ritz vector y = V * eigvec[:, fi].
    let mut y = vec![0.0f64; n];
    for (j, v) in vs.iter().enumerate().take(m) {
        let c = eigvec[j][fi];
        for (a, b) in y.iter_mut().zip(v.iter()) {
            *a += c * b;
        }
    }
    if !y.iter().all(|v| v.is_finite()) {
        return Some(order_degree());
    }
    // Order by Fiedler value, ties by index. Sign is arbitrary but fixed
    // (Lanczos/Jacobi are deterministic), so no canonicalisation needed.
    let mut perm: Vec<usize> = (0..n).collect();
    perm.sort_by(|&a, &b| {
        y[a].partial_cmp(&y[b])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(&b))
    });
    Some(perm.into_iter().map(|u| u as i32).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pattern(n: usize, edges: &[(usize, usize)]) -> (Vec<usize>, Vec<usize>) {
        let mut cols = vec![Vec::new(); n];
        for &(a, b) in edges {
            cols[a].push(b);
            cols[b].push(a);
        }
        let mut col_ptr = vec![0usize];
        let mut row_idx = Vec::new();
        for col in cols.iter_mut() {
            col.sort_unstable();
            col.dedup();
            row_idx.extend_from_slice(col);
            col_ptr.push(row_idx.len());
        }
        (col_ptr, row_idx)
    }

    #[test]
    fn spectral_path_is_monotone() {
        // A path's Fiedler vector is monotone along the path (up to sign),
        // so the order is the path or its reverse — both optimal (zero fill
        // beyond the path edges is impossible; any pendant-first order ties).
        let (cp, ri) = pattern(6, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]);
        let p = spectral_order(&cp, &ri, 6).expect("path orders");
        assert_eq!(p.len(), 6);
        let mut s: Vec<i32> = p.clone();
        s.sort_unstable();
        assert_eq!(s, vec![0, 1, 2, 3, 4, 5]);
        let pos = |v: i32| p.iter().position(|&x| x == v).unwrap();
        let fwd = (0..5).all(|v| pos(v) < pos(v + 1));
        let rev = (0..5).all(|v| pos(v) > pos(v + 1));
        assert!(fwd || rev, "path order must be monotone, got {p:?}");
    }

    #[test]
    fn spectral_deterministic_and_bijection() {
        let (cp, ri) = pattern(
            8,
            &[
                (0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3), (5, 6), (6, 7),
            ],
        );
        let a = spectral_order(&cp, &ri, 8).expect("orders");
        let b = spectral_order(&cp, &ri, 8).expect("orders");
        assert_eq!(a, b);
        let mut s = a.clone();
        s.sort_unstable();
        assert_eq!(s, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }
}
