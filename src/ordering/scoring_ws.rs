//! Reusable scoring/permutation workspace — the allocation-elimination layer.
//!
//! `order()` scores every candidate with `flops_of` (permute_pattern → etree →
//! column_counts_gnp) and, for the relabelled-AMD multi-start and hill-climb
//! phases, additionally permutes the pattern to build the relabelled copy fed
//! to `feral_amd::amd_order`. Both paths, as written against vendor `feral`,
//! allocate a fresh set of `Vec`s EVERY call — roughly 15 heap allocations per
//! `flops_of` call, several O(n) and two O(nnz). A single `order()` call
//! scores the AMD anchor, ~30 portfolio candidates, up to 24 relabel
//! restarts, and up to 24 hill-climb steps — up to ~80 calls on the matrices
//! that receive every phase — so this churns megabytes of allocator traffic
//! per matrix, repeatedly, at a size that never changes within one `order()`
//! call (n and nnz are fixed for the whole call).
//!
//! [`ScoreWorkspace`] fixes this by allocating every buffer exactly ONCE, at
//! `order()`'s (n, nnz), and reusing it across every subsequent call within
//! that `order()` invocation (never across matrices — `order()` runs in a
//! fresh child process per matrix, so there is no cross-matrix state to
//! reuse, and reusing across matrices would anyway violate the determinism
//! gate's "pure function of the current pattern" requirement).
//!
//! ## fleet-s (0041): u32-indexed fast path, VALUE-identical (not byte-port)
//!
//! The original workspace was a buffer-reusing byte-port of the vendor
//! functions. This revision keeps the SAME algorithm (permute → Liu etree via
//! path-compressed union-find → postorder → first-descendants → Gilbert–
//! Ng–Peyton column counts) but narrows every index buffer to `u32`/`i32`
//! (n and nnz are far below 2^32 on this corpus; `new` asserts it), replaces
//! the `Vec<Vec<usize>>` child lists with a flat counting-sort CSR, and DROPS
//! the per-column sort of the permuted pattern. The contract is now
//! VALUE-equality of the returned `u64`, not intermediate-buffer equality:
//!
//! - The elimination tree of a pattern is unique, and the union-find
//!   construction reaches the same `parent[]` regardless of within-column
//!   visit order (each link `parent[r] = j` is decided by root identity, not
//!   by arrival order).
//! - The postorder built here visits each node's children in ascending child
//!   index — exactly the order the vendor's push-in-ascending-`j` `children()`
//!   lists produce — so `post` is identical to the vendor's.
//! - The GNP pass treats each off-diagonal partner `u` of row `i`
//!   independently (`maxfirst[u]`/`prevleaf[u]` are per-partner state and a
//!   partner appears at most once per row), so within-row order cannot change
//!   any `delta`, hence the sort is pure cost with zero effect on the result.
//!
//! The `matches_vendor_flops_of_synthetic` test below and the full-corpus
//! `probe_ws_equivalence`-style checks assert `u64`-equality against the
//! vendor-backed `flops_of` path, which is the only observable this
//! workspace has.

use super::ScoringPattern;

pub(crate) struct ScoreWorkspace {
    n: usize,

    // -- permute scratch (u32-indexed; unsorted columns, see module doc) --
    inv_perm: Vec<u32>,
    pcol_ptr: Vec<u32>,
    prow_idx: Vec<u32>,
    offsets: Vec<u32>,

    // -- etree scratch --
    /// `-1` = root, else the parent index.
    parent: Vec<i32>,
    uf_ancestor: Vec<u32>,

    // -- child CSR + postorder scratch --
    child_ptr: Vec<u32>,  // n + 1
    child_idx: Vec<u32>,  // n (each non-root appears once)
    next_child: Vec<u32>, // per-node cursor into its child slice
    stack: Vec<u32>,
    post: Vec<u32>,

    // -- first_descendants scratch --
    post_of: Vec<u32>,
    first: Vec<u32>,

    // -- column_counts_gnp scratch --
    delta: Vec<i32>,
    maxfirst: Vec<i32>,
    prevleaf: Vec<i32>,
    dsu_ancestor: Vec<u32>,
}

impl ScoreWorkspace {
    /// Allocate every buffer once, sized for this `order()` call's fixed `(n,
    /// nnz)` — `nnz` is the symmetric pattern's total entry count, which
    /// permutation preserves exactly (a relabeling is a bijection on edges),
    /// so it never changes across the calls this workspace serves.
    pub(crate) fn new(n: usize, nnz: usize) -> Self {
        // The u32 narrowing is sound for every matrix this challenge can
        // pose (and `Pattern` is i32-indexed upstream anyway); assert rather
        // than silently truncate.
        assert!(n < u32::MAX as usize && nnz < u32::MAX as usize);
        ScoreWorkspace {
            n,
            inv_perm: vec![0; n],
            pcol_ptr: vec![0; n + 1],
            prow_idx: vec![0; nnz],
            offsets: vec![0; n],
            parent: vec![-1; n],
            uf_ancestor: vec![0; n],
            child_ptr: vec![0; n + 1],
            child_idx: vec![0; n],
            next_child: vec![0; n],
            stack: Vec::with_capacity(n),
            post: Vec::with_capacity(n),
            post_of: vec![0; n],
            first: vec![0; n],
            delta: vec![0i32; n],
            maxfirst: vec![-1i32; n],
            prevleaf: vec![-1i32; n],
            dsu_ancestor: vec![0; n],
        }
    }

    /// Permute the pattern into `self.pcol_ptr` / `self.prow_idx` (both fully
    /// overwritten every call — the bucket-fill writes each of the `nnz`
    /// positions exactly once). Columns are NOT sorted (see the module doc's
    /// value-equality argument; nothing downstream of this workspace reads
    /// them).
    fn permute_into(&mut self, pat: &ScoringPattern, perm: &[usize]) {
        let n = self.n;

        // `inv_perm[old] = new`. Every index 0..n is written exactly once
        // (perm is a bijection), so this fully overwrites the buffer.
        for (new, &old) in perm.iter().enumerate() {
            self.inv_perm[old] = new as u32;
        }

        // `pcol_ptr[0]` is ALWAYS 0 (set once in `new`, never written again).
        // `pcol_ptr[1..=n]`: `new_j` ranges over all of 0..n exactly once
        // (inv_perm bijection) — full overwrite, then prefix-sum in place.
        for old_j in 0..n {
            let new_j = self.inv_perm[old_j] as usize;
            let nnz_j = pat.col_ptr[old_j + 1] - pat.col_ptr[old_j];
            self.pcol_ptr[new_j + 1] = nnz_j as u32;
        }
        for j in 0..n {
            self.pcol_ptr[j + 1] += self.pcol_ptr[j];
        }

        self.offsets[..n].copy_from_slice(&self.pcol_ptr[..n]);

        for old_j in 0..n {
            let new_j = self.inv_perm[old_j] as usize;
            let start = pat.col_ptr[old_j];
            let end = pat.col_ptr[old_j + 1];
            for k in start..end {
                let new_i = self.inv_perm[pat.row_idx[k]];
                let pos = self.offsets[new_j];
                self.prow_idx[pos as usize] = new_i;
                self.offsets[new_j] += 1;
            }
        }
    }

    /// Same permutation, ALSO written into caller-owned i32 buffers with
    /// SORTED columns (the vendor `permute_pattern` invariant), for a caller
    /// that builds a relabelled pattern to feed `feral_amd::amd_order`.
    ///
    /// NOT wired into this build's relabel-multi-start/hill-climb loops (they
    /// still use vendor `permute_pattern` directly) — kept for a future
    /// pass, and exercised directly by `permute_i32_matches_vendor` below so
    /// it stays correct if wired in later.
    #[allow(dead_code)]
    pub(crate) fn permute_i32_into(
        &mut self,
        pat: &ScoringPattern,
        perm: &[usize],
        col_ptr_i32: &mut Vec<i32>,
        row_idx_i32: &mut Vec<i32>,
    ) {
        self.permute_into(pat, perm);
        // Restore the vendor's sorted-column invariant on the i32 copy (the
        // internal buffers stay unsorted; only this export needs it).
        col_ptr_i32.clear();
        col_ptr_i32.extend(self.pcol_ptr.iter().map(|&x| x as i32));
        row_idx_i32.clear();
        row_idx_i32.extend(self.prow_idx.iter().map(|&x| x as i32));
        for j in 0..self.n {
            let start = self.pcol_ptr[j] as usize;
            let end = self.pcol_ptr[j + 1] as usize;
            row_idx_i32[start..end].sort_unstable();
        }
    }

    /// Liu's elimination-tree construction (path-compressed union-find) on
    /// the permuted pattern currently held in `self.pcol_ptr`/`self.prow_idx`.
    /// The etree of a pattern is unique; see the module doc for why the
    /// unsorted columns cannot change `parent`.
    fn build_etree(&mut self) {
        let n = self.n;

        // MUST reset: a root in this call (parent[j] == -1) may have held a
        // parent from a previous call, and roots are exactly the positions
        // the loop below never writes.
        self.parent.fill(-1);

        for j in 0..n {
            self.uf_ancestor[j] = j as u32;
            let start = self.pcol_ptr[j] as usize;
            let end = self.pcol_ptr[j + 1] as usize;
            for k in start..end {
                let i = self.prow_idx[k] as usize;
                if i >= j {
                    continue;
                }
                let mut r = i;
                while self.uf_ancestor[r] as usize != r {
                    r = self.uf_ancestor[r] as usize;
                }
                let mut node = i;
                while node != r {
                    let next = self.uf_ancestor[node] as usize;
                    self.uf_ancestor[node] = r as u32;
                    node = next;
                }
                if r != j {
                    self.parent[r] = j as i32;
                    self.uf_ancestor[r] = j as u32;
                }
            }
        }
    }

    /// Child CSR (counting sort, ascending child index — the same visit order
    /// as the vendor's push-based `children()` lists), postorder, and first
    /// descendants.
    fn build_postorder_and_first(&mut self) {
        let n = self.n;

        // Counting sort of children by parent. `child_ptr` is rebuilt from
        // zero every call (full overwrite of 0..=n).
        self.child_ptr[..=n].fill(0);
        for j in 0..n {
            let p = self.parent[j];
            if p >= 0 {
                self.child_ptr[p as usize + 1] += 1;
            }
        }
        for j in 0..n {
            self.child_ptr[j + 1] += self.child_ptr[j];
        }
        // Fill in ascending j so each parent's slice is ascending — matching
        // the vendor's `children[p].push(j)` order. `next_child` doubles as
        // the fill cursor here, then is reset for the DFS below.
        self.next_child[..n].copy_from_slice(&self.child_ptr[..n]);
        for j in 0..n {
            let p = self.parent[j];
            if p >= 0 {
                let pos = self.next_child[p as usize];
                self.child_idx[pos as usize] = j as u32;
                self.next_child[p as usize] = pos + 1;
            }
        }

        // Iterative DFS postorder over roots in ascending order. `next_child`
        // now counts how many children of a node have been pushed.
        self.next_child[..n].fill(0);
        self.post.clear();
        self.stack.clear();
        for root in 0..n {
            if self.parent[root] >= 0 {
                continue; // not a root
            }
            self.stack.push(root as u32);
            while let Some(&node) = self.stack.last() {
                let node = node as usize;
                let k = self.next_child[node];
                let kids = self.child_ptr[node + 1] - self.child_ptr[node];
                if k < kids {
                    self.next_child[node] = k + 1;
                    self.stack
                        .push(self.child_idx[(self.child_ptr[node] + k) as usize]);
                } else {
                    self.post.push(node as u32);
                    self.stack.pop();
                }
            }
        }

        // `first_descendants(&post)`: fully overwritten for every index
        // (post is a permutation of 0..n).
        for (pnum, &node) in self.post.iter().enumerate() {
            self.post_of[node as usize] = pnum as u32;
        }
        self.first.copy_from_slice(&self.post_of);
        for &node in &self.post {
            let node = node as usize;
            let p = self.parent[node];
            if p >= 0 {
                let p = p as usize;
                if self.first[node] < self.first[p] {
                    self.first[p] = self.first[node];
                }
            }
        }
    }

    /// Gilbert–Ng–Peyton column counts, folded directly into `Σ_j c_j²`
    /// (nothing else ever reads the per-column counts). Consumes `self.post`
    /// / `self.first` / the child CSR plus the permuted pattern.
    fn column_counts_gnp_flops(&mut self) -> u64 {
        let n = self.n;

        // `delta`: fully overwritten for every index (leaf test via the child
        // CSR — a node is a leaf iff its child slice is empty, exactly the
        // vendor's `children[i].is_empty()`).
        for i in 0..n {
            self.delta[i] = i32::from(self.child_ptr[i + 1] == self.child_ptr[i]);
        }
        // MUST reset: read (as "no leaf seen yet" sentinels) before
        // necessarily being written this call.
        self.maxfirst.fill(-1);
        self.prevleaf.fill(-1);
        // MUST reset: DSU identity init.
        for i in 0..n {
            self.dsu_ancestor[i] = i as u32;
        }

        for idx in 0..n {
            let i = self.post[idx] as usize;
            let pi = self.parent[i];
            if pi >= 0 {
                self.delta[pi as usize] -= 1;
            }
            let fi = self.first[i] as i32;
            let row_start = self.pcol_ptr[i] as usize;
            let row_end = self.pcol_ptr[i + 1] as usize;
            for k in row_start..row_end {
                let partner = self.prow_idx[k] as usize;
                if partner <= i {
                    continue;
                }
                if fi > self.maxfirst[partner] {
                    self.delta[i] += 1;
                    let pl = self.prevleaf[partner];
                    if pl != -1 {
                        let mut q = pl as usize;
                        while self.dsu_ancestor[q] as usize != q {
                            q = self.dsu_ancestor[q] as usize;
                        }
                        let root = q;
                        let mut cur = pl as usize;
                        while cur != root {
                            let next = self.dsu_ancestor[cur] as usize;
                            self.dsu_ancestor[cur] = root as u32;
                            cur = next;
                        }
                        self.delta[root] -= 1;
                    }
                    self.prevleaf[partner] = i as i32;
                    self.maxfirst[partner] = fi;
                }
            }
            if pi >= 0 {
                self.dsu_ancestor[i] = pi as u32;
            }
        }

        for idx in 0..n {
            let i = self.post[idx] as usize;
            let p = self.parent[i];
            if p >= 0 {
                let di = self.delta[i];
                self.delta[p as usize] += di;
            }
        }

        let mut flops: u64 = 0;
        for &d in &self.delta {
            let c = d as u64;
            flops += c * c;
        }
        flops
    }

    /// `Σ_j c_j²` for `perm` on `pat` — the workspace-reusing, u32-indexed
    /// equivalent of the vendor-backed `flops_of` in `mod.rs`. Same
    /// algorithm, same exact `u64` result (asserted by the tests below and
    /// the corpus-wide equivalence probes), a fraction of the memory
    /// traffic.
    /// Valid after flops(): exact number of entries in the symbolic factor.
    pub(crate) fn nnz_l(&self) -> u64 {
        self.delta.iter().map(|&c| c as u64).sum()
    }

    pub(crate) fn probe_counts(&self) -> &[i32] { &self.delta }
    pub(crate) fn probe_parent(&self) -> &[i32] { &self.parent }
    pub(crate) fn probe_post(&self) -> &[u32] { &self.post }

    pub(crate) fn flops(&mut self, pat: &ScoringPattern, perm: &[usize]) -> u64 {
        self.permute_into(pat, perm);
        self.build_etree();
        self.build_postorder_and_first();
        self.column_counts_gnp_flops()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pattern;

    #[test]
    fn fill_free_certificate_exhaustive() {
        let n = 5;
        let pairs: Vec<_> = (0..n).flat_map(|i| (i+1..n).map(move |j| (i,j))).collect();
        let mut certificates = 0;
        for mask in 0..(1usize << pairs.len()) {
            let edges: Vec<_> = pairs.iter().enumerate().filter(|(bit, _)| mask & (1 << bit) != 0).map(|(_, &e)| e).collect();
            let mut adjacency = vec![vec![false; n]; n];
            for &(i,j) in &edges { adjacency[i][j] = true; adjacency[j][i] = true; }
            let mut triangles = 0;
            for i in 0..n { for j in i+1..n { for k in j+1..n {
                triangles += usize::from(adjacency[i][j] && adjacency[j][k] && adjacency[i][k]);
            } } }
            let lower_bound = (n + 3*edges.len() + 2*triangles) as u64;
            let pat = Pattern::from_edges(n, &edges);
            let sp = scoring_pattern(&pat);
            let mut ws = ScoreWorkspace::new(n, pat.nnz());
            let mut perm: Vec<_> = (0..n).collect();
            loop {
                let exact = super::super::flops_of(&sp, &perm);
                assert!(exact >= lower_bound);
                assert_eq!(ws.flops(&sp, &perm), exact);
                if ws.nnz_l() == (n + edges.len()) as u64 {
                    assert_eq!(exact, lower_bound);
                    certificates += 1;
                }
                let Some(i) = (0..n-1).rev().find(|&i| perm[i] < perm[i+1]) else { break };
                let j = (i+1..n).rev().find(|&j| perm[j] > perm[i]).unwrap();
                perm.swap(i,j); perm[i+1..].reverse();
            }
        }
        assert!(certificates > 0);
        println!("FILL_FREE graphs=1024 permutations=122880 certificates={certificates}");
    }

    #[test]
    fn workspace_corpus_equivalence() {
        let mut cases = 0;
        let mut checks = 0;
        let mut oracle_ns = 0;
        let mut workspace_ns = 0;
        for (name, pat) in crate::corpus::corpus() {
            let sp = scoring_pattern(&pat);
            let mut ws = ScoreWorkspace::new(pat.n, pat.nnz());
            for ticket in 0..4 {
                let perm: Vec<usize> = match ticket {
                    0 => (0..pat.n).collect(),
                    1 => (0..pat.n).rev().collect(),
                    _ => super::super::relabel(pat.n, ticket),
                };
                let (expected, actual);
                if ticket % 2 == 0 {
                    let t = std::time::Instant::now();
                    expected = super::super::flops_of(&sp, &perm);
                    oracle_ns += t.elapsed().as_nanos();
                    let t = std::time::Instant::now();
                    actual = ws.flops(&sp, &perm);
                    workspace_ns += t.elapsed().as_nanos();
                } else {
                    let t = std::time::Instant::now();
                    actual = ws.flops(&sp, &perm);
                    workspace_ns += t.elapsed().as_nanos();
                    let t = std::time::Instant::now();
                    expected = super::super::flops_of(&sp, &perm);
                    oracle_ns += t.elapsed().as_nanos();
                }
                assert_eq!(actual, expected, "{name}, ticket {ticket}");
                checks += 1;
            }
            cases += 1;
        }
        assert_eq!(cases, 300);
        println!("WORKSPACE cases={cases} checks={checks} oracle_ns={oracle_ns} workspace_ns={workspace_ns}");
    }

    fn scoring_pattern(pattern: &Pattern) -> ScoringPattern {
        ScoringPattern {
            n: pattern.n,
            col_ptr: pattern.col_ptr.clone(),
            row_idx: pattern.row_idx.clone(),
        }
    }

    /// The workspace path must match `super::flops_of` (the vendor-backed
    /// path) on a battery of small synthetic patterns and permutations,
    /// including repeated calls that exercise buffer reuse across DIFFERENT
    /// permutations (catching any missing reset).
    #[test]
    fn matches_vendor_flops_of_synthetic() {
        let cases: Vec<Pattern> = vec![
            Pattern::from_edges(1, &[]),
            Pattern::from_edges(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]), // chain
            Pattern::from_edges(5, &[(0, 1), (0, 2), (0, 3), (0, 4)]), // star
            Pattern::from_edges(6, &[(0, 1), (0, 2), (1, 2), (3, 4), (4, 5)]), // block+chain
            {
                let n = 20;
                let mut edges = Vec::new();
                for v in 0..n - 1 {
                    edges.push((v, v + 1));
                }
                for v in 0..n - 5 {
                    edges.push((v, v + 5));
                }
                Pattern::from_edges(n, &edges)
            },
            {
                // Larger pseudo-random case: enough structure for nontrivial
                // etrees, exercised under many permutations below.
                let n = 257;
                let mut edges = Vec::new();
                let mut s = 0x9E3779B97F4A7C15u64;
                let mut rnd = || {
                    s ^= s << 13;
                    s ^= s >> 7;
                    s ^= s << 17;
                    s
                };
                for v in 0..n - 1 {
                    edges.push((v, v + 1));
                }
                for _ in 0..3 * n {
                    let a = (rnd() % n as u64) as usize;
                    let b = (rnd() % n as u64) as usize;
                    if a != b {
                        edges.push((a.min(b), a.max(b)));
                    }
                }
                Pattern::from_edges(n, &edges)
            },
        ];

        for pat in &cases {
            let n = pat.n;
            let sp = scoring_pattern(pat);
            let mut ws = ScoreWorkspace::new(n, pat.nnz());
            // A handful of permutations per pattern, including identity,
            // reversed, and pseudo-random shuffles, to exercise different
            // fill patterns through the same reused workspace.
            let mut perms: Vec<Vec<usize>> = vec![(0..n).collect(), (0..n).rev().collect()];
            if n > 3 {
                let mut p = (0..n).collect::<Vec<_>>();
                p.swap(0, n - 1);
                p.swap(1, n - 2);
                perms.push(p);
                let mut s = 0xDEADBEEFCAFEF00Du64;
                for _ in 0..4 {
                    let mut p: Vec<usize> = (0..n).collect();
                    for i in (1..n).rev() {
                        s ^= s << 13;
                        s ^= s >> 7;
                        s ^= s << 17;
                        p.swap(i, (s % (i as u64 + 1)) as usize);
                    }
                    perms.push(p);
                }
            }
            for perm in &perms {
                let expected = super::super::flops_of(&sp, perm);
                let got = ws.flops(&sp, perm);
                assert_eq!(
                    got, expected,
                    "workspace flops diverged from vendor flops_of on n={n} perm={perm:?}"
                );
            }
        }
    }

    /// `permute_i32_into` must match vendor `permute_pattern` cast to i32,
    /// including across repeated calls into the SAME reused output buffers.
    #[test]
    fn permute_i32_matches_vendor() {
        let n = 15;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 4 {
            edges.push((v, v + 4));
        }
        let pat = Pattern::from_edges(n, &edges);
        let sp = scoring_pattern(&pat);
        let mut ws = ScoreWorkspace::new(n, pat.nnz());
        let mut col_ptr_i32 = Vec::new();
        let mut row_idx_i32 = Vec::new();

        let perms: Vec<Vec<usize>> = vec![
            (0..n).collect(),
            (0..n).rev().collect(),
            {
                let mut p: Vec<usize> = (0..n).collect();
                p.swap(0, 7);
                p.swap(3, 12);
                p
            },
        ];
        for perm in &perms {
            ws.permute_i32_into(&sp, perm, &mut col_ptr_i32, &mut row_idx_i32);
            let expected = feral::ordering::amd::permute_pattern(&sp, perm);
            let expected_cp: Vec<i32> = expected.col_ptr.iter().map(|&x| x as i32).collect();
            let expected_ri: Vec<i32> = expected.row_idx.iter().map(|&x| x as i32).collect();
            assert_eq!(col_ptr_i32, expected_cp, "col_ptr mismatch for perm {perm:?}");
            assert_eq!(row_idx_i32, expected_ri, "row_idx mismatch for perm {perm:?}");
        }
    }
}
