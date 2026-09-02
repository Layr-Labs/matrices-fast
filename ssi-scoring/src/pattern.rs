//! The sparsity-pattern type — THE CONTRACT's input type.
//!
//! `Pattern` is the *only* thing a contestant ordering ever sees: structure,
//! never values, never a right-hand side. It is a stdlib-only type, so no feral
//! type is forced across the `order()` boundary. The `usize`↔`i32` conversion to
//! feral's `CscPattern` happens here, inside the trusted scoring wrapper, never
//! in contestant code.
//!
//! NARROW INPUT: this type carries the full symmetric pattern
//! and nothing else. There is no field for values or a RHS, by construction —
//! the "answer" is physically absent from the contestant's address space.

/// Sparsity pattern of a symmetric matrix, stored as the *full* (both
/// triangles) pattern in compressed-sparse-column form, diagonal omitted.
///
/// Invariants (enforced by the constructors):
/// - `col_ptr.len() == n + 1`, `col_ptr[0] == 0`, non-decreasing
/// - row indices within each column are sorted, unique, in `0..n`,
///   and never equal to the column index
/// - the pattern is structurally symmetric: (i,j) present iff (j,i) present
#[derive(Clone, Debug)]
pub struct Pattern {
    pub n: usize,
    pub col_ptr: Vec<usize>,
    pub row_idx: Vec<usize>,
}

impl Pattern {
    /// Build a symmetric pattern from an edge list. Each (r, c) pair is
    /// symmetrized, deduplicated, and the diagonal is dropped.
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
        Pattern::from_adjacency(n, &mut adj)
    }

    /// Build directly from mutable per-vertex adjacency lists, sorting and
    /// deduplicating each. Shared by `from_edges` and the .mtx loader.
    pub(crate) fn from_adjacency(n: usize, adj: &mut [Vec<usize>]) -> Pattern {
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

    /// Off-diagonal structural nonzeros (both triangles).
    pub fn nnz(&self) -> usize {
        self.row_idx.len()
    }

    /// Iterate row indices of column `j`.
    pub fn col(&self, j: usize) -> &[usize] {
        &self.row_idx[self.col_ptr[j]..self.col_ptr[j + 1]]
    }
}
