//! Prototype symbolic scorer + synthetic generators, kept as a TEST ORACLE.
//!
//! This is the original prototype scorer (elimination tree + row-subtree column
//! counts, `nnz(L) = Σ c_j`, `flops = Σ c_j²`) and the synthetic corpus
//! generators (`arrow`, `grid2d`, `grid3d`, `kkt`), preserved verbatim and made
//! self-contained. It exists ONLY so the workspace's cross-check test can
//! confirm the feral-backed `ssi-scoring` agrees with an INDEPENDENT
//! implementation: the closed-form tests were ported onto the new scorer
//! before the prototype was retired, and this oracle keeps them honest.
//!
//! It is a dev-dependency of the harness and is never linked into the shipped
//! binary. It deliberately shares no code with `ssi-scoring`.

/// Minimal owned full-symmetric pattern (both triangles, diagonal omitted).
#[derive(Clone)]
pub struct Pattern {
    pub n: usize,
    pub col_ptr: Vec<usize>,
    pub row_idx: Vec<usize>,
}

impl Pattern {
    pub fn from_edges(n: usize, edges: &[(usize, usize)]) -> Pattern {
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for &(r, c) in edges {
            assert!(r < n && c < n, "edge out of range");
            if r == c {
                continue;
            }
            adj[c].push(r);
            adj[r].push(c);
        }
        let mut col_ptr = Vec::with_capacity(n + 1);
        let mut row_idx = Vec::new();
        col_ptr.push(0);
        for list in adj.iter_mut() {
            list.sort_unstable();
            list.dedup();
            row_idx.extend_from_slice(list);
            col_ptr.push(row_idx.len());
        }
        Pattern { n, col_ptr, row_idx }
    }

    pub fn nnz(&self) -> usize {
        self.row_idx.len()
    }

    pub fn col(&self, j: usize) -> &[usize] {
        &self.row_idx[self.col_ptr[j]..self.col_ptr[j + 1]]
    }
}

/// Symbolic cost: nnz(L) including diagonal, and the Σc² flop proxy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbolic {
    pub nnz_l: u64,
    pub flops: u64,
}

const NONE: usize = usize::MAX;

/// Compute the symbolic factorization cost of P·A·Pᵀ.
/// `perm[k]` is the original index of the vertex eliminated k-th.
pub fn analyze(a: &Pattern, perm: &[usize]) -> Symbolic {
    let n = a.n;
    assert_eq!(perm.len(), n);

    let mut pinv = vec![0usize; n];
    for (k, &v) in perm.iter().enumerate() {
        pinv[v] = k;
    }

    // Materialize the permuted pattern B = P·A·Pᵀ in CSC.
    let mut counts = vec![0usize; n + 1];
    for c in 0..n {
        counts[pinv[c] + 1] += a.col_ptr[c + 1] - a.col_ptr[c];
    }
    for j in 0..n {
        counts[j + 1] += counts[j];
    }
    let cp = counts;
    let mut rows = vec![0usize; a.nnz()];
    {
        let mut head: Vec<usize> = cp[..n].to_vec();
        for c in 0..n {
            let jc = pinv[c];
            for &r in a.col(c) {
                rows[head[jc]] = pinv[r];
                head[jc] += 1;
            }
        }
    }

    // Elimination tree of B (Liu 1986, with ancestor path compression).
    let mut parent = vec![NONE; n];
    let mut ancestor = vec![NONE; n];
    for j in 0..n {
        for idx in cp[j]..cp[j + 1] {
            let mut i = rows[idx];
            if i >= j {
                continue;
            }
            while ancestor[i] != NONE && ancestor[i] != j {
                let next = ancestor[i];
                ancestor[i] = j;
                i = next;
            }
            if ancestor[i] == NONE {
                ancestor[i] = j;
                parent[i] = j;
            }
        }
    }

    // Column counts via row-subtree traversal.
    let mut count = vec![1u64; n];
    let mut mark = vec![NONE; n];
    for i in 0..n {
        mark[i] = i;
        for idx in cp[i]..cp[i + 1] {
            let mut k = rows[idx];
            if k >= i {
                continue;
            }
            while mark[k] != i {
                mark[k] = i;
                count[k] += 1;
                match parent[k] {
                    NONE => break,
                    p => k = p,
                }
            }
        }
    }

    let nnz_l: u64 = count.iter().sum();
    let flops: u64 = count.iter().map(|&c| c * c).sum();
    Symbolic { nnz_l, flops }
}

// --- Synthetic generators (kept as `cargo test` fixtures only). ---

pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Lcg {
        Lcg(seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(1))
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    pub fn below(&mut self, m: usize) -> usize {
        (self.next_u64() % m as u64) as usize
    }
}

pub fn grid2d(k: usize) -> Pattern {
    let n = k * k;
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
    Pattern::from_edges(n, &edges)
}

pub fn grid3d(k: usize) -> Pattern {
    let n = k * k * k;
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
    Pattern::from_edges(n, &edges)
}

pub fn kkt(nh: usize, m: usize, seed: u64) -> Pattern {
    let n = nh + m;
    let mut rng = Lcg::new(seed);
    let mut edges = Vec::new();
    let bw = 40;
    for i in 0..nh {
        if i + 1 < nh {
            edges.push((i, i + 1));
        }
        for _ in 0..2 {
            let off = 1 + rng.below(bw);
            let j = if rng.below(2) == 0 {
                i.saturating_sub(off)
            } else {
                (i + off).min(nh - 1)
            };
            if j != i {
                edges.push((i, j));
            }
        }
    }
    for c in 0..m {
        let row = nh + c;
        let center = c * nh / m;
        let deg = 3 + rng.below(4);
        for _ in 0..deg {
            let off = rng.below(2 * bw);
            let j = (center + off).min(nh - 1);
            edges.push((row, j));
        }
    }
    Pattern::from_edges(n, &edges)
}

pub fn arrow(n: usize) -> Pattern {
    let mut edges = Vec::new();
    for v in 1..n {
        edges.push((0, v));
        if v + 1 < n {
            edges.push((v, v + 1));
        }
    }
    Pattern::from_edges(n, &edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(n: usize) -> Vec<usize> {
        (0..n).collect()
    }

    #[test]
    fn dense_3x3() {
        let p = Pattern::from_edges(3, &[(0, 1), (0, 2), (1, 2)]);
        let s = analyze(&p, &identity(3));
        assert_eq!(s.nnz_l, 6);
        assert_eq!(s.flops, 14);
    }

    #[test]
    fn tridiagonal_has_no_fill() {
        let n = 100;
        let edges: Vec<_> = (0..n - 1).map(|i| (i, i + 1)).collect();
        let p = Pattern::from_edges(n, &edges);
        let s = analyze(&p, &identity(n));
        assert_eq!(s.nnz_l, (2 * n - 1) as u64);
    }

    #[test]
    fn arrow_ordering_matters() {
        let n = 50;
        let p = arrow(n);
        let dense = analyze(&p, &identity(n));
        assert_eq!(dense.nnz_l, (n * (n + 1) / 2) as u64);
        let mut hub_last: Vec<usize> = (1..n).collect();
        hub_last.push(0);
        let sparse = analyze(&p, &hub_last);
        assert!(sparse.nnz_l < (3 * n) as u64);
        assert!(dense.flops > 50 * sparse.flops);
    }
}
