//! DM/BTF-RELABELLED AMD (IDEAS #6, fleet-n) — a detector-first candidate for
//! the KKT block-angular class (unitcommit). The 0036 probes validated a
//! pattern-only detector for chainyoda's published KKT layout
//! [primals ‖ ineq slacks ‖ eq duals ‖ ineq duals] and measured the
//! constraint×primal bipartite graph B highly reducible (largest fine
//! Dulmage–Mendelsohn block 0.0–0.4 of rows). Strict two-phase ordering is
//! dead (0036: the Schur block explodes), and raw BTF emission is dead too
//! (probe_fleet_n_btf: 1e2–1e7× the incumbent — the unmatched-dual mass
//! collapses onto whatever remains). What survives is the RELABELLING form:
//! emit the BTF structure (slacks, then fine DM blocks sinks-first with each
//! block's (primal, matched-dual) pairs adjacent, then unmatched primals,
//! then unmatched duals) as a *relabelling* q of the whole pattern, run AMD
//! on the relabelled pattern, and compose back. AMD's tie-breaking follows
//! label order, so the sweep tracks the BTF block structure while AMD stays
//! free to interleave locally — the interleaving whose absence killed strict
//! two-phase. Measured (probe_fleet_n_btf2): beats the plain AMD anchor on
//! all three unitcommit_200 rows (0.9906–0.9991×) where every other relabel
//! seed is structure-blind.
//!
//! Deterministic: fixed adjacency construction (sorted, deduped), fixed
//! Hopcroft–Karp BFS/DFS iteration order, iterative Tarjan SCC with fixed
//! start order, ascending-id tie-breaks throughout. All stacks explicit (the
//! corpus reaches n ≈ 340k). Returns None when the layout detector finds no
//! primal/dual split — callers pay one O(nnz) scan for the fallback.

use crate::Pattern;

/// Sub-ordering run on the BTF-relabelled pattern.
#[derive(Clone, Copy)]
pub enum DmBtfSub {
    Amd,
    /// AMF with the given dense_alpha (transswitch cell, probe_fleet_n_btf2).
    Amf(f64),
}

/// Emission/sub-ordering options. `topo`: emit fine blocks sources-first
/// (topological) instead of the default sinks-first (SCC pop order).
/// `pair_dual_first`: (dual, primal) inside a pair instead of (primal, dual).
#[derive(Clone, Copy)]
pub struct DmBtfOptions {
    pub topo: bool,
    pub pair_dual_first: bool,
    pub sub: DmBtfSub,
}

/// BTF-relabelled AMD in the unitcommit-winning configuration (sinks-first,
/// (primal, dual) pairs, AMD sub-ordering). Returns `perm[k]` = original
/// index eliminated k-th, or None when the KKT layout detector fails or the
/// sub-ordering errors.
pub fn dm_btf_relabel_amd_order(pattern: &Pattern) -> Option<Vec<i32>> {
    dm_btf_relabel_order(
        pattern,
        &DmBtfOptions { topo: false, pair_dual_first: false, sub: DmBtfSub::Amd },
    )
}

/// General form; see [`DmBtfOptions`].
pub fn dm_btf_relabel_order(pattern: &Pattern, opts: &DmBtfOptions) -> Option<Vec<i32>> {
    let n = pattern.n;
    if n == 0 {
        return None;
    }

    // ── KKT layout detector (validated in 0036) ─────────────────────────
    // Dual block = maximal independent suffix [d0, n) (no dual–dual edges);
    // slack run = contiguous degree-1 vertices just below d0 whose unique
    // neighbour sits inside the dual block; primals = the rest.
    let mut deg = vec![0u32; n];
    let mut nbr1 = vec![u32::MAX; n];
    let mut max_min_endpoint: i64 = -1;
    for j in 0..n {
        for &i in pattern.col(j) {
            if i > j && i < n {
                deg[j] += 1;
                deg[i] += 1;
                nbr1[j] = i as u32;
                nbr1[i] = j as u32;
                max_min_endpoint = max_min_endpoint.max(j as i64);
            }
        }
    }
    let d0 = (max_min_endpoint + 1) as usize;
    let mut a = d0;
    while a > 0 && deg[a - 1] == 1 && (nbr1[a - 1] as usize) >= d0 {
        a -= 1;
    }
    let np = a; // primals 0..np
    let nd = n - d0; // duals d0..n
    if np == 0 || nd == 0 {
        return None;
    }

    // ── Bipartite B: rows = duals, cols = primals ───────────────────────
    let mut radj: Vec<Vec<u32>> = vec![Vec::new(); nd];
    for j in 0..n {
        for &i in pattern.col(j) {
            if i > j && i < n {
                let (u, v) = (j, i);
                if u < np && v >= d0 {
                    radj[v - d0].push(u as u32);
                } else if v < np && u >= d0 {
                    radj[u - d0].push(v as u32);
                }
            }
        }
    }
    for l in radj.iter_mut() {
        l.sort_unstable();
        l.dedup();
    }

    // ── Hopcroft–Karp maximum matching (iterative) ──────────────────────
    const NIL: u32 = u32::MAX;
    let mut mrow = vec![NIL; nd]; // row -> col
    let mut mcol = vec![NIL; np]; // col -> row
    let mut dist = vec![u32::MAX; nd];
    let mut queue: std::collections::VecDeque<u32> = std::collections::VecDeque::new();
    loop {
        queue.clear();
        for r in 0..nd {
            if mrow[r] == NIL {
                dist[r] = 0;
                queue.push_back(r as u32);
            } else {
                dist[r] = u32::MAX;
            }
        }
        let mut found = false;
        while let Some(r) = queue.pop_front() {
            for &c in &radj[r as usize] {
                let r2 = mcol[c as usize];
                if r2 == NIL {
                    found = true;
                } else if dist[r2 as usize] == u32::MAX {
                    dist[r2 as usize] = dist[r as usize] + 1;
                    queue.push_back(r2);
                }
            }
        }
        if !found {
            break;
        }
        let mut aug_total = 0usize;
        let mut it = vec![0u32; nd];
        for r0 in 0..nd {
            if mrow[r0] != NIL {
                continue;
            }
            let mut stack: Vec<u32> = vec![r0 as u32];
            let mut path: Vec<(u32, u32)> = Vec::new();
            it[r0] = 0;
            let mut augmented = false;
            while let Some(&r) = stack.last() {
                let ru = r as usize;
                let mut advanced = false;
                while (it[ru] as usize) < radj[ru].len() {
                    let c = radj[ru][it[ru] as usize];
                    it[ru] += 1;
                    let r2 = mcol[c as usize];
                    if r2 == NIL {
                        path.push((r, c));
                        for &(pr, pc) in path.iter().rev() {
                            mrow[pr as usize] = pc;
                            mcol[pc as usize] = pr;
                        }
                        augmented = true;
                        break;
                    }
                    if dist[r2 as usize] == dist[ru] + 1 && (it[r2 as usize] as usize) == 0 {
                        path.push((r, c));
                        stack.push(r2);
                        advanced = true;
                        break;
                    }
                }
                if augmented {
                    break;
                }
                if !advanced {
                    dist[ru] = u32::MAX;
                    stack.pop();
                    path.pop();
                }
            }
            if augmented {
                aug_total += 1;
            }
        }
        if aug_total == 0 {
            break;
        }
    }

    // ── Tarjan SCC on the matched square part (fine DM blocks) ──────────
    // Digraph on matched rows: r -> match(c) for every c in row r. Pop order
    // is reverse topological on the condensation (all successors of a block
    // pop before it) — the emission below uses pop order as-is ("sinks
    // first"), the variant probe_fleet_n_btf2 measured best.
    let mut idx = vec![u32::MAX; nd];
    let mut low = vec![0u32; nd];
    let mut on = vec![false; nd];
    let mut sccstack: Vec<u32> = Vec::new();
    let mut scc_of = vec![u32::MAX; nd];
    let mut n_scc = 0u32;
    let mut counter = 0u32;
    let mut call: Vec<(u32, u32)> = Vec::new();
    for s in 0..nd {
        if mrow[s] == NIL || idx[s] != u32::MAX {
            continue;
        }
        call.push((s as u32, 0));
        idx[s] = counter;
        low[s] = counter;
        counter += 1;
        on[s] = true;
        sccstack.push(s as u32);
        while let Some(top) = call.last_mut() {
            let r = top.0 as usize;
            if (top.1 as usize) < radj[r].len() {
                let c = radj[r][top.1 as usize];
                top.1 += 1;
                let r2 = mcol[c as usize];
                if r2 == NIL {
                    continue;
                }
                let r2u = r2 as usize;
                if idx[r2u] == u32::MAX {
                    idx[r2u] = counter;
                    low[r2u] = counter;
                    counter += 1;
                    on[r2u] = true;
                    sccstack.push(r2);
                    call.push((r2, 0));
                } else if on[r2u] && idx[r2u] < low[r] {
                    low[r] = idx[r2u];
                }
            } else {
                call.pop();
                if let Some(&(pr, _)) = call.last() {
                    let pu = pr as usize;
                    if low[r] < low[pu] {
                        low[pu] = low[r];
                    }
                }
                if low[r] == idx[r] {
                    while let Some(&x) = sccstack.last() {
                        sccstack.pop();
                        on[x as usize] = false;
                        scc_of[x as usize] = n_scc;
                        if x as usize == r {
                            break;
                        }
                    }
                    n_scc += 1;
                }
            }
        }
    }

    // ── BTF emission as a relabelling q (new label -> original vertex) ──
    // Slacks, then fine blocks in SCC pop order with (primal, dual) pairs
    // (rows grouped per block ascending), then unmatched primals, then
    // unmatched duals. Grouping by one counting pass keeps this O(n).
    let mut block_start = vec![0usize; n_scc as usize + 1];
    for r in 0..nd {
        if scc_of[r] != u32::MAX {
            block_start[scc_of[r] as usize + 1] += 1;
        }
    }
    for b in 0..n_scc as usize {
        block_start[b + 1] += block_start[b];
    }
    let matched_total = block_start[n_scc as usize];
    let mut block_rows = vec![0u32; matched_total];
    {
        let mut pos = block_start[..n_scc as usize].to_vec();
        // Rows ascending within each block: r ascending fills each block's
        // slice in ascending order.
        for r in 0..nd {
            let s = scc_of[r];
            if s != u32::MAX {
                block_rows[pos[s as usize]] = r as u32;
                pos[s as usize] += 1;
            }
        }
    }

    let mut q: Vec<usize> = Vec::with_capacity(n);
    q.extend(np..d0); // slacks
    let block_ids: Vec<usize> = if opts.topo {
        (0..n_scc as usize).rev().collect()
    } else {
        (0..n_scc as usize).collect()
    };
    for b in block_ids {
        for &r in &block_rows[block_start[b]..block_start[b + 1]] {
            if opts.pair_dual_first {
                q.push(d0 + r as usize); // matched dual
                q.push(mrow[r as usize] as usize); // primal
            } else {
                q.push(mrow[r as usize] as usize); // primal
                q.push(d0 + r as usize); // its matched dual
            }
        }
    }
    for c in 0..np {
        if mcol[c] == NIL {
            q.push(c);
        }
    }
    for r in 0..nd {
        if mrow[r] == NIL {
            q.push(d0 + r);
        }
    }
    if q.len() != n {
        // Defensive: the emission above covers every vertex exactly once by
        // construction; bail out rather than emit a non-bijection.
        return None;
    }

    // ── AMD on the relabelled pattern, composed back through q ──────────
    let spat = super::ScoringPattern {
        n,
        col_ptr: pattern.col_ptr.clone(),
        row_idx: pattern.row_idx.clone(),
    };
    let bpat = feral::ordering::amd::permute_pattern(&spat, &q);
    let bcp: Vec<i32> = bpat.col_ptr.iter().map(|&x| x as i32).collect();
    let bri: Vec<i32> = bpat.row_idx.iter().map(|&x| x as i32).collect();
    let bc = feral_ordering_core::CscPattern::new(n, &bcp, &bri)?;
    let pb = match opts.sub {
        DmBtfSub::Amd => feral_amd::amd_order(&bc).ok()?,
        DmBtfSub::Amf(alpha) => {
            let amf_opts = feral_amf::AmfOptions { dense_alpha: alpha, ..Default::default() };
            feral_amf::amf_order_opts(&bc, &amf_opts).ok()?.0
        }
    };
    Some(pb.iter().map(|&x| q[x as usize] as i32).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bijection(perm: &[i32], n: usize) {
        assert_eq!(perm.len(), n, "permutation length");
        let mut seen = vec![false; n];
        for &v in perm {
            let v = v as usize;
            assert!(v < n && !seen[v], "not a bijection of 0..{n}");
            seen[v] = true;
        }
    }

    #[test]
    fn none_on_empty_and_degenerate() {
        assert!(dm_btf_relabel_amd_order(&Pattern::from_edges(0, &[])).is_none());
        // No edges at all -> no dual suffix -> None.
        assert!(dm_btf_relabel_amd_order(&Pattern::from_edges(4, &[])).is_none());
    }

    #[test]
    fn kkt_shaped_pattern_gives_bijection() {
        // [primals 0..3 | slacks 3..5 | duals 5..9]: primal-primal edges,
        // dual-primal edges, slack->own-dual edges, no dual-dual edges.
        let edges = vec![
            (0, 1),
            (1, 2),
            (0, 5),
            (1, 5),
            (1, 6),
            (2, 6),
            (0, 7),
            (3, 7), // slack 3 -> dual 7
            (4, 8), // slack 4 -> dual 8
            (2, 8),
        ];
        let pat = Pattern::from_edges(9, &edges);
        let perm = dm_btf_relabel_amd_order(&pat).expect("detector should fire");
        assert_bijection(&perm, 9);
    }

    #[test]
    fn deterministic() {
        let edges = vec![
            (0, 1),
            (1, 2),
            (0, 5),
            (1, 5),
            (1, 6),
            (2, 6),
            (0, 7),
            (3, 7),
            (4, 8),
            (2, 8),
        ];
        let pat = Pattern::from_edges(9, &edges);
        let a = dm_btf_relabel_amd_order(&pat);
        let b = dm_btf_relabel_amd_order(&pat);
        assert_eq!(a, b);
    }
}
